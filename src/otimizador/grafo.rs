use crate::mepa::code::MepaCode;
use crate::mepa::instruction::Instruction;
use crate::mepa::label::Label;
use petgraph::dot::{Config, Dot};
use petgraph::graph::NodeIndex;
use petgraph::visit::EdgeRef;
use petgraph::{visit::Dfs, Graph};
use std::collections::HashMap;
use std::collections::HashSet;
use std::fs::{self, File};
use std::io::{self, Write};
use std::path::PathBuf;
use std::usize;

pub fn map_code_to_graph(code: &MepaCode) -> Graph<(usize, usize), ()> {
    let mut grafo = Graph::new();

    let mut lideres: Vec<usize> = code
        .iter()
        .enumerate()
        .flat_map(|(i, (_, code))| {
            if i == 0 {
                // The first index is always a leader
                Some(vec![0])
            } else {
                match code {
                    Instruction::DSVF(label)
                    | Instruction::DSVS(label)
                    | Instruction::CHPR(label) => {
                        if let Label::Literal(addr) = label {
                            Some(vec![i + 1, *addr]) // Add the next instruction and jump address
                        } else {
                            panic!("Should never reach here");
                        }
                    }
                    Instruction::RTPR(_, _) => Some(vec![i + 1]), // Add the next instruction
                    _ => None,
                }
            }
        })
        .flatten()
        .collect::<HashSet<_>>()
        .into_iter()
        .collect();

    lideres.sort_unstable();

    let vertices: Vec<NodeIndex> = lideres
        .iter()
        .zip(lideres.iter().skip(1).chain(std::iter::once(&code.len())))
        .map(|(&start, &end)| grafo.add_node((start, end)))
        .collect();

    let mut arestas: Vec<(NodeIndex, NodeIndex)> = Vec::with_capacity(code.len());
    for v in &vertices {
        let linhas = grafo.node_weight(*v).unwrap();
        for linha in linhas.0..linhas.1 {
            match &code[linha].1 {
                Instruction::DSVS(label) => {
                    if let Label::Literal(jmp_addr) = label {
                        let target_block = vertices
                            .iter()
                            .find(|&&output_index| {
                                let &(start, end) = grafo.node_weight(output_index).unwrap();
                                start <= *jmp_addr && *jmp_addr < end
                            })
                            .expect("Address not found in groupings")
                            .clone();
                        arestas.push((*v, target_block));
                    } else {
                        panic!("não removeu labels simbolicos")
                    }
                }
                Instruction::DSVF(label) => {
                    if let Label::Literal(jmp_addr) = label {
                        let jmp_block = vertices
                            .iter()
                            .find(|&&output_index| {
                                let &(start, end) = grafo.node_weight(output_index).unwrap();
                                start <= *jmp_addr && *jmp_addr < end
                            })
                            .expect("Address not found in groupings")
                            .clone();
                        let next_block = vertices
                            .iter()
                            .find(|&&output_index| {
                                let &(start, end) = grafo.node_weight(output_index).unwrap();
                                start <= linhas.1 && linhas.1 < end
                            })
                            .expect("Address not found in groupings")
                            .clone();
                        arestas.push((*v, jmp_block));
                        arestas.push((*v, next_block));
                    } else {
                        panic!("não removeu labels simbolicos")
                    }
                }
                Instruction::CHPR(label) => {
                    if let Label::Literal(jmp_addr) = label {
                        let procedure_block = vertices
                            .iter()
                            .find(|&&output_index| {
                                let &(start, end) = grafo.node_weight(output_index).unwrap();
                                start <= *jmp_addr && *jmp_addr < end
                            })
                            .expect("Address not found in groupings")
                            .clone();
                        let return_block = code
                            .iter()
                            .enumerate()
                            .skip(*jmp_addr)
                            .find_map(|(ret_addr, (_, inst))| {
                                if let Instruction::RTPR(_, _) = inst {
                                    Some(
                                        vertices
                                            .iter()
                                            .find(|&&output_index| {
                                                let &(start, end) =
                                                    grafo.node_weight(output_index).unwrap();
                                                start <= ret_addr && ret_addr < end
                                            })
                                            .expect("Address not found in groupings")
                                            .clone(),
                                    )
                                } else {
                                    None
                                }
                            })
                            .unwrap();
                        let next_block = vertices
                            .iter()
                            .find(|&&output_index| {
                                let &(start, end) = grafo.node_weight(output_index).unwrap();
                                start <= linhas.1 && linhas.1 < end
                            })
                            .expect("Address not found in groupings")
                            .clone();
                        arestas.push((*v, procedure_block));
                        arestas.push((return_block, next_block));
                    } else {
                        panic!("não removeu labels simbolicos")
                    }
                }
                Instruction::PARA | Instruction::RTPR(_, _) => (),
                _ => {
                    // se é a ultima linha do bloco e não é nenhum pulo, então é um pulo para o próximo
                    if linha + 1 == linhas.1 {
                        let next_block = vertices
                            .iter()
                            .find(|&&output_index| {
                                let &(start, end) = grafo.node_weight(output_index).unwrap();
                                start <= linhas.1 && linhas.1 < end
                            })
                            .expect("Address not found in groupings")
                            .clone();
                        arestas.push((*v, next_block));
                    }
                }
            }
        }
    }
    grafo.extend_with_edges(&arestas);

    grafo
}

