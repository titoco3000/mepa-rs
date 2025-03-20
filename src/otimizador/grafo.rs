use crate::mepa::code::MepaCode;
use crate::mepa::instruction::Instruction;
use crate::mepa::label::Label;
use petgraph::dot::{Config, Dot};
use petgraph::graph::NodeIndex;
use petgraph::visit::{Dfs, EdgeRef};
use petgraph::Graph;
use std::collections::HashMap;
use std::collections::HashSet;
use std::fs::{self, File};
use std::io::{self, Write};
use std::path::PathBuf;
use std::usize;

#[derive(Debug, Clone)]
pub struct Variavel {
    pub atribuicoes: HashSet<usize>,
    pub usos: HashSet<usize>,
    pub referencias: HashSet<usize>,
}

impl Variavel {
    pub fn new() -> Variavel {
        Variavel {
            atribuicoes: HashSet::new(),
            usos: HashSet::new(),
            referencias: HashSet::new(),
        }
    }
    pub fn new_vec(amount: usize) -> Vec<Variavel> {
        (0..amount).map(|_| Variavel::new()).collect()
    }
}

#[derive(Debug, Clone)]
pub struct Allocation {
    pub addr: usize,
    pub liberation_address: Option<usize>,
    pub nivel_memoria: usize,
    pub variaveis: Vec<Variavel>,
}

impl Allocation {
    pub fn new(addr: usize, amount: usize, nivel_memoria: usize) -> Allocation {
        Allocation {
            addr,
            liberation_address: None,
            variaveis: Variavel::new_vec(amount),
            nivel_memoria,
        }
    }
    pub fn liberate(mut self, addr: usize) -> Self {
        self.liberation_address = Some(addr);
        self
    }
}

#[derive(Debug, Clone)]
pub struct InstructionAndMetadata {
    pub address: usize,
    pub instruction: Instruction,
    pub initial_memory_usage: Option<usize>,
    pub allocation: Option<Allocation>,
    pub armazena_em: Option<(usize, usize)>, //addr de alocacao, addr da variavel
    pub carrega_de: Option<(usize, usize)>,
    pub ref_de: Option<(usize, usize)>,
}
impl InstructionAndMetadata {
    pub fn strip_metadata(&mut self){
        self.initial_memory_usage = None;
        self.allocation = None;
        self.armazena_em = None;
        self.carrega_de = None;
        self.ref_de = None;
    }
}
#[derive(Debug, Clone)]
pub struct FuncMetadata {
    pub addr_inicio: usize,
    pub addr_retorno: usize,
    pub atribuicao_memoria_externa: HashSet<i32>,
    pub uso_memoria_externa: HashSet<i32>,
    pub referencias_memoria_externa: HashSet<i32>,
    pub args: usize,
    pub usos: HashSet<usize>,
}

pub struct CodeGraph {
    pub grafo: Graph<Vec<InstructionAndMetadata>, ()>,
    pub funcoes: Vec<FuncMetadata>,
    pub memoria_consistente: bool,
}

impl CodeGraph {
    pub fn new(mut code: MepaCode) -> CodeGraph {
        // este pré-processamento deve ser feito para que a montagem do grafo funcione normalmente
        code = remover_rotulos_simbolicos(code);

        let mut grafo = CodeGraph {
            grafo: Graph::new(),
            funcoes: Vec::new(),
            memoria_consistente: false,
        };

        let mut lideres: Vec<usize> = code
            .iter()
            .enumerate()
            .flat_map(|(i, (_, code))| {
                if i == 0 {
                    // The first index is always a leader
                    Some(vec![0])
                } else {
                    match code {
                        Instruction::DSVF(label) | Instruction::DSVS(label) => {
                            if let Label::Literal(addr) = label {
                                Some(vec![i + 1, *addr]) // Add the next instruction and jump address
                            } else {
                                panic!("Should never reach here");
                            }
                        }
                        Instruction::RTPR(_, _) => Some(vec![i + 1]), // Add the next instruction
                        _ => None,
                    }
                }
            })
            .flatten()
            .collect::<HashSet<_>>()
            .into_iter()
            .collect();

        lideres.sort_unstable();

        let vertices: Vec<NodeIndex> = lideres
            .iter()
            .zip(lideres.iter().skip(1).chain(std::iter::once(&code.len())))
            .map(|(&start, &end)| {
                // map the instructions from start to end into a new vec
                let instructions: Vec<InstructionAndMetadata> = (start..end)
                    .map(|addr| InstructionAndMetadata {
                        address: addr,
                        instruction: code[addr].1.clone(),
                        initial_memory_usage: None,
                        allocation: None,
                        carrega_de: None,
                        armazena_em: None,
                        ref_de: None,
                    })
                    .collect();
                grafo.grafo.add_node(instructions)
            })
            .collect();

        let mut arestas: Vec<(NodeIndex, NodeIndex)> = Vec::with_capacity(code.len());

        for block_index in &vertices {
            let linhas = grafo.grafo.node_weight(*block_index).unwrap();

            for linha in linhas {
                match &linha.instruction {
                    Instruction::DSVS(label) => {
                        // adiciona o vertice que contem o endereco label
                        if let Label::Literal(jmp_addr) = label {
                            let target_block = grafo
                                .locate_address(*jmp_addr)
                                .expect("Address not found in groupings");
                            arestas.push((*block_index, target_block));
                        } else {
                            panic!("não removeu labels simbolicos")
                        }
                    }
                    Instruction::DSVF(label) => {
                        // adiciona o vertice que contem o endereco label e o da proxima instrucao
                        if let Label::Literal(jmp_addr) = label {
                            let target_block = grafo
                                .locate_address(*jmp_addr)
                                .expect("Address not found in groupings");
                            arestas.push((*block_index, target_block));
                        } else {
                            panic!("não removeu labels simbolicos")
                        }
                        if let Some(target_block) = grafo.locate_address(linha.address + 1) {
                            arestas.push((*block_index, target_block));
                        }
                    }
                    Instruction::PARA | Instruction::RTPR(_, _) => (),
                    _ => {
                        if linha.address == linhas.last().unwrap().address {
                            // se é a ultima linha do bloco e não é nenhum pulo, então é um pulo para o próximo
                            if let Some(target_block) = grafo.locate_address(linha.address + 1) {
                                arestas.push((*block_index, target_block));
                            }
                        }
                    }
                }
            }
        }

        grafo.grafo.extend_with_edges(&arestas);

        // mapeia o uso de memoria
        grafo.mapear_memoria();

        grafo
    }

