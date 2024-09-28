use std::path::PathBuf;

use mepa::interactive_execution;

mod mepa;
fn main() {

     interactive_execution( &PathBuf::from("samples/recursao.mepa"));
}
