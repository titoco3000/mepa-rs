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

#[derive(Debug, Clone)]
pub struct Allocation{
    pub amount: usize,
    pub liberation_address: usize
}

#[derive(Debug, Clone)]
pub struct InstructionAndMetadata {
    pub address: usize,
    pub instruction: Instruction,
    pub initial_memory_usage: Option<usize>,
    pub allocation: Option<Allocation>
}
#[derive(Debug, Clone)]
pub struct FuncMetadata {
    pub addr_inicio: usize,
    pub addr_retorno: usize,
    pub acesso: usize, //maximo de memoria acessado externo
    pub args: usize,
    pub usos: Vec<usize>,
}

#[derive(Debug, Clone)]
pub struct Variavel{
    pub atribuicoes:Vec<usize>,
    pub usos:Vec<usize>
}

#[derive(Debug, Clone)]
pub struct BlocoDeVariaveis{
    pub addr: usize,
    pub vars: Vec<Variavel>
}
impl BlocoDeVariaveis {
    pub fn new(addr:usize, qtd:usize)->BlocoDeVariaveis{
        BlocoDeVariaveis { addr, vars: vec![Variavel{atribuicoes:Vec::new(), usos:Vec::new()};qtd] }
    }
}

pub struct CodeGraph {
    pub grafo: Graph<Vec<InstructionAndMetadata>, ()>,
    pub funcoes: Vec<FuncMetadata>,
    pub memoria_consistente: bool,
    pub variaveis: Vec<BlocoDeVariaveis>
}

