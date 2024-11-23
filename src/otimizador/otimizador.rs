use crate::mepa::code::MepaCode;
use petgraph::Graph;
use std::io;
use std::path::{Path, PathBuf};

use super::pre_processamento::remover_rotulos_simbolicos;
use super::grafo::{map_code_to_graph, graph_to_file};

pub struct Otimizador {
    code: MepaCode,
    grafo: Graph<(usize, usize), ()>,
}

impl Otimizador {
    pub fn from_file<P>(filename: P) -> io::Result<Otimizador>
    where
        P: AsRef<Path>,
    {
        let code = remover_rotulos_simbolicos(MepaCode::from_file(filename)?);        
        code.clone().to_file(&PathBuf::from("output/debug/code.mepa")).unwrap();
        let grafo = map_code_to_graph(&code);
        graph_to_file(&PathBuf::from("output/debug/graph.dot"), &code, &grafo).unwrap();
        let otm = Otimizador { code, grafo };

        Ok(otm)
    }
}
