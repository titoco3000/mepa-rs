use std::fs::{self, File};
use std::io::{self, Write};
use std::path::PathBuf;

use crate::mepa::label::Label;
use crate::{ensure_is_token, is_token, mepa::instruction::Instruction};

use super::lexic::{Lexic, Token};
use super::SimbolTable;
use crate::mepa::code::MepaCode;

struct Compiler {
    tokens: Lexic,
    simbols: SimbolTable,
    generated_code: MepaCode,
    current_function: Option<String>,
}
impl Compiler {
    pub fn new(file_path: &PathBuf) -> Compiler {
        Compiler {
            tokens: Lexic::new(file_path),
            simbols: SimbolTable::new(),
            generated_code: MepaCode::with_capacity(256),
            current_function: None,
        }
    }

    fn program(&mut self) {
        self.generated_code.insert((None, Instruction::INPP));
        let global_vars = self.declarations();
        while is_token!(self.tokens.next(), Token::Function) {
            self.function_def();
        }

        //after defining the functions, there should be no token left
        if let None = self.tokens.next() {
            self.generated_code.insert((
                None,
                Instruction::CHPR(Label::new(self.simbols.get_fn_label("main").unwrap())),
            ));
            self.generated_code
                .insert((None, Instruction::DMEM(global_vars as i32)));
            self.generated_code.insert((None, Instruction::PARA));
        } else {
            panic!("Extra tokens");
        }
    }