    pub fn mapear_memoria(&mut self) {
        self.mapear_funcoes();
        if !self.memoria_consistente {
            return;
        }
        // limpa a memoria de todos as linhas

        for line in self.instructions_mut(){
            line.strip_metadata();
        }


        let mut bases_mapeamento: Vec<(usize, usize)> = self
            .funcoes
            .iter()
            .map(|func| (func.addr_inicio, 1)) // este +1 é para contar com a memoria alocada por CHPR
            .chain(std::iter::once((0, 0)))
            .rev()
            .collect();

        // origem, endereco
        let mut atribuicoes_globais: HashSet<(usize, usize)> = HashSet::new();
        let mut usos_globais: HashSet<(usize, usize)> = HashSet::new();
        let mut referencias_globais: HashSet<(usize, usize)> = HashSet::new();

        while let Some((addr, initial_value)) = bases_mapeamento.pop() {
            let raiz = self.locate_address(addr).unwrap();
            let current_func = self.get_fn_index(self.grafo.node_weight(raiz).unwrap()[0].address);
            // valores da instrucao
            for line in self.grafo.node_weight_mut(raiz).unwrap() {
                if line.address == addr {
                    line.initial_memory_usage = Some(initial_value);
                    line.allocation = None;
                }
            }

            let mut nodes_stack = Vec::with_capacity(self.grafo.node_count());
            nodes_stack.push(raiz);
            let mut visited_nodes = Vec::with_capacity(self.grafo.node_count());

            // let mut dfs = Dfs::new(&self.grafo, raiz);

            let mut alocation_stack: Vec<Allocation> = Vec::new();
            // (addr, aloc)
            let mut alocation_map = Vec::new();

            while let Some(visited) = nodes_stack.pop() {
                let lines = if visited == raiz {
                    let lines = self.grafo.node_weight_mut(visited).unwrap();
                    let first_addr = lines[0].address;
                    &mut lines[addr - first_addr..]
                } else {
                    self.grafo.node_weight_mut(visited).unwrap()
                };

                println!("lines: ");
                for l in lines.iter(){
                    println!("{}: {:?}",l.address, l.instruction);
                }
                let mut memory: usize = lines.first().unwrap().initial_memory_usage.unwrap();
                for line_idx in 0..lines.len() {
                    let memory_delta = match &lines[line_idx].instruction {
                        Instruction::CRCT(_) | Instruction::LEIT | Instruction::ENPR(_) => 1,
                        Instruction::CRVL(nivel_lexico, nivel_memoria)
                        | Instruction::CRVI(nivel_lexico, nivel_memoria) => {
                            println!("{:?}", lines[line_idx]);
                            if *nivel_lexico == 1 {
                                if let Some(_) = current_func {
                                    let endereco_real = nivel_memoria + 2;
                                    if endereco_real >= 0 {
                                        let achou = alocation_stack.iter_mut().rev().any(|item| {
                                            let endereco_relativo =
                                                endereco_real - item.nivel_memoria as i32;
                                            if endereco_relativo >= 0 {
                                                lines[line_idx].carrega_de =
                                                    Some((item.addr, endereco_relativo as usize));
                                                item.variaveis[endereco_relativo as usize]
                                                    .usos
                                                    .insert(lines[line_idx].address);
                                                true
                                            } else {
                                                false
                                            }
                                        });
                                        if !achou {
                                            self.memoria_consistente = false;
                                            return;
                                        }
                                    }
                                } else {
                                    panic!(
                                        "Instrução em escopo global tentou acessar nivel lexico {}",
                                        nivel_lexico
                                    );
                                }
                            } else {
                                usos_globais
                                    .insert((lines[line_idx].address, *nivel_memoria as usize));
                            }
                            1
                        }
                        Instruction::CREN(nivel_lexico, nivel_memoria) => {
                            if *nivel_lexico == 1 {
                                if let Some(_) = current_func {
                                    let endereco_real = nivel_memoria + 2;
                                    if endereco_real >= 0 {
                                        let achou = alocation_stack.iter_mut().rev().any(|item| {
                                            let endereco_relativo =
                                                endereco_real - item.nivel_memoria as i32;
                                            if endereco_relativo >= 0 {
                                                lines[line_idx].ref_de =
                                                    Some((item.addr, endereco_relativo as usize));
                                                item.variaveis[endereco_relativo as usize]
                                                    .referencias
                                                    .insert(lines[line_idx].address);
                                                true
                                            } else {
                                                false
                                            }
                                        });
                                        if !achou {
                                            self.memoria_consistente = false;
                                            return;
                                        }
                                    }
                                } else {
                                    panic!(
                                        "Instrução em escopo global tentou acessar nivel lexico {}",
                                        nivel_lexico
                                    );
                                }
                            } else {
                                referencias_globais
                                    .insert((lines[line_idx].address, *nivel_memoria as usize));
                            }
                            1
                        }
                        Instruction::ARMZ(nivel_lexico, nivel_memoria) => {
                            if *nivel_lexico == 1 {
                                if let Some(_) = current_func {
                                    let endereco_real = nivel_memoria + 2;
                                    if endereco_real >= 0 {
                                        let achou = alocation_stack.iter_mut().rev().any(|item| {
                                            let endereco_relativo =
                                                endereco_real - item.nivel_memoria as i32;
                                            if endereco_relativo >= 0 {
                                                lines[line_idx].armazena_em =
                                                    Some((item.addr, endereco_relativo as usize));
                                                println!("{:?}",lines[line_idx]);
                                                item.variaveis[endereco_relativo as usize]
                                                    .atribuicoes
                                                    .insert(lines[line_idx].address);
                                                true
                                            } else {
                                                false
                                            }
                                        });
                                        if !achou {
                                            self.memoria_consistente = false;
                                            return;
                                        }
                                    }
                                } else {
                                    panic!(
                                        "Instrução em escopo global tentou acessar nivel lexico {}",
                                        nivel_lexico
                                    );
                                }
                            } else {
                                atribuicoes_globais
                                    .insert((lines[line_idx].address, *nivel_memoria as usize));
                            }
                            -1
                        }
                        //operadores de dois args
                        Instruction::SOMA
                        | Instruction::SUBT
                        | Instruction::MULT
                        | Instruction::DIVI
                        | Instruction::CONJ
                        | Instruction::DISJ
                        | Instruction::CMME
                        | Instruction::CMMA
                        | Instruction::CMIG
                        | Instruction::CMDG
                        | Instruction::CMEG
                        | Instruction::CMAG => {
                            //registra atribuição ao primeiro argumento
                            let endereco_real = memory as i32 - 2;
                            let achou = alocation_stack.iter_mut().rev().any(|item| {
                                let endereco_relativo = endereco_real - item.nivel_memoria as i32;
                                if endereco_relativo >= 0 {
                                    lines[line_idx].armazena_em =
                                        Some((item.addr, endereco_relativo as usize));
                                    item.variaveis[endereco_relativo as usize]
                                        .atribuicoes
                                        .insert(lines[line_idx].address);
                                    true
                                } else {
                                    false
                                }
                            });
                            if !achou {
                                self.memoria_consistente = false;
                                return;
                            }
                            -1
                        }
                        Instruction::ARMI(nivel_lexico, nivel_memoria) => {
                            println!("{:?}", lines[line_idx]);
                            if *nivel_lexico == 1 {
                                if let Some(_) = current_func {
                                    let endereco_real = nivel_memoria + 2;
                                    if endereco_real >= 0 {
                                        let achou = alocation_stack.iter_mut().rev().any(|item| {
                                            let endereco_relativo =
                                                endereco_real - item.nivel_memoria as i32;
                                            if endereco_relativo >= 0 {
                                                lines[line_idx].carrega_de =
                                                    Some((item.addr, endereco_relativo as usize));
                                                item.variaveis[endereco_relativo as usize]
                                                    .usos
                                                    .insert(lines[line_idx].address);
                                                true
                                            } else {
                                                false
                                            }
                                        });
                                        if !achou {
                                            self.memoria_consistente = false;
                                            return;
                                        }
                                    }
                                } else {
                                    panic!(
                                        "Instrução em escopo global tentou acessar nivel lexico {}",
                                        nivel_lexico
                                    );
                                }
                            } else {
                                usos_globais
                                    .insert((lines[line_idx].address, *nivel_memoria as usize));
                            }
                            -1
                        }
                        Instruction::DSVF(_) | Instruction::IMPR => -1,
                        Instruction::AMEM(n) => *n,
                        Instruction::DMEM(n) => -n,
                        Instruction::RTPR(_, _n) => -2, // ignora o 'n' para só ser considerado na chamada
                        Instruction::CHPR(k) => {
                            println!("Chamada a  funcao {}", k);
                            //localiza a funcao
                            let func = self
                                .funcoes
                                .iter_mut()
                                .find(|f| f.addr_inicio == k.unwrap())
                                .unwrap();

                            // registra uso
                            func.usos.insert(lines[line_idx].address);

                            // registra acessos
                            for acesso in func.atribuicao_memoria_externa.iter() {
                                let endereco_real = memory as i32 + acesso;
                                let achou = alocation_stack.iter_mut().rev().any(|item| {
                                    let endereco_relativo =
                                        endereco_real - item.nivel_memoria as i32;
                                    if endereco_relativo >= 0 {
                                        lines[line_idx].armazena_em =
                                            Some((item.addr, endereco_relativo as usize));
                                        item.variaveis[endereco_relativo as usize]
                                            .atribuicoes
                                            .insert(lines[line_idx].address);
                                        true
                                    } else {
                                        false
                                    }
                                });
                                if !achou {
                                    self.memoria_consistente = false;
                                    return;
                                }
                            }
                            for acesso in func.uso_memoria_externa.iter() {
                                let endereco_real = memory as i32 + acesso;
                                let achou = alocation_stack.iter_mut().rev().any(|item| {
                                    let endereco_relativo =
                                        endereco_real - item.nivel_memoria as i32;
                                    if endereco_relativo >= 0 {
                                        lines[line_idx].carrega_de =
                                            Some((item.addr, endereco_relativo as usize));
                                        item.variaveis[endereco_relativo as usize]
                                            .usos
                                            .insert(lines[line_idx].address);
                                        true
                                    } else {
                                        false
                                    }
                                });
                                if !achou {
                                    self.memoria_consistente = false;
                                    return;
                                }
                            }
                            for acesso in func.referencias_memoria_externa.iter() {
                                let endereco_real = memory as i32 + acesso;
                                let achou = alocation_stack.iter_mut().rev().any(|item| {
                                    let endereco_relativo =
                                        endereco_real - item.nivel_memoria as i32;
                                    if endereco_relativo >= 0 {
                                        lines[line_idx].ref_de =
                                            Some((item.addr, endereco_relativo as usize));
                                        item.variaveis[endereco_relativo as usize]
                                            .referencias
                                            .insert(lines[line_idx].address);
                                        true
                                    } else {
                                        false
                                    }
                                });
                                if !achou {
                                    self.memoria_consistente = false;
                                    return;
                                }
                            }

                            //libera qtd de args
                            -(func.args as i32)
                        }
                        _ => 0,
                    };
                    // verifica se memoria total não será negativa, caso que invalida mapeamento
                    if memory_delta < 0 && memory < (-memory_delta) as usize {
                        self.memoria_consistente = false;
                        return;
                    }
                    memory = (memory as i32 + memory_delta) as usize;
                    if memory_delta > 0 {
                        alocation_stack.push(Allocation::new(
                            lines[line_idx].address,
                            memory_delta as usize,
                            memory - memory_delta as usize,
                        ));
                    } else if memory_delta < 0 {
                        // se for uma desaloc que usa o valor, adiciona como uso
                        match &lines[line_idx].instruction {
                            Instruction::DMEM(_) | Instruction::RTPR(_, _) => (),
                            _ => {
                                if let Some(item) = alocation_stack.iter_mut().last() {
                                    item.variaveis[0].usos.insert(lines[line_idx].address);
                                } else {
                                    self.memoria_consistente = false;
                                    return;
                                }
                            }
                        }

                        // Enquanto tiver itens no stack
                        let mut a_liberar = memory_delta.abs();
                        while let Some(top) = alocation_stack.pop() {
                            // se ainda pode liberar mais
                            if a_liberar > 0 {
                                a_liberar -= top.variaveis.len() as i32;

                                let mut aloc = top.liberate(lines[line_idx].address);

                                // se for uma variavel global
                                if let None = current_func {
                                    usos_globais.retain(|&(addr_causador, nivel_memoria)| {
                                        if nivel_memoria >= memory
                                            && nivel_memoria <= memory + aloc.variaveis.len()
                                        {
                                            aloc.variaveis[nivel_memoria - memory]
                                                .usos
                                                .insert(addr_causador);
                                            false
                                        } else {
                                            true
                                        }
                                    });
                                    atribuicoes_globais.retain(
                                        |&(addr_causador, nivel_memoria)| {
                                            if nivel_memoria >= memory
                                                && nivel_memoria <= memory + aloc.variaveis.len()
                                            {
                                                aloc.variaveis[nivel_memoria - memory]
                                                    .atribuicoes
                                                    .insert(addr_causador);
                                                false
                                            } else {
                                                true
                                            }
                                        },
                                    );
                                    referencias_globais.retain(
                                        |&(addr_causador, nivel_memoria)| {
                                            if nivel_memoria >= memory
                                                && nivel_memoria <= memory + aloc.variaveis.len()
                                            {
                                                aloc.variaveis[nivel_memoria - memory]
                                                    .referencias
                                                    .insert(addr_causador);
                                                false
                                            } else {
                                                true
                                            }
                                        },
                                    );
                                }

                                alocation_map.push(aloc);

                                if a_liberar < 0 {
                                    self.memoria_consistente = false;
                                    return;
                                }
                            }
                            // se não, devolve a pilha e encerra
                            else {
                                alocation_stack.push(top);
                                break;
                            }
                        }
                    }
                    // se houver proximo (não for ultima passada do loop), adiciona memoria inicial ao proximo
                    if line_idx + 1 < lines.len() {
                        lines[line_idx + 1].initial_memory_usage = Some(memory as usize);
                    }
                }
                let neighbors: Vec<NodeIndex> =
                    self.grafo.neighbors(visited).map(|n| n.clone()).collect();
                println!("neghbor count: {}",neighbors.len());
                // Propaga para os vizinhos
                for neighbor_index in neighbors.iter() {
                    let neighbor = self
                        .grafo
                        .node_weight_mut(*neighbor_index)
                        .unwrap()
                        .first_mut()
                        .unwrap();
                    if let Some(existing_value) = neighbor.initial_memory_usage {
                        if existing_value != memory as usize {
                            // falhou o mapeamento
                            self.memoria_consistente = false;
                            return;
                        }
                    }
                    neighbor.initial_memory_usage = Some(memory as usize);
                }

                visited_nodes.push(visited);

                let mut neighbors: Vec<NodeIndex> =
                    self.grafo.neighbors(visited).into_iter().collect();

                // faz antes nodes que voltam ao atual
                neighbors.sort_by_key(|&n| {
                    petgraph::algo::has_path_connecting(&self.grafo, n, visited, None)
                });

                for neighbor_index in neighbors {
                    //propaga valores de memoria
                    let neighbor = self
                        .grafo
                        .node_weight_mut(neighbor_index)
                        .unwrap()
                        .first_mut()
                        .unwrap();
                    if let Some(existing_value) = neighbor.initial_memory_usage {
                        if existing_value != memory as usize {
                            // falhou o mapeamento
                            self.memoria_consistente = false;
                        }
                    }
                    neighbor.initial_memory_usage = Some(memory as usize);
                    if !visited_nodes.contains(&neighbor_index)
                        && !nodes_stack.contains(&neighbor_index)
                    {
                        nodes_stack.push(neighbor_index);
                    }
                }
            }

            for aloc in alocation_map {
                let addr = aloc.addr;
                self.instruction_mut(addr).unwrap().allocation = Some(aloc);
            }
        }
        self.debug_print();
    }