pub fn graph_to_file(
    filename: &PathBuf,
    code: &MepaCode,
    graph: &Graph<(usize, usize), ()>,
) -> io::Result<()> {
    if let Some(parent) = filename.parent() {
        fs::create_dir_all(parent)?;
    }

    // Create or open the file
    let file = File::create(filename)?;

    let mut graph_with_code: Graph<String, ()> = Graph::new();
    for &(inicio, fim) in graph.node_weights() {
        let linhas_de_mepa: Vec<String> = (inicio..fim)
            .map(|linha| format!("{}: {}", linha, code[linha].1))
            .collect();
        let s = linhas_de_mepa.join("\n");
        graph_with_code.add_node(s);
    }

    let arestas: Vec<(NodeIndex, NodeIndex)> = graph
        .raw_edges()
        .iter()
        .map(|edge| (edge.source(), edge.target()))
        .collect();

    graph_with_code.extend_with_edges(&arestas);

    let raw_dot = format!(
        "{:?}",
        Dot::with_config(&graph_with_code, &[Config::EdgeNoLabel])
    );

    let processed_dot = raw_dot.replace("\\\"", "").replace("\\\\", "\\");

    // Write the processed string to the file
    write!(&file, "{}", processed_dot)
}

pub fn graph_to_file_with_memory_usage(
    filename: &PathBuf,
    code: &MepaCode,
    graph: &Graph<(usize, usize), ()>,
    memory_graph: &Graph<Vec<usize>, ()>,
) -> io::Result<()> {
    if let Some(parent) = filename.parent() {
        fs::create_dir_all(parent)?;
    }

    // Create or open the file
    let file = File::create(filename)?;

    let mut graph_with_code: Graph<String, ()> = Graph::new();
    for (node_index, &(inicio, fim)) in graph.node_weights().enumerate() {
        let memory_usage = memory_graph
            .node_weight(NodeIndex::new(node_index))
            .unwrap();

        let linhas_de_mepa: Vec<String> = (inicio..fim)
            .map(|linha| {
                format!(
                    "{}: {} - {}",
                    linha,
                    code[linha].1,
                    memory_usage[linha - inicio]
                )
            })
            .collect();
        let s = linhas_de_mepa.join("\n");
        graph_with_code.add_node(s);
    }

    let arestas: Vec<(NodeIndex, NodeIndex)> = graph
        .raw_edges()
        .iter()
        .map(|edge| (edge.source(), edge.target()))
        .collect();

    graph_with_code.extend_with_edges(&arestas);

    let raw_dot = format!(
        "{:?}",
        Dot::with_config(&graph_with_code, &[Config::EdgeNoLabel])
    );

    let processed_dot = raw_dot.replace("\\\"", "").replace("\\\\", "\\");

    // Write the processed string to the file
    write!(&file, "{}", processed_dot)
}

