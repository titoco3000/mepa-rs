use wasm_bindgen::prelude::wasm_bindgen;
use wasm_bindgen::JsValue;

use crate::machine::basic_machine::BasicMachine;
#[wasm_bindgen]
pub struct MepaMachine {
    internal: Option<BasicMachine>,
    error: Option<String>,
}

#[wasm_bindgen]
impl MepaMachine {
    #[wasm_bindgen(constructor)]
    pub fn new(input: &str) -> MepaMachine {
        match BasicMachine::from_str(input) {
            Ok(machine) => MepaMachine {
                internal: Some(machine),
                error: None,
            },
            Err(e) => MepaMachine {
                internal: None,
                error: Some(e.to_string()),
            },
        }
    }

    /// Call this after `new` to check why it failed
    pub fn get_error(&self) -> Option<String> {
        self.error.clone()
    }

    pub fn step(&mut self, input: Option<i32>) -> Result<JsValue, JsValue> {
        if let Some(machine) = &mut self.internal {
            match machine.step_with_input(input) {
                Ok(Some(output)) => Ok(JsValue::from_f64(output as f64)),
                Ok(None) => Ok(JsValue::null()),
                Err(e) => Err(serde_wasm_bindgen::to_value(&e)?),
            }
        } else {
            Err(JsValue::from_str(
                self.error.as_deref().unwrap_or("Machine not initialized"),
            ))
        }
    }

    pub fn get_state(&self) -> JsValue {
        serde_wasm_bindgen::to_value(&self.internal).unwrap()
    }
}
