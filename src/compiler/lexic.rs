use std::fs::File;
use std::io::{BufReader, Read};
use std::path::PathBuf;

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
    Function
}

#[macro_export]
macro_rules! is_token {
    ($x:expr, $p:pat) => {
        matches!($x, Some($p))
    };
}

#[macro_export]
macro_rules! ensure_is_token {
    ($x:expr, $p:pat) => {
        if !matches!($x, Some($p)){
            panic!("Got {:?}, expected something else",$x.unwrap());
        }
    };
}

pub struct Reader {
    reader: BufReader<File>,
    next_char: Option<char>,
}

impl Reader {
    pub fn new(file_path: &PathBuf) -> Reader {
        let file = File::open(file_path).expect("unable to open file");
        let mut reader = BufReader::new(file);
        let mut single_char = [0; 1]; // Buffer for reading one byte at a time        
        let mut l = Reader {
            next_char: {
                if reader.read(&mut single_char).unwrap() == 0{
                    panic!("Error on first character");
                }
                char::from_u32(single_char[0] as u32)},
            reader,
        };

        l
    }

    fn consume_char(&mut self) {
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

    pub fn get_next_token(&mut self) -> Result<Option<Token>, &str> {
        let mut buffer = Vec::new();
        let mut in_comment = false;
        while let Some(lookahead) = self.next_char {
            if in_comment {
                if lookahead == '\n' {
                    in_comment = false;
                }
                self.consume_char();
            } else {
                // all special pairs (==, !=, &&, ||, >=, <=, //)
                if buffer.len() == 1 {
                    if buffer[0] == '=' {
                        return Ok(Some(if lookahead == '=' {
                            self.consume_char();
                            Token::Equals
                        } else {
                            Token::Assign
                        }));
                    } else if buffer[0] == '!' {
                        return Ok(Some(if lookahead == '=' {
                            self.consume_char();
                            Token::Different
                        } else {
                            Token::Not
                        }));
                    } else if buffer[0] == '&' {
                        return Ok(Some(if lookahead == '&' {
                            self.consume_char();
                            Token::And
                        } else {
                            Token::AddressOf
                        }));
                    } else if buffer[0] == '|' {
                        return if lookahead == '|' {
                            self.consume_char();
                            Ok(Some(Token::Or))
                        } else {
                            Err("Parse error")
                        };
                    } else if buffer[0] == '>' {
                        return Ok(Some(if lookahead == '=' {
                            self.consume_char();
                            Token::GreaterOrEqualThan
                        } else {
                            Token::GraterThan
                        }));
                    } else if buffer[0] == '<' {
                        return Ok(Some(if lookahead == '=' {
                            self.consume_char();
                            Token::LesserOrEqualThan
                        } else {
                            Token::LesserThan
                        }));
                    } else if buffer[0] == '/' {
                        if lookahead == '/'{
                            in_comment = true;
                        }
                        else{
                            return Ok(Some(Token::Division));
                        }
                    }
                }

                //characters from numbers and identifiers
                if lookahead.is_alphanumeric() || lookahead == '_' {
                    buffer.push(lookahead);
                    self.consume_char();
                }
                 else if buffer.len() > 0 {
                    let s: String = buffer.iter().collect();
                    buffer.clear();
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
                        return Ok(Some(keyword));
                    } else if let Ok(n) = s.parse::<i32>() {
                        return Ok(Some(Token::Number(n)));
                    } else {
                        return Ok(Some(Token::Identifier(s)));
                    }
                } else {
                    self.consume_char();
                    if let Some(single) = match lookahead {
                        '+' => Some(Token::Plus),
                        '-' => Some(Token::Minus),
                        '*' => Some(Token::Asterisc),
                        '>' => Some(Token::GraterThan),
                        '<' => Some(Token::LesserThan),
                        '(' => Some(Token::OpenParenthesis),
                        ')' => Some(Token::CloseParenthesis),
                        '{' => Some(Token::OpenBraces),
                        '}' => Some(Token::CloseBraces),
                        ',' => Some(Token::Comma),
                        ';' => Some(Token::SemiColon),
                        _ => None,
                    } {
                        return Ok(Some(single));
                    }
                    else{
                        if !lookahead.is_whitespace(){
                            buffer.push(lookahead);
                        }
                    }
                }
            }
        }
        Ok(None)
    }

}

#[derive(Debug)]
pub struct Lexic(Vec<Token>);

impl Lexic {
    pub fn new(file_path: &PathBuf) ->Lexic{
        let mut list = Vec::with_capacity(128);

        let mut l = Reader::new(file_path);
    
        while let Some(token) = l.get_next_token().unwrap() {
            list.push(token);
        }
    
        list = list.into_iter().rev().collect();
    
        Lexic(list)
    }

    pub fn next(&self) -> Option<&Token>{
        self.0.last()
    }
    pub fn next_to_next(&self) -> Option<&Token>{
        self.0.get(self.0.len()-2)
    }
    pub fn consume(&mut self) -> Token{
        self.0.pop().unwrap()
    }
}