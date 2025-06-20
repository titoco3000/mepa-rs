use super::{instruction::Instruction, label::Label};
use crate::utils::{matrix_to_string, write_matrix};
use std::fs::{self, File};
use std::io::{self, BufRead};
use std::ops::{Deref, DerefMut};
use std::path::Path;

#[derive(Clone, Debug)]
pub struct MepaCode(pub Vec<(Option<Label>, Instruction)>);

impl Deref for MepaCode {
    type Target = [(Option<Label>, Instruction)];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for MepaCode {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<T> From<T> for MepaCode
where
    T: IntoIterator<Item = Instruction>,
{
    fn from(instructions: T) -> Self {
        // Map each instruction to a tuple with `None` as the label
        let instructions_with_labels = instructions
            .into_iter()
            .map(|instruction| (None, instruction))
            .collect();

        MepaCode(instructions_with_labels)
    }
}

impl MepaCode {
    pub fn with_capacity(capacity: usize) -> MepaCode {
        MepaCode(Vec::with_capacity(capacity))
    }
    pub fn insert(&mut self, new: (Option<Label>, Instruction)) {
        //println!("{:?}",new);
        self.0.push(new);
    }

    pub fn from_file<P>(filename: P) -> io::Result<MepaCode>
    where
        P: AsRef<Path>,
    {
        let delimiters = [',', ' ', '\t', ';', ':'];
        let file = File::open(&filename)?;
        let mut instruction_count = 0;
        for _ in io::BufReader::new(&file).lines().flatten() {
            instruction_count += 1;
        }

        let mut mc = MepaCode(Vec::with_capacity(instruction_count));

        let file = File::open(&filename)?;
        for line in io::BufReader::new(file).lines().flatten() {
            let without_comments = &line[..line
                .find('#')
                .unwrap_or(line.len())
                .min(line.find("//").unwrap_or(line.len()))];

            let line: Vec<&str> = without_comments
                .split(|c| delimiters.contains(&c))
                .filter(|s| !s.is_empty()) // To remove empty strings
                .collect();

            if line.len() > 0 {
                match Instruction::parse(&line) {
                    Ok(value) => mc.insert(value),
                    Err(_) => panic!("Failed to interpret line {:?}", line),
                }
            }
        }

        Ok(mc)
    }

    pub fn remove_instruction(&mut self, index: usize) {
        self.0.remove(index);

        // atualiza todas as instruções que dependem de endereço
        for (_, instruction) in &mut self.0 {
            match instruction {
                Instruction::DSVS(label) | Instruction::DSVF(label) | Instruction::CHPR(label) => {
                    if let Label::Literal(n) = label {
                        if *n > index {
                            // println!("Mudando {} para {}",*n, *n-1);
                            *label = Label::Literal(*n - 1);
                        }
                    }
                }
                _ => {}
            }
        }
    }

    pub fn to_file<P>(self, filename: P) -> io::Result<()>
    where
        P: AsRef<Path>,
    {
        if let Some(parent) = filename.as_ref().parent() {
            fs::create_dir_all(parent)?;
        }

        // Create or open the file
        let file = File::create(filename)?;

        // Write each string to the file, separated by newlines
        let matrix: Vec<Vec<String>> = self
            .0
            .into_iter()
            .map(|line| {
                let mut v = Vec::with_capacity(5);
                v.push(if let Some(label) = line.0 {
                    format!("{}", label)
                } else {
                    "   ".to_string()
                });
                v.append(&mut line.1.to_string_vec());
                v
            })
            .collect();

        write_matrix(&matrix, file)
    }

    pub fn to_string(&self) -> io::Result<String> {
        // First, transform the internal vector of instructions into a
        // matrix of strings, similar to the `to_file` method.
        let matrix: Vec<Vec<String>> = self
            .clone()
            .0
            .into_iter()
            .map(|line| {
                let mut v = Vec::with_capacity(5);
                // Add the label to the row, or a placeholder if it's absent.
                v.push(if let Some(label) = line.0 {
                    format!("{}:", label) // Using a colon for standard assembly style
                } else {
                    String::new() // Using an empty string for better alignment
                });
                // Add the string parts of the instruction itself.
                v.append(&mut line.1.to_string_vec());
                v
            })
            .collect();

        // Use the helper function to format the matrix into a single string.
        // The result is wrapped in `Ok` to match the function's return type.
        Ok(matrix_to_string(&matrix))
    }
}
