use crate::mepa::code::MepaCode;
use crate::mepa::instruction::Instruction;
use crate::utils::{input_i32, print_matrix};

use std::{
    io::{self, BufReader, Read},
    path::PathBuf,
};

enum InputSource {
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
    code: MepaCode,
    m: Vec<i32>,                      //main memory
    d: Vec<i32>,                      //auxiliary
    i: usize,                         //next instruction
    s: i32,                           //next memory addr
    input: InputSource,               // input source
    output: Option<&'a mut Vec<i32>>, // output (if none, will print to stdout)
}

impl<'a> MepaMachine<'a> {
    pub fn new(code: MepaCode) -> MepaMachine<'a> {
        let mut m = Vec::with_capacity(1024);
        let d = vec![-1; 256];
        unsafe {
            m.set_len(m.capacity());
        }
        MepaMachine {
            code,
            m,
            d,
            i: 0,
            s: -1,
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
        if let Instruction::PARA = self.code.0[self.i].1 {
            true
        } else {
            false
        }
    }
    pub fn show_state(&self, historico: Option<&[&[i32]]>) {
        let code_len = self.code.0.len();

        let s = (self.s + 1) as usize;

        let max_atingido_m = s.max(historico.iter().map(|v| v.len()).max().unwrap_or(0));

        let mut matrix = Vec::with_capacity(code_len + 2);
        matrix.push(vec![
            "".to_owned(),
            "i".to_owned(),
            "".to_owned(),
            "".to_owned(),
            "".to_owned(),
            "".to_owned(),
            "D".to_owned(),
            "M".to_owned(),
        ]);
        for j in 0..10.max(max_atingido_m) {
            let i = j + self.i as usize;
            let mut v = vec![
                (if i == self.i as usize { ">" } else { "" }).to_string(),
                i.to_string(),
            ];
            if i < self.code.0.len() {
                v.append(&mut self.code.0[i].1.to_string_vec());
            }
            while v.len() < 6 {
                v.push("".to_string());
            }

            v.push(if let Some(value) = self.d.get(j) {
                value.to_string()
            } else {
                "".to_string()
            });
            v.push(if j < s {
                self.m[j].to_string()
            } else {
                "".to_string()
            });

            matrix.push(v);
        }

        print_matrix(&matrix);
        println!("");
    }
    pub fn execute_step(&mut self) -> Result<(), &str> {
        if let Some(code) = self.code.0.get(self.i) {
            match &code.1 {
                Instruction::CRCT(k) => {
                    self.s += 1;
                    self.m[self.s as usize] = *k;
                    self.i += 1;
                }
                Instruction::CRVL(m, n) => {
                    self.s += 1;
                    self.m[self.s as usize] = self.m[(self.d[*m as usize] + n) as usize];
                    self.i += 1;
                }
                Instruction::CREN(m, n) => {
                    self.s += 1;
                    self.m[self.s as usize] = self.d[*m as usize] + n;
                    self.i += 1;
                }
                Instruction::ARMZ(m, n) => {
                    self.m[(self.d[*m as usize] + n) as usize] = self.m[self.s as usize];
                    self.s -= 1;
                    self.i += 1;
                }
                Instruction::CRVI(m, n) => {
                    self.s += 1;
                    self.m[self.s as usize] =
                        self.m[self.m[(self.d[*m as usize] + n) as usize] as usize];
                    self.i += 1;
                }
                Instruction::ARMI(m, n) => {
                    let temp = self.m[(self.d[*m as usize] + n) as usize] as usize;
                    self.m[temp] = self.m[self.s as usize];
                    self.s -= 1;
                    self.i += 1;
                }
                Instruction::SOMA => {
                    self.m[self.s as usize - 1] =
                        self.m[self.s as usize - 1] + self.m[self.s as usize];
                    self.s -= 1;
                    self.i += 1;
                }
                Instruction::SUBT => {
                    self.m[self.s as usize - 1] =
                        self.m[self.s as usize - 1] - self.m[self.s as usize];
                    self.s -= 1;
                    self.i += 1;
                }
                Instruction::MULT => {
                    self.m[self.s as usize - 1] =
                        self.m[self.s as usize - 1] * self.m[self.s as usize];
                    self.s -= 1;
                    self.i += 1;
                }
                Instruction::DIVI => {
                    self.m[self.s as usize - 1] =
                        self.m[self.s as usize - 1] / self.m[self.s as usize];
                    self.s -= 1;
                    self.i += 1;
                }
                Instruction::INVR => {
                    self.m[self.s as usize] = -self.m[self.s as usize];
                    self.i += 1
                }
                Instruction::CONJ => {
                    self.m[self.s as usize - 1] =
                        if self.m[self.s as usize - 1] != 0 && self.m[self.s as usize] != 0 {
                            1
                        } else {
                            0
                        };
                    self.s -= 1;
                    self.i += 1
                }
                Instruction::DISJ => {
                    self.m[self.s as usize - 1] =
                        if self.m[self.s as usize - 1] != 0 || self.m[self.s as usize] != 0 {
                            1
                        } else {
                            0
                        };
                    self.s -= 1;
                    self.i += 1
                }
                Instruction::NEGA => {
                    self.m[self.s as usize] = if self.m[self.s as usize] == 0 { 1 } else { 0 };
                    self.i += 1
                }
                Instruction::CMME => {
                    self.m[self.s as usize - 1] =
                        if self.m[self.s as usize - 1] < self.m[self.s as usize] {
                            1
                        } else {
                            0
                        };
                    self.s -= 1;
                    self.i += 1
                }
                Instruction::CMMA => {
                    self.m[self.s as usize - 1] =
                        if self.m[self.s as usize - 1] > self.m[self.s as usize] {
                            1
                        } else {
                            0
                        };
                    self.s -= 1;
                    self.i += 1
                }
                Instruction::CMIG => {
                    self.m[self.s as usize - 1] =
                        if self.m[self.s as usize - 1] == self.m[self.s as usize] {
                            1
                        } else {
                            0
                        };
                    self.s -= 1;
                    self.i += 1
                }
                Instruction::CMDG => {
                    self.m[self.s as usize - 1] =
                        if self.m[self.s as usize - 1] != self.m[self.s as usize] {
                            1
                        } else {
                            0
                        };
                    self.s -= 1;
                    self.i += 1
                }
                Instruction::CMEG => {
                    self.m[self.s as usize - 1] =
                        if self.m[self.s as usize - 1] <= self.m[self.s as usize] {
                            1
                        } else {
                            0
                        };
                    self.s -= 1;
                    self.i += 1
                }
                Instruction::CMAG => {
                    self.m[self.s as usize - 1] =
                        if self.m[self.s as usize - 1] >= self.m[self.s as usize] {
                            1
                        } else {
                            0
                        };
                    self.s -= 1;
                    self.i += 1
                }
                Instruction::DSVS(p) => self.i = p.locate(&self.code).unwrap(),
                Instruction::DSVF(p) => {
                    if self.m[self.s as usize] == 0 {
                        self.i = p.locate(&self.code).unwrap()
                    } else {
                        self.i += 1
                    }
                    self.s -= 1;
                }
                Instruction::NADA => self.i += 1,
                Instruction::PARA => (),
                Instruction::LEIT => {
                    self.s += 1;
                    self.m[self.s as usize] = self.input.read().expect("Erro no input");
                    self.i += 1;
                }
                Instruction::IMPR => {
                    if let Some(output) = &mut self.output {
                        output.push(self.m[self.s as usize]);
                    } else {
                        println!("{}", self.m[self.s as usize])
                    }
                    self.s -= 1;
                    self.i += 1;
                }
                Instruction::AMEM(n) => {
                    self.s += n;
                    self.i += 1;
                }
                Instruction::DMEM(n) => {
                    self.s -= n;
                    self.i += 1;
                }
                Instruction::INPP => {
                    self.s = -1;
                    self.d[0] = 0;
                    self.i = 1;
                }
                Instruction::CHPR(p) => {
                    self.s += 1;
                    self.m[self.s as usize] = self.i as i32 + 1;
                    self.i = p.locate(&self.code).unwrap();
                }
                Instruction::ENPR(k) => {
                    self.s += 1;
                    self.m[self.s as usize] = self.d[*k as usize];
                    self.d[*k as usize] = self.s + 1;
                    self.i += 1;
                }
                Instruction::RTPR(k, n) => {
                    self.d[*k as usize] = self.m[self.s as usize];
                    self.i = self.m[self.s as usize - 1] as usize;
                    self.s -= n + 2;
                }
            }
            Ok(())
        } else {
            Err("End of instructions without PARA")
        }
    }
    pub fn current_memory_usage(&self) -> i32 {
        self.s
    }
    pub fn execute(&mut self) -> Result<(), String> {
        while !self.ended() {
            self.execute_step()?;
        }
        Ok(())
    }
}

pub fn interactive_execution(filename: &PathBuf, input: Vec<i32>) {
    let mc = MepaCode::from_file(filename).unwrap();
    let mut machine = MepaMachine::new(mc);
    if input.len() > 0 {
        machine = machine.add_input_vec(input);
    }
    let mut input_line = String::new();

    while !machine.ended() {
        machine.show_state(None);
        machine.execute_step().unwrap();
        io::stdin()
            .read_line(&mut input_line)
            .expect("Failed to read line");
    }
    machine.show_state(None);
    println!("Program executed successfully");
}

pub fn execute(filename: &PathBuf, input: Vec<i32>, output: Option<&mut Vec<i32>>) {
    let mc = MepaCode::from_file(filename).unwrap();
    let mut machine = MepaMachine::new(mc);
    if input.len() > 0 {
        machine = machine.add_input_vec(input);
    }
    if let Some(output) = output {
        machine = machine.add_output(output);
    }
    machine.execute().unwrap();
}
