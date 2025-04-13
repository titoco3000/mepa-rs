use std::fmt;

#[derive(Debug, Clone)]
pub enum CompileError {
    Lexic(String),
    Sintatic(String),
    Semantic(String),
}

impl fmt::Display for CompileError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CompileError::Lexic(e) => {
                write!(f, "Erro léxico: {}", e)
            }
            CompileError::Sintatic(e) => {
                write!(f, "Erro sintático: {}", e)
            }
            CompileError::Semantic(e) => {
                write!(f, "Erro semântico: {}", e)
            }
        }
    }
}

impl std::error::Error for CompileError {}
