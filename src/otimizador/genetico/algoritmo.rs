use std::{fmt::Debug, mem::swap};

use rand::{random_range, rng, rngs::ThreadRng, Rng};

// implementacao algoritmo genetico

pub struct Genetico<T> {
    func_gene_aleatorio: fn(&mut ThreadRng) -> T,
    func_avaliadora: fn(&[T]) -> f32,
    tamanho_inicial_genoma: usize,
    pop: usize,
    mut_rate: f32,
    geracoes: usize,
}

impl<T> Genetico<T>
where
    T: Clone + Debug,
{
    pub fn new(
        func_gene_aleatorio: fn(&mut ThreadRng) -> T,
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
    fn escolher_progenitores(rng: &mut ThreadRng, pontuacoes: &[f32]) -> (usize, usize) {
        let mut pai = None;
        let mut mae = None;
        let total: f32 = pontuacoes.iter().sum();
        while pai.is_none() || mae.is_none() || pai == mae {
            let escolha: f32 = rng.random_range(0.0..total);
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
    pub fn reproduzir(&self, rng: &mut ThreadRng, pais: &[&[T]; 2]) -> [Vec<T>; 2] {
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
                    filhos[lado][gene] = (self.func_gene_aleatorio)(rng);
                }
            }
            //diminuição de genoma
            while filhos[lado].len() > 1 && random_range(0.0..1.0) < self.mut_rate {
                filhos[lado].pop();
            }
            // aumento de genoma do filho
            while random_range(0.0..1.0) < self.mut_rate {
                filhos[lado].push((self.func_gene_aleatorio)(rng));
            }
        }

        filhos
    }

    pub fn otimizar(&mut self) -> (Vec<T>, Vec<f32>) {
        let mut rng = rng();

        let mut populacao: Vec<Vec<T>> = (0..self.pop)
            .map(|_| {
                (0..self.tamanho_inicial_genoma)
                    .map(|_| (self.func_gene_aleatorio)(&mut rng))
                    .collect()
            })
            .collect();

        let mut alt_pop = Vec::with_capacity(self.pop);

        let mut melhor_individuo = populacao[0].clone();
        let mut melhor_pontuacao = (self.func_avaliadora)(&populacao[0]);

        let melhor_pontuacao_por_geracao: Vec<f32> = (0..self.geracoes)
            .map(|geracao| {
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

                let melhor_da_geracao = pontuacoes[melhor_index];
                println!(
                    "{}: {:?} -> {}",
                    geracao, populacao[melhor_index], melhor_da_geracao
                );
                if melhor_pontuacao > melhor_da_geracao {
                    melhor_pontuacao = melhor_da_geracao;
                    melhor_individuo = populacao[melhor_index].clone();
                }

                for score in &mut pontuacoes {
                    *score = 1.0 / (*score + 0.01);
                }

                alt_pop.clear();
                for _relacao in 0..self.pop / 2 {
                    let (pai, mae) = Self::escolher_progenitores(&mut rng, &pontuacoes);
                    for filho in self.reproduzir(&mut rng, &[&populacao[pai], &populacao[mae]]) {
                        alt_pop.push(filho);
                    }
                }

                swap(&mut alt_pop, &mut populacao);

                melhor_da_geracao
            })
            .collect();
        (melhor_individuo, melhor_pontuacao_por_geracao)
    }
}
