use std::collections::HashMap;

use crate::mepa::{
    code::MepaCode,
    instruction::Instruction,
    label::Label,
};

pub fn remover_rotulos_simbolicos(mc: MepaCode)->MepaCode {
    let mut labels = HashMap::new();

   // Localiza todas labels
    mc.0.iter().enumerate().for_each(|(line, (label_current_line, _))| {
        if let Some(Label::Simbolic(s)) = label_current_line {
            labels.insert(s.clone(), line);
        }
    });

    // Transforma instruções usando mapa de labels
    let mut mc = MepaCode(mc.0
        .into_iter()
        .map(|(_, instruction)| {
            (
                None,
                match instruction {
                    Instruction::DSVS(label) => {
                        if let Label::Simbolic(s) = label {
                            Instruction::DSVS(Label::Literal(*labels.get(&s).unwrap()))
                        } else {
                            Instruction::DSVS(label)
                        }
                    },
                    Instruction::DSVF(label) => {
                        if let Label::Simbolic(s) = label {
                            Instruction::DSVF(Label::Literal(*labels.get(&s).unwrap()))
                        } else {
                            Instruction::DSVF(label)
                        }
                    },
                    Instruction::CHPR(label) => {
                        if let Label::Simbolic(s) = label {
                            Instruction::CHPR(Label::Literal(*labels.get(&s).unwrap()))
                        } else {
                            Instruction::CHPR(label)
                        }
                    }
                    _ => instruction,
                },
            )
        }).collect());
    
    let mut i = 0;
    while i < mc.0.len() {
        if matches!(mc.0[i].1, Instruction::NADA){
            mc.remove_instruction(i);
        }
        else {
            i+=1;
        }
    }

    mc
}
