#[allow(unused_imports)]
use mepa_rs::compiler::compile;
#[allow(unused_imports)]
use mepa_rs::mepa::interactive_execution;
use std::path::PathBuf;

fn main() {
    compile(&PathBuf::from("samples/movimentacao.cl"),&PathBuf::from( "output/movimentacao.mepa")).unwrap();
    //interactive_execution(&PathBuf::from("output/code.mepa"));
}
