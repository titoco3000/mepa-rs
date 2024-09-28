use super::label::Label;
use std::fmt;

#[derive(Debug)]
pub enum Instruction {
    CRCT(i32),
    CRVL(i32, i32),
    CREN(i32, i32),
    ARMZ(i32, i32),
    CRVI(i32, i32),
    ARMI(i32, i32),
    SOMA,
    SUBT,
    MULT,
    DIVI,
    INVR,
    CONJ,
    DISJ,
    NEGA,
    CMME,
    CMMA,
    CMIG,
    CMDG,
    CMEG,
    CMAG,
    DSVS(Label),
    DSVF(Label),
    NADA,
    PARA,
    LEIT,
    IMPR,
    AMEM(i32),
    DMEM(i32),
    INPP,
    CHPR(Label),
    ENPR(i32),
    RTPR(i32, i32),
}

impl Instruction {
    fn valid(s: &str) -> bool {
        [
            "CRCT", "CRVL", "CREN", "ARMZ", "CRVI", "ARMI", "SOMA", "SUBT", "MULT", "DIVI", "INVR",
            "CONJ", "DISJ", "NEGA", "CMME", "CMMA", "CMIG", "CMDG", "CMEG", "CMAG", "DSVS", "DSVF",
            "NADA", "PARA", "LEIT", "IMPR", "AMEM", "DMEM", "INPP", "CHPR", "ENPR", "RTPR",
        ]
        .contains(&s)
    }
    pub fn parse(line: &[&str]) -> Result<(Option<Label>, Instruction), &'static str> {
        let label = if Instruction::valid(line[0]) {
            None
        } else {
            Some(Label::Simbolic(line[0].to_owned()))
        };
        let i = if label.is_some() { 1 } else { 0 };

        let missing_arg_err = "Missing argument";
        let parse_err = "Failed to parse argument";