impl CodeGraph {
    pub fn new(mut code: MepaCode) -> CodeGraph {
        // este pré-processamento deve ser feito para que a montagem do grafo funcione normalmente
        code = remover_rotulos_simbolicos(code);

        let mut grafo = CodeGraph {
            grafo: Graph::new(),
            funcoes: Vec::new(),
            memoria_consistente: false,
            variaveis: Vec::new()
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
                        initial_memory_usage: None,
                        allocation:None
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

        // procura usos de funcoes e usos de variaveis
        let mut updates = Vec::new();
        for line in grafo.instructions_unordered() {
            match &line.instruction {
                Instruction::CHPR(label)=>{
                    updates.push((label.unwrap(), line.address));
                },
                _=>()
            }
        }
        for (label, address) in updates {
            for f in &mut grafo.funcoes {
                if f.addr_inicio == label {
                    f.usos.push(address);
                }
            }
        }

        // lista as raizes (inicios de funcao e inicio do programa) e a memoria inicial de cada um
        let bases_mapeamento: Vec<(usize, usize)> = grafo
            .funcoes
            .iter()
            .map(|func| (func.addr_inicio, func.args+1)) // este +1 é para contar com a memoria alocada por CHPR
            .chain(std::iter::once((0,0)))
            .collect();
        
        println!("bases_mapeamento: {:?}",bases_mapeamento);
        // mapeia o uso de memoria
        grafo.memoria_consistente = bases_mapeamento
            .iter()
            .map(|raiz| {
                let r = grafo.mapear_memoria_a_partir_de(raiz.0, raiz.1);
                println!("Mapeada a memoria a partir de {}: {}",raiz.0, r);
                r
            })
            .all(|r| r);

        grafo
    }
    // retorna se teve sucesso ou não
    pub fn mapear_memoria_a_partir_de(&mut self, addr: usize, initial_value: usize) -> bool {
        let raiz = self.locate_address(addr).unwrap();

        // valores da instrucao
        for line in self.grafo.node_weight_mut(raiz).unwrap() {
            if line.address == addr {
                line.initial_memory_usage = Some(initial_value);
                line.allocation = None;
            }
        }

        let mut dfs = Dfs::new(&self.grafo, raiz);

        // (address, memory, memory_delta)
        let mut alocation_stack: Vec<(usize, usize, i32)> = Vec::new();

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
            let mut memory:usize = lines.first().unwrap().initial_memory_usage.unwrap();
            for line_idx in 0..lines.len() {
                // println!("{:?}",lines[line_idx]);
                let memory_delta = match &lines[line_idx].instruction {
                    Instruction::CRCT(_)
                    | Instruction::CRVL(_, _)
                    | Instruction::CREN(_, _)
                    | Instruction::CRVI(_, _)
                    | Instruction::LEIT
                    | Instruction::ENPR(_) => 1,
                    Instruction::ARMZ(nivel_lexico, nivel_memoria) =>{
                        // if *nivel_lexico==1{
                        //     println!("addr: {} | nivel_memoria: {} | memory: {}", lines[line_idx].address, nivel_memoria, memory);
                        //     // Passa por cada elemento de alocation_stack, do ultimo ao primeiro, até encontrar aquele em que 
                        //     // item.memory - item.memory_delta < nivel_memoria-4
                        //     // se achar, print
                        //     // se não, retorne falso
                        //     println!("alocation_stack: {:?}",alocation_stack);
                        //     let mut achou = false;
                        //     for item in alocation_stack.iter().rev() {
                        //         println!("item: {:?}",item);
                        //         println!(" {} < {} + 4", item.1 as i32 , nivel_memoria );
                        //         // if item.1 as i32 < nivel_memoria + 4 {
                        //         //     println!("alocado em: {}", item.0);
                        //         //     achou = true;
                        //         //     break;
                        //         // }
                        //     }
                        //     if !achou{
                        //         return false;
                        //     }
                        // }
                        -1
                    },
                    Instruction::ARMI(_, _)
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
                // verifica se memoria total não será negativa, caso que invalida mapeamento
                if memory_delta < 0 && memory < (-memory_delta) as usize { 
                    return false;
                } 
                memory = (memory as i32 + memory_delta) as usize;
                if memory_delta > 0 {
                    alocation_stack.push((lines[line_idx].address, memory, memory_delta));
                } else if memory_delta < 0 {
                    println!("{} libera {}",lines[line_idx].address, memory_delta);
                    println!("alocation_stack: {:?}",alocation_stack);

                    // for each item on the stack, from last to first
                    while let Some(&top) = alocation_stack.last() {
                        // if it is greater or equal than memory, pop
                        println!("{} > {}",top.1, memory);
                        if top.1 > memory {
                            alocation_stack.pop();
                            alocation_map.push((top.0, Allocation{amount:top.2 as usize,liberation_address:lines[line_idx].address}));
                        }
                        // else, stop
                        else {
                            break;
                        }
                    }
                }
                // se houver proximo (não for ultima passada do loop), adiciona memoria inicial ao proximo
                if line_idx + 1 < lines.len() {
                    lines[line_idx + 1].initial_memory_usage = Some(memory as usize);
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
                if let Some(existing_value) = neighbor.initial_memory_usage {
                    if existing_value != memory as usize {
                        // falhou o mapeamento
                        return false;
                    }
                }
                neighbor.initial_memory_usage = Some(memory as usize);
            }
        }

        for (addr, aloc) in alocation_map {
            self.instruction_mut(addr).unwrap().allocation = Some(aloc);
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
        if remap && self.memoria_consistente{
            if let Some(memory_usage_before) = self.instruction_mut(addr).unwrap().initial_memory_usage {
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
                    if self.memoria_consistente{
                        format!(
                            "[{}] {}: {}{}",
                            linha.initial_memory_usage.unwrap(),
                            linha.address,
                            linha.instruction,
                            if let Some(aloc) = &linha.allocation {
                                format!(
                                    " ({} | {})",
                                    aloc.amount, aloc.liberation_address
                                )
                            } else {
                                "".to_string()
                            }
                        )
                    }
                    else {
                        format!(
                            "{}: {}",
                            linha.address,
                            linha.instruction
                        )
                    }
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
