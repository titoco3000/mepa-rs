use std::fs::File;
use std::io::{self, BufRead};
use std::path::Path;
use std::usize;
use super::instruction::Instruction;
use super::label::Label;

pub struct MepaCode(pub Vec<(Option<Label>, Instruction)>);

impl MepaCode {
    pub fn with_capacity(capacity: usize)->MepaCode{
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
            
            let without_comments = &line[..line.find('#').unwrap_or(line.len()).min(line.find("//").unwrap_or(line.len()))];
            
            let line: Vec<&str> = without_comments
                .split(|c| delimiters.contains(&c))
                .filter(|s| !s.is_empty()) // To remove empty strings
                .collect();

            if line.len() > 0 {
                match Instruction::parse(&line) {
                    Ok(value) =>   mc.insert(value),
                    Err(_) => panic!("Failed to interpret line {:?}",line),
                }
            }
        }

        Ok(mc)
    }
}