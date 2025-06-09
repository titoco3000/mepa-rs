mod compiler;
mod error;
mod lexic;
mod simbol_table;
pub use compiler::{compile, compile_from_str};
pub use error::CompileError;
