use std::io;
use std::path::PathBuf;

use crate::mepa::label::Label;
use crate::otimizador::{self, Otimizador};
use crate::{ensure_is_token, is_token, mepa::instruction::Instruction};

use super::error::CompileError;
use super::lexic::{Lexic, Token};
use super::simbol_table::{SimbolTable, VarType, Variable};
use crate::mepa::code::MepaCode;

struct Compiler {
    tokens: Lexic,
    simbols: SimbolTable,
    generated_code: MepaCode,
    current_function: Option<String>,
}
impl Compiler {
    pub fn new(file_path: &PathBuf) -> Result<Compiler, CompileError> {
        Ok(Compiler {
            tokens: Lexic::new(file_path)?,
            simbols: SimbolTable::new(),
            generated_code: MepaCode::with_capacity(256),
            current_function: None,
        })
    }

    fn program(&mut self) -> Result<(), CompileError> {
        self.generated_code.insert((None, Instruction::INPP));
        let global_vars = self.declarations()?;
        while is_token!(self.tokens.next(), Token::Function) {
            self.function_def()?;
        }

        //after defining the functions, there should be no token left
        if let None = self.tokens.next() {
            self.generated_code.insert((None, Instruction::AMEM(1)));
            self.generated_code.insert((
                None,
                Instruction::CHPR(Label::new(self.simbols.get_fn_label("main").ok_or_else(
                    || CompileError::Semantic(format!("Função 'main' não encontrada")),
                )?)),
            ));
            //libera as variaveis globais + vars reservada para offset + variavel de retorno da main
            self.generated_code
                .insert((None, Instruction::DMEM(global_vars as i32 + 3)));
            self.generated_code.insert((None, Instruction::PARA));
            Ok(())
        } else {
            Err(CompileError::Sintatic(
                "Tokens extras depois do final do programa".to_owned(),
            ))
        }
    }

