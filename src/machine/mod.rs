mod basic_machine;
mod full_machine;

use crate::mepa::code::MepaCode;
pub use full_machine::MepaMachine;

use std::path::PathBuf;

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
        std::io::stdin()
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
