use std::error::Error;
use std::fmt;
use std::io;

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub enum MepaError {
    IO(String),
    Runtime(String),
    MissingInput(usize),
    Other(String),
}

impl fmt::Display for MepaError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MepaError::IO(e) => write!(f, "Erro de IO: {}", e),
            MepaError::Runtime(msg) => write!(f, "Erro de execução: {}", msg),
            MepaError::MissingInput(linha) => write!(f, "Falta de input: linha {}", linha + 1),
            MepaError::Other(msg) => write!(f, "Erro: {}", msg),
        }
    }
}

impl Error for MepaError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            //MepaError::IO(e) => Some(e),
            _ => None,
        }
    }
}

impl From<io::Error> for MepaError {
    fn from(err: io::Error) -> Self {
        MepaError::IO(err.to_string())
    }
}

impl From<String> for MepaError {
    fn from(msg: String) -> Self {
        MepaError::Other(msg)
    }
}

impl From<&str> for MepaError {
    fn from(msg: &str) -> Self {
        MepaError::Other(msg.to_string())
    }
}

// Define the Mepa result type
pub type MepaResult<T> = Result<T, MepaError>;
