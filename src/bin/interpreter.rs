use mepa_rs::mepa;
use std::path::PathBuf;

fn main() {
     mepa::interactive_execution( &PathBuf::from("samples/c1.mepa"));
}