    fn function_def(&mut self) {
        ensure_is_token!(self.tokens.next(), Token::Function);
        self.tokens.consume();
        if let Token::Identifier(id) = self.tokens.consume() {
            self.current_function = Some(id.clone());
            let label_init = Label::new(self.simbols.new_function(id.clone()).unwrap());
            let label_end = Label::new(self.simbols.new_label());
            self.generated_code
                .insert((None, Instruction::DSVS(label_end.clone())));
            self.generated_code
                .insert((Some(label_init), Instruction::ENPR(1)));
            ensure_is_token!(self.tokens.next(), Token::OpenParenthesis);
            self.tokens.consume();
            let parameters = self.parameter_list();
            let l = parameters.len() as i32;
            for (i, p) in parameters.into_iter().enumerate() {
                self.simbols
                    .new_variable(Some(id.clone()), p, i as i32 - (2 + l))
                    .unwrap();
            }
            ensure_is_token!(self.tokens.next(), Token::CloseParenthesis);
            self.tokens.consume();
            ensure_is_token!(self.tokens.next(), Token::OpenBraces);
            self.tokens.consume();

            let local_vars = self.declarations();

            self.commands();

            ensure_is_token!(self.tokens.next(), Token::Return);
            self.tokens.consume();
            self.expression();
            //store at reserved position
            self.generated_code
                .insert((None, Instruction::ARMZ(1, -(3 + l))));

            ensure_is_token!(self.tokens.next(), Token::SemiColon);
            self.tokens.consume();
            ensure_is_token!(self.tokens.next(), Token::CloseBraces);
            self.tokens.consume();
            self.current_function = None;

            self.generated_code
                .insert((None, Instruction::DMEM(local_vars as i32)));
            self.generated_code
                .insert((None, Instruction::RTPR(1, l as i32)));
            self.generated_code
                .insert((Some(label_end), Instruction::NADA));
        } else {
            panic!("Should be identifier");
        }
    }
    fn declarations(&mut self) -> usize {
        let mut v: Vec<String> = Vec::with_capacity(8);
        while is_token!(self.tokens.next(), Token::Int) || is_token!(self.tokens.next(), Token::Ptr)
        {
            v.append(&mut self.declaration());
        }

        let l = v.len();
        self.generated_code
            .insert((None, Instruction::AMEM(l as i32)));
        for (i, s) in v.into_iter().enumerate() {
            self.simbols
                .new_variable(self.current_function.clone(), s, i as i32)
                .unwrap();
        }
        l
    }
    fn commands(&mut self) {
        while is_token!(self.tokens.next(), Token::OpenBraces)
            || is_token!(self.tokens.next(), Token::Identifier(_))
            || is_token!(self.tokens.next(), Token::If)
            || is_token!(self.tokens.next(), Token::While)
            || is_token!(self.tokens.next(), Token::Print)
            || is_token!(self.tokens.next(), Token::Read)
        {
            self.command();
        }
    }
    fn vartype(&mut self) {
        if is_token!(self.tokens.next(), Token::Int) {
            self.tokens.consume();
        } else if is_token!(self.tokens.next(), Token::Ptr) {
            self.tokens.consume();
        } else {
            panic!("no reasonable type");
        }
    }
    fn parameter_list(&mut self) -> Vec<String> {
        let mut v = Vec::with_capacity(8);
        if is_token!(self.tokens.next(), Token::Int) || is_token!(self.tokens.next(), Token::Ptr) {
            self.tokens.consume();
            ensure_is_token!(self.tokens.next(), Token::Identifier(_));
            if let Token::Identifier(s) = self.tokens.consume() {
                v.push(s);
            }

            while is_token!(self.tokens.next(), Token::Comma) {
                self.tokens.consume();
                if is_token!(self.tokens.next(), Token::Int)
                    || is_token!(self.tokens.next(), Token::Ptr)
                {
                    self.tokens.consume();
                } else {
                    panic!("expected type");
                }
                ensure_is_token!(self.tokens.next(), Token::Identifier(_));
                if let Token::Identifier(s) = self.tokens.consume() {
                    v.push(s);
                }
            }
        }
        v
    }
    fn command_block(&mut self) {
        ensure_is_token!(self.tokens.next(), Token::OpenBraces);
        self.tokens.consume();
        self.commands();
        ensure_is_token!(self.tokens.next(), Token::CloseBraces);
        self.tokens.consume();
    }
    fn declaration(&mut self) -> Vec<String> {
        let mut v = Vec::with_capacity(8);
        self.vartype();
        ensure_is_token!(self.tokens.next(), Token::Identifier(_));
        if let Token::Identifier(s) = self.tokens.consume() {
            v.push(s);
        }
        while is_token!(self.tokens.next(), Token::Comma) {
            self.tokens.consume();
            ensure_is_token!(self.tokens.next(), Token::Identifier(_));
            if let Token::Identifier(s) = self.tokens.consume() {
                v.push(s);
            }
        }
        ensure_is_token!(self.tokens.next(), Token::SemiColon);
        self.tokens.consume();
        v
    }
    fn attribuition(&mut self) {
        ensure_is_token!(self.tokens.next(), Token::Identifier(_));
        if let Token::Identifier(s) = self.tokens.consume() {
            let (m, n) = self
                .simbols
                .get_var_addr(&s, self.current_function.clone())
                .unwrap();
            ensure_is_token!(self.tokens.next(), Token::Assign);
            self.tokens.consume();
            self.expression();
            self.generated_code.insert((None, Instruction::ARMZ(m, n)));
        }
    }
    fn expression(&mut self) {
        self.logic_expr();
        while is_token!(self.tokens.next(), Token::Or) {
            self.tokens.consume();
            self.logic_expr();
            self.generated_code.insert((None, Instruction::DISJ));
        }
    }
    fn logic_expr(&mut self) {
        self.relational_expr();
        while is_token!(self.tokens.next(), Token::And) {
            self.tokens.consume();
            self.relational_expr();
            self.generated_code.insert((None, Instruction::CONJ));
        }
    }
    fn relational_expr(&mut self) {
        self.sum();
        if is_token!(self.tokens.next(), Token::LesserThan)
            || is_token!(self.tokens.next(), Token::GraterThan)
            || is_token!(self.tokens.next(), Token::LesserOrEqualThan)
            || is_token!(self.tokens.next(), Token::GreaterOrEqualThan)
            || is_token!(self.tokens.next(), Token::Equals)
            || is_token!(self.tokens.next(), Token::Different)
        {
            let comparison = self.tokens.consume();
            self.sum();
            self.generated_code.insert((
                None,
                match comparison {
                    Token::LesserThan => Instruction::CMME,
                    Token::GraterThan => Instruction::CMMA,
                    Token::LesserOrEqualThan => Instruction::CMEG,
                    Token::GreaterOrEqualThan => Instruction::CMAG,
                    Token::Equals => Instruction::CMIG,
                    Token::Different => Instruction::CMDG,
                    _ => panic!("Impossible result"),
                },
            ));
        }
    }
    fn sum(&mut self) {
        self.factor();
        while is_token!(self.tokens.next(), Token::Plus)
            || is_token!(self.tokens.next(), Token::Minus)
        {
            let op = self.tokens.consume();
            self.factor();
            self.generated_code.insert((
                None,
                match op {
                    Token::Plus => Instruction::SOMA,
                    Token::Minus => Instruction::SUBT,
                    _ => panic!("Impossible result"),
                },
            ));
        }
    }
    fn factor(&mut self) {
        self.operand();
        while is_token!(self.tokens.next(), Token::Asterisc)
            || is_token!(self.tokens.next(), Token::Division)
        {
            let op = self.tokens.consume();
            self.operand();
            self.generated_code.insert((
                None,
                match op {
                    Token::Asterisc => Instruction::MULT,
                    Token::Division => Instruction::DIVI,
                    _ => panic!("Impossible result"),
                },
            ));
        }
    }
    fn command(&mut self) {
        if is_token!(self.tokens.next(), Token::OpenBraces) {
            self.command_block();
        } else if is_token!(self.tokens.next(), Token::Identifier(_)) {
            self.attribuition();
        } else if is_token!(self.tokens.next(), Token::If) {
            self.if_command();
        } else if is_token!(self.tokens.next(), Token::While) {
            self.while_command();
        } else if is_token!(self.tokens.next(), Token::Print) {
            self.print_command();
        } else if is_token!(self.tokens.next(), Token::Read) {
            self.read_command();
        }
        ensure_is_token!(self.tokens.next(), Token::SemiColon);
        self.tokens.consume();
    }
    fn if_command(&mut self) {
        ensure_is_token!(self.tokens.next(), Token::If);
        self.tokens.consume();
        let label_if = Label::new(self.simbols.new_label());

        ensure_is_token!(self.tokens.next(), Token::OpenParenthesis);
        self.tokens.consume();
        self.expression();
        ensure_is_token!(self.tokens.next(), Token::CloseParenthesis);
        self.generated_code
            .insert((None, Instruction::DSVF(label_if.clone())));
        self.tokens.consume();
        self.command();
        if is_token!(self.tokens.next(), Token::Else) {
            let label_else = Label::new(self.simbols.new_label());
            self.generated_code
                .insert((None, Instruction::DSVS(label_else.clone())));
            self.generated_code
                .insert((Some(label_if), Instruction::NADA));
            self.tokens.consume();
            self.command();
            self.generated_code
                .insert((Some(label_else), Instruction::NADA));
        } else {
            self.generated_code
                .insert((Some(label_if), Instruction::NADA));
        }
    }
    fn while_command(&mut self) {
        ensure_is_token!(self.tokens.next(), Token::While);
        self.tokens.consume();
        let label_init = Label::new(self.simbols.new_label());
        let label_end = Label::new(self.simbols.new_label());
        self.generated_code
            .insert((Some(label_init.clone()), Instruction::NADA));
        ensure_is_token!(self.tokens.next(), Token::OpenParenthesis);
        self.tokens.consume();
        self.expression();
        self.generated_code
            .insert((None, Instruction::DSVF(label_end.clone())));
        ensure_is_token!(self.tokens.next(), Token::CloseParenthesis);
        self.tokens.consume();
        self.command();
        self.generated_code
            .insert((None, Instruction::DSVS(label_init)));
        self.generated_code
            .insert((Some(label_end), Instruction::NADA));
    }
    fn read_command(&mut self) {
        ensure_is_token!(self.tokens.next(), Token::Read);
        self.tokens.consume();
        ensure_is_token!(self.tokens.next(), Token::OpenParenthesis);
        self.tokens.consume();
        ensure_is_token!(self.tokens.next(), Token::Identifier(_));
        if let Token::Identifier(s) = self.tokens.consume() {
            let (m, n) = self
                .simbols
                .get_var_addr(&s, self.current_function.clone())
                .unwrap();
            self.generated_code.insert((None, Instruction::LEIT));
            self.generated_code.insert((None, Instruction::ARMZ(m, n)));
        }
        ensure_is_token!(self.tokens.next(), Token::CloseParenthesis);
        self.tokens.consume();
    }
    fn print_command(&mut self) {
        ensure_is_token!(self.tokens.next(), Token::Print);
        self.tokens.consume();
        ensure_is_token!(self.tokens.next(), Token::OpenParenthesis);
        self.tokens.consume();
        let args = self.argument_list();
        for _ in 0..args {
            self.generated_code.insert((None, Instruction::IMPR));
        }
        ensure_is_token!(self.tokens.next(), Token::CloseParenthesis);
        self.tokens.consume();
    }
    fn function_call(&mut self) {
        ensure_is_token!(self.tokens.next(), Token::Identifier(_));
        if let Token::Identifier(s) = self.tokens.consume() {
            let label = Label::new(self.simbols.get_fn_label(&s).unwrap());
            ensure_is_token!(self.tokens.next(), Token::OpenParenthesis);
            self.tokens.consume();
            //reserve a position for return value
            self.generated_code.insert((None, Instruction::AMEM(1)));
            self.argument_list();
            ensure_is_token!(self.tokens.next(), Token::CloseParenthesis);
            self.tokens.consume();
            self.generated_code.insert((None, Instruction::CHPR(label)));
        }
    }
    //retorna quantos foram carregados
    fn argument_list(&mut self) -> usize {
        let mut count = 0;
        if !is_token!(self.tokens.next(), Token::CloseBraces) {
            self.argument();
            count += 1;
        }
        while is_token!(self.tokens.next(), Token::Comma) {
            self.tokens.consume();
            self.argument();
            count += 1;
        }
        count
    }
    fn argument(&mut self) {
        if is_token!(self.tokens.next(), Token::Identifier(_)) {
            if let Token::Identifier(s) = self.tokens.consume() {
                let (m, n) = self
                    .simbols
                    .get_var_addr(&s, self.current_function.clone())
                    .unwrap();
                self.generated_code.insert((None, Instruction::CRVL(m, n)));
            }
        } else {
            self.expression();
        }
    }
    fn operand(&mut self) {
        if is_token!(self.tokens.next(), Token::Identifier(_)) {
            //function call
            if is_token!(self.tokens.next_to_next(), Token::OpenParenthesis) {
                self.function_call();
            } else {
                //identifier
                if let Token::Identifier(s) = self.tokens.consume() {
                    let (m, n) = self
                        .simbols
                        .get_var_addr(&s, self.current_function.clone())
                        .unwrap();
                    self.generated_code.insert((None, Instruction::CRVL(m, n)));
                }
            }
        } else if is_token!(self.tokens.next(), Token::Number(_)) {
            if let Token::Number(n) = self.tokens.consume() {
                self.generated_code.insert((None, Instruction::CRCT(n)));
            }
        } else if is_token!(self.tokens.next(), Token::OpenParenthesis) {
            self.tokens.consume();
            self.expression();
            ensure_is_token!(self.tokens.next(), Token::CloseParenthesis);
            self.tokens.consume();
        } else if is_token!(self.tokens.next(), Token::Minus) {
            self.tokens.consume();
            self.generated_code.insert((None, Instruction::INVR));
            self.operand();
        } else if is_token!(self.tokens.next(), Token::Not) {
            self.tokens.consume();
            self.generated_code.insert((None, Instruction::NEGA));
            self.operand();
        } else if is_token!(self.tokens.next(), Token::AddressOf) {
            self.tokens.consume();
            ensure_is_token!(self.tokens.next(), Token::Identifier(_));
            if let Token::Identifier(s) = self.tokens.consume() {
                let (m, n) = self
                    .simbols
                    .get_var_addr(&s, self.current_function.clone())
                    .unwrap();
                self.generated_code.insert((None, Instruction::CREN(m, n)));
            }
        } else if is_token!(self.tokens.next(), Token::Asterisc) {
            self.tokens.consume();
            ensure_is_token!(self.tokens.next(), Token::Identifier(_));
            if let Token::Identifier(s) = self.tokens.consume() {
                let (m, n) = self
                    .simbols
                    .get_var_addr(&s, self.current_function.clone())
                    .unwrap();
                self.generated_code.insert((None, Instruction::CRVI(m, n)));
            }
        }
    }
}

pub fn compile(origin: &PathBuf, target: &PathBuf) -> io::Result<()> {
    let mut c = Compiler::new(origin);
    c.program();
    if let Some(parent) = target.parent() {
        fs::create_dir_all(parent)?;
    }

    // Create or open the file
    let file = File::create(&target)?;

    // Write each string to the file, separated by newlines
    let matrix: Vec<Vec<String>> = c
        .generated_code
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

fn write_matrix(matrix: &Vec<Vec<String>>, file: File) -> io::Result<()> {
    // Calculate the maximum width for each column
    let mut max_widths: Vec<usize> = Vec::with_capacity(matrix[0].len());

    for row in matrix {
        for (i, item) in row.iter().enumerate() {
            let item_len = format!("{}", item).len();
            if max_widths.get(i).is_some() {
                max_widths[i] = max_widths[i].max(item_len);
            } else {
                max_widths.push(item_len);
            }
        }
    }

    // Write the matrix with proper alignment
    for row in matrix {
        for (i, item) in row.iter().enumerate() {
            let width = max_widths.get(i).unwrap_or(&0);
            write!(&file, "{:width$} ", item, width = width)?;
        }
        writeln!(&file)?;
    }
    Ok(())
}
