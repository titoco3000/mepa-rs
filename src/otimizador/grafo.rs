use crate::mepa::code::MepaCode;
use crate::mepa::instruction::{self, Instruction};
use crate::mepa::label::Label;
use petgraph::dot::{Config, Dot};
use petgraph::graph::NodeIndex;
use petgraph::{visit::Dfs, Graph};
use std::collections::HashMap;
use std::collections::HashSet;
use std::fs::{self, File};
use std::io::{self, Write};
use std::iter::Fuse;
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

struct InstructionAndMetadata {
    address: usize,
    instruction: Instruction,
    memory_usage: Option<usize>,
    liberation_address: Option<usize>,
}

struct FuncMetadata {
    addr_inicio: usize,
    addr_retorno: usize,
    acesso: usize, //maximo de memoria acessado externo
    args: usize,
}

pub struct CodeGraph {
    grafo: Graph<Vec<InstructionAndMetadata>, ()>,
    funcoes: Vec<FuncMetadata>,
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

        // mapeia o uso de memoria

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
                    }),
                    _ => None,
                })
                .collect()
        };

        let raizes: Vec<NodeIndex> = grafo
            .funcoes
            .iter()
            .map(|func| grafo.locate_address(func.addr_inicio).unwrap().clone())
            .chain(std::iter::once(0.into()))
            .collect();
        for raiz in raizes {
            // primeira instrucao usa 0
            grafo
                .grafo
                .node_weight_mut(raiz)
                .unwrap()
                .first_mut()
                .unwrap()
                .memory_usage = Some(0);
            let mut dfs = Dfs::new(&grafo.grafo, raiz);

            let mut inconsistent_memory_usage = false;

            while let Some(visited) = dfs.next(&grafo.grafo) {
                println!("Visitando o node {:?}", visited);
                let neighbors: Vec<NodeIndex> =
                    grafo.grafo.neighbors(visited).map(|n| n.clone()).collect();
                let lines = grafo.grafo.node_weight_mut(visited).unwrap();
                let mut memory = lines.first().unwrap().memory_usage.unwrap() as i32;
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
                            - (grafo.funcoes.iter().find(|f|f.addr_inicio==k.unwrap()).unwrap().args as i32)
                        },
                        _ => 0,
                    };
                    // assign to the next line, if available
                    if line + 1 < lines.len() {
                        lines[line + 1].memory_usage = Some(memory as usize);
                    }
                }
                // Propagate to neighbors
                for neighbor_index in neighbors.into_iter() {
                    let neighbor = grafo
                        .grafo
                        .node_weight_mut(neighbor_index)
                        .unwrap()
                        .first_mut()
                        .unwrap();
                    if let Some(existing_value) = neighbor.memory_usage {
                        if existing_value != memory as usize {
                            inconsistent_memory_usage = true;
                            break;
                        }
                    }
                    neighbor.memory_usage = Some(memory as usize);
                }
                if inconsistent_memory_usage {
                    break;
                }
            }
        }

        // for funcao in &grafo.funcoes {
        //     let mut dfs_stack = Vec::with_capacity(grafo.grafo.node_count());
        //     let mut visited = Vec::with_capacity(grafo.grafo.node_count());
        //     dfs_stack.push(grafo.locate_address(funcao.addr_inicio).unwrap());

        //     while let Some(node) = dfs_stack.pop() {
        //         visited.push(node);
        //         let lines = grafo.grafo.node_weight(node).unwrap();
        //         let start = lines[0].address;
        //         let end = lines.last().unwrap().address;
        //         println!("Visiting node with addr: {}", start);

        //         if funcao.addr_retorno >= start && funcao.addr_retorno <= end {
        //             println!("Achou retorno de {}", funcao.addr_inicio);
        //         } else {
        //             for n in grafo
        //                 .grafo
        //                 .neighbors_directed(node, petgraph::Direction::Outgoing)
        //             {
        //                 if !visited.contains(&n) && !dfs_stack.contains(&n) {
        //                     dfs_stack.push(n);
        //                 }
        //             }
        //         }
        //     }
        // }

        // primeira instrucao usa 0
        // grafo
        //     .grafo
        //     .node_weight_mut(0.into())
        //     .unwrap()
        //     .first_mut()
        //     .unwrap()
        //     .memory_usage = Some(0);

        // // Use DFS for graph traversal
        // let mut dfs = Dfs::new(&grafo.grafo, 0.into());

        // let mut inconsistent_memory_usage = false;

        // while let Some(visited) = dfs.next(&grafo.grafo) {
        //     println!("Visitando o node {:?}", visited);
        //     let neighbors: Vec<NodeIndex> = grafo.grafo.neighbors(visited).map(|n| n.clone()).collect();
        //     let lines = grafo.grafo.node_weight_mut(visited).unwrap();
        //     let mut memory = lines.first().unwrap().memory_usage.unwrap() as i32;
        //     for line in 0..lines.len() {
        //         memory += match lines[line].instruction {
        //             Instruction::CRCT(_)
        //             | Instruction::CRVL(_, _)
        //             | Instruction::CREN(_, _)
        //             | Instruction::CRVI(_, _)
        //             | Instruction::LEIT
        //             | Instruction::CHPR(_)
        //             | Instruction::ENPR(_) => 1,
        //             Instruction::ARMZ(_, _)
        //             | Instruction::ARMI(_, _)
        //             | Instruction::SOMA
        //             | Instruction::SUBT
        //             | Instruction::MULT
        //             | Instruction::DIVI
        //             | Instruction::CONJ
        //             | Instruction::DISJ
        //             | Instruction::CMME
        //             | Instruction::CMMA
        //             | Instruction::CMIG
        //             | Instruction::CMDG
        //             | Instruction::CMEG
        //             | Instruction::CMAG
        //             | Instruction::DSVF(_)
        //             | Instruction::IMPR => -1,
        //             Instruction::AMEM(n) => n,
        //             Instruction::DMEM(n) => -n,
        //             Instruction::RTPR(_, n) => -n - 2,
        //             _ => 0,
        //         };
        //         // assign to the next line, if available
        //         if line + 1 < lines.len() {
        //             lines[line + 1].memory_usage = Some(memory as usize);
        //         }
        //     }
        //     // Propagate to neighbors
        //     for neighbor_index in neighbors.into_iter() {
        //         let neighbor = grafo
        //             .grafo
        //             .node_weight_mut(neighbor_index)
        //             .unwrap()
        //             .first_mut()
        //             .unwrap();
        //         if let Some(existing_value) = neighbor.memory_usage {
        //             if existing_value != memory as usize {
        //                 inconsistent_memory_usage = true;
        //                 break;
        //             }
        //         }
        //         neighbor.memory_usage = Some(memory as usize);
        //     }
        //     if inconsistent_memory_usage {
        //         break;
        //     }
        // }

        // if inconsistent_memory_usage {
        //     for block in grafo.grafo.node_weights_mut() {
        //         for line in block{
        //             line.memory_usage = None;
        //             line.liberation_address = None;
        //         }
        //     }
        // }

        grafo
    }

    fn locate_address(&self, addr: usize) -> Option<NodeIndex> {
        self.grafo.node_indices().find(|&output_index| {
            let (start, end) = (
                self.grafo
                    .node_weight(output_index)
                    .unwrap()
                    .first()
                    .unwrap()
                    .address,
                self.grafo
                    .node_weight(output_index)
                    .unwrap()
                    .last()
                    .unwrap()
                    .address,
            );
            addr >= start && addr <= end
        })
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

    pub fn remove_instruction(addr: usize) {}

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
                        "{}: {} - {}",
                        linha.address,
                        linha.instruction,
                        if let Some(memory_usage) = linha.memory_usage {
                            memory_usage.to_string()
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