    fn function_def(&mut self) -> Result<(), CompileError> {
        ensure_is_token!(
            self.tokens.next(),
            Token::Function,
            self.tokens.current_line()
        );
        self.tokens.consume()?;
        if let Token::Identifier(id) = self.tokens.consume()? {
            self.current_function = Some(id.clone());
            let label_init = Label::new(self.simbols.new_function(id.clone())?);
            let label_end = Label::new(self.simbols.new_label());
            self.generated_code
                .insert((None, Instruction::DSVS(label_end.clone())));
            self.generated_code
                .insert((Some(label_init), Instruction::ENPR(1)));
            ensure_is_token!(
                self.tokens.next(),
                Token::OpenParenthesis,
                self.tokens.current_line()
            );
            self.tokens.consume()?;
            let parameters = self.parameter_list()?;
            let l = parameters.len() as i32;
            for (i, (_, name)) in parameters.into_iter().enumerate() {
                self.simbols
                    .new_variable(Some(id.clone()), Variable::new(name, i as i32 - (2 + l)))?;
            }
            ensure_is_token!(
                self.tokens.next(),
                Token::CloseParenthesis,
                self.tokens.current_line()
            );
            self.tokens.consume()?;
            ensure_is_token!(
                self.tokens.next(),
                Token::OpenBraces,
                self.tokens.current_line()
            );
            self.tokens.consume()?;

            let local_vars = self.declarations()?;

            self.commands()?;

            if is_token!(self.tokens.next(), Token::Return) {
                self.tokens.consume()?;
                self.expression()?;
                ensure_is_token!(
                    self.tokens.next(),
                    Token::SemiColon,
                    self.tokens.current_line()
                );
                self.tokens.consume()?;
            } else {
                self.generated_code.insert((None, Instruction::CRCT(0)));
            }
            //store at reserved return position
            self.generated_code
                .insert((None, Instruction::ARMZ(1, -(3 + l))));

            ensure_is_token!(
                self.tokens.next(),
                Token::CloseBraces,
                self.tokens.current_line()
            );
            self.tokens.consume()?;
            self.current_function = None;

            self.generated_code
                .insert((None, Instruction::DMEM(local_vars as i32 + 2)));
            self.generated_code
                .insert((None, Instruction::RTPR(1, l as i32)));
            self.generated_code
                .insert((Some(label_end), Instruction::NADA));
            Ok(())
        } else {
            unreachable!()
        }
    }
    fn declarations(&mut self) -> Result<usize, CompileError> {
        let mut v = Vec::with_capacity(8);
        while is_token!(self.tokens.next(), Token::Int) || is_token!(self.tokens.next(), Token::Ptr)
        {
            v.append(&mut self.declaration()?);
        }

        let l: usize = (v.iter().map(|(_, _, value)| value).sum::<i32>()) as usize;
        //reserva duas variaveis para  calculo de offsets de array: uma para lvalue e outra para rvalue
        self.generated_code
            .insert((None, Instruction::AMEM(l as i32 + 2)));
        let mut acumulator = 2;
        for (var_type, name, size) in v.into_iter() {
            self.simbols.new_variable(
                self.current_function.clone(),
                Variable::new(name, acumulator),
            )?;
            if let VarType::Array = var_type {
                self.generated_code.insert((
                    None,
                    Instruction::CREN(
                        if self.current_function.is_none() {
                            0
                        } else {
                            1
                        },
                        acumulator + 1,
                    ),
                ));
                self.generated_code.insert((
                    None,
                    Instruction::ARMZ(
                        if self.current_function.is_none() {
                            0
                        } else {
                            1
                        },
                        acumulator,
                    ),
                ));
            }
            acumulator += size;
        }
        Ok(l)
    }
    fn commands(&mut self) -> Result<(), CompileError> {
        while is_token!(self.tokens.next(), Token::OpenBraces)
            || is_token!(self.tokens.next(), Token::Identifier(_))
            || is_token!(self.tokens.next(), Token::Asterisc)
            || is_token!(self.tokens.next(), Token::If)
            || is_token!(self.tokens.next(), Token::While)
            || is_token!(self.tokens.next(), Token::Print)
            || is_token!(self.tokens.next(), Token::Read)
        {
            self.command()?;
        }
        Ok(())
    }
    fn vartype(&mut self) -> Result<VarType, CompileError> {
        match self.tokens.consume()? {
            Token::Int => Ok(VarType::Int),
            Token::Ptr => Ok(VarType::Ptr),
            token => Err(CompileError::Sintatic(format!(
                "Esperava um tipo, obteve {:?} na linha {}",
                token,
                self.tokens.current_line()
            ))), //Deveria ser {} aqui! Consertar depois
        }
    }
    fn parameter_list(&mut self) -> Result<Vec<(VarType, String)>, CompileError> {
        let mut v = Vec::with_capacity(8);
        if is_token!(self.tokens.next(), Token::Int) || is_token!(self.tokens.next(), Token::Ptr) {
            let var_type = match self.tokens.consume()? {
                Token::Int => VarType::Int,
                Token::Ptr => VarType::Ptr,
                _ => panic!(),
            };
            ensure_is_token!(
                self.tokens.next(),
                Token::Identifier(_),
                self.tokens.current_line()
            );
            if let Token::Identifier(s) = self.tokens.consume()? {
                v.push((var_type, s));
            }

            while is_token!(self.tokens.next(), Token::Comma) {
                self.tokens.consume()?;
                if is_token!(self.tokens.next(), Token::Int)
                    || is_token!(self.tokens.next(), Token::Ptr)
                {
                    self.tokens.consume()?;
                } else {
                    panic!("expected type");
                }
                ensure_is_token!(
                    self.tokens.next(),
                    Token::Identifier(_),
                    self.tokens.current_line()
                );
                if let Token::Identifier(s) = self.tokens.consume()? {
                    v.push((var_type, s));
                }
            }
        }
        Ok(v)
    }
    fn command_block(&mut self) -> Result<(), CompileError> {
        ensure_is_token!(
            self.tokens.next(),
            Token::OpenBraces,
            self.tokens.current_line()
        );
        self.tokens.consume()?;
        self.commands()?;
        ensure_is_token!(
            self.tokens.next(),
            Token::CloseBraces,
            self.tokens.current_line()
        );
        self.tokens.consume()?;
        Ok(())
    }
    fn declaration(&mut self) -> Result<Vec<(VarType, String, i32)>, CompileError> {
        let mut v = Vec::with_capacity(8);
        let var_type = self.vartype()?;
        let mut need_to_find_identifier = true;
        while need_to_find_identifier {
            ensure_is_token!(
                self.tokens.next(),
                Token::Identifier(_),
                self.tokens.current_line()
            );
            if let Token::Identifier(s) = self.tokens.consume()? {
                let (size, is_array) = if is_token!(self.tokens.next(), Token::OpenBrackets) {
                    self.tokens.consume()?;
                    ensure_is_token!(
                        self.tokens.next(),
                        Token::Number(_),
                        self.tokens.current_line()
                    );
                    if let Token::Number(n) = self.tokens.consume()? {
                        ensure_is_token!(
                            self.tokens.next(),
                            Token::CloseBrackets,
                            self.tokens.current_line()
                        );
                        self.tokens.consume()?;
                        (n + 1, true)
                    } else {
                        panic!() //impossivel de chegar aqui
                    }
                } else {
                    (1, false)
                };
                v.push((if is_array { VarType::Array } else { var_type }, s, size));
            }
            need_to_find_identifier = is_token!(self.tokens.next(), Token::Comma);
            if need_to_find_identifier {
                self.tokens.consume()?;
            }
        }
        ensure_is_token!(
            self.tokens.next(),
            Token::SemiColon,
            self.tokens.current_line()
        );
        self.tokens.consume()?;
        Ok(v)
    }
    fn attribuition(&mut self) -> Result<(), CompileError> {
        let is_indirect_assignment = if is_token!(self.tokens.next(), Token::Asterisc) {
            self.tokens.consume()?;
            true
        } else {
            false
        };

        ensure_is_token!(
            self.tokens.next(),
            Token::Identifier(_),
            self.tokens.current_line()
        );
        if let Token::Identifier(s) = self.tokens.consume()? {
            let (m, n) = self
                .simbols
                .get_var_addr_and_type(&s, self.current_function.clone())
                .ok_or_else(|| {
                    CompileError::Semantic(format!(
                        "Variavel `{}` não foi declarada neste escopo, na linha {}",
                        s,
                        self.tokens.current_line()
                    ))
                })?;
            if is_token!(self.tokens.next(), Token::OpenBrackets) {
                self.tokens.consume()?;

                self.generated_code.insert((None, Instruction::CRVL(m, n)));
                self.expression()?;
                self.generated_code.insert((None, Instruction::SOMA));
                self.generated_code.insert((
                    None,
                    Instruction::ARMZ(
                        if self.current_function.is_none() {
                            0
                        } else {
                            1
                        },
                        1, //guarda endereço na pos reservada para lvalue
                    ),
                ));

                if is_indirect_assignment {
                    self.generated_code.insert((
                        None,
                        Instruction::CRVI(
                            if self.current_function.is_none() {
                                0
                            } else {
                                1
                            },
                            1, //guarda endereço na pos reservada para lvalue
                        ),
                    ));
                    self.generated_code.insert((
                        None,
                        Instruction::ARMZ(
                            if self.current_function.is_none() {
                                0
                            } else {
                                1
                            },
                            1, //guarda endereço na pos reservada para lvalue
                        ),
                    ));
                }

                ensure_is_token!(
                    self.tokens.next(),
                    Token::CloseBrackets,
                    self.tokens.current_line()
                );
                self.tokens.consume()?;
                ensure_is_token!(
                    self.tokens.next(),
                    Token::Assign,
                    self.tokens.current_line()
                );
                self.tokens.consume()?;
                self.expression()?;

                self.generated_code.insert((
                    None,
                    Instruction::ARMI(
                        if self.current_function.is_none() {
                            0
                        } else {
                            1
                        },
                        1, //guarda endereço na pos reservada para lvalue
                    ),
                ));
            } else {
                ensure_is_token!(
                    self.tokens.next(),
                    Token::Assign,
                    self.tokens.current_line()
                );
                self.tokens.consume()?;
                self.expression()?;
                self.generated_code.insert((
                    None,
                    if is_indirect_assignment {
                        Instruction::ARMI(m, n)
                    } else {
                        Instruction::ARMZ(m, n)
                    },
                ));
            }
        }
        Ok(())
    }
    fn expression(&mut self) -> Result<(), CompileError> {
        self.logic_expr()?;
        while is_token!(self.tokens.next(), Token::Or) {
            self.tokens.consume()?;
            self.logic_expr()?;
            self.generated_code.insert((None, Instruction::DISJ));
        }
        Ok(())
    }
    fn logic_expr(&mut self) -> Result<(), CompileError> {
        self.relational_expr()?;
        while is_token!(self.tokens.next(), Token::And) {
            self.tokens.consume()?;
            self.relational_expr()?;
            self.generated_code.insert((None, Instruction::CONJ));
        }
        Ok(())
    }
    fn relational_expr(&mut self) -> Result<(), CompileError> {
        self.sum()?;
        if is_token!(self.tokens.next(), Token::LesserThan)
            || is_token!(self.tokens.next(), Token::GraterThan)
            || is_token!(self.tokens.next(), Token::LesserOrEqualThan)
            || is_token!(self.tokens.next(), Token::GreaterOrEqualThan)
            || is_token!(self.tokens.next(), Token::Equals)
            || is_token!(self.tokens.next(), Token::Different)
        {
            let comparison = self.tokens.consume()?;
            self.sum()?;
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
        Ok(())
    }
    fn sum(&mut self) -> Result<(), CompileError> {
        self.factor()?;
        while is_token!(self.tokens.next(), Token::Plus)
            || is_token!(self.tokens.next(), Token::Minus)
        {
            let op = self.tokens.consume()?;
            self.factor()?;
            self.generated_code.insert((
                None,
                match op {
                    Token::Plus => Instruction::SOMA,
                    Token::Minus => Instruction::SUBT,
                    _ => panic!("Impossible result"),
                },
            ));
        }
        Ok(())
    }
    fn factor(&mut self) -> Result<(), CompileError> {
        self.operand()?;
        while is_token!(self.tokens.next(), Token::Asterisc)
            || is_token!(self.tokens.next(), Token::Division)
        {
            let op = self.tokens.consume()?;
            self.operand()?;
            self.generated_code.insert((
                None,
                match op {
                    Token::Asterisc => Instruction::MULT,
                    Token::Division => Instruction::DIVI,
                    _ => panic!("Impossible result"),
                },
            ));
        }
        Ok(())
    }
    fn command(&mut self) -> Result<(), CompileError> {
        if is_token!(self.tokens.next(), Token::OpenBraces) {
            self.command_block()?;
        } else if is_token!(self.tokens.next(), Token::Asterisc) {
            self.attribuition()?;
            ensure_is_token!(
                self.tokens.next(),
                Token::SemiColon,
                self.tokens.current_line()
            );
            self.tokens.consume()?;
        } else if is_token!(self.tokens.next(), Token::Identifier(_)) {
            if is_token!(self.tokens.next_to_next(), Token::OpenParenthesis) {
                self.function_call()?;
                self.generated_code.insert((None, Instruction::DMEM(1)));
            } else {
                self.attribuition()?;
                ensure_is_token!(
                    self.tokens.next(),
                    Token::SemiColon,
                    self.tokens.current_line()
                );
            }
            self.tokens.consume()?;
        } else if is_token!(self.tokens.next(), Token::If) {
            self.if_command()?;
        } else if is_token!(self.tokens.next(), Token::While) {
            self.while_command()?;
        } else if is_token!(self.tokens.next(), Token::Print) {
            self.print_command()?;
            ensure_is_token!(
                self.tokens.next(),
                Token::SemiColon,
                self.tokens.current_line()
            );
            self.tokens.consume()?;
        } else if is_token!(self.tokens.next(), Token::Read) {
            self.read_command()?;
            ensure_is_token!(
                self.tokens.next(),
                Token::SemiColon,
                self.tokens.current_line()
            );
            self.tokens.consume()?;
        } else {
            ensure_is_token!(
                self.tokens.next(),
                Token::SemiColon,
                self.tokens.current_line()
            );
            self.tokens.consume()?;
        }
        Ok(())
    }
    fn if_command(&mut self) -> Result<(), CompileError> {
        ensure_is_token!(self.tokens.next(), Token::If, self.tokens.current_line());
        self.tokens.consume()?;
        let label_if = Label::new(self.simbols.new_label());

        ensure_is_token!(
            self.tokens.next(),
            Token::OpenParenthesis,
            self.tokens.current_line()
        );
        self.tokens.consume()?;
        self.expression()?;
        ensure_is_token!(
            self.tokens.next(),
            Token::CloseParenthesis,
            self.tokens.current_line()
        );
        self.generated_code
            .insert((None, Instruction::DSVF(label_if.clone())));
        self.tokens.consume()?;
        self.command()?;
        if is_token!(self.tokens.next(), Token::Else) {
            let label_else = Label::new(self.simbols.new_label());
            self.generated_code
                .insert((None, Instruction::DSVS(label_else.clone())));
            self.generated_code
                .insert((Some(label_if), Instruction::NADA));
            self.tokens.consume()?;
            self.command()?;
            self.generated_code
                .insert((Some(label_else), Instruction::NADA));
        } else {
            self.generated_code
                .insert((Some(label_if), Instruction::NADA));
        }
        Ok(())
    }
    fn while_command(&mut self) -> Result<(), CompileError> {
        ensure_is_token!(self.tokens.next(), Token::While, self.tokens.current_line());
        self.tokens.consume()?;
        let label_init = Label::new(self.simbols.new_label());
        let label_end = Label::new(self.simbols.new_label());
        self.generated_code
            .insert((Some(label_init.clone()), Instruction::NADA));
        ensure_is_token!(
            self.tokens.next(),
            Token::OpenParenthesis,
            self.tokens.current_line()
        );
        self.tokens.consume()?;
        self.expression()?;
        self.generated_code
            .insert((None, Instruction::DSVF(label_end.clone())));
        ensure_is_token!(
            self.tokens.next(),
            Token::CloseParenthesis,
            self.tokens.current_line()
        );
        self.tokens.consume()?;
        self.command()?;
        self.generated_code
            .insert((None, Instruction::DSVS(label_init)));
        self.generated_code
            .insert((Some(label_end), Instruction::NADA));
        Ok(())
    }
    fn read_command(&mut self) -> Result<(), CompileError> {
        ensure_is_token!(self.tokens.next(), Token::Read, self.tokens.current_line());
        self.tokens.consume()?;
        ensure_is_token!(
            self.tokens.next(),
            Token::OpenParenthesis,
            self.tokens.current_line()
        );
        self.tokens.consume()?;
        ensure_is_token!(
            self.tokens.next(),
            Token::Identifier(_),
            self.tokens.current_line()
        );
        if let Token::Identifier(s) = self.tokens.consume()? {
            let (m, n) = self
                .simbols
                .get_var_addr_and_type(&s, self.current_function.clone())
                .ok_or_else(|| {
                    CompileError::Semantic(format!(
                        "Variavel `{}` não foi declarada neste escopo, na linha {}",
                        s,
                        self.tokens.current_line()
                    ))
                })?;
            if is_token!(self.tokens.next(), Token::OpenBrackets) {
                self.tokens.consume()?;
                self.generated_code.insert((None, Instruction::CREN(m, n)));
                self.expression()?;
                self.generated_code.insert((None, Instruction::SOMA));
                self.generated_code.insert((
                    None,
                    Instruction::ARMZ(
                        if self.current_function.is_none() {
                            0
                        } else {
                            1
                        },
                        0,
                    ),
                ));

                self.generated_code.insert((None, Instruction::LEIT));

                self.generated_code.insert((
                    None,
                    Instruction::ARMI(
                        if self.current_function.is_none() {
                            0
                        } else {
                            1
                        },
                        0,
                    ),
                ));

                ensure_is_token!(
                    self.tokens.next(),
                    Token::CloseBrackets,
                    self.tokens.current_line()
                );
                self.tokens.consume()?;
            } else {
                self.generated_code.insert((None, Instruction::LEIT));
                self.generated_code.insert((None, Instruction::ARMZ(m, n)));
            }
        }
        ensure_is_token!(
            self.tokens.next(),
            Token::CloseParenthesis,
            self.tokens.current_line()
        );
        self.tokens.consume()?;
        Ok(())
    }

    fn print_command(&mut self) -> Result<(), CompileError> {
        ensure_is_token!(self.tokens.next(), Token::Print, self.tokens.current_line());
        self.tokens.consume()?;
        ensure_is_token!(
            self.tokens.next(),
            Token::OpenParenthesis,
            self.tokens.current_line()
        );
        self.tokens.consume()?;
        let args = self.argument_list()?;
        for _ in 0..args {
            self.generated_code.insert((None, Instruction::IMPR));
        }
        ensure_is_token!(
            self.tokens.next(),
            Token::CloseParenthesis,
            self.tokens.current_line()
        );
        self.tokens.consume()?;
        Ok(())
    }
    fn function_call(&mut self) -> Result<(), CompileError> {
        ensure_is_token!(
            self.tokens.next(),
            Token::Identifier(_),
            self.tokens.current_line()
        );
        if let Token::Identifier(s) = self.tokens.consume()? {
            if let Some(fn_loc) = self.simbols.get_fn_label(&s) {
                let label = Label::new(fn_loc);
                ensure_is_token!(
                    self.tokens.next(),
                    Token::OpenParenthesis,
                    self.tokens.current_line()
                );
                self.tokens.consume()?;
                //reserve a position for return value
                self.generated_code.insert((None, Instruction::AMEM(1)));
                self.argument_list()?;
                ensure_is_token!(
                    self.tokens.next(),
                    Token::CloseParenthesis,
                    self.tokens.current_line()
                );
                self.tokens.consume()?;
                self.generated_code.insert((None, Instruction::CHPR(label)));
            } else {
                return Err(CompileError::Semantic(format!(
                    "Função \"{}\" não foi declarada, na linha {}",
                    s,
                    self.tokens.current_line()
                )));
            }
        }
        Ok(())
    }
    //retorna quantos foram carregados
    fn argument_list(&mut self) -> Result<usize, CompileError> {
        let mut count = 0;
        if !is_token!(self.tokens.next(), Token::CloseBraces) {
            self.expression()?;
            count += 1;
        }
        while is_token!(self.tokens.next(), Token::Comma) {
            self.tokens.consume()?;
            self.expression()?;
            count += 1;
        }
        Ok(count)
    }
    fn operand(&mut self) -> Result<(), CompileError> {
        if is_token!(self.tokens.next(), Token::Identifier(_)) {
            //function call
            if is_token!(self.tokens.next_to_next(), Token::OpenParenthesis) {
                self.function_call()?;
            } else {
                //identifier
                if let Token::Identifier(s) = self.tokens.consume()? {
                    let (m, n) = self
                        .simbols
                        .get_var_addr_and_type(&s, self.current_function.clone())
                        .ok_or_else(|| {
                            CompileError::Semantic(format!(
                                "Variavel `{}` não foi declarada neste escopo, na linha {}",
                                s,
                                self.tokens.current_line()
                            ))
                        })?;
                    if is_token!(self.tokens.next(), Token::OpenBrackets) {
                        self.tokens.consume()?;
                        self.generated_code.insert((None, Instruction::CRVL(m, n)));
                        self.expression()?;
                        self.generated_code.insert((None, Instruction::SOMA));
                        self.generated_code.insert((
                            None,
                            Instruction::ARMZ(
                                if self.current_function.is_none() {
                                    0
                                } else {
                                    1
                                },
                                0,
                            ),
                        ));

                        self.generated_code.insert((
                            None,
                            Instruction::CRVI(
                                if self.current_function.is_none() {
                                    0
                                } else {
                                    1
                                },
                                0,
                            ),
                        ));

                        ensure_is_token!(
                            self.tokens.next(),
                            Token::CloseBrackets,
                            self.tokens.current_line()
                        );
                        self.tokens.consume()?;
                    } else {
                        self.generated_code.insert((None, Instruction::CRVL(m, n)));
                    }
                }
            }
        } else if is_token!(self.tokens.next(), Token::Number(_)) {
            if let Token::Number(n) = self.tokens.consume()? {
                self.generated_code.insert((None, Instruction::CRCT(n)));
            }
        } else if is_token!(self.tokens.next(), Token::OpenParenthesis) {
            self.tokens.consume()?;
            self.expression()?;
            ensure_is_token!(
                self.tokens.next(),
                Token::CloseParenthesis,
                self.tokens.current_line()
            );
            self.tokens.consume()?;
        } else if is_token!(self.tokens.next(), Token::Minus) {
            self.tokens.consume()?;
            self.operand()?;
            self.generated_code.insert((None, Instruction::INVR));
        } else if is_token!(self.tokens.next(), Token::Not) {
            self.tokens.consume()?;
            self.operand()?;
            self.generated_code.insert((None, Instruction::NEGA));
        } else if is_token!(self.tokens.next(), Token::AddressOf) {
            self.tokens.consume()?;
            ensure_is_token!(
                self.tokens.next(),
                Token::Identifier(_),
                self.tokens.current_line()
            );
            if let Token::Identifier(s) = self.tokens.consume()? {
                let (m, n) = self
                    .simbols
                    .get_var_addr_and_type(&s, self.current_function.clone())
                    .ok_or_else(|| {
                        CompileError::Semantic(format!(
                            "Variavel `{}` não foi declarada neste escopo, na linha {}",
                            s,
                            self.tokens.current_line()
                        ))
                    })?;
                if is_token!(self.tokens.next(), Token::OpenBrackets) {
                    self.tokens.consume()?;
                    self.generated_code.insert((None, Instruction::CRVL(m, n)));
                    self.expression()?;
                    self.generated_code.insert((None, Instruction::SOMA));

                    ensure_is_token!(
                        self.tokens.next(),
                        Token::CloseBrackets,
                        self.tokens.current_line()
                    );
                    self.tokens.consume()?;
                } else {
                    self.generated_code.insert((None, Instruction::CREN(m, n)));
                }
            }
        } else if is_token!(self.tokens.next(), Token::Asterisc) {
            self.tokens.consume()?;
            ensure_is_token!(
                self.tokens.next(),
                Token::Identifier(_),
                self.tokens.current_line()
            );
            if let Token::Identifier(s) = self.tokens.consume()? {
                let (m, n) = self
                    .simbols
                    .get_var_addr_and_type(&s, self.current_function.clone())
                    .ok_or_else(|| {
                        CompileError::Semantic(format!(
                            "Variavel `{}` não foi declarada neste escopo, na linha {}",
                            s,
                            self.tokens.current_line()
                        ))
                    })?;
                if is_token!(self.tokens.next(), Token::OpenBrackets) {
                    self.tokens.consume()?;
                    self.generated_code.insert((None, Instruction::CRVL(m, n)));
                    self.expression()?;
                    self.generated_code.insert((None, Instruction::SOMA));
                    self.generated_code.insert((
                        None,
                        Instruction::ARMZ(
                            if self.current_function.is_none() {
                                0
                            } else {
                                1
                            },
                            0,
                        ),
                    ));

                    self.generated_code.insert((
                        None,
                        Instruction::CRVI(
                            if self.current_function.is_none() {
                                0
                            } else {
                                1
                            },
                            0,
                        ),
                    ));
                    self.generated_code.insert((
                        None,
                        Instruction::ARMZ(
                            if self.current_function.is_none() {
                                0
                            } else {
                                1
                            },
                            0,
                        ),
                    ));
                    self.generated_code.insert((
                        None,
                        Instruction::CRVI(
                            if self.current_function.is_none() {
                                0
                            } else {
                                1
                            },
                            0,
                        ),
                    ));

                    ensure_is_token!(
                        self.tokens.next(),
                        Token::CloseBrackets,
                        self.tokens.current_line()
                    );
                    self.tokens.consume()?;
                } else {
                    self.generated_code.insert((None, Instruction::CRVI(m, n)));
                }
            }
        }
        Ok(())
    }
}

pub fn compile(
    origin: &PathBuf,
    target: &PathBuf,
    otimizar: bool,
) -> Result<io::Result<()>, CompileError> {
    let mut c = Compiler::new(origin)?;
    c.program()?;
    // println!("Compilado com sucesso!");
    Ok({
        let e = c.generated_code.to_file(target);
        if otimizar {
            println!("Otimizando...");
            Otimizador::from(target)
                .otimizar()
                .unwrap()
                .save()
                .expect("Falha ao salvar otimizado");
        }
        e
    })
}
