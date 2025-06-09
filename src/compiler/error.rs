use std::fmt;

#[derive(Debug)]
pub enum CompileError {
    Lexic(String),
    Sintatic(String),
    Semantic(String),
}

impl fmt::Display for CompileError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            CompileError::Lexic(msg) => write!(f, "Lexical Error: {}", msg),
            CompileError::Sintatic(msg) => write!(f, "Sintatic Error: {}", msg),
            CompileError::Semantic(msg) => write!(f, "Semantic Error: {}", msg),
        }
    }
}
