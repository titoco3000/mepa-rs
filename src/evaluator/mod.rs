use crate::compiler::CompileError;
use crate::{
    compiler::compile, machine::MepaMachine, mepa::code::MepaCode, otimizador::Otimizador,
};
use std::fmt;
use std::path::PathBuf;

pub struct ExecutionInfo {
    pub steps: usize,
    pub max_memory: usize,
    pub instructions: usize,
    pub output: Vec<i32>,
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

#[derive(Debug, Clone)]
pub enum EvaluationError {
    WrongOutput {
        expected: Vec<i32>,
        resulted: Vec<i32>,
    },
    FailedCompilation(CompileError),
    FailedOptimization,
}
impl fmt::Display for EvaluationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            EvaluationError::WrongOutput { expected, resulted } => {
                write!(
                    f,
                    "Output incorreto: esperava {:?}, obteve {:?}",
                    expected, resulted
                )
            }
            EvaluationError::FailedCompilation(msg) => {
                write!(f, "Falha na compilação: {}", msg)
            }
            &EvaluationError::FailedOptimization => {
                write!(f, "Falha na otimização")
            }
        }
    }
}
impl std::error::Error for EvaluationError {}

pub fn evaluate_samples_with_specified_order(
    order: &[usize],
) -> Vec<(
    String,
    Result<ExecutionInfo, EvaluationError>,
    Result<ExecutionInfo, EvaluationError>,
    usize,
)> {
    let samples_dir = PathBuf::from("samples/ipt");
    if !samples_dir.exists() {
        println!("Samples directory not found!");
        return vec![];
    }

    let work_material = [
        ("acesso-aleatorio", vec![1], vec![20]),
        ("cod-morto", vec![], vec![]),
        ("copia", vec![], vec![12]),
        ("deadvar", vec![13], vec![13]),
        ("global-var", vec![], vec![7]),
        ("indirecao", vec![], vec![]),
        ("inner", vec![], vec![20]),
        ("lifetime", vec![], vec![]),
        ("movimentacao", vec![], vec![0, 1, 2, 3, 4]),
        ("sort", vec![], vec![2, 1, 3, 2, 1, 3, 1, 2, 3]),
    ];

    work_material
        .iter()
        .map(|item| {
            let (filename, input, expected_output) = item;
            let input_path = samples_dir.join(format!("{}.ipt", filename));
            let output_path = PathBuf::from("output").join(format!("{}.mepa", filename));

            match compile(&input_path, &output_path, false) {
                Ok(_) => {
                    let exec_info = ExecutionInfo::new(&output_path, input.clone()).unwrap();
                    if exec_info.output != *expected_output {
                        let error = EvaluationError::WrongOutput {
                            expected: expected_output.to_vec(),
                            resulted: exec_info.output,
                        };
                        (filename.to_string(), Err(error.clone()), Err(error), 0)
                    } else {
                        match Otimizador::from(&output_path).otimizar_com_ordem(order) {
                            Ok(otimizado) => {
                                let etapas_otimizacao = otimizado.etapas_otimizacao;
                                otimizado.save().expect("Falha ao salvar arquivo otimizado");
                                let optimized_exec_info =
                                    ExecutionInfo::new(&output_path, input.clone()).unwrap();
                                if optimized_exec_info.output != *expected_output {
                                    let error = Err(EvaluationError::WrongOutput {
                                        expected: expected_output.to_vec(),
                                        resulted: optimized_exec_info.output,
                                    });
                                    (filename.to_string(), Ok(exec_info), error, 0)
                                } else {
                                    (
                                        filename.to_string(),
                                        Ok(exec_info),
                                        Ok(optimized_exec_info),
                                        etapas_otimizacao,
                                    )
                                }
                            }
                            Err(_e) => (
                                filename.to_string(),
                                Ok(exec_info),
                                Err(EvaluationError::FailedOptimization),
                                0,
                            ),
                        }
                    }
                }
                Err(e) => (
                    filename.to_string(),
                    Err(EvaluationError::FailedCompilation(e.clone())),
                    Err(EvaluationError::FailedCompilation(e)),
                    0,
                ),
            }
        })
        .collect()
}