// pub fn raw_graph_to_file<T>(filename: &PathBuf, graph: &Graph<T, ()>) -> io::Result<()>
// where
//     T: Debug,
// {
//     if let Some(parent) = filename.parent() {
//         fs::create_dir_all(parent)?;
//     }

//     // Create or open the file
//     let file = File::create(filename)?;

//     let raw_dot = format!("{:?}", Dot::with_config(&graph, &[Config::EdgeNoLabel]));

//     let processed_dot = raw_dot.replace("\\\"", "").replace("\\\\", "\\");

//     // Write the processed string to the file
//     write!(&file, "{}", processed_dot)
// }

#[derive(Debug, Clone)]
pub struct InstructionAndMetadata {
    pub address: usize,
    pub instruction: Instruction,
    pub memory_usage: Option<usize>,
    pub memory_delta: i32,
    pub liberation_address: Option<usize>,
}

pub struct FuncMetadata {
    pub addr_inicio: usize,
    pub addr_retorno: usize,
    pub acesso: usize, //maximo de memoria acessado externo
    pub args: usize,
    pub usos: Vec<usize>,
}

pub struct CodeGraph {
    pub grafo: Graph<Vec<InstructionAndMetadata>, ()>,
    pub funcoes: Vec<FuncMetadata>,
}

