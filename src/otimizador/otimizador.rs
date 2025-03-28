use super::grafo::{CodeGraph, InstructionAndMetadata};
use crate::mepa::code::MepaCode;
use crate::mepa::instruction::Instruction;
use crate::mepa::label::Label;
use petgraph::graph::NodeIndex;
use petgraph::visit::EdgeRef;
use std::error::Error;
use std::io;
use std::path::Path;

pub struct Otimizador<P>
where
    P: AsRef<Path>,
{
    code: CodeGraph,
    verbose_level: usize,
    file_path: Option<P>,
}

impl<P> Otimizador<P>
where
    P: AsRef<Path>,
{
    pub fn new(code: MepaCode, file_path: Option<P>) -> Result<Self, Box<dyn Error>> {
        let code = CodeGraph::new(code);
        if !code.memoria_consistente {
            return Err("Memoria inconsistente; não é possivel otimizar".into());
        }
        Ok(Otimizador {
            code,
            verbose_level: 0,
            file_path,
        })
    }

    pub fn from_file(file_path: P) -> Result<Self, Box<dyn Error>> {
        let raw_code = MepaCode::from_file(&file_path)?;
        Otimizador::new(raw_code, Some(file_path))
    }

    pub fn verbose(mut self) -> Self {
        self.verbose_level = 1;
        self
    }

    pub fn open_browser_visualization(&self) -> Result<(), std::io::Error> {
        self.code.open_browser_visualization()
    }

    pub fn otimizar(mut self) -> Self {
        let functions = [
            fluxo,
            elimidar_codigo_morto,
            propagar_constantes,
            eliminar_variaveis_mortas,
        ];

        loop {
            let mut mudou = false;

            for &func in &functions {
                while func(&mut self.code) {
                    mudou = true;
                }
            }

            if !mudou {
                break;
            }
        }
        return self;
        // code.export_to_file(&PathBuf::from("output/debug/depois.dot"))
        //     .unwrap();
        // code.open_browser_visualization()
        //     .expect("Falha ao abrir visualizacao");

        // code.to_mepa_code()
    }
    pub fn save(self) -> io::Result<()> {
        if let Some(file_path) = self.file_path {
            let code = self.code.to_mepa_code();
            code.to_file(&file_path)
        } else {
            Err(io::Error::new(
                io::ErrorKind::Other,
                "No file path provided",
            ))
        }
    }

    pub fn save_at(mut self, file_path: P) -> io::Result<()> {
        self.file_path = Some(file_path);
        self.save()
    }
}

impl<P> From<MepaCode> for Otimizador<P>
where
    P: AsRef<Path>,
{
    fn from(code: MepaCode) -> Self {
        Otimizador {
            code: CodeGraph::new(code),
            verbose_level: 0,
            file_path: None,
        }
    }
}

//se tem um desvio que cai em outro desvio, pula direto para pos final
//retorna se mudou algo
fn fluxo(code: &mut CodeGraph) -> bool {
    // cria vec de (origem, destino)
    let mut desvio_para_desvio: Vec<(usize, usize)> = code
        .grafo
        .node_indices()
        .filter_map(
            |node_index| match code.grafo.node_weight(node_index).unwrap().first() {
                Some(line) => match &line.instruction {
                    Instruction::DSVS(label) => Some((line.address, label.unwrap())),
                    _ => None,
                },
                _ => None,
            },
        )
        .collect();

    // elimina o problema entre os próprios redundantes
    for i in 0..desvio_para_desvio.len() {
        for j in 0..desvio_para_desvio.len() {
            if desvio_para_desvio[i].1 == desvio_para_desvio[j].0 {
                desvio_para_desvio[i].1 = desvio_para_desvio[j].1;
            }
        }
    }

    code.grafo.node_indices().any(|node| {
        let mut mudancas = None; //(old, new)
        let last_line = code
            .grafo
            .node_weight_mut(node)
            .unwrap()
            .iter_mut()
            .last()
            .unwrap();
        match &mut last_line.instruction {
            Instruction::DSVS(label) | Instruction::DSVF(label) | Instruction::CHPR(label) => {
                for inutil in &desvio_para_desvio {
                    if inutil.0 == label.unwrap() {
                        mudancas = Some((label.unwrap(), inutil.1));
                        *label = Label::Literal(inutil.1);
                        break;
                    }
                }
            }
            _ => {}
        }
        if let Some((old_label, new_label)) = mudancas {
            let old_node_index = code.locate_address(old_label).unwrap();
            let new_node_index = code.locate_address(new_label).unwrap();

            // Remove aresta antiga
            code.grafo.remove_edge(
                code.grafo
                    .edges(node)
                    .find(|edge| edge.source() == node && edge.target() == old_node_index)
                    .map(|edge| edge.id())
                    .unwrap(),
            );
            // verifica se ja existe uma equivalente a nova
            if let None = code
                .grafo
                .edges(node)
                .find(|edge| edge.source() == node && edge.target() == new_node_index)
                .map(|edge| edge.id())
            {
                // Adiciona substituta
                code.grafo.add_edge(node, new_node_index, ());
            }
            true
        } else {
            false
        }
    })
}