pub fn evaluate_samples() -> Vec<(
    String,
    Result<ExecutionInfo, EvaluationError>,
    Result<ExecutionInfo, EvaluationError>,
)> {
    let samples_dir = PathBuf::from("samples/ipt");
    if !samples_dir.exists() {
        println!("Samples directory not found!");
        return vec![];
    }

    let work_material = [
        ("acesso-aleatorio", vec![1], vec![20]),
        ("cod-morto", vec![], vec![]),
        ("copia", vec![], vec![12]),
        ("deadvar", vec![13], vec![13]),
        ("global-var", vec![], vec![7]),
        ("indirecao", vec![], vec![]),
        ("inner", vec![], vec![20]),
        ("lifetime", vec![], vec![]),
        ("movimentacao", vec![], vec![0, 1, 2, 3, 4]),
        ("sort", vec![], vec![2, 1, 3, 2, 1, 3, 1, 2, 3]),
    ];

    work_material
        .iter()
        .map(|item| {
            let (filename, input, expected_output) = item;
            let input_path = samples_dir.join(format!("{}.ipt", filename));
            let output_path = PathBuf::from("output").join(format!("{}.mepa", filename));

            match compile(&input_path, &output_path, false) {
                Ok(_) => {
                    let exec_info = ExecutionInfo::new(&output_path, input.clone()).unwrap();
                    if exec_info.output != *expected_output {
                        let error = EvaluationError::WrongOutput {
                            expected: expected_output.to_vec(),
                            resulted: exec_info.output,
                        };
                        (filename.to_string(), Err(error.clone()), Err(error))
                    } else {
                        match Otimizador::from(&output_path).otimizar() {
                            Ok(otimizado) => {
                                otimizado.save().expect("Falha ao salvar arquivo otimizado");
                                let optimized_exec_info =
                                    ExecutionInfo::new(&output_path, input.clone()).unwrap();
                                if optimized_exec_info.output != *expected_output {
                                    let error = Err(EvaluationError::WrongOutput {
                                        expected: expected_output.to_vec(),
                                        resulted: optimized_exec_info.output,
                                    });
                                    (filename.to_string(), Ok(exec_info), error)
                                } else {
                                    (filename.to_string(), Ok(exec_info), Ok(optimized_exec_info))
                                }
                            }
                            Err(_e) => (
                                filename.to_string(),
                                Ok(exec_info),
                                Err(EvaluationError::FailedOptimization),
                            ),
                        }
                    }
                }
                Err(e) => (
                    filename.to_string(),
                    Err(EvaluationError::FailedCompilation(e.clone())),
                    Err(EvaluationError::FailedCompilation(e)),
                ),
            }
        })
        .collect()
}

pub fn print_eval() {
    let eval = evaluate_samples();

    let mut delta_memoria_total = 0.0;
    let mut delta_passos_total = 0.0;
    let mut delta_instruc_total = 0.0;

    for (filename, original, otimizado) in &eval {
        match original {
            Ok(exec_original) => match otimizado {
                Ok(exec_otimizado) => {
                    delta_memoria_total +=
                        exec_otimizado.max_memory as f32 / exec_original.max_memory as f32;
                    delta_passos_total += exec_otimizado.steps as f32 / exec_original.steps as f32;
                    delta_instruc_total +=
                        exec_otimizado.instructions as f32 / exec_original.instructions as f32;
                    println!(
                        "{} otimizado: {} → {} passos, {} → {} max de memoria, {} → {} instruções",
                        filename,
                        exec_original.steps,
                        exec_otimizado.steps,
                        exec_original.max_memory,
                        exec_otimizado.max_memory,
                        exec_original.instructions,
                        exec_otimizado.instructions
                    );
                }
                Err(e) => {
                    println!("Falha no otimizado de {}: {}", filename, e);
                }
            },
            Err(e) => {
                println!("Falha no original de {}: {}", filename, e);
            }
        }
    }
    println!(
        "Diminuição média de passos:     {}%",
        (100.0 * (1.0 - delta_passos_total as f32 / eval.len() as f32)).round()
    );
    println!(
        "Diminuição média de memória:    {}%",
        (100.0 * (1.0 - delta_memoria_total as f32 / eval.len() as f32)).round()
    );
    println!(
        "Diminuição média de instruções: {}%",
        (100.0 * (1.0 - delta_instruc_total as f32 / eval.len() as f32)).round()
    );
}
