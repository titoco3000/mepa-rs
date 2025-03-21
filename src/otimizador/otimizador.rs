use super::grafo::{CodeGraph, InstructionAndMetadata};
use crate::mepa::code::MepaCode;
use crate::mepa::instruction::Instruction;
use crate::mepa::label::Label;
use core::panic;
use petgraph::graph::NodeIndex;
use petgraph::visit::EdgeRef;
use std::io;
use std::path::{Path, PathBuf};

pub fn otimizar_arquivo<P>(filename: P) -> io::Result<()>
where
    P: AsRef<Path>,
{
    let code = otimizar(MepaCode::from_file(&filename)?);

    code.to_file(&filename).unwrap();
    Ok(())
}

pub fn otimizar(code: MepaCode) -> MepaCode {
    let c = code;
    let mut code = CodeGraph::new(c.clone());

    code.export_to_file(&PathBuf::from("output/debug/antes.dot"))
        .unwrap();

    let functions = [
        fluxo,
        elimidar_codigo_morto,
        propagar_constantes,
        eliminar_variaveis_mortas,
        // eliminar_inconsequentes
    ];

    loop {
        let mut mudou = false;

        for &func in &functions {
            while func(&mut code) {
                mudou = true;
            }
        }

        if !mudou {
            break;
        }
    }

    // code.print_vars();

    code.export_to_file(&PathBuf::from("output/debug/depois.dot"))
        .unwrap();
    code.open_browser_visualization().expect("Falha ao abrir visualizacao");

    code.to_mepa_code()
}

