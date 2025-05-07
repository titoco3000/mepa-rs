#[allow(dead_code)]
mod algoritmo;
use std::fmt::format;

use crate::evaluator::evaluate_samples_with_specified_order;
use algoritmo::Genetico;
use plotters::prelude::*;
use rand::{random_range, rngs::ThreadRng, Rng};

const PESO_RESULTADOS: f32 = 10.0;
const PESO_PASSOS: f32 = 1.0;
const PESO_TAMANHO: f32 = 0.0;

fn obter_pontuacao(individuo: &[usize]) -> f32 {
    let eval = evaluate_samples_with_specified_order(&individuo);
    let len = eval.len();
    eval.into_iter()
        .map(|item| {
            let (filename, original, otimizado, etapas_otimizacao) = item;
            let exec_original = original.unwrap();
            let exec_otimizado = otimizado.unwrap();
            let r = (exec_otimizado.max_memory as f32 / exec_original.max_memory as f32)
                + (exec_otimizado.steps as f32 / exec_original.steps as f32)
                + (exec_otimizado.instructions as f32 / exec_original.instructions as f32)
                    * PESO_RESULTADOS
                + etapas_otimizacao as f32 * PESO_PASSOS;
            // println!(
            //     "{} otimizado: {} → {} passos, {} → {} max de memoria, {} → {} instruções. Pontuacao: {}",
            //     filename,
            //     exec_original.steps,
            //     exec_otimizado.steps,
            //     exec_original.max_memory,
            //     exec_otimizado.max_memory,
            //     exec_original.instructions,
            //     exec_otimizado.instructions, r
            // );
            r
        })
        .sum::<f32>()
        / len as f32
        + (individuo.len() as f32) * PESO_TAMANHO
}

fn func_gene_aleatorio(rng: &mut ThreadRng) -> usize {
    rng.random_range(0..4)
}

fn plot(resultado: Vec<usize>, progresso: &[f32]) {
    let root = BitMapBackend::new("plot.png", (640, 480)).into_drawing_area();
    root.fill(&WHITE).unwrap();

    let mut chart = ChartBuilder::on(&root)
        .caption(
            &format!("Progresso para atingir {:?}", resultado),
            ("sans-serif", 30),
        )
        .margin(10)
        .x_label_area_size(30)
        .y_label_area_size(30)
        .build_cartesian_2d(
            0f32..progresso.len() as f32,
            (progresso
                .iter()
                .min_by(|a, b| a.partial_cmp(b).unwrap())
                .unwrap()
                - 1.0)
                ..(progresso
                    .iter()
                    .max_by(|a, b| a.partial_cmp(b).unwrap())
                    .unwrap()
                    + 1.0),
        )
        .unwrap();

    chart.configure_mesh().draw().unwrap();

    chart
        .draw_series(LineSeries::new(
            progresso.iter().enumerate().map(|(i, &y)| (i as f32, y)),
            &RED,
        ))
        .unwrap();
}

pub fn encontrar_ordem_otimizacao() {
    // let individuo = [0, 1, 2, 3];
    // let p = obter_pontuacao(&individuo);
    // println!("{}", p);

    let mut g = Genetico::new(func_gene_aleatorio, obter_pontuacao, 3)
        .set_mut_rate(0.1)
        .set_pop(100)
        .set_geracoes(30);
    let (resultado, progresso) = g.otimizar();
    println!("{:?}, {:?}", resultado, progresso);

    plot(resultado, &progresso);
}