        let instruction = match line.get(i) {
            Some(&"CRCT") => Instruction::CRCT(
                line.get(i + 1)
                    .ok_or(missing_arg_err)?
                    .parse::<i32>()
                    .map_err(|_| parse_err)?,
            ),
            Some(&"CRVL") => {
                if line.len() - i == 2 {
                    Instruction::CRVL(
                        0,
                        line.get(i + 1)
                            .ok_or(missing_arg_err)?
                            .parse::<i32>()
                            .map_err(|_| parse_err)?,
                    )
                } else {
                    Instruction::CRVL(
                        line.get(i + 1)
                            .ok_or(missing_arg_err)?
                            .parse::<i32>()
                            .map_err(|_| parse_err)?,
                        line.get(i + 2)
                            .ok_or(missing_arg_err)?
                            .parse::<i32>()
                            .map_err(|_| parse_err)?,
                    )
                }
            }
            Some(&"CREN") => {
                if line.len() - i == 2 {
                    Instruction::CREN(
                        0,
                        line.get(i + 1)
                            .ok_or(missing_arg_err)?
                            .parse::<i32>()
                            .map_err(|_| parse_err)?,
                    )
                } else {
                    Instruction::CREN(
                        line.get(i + 1)
                            .ok_or(missing_arg_err)?
                            .parse::<i32>()
                            .map_err(|_| parse_err)?,
                        line.get(i + 2)
                            .ok_or(missing_arg_err)?
                            .parse::<i32>()
                            .map_err(|_| parse_err)?,
                    )
                }
            }
            Some(&"ARMZ") => {
                if line.len() - i == 2 {
                    Instruction::ARMZ(
                        0,
                        line.get(i + 1)
                            .ok_or(missing_arg_err)?
                            .parse::<i32>()
                            .map_err(|_| parse_err)?,
                    )
                } else {
                    Instruction::ARMZ(
                        line.get(i + 1)
                            .ok_or(missing_arg_err)?
                            .parse::<i32>()
                            .map_err(|_| parse_err)?,
                        line.get(i + 2)
                            .ok_or(missing_arg_err)?
                            .parse::<i32>()
                            .map_err(|_| parse_err)?,
                    )
                }
            }
            Some(&"CRVI") => {
                if line.len() - i == 2 {
                    Instruction::CRVI(
                        0,
                        line.get(i + 1)
                            .ok_or(missing_arg_err)?
                            .parse::<i32>()
                            .map_err(|_| parse_err)?,
                    )
                } else {
                    Instruction::CRVI(
                        line.get(i + 1)
                            .ok_or(missing_arg_err)?
                            .parse::<i32>()
                            .map_err(|_| parse_err)?,
                        line.get(i + 2)
                            .ok_or(missing_arg_err)?
                            .parse::<i32>()
                            .map_err(|_| parse_err)?,
                    )
                }
            }
            Some(&"ARMI") => {
                if line.len() - i == 2 {
                    Instruction::ARMI(
                        0,
                        line.get(i + 1)
                            .ok_or(missing_arg_err)?
                            .parse::<i32>()
                            .map_err(|_| parse_err)?,
                    )
                } else {
                    Instruction::ARMI(
                        line.get(i + 1)
                            .ok_or(missing_arg_err)?
                            .parse::<i32>()
                            .map_err(|_| parse_err)?,
                        line.get(i + 2)
                            .ok_or(missing_arg_err)?
                            .parse::<i32>()
                            .map_err(|_| parse_err)?,
                    )
                }
            }
            Some(&"SOMA") => Self::SOMA,
            Some(&"SUBT") => Self::SUBT,
            Some(&"MULT") => Self::MULT,
            Some(&"DIVI") => Self::DIVI,
            Some(&"INVR") => Self::INVR,
            Some(&"CONJ") => Self::CONJ,
            Some(&"DISJ") => Self::DISJ,
            Some(&"NEGA") => Self::NEGA,
            Some(&"CMME") => Self::CMME,
            Some(&"CMMA") => Self::CMMA,
            Some(&"CMIG") => Self::CMIG,
            Some(&"CMDG") => Self::CMDG,
            Some(&"CMEG") => Self::CMEG,
            Some(&"CMAG") => Self::CMAG,
            Some(&"DSVS") => Self::DSVS(
                line.get(i + 1)
                    .ok_or(missing_arg_err)?
                    .parse::<Label>()
                    .map_err(|_| parse_err)?,
            ),
            Some(&"DSVF") => Self::DSVF(
                line.get(i + 1)
                    .ok_or(missing_arg_err)?
                    .parse::<Label>()
                    .map_err(|_| parse_err)?,
            ),
            Some(&"NADA") => Self::NADA,
            Some(&"PARA") => Self::PARA,
            Some(&"LEIT") => Self::LEIT,
            Some(&"IMPR") => Self::IMPR,
            Some(&"AMEM") => Instruction::AMEM(
                line.get(i + 1)
                    .ok_or(missing_arg_err)?
                    .parse::<i32>()
                    .map_err(|_| parse_err)?,
            ),
            Some(&"DMEM") => Instruction::DMEM(
                line.get(i + 1)
                    .ok_or(missing_arg_err)?
                    .parse::<i32>()
                    .map_err(|_| parse_err)?,
            ),
            Some(&"INPP") => Self::INPP,
            Some(&"ENPR") => Self::ENPR(
                line.get(i + 1)
                    .ok_or(missing_arg_err)?
                    .parse::<i32>()
                    .map_err(|_| parse_err)?,
            ),
            Some(&"CHPR") => Self::CHPR(
                line.get(i + 1)
                    .ok_or(missing_arg_err)?
                    .parse::<Label>()
                    .map_err(|_| parse_err)?,
            ),
            Some(&"RTPR") => Self::RTPR(
                line.get(i + 1)
                    .ok_or(missing_arg_err)?
                    .parse::<i32>()
                    .map_err(|_| parse_err)?,
                line.get(i + 2)
                    .ok_or(missing_arg_err)?
                    .parse::<i32>()
                    .map_err(|_| parse_err)?,
            ),
            _ => return Err("Unknown instruction"),
        };

