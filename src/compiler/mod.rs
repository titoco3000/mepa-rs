mod lexic;
mod compiler;
mod simbol_table;
mod error;
pub use compiler::compile;
pub use error::CompileError;