    // mapeia todas as informações de funções, exceto quando são chamadas
    pub fn mapear_funcoes(&mut self) {
        let mut memoria_consistente = true;
        self.funcoes = self
            .instructions_unordered()
            // localiza todos os nodes que são inicio de função
            .filter_map(|line| match line.instruction {
                Instruction::ENPR(_) => Some(line.address),
                _ => None,
            })
            // para cada um deles, encontra:
            // - o endereço de retorno
            // - o numero de argumentos
            // - os acessos externos
            .map(|addr_inicio| {
                let node_func = self.locate_address(addr_inicio).unwrap();
                let mut atribuicao_memoria_externa: HashSet<i32> = HashSet::new();
                let mut uso_memoria_externa: HashSet<i32> = HashSet::new();
                let mut referencias_memoria_externa: HashSet<i32> = HashSet::new();
                let mut args = 0;
                let mut addr_retorno = 0;

                let mut dfs = Dfs::new(&self.grafo, node_func);
                while let Some(visited) = dfs.next(&self.grafo) {
                    for line in self.grafo.node_weight(visited).unwrap() {
                        match line.instruction {
                            Instruction::ARMZ(nivel_lexico, nivel_memoria) => {
                                if nivel_lexico == 1 && nivel_memoria < 0 {
                                    if nivel_memoria > -3 {
                                        memoria_consistente = false;
                                    } else {
                                        atribuicao_memoria_externa.insert(nivel_memoria + 2);
                                    }
                                }
                            }
                            Instruction::CRVL(nivel_lexico, nivel_memoria) => {
                                if nivel_lexico == 1 && nivel_memoria < 0 {
                                    if nivel_memoria > -3 {
                                        memoria_consistente = false;
                                    } else {
                                        uso_memoria_externa.insert(nivel_memoria + 2);
                                    }
                                }
                            }
                            Instruction::CREN(nivel_lexico, nivel_memoria) => {
                                if nivel_lexico == 1 && nivel_memoria < 0 {
                                    if nivel_memoria > -3 {
                                        memoria_consistente = false;
                                    } else {
                                        referencias_memoria_externa.insert(nivel_memoria + 2);
                                    }
                                }
                            }
                            Instruction::RTPR(_, k) => {
                                addr_retorno = line.address;
                                args = k as usize;
                            }
                            _ => (),
                        }
                    }
                }
                FuncMetadata {
                    addr_inicio,
                    addr_retorno,
                    atribuicao_memoria_externa,
                    uso_memoria_externa,
                    referencias_memoria_externa,
                    args,
                    usos: HashSet::new(),
                }
            })
            .collect();
        self.memoria_consistente = memoria_consistente;
    }

