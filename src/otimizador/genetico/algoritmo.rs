use std::{fmt::Debug, mem::swap};

// import random
use rand::random_range;

// # implementacao algoritmo genetico

pub struct Genetico<T> {
    func_gene_aleatorio: fn() -> T,
    func_avaliadora: fn(&[T]) -> f32,
    tamanho_inicial_genoma: usize,
    pop: usize,
    mut_rate: f32,
    geracoes: usize,
    minimizar: bool,
}

impl<T> Genetico<T>
where
    T: Clone + Debug,
{
    // def algoritmo_genetico(func_gene_aleatorio, func_avaliadora, tamanho_genoma, func_correcao=None, pop=100, mut_rate=0.01, geracoes=100, minimizar=False):
    pub fn new(
        func_gene_aleatorio: fn() -> T,
        func_avaliadora: fn(&[T]) -> f32,
        tamanho_inicial_genoma: usize,
    ) -> Genetico<T> {
        Genetico {
            func_gene_aleatorio,
            func_avaliadora,
            tamanho_inicial_genoma,
            pop: 100,
            mut_rate: 0.01,
            geracoes: 100,
            minimizar: false,
        }
    }
    pub fn set_pop(mut self, pop: usize) -> Self {
        self.pop = pop;
        self
    }
    pub fn set_mut_rate(mut self, mut_rate: f32) -> Self {
        self.mut_rate = mut_rate;
        self
    }
    pub fn set_geracoes(mut self, geracoes: usize) -> Self {
        self.geracoes = geracoes;
        self
    }
    pub fn set_minimizar(mut self, minimizar: bool) -> Self {
        self.minimizar = minimizar;
        self
    }
    fn escolher_progenitores(pontuacoes: &[f32]) -> (usize, usize) {
        let mut pai = None;
        let mut mae = None;
        let total: f32 = pontuacoes.iter().sum();
        while pai.is_none() || mae.is_none() || pai == mae {
            let escolha: f32 = random_range(0.0..total);
            let mut soma = 0.0;
            for i in 0..pontuacoes.len() {
                soma += pontuacoes[i];
                if escolha < soma {
                    if pai.is_none() {
                        pai = Some(i);
                    } else {
                        mae = Some(i);
                    }
                    break;
                }
            }
        }
        return (pai.unwrap(), mae.unwrap());
    }
    // temp pub
    pub fn reproduzir(&self, pais: &[&[T]; 2]) -> [Vec<T>; 2] {
        let particao: usize = random_range(0..pais[0].len());

        let mut filhos = [
            Vec::with_capacity(particao + pais[1].len().saturating_sub(particao)),
            Vec::with_capacity(
                pais[0].len().saturating_sub(particao) + pais[1].len().min(particao),
            ),
        ];

        for lado in 0..2 {
            // adiciona genes do pai
            for i in 0..particao.min(pais[lado].len()) {
                filhos[lado].push(pais[lado][i].clone());
            }
            // adiciona genes da mae
            for i in particao.min(pais[(lado + 1) % 2].len())..pais[(lado + 1) % 2].len() {
                filhos[(lado + 1) % 2].push(pais[(lado + 1) % 2][i].clone());
            }

            // mutacoes geneticas
            for gene in 0..filhos[lado].len() {
                if random_range(0.0..1.0) < self.mut_rate {
                    filhos[lado][gene] = (self.func_gene_aleatorio)();
                }
            }
            //diminuição de genoma
            while filhos[lado].len() > 1 && random_range(0.0..1.0) < self.mut_rate {
                filhos[lado].pop();
            }
            // aumento de genoma do filho
            while random_range(0.0..1.0) < self.mut_rate {
                filhos[lado].push((self.func_gene_aleatorio)());
            }
        }

        filhos
    }

    pub fn otimizar(&mut self) -> (Vec<T>, Vec<f32>) {
        let mut populacao: Vec<Vec<T>> = (0..self.pop)
            .map(|_| {
                (0..self.tamanho_inicial_genoma)
                    .map(|_| (self.func_gene_aleatorio)())
                    .collect()
            })
            .collect();

        let mut alt_pop = Vec::with_capacity(self.pop);

        let mut melhor_individuo = populacao[0].clone();
        let mut melhor_pontuacao = (self.func_avaliadora)(&populacao[0]);

        let melhor_pontuacao_por_geracao: Vec<f32> = (0..self.geracoes)
            .map(|_| {
                let mut pontuacoes: Vec<f32> = populacao
                    .iter()
                    .map(|individuo| (self.func_avaliadora)(individuo))
                    .collect();

                // for i in 0..pontuacoes.len() {
                //     println!("    {:?}: {}", populacao[i], pontuacoes[i]);
                // }
                let melhor_index = pontuacoes
                    .iter()
                    .enumerate()
                    .min_by(|a, b| a.1.partial_cmp(b.1).unwrap())
                    .unwrap()
                    .0;

                let maior = *(pontuacoes
                    .iter()
                    .max_by(|a, b| a.partial_cmp(b).unwrap())
                    .unwrap());

                if melhor_pontuacao > pontuacoes[melhor_index] {
                    melhor_pontuacao = pontuacoes[melhor_index];
                    melhor_individuo = populacao[melhor_index].clone();
                }

                // inverte pontuacao
                for item in &mut pontuacoes {
                    *item = maior - *item;
                }

                alt_pop.clear();
                for _relacao in 0..self.pop / 2 {
                    let (pai, mae) = Self::escolher_progenitores(&pontuacoes);
                    for filho in self.reproduzir(&[&populacao[pai], &populacao[mae]]) {
                        alt_pop.push(filho);
                    }
                }

                swap(&mut alt_pop, &mut populacao);

                println!("{}", pontuacoes[melhor_index]);
                pontuacoes[melhor_index]
            })
            .collect();
        (melhor_individuo, melhor_pontuacao_por_geracao)
    }
}

