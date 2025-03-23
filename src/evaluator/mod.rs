use crate::{
    compiler::compile, machine::MepaMachine, mepa::code::MepaCode, otimizador::otimizar_arquivo,
};
use std::path::PathBuf;

struct ExecutionInfo {
    steps: usize,
    max_memory: usize,
    instructions: usize,
    output: Vec<i32>,
}

impl ExecutionInfo {
    pub fn new(filename: &PathBuf, input: Vec<i32>) -> Result<Self, String> {
        let mc: MepaCode = MepaCode::from_file(filename).unwrap();
        let mut info = ExecutionInfo {
            steps: 0,
            max_memory: 0,
            instructions: mc.len(),
            output: Vec::new(),
        };
        let mut machine = MepaMachine::new(mc)
            .add_input_vec(input)
            .add_output(&mut info.output);
        while !machine.ended() {
            machine.execute_step()?;
            info.max_memory = info.max_memory.max(machine.current_memory_usage() as usize);
            info.steps += 1
        }

        Ok(info)
    }
}

pub fn evaluate() {
    let samples_dir = PathBuf::from("samples/ipt");
    if !samples_dir.exists() {
        println!("Samples directory not found!");
        return;
    }

    let work_material = [("acesso-aleatorio", vec![1], vec![20])];

    for (filename, input, expected_output) in work_material.iter() {
        let input_path = samples_dir.join(format!("{}.ipt", filename));
        let output_path = PathBuf::from("output").join(format!("{}.mepa", filename));

        match compile(&input_path, &output_path, false) {
            Ok(_) => {
                let exec_info = ExecutionInfo::new(&output_path, input.clone()).unwrap();
                if exec_info.output != *expected_output {
                    println!("{} failed (original)", filename);
                    println!(
                        "\texpected {:?}, got {:?}",
                        expected_output, exec_info.output
                    );
                    continue;
                }

                otimizar_arquivo(&output_path).unwrap();
                let optimized_exec_info = ExecutionInfo::new(&output_path, input.clone()).unwrap();
                if optimized_exec_info.output != *expected_output {
                    println!("{} failed (optimized)", filename);
                    println!(
                        "\texpected {:?}, got {:?}",
                        expected_output, optimized_exec_info.output
                    );
                    continue;
                }

                println!(
                    "{} passed: {} → {} steps, {} → {} max memory, {} → {} instructions",
                    filename,
                    exec_info.steps,
                    optimized_exec_info.steps,
                    exec_info.max_memory,
                    optimized_exec_info.max_memory,
                    exec_info.instructions,
                    optimized_exec_info.instructions
                );
            }
            Err(e) => println!("Failed to compile {}: {:?}", filename, e),
        }
    }
}
