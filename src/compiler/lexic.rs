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
    ($x:expr, $p:pat) => {
        if !matches!($x, Some($p)) {
            panic!("Got {:?}, expected something else", $x.unwrap());
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
        Reader {
            next_char: {
                if reader.read(&mut single_char).unwrap() == 0 {
                    panic!("Error on first character");
                }
                char::from_u32(single_char[0] as u32)
            },
            reader,
        }
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
        //skip whitespaces
        loop{
            if let Some(c) = self.next_char {
                if c.is_whitespace(){
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
                '(' => Ok(Some(Token::OpenParenthesis)),
                ')' => Ok(Some(Token::CloseParenthesis)),
                '{' => Ok(Some(Token::OpenBraces)),
                '}' => Ok(Some(Token::CloseBraces)),
                ',' => Ok(Some(Token::Comma)),
                ';' => Ok(Some(Token::SemiColon)),
                '+' => Ok(Some(Token::Plus)),
                '-' => Ok(Some(Token::Minus)),
                '*' => Ok(Some(Token::Asterisc)),
                _ => {
                    if c == '/' {
                        //into comment
                        if self.next_char.is_some() && self.next_char.unwrap() == '/' {
                            while self.next_char.is_some() && self.next_char.unwrap() != '\n' {
                                self.consume_char();
                            }
                            self.get_next_token()
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
                                    return Err("Unfinished multi-line comment");
                                }
                            }
                            self.get_next_token()
                        } else {
                            return Ok(Some(Token::Division));
                        }
                    }
                    //equals and assign
                    else if c == '=' {
                        if self.next_char.is_some() && self.next_char.unwrap() == '=' {
                            self.consume_char();
                            Ok(Some(Token::Equals))
                        } else {
                            Ok(Some(Token::Assign))
                        }
                    }
                    //not and different
                    else if c == '!' {
                        if self.next_char.is_some() && self.next_char.unwrap() == '=' {
                            self.consume_char();
                            Ok(Some(Token::Different))
                        } else {
                            Ok(Some(Token::Not))
                        }
                    }
                    //or
                    else if c == '|' {
                        if self.next_char.is_some() && self.next_char.unwrap() == '|' {
                            self.consume_char();
                            Ok(Some(Token::Or))
                        } else {
                            Err("Expected '|'")
                        }
                    }
                    //or
                    else if c == '&' {
                        if self.next_char.is_some() && self.next_char.unwrap() == '&' {
                            self.consume_char();
                            Ok(Some(Token::And))
                        } else {
                            Ok(Some(Token::AddressOf))
                        }
                    }
                    // < and <=
                    else if c == '<' {
                        if self.next_char.is_some() && self.next_char.unwrap() == '=' {
                            self.consume_char();
                            Ok(Some(Token::LesserOrEqualThan))
                        } else {
                            Ok(Some(Token::LesserThan))
                        }
                    }
                    // > and >=
                    else if c == '>' {
                        if self.next_char.is_some() && self.next_char.unwrap() == '=' {
                            self.consume_char();
                            Ok(Some(Token::GreaterOrEqualThan))
                        } else {
                            Ok(Some(Token::GraterThan))
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
                                Ok(Some(keyword))
                            } else if let Ok(n) = s.parse::<i32>() {
                                Ok(Some(Token::Number(n)))
                            } else {
                                Ok(Some(Token::Identifier(s)))
                            }
                        } else {
                            println!("Unexpected char: '{}'", c);
                            Err("Unexpected char")
                        }
                    }
                }
            }
        } else {
            Ok(None)
        }
    }
}

#[derive(Debug)]
pub struct Lexic(Vec<Token>);

impl Lexic {
    pub fn new(file_path: &PathBuf) -> Lexic {
        let mut list = Vec::with_capacity(128);

        let mut l = Reader::new(file_path);

        while let Some(token) = l.get_next_token().unwrap() {
            list.push(token);
        }

        list = list.into_iter().rev().collect();

        Lexic(list)
    }

    pub fn next(&self) -> Option<&Token> {
        self.0.last()
    }
    pub fn next_to_next(&self) -> Option<&Token> {
        self.0.get(self.0.len() - 2)
    }
    pub fn consume(&mut self) -> Token {
        self.0.pop().unwrap()
    }
}