impl CodeGraph {
    pub fn new(mut code: MepaCode) -> CodeGraph {
        // este pré-processamento deve ser feito para que a montagem do grafo funcione normalmente
        code = remover_rotulos_simbolicos(code);

        let mut grafo = CodeGraph {
            grafo: Graph::new(),
            funcoes: Vec::new(),
        };

        let mut lideres: Vec<usize> = code
            .iter()
            .enumerate()
            .flat_map(|(i, (_, code))| {
                if i == 0 {
                    // The first index is always a leader
                    Some(vec![0])
                } else {
                    match code {
                        Instruction::DSVF(label)
                        | Instruction::DSVS(label)
                        | Instruction::CHPR(label) => {
                            if let Label::Literal(addr) = label {
                                Some(vec![i + 1, *addr]) // Add the next instruction and jump address
                            } else {
                                panic!("Should never reach here");
                            }
                        }
                        Instruction::RTPR(_, _) => Some(vec![i + 1]), // Add the next instruction
                        _ => None,
                    }
                }
            })
            .flatten()
            .collect::<HashSet<_>>()
            .into_iter()
            .collect();

        lideres.sort_unstable();

        let vertices: Vec<NodeIndex> = lideres
            .iter()
            .zip(lideres.iter().skip(1).chain(std::iter::once(&code.len())))
            .map(|(&start, &end)| {
                // map the instructions from start to end into a new vec
                let instructions: Vec<InstructionAndMetadata> = (start..end)
                    .map(|addr| InstructionAndMetadata {
                        address: addr,
                        instruction: code[addr].1.clone(),
                        memory_usage: None,
                        memory_delta: 0,
                        liberation_address: None,
                    })
                    .collect();
                grafo.grafo.add_node(instructions)
            })
            .collect();

        let mut arestas: Vec<(NodeIndex, NodeIndex)> = Vec::with_capacity(code.len());

        for block_index in &vertices {
            let linhas = grafo.grafo.node_weight(*block_index).unwrap();

            for linha in linhas {
                match &linha.instruction {
                    Instruction::DSVS(label) => {
                        // adiciona o vertice que contem o endereco label
                        if let Label::Literal(jmp_addr) = label {
                            let target_block = grafo
                                .locate_address(*jmp_addr)
                                .expect("Address not found in groupings");
                            arestas.push((*block_index, target_block));
                        } else {
                            panic!("não removeu labels simbolicos")
                        }
                    }
                    Instruction::DSVF(label) => {
                        // adiciona o vertice que contem o endereco label e o da proxima instrucao
                        if let Label::Literal(jmp_addr) = label {
                            let target_block = grafo
                                .locate_address(*jmp_addr)
                                .expect("Address not found in groupings");
                            arestas.push((*block_index, target_block));
                        } else {
                            panic!("não removeu labels simbolicos")
                        }
                        if let Some(target_block) = grafo.locate_address(linha.address + 1) {
                            arestas.push((*block_index, target_block));
                        }
                    }
                    // Instruction::CHPR(label) => {
                    //     if let Label::Literal(jmp_addr) = label {
                    //         // adiciona o vertice que contem o endereco label
                    //         let target_block = grafo
                    //             .locate_address(*jmp_addr)
                    //             .expect("Address not found in groupings");
                    //         arestas.push((*block_index, target_block));
                    //         let next_block = grafo
                    //             .locate_address(linha.address + 1)
                    //             .expect("Address not found in groupings");
                    //
                    //         // encontra o retorno RTPR correspondente e adiciona uma transicao dele para a proxima instrucao
                    //         let return_addr = code
                    //             .iter()
                    //             .enumerate()
                    //             .skip(*jmp_addr)
                    //             .find_map(|(ret_addr, (_, inst))| {
                    //                 if let Instruction::RTPR(_, _) = inst {
                    //                     println!(
                    //                         "Para o CHPR {} encontrou o RTPR {}",
                    //                         linha.address, ret_addr
                    //                     );
                    //                     Some(ret_addr)
                    //                 } else {
                    //                     None
                    //                 }
                    //             })
                    //             .expect("RTPR not found");
                    //         let return_block = grafo
                    //             .locate_address(return_addr)
                    //             .expect("Address not found in groupings");
                    //         arestas.push((return_block, next_block));
                    //     } else {
                    //         panic!("não removeu labels simbolicos")
                    //     }
                    // }
                    Instruction::PARA | Instruction::RTPR(_, _) => (),
                    _ => {
                        if linha.address == linhas.last().unwrap().address {
                            // se é a ultima linha do bloco e não é nenhum pulo, então é um pulo para o próximo
                            if let Some(target_block) = grafo.locate_address(linha.address + 1) {
                                arestas.push((*block_index, target_block));
                            }
                        }
                    }
                }
            }
        }

        grafo.grafo.extend_with_edges(&arestas);

        //localiza todas as funcoes (inicio e fim)
        grafo.funcoes = {
            let mut addr_inicio = 0;
            grafo
                .instructions_mut()
                .filter_map(|instruction| match instruction.instruction {
                    Instruction::ENPR(_) => {
                        addr_inicio = instruction.address;
                        None
                    }
                    Instruction::RTPR(_, args) => Some(FuncMetadata {
                        addr_inicio,
                        addr_retorno: instruction.address,
                        acesso: 0,
                        args: args as usize,
                        usos: Vec::new(),
                    }),
                    _ => None,
                })
                .collect()
        };

        // mapeia o uso de memoria

        // lista as raizes: inicios de funcao e inicio do programa
        let raizes: Vec<usize> = grafo
            .funcoes
            .iter()
            .map(|func| func.addr_inicio)
            .chain(std::iter::once(0))
            .collect();

        let inconsistent_memory_usage = raizes
            .iter()
            .map(|raiz| !grafo.mapear_memoria_a_partir_de(*raiz, 0))
            .any(|r| r);

        if inconsistent_memory_usage {
            for line in grafo.instructions_mut() {
                line.memory_usage = None;
            }
        } else {
            let alocs: Vec<InstructionAndMetadata> = grafo
                .instructions_mut()
                .filter_map(|i| {
                    if i.memory_delta > 0 {
                        Some(i.clone())
                    } else {
                        None
                    }
                })
                .collect();

            // para cada instrucao com delta positivo, encontrar aquela que retorna o valor para o original
            for aloc in alocs {
                let node_inicio = grafo.locate_address(aloc.address).unwrap();
                let mut dfs = Dfs::new(&grafo.grafo, node_inicio);

                'outer: while let Some(visited) = dfs.next(&grafo.grafo) {
                    let weight = grafo.grafo.node_weight(visited).unwrap();
                    for line in if visited == node_inicio {
                        aloc.address - weight[0].address + 1
                    } else {
                        0
                    }..weight.len()
                    {
                        if weight[line].memory_usage == aloc.memory_usage {
                            grafo
                                .instructions_mut()
                                .find(|i| i.address == aloc.address)
                                .unwrap()
                                .liberation_address = Some(weight[line].address);
                            break 'outer;
                        }
                    }
                }
            }
        }

