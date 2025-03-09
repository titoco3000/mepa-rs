#[derive(Debug)]
pub enum CompileError {
    Lexic(String),
    Sintatic(String),
    Semantic(String),
}