    pub fn mapear_memoria_a_partir_de(&mut self, _addr: usize, _initial_value: usize) -> bool {
        panic!("Talvez implementar isso depois");
    }

    pub fn locate_address(&self, addr: usize) -> Option<NodeIndex> {
        self.grafo.node_indices().find(|&output_index| {
            if let Some(lines) = self.grafo.node_weight(output_index) {
                lines.iter().any(|line| line.address == addr)
            } else {
                false
            }
        })
    }

    pub fn instruction_mut(&mut self, addr: usize) -> Option<&mut InstructionAndMetadata> {
        self.grafo
            .node_weights_mut()
            .flat_map(|node_instructions| node_instructions.iter_mut())
            .find(|f| f.address == addr)
    }

    pub fn instruction(&mut self, addr: usize) -> Option<&InstructionAndMetadata> {
        self.grafo
            .node_weights()
            .flat_map(|node_instructions| node_instructions.iter())
            .find(|f| f.address == addr)
    }

    pub fn instructions_mut(&mut self) -> impl Iterator<Item = &mut InstructionAndMetadata> {
        // Collect all instructions from all nodes
        let mut instructions: Vec<_> = self
            .grafo
            .node_weights_mut()
            .flat_map(|node_instructions| node_instructions.iter_mut())
            .collect();

        // Sort instructions by address
        instructions.sort_by_key(|instr| instr.address);

        // Return an iterator over mutable references
        instructions.into_iter()
    }

