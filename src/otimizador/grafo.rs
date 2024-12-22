use crate::mepa::code::MepaCode;
use crate::mepa::instruction::Instruction;
use crate::mepa::label::Label;
use petgraph::dot::{Config, Dot};
use petgraph::graph::NodeIndex;
use petgraph::Graph;
use std::collections::HashSet;
use std::fmt::Debug;
use std::fs::{self, File};
use std::io::{self, Write};
use std::path::PathBuf;

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
                    Instruction::RTPR(_, _) => {
                        Some(vec![i + 1])
                    } // Add the next instruction
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
                        let return_block = code.iter().enumerate().skip(*jmp_addr).find_map(|(ret_addr, (_, inst))| {
                            if let Instruction::RTPR(_, _) = inst {
								Some(
									vertices
										.iter()
										.find(|&&output_index| {
											let &(start, end) = grafo.node_weight(output_index).unwrap();
											start <= ret_addr && ret_addr < end
										})
										.expect("Address not found in groupings")
										.clone()
									)
                            } else {								
                                None
                            }
                        }).unwrap();
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
				Instruction::PARA|Instruction::RTPR(_,_)=>(),
				_=>{
					// se é a ultima linha do bloco e não é nenhum pulo, então é um pulo para o próximo
					if linha+1 == linhas.1{
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
		let linhas_de_mepa:Vec<String> = (inicio..fim).map(|linha|format!("{}: {}",linha, code[linha].1)).collect();
		let s = linhas_de_mepa.join("\n");
		graph_with_code.add_node(s);
	}

	let arestas:Vec<(NodeIndex, NodeIndex)> =  graph.raw_edges()
		.iter()
		.map(|edge|(edge.source(), edge.target())).collect();

		graph_with_code.extend_with_edges(&arestas);

		let raw_dot = format!(
			"{:?}",
			Dot::with_config(&graph_with_code, &[Config::EdgeNoLabel])
		);
	
		let processed_dot = raw_dot
			.replace("\\\"", "")
			.replace("\\\\", "\\")
		;
	
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

		let linhas_de_mepa:Vec<String> = (inicio..fim).map(|linha|format!("{}: {} - {}",linha, code[linha].1, memory_usage[linha-inicio])).collect();
		let s = linhas_de_mepa.join("\n");
		graph_with_code.add_node(s);
	}

	let arestas:Vec<(NodeIndex, NodeIndex)> =  graph.raw_edges()
		.iter()
		.map(|edge|(edge.source(), edge.target())).collect();

		graph_with_code.extend_with_edges(&arestas);

		let raw_dot = format!(
			"{:?}",
			Dot::with_config(&graph_with_code, &[Config::EdgeNoLabel])
		);
	
		let processed_dot = raw_dot
			.replace("\\\"", "")
			.replace("\\\\", "\\")
		;
	
		// Write the processed string to the file
		write!(&file, "{}", processed_dot)
}

pub fn raw_graph_to_file<T>(
    filename: &PathBuf,
    graph: &Graph<T, ()>,
) -> io::Result<()> where T:Debug{
    if let Some(parent) = filename.parent() {
        fs::create_dir_all(parent)?;
    }

    // Create or open the file
    let file = File::create(filename)?;

    let raw_dot = format!(
        "{:?}",
        Dot::with_config(&graph, &[Config::EdgeNoLabel])
    );
	
    let processed_dot = raw_dot
        .replace("\\\"", "")
        .replace("\\\\", "\\")
    ;

    // Write the processed string to the file
    write!(&file, "{}", processed_dot)
}
