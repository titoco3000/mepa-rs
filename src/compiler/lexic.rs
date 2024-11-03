use super::error::CompileError;
use std::fs::File;
use std::io::{BufReader, Read};
use std::path::PathBuf;
use std::usize;

#[derive(Debug, PartialEq)]
pub enum Token {
    Identifier(String),
    Number(i32),
    Plus,
    Minus,
    Asterisc,
    Division,
    AddressOf,
    And,
    Or,
    Not,
    Equals,
    Different,
    Assign,
    GraterThan,
    LesserThan,
    GreaterOrEqualThan,
    LesserOrEqualThan,
    OpenParenthesis,
    CloseParenthesis,
    OpenBraces,
    CloseBraces,
    OpenBrackets,
    CloseBrackets,
    Comma,
    SemiColon,
    Int,
    Ptr,
    Print,
    Read,
    If,
    Else,
    While,
    Return,
    Function,
}

#[macro_export]
macro_rules! is_token {
    ($x:expr, $p:pat) => {
        matches!($x, Some($p))
    };
}

#[macro_export]
macro_rules! ensure_is_token {
    ($option:expr, $expected:pat_param, $line_number:expr) => {
        match $option {
            Some($expected) => {}
            Some(actual) => {
                return Err(CompileError::Sintatic(format!(
                    "Esperava '{}', obteve 'Token:{:?}' na linha {}",
                    stringify!($expected),
                    actual,
                    $line_number
                )))
            }
            None => {
                return Err(CompileError::Sintatic(format!(
                    "Esperava '{}', mas chegou ao fim do arquivo",
                    stringify!($expected)
                )))
            }
        }
    };
}

pub struct Reader {
    reader: BufReader<File>,
    next_char: Option<char>,
    current_line: usize,
}

impl Reader {
    pub fn new(file_path: &PathBuf) -> Result<Reader, CompileError> {
        let file = File::open(file_path).expect("unable to open file");
        let mut reader = BufReader::new(file);
        let mut single_char = [0; 1]; // Buffer for reading one byte at a time
        Ok(Reader {
            next_char: {
                if reader
                    .read(&mut single_char)
                    .map_err(|e| CompileError::Lexic(format!("{}", e)))?
                    == 0
                {
                    panic!("Error on first character");
                }
                char::from_u32(single_char[0] as u32)
            },
            reader,
            current_line: 1,
        })
    }

    fn consume_char(&mut self) {
        if self.next_char.is_some() && self.next_char.unwrap() == '\n' {
            self.current_line += 1;
        }
        let mut single_char = [0; 1]; // Buffer for reading one byte at a time
        self.next_char = if self.reader.read(&mut single_char).expect("Error on read") > 0 {
            if let Some(c) = char::from_u32(single_char[0] as u32) {
                Some(c)
            } else {
                None
            }
        } else {
            None
        };
    }