    pub fn instructions_unordered(&self) -> impl Iterator<Item = &InstructionAndMetadata> {
        self.grafo
            .node_weights()
            .flat_map(|node_instructions| node_instructions.iter())
    }

    pub fn instructions_between(
        &self,
        addr_inicio: usize,
        addr_fim: usize,
    ) -> impl Iterator<Item = &InstructionAndMetadata> {
        let mut visited = HashSet::new();
        let mut stack = Vec::new();

        let start_node = self.locate_address(addr_inicio).unwrap();
        let mut end_node = start_node;
        stack.push(start_node.clone());

        while let Some(node) = stack.pop() {
            if self
                .grafo
                .node_weight(node)
                .unwrap()
                .iter()
                .all(|line| line.address != addr_fim)
            {
                for vizinho in self.grafo.neighbors(node) {
                    if !visited.contains(&vizinho) && stack.contains(&vizinho) {
                        stack.push(vizinho);
                    }
                }
            } else {
                end_node = node.clone();
            }
            visited.insert(node);
        }
        visited
            .into_iter()
            .map(move |node| {
                self.grafo
                    .node_weight(node)
                    .unwrap()
                    .iter()
                    .skip_while(move |inst| node == start_node && inst.address < addr_inicio)
                    .take_while(move |inst| node != end_node || inst.address <= addr_fim)
            })
            .flatten()
    }