//     def reproduzir(pai, mae):
//         particao = random.randint(0, len(pai)-1)
//         filhos = [[],[]]
//         # Produz os filhos
//         for i in range(len(pai)):
//             if i <particao:
//                 filhos[0].append(pai[i])
//                 filhos[1].append(mae[i])
//             else:
//                 filhos[1].append(pai[i])
//                 filhos[0].append(mae[i])

//         # Causa mutacoes
//         for filho in filhos:
//             for i in range(len(filho)):
//                 if random.random()<mut_rate:
//                     filho[i] = func_gene_aleatorio()
//             if func_correcao:
//                 func_correcao(filho)

//         return filhos

//     # gera populacao inicial aleatoria
//     populacao = [[func_gene_aleatorio() for _ in range(tamanho_genoma)] for _ in range(pop)]

//     if func_correcao:
//         for p in populacao:
//             func_correcao(p)

//     melhor_pontuacao = -1
//     melhor = []

//     melhor_por_geracao = []

//     for geracao in range(geracoes):
//         pontuacoes = [func_avaliadora(genoma) for genoma in populacao]
//         index_melhor = (min if minimizar else max)(range(len(pontuacoes)), key=pontuacoes.__getitem__)
//         melhor_por_geracao.append(pontuacoes[index_melhor])

//         if pontuacoes[index_melhor] > melhor_pontuacao:
//             melhor_pontuacao = pontuacoes[index_melhor]
//             melhor = populacao[index_melhor]

//         populacao = [
//             ind for p in [
//                 escolher_progenitores(pontuacoes) for _ in range(pop)
//             ] for ind in reproduzir(populacao[p[0]], populacao[p[1]])
//         ]

//     pontuacoes = [func_avaliadora(genoma) for genoma in populacao]
//     index_melhor = (min if minimizar else max)(range(len(pontuacoes)), key=pontuacoes.__getitem__)
//     if pontuacoes[index_melhor] > melhor_pontuacao:
//         melhor_pontuacao = pontuacoes[index_melhor]
//         melhor = populacao[index_melhor]

//     return {
//         "melhor":melhor,
//         "pontuacao_por_geracao":melhor_por_geracao,
//         "pontuacao":melhor_pontuacao,
//         "pop":pop,
//         "mut_rate":mut_rate,
//         "geracoes":geracoes,
//         "target": "min" if minimizar else "max"
//     }

// current_index = 0

// def plot(resultados):
//     global current_index
//     import matplotlib.pyplot as plt
//     import matplotlib.widgets as widgets

//     if type(resultados) != list:
//         resultados = [resultados]

//     fig, ax = plt.subplots(figsize=(8, 5))
//     plt.subplots_adjust(bottom=0.2)

//     current_index = 0

//     # Initial plot
//     def plot_data(index):
//         ax.clear()
//         ax.plot(resultados[index]["pontuacao_por_geracao"], label=f'pontuação ({resultados[index]["target"]}: {(max if resultados[index]["target"]=="max" else min)(resultados[index]["pontuacao_por_geracao"])})')
//         ax.set_xlabel('Gerações')
//         ax.set_ylabel('Pontuação')
//         ax.set_title(f'Evolução da Pontuação - População {resultados[index]["pop"]}, Mutação {resultados[index]["mut_rate"]}, Gerações {resultados[index]["geracoes"]}')
//         ax.legend()
//         ax.grid()
//         fig.canvas.draw()

//     plot_data(current_index)

//     # Button callback functions
//     def next_plot(event):
//         global current_index
//         current_index = min(current_index + 1,len(resultados)-1)
//         plot_data(current_index)

//     def prev_plot(event):
//         global current_index
//         current_index = max(current_index - 1, 0)
//         plot_data(current_index)

//     if len(resultados) > 1:

//         # Add navigation buttons
//         axprev = plt.axes([0.7, 0.05, 0.1, 0.075])
//         axnext = plt.axes([0.81, 0.05, 0.1, 0.075])
//         btn_next = widgets.Button(axnext, 'Next')
//         btn_prev = widgets.Button(axprev, 'Previous')
//         btn_next.on_clicked(next_plot)
//         btn_prev.on_clicked(prev_plot)

//     plt.show()
