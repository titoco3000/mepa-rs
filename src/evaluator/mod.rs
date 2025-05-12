use crate::{
    compiler::compile, machine::MepaMachine, mepa::code::MepaCode, otimizador::Otimizador,
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
            info.max_memory = info
                .max_memory
                .max(machine.current_memory_usage().max(0) as usize);
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

    let work_material = [
        // ("elementares/acesso-aleatorio", vec![1], vec![20]),
        // ("elementares/cod-morto", vec![], vec![]),
        // ("elementares/copia", vec![], vec![12]),
        // ("elementares/deadvar", vec![13], vec![13]),
        // ("elementares/global-var", vec![], vec![7]),
        // ("elementares/indirecao", vec![], vec![]),
        // ("elementares/inner", vec![], vec![20]),
        // ("elementares/lifetime", vec![], vec![]),
        // ("elementares/movimentacao", vec![], vec![0, 1, 2, 3, 4]),
        ("algoritmos/binary_search", vec![], vec![4]),
        (
            "algoritmos/bubble_sort",
            vec![],
            vec![2, 1, 3, 7, 1, 2, 1, 3, 7, 1, 1, 1, 2, 3, 7],
        ),
        (
            "algoritmos/decimal_to_binary",
            vec![123],
            vec![1, 1, 0, 1, 1, 1, 1],
        ),
        (
            "algoritmos/euler_spigot",
            vec![],
            vec![
                2, 7, 1, 8, 2, 8, 1, 8, 2, 8, 4, 5, 9, 0, 4, 5, 2, 3, 5, 3, 6, 0, 2, 8, 7, 4, 7, 1,
                3, 5, 2, 6, 6, 2, 4, 9, 7, 7, 5, 7, 2, 4, 7, 0, 9, 3, 6, 9, 9, 9, 5, 9, 5, 7, 4, 9,
                6, 6, 9, 6, 7, 6, 2, 7, 7, 2, 4, 0, 7, 6, 6, 3, 0, 3, 5, 3, 5, 4, 7, 5, 9, 4, 5, 7,
                1, 3, 8, 2, 1, 7, 8, 5, 2, 5, 1, 6, 6, 4, 2, 7, 4,
            ],
        ),
        ("algoritmos/fibonacci", vec![30], vec![832040]),
        //("algoritmos/floyd_warshall", vec![], vec![]),
        (
            "algoritmos/merge_sort",
            vec![],
            vec![38, 27, 43, 10, 3, 3, 10, 27, 38, 43],
        ),
        ("algoritmos/quicksort", vec![], vec![1, 5, 7, 8, 9, 10]),
        (
            "algoritmos/rng",
            vec![],
            vec![306, 437, 4554, 4199, 2142, 3119, 4584, 4517, 654, 2435],
        ),
        (
            "algoritmos/tower_of_hanoi",
            vec![],
            vec![
                1, 1, 2, 2, 1, 3, 1, 2, 3, 3, 1, 2, 1, 3, 1, 2, 3, 2, 1, 1, 2,
            ],
        ),
    ];

    let mut sum_reduc_steps = 0.0;
    let mut sum_reduc_instructs = 0.0;
    let mut sum_reduc_memory = 0.0;

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
                match Otimizador::from(&output_path).otimizar() {
                    Ok(otimizado) => {
                        otimizado.save().expect("Falha ao salvar arquivo otimizado");
                        let optimized_exec_info =
                            ExecutionInfo::new(&output_path, input.clone()).unwrap();
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
                        sum_reduc_steps +=
                            1.0 - (optimized_exec_info.steps as f32 / exec_info.steps as f32);
                        sum_reduc_instructs += 1.0
                            - (optimized_exec_info.instructions as f32
                                / exec_info.instructions as f32);
                        sum_reduc_memory += 1.0
                            - (optimized_exec_info.max_memory as f32 / exec_info.max_memory as f32);
                    }
                    Err(e) => {
                        println!("{} falhou: {:?}", filename, e)
                    }
                }
            }
            Err(e) => println!("Failed to compile {}: {:?}", filename, e),
        }
    }
    println!(
        "Reduções médias: {}% steps, {}% max memory, {}% instructions",
        (sum_reduc_steps / work_material.len() as f32 * 100.0).round(),
        (sum_reduc_memory / work_material.len() as f32 * 100.0).round(),
        (sum_reduc_instructs / work_material.len() as f32 * 100.0).round(),
    );
}
