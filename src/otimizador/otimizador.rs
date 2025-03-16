use super::grafo::{CodeGraph, InstructionAndMetadata};
use crate::mepa::code::MepaCode;
use crate::mepa::instruction::Instruction;
use crate::mepa::label::Label;
use petgraph::graph::NodeIndex;
use petgraph::visit::EdgeRef;
use std::io;
use std::path::{Path, PathBuf};

pub fn otimizar_arquivo<P>(filename: P) -> io::Result<()>
where
    P: AsRef<Path>,
{
    let code = otimizar(MepaCode::from_file(&filename)?);

    code.to_file(&filename).unwrap();
    Ok(())
}

pub fn otimizar(code: MepaCode) -> MepaCode {
    let c = code;
    let mut code = CodeGraph::new(c.clone());

    code.export_to_file(&PathBuf::from("output/debug/antes.dot"))
        .unwrap();

    let functions = [fluxo, elimidar_codigo_morto, propagar_constantes];

    loop {
        let mut mudou = false;

        for &func in &functions {
            while func(&mut code) {
                mudou = true;
            }
        }

        if !mudou {
            break;
        }
    }

    code.print_vars();

    code.export_to_file(&PathBuf::from("output/debug/depois.dot"))
        .unwrap();

    code.to_mepa_code()
}

//se tem um desvio que cai em outro desvio, pula direto para pos final
//retorna se mudou algo
fn fluxo(code: &mut CodeGraph) -> bool {
    println!("Otimizando fluxo");
    // cria vec de (origem, destino)
    let mut desvio_para_desvio: Vec<(usize, usize)> = code
        .grafo
        .node_indices()
        .filter_map(
            |node_index| match code.grafo.node_weight(node_index).unwrap().first() {
                Some(line) => match &line.instruction {
                    Instruction::DSVS(label) => Some((line.address, label.unwrap())),
                    _ => None,
                },
                _ => None,
            },
        )
        .collect();

    // elimina o problema entre os próprios redundantes
    for i in 0..desvio_para_desvio.len() {
        for j in 0..desvio_para_desvio.len() {
            if desvio_para_desvio[i].1 == desvio_para_desvio[j].0 {
                desvio_para_desvio[i].1 = desvio_para_desvio[j].1;
            }
        }
    }

    code.grafo.node_indices().any(|node| {
        let mut mudancas = None; //(old, new)
        let last_line = code
            .grafo
            .node_weight_mut(node)
            .unwrap()
            .iter_mut()
            .last()
            .unwrap();
        match &mut last_line.instruction {
            Instruction::DSVS(label) | Instruction::DSVF(label) | Instruction::CHPR(label) => {
                for inutil in &desvio_para_desvio {
                    if inutil.0 == label.unwrap() {
                        mudancas = Some((label.unwrap(), inutil.1));
                        *label = Label::Literal(inutil.1);
                        break;
                    }
                }
            }
            _ => {}
        }
        if let Some((old_label, new_label)) = mudancas {
            let old_node_index = code.locate_address(old_label).unwrap();
            let new_node_index = code.locate_address(new_label).unwrap();

            // Remove aresta antiga
            code.grafo.remove_edge(
                code.grafo
                    .edges(node)
                    .find(|edge| edge.source() == node && edge.target() == old_node_index)
                    .map(|edge| edge.id())
                    .unwrap(),
            );
            // verifica se ja existe uma equivalente a nova
            if let None = code
                .grafo
                .edges(node)
                .find(|edge| edge.source() == node && edge.target() == new_node_index)
                .map(|edge| edge.id())
            {
                // Adiciona substituta
                code.grafo.add_edge(node, new_node_index, ());
            }
            true
        } else {
            false
        }
    })
}

//se tem codigo inacessivel, remove
fn elimidar_codigo_morto(code: &mut CodeGraph) -> bool {
    println!("Eliminando codigo morto");
    let mut mudou = false;
    // acha nodes inacessiveis
    let inacessiveis: Vec<NodeIndex> = code
        .grafo
        .node_indices()
        .filter(|node_idx| {
            code.grafo
                .edges_directed(*node_idx, petgraph::Direction::Incoming)
                .count()
                == 0
                && {
                    let line = code.grafo.node_weight(*node_idx).unwrap().first().unwrap();
                    // node inicial não deve ser removido
                    if line.address == 0 {
                        false
                    } else {
                        match line.instruction {
                            // se é entrada de uma função
                            Instruction::ENPR(_) => {
                                // remove se não tiver nenhuma chamada
                                code.funcoes
                                    .iter()
                                    .find(|f| f.addr_inicio == line.address)
                                    .unwrap()
                                    .usos
                                    .len()
                                    == 0
                            }
                            // se não pode remover
                            _ => true,
                        }
                    }
                }
        })
        .collect();

    for i in inacessiveis {
        code.remove_node(i);
        mudou = true;
    }
    if mudou {
        code.mapear_memoria();
    }
    mudou
}

fn propagar_constantes(code: &mut CodeGraph) -> bool {
    if !code.memoria_consistente {
        return false;
    }
    let mut mudou = false;
    // localiza todos CRCT
    let declaracoes_de_constantes: Vec<InstructionAndMetadata> = code
        .instructions_unordered()
        .filter_map(|line| {
            if matches!(line.instruction, Instruction::CRCT(_)) {
                Some(line.clone())
            } else {
                None
            }
        })
        .collect();
    // para cada um, localiza todos os usos
    for declaracao in declaracoes_de_constantes {
        // Para o ARMZ que usa esse valor (que será ou um ou zero)
        let aloc_addresses: Vec<(usize, usize)> = declaracao.allocation.unwrap().variaveis[0]
            .usos
            .iter()
            .filter_map(|uso| {
                let line = code.instruction(*uso).unwrap();
                if matches!(line.instruction, Instruction::ARMZ(_, _)) {
                    line.armazena_em
                } else {
                    None
                }
            })
            .collect();

        //para cada alocacao-destino
        for aloc_addr in &aloc_addresses {
            // cada CRVL que usa essa instrucao

            let carregamentos: Vec<usize> = code
                .instruction(aloc_addr.0)
                .unwrap()
                .allocation
                .as_ref()
                .unwrap()
                .variaveis[aloc_addr.1]
                .usos
                .clone()
                .iter()
                .filter_map(|addr| {
                    let line = code.instruction(*addr).unwrap();
                    if matches!(line.instruction, Instruction::CRVL(_, _)) {
                        Some(line.address.clone())
                    } else {
                        None
                    }
                })
                .collect();
            
            // remove usos da lista, ja que vao ser removidos
            if let Some(aloc) = &mut code
                .instruction_mut(aloc_addr.0)
                .unwrap()
                .allocation
                .as_mut()
            {
                for c in &carregamentos {
                    aloc.variaveis[aloc_addr.1].usos.remove(&c);
                }
            }
            for c in carregamentos {
                if let Instruction::CRCT(n) = declaracao.instruction{
                    if let Some(line) = code.instruction_mut(c){
                        println!("Substituindo {} da linha {} por CRCT({})",line.instruction, c,n);
                        line.instruction = Instruction::CRCT(n);
                        line.carrega_de = None;
                        mudou = true;
                    }
                }
            }

        }
    }
    mudou
}