        // procura usos de funcoes
        let mut updates = Vec::new();
        for line in grafo.instructions_unordered() {
            if let Instruction::CHPR(label) = &line.instruction {
                updates.push((label.unwrap(), line.address));
            }
        }
        for (label, address) in updates {
            for f in &mut grafo.funcoes {
                if f.addr_inicio == label {
                    f.usos.push(address);
                }
            }
        }

        grafo
    }
    // retorna se teve sucesso ou não
    pub fn mapear_memoria_a_partir_de(&mut self, addr: usize, initial_value: usize) -> bool {
        let raiz = self.locate_address(addr).unwrap();

        // primeira instrucao usa 0
        for line in self.grafo.node_weight_mut(raiz).unwrap() {
            if line.address == addr {
                line.memory_usage = Some(initial_value);
                line.memory_delta = 0;
            }
        }

        let mut dfs = Dfs::new(&self.grafo, raiz);

        let mut alocation_stack = Vec::new();
        let mut alocation_map = Vec::new();

        while let Some(visited) = dfs.next(&self.grafo) {
            let neighbors: Vec<NodeIndex> =
                self.grafo.neighbors(visited).map(|n| n.clone()).collect();
            let lines = if visited == raiz {
                let lines = self.grafo.node_weight_mut(visited).unwrap();
                let first_addr = lines[0].address;
                &mut lines[addr - first_addr..]
            } else {
                self.grafo.node_weight_mut(visited).unwrap()
            };
            let mut memory = lines.first().unwrap().memory_usage.unwrap() as i32;
            let mut last_memory = lines.first().unwrap().memory_usage.unwrap() as i32;
            for line in 0..lines.len() {
                memory += match &lines[line].instruction {
                    Instruction::CRCT(_)
                    | Instruction::CRVL(_, _)
                    | Instruction::CREN(_, _)
                    | Instruction::CRVI(_, _)
                    | Instruction::LEIT
                    | Instruction::ENPR(_) => 1,
                    Instruction::ARMZ(_, _)
                    | Instruction::ARMI(_, _)
                    | Instruction::SOMA
                    | Instruction::SUBT
                    | Instruction::MULT
                    | Instruction::DIVI
                    | Instruction::CONJ
                    | Instruction::DISJ
                    | Instruction::CMME
                    | Instruction::CMMA
                    | Instruction::CMIG
                    | Instruction::CMDG
                    | Instruction::CMEG
                    | Instruction::CMAG
                    | Instruction::DSVF(_)
                    | Instruction::IMPR => -1,
                    Instruction::AMEM(n) => *n,
                    Instruction::DMEM(n) => -n,
                    Instruction::RTPR(_, n) => -n - 2,
                    Instruction::CHPR(k) => {
                        //locate the function
                        -(self
                            .funcoes
                            .iter()
                            .find(|f| f.addr_inicio == k.unwrap())
                            .unwrap()
                            .args as i32)
                    }
                    _ => 0,
                };
                let memory_delta = memory - last_memory;
                if memory_delta > 0 {
                    alocation_stack.push((lines[line].address, memory));
                } else if memory_delta < 0 {
                    // for each item on the stack, from last to first
                    while let Some(&top) = alocation_stack.last() {
                        // if it is greater or equal than memory, pop
                        if top.1 >= memory {
                            alocation_stack.pop();
                            alocation_map.push((top.0, lines[line].address));
                        }
                        // else, stop
                        else {
                            break;
                        }
                    }
                }
                lines[line].memory_delta = memory_delta;
                last_memory = memory;
                // assign to the next line, if available
                if line + 1 < lines.len() {
                    lines[line + 1].memory_usage = Some(memory as usize);
                }
            }
            // Propagate to neighbors
            for neighbor_index in neighbors.into_iter() {
                let neighbor = self
                    .grafo
                    .node_weight_mut(neighbor_index)
                    .unwrap()
                    .first_mut()
                    .unwrap();
                if let Some(existing_value) = neighbor.memory_usage {
                    if existing_value != memory as usize {
                        return false;
                    }
                }
                neighbor.memory_usage = Some(memory as usize);
            }
        }

        for (aloc, dealoc) in alocation_map {
            self.instruction_mut(aloc).unwrap().liberation_address = Some(dealoc);
        }
        true
    }
    pub fn locate_address(&self, addr: usize) -> Option<NodeIndex> {
        self.grafo.node_indices().find(|&output_index| {
            if let Some(lines) = self.grafo.node_weight(output_index) {
                lines.iter().any(|line|line.address==addr)
            } else {
                false
            }
        })
    }

    pub fn instruction_mut(&mut self, addr: usize) -> Option<&mut InstructionAndMetadata> {
        self.grafo
            .node_weights_mut()
            .flat_map(|node_instructions| node_instructions.iter_mut())
            .find(|f| f.address == addr)
    }

    pub fn instructions_mut(&mut self) -> impl Iterator<Item = &mut InstructionAndMetadata> {
        // Collect all instructions from all nodes
        let mut instructions: Vec<_> = self
            .grafo
            .node_weights_mut()
            .flat_map(|node_instructions| node_instructions.iter_mut())
            .collect();

        // Sort instructions by address
        instructions.sort_by_key(|instr| instr.address);

        // Return an iterator over mutable references
        instructions.into_iter()
    }

    pub fn instructions_unordered(&self) -> impl Iterator<Item = &InstructionAndMetadata> {
        self.grafo
            .node_weights()
            .flat_map(|node_instructions| node_instructions.iter())
    }

    //falta incluir mudança de contagem de chamadas de funcao
    pub fn remove_instruction(&mut self, addr: usize) {
        self.remove_instruction_controlled(addr, true);
    }

    pub fn remove_instruction_controlled(&mut self, addr: usize, remap: bool) {
        if remap {
            if let Some(memory_usage_before) = self.instruction_mut(addr).unwrap().memory_usage {
                //remapeia instrucoes atingiveis por esta, considerando que ela não vai mais afetar a execucao
                self.mapear_memoria_a_partir_de(addr + 1, memory_usage_before);
            }
        }

        let node = self.locate_address(addr).unwrap();
        let line = {
            let lines = self
                .grafo
                .node_weight_mut(self.locate_address(addr).unwrap())
                .unwrap();
            let i = lines.iter().position(|l| l.address == addr).unwrap();
            //remove a linha
            lines.remove(i)
        };

        //aponta qqr desvio q apontaria para ela para a proxima
        for line in self.instructions_mut() {
            match &mut line.instruction {
                Instruction::DSVS(label) | Instruction::DSVF(label) | Instruction::CHPR(label) => {
                    if label.unwrap() == addr {
                        *label = Label::Literal(addr + 1);
                    }
                }
                _ => {}
            }
        }        

        // transformacoes especificas
        match line.instruction {
            Instruction::CHPR(label) => {
                // remove uso da funcao de funcoes
                if let Some(index) = self
                    .funcoes
                    .iter()
                    .position(|f| f.addr_inicio == label.unwrap())
                {
                    if let Some(f) = self.funcoes.get_mut(index) {
                        if let Some(i) = f.usos.iter().position(|n| *n == addr) {
                            f.usos.swap_remove(i);
                        }
                    }
                }
            }
            Instruction::DSVS(label) => {
                // remove pulo associado
                let target = self.locate_address(label.unwrap()).unwrap();
                if let Some(e) = self
                    .grafo
                    .edges_connecting(node, target)
                    .map(|e| e.id())
                    .next()
                {
                    self.grafo.remove_edge(e);
                }
                // adiciona pulo para proximo endereço que ainda existir
                for i in 1..self.len() {
                    if let Some(target) = self.locate_address(addr + i) {
                        self.grafo.add_edge(node, target, ());
                        break;
                    }
                }
            }
            Instruction::DSVF(label) => {
                // remove pulo associado
                let target = self.locate_address(label.unwrap()).unwrap();
                if let Some(e) = self
                    .grafo
                    .edges_connecting(node, target)
                    .map(|e| e.id())
                    .next()
                {
                    self.grafo.remove_edge(e);
                }
            }
            Instruction::ENPR(_) => {
                // remove funcao de funcoes
                if let Some(i) = self
                    .funcoes
                    .iter()
                    .position(|f| f.addr_inicio == line.address)
                {
                    self.funcoes.swap_remove(i);
                }
            }
            _ => (),
        }

        // Lida com nodes vazios
        let empty_nodes: Vec<NodeIndex> = self
            .grafo
            .node_indices()
            .filter(|&node_index| self.grafo.node_weight(node_index).unwrap().is_empty())
            .collect();

        for empty_node in empty_nodes {
            let neighbors: Vec<NodeIndex> = self.grafo.neighbors(empty_node).collect();
            if neighbors.len() > 1 {
                panic!("Como um node vazio pode apontar para mais de um vizinho?");
            } else {
                // Find the node that the empty node points to
                if let Some(target) = neighbors.into_iter().next() {
                    let edges_to_redirect: Vec<_> = self
                        .grafo
                        .edges_directed(empty_node, petgraph::Direction::Incoming)
                        .map(|edge| edge.source())
                        .collect();

                    for source in edges_to_redirect {
                        self.grafo.add_edge(source, target, ());
                    }
                }
            }

            // Remove the empty node from the graph
            self.grafo.remove_node(empty_node);
        }
    }

    pub fn len(&self) -> usize {
        self.grafo.node_weights().map(|lines| lines.len()).sum()
    }

    pub fn remove_node(&mut self, inicio: NodeIndex) {
        // verifica se a primeira instrucao é ENPR
        let removidos: Vec<usize> = self
            .grafo
            .node_weight(inicio)
            .unwrap()
            .iter()
            .map(|line| line.address)
            .collect();

        for r in removidos {
            self.remove_instruction_controlled(r, false);
        }
    }

    pub fn to_mepa_code(self) -> MepaCode {
        // Collect all instructions from all nodes into a single Vec
        let mut instructions: Vec<InstructionAndMetadata> = self
            .grafo.into_nodes_edges().0.into_iter().flat_map(|node| node.weight.into_iter())
            .collect();
    
        // Sort instructions by address
        instructions.sort_by_key(|instr| instr.address);
    
        // First, collect the positions of the labels in an immutable context
        let label_positions: Vec<(usize, usize)> = instructions.iter()
            .enumerate()
            .filter_map(|(i, instr)| {
                match &instr.instruction {
                    Instruction::DSVS(label) |
                    Instruction::DSVF(label) |
                    Instruction::CHPR(label) => Some((i, label.unwrap())),
                    _ => None,
                }
            })
            .map(|(i, label)| (i, instructions.iter().position(|l| l.address == label).unwrap()))
            .map(|(i, pos)| (i, pos))
            .collect();
    
        // Then, update the labels in a mutable context
        for (i, pos) in label_positions {
            match &mut instructions[i].instruction {
                Instruction::DSVS(label) |
                Instruction::DSVF(label) |
                Instruction::CHPR(label) => {
                    *label = Label::Literal(pos);
                }
                _ => (),
            }
        }
    
        // Return the sorted Vec
        MepaCode::from(instructions.into_iter().map(|line| line.instruction))
    }

    pub fn export_to_file(&self, filename: &PathBuf) -> io::Result<()> {
        if let Some(parent) = filename.parent() {
            fs::create_dir_all(parent)?;
        }

        // Create or open the file
        let file = File::create(filename)?;

        let mut graph_with_code: Graph<String, ()> = Graph::new();
        for instructions in self.grafo.node_weights() {
            let linhas_de_mepa: Vec<String> = instructions
                .iter()
                .map(|linha| {
                    format!(
                        "{}: {}{}",
                        linha.address,
                        linha.instruction,
                        if let Some(memory_usage) = linha.memory_usage {
                            format!(
                                " [m: {}{}]",
                                memory_usage,
                                if let Some(liberation_address) = linha.liberation_address {
                                    format!(" | L: {}", liberation_address)
                                } else {
                                    "".to_string()
                                }
                            )
                        } else {
                            "".to_string()
                        }
                    )
                })
                .collect();
            let s = linhas_de_mepa.join("\n");
            graph_with_code.add_node(s);
        }

        let arestas: Vec<(NodeIndex, NodeIndex)> = self
            .grafo
            .raw_edges()
            .iter()
            .map(|edge| (edge.source(), edge.target()))
            .collect();

        graph_with_code.extend_with_edges(&arestas);

        let raw_dot = format!(
            "{:?}",
            Dot::with_config(&graph_with_code, &[Config::EdgeNoLabel])
        );

        let processed_dot = raw_dot.replace("\\\"", "").replace("\\\\", "\\");

        // Write the processed string to the file
        write!(&file, "{}", processed_dot)
    }
}

