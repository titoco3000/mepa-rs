use mepa_rs::{
    compiler::{compile, CompileError},
    evaluator::print_eval,
    machine,
    otimizador::{genetico::encontrar_ordem_otimizacao, Otimizador},
};

use clap::{Arg, Command};
use std::{fs, path::PathBuf};

const DEBUG: bool = true;

fn main() {
    if DEBUG {
        encontrar_ordem_otimizacao();
    } else {
        let matches = Command::new("MepaC")
            .about("A compiler and MEPA execution tool")
            .arg(
                Arg::new("action")
                    .required(true)
                    .value_parser(["compile", "run", "debug", "optimize", "evaluate"])
                    .help("Action to perform (compile, run, debug, optimize or evaluate)"),
            )
            .arg(
                Arg::new("input")
                    .required(false) // Make it optional
                    .help("Input file for the action"),
            )
            .arg(
                Arg::new("output")
                    .short('o')
                    .help("Optional output file for the compilation"),
            )
            .arg(
                Arg::new("optimize")
                    .long("optimize")
                    .action(clap::ArgAction::SetTrue)
                    .help("Optimize the program after compilation"),
            )
            .arg(
                Arg::new("run")
                    .long("run")
                    .action(clap::ArgAction::SetTrue)
                    .help("Run the program after compilation"),
            )
            .arg(
                Arg::new("debug")
                    .long("debug")
                    .action(clap::ArgAction::SetTrue)
                    .help("Test the program interactively"),
            )
            .arg(
                Arg::new("input_values")
                    .long("input")
                    .help("Comma-separated input values for execution")
                    .value_delimiter(','),
            )
            .get_matches();

        let action = matches.get_one::<String>("action").unwrap();

        let input_path = matches
            .get_one::<String>("input")
            .map(|s| Some(PathBuf::from(s)))
            .unwrap_or_else(|| None);

        let output_path = matches
            .get_one::<String>("output")
            .map(|s| Some(PathBuf::from(s)))
            .unwrap_or_else(|| None);

        let input_values: Vec<i32> = matches
            .get_many::<String>("input_values")
            .map(|vals| vals.map(|s| s.parse().unwrap()).collect::<Vec<i32>>())
            .unwrap_or_else(Vec::new);

        let should_run = *matches.get_one::<bool>("run").unwrap_or(&false);
        let should_debug = *matches.get_one::<bool>("debug").unwrap_or(&false);
        let should_optimize = *matches.get_one::<bool>("optimize").unwrap_or(&false);

        if let Some(input_path) = input_path {
            // Handle directory or file input
            if input_path.is_dir() {
                let entries = fs::read_dir(&input_path).unwrap();
                for entry in entries {
                    let entry = entry.unwrap();
                    let file_path = entry.path();
                    if file_path.is_file() {
                        let p = match &output_path {
                            Some(p) => p.clone(),
                            None => {
                                let mut p = PathBuf::from("output");
                                p.push(format!(
                                    "{}.mepa",
                                    file_path.file_stem().unwrap().to_str().unwrap()
                                ));
                                p
                            }
                        };
                        handle_action(
                            action,
                            &file_path,
                            &p,
                            should_run,
                            should_debug,
                            should_optimize,
                            &input_values,
                        );
                    }
                }
            } else {
                let p = match output_path {
                    Some(p) => p,
                    None => {
                        let mut p = PathBuf::from("output");
                        p.push(format!(
                            "{}.mepa",
                            input_path.file_stem().unwrap().to_str().unwrap()
                        ));
                        p
                    }
                };
                handle_action(
                    action,
                    &input_path,
                    &p,
                    should_run,
                    should_debug,
                    should_optimize,
                    &input_values,
                );
            }
        } else if action != "evaluate" {
            eprintln!("Error: The 'input' argument is required for '{}'.", action);
            std::process::exit(1);
        } else {
            print_eval();
        }
    }
}

fn handle_action(
    action: &str,
    input_path: &PathBuf,
    output_path: &PathBuf,
    should_run: bool,
    should_debug: bool,
    should_optimize: bool,
    input_values: &[i32],
) {
    match action {
        "compile" => {
            let output = if output_path.is_dir() {
                output_path
                    .join(input_path.file_stem().unwrap())
                    .with_extension("mepa")
            } else {
                output_path.clone()
            };
            println!("compilando {:?}", input_path.file_name().unwrap());
            match compile(input_path, &output, should_optimize) {
                Ok(r) => match r {
                    Ok(_) => {
                        if should_debug {
                            machine::interactive_execution(&output_path, input_values.to_vec());
                        } else if should_run {
                            machine::execute(&output_path, input_values.to_vec(), None);
                        }
                    }
                    Err(e) => println!("Erro de IO: {:?}", e),
                },
                Err(e) => match e {
                    CompileError::Lexic(s) => println!("Erro léxico: {}", s),
                    CompileError::Sintatic(s) => println!("Erro sintático: {}", s),
                    CompileError::Semantic(s) => println!("Erro semântico: {}", s),
                },
            }
        }
        "optimize" => {
            Otimizador::from(input_path)
                .otimizar()
                .expect("Não foi possível otimizar o arquivo")
                .save()
                .expect("Erro ao salvar arquivo otimizado");
        }
        "run" => {
            machine::execute(input_path, input_values.to_vec(), None);
        }
        "debug" => {
            machine::interactive_execution(input_path, input_values.to_vec());
        }
        _ => unreachable!(),
    }
}
