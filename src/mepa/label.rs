use std::fmt;
use std::str::FromStr;
use super::code::MepaCode;

#[derive(Debug)]
pub enum Label {
    Simbolic(String),
    Literal(usize),
}
impl Label {
    pub fn locate(&self, mc: &MepaCode) -> Option<usize> {
        match self {
            Self::Simbolic(s) => {
                for (i, (op_label, _)) in mc.0.iter().enumerate() {
                    if let Some(label) = op_label {
                        if let Self::Simbolic(simbol) = label {
                            if s == simbol {
                                return Some(i);
                            }
                        }
                    }
                }
                None
            }
            Self::Literal(u) => Some(*u),
        }
    }
}
impl FromStr for Label {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let trimmed = s.trim();

        if trimmed.is_empty() {
            Err("Invalid label")
        } else {
            Ok(if let Ok(literal) = trimmed.parse() {
                Label::Literal(literal)
            } else {
                Label::Simbolic(s.to_string())
            })
        }
    }
}

impl fmt::Display for Label {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Label::Simbolic(s) => write!(f, "{}", s),
            Label::Literal(n) => write!(f, "{}", n),
        }
    }
}