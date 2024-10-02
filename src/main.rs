mod mepa;
mod compiler;

use std::path::PathBuf;

fn main() {

     mepa::interactive_execution( &PathBuf::from("samples/c1.mepa"));
     //compiler::compile(&PathBuf::from("samples/code.txt"), &PathBuf::from("output/code.mepa")).unwrap();     
}