//se tem codigo inacessivel, remove
fn elimidar_codigo_morto(code: &mut CodeGraph) -> bool {
    let mut mudou = false;

    // acha nodes inacessiveis
    let inacessiveis: Vec<NodeIndex> = code
        .grafo
        .node_indices()
        .filter(|node_idx| {
            code.grafo
                .edges_directed(*node_idx, petgraph::Direction::Incoming)
                .count()
                == 0
                && {
                    let line = code.grafo.node_weight(*node_idx).unwrap().first().unwrap();
                    // node inicial não deve ser removido
                    if line.address == 0 {
                        false
                    } else {
                        match line.instruction {
                            // se é entrada de uma função
                            Instruction::ENPR(_) => {
                                // remove se não tiver nenhuma chamada
                                code.funcoes
                                    .iter()
                                    .find(|f| f.addr_inicio == line.address)
                                    .unwrap()
                                    .usos
                                    .len()
                                    == 0
                            }
                            // se não pode remover
                            _ => true,
                        }
                    }
                }
        })
        .collect();

    for i in inacessiveis {
        code.remove_node(i);
        mudou = true;
    }
    if mudou {
        code.mapear_memoria();
    }
    mudou
}

fn propagar_constantes(code: &mut CodeGraph) -> bool {
    if !code.memoria_consistente {
        return false;
    }
    let mut mudou = false;
    // localiza todos CRCT
    let declaracoes_de_constantes: Vec<InstructionAndMetadata> = code
        .instructions_unordered()
        .filter_map(|line| {
            if matches!(line.instruction, Instruction::CRCT(_)) {
                Some(line.clone())
            } else {
                None
            }
        })
        .collect();
    // para cada um, localiza todos os usos
    for declaracao in declaracoes_de_constantes {
        // Para o ARMZ que usa esse valor (que será ou um ou zero)
        let aloc_addresses: Vec<(usize, usize)> = declaracao.allocation.unwrap().variaveis[0]
            .usos
            .iter()
            .filter_map(|uso| {
                let (inst, addr) = if let Some(line) = code.instruction(*uso) {
                    (line.instruction.clone(), line.address)
                } else {
                    panic!()
                };
                if matches!(inst, Instruction::ARMZ(_, _)) {
                    code.linhas_atribuidas_por(addr).next()
                } else {
                    None
                }
            })
            .collect();

        //para cada alocacao-destino
        for (aloc_addr, var) in &aloc_addresses {
            // verifica se a atribuição é unica
            let n_atribuicoes = code
                .instruction(*aloc_addr)
                .as_ref()
                .unwrap()
                .allocation
                .as_ref()
                .unwrap()
                .variaveis[*var]
                .atribuicoes
                .len();

            if n_atribuicoes <= 1 {
                // cada CRVL que usa essa instrucao

                let carregamentos: Vec<usize> = code
                    .instruction(*aloc_addr)
                    .unwrap()
                    .allocation
                    .as_ref()
                    .unwrap()
                    .variaveis[*var]
                    .usos
                    .clone()
                    .iter()
                    .filter_map(|addr| {
                        let line = code.instruction(*addr).unwrap();
                        if matches!(line.instruction, Instruction::CRVL(_, _)) {
                            Some(line.address.clone())
                        } else {
                            None
                        }
                    })
                    .collect();

                // remove usos da lista, ja que vao ser removidos
                if let Some(aloc) = &mut code
                    .instruction_mut(*aloc_addr)
                    .unwrap()
                    .allocation
                    .as_mut()
                {
                    for c in &carregamentos {
                        aloc.variaveis[*var].usos.remove(&c);
                    }
                }
                for c in carregamentos {
                    if let Instruction::CRCT(n) = declaracao.instruction {
                        if let Some(line) = code.instruction_mut(c) {
                            line.instruction = Instruction::CRCT(n);
                            mudou = true;
                        }
                    }
                }
            }
        }
    }
    mudou
}

