use super::grafo::{graph_to_file, map_code_to_graph};
use super::pre_processamento::remover_rotulos_simbolicos;
use crate::mepa::code::MepaCode;
use crate::mepa::instruction::Instruction;
use crate::mepa::label::Label;
use petgraph::Graph;
use std::io;
use std::path::{Path, PathBuf};

pub fn otimizar(code:MepaCode)->MepaCode{
    let mut code = remover_rotulos_simbolicos(code);
    let mut grafo = map_code_to_graph(&code);

    let functions = [fluxo, elimidar_codigo_morto];
    
    loop {
        let mut mudou = false;

        for &func in &functions {
            while func(&mut code, &mut grafo) {
                mudou = true;
            }
        }

        if !mudou {
            break; 
        }
    }    
    graph_to_file(&PathBuf::from("output/debug/graph.dot"), &code, &grafo).unwrap();
    code
}

pub fn otimizar_arquivo<P>(filename: P) -> io::Result<()>
where
    P: AsRef<Path>,
{
    let code = otimizar(MepaCode::from_file(&filename)?);
    
    code.to_file(&filename)
        .unwrap();
    Ok(())
}

//se tem um desvio que cai em outro desvio, pula direto para pos final
//retorna se mudou algo
fn fluxo(code: &mut MepaCode, grafo: &mut Graph<(usize, usize), ()>) -> bool {
    println!("Otimizando fluxo");
    let mut mudou = false;
    for i in 0..code.len() {
        if let Instruction::DSVS(label) = &code[i].1 {
            if let Label::Literal(addr) = label {
                if let Instruction::DSVS(label) = &code[*addr].1 {
                    if let Label::Literal(novo_addr) = label {
                        if addr != novo_addr{
                            code[i] = (None, Instruction::DSVS(Label::Literal(*novo_addr)));
                            mudou = true;
                        }
                    }
                }
            }
        }
    }
    if mudou {
        *grafo = map_code_to_graph(&code);
    }
    mudou
}

//se tem codigo inacessivel, remove
fn elimidar_codigo_morto(code: &mut MepaCode, grafo: &mut Graph<(usize, usize), ()>) -> bool {
    println!("Eliminando codigo morto");
    let mut mudou = false;
    let mut index = code.len() - 2;
    while let Some(bloco) = grafo.node_indices().find(|i| {
        let (inicio, fim) = grafo[*i];
        inicio > 0 && inicio <= index && index < fim
    }) {
        let &(inicio, fim) = grafo.node_weight(bloco).unwrap();
        let isolado = grafo
            .edges_directed(bloco, petgraph::Direction::Incoming)
            .peekable()
            .peek()
            .is_none();
        if isolado {
            //println!("Achou bloco isolado: ({},{})", inicio, fim);
            mudou = true;
            for i in (inicio..fim).rev() {
                code.remove_instruction(i);
            }
            grafo.remove_node(bloco);
        }
        index = if inicio == 0 { 0 } else { inicio - 1 };
    }
    if mudou {
        *grafo = map_code_to_graph(&code);
    }
    mudou
}
