use super::grafo::graph_to_file_with_memory_usage;
use super::grafo::{graph_to_file, map_code_to_graph};
use super::pre_processamento::remover_rotulos_simbolicos;
use crate::mepa::code::MepaCode;
use crate::mepa::instruction::Instruction;
use crate::mepa::label::Label;
use petgraph::{visit::Dfs, Graph};
use std::path::{Path, PathBuf};

pub fn mapear_variaveis(code: &MepaCode, grafo: &Graph<(usize, usize), ()>) {
    let mut memory_usage = Graph::<Vec<usize>, ()>::new();

    // Map old node indices to new node indices
    let mut node_map = std::collections::HashMap::new();

    // Copy nodes with converted weights
    for node_index in grafo.node_indices() {
        let (start, end) = grafo[node_index];
        let new_node_index = memory_usage.add_node(Vec::with_capacity(end - start));
        node_map.insert(node_index, new_node_index);
    }

    // Copy edges
    for edge in grafo.edge_indices() {
        if let Some((source, target)) = grafo.edge_endpoints(edge) {
            let new_source = node_map[&source];
            let new_target = node_map[&target];
            memory_usage.add_edge(new_source, new_target, ());
        }
    }

    memory_usage.node_weight_mut(0.into()).unwrap().push(0);

    // Use DFS for graph traversal
    let mut dfs = Dfs::new(&*grafo, 0.into());

    while let Some(visited) = dfs.next(&*grafo) {
        println!("Visitando o node {:?}", visited);
        let (start, end) = *grafo.node_weight(visited).unwrap();
        let current_memory = memory_usage.node_weight(visited).unwrap();
        let mut current_value = *current_memory.last().unwrap_or(&0);

        for i in start..end {
            // println!("Vendo instrução {:?}",i);
            current_value = current_value.saturating_add_signed(
                match code[i].1 {
                    Instruction::CRCT(_)
                    | Instruction::CRVL(_, _)
                    | Instruction::CREN(_, _)
                    | Instruction::CRVI(_, _)
                    | Instruction::LEIT
                    | Instruction::CHPR(_)
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
                    Instruction::AMEM(n) => n,
                    Instruction::DMEM(n) => -n,
                    Instruction::RTPR(_, n) => -n - 2,
                    _ => 0,
                }
                .try_into()
                .unwrap(),
            );
            if i + 1 < end {
                memory_usage
                    .node_weight_mut(visited)
                    .unwrap()
                    .push(current_value);
            }
        }

        // Propagate to neighbors
        for neighbor in grafo.neighbors(visited) {
            let neighbor_index = node_map[&neighbor];
            let neighbor_memory = memory_usage.node_weight_mut(neighbor_index).unwrap();

            if let Some(existing_value) = neighbor_memory.first() {
                if *existing_value != current_value {
                    println!(
                        "Warning: Overwriting memory usage in node {:?} from {} to {}",
                        grafo.node_weight(neighbor_index),
                        existing_value,
                        current_value
                    );
                }
            }

            if neighbor_memory.is_empty() {
                neighbor_memory.push(current_value);
            } else {
                neighbor_memory[0] = current_value; // Overwrite with the new value
            }
        }
    }
    graph_to_file_with_memory_usage(
        &PathBuf::from("output/debug/mem.dot"),
        &code,
        &grafo,
        &memory_usage,
    )
    .unwrap();
}