fn eliminar_variaveis_mortas(code: &mut CodeGraph) -> bool {
    if !code.memoria_consistente {
        return false;
    }
    let mut mudou = false;

    // localiza alocações
    let lines_with_aloc_compativel: Vec<_> = code
        .instructions_unordered()
        .filter_map(|line| match line.instruction {
            Instruction::CRCT(_)
            | Instruction::CRVL(_, _)
            | Instruction::CREN(_, _)
            | Instruction::CRVI(_, _)
            | Instruction::AMEM(_) => Some(line.clone()),
            _ => None,
        })
        .collect();

    for line in lines_with_aloc_compativel {
        if let Some(aloc) = line.allocation {
            // Só pode remover variaveis se a sua liberação não usar o seu valor;
            // ou seja, se for DMEM (teoricamente incluiria RTPR também, a ser explorado no futuro);
            if matches!(
                code.instruction(aloc.liberation_address.unwrap())
                    .unwrap()
                    .instruction,
                Instruction::DMEM(_)
            ) {
                let vars = aloc.variaveis;
                // para cada variavel de cada alocação
                for (_var_idx, var) in vars.iter().enumerate() {
                    if var.referencias.len() > 0 {
                        // se possui referencias, os items subsequentes podem ser atingidos e não podem ser modificados
                        break;
                    }
                    // se não tem nenhum uso ou referencia, é variavel morta. Mas podemos eliminar?
                    if var.usos.len() == 0 {
                        let mut algum_atribuido_por_procedimento = false;
                        let mut usos = Vec::new();
                        // substitui todas as atribuições que não forem CHPR por DMEM(1)
                        for atribuicao in &var.atribuicoes {
                            // carrega line mut
                            if let Some(attr_line) = code.instruction_mut(*atribuicao) {
                                // se for CHPR, atualiza flag
                                if matches!(attr_line.instruction, Instruction::CHPR(_)) {
                                    algum_atribuido_por_procedimento = true;
                                }
                                // se não:
                                else {
                                    // substitui por DMEM(1)
                                    attr_line.instruction = Instruction::DMEM(1);
                                    // encontra todos os lugares de onde carregava
                                    usos = code.linhas_usadas_por(*atribuicao).collect();

                                    mudou = true;
                                }
                            }
                            // remove os usos
                            for (aloc_addr, var) in &usos {
                                code.instruction_mut(*aloc_addr)
                                    .as_mut()
                                    .unwrap()
                                    .allocation
                                    .as_mut()
                                    .unwrap()
                                    .variaveis[*var]
                                    .usos
                                    .remove(atribuicao);
                            }
                        }
                        // se não foi atribuido por procedimento, pode remover a sua alocação
                        if !algum_atribuido_por_procedimento {
                            mudou = true;
                            code.decrease_memory_alocation(line.address);
                        }
                    }
                }
            }
        }
    }
    mudou
}

// elimina AMEM(0), DMEM(0) e NADA
// fn eliminar_inconsequentes(code: &mut CodeGraph) -> bool {
//     let inconsequentes: Vec<usize> = code
//         .instructions_unordered()
//         .filter(|line| match line.instruction {
//             Instruction::AMEM(n) | Instruction::DMEM(n) => n == 0,
//             Instruction::NADA => true,
//             _ => false,
//         })
//         .map(|line| line.address)
//         .collect();

//     for i in &inconsequentes {
//         code.remove_instruction_controlled(*i, false);
//     }
//     inconsequentes.len() > 0
// }