pub fn remover_rotulos_simbolicos(mc: MepaCode) -> MepaCode {
    let mut labels = HashMap::new();

    // Localiza todas labels
    mc.0.iter()
        .enumerate()
        .for_each(|(line, (label_current_line, _))| {
            if let Some(Label::Simbolic(s)) = label_current_line {
                labels.insert(s.clone(), line);
            }
        });

    // Transforma instruções usando mapa de labels
    let mut mc = MepaCode(
        mc.0.into_iter()
            .map(|(_, instruction)| {
                (
                    None,
                    match instruction {
                        Instruction::DSVS(label) => {
                            if let Label::Simbolic(s) = label {
                                Instruction::DSVS(Label::Literal(*labels.get(&s).unwrap()))
                            } else {
                                Instruction::DSVS(label)
                            }
                        }
                        Instruction::DSVF(label) => {
                            if let Label::Simbolic(s) = label {
                                Instruction::DSVF(Label::Literal(*labels.get(&s).unwrap()))
                            } else {
                                Instruction::DSVF(label)
                            }
                        }
                        Instruction::CHPR(label) => {
                            if let Label::Simbolic(s) = label {
                                Instruction::CHPR(Label::Literal(*labels.get(&s).unwrap()))
                            } else {
                                Instruction::CHPR(label)
                            }
                        }
                        _ => instruction,
                    },
                )
            })
            .collect(),
    );

    let mut i = 0;
    while i < mc.0.len() {
        if matches!(mc.0[i].1, Instruction::NADA) {
            mc.remove_instruction(i);
        } else {
            i += 1;
        }
    }

    mc
}
