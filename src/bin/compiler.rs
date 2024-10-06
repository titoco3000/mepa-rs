use mepa_rs::compiler::compile;
use std::path::PathBuf;

fn main() {
    compile(&PathBuf::from("samples/code.txt"),&PathBuf::from( "output/code.mepa")).unwrap();
}
