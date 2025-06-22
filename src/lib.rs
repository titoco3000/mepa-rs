pub mod compiler;
pub mod evaluator;
pub mod machine;
pub mod mepa;
pub mod otimizador;
pub mod utils;
use std::path::PathBuf;

use serde::Serialize;
use wasm_bindgen::prelude::*;

use crate::otimizador::Otimizador;

// #[cfg(target_arch = "wasm32")]
pub use machine::wasm_machine::MepaMachine;

#[derive(Serialize)]
struct CompilationOutput {
    mepa: Result<String, String>,
    optimized: Option<Result<String, String>>,
}

impl CompilationOutput {
    fn to_json_string(&self) -> Result<String, JsValue> {
        serde_json::to_string(self)
            .map_err(|e| JsValue::from_str(&format!("Failed to serialize result: {}", e)))
    }
}

#[wasm_bindgen]
pub fn compile_code(input: &str) -> Result<String, JsValue> {
    let output = match compiler::compile_from_str(input) {
        Ok(mepacode) => {
            let mepa_string = mepacode.to_string().unwrap_or_else(|e| e.to_string());
            let optimization_result = match Otimizador::<PathBuf>::new(mepacode, None).otimizar() {
                Ok(optimized_code) => Some(Ok(optimized_code.to_string())),
                Err(e) => Some(Err(e.to_string())),
            };
            CompilationOutput {
                mepa: Ok(mepa_string),
                optimized: optimization_result,
            }
        }
        Err(compile_err) => CompilationOutput {
            mepa: Err(compile_err.to_string()),
            optimized: None,
        },
    };

    output.to_json_string()
}
