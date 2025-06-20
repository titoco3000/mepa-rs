use crate::machine::basic_machine::BasicMachine;
use crate::mepa::code::MepaCode;
use crate::mepa::error::MepaResult;
use crate::utils::input_i32;

use std::io::{BufReader, Read};

pub enum InputSource {
    Vec(Vec<i32>),
    Readable(Box<dyn Read>),
    Stdin,
}
impl InputSource {
    pub fn readable<R: Read + 'static>(readable: R) -> Self {
        InputSource::Readable(Box::new(readable))
    }
    pub fn read(&mut self) -> Option<i32> {
        match self {
            Self::Vec(v) => {
                if v.len() > 1 {
                    v.pop()
                } else {
                    v.get(0).copied()
                }
            }

            Self::Stdin => Some(input_i32()),

            Self::Readable(readable) => {
                let mut buf_reader = BufReader::new(readable);
                let mut buffer = String::new();

                if buf_reader.read_to_string(&mut buffer).is_ok() {
                    buffer
                        .split_whitespace()
                        .next()
                        .and_then(|s| s.parse::<i32>().ok())
                } else {
                    None
                }
            }
        }
    }
}

pub struct MepaMachine<'a> {
    basic_machine: BasicMachine,
    input: InputSource,
    output: Option<&'a mut Vec<i32>>,
}

impl<'a> MepaMachine<'a> {
    pub fn new(code: MepaCode) -> MepaMachine<'a> {
        let basic_machine = BasicMachine::new(code);
        MepaMachine {
            basic_machine,
            input: InputSource::Stdin,
            output: None,
        }
    }
    pub fn add_input_vec(mut self, input: Vec<i32>) -> Self {
        self.input = InputSource::Vec(input);
        self
    }
    pub fn add_output(mut self, output: &'a mut Vec<i32>) -> Self {
        self.output = Some(output);
        self
    }
    pub fn add_input<R: Read + 'static>(mut self, readable: R) -> Self {
        self.input = InputSource::readable(readable);
        self
    }

    pub fn ended(&self) -> bool {
        self.basic_machine.ended()
    }
    pub fn show_state(&self, historico: Option<&[&[i32]]>) {
        self.basic_machine.show_state(historico);
    }

    pub fn execute_step(&mut self) -> MepaResult<()> {
        let r: MepaResult<Option<i32>> = match self.basic_machine.step_with_input(None) {
            Ok(n) => Ok(n),
            Err(crate::mepa::error::MepaError::MissingInput(_)) => {
                // se causou erro MissingInput uma vez, vou rodar com input
                self.basic_machine.step_with_input(self.input.read())
            }
            Err(e) => Err(e),
        };

        match r {
            Ok(Some(output)) => {
                if let Some(buffer) = &mut self.output {
                    buffer.push(output);
                } else {
                    println!("{}", output);
                }
                Ok(())
            }
            Ok(None) => Ok(()),
            Err(e) => Err(e),
        }
    }
    pub fn current_memory_usage(&self) -> i32 {
        self.basic_machine.current_memory_usage()
    }
    pub fn execute(&mut self) -> MepaResult<()> {
        while !self.ended() {
            self.execute_step()?;
        }
        Ok(())
    }
}