    pub fn get_next_token(&mut self, line:usize) -> Result<(Option<Token>, usize), CompileError> {
        //skip whitespaces
        loop {
            if let Some(c) = self.next_char {
                if c.is_whitespace() {
                    self.consume_char();
                    continue;
                }
            }
            break;
        }

        if let Some(c) = self.next_char {
            self.consume_char();
            //single character tokens
            match c {
                '(' => Ok((Some(Token::OpenParenthesis), self.current_line)),
                ')' => Ok((Some(Token::CloseParenthesis), self.current_line)),
                '{' => Ok((Some(Token::OpenBraces), self.current_line)),
                '}' => Ok((Some(Token::CloseBraces), self.current_line)),
                '[' => Ok((Some(Token::OpenBrackets), self.current_line)),
                ']' => Ok((Some(Token::CloseBrackets), self.current_line)),
                ',' => Ok((Some(Token::Comma), self.current_line)),
                ';' => Ok((Some(Token::SemiColon), self.current_line)),
                '+' => Ok((Some(Token::Plus), self.current_line)),
                '-' => Ok((Some(Token::Minus), self.current_line)),
                '*' => Ok((Some(Token::Asterisc), self.current_line)),
                _ => {
                    if c == '/' {
                        //into comment
                        if self.next_char.is_some() && self.next_char.unwrap() == '/' {
                            while self.next_char.is_some() && self.next_char.unwrap() != '\n' {
                                self.consume_char();
                            }
                            self.get_next_token(line)
                        } else if self.next_char.is_some() && self.next_char.unwrap() == '*' {
                            loop {
                                if let Some(c) = self.next_char {
                                    self.consume_char();
                                    if c == '*' {
                                        if self.next_char.is_some()
                                            && self.next_char.unwrap() == '/'
                                        {
                                            self.consume_char();
                                            break;
                                        }
                                    }
                                } else {
                                    return Err(CompileError::Lexic(
                                        "Comentário multi-linhas inacabado no fim do arquivo".to_owned(),
                                    ));
                                }
                            }
                            self.get_next_token(line)
                        } else {
                            return Ok((Some(Token::Division), self.current_line));
                        }
                    }
                    //equals and assign
                    else if c == '=' {
                        if self.next_char.is_some() && self.next_char.unwrap() == '=' {
                            self.consume_char();
                            Ok((Some(Token::Equals), self.current_line))
                        } else {
                            Ok((Some(Token::Assign), self.current_line))
                        }
                    }
                    //not and different
                    else if c == '!' {
                        if self.next_char.is_some() && self.next_char.unwrap() == '=' {
                            self.consume_char();
                            Ok((Some(Token::Different), self.current_line))
                        } else {
                            Ok((Some(Token::Not), self.current_line))
                        }
                    }
                    //or
                    else if c == '|' {
                        if self.next_char.is_some() && self.next_char.unwrap() == '|' {
                            self.consume_char();
                            Ok((Some(Token::Or), self.current_line))
                        } else {
                            Err(CompileError::Lexic(format!("Esperava '|', obteve '{}' na linha {}", c, line)))
                        }
                    }
                    //or
                    else if c == '&' {
                        if self.next_char.is_some() && self.next_char.unwrap() == '&' {
                            self.consume_char();
                            Ok((Some(Token::And), self.current_line))
                        } else {
                            Ok((Some(Token::AddressOf), self.current_line))
                        }
                    }
                    // < and <=
                    else if c == '<' {
                        if self.next_char.is_some() && self.next_char.unwrap() == '=' {
                            self.consume_char();
                            Ok((Some(Token::LesserOrEqualThan), self.current_line))
                        } else {
                            Ok((Some(Token::LesserThan), self.current_line))
                        }
                    }
                    // > and >=
                    else if c == '>' {
                        if self.next_char.is_some() && self.next_char.unwrap() == '=' {
                            self.consume_char();
                            Ok((Some(Token::GreaterOrEqualThan), self.current_line))
                        } else {
                            Ok((Some(Token::GraterThan), self.current_line))
                        }
                    }
                    // identifiers, numbers and keywords
                    else {
                        let mut buffer = Vec::with_capacity(64);
                        if c.is_alphanumeric() || c == '_' {
                            buffer.push(c);
                            while self.next_char.is_some()
                                && (self.next_char.unwrap().is_alphanumeric()
                                    || self.next_char.unwrap() == '_')
                            {
                                buffer.push(self.next_char.unwrap());
                                self.consume_char();
                            }
                            let s: String = buffer.iter().collect();
                            if let Some(keyword) = match s.as_str() {
                                "int" => Some(Token::Int),
                                "ptr" => Some(Token::Ptr),
                                "print" => Some(Token::Print),
                                "read" => Some(Token::Read),
                                "if" => Some(Token::If),
                                "else" => Some(Token::Else),
                                "while" => Some(Token::While),
                                "return" => Some(Token::Return),
                                "fn" => Some(Token::Function),
                                _ => None,
                            } {
                                Ok((Some(keyword), self.current_line))
                            } else if let Ok(n) = s.parse::<i32>() {
                                Ok((Some(Token::Number(n)), self.current_line))
                            } else {
                                Ok((Some(Token::Identifier(s)), self.current_line))
                            }
                        } else {
                            Err(CompileError::Lexic(format!("Char inesperado: '{}' na linha {}", c, line)))
                        }
                    }
                }
            }
        } else {
            Ok((None, self.current_line))
        }
    }
}

#[derive(Debug)]
pub struct Lexic(Vec<(Token, usize)>, usize);

impl Lexic {
    pub fn new(file_path: &PathBuf) -> Result<Lexic, CompileError> {
        let mut list = Vec::with_capacity(128);

        let mut l = Reader::new(file_path)?;

        let mut current_line = 1;
        while let (Some(token), line) = l.get_next_token(current_line)? {
            current_line = line;
            list.push((token, line));
        }

        list = list.into_iter().rev().collect();
        let line = list.last()
        .ok_or_else(|| CompileError::Lexic("Arquivo sem tokens".to_owned()))?.1;
        Ok(Lexic(
            list,
            line
        ))
    }

    pub fn next(&self) -> Option<&Token> {
        match self.0.last() {
            Some(t) => Some(&t.0),
            None => None,
        }
    }
    pub fn next_to_next(&self) -> Option<&Token> {
        match self.0.get(self.0.len() - 2) {
            Some(t) => Some(&t.0),
            None => None,
        }
    }
    pub fn consume(&mut self) -> Result<Token, CompileError> {
        match self.0.pop() {
            Some((t, l)) => {
                self.1 = match self.0.last() {
                    Some(last)=>last.1,
                    None => l
                };
                Ok(t)
            }
            None => Err(CompileError::Sintatic(format!("Fim de arquivo inesperado"))),
        }
    }
    pub fn current_line(&self) -> usize {
        self.1
    }
}