    pub fn insert_instruction(&mut self, addr:usize, intruction:Instruction){

    }

    pub fn get_fn_index(&self, addr: usize) -> Option<usize> {
        self.funcoes.iter().enumerate().find_map(|(i, f)| {
            if addr >= f.addr_inicio && addr <= f.addr_retorno {
                Some(i)
            } else {
                None
            }
        })
    }

    // pub fn replace_instruction(&mut self, addr: usize, nova:Instruction) {
    //     if let Some(line) = self.instruction(addr).cloned(){
    //         match line.instruction {
                
    //         }
    //     }
    // }

    pub fn remove_instruction_controlled(&mut self, addr: usize, remap: bool) {
        if remap && self.memoria_consistente {
            if let Some(memory_usage_before) =
                self.instruction_mut(addr).unwrap().initial_memory_usage
            {
                //remapeia instrucoes atingiveis por esta, considerando que ela não vai mais afetar a execucao
                self.mapear_memoria_a_partir_de(addr + 1, memory_usage_before);
            }
        }

        let node = self.locate_address(addr).unwrap();
        let line = {
            let lines = self
                .grafo
                .node_weight_mut(self.locate_address(addr).unwrap())
                .unwrap();
            let i = lines.iter().position(|l| l.address == addr).unwrap();
            //remove a linha
            lines.remove(i)
        };

        //aponta qqr desvio q apontaria para ela para a proxima
        for line in self.instructions_mut() {
            match &mut line.instruction {
                Instruction::DSVS(label) | Instruction::DSVF(label) | Instruction::CHPR(label) => {
                    if label.unwrap() == addr {
                        *label = Label::Literal(addr + 1);
                    }
                }
                _ => {}
            }
        }

        // transformacoes especificas
        match line.instruction {
            Instruction::CHPR(label) => {
                // remove uso da funcao de funcoes
                if let Some(index) = self
                    .funcoes
                    .iter()
                    .position(|f| f.addr_inicio == label.unwrap())
                {
                    if let Some(f) = self.funcoes.get_mut(index) {
                        if let Some(i) = f.usos.iter().position(|n| *n == addr) {
                            f.usos.remove(&i);
                        }
                    }
                }
            }
            Instruction::DSVS(label) => {
                // remove pulo associado
                let target = self.locate_address(label.unwrap()).unwrap();
                if let Some(e) = self
                    .grafo
                    .edges_connecting(node, target)
                    .map(|e| e.id())
                    .next()
                {
                    self.grafo.remove_edge(e);
                }
                // adiciona pulo para proximo endereço que ainda existir
                for i in 1..self.len() {
                    if let Some(target) = self.locate_address(addr + i) {
                        self.grafo.add_edge(node, target, ());
                        break;
                    }
                }
            }
            Instruction::DSVF(label) => {
                // remove pulo associado
                let target = self.locate_address(label.unwrap()).unwrap();
                if let Some(e) = self
                    .grafo
                    .edges_connecting(node, target)
                    .map(|e| e.id())
                    .next()
                {
                    self.grafo.remove_edge(e);
                }
            }
            Instruction::ENPR(_) => {
                // remove funcao de funcoes
                if let Some(i) = self
                    .funcoes
                    .iter()
                    .position(|f| f.addr_inicio == line.address)
                {
                    self.funcoes.swap_remove(i);
                }
            }
            _ => (),
        }

        // Lida com nodes vazios
        let empty_nodes: Vec<NodeIndex> = self
            .grafo
            .node_indices()
            .filter(|&node_index| self.grafo.node_weight(node_index).unwrap().is_empty())
            .collect();

        for empty_node in empty_nodes {
            let neighbors: Vec<NodeIndex> = self.grafo.neighbors(empty_node).collect();
            if neighbors.len() > 1 {
                panic!("Como um node vazio pode apontar para mais de um vizinho?");
            } else {
                // Find the node that the empty node points to
                if let Some(target) = neighbors.into_iter().next() {
                    let edges_to_redirect: Vec<_> = self
                        .grafo
                        .edges_directed(empty_node, petgraph::Direction::Incoming)
                        .map(|edge| edge.source())
                        .collect();

                    for source in edges_to_redirect {
                        self.grafo.add_edge(source, target, ());
                    }
                }
            }

            // Remove the empty node from the graph
            self.grafo.remove_node(empty_node);
        }
    }