        Ok((label, instruction))
    }
    pub fn to_string_vec(&self) -> Vec<String> {
        match self {
            Instruction::CRCT(val) => vec!["CRCT".to_string(), val.to_string()],
            Instruction::CRVL(a, b) => vec!["CRVL".to_string(), a.to_string(), b.to_string()],
            Instruction::CREN(a, b) => vec!["CREN".to_string(), a.to_string(), b.to_string()],
            Instruction::ARMZ(a, b) => vec!["ARMZ".to_string(), a.to_string(), b.to_string()],
            Instruction::CRVI(a, b) => vec!["CRVI".to_string(), a.to_string(), b.to_string()],
            Instruction::ARMI(a, b) => vec!["ARMI".to_string(), a.to_string(), b.to_string()],
            Instruction::SOMA => vec!["SOMA".to_string()],
            Instruction::SUBT => vec!["SUBT".to_string()],
            Instruction::MULT => vec!["MULT".to_string()],
            Instruction::DIVI => vec!["DIVI".to_string()],
            Instruction::INVR => vec!["INVR".to_string()],
            Instruction::CONJ => vec!["CONJ".to_string()],
            Instruction::DISJ => vec!["DISJ".to_string()],
            Instruction::NEGA => vec!["NEGA".to_string()],
            Instruction::CMME => vec!["CMME".to_string()],
            Instruction::CMMA => vec!["CMMA".to_string()],
            Instruction::CMIG => vec!["CMIG".to_string()],
            Instruction::CMDG => vec!["CMDG".to_string()],
            Instruction::CMEG => vec!["CMEG".to_string()],
            Instruction::CMAG => vec!["CMAG".to_string()],
            Instruction::DSVS(label) => vec!["DSVS".to_string(), label.to_string()],
            Instruction::DSVF(label) => vec!["DSVF".to_string(), label.to_string()],
            Instruction::NADA => vec!["NADA".to_string()],
            Instruction::PARA => vec!["PARA".to_string()],
            Instruction::LEIT => vec!["LEIT".to_string()],
            Instruction::IMPR => vec!["IMPR".to_string()],
            Instruction::AMEM(val) => vec!["AMEM".to_string(), val.to_string()],
            Instruction::DMEM(val) => vec!["DMEM".to_string(), val.to_string()],
            Instruction::INPP => vec!["INPP".to_string()],
            Instruction::CHPR(label) => {
                vec!["CHPR".to_string(), label.to_string()]
            }
            Instruction::ENPR(val) => vec!["ENPR".to_string(), val.to_string()],
            Instruction::RTPR(a, b) => vec!["RTPR".to_string(), a.to_string(), b.to_string()],
        }
    }
}

impl fmt::Display for Instruction {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Instruction::CRCT(val) => write!(f, "CRCT {}", val),
            Instruction::CRVL(a, b) => write!(f, "CRVL {} {}", a, b),
            Instruction::CREN(a, b) => write!(f, "CREN {} {}", a, b),
            Instruction::ARMZ(a, b) => write!(f, "ARMZ {} {}", a, b),
            Instruction::CRVI(a, b) => write!(f, "CRVI {} {}", a, b),
            Instruction::ARMI(a, b) => write!(f, "ARMI {} {}", a, b),
            Instruction::SOMA => write!(f, "SOMA"),
            Instruction::SUBT => write!(f, "SUBT"),
            Instruction::MULT => write!(f, "MULT"),
            Instruction::DIVI => write!(f, "DIVI"),
            Instruction::INVR => write!(f, "INVR"),
            Instruction::CONJ => write!(f, "CONJ"),
            Instruction::DISJ => write!(f, "DISJ"),
            Instruction::NEGA => write!(f, "NEGA"),
            Instruction::CMME => write!(f, "CMME"),
            Instruction::CMMA => write!(f, "CMMA"),
            Instruction::CMIG => write!(f, "CMIG"),
            Instruction::CMDG => write!(f, "CMDG"),
            Instruction::CMEG => write!(f, "CMEG"),
            Instruction::CMAG => write!(f, "CMAG"),
            Instruction::DSVS(label) => write!(f, "DSVS {}", label),
            Instruction::DSVF(label) => write!(f, "DSVF {}", label),
            Instruction::NADA => write!(f, "NADA"),
            Instruction::PARA => write!(f, "PARA"),
            Instruction::LEIT => write!(f, "LEIT"),
            Instruction::IMPR => write!(f, "IMPR"),
            Instruction::AMEM(val) => write!(f, "AMEM {}", val),
            Instruction::DMEM(val) => write!(f, "DMEM {}", val),
            Instruction::INPP => write!(f, "INPP"),
            Instruction::CHPR(label) => write!(f, "CHPR {}", label),
            Instruction::ENPR(val) => write!(f, "ENPR {}", val),
            Instruction::RTPR(a, b) => write!(f, "RTPR {} {}", a, b),
        }
    }
}