//se tem um desvio que cai em outro desvio, pula direto para pos final
//retorna se mudou algo
fn fluxo(code: &mut CodeGraph) -> bool {
    println!("Otimizando fluxo");
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
    println!("Eliminando codigo morto");
    let mut mudou = false;
    for node in code
    .grafo
    .node_indices(){
        println!("Verificando o node {}:", node.index());
        println!("    incoming: {}:", code.grafo.edges_directed(node, petgraph::Direction::Incoming)
        .count());
    }

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
                let line = code.instruction(*uso).unwrap();
                if matches!(line.instruction, Instruction::ARMZ(_, _)) {
                    line.armazena_em
                } else {
                    None
                }
            })
            .collect();

        //para cada alocacao-destino
        for aloc_addr in &aloc_addresses {
            // cada CRVL que usa essa instrucao

            let carregamentos: Vec<usize> = code
                .instruction(aloc_addr.0)
                .unwrap()
                .allocation
                .as_ref()
                .unwrap()
                .variaveis[aloc_addr.1]
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
                .instruction_mut(aloc_addr.0)
                .unwrap()
                .allocation
                .as_mut()
            {
                for c in &carregamentos {
                    aloc.variaveis[aloc_addr.1].usos.remove(&c);
                }
            }
            for c in carregamentos {
                if let Instruction::CRCT(n) = declaracao.instruction {
                    if let Some(line) = code.instruction_mut(c) {
                        println!(
                            "Substituindo {} da linha {} por CRCT({})",
                            line.instruction, c, n
                        );
                        line.instruction = Instruction::CRCT(n);
                        line.carrega_de = None;
                        mudou = true;
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
                for (var_idx, var) in vars.iter().enumerate() {
                    if var.referencias.len() > 0 {
                        // se possui referencias, os items subsequentes podem ser atingidos e não podem ser modificados
                        break;
                    }
                    // se não tem nenhum uso ou referencia, é variavel morta. Mas podemos eliminar?
                    if var.usos.len() == 0 {
                        let mut atribuido_por_procedimento = false;
                        // substitui todas as atribuições que não forem CHPR por DMEM(1)
                        for atribuicao in &var.atribuicoes {
                            if let Some(attr_line) = code.instruction_mut(*atribuicao) {
                                if !matches!(attr_line.instruction, Instruction::CHPR(_)){
                                    attr_line.instruction = Instruction::DMEM(1);
                                    attr_line.armazena_em = None;
                                    mudou = true;
                                }
                                else{
                                    atribuido_por_procedimento = true;
                                }
                            }
                        }
                        // se não foi atribuido por procedimento, pode remover a sua alocação
                        if !atribuido_por_procedimento{
                            mudou = true;
                            match line.instruction {
                                //se alocação for CRCT, CRVL, CREN ou CRVI, remove a instrução
                                Instruction::CRCT(_)
                                | Instruction::CRVL(_, _)
                                | Instruction::CREN(_, _)
                                | Instruction::CRVI(_, _) => {
                                    code.remove_instruction_controlled(line.address, false);
                                }
                                //se alocação for AMEM:
                                _ => {
                                    let tamanho = if let Instruction::AMEM(n) = line.instruction {
                                        n
                                    } else {
                                        panic!()
                                    };
                                    //diminui a alocação em uma unidade
                                    if let Some(line) = code.instruction_mut(line.address) {
                                        line.instruction = Instruction::AMEM(tamanho - 1);
                                        line.allocation.as_mut().unwrap().variaveis.remove(var_idx);
                                    }
    
                                    //localiza escopo
                                    let escopo = if code.get_fn_index(line.address).is_some() {
                                        1
                                    } else {
                                        0
                                    };
                                    // localiza endereço usado para acessar essa variavel
                                    let var_n = (aloc.nivel_memoria + var_idx) as i32;
    
                                    //localiza todos os usos, referencias e atribuicoes com numeros acima do removido, até a liberação
                                    let usos_subsequentes: Vec<usize> = code
                                        .instructions_between(
                                            line.address,
                                            aloc.liberation_address.unwrap(),
                                        )
                                        .filter_map(|line_between| match line_between.instruction {
                                            Instruction::CRVL(m, n)
                                            | Instruction::CREN(m, n)
                                            | Instruction::ARMZ(m, n)
                                            | Instruction::CRVI(m, n)
                                            | Instruction::ARMI(m, n) => {
                                                if m == escopo && n > var_n {
                                                    Some(line_between.address)
                                                } else {
                                                    None
                                                }
                                            }
                                            _ => None,
                                        })
                                        .collect();
                                    for uso in usos_subsequentes {
                                        //para cada um, diminui 1
                                        if let Some(uso) = code.instruction_mut(uso) {
                                            uso.instruction = match uso.instruction {
                                                Instruction::CRVL(m, n) => Instruction::CRVL(m, n - 1),
                                                Instruction::CREN(m, n) => Instruction::CREN(m, n - 1),
                                                Instruction::ARMZ(m, n) => Instruction::ARMZ(m, n - 1),
                                                Instruction::CRVI(m, n) => Instruction::CRVI(m, n - 1),
                                                Instruction::ARMI(m, n) => Instruction::ARMI(m, n - 1),
                                                _ => unreachable!(),
                                            };
                                            if let Some(a) = uso.carrega_de {
                                                uso.carrega_de = Some((a.0, a.1 - 1));
                                            }
                                            if let Some(a) = uso.ref_de {
                                                uso.ref_de = Some((a.0, a.1 - 1));
                                            }
                                            if let Some(a) = uso.armazena_em {
                                                uso.armazena_em = Some((a.0, a.1 - 1));
                                            }
                                        }
                                    }
                                    //diminui em 1 o uso de memoria de todas as instrucoes entre a aloc e o DMEM
                                    let instrucoes_subsequentes:Vec<_> = code.instructions_between(line.address,aloc.liberation_address.unwrap()).map(|line|line.address).skip(1).collect();
                                    for instrucoes_subsequente in instrucoes_subsequentes{
                                        if let Some(line) = code.instruction_mut(instrucoes_subsequente){
                                            println!("{:?}",line);
                                            if let Some(m) = line.initial_memory_usage.as_mut(){
                                                *m = *m-1;
                                            }

                                        }
                                    }


                                    println!("Reduzindo DMEM de {}",aloc.liberation_address.unwrap());
                                    // diminui um numero da DMEM (se atingir 0, remove a instrução)
                                    if let Some(dealoc) = code.instruction_mut(aloc.liberation_address.unwrap())
                                    {
                                        dealoc.instruction = Instruction::DMEM(
                                            if let Instruction::DMEM(n) = dealoc.instruction {
                                                n-1
                                            } else {
                                                unreachable!()
                                            },
                                        )
                                    }
                                    
                                    // se já removeu uma variavel, não lida com as outras para não usar info desatualizada (em lines_with_aloc_compativel)
                                    break;
                                }
                            }                        
                        }
                    }
                }
            }
        }
    }
    mudou
}

// elimina AMEM(0), DMEM(0) e NADA
fn eliminar_inconsequentes(code:&mut CodeGraph) ->bool{
    let inconsequentes:Vec<usize> = code.instructions_unordered().filter(|line|match line.instruction{
        Instruction::AMEM(n)|Instruction::DMEM(n) => n==0,
        Instruction::NADA =>true,
        _=>false
    }).map(|line|line.address).collect();

    for i in &inconsequentes{
        code.remove_instruction_controlled(*i, false);
    }
    inconsequentes.len()>0
}