    pub fn len(&self) -> usize {
        self.grafo.node_weights().map(|lines| lines.len()).sum()
    }

    pub fn remove_node(&mut self, inicio: NodeIndex) {
        // verifica se a primeira instrucao é ENPR
        let removidos: Vec<usize> = self
            .grafo
            .node_weight(inicio)
            .unwrap()
            .iter()
            .map(|line| line.address)
            .collect();

        for r in removidos {
            self.remove_instruction_controlled(r, false);
        }
    }

    pub fn to_mepa_code(self) -> MepaCode {
        // Collect all instructions from all nodes into a single Vec
        let mut instructions: Vec<InstructionAndMetadata> = self
            .grafo
            .into_nodes_edges()
            .0
            .into_iter()
            .flat_map(|node| node.weight.into_iter())
            .collect();

        // Sort instructions by address
        instructions.sort_by_key(|instr| instr.address);

        // First, collect the positions of the labels in an immutable context
        let label_positions: Vec<(usize, usize)> = instructions
            .iter()
            .enumerate()
            .filter_map(|(i, instr)| match &instr.instruction {
                Instruction::DSVS(label) | Instruction::DSVF(label) | Instruction::CHPR(label) => {
                    Some((i, label.unwrap()))
                }
                _ => None,
            })
            .map(|(i, label)| {
                (
                    i,
                    instructions
                        .iter()
                        .position(|l| l.address == label)
                        .unwrap(),
                )
            })
            .map(|(i, pos)| (i, pos))
            .collect();

        // Then, update the labels in a mutable context
        for (i, pos) in label_positions {
            match &mut instructions[i].instruction {
                Instruction::DSVS(label) | Instruction::DSVF(label) | Instruction::CHPR(label) => {
                    *label = Label::Literal(pos);
                }
                _ => (),
            }
        }

        // Return the sorted Vec
        MepaCode::from(instructions.into_iter().map(|line| line.instruction))
    }

    pub fn allocations(&self) -> impl Iterator<Item = &Allocation> {
        self.instructions_unordered()
            .filter_map(|i| i.allocation.as_ref())
    }

    pub fn print_vars(&self) {
        if self.memoria_consistente {
            for aloc in self.allocations() {
                println!("───────────────┬───────────────────────────────");
                println!(
                    "Alocação: {:<4} │ Liberação: {:<4}",
                    aloc.addr,
                    aloc.liberation_address.unwrap()
                );
                println!("─────┬─────────┴──┬─────────────┬──────────────");
                println!("Item │ Usos       │ Atribuições │ Referências ");
                println!("─────┼────────────┼─────────────┼──────────────");

                for (i, var) in aloc.variaveis.iter().enumerate() {
                    let usos = var
                        .usos
                        .iter()
                        .map(|x| x.to_string())
                        .collect::<Vec<_>>()
                        .join(", ");
                    let atribuicoes = var
                        .atribuicoes
                        .iter()
                        .map(|x| x.to_string())
                        .collect::<Vec<_>>()
                        .join(", ");
                    let referencias = var
                        .referencias
                        .iter()
                        .map(|x| x.to_string())
                        .collect::<Vec<_>>()
                        .join(", ");

                    println!(
                        "{:<4} │ {:<10} │ {:<11} │ {:<12}",
                        i, usos, atribuicoes, referencias
                    );
                }

                println!("─────┴────────────┴─────────────┴──────────────");
            }
        } else {
            println!("Memoria inconsistente: não é possível imprimir uso de memória");
        }
    }

    pub fn export_to_file(&self, filename: &PathBuf) -> io::Result<()> {
        if let Some(parent) = filename.parent() {
            fs::create_dir_all(parent)?;
        }

        // Create or open the file
        let file = File::create(filename)?;

        let mut graph_with_code: Graph<String, ()> = Graph::new();
        for instructions in self.grafo.node_weights() {
            let linhas_de_mepa: Vec<String> = instructions
                .iter()
                .map(|linha| {
                    if self.memoria_consistente {
                        format!(
                            "[{}] {}: {}{}",
                            linha.initial_memory_usage.unwrap(),
                            linha.address,
                            linha.instruction,
                            if let Some(aloc) = &linha.allocation {
                                format!(
                                    " ({} | {} | {} | {})",
                                    aloc.variaveis
                                        .iter()
                                        .flat_map(|s| &s.atribuicoes)
                                        .map(|x| x.to_string())
                                        .collect::<Vec<_>>()
                                        .join(", "),
                                    aloc.variaveis
                                        .iter()
                                        .flat_map(|s| &s.usos)
                                        .map(|x| x.to_string())
                                        .collect::<Vec<_>>()
                                        .join(", "),
                                    aloc.variaveis
                                        .iter()
                                        .flat_map(|s| &s.referencias)
                                        .map(|x| x.to_string())
                                        .collect::<Vec<_>>()
                                        .join(", "),
                                    aloc.liberation_address.unwrap()
                                )
                            } else {
                                "".to_string()
                            }
                        )
                    } else {
                        format!("{}: {}", linha.address, linha.instruction)
                    }
                })
                .collect();
            let s = linhas_de_mepa.join("\n");
            graph_with_code.add_node(s);
        }

        let arestas: Vec<(NodeIndex, NodeIndex)> = self
            .grafo
            .raw_edges()
            .iter()
            .map(|edge| (edge.source(), edge.target()))
            .collect();

        graph_with_code.extend_with_edges(&arestas);

        let raw_dot = format!(
            "{:?}",
            Dot::with_config(&graph_with_code, &[Config::EdgeNoLabel])
        );

        let mut processed_dot = raw_dot.replace("\\\"", "").replace("\\\\", "\\");

        // adiciona legenda
        if let Some(pos) = processed_dot.rfind('}') {
            processed_dot.insert_str(pos,
                if self.memoria_consistente{"\tgraph [labelloc=\"b\", label=\"REFERÊNCIA\\naddr: instrução (atribuições | usos | referências | addr de dealoc)\"]\n"}
                else{"\tgraph [labelloc=\"b\", label=\"REFERÊNCIA (caso: memória inconsistente)\\naddr: instrução\"]\n"});
        }

        // Write the processed string to the file
        write!(&file, "{}", processed_dot)
    }

    fn debug_print(&self){
        println!("-------CODE-----------------");
        for node in self.grafo.node_indices(){
            println!("Node {}",node.index());
            for line in self.grafo.node_weight(node).unwrap(){
                println!("    {}: {:?}",line.address, line.instruction);
            }
        }
        println!("edges:");
        for edge in self.grafo.edge_indices(){
            let edge = self.grafo.edge_endpoints(edge).unwrap();
            println!("    {} -> {}",edge.0.index(), edge.1.index());
        }
        println!("fn:");
        for f in &self.funcoes{
            println!("    {}: usos: {:?}",f.addr_inicio, f.usos);
        }
        println!("----------------------------");
    }

}

pub fn remover_rotulos_simbolicos(mc: MepaCode) -> MepaCode {
    let mut labels = HashMap::new();

    // Localiza todas labels
    mc.0.iter()
        .enumerate()
        .for_each(|(line, (label_current_line, _))| {
            if let Some(Label::Simbolic(s)) = label_current_line {
                labels.insert(s.clone(), line);
            }
        });

    // Transforma instruções usando mapa de labels
    let mut mc = MepaCode(
        mc.0.into_iter()
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
                        }
                        Instruction::DSVF(label) => {
                            if let Label::Simbolic(s) = label {
                                Instruction::DSVF(Label::Literal(*labels.get(&s).unwrap()))
                            } else {
                                Instruction::DSVF(label)
                            }
                        }
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
            })
            .collect(),
    );

    let mut i = 0;
    while i < mc.0.len() {
        if matches!(mc.0[i].1, Instruction::NADA) {
            mc.remove_instruction(i);
        } else {
            i += 1;
        }
    }

    mc
}
