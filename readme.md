# Mepa - rs

Este projeto vai conter um otimizador para código intermediário MEPA. Mas como suporte, ele possui também um compilador para uma linguagem simples, *ipt* (**I**nteiro e **P**on**T**eiro) e uma máquina virtual de MEPA.

Este projeto é parte do meu TCC, com o título "Implementação de otimizacões de código intermediário no código MEPA".


## MEPA

MEPA (Máquina de Execução de PAscal) é uma linguagem intermediária criada pelo prof. Tomasz Kowatoski.

### Instruções
 No livro *Implementação de Linguagens de Programação*, a liguagem é descrita incrementalmente; as instruções aceitas pelo programa estão descritas abaixo.
 
 Para ter compatibilidade com a versão mais simples descrita no livro, algumas instruções possuem um "argumento-padrão".

| Instrução | Argumento 1 | Argumento 2 | Ações                             |
| --------- | ----------- | ----------- | --------------------------------- |
| CRCT      | k           |             | s+=1; M[s]=k                      |
| CRVL      | m [0]       | n           | s+=1; M[s]=M[D[m]+n]              |
| CREN      | m [0]       | n           | s+=1; M[s]=D[m]+n                 |
| ARMZ      | m [0]       | n           | M[D[m]+n]=M[s], s-=1              |
| CRVI      | m [0]       | n           | s+=1; M[s]=M[M[D[m]+n]]           |
| ARMI      | m [0]       | n           | M[M[D[m]+n]]=M[s], s-=1           |
| SOMA      |             |             | M[s-1] = M[s-1] + M[s]; s-=1      |
| SUBT      |             |             | M[s-1] = M[s-1] - M[s]; s-=1      |
| MULT      |             |             | M[s-1] = M[s-1] \* M[s]; s-=1     |
| DIVI      |             |             | M[s-1] = M[s-1] / M[s]; s-=1      |
| INVR      |             |             | M[s] = -M[s]                      |
| CONJ      |             |             | M[s-1] = M[s-1] && M[s]; s-=1     |
| DISJ      |             |             | M[s-1] = M[s-1] \|\| M[s]; s-=1   |
| NEGA      |             |             | M[s] = 1 - M[s]                   |
| CMME      |             |             | M[s-1] = M[s-1] < M[s]; s-=1      |
| CMMA      |             |             | M[s-1] = M[s-1] > M[s]; s-=1      |
| CMIG      |             |             | M[s-1] = M[s-1] == M[s]; s-=1     |
| CMDG      |             |             | M[s-1] = M[s-1] != M[s]; s-=1     |
| CMEG      |             |             | M[s-1] = M[s-1] <= M[s]; s-=1     |
| CMAG      |             |             | M[s-1] = M[s-1] >= M[s]; s-=1     |
| DSVS      | p           |             | i = p                             |
| DSVF      | p           |             | Se M[s]==0: i=p; s-=1 senão: s-=1 |
| NADA      |             |             |                                   |
| PARA      |             |             | Encerra o programa                |
| LEIT      |             |             | s+=1; M[s] = “próxima entrada”    |
| IMPR      |             |             | Imprime M[s]; s-=1                |
| AMEM      | n           |             | s+=n                              |
| DMEM      | n           |             | s-=n                              |
| INPP      |             |             | s=-1; D[0] = 0                    |
| CHPR      | p           |             | s+=1; M[s] = i+1; i=p             |
| ENPR      | k           |             | s+=1; M[s] = D[k]; D[k] = s+1     |
| RTPR      | k           | n           | D[k]=M[s]; i=M[s-1]; s-=n+2       |

### Formatação arquivos .mepa

Os arquivos MEPA devem ser formatados de maneira que cada instrução esteja em uma linha junto com seus argumentos:

    CRVL 1 2

Cada instrução pode possuir um rótulo:

    A: DSVS B

Comandos de desvio (DSVS, DSVF, CHPR) podem receber como argumento rótulos simbólicos (string) ou literais (numero da instrução)

    DSVF 12

O rótulo, instrução e argumentos podem ser separados por qualquer um dos seguintes simbolos: ```[',', ' ', '\t', ';', ':']```

    L1: CRVI ; 1,, 2 

Em cada linha, qualquer texto depois de ```#``` ou ```//``` é considerado como um comentário.
    
    # a linha abaixo é a entrada de um procedimento
    P: ENPR k  //isso é um procedimento 

## Compilador

O funcionamento interno do compilador que transforma ipt em MEPA está descrito em `src/compiler/readme.md`, bem como a gramática da linguagem.

## Máquina

A máquina virtual roda código MEPA, com o conjunto de instruções descritas acima.

## Instalação

- Instale Rust e Cargo no seu computador: https://rustup.rs/

- Baixe este repositório

## Para rodar

Como este programa pode fazer três coisas diferentes (compilar, executar e, futuramente, otimizar) é preciso passar como argumentos a ação que se quer. Abaixo está demonstrado como rodar dentro do ambiente cargo, se fosse o programa compilado, trocaria a parte `cargo run --` por `./nome-do-executavel`.

#### Compilação
```
$ cargo run -- compile samples/ipt/sort.ipt [-o output.o]
```

Se não for especificado -o, o objeto produzido para a linha acima ficará em `output/sort.mepa`.

#### Execução interativa
```
$ cargo run -- test samples/mepa/recursao.mepa
```

#### Execução imediata
```
$ cargo run -- run samples/mepa/recursao.mepa
```

#### Encadeamento

Além disso, é possível encadear execução com a compilação: 

```
$ cargo run -- compile samples/ipt/sort.ipt [--run | --test]
```

#### Entrada
No teste e execução, é possível especificar as entradas a serem feitas no programa:

```
$ cargo run -- run samples/mepa/recursao.mepa --input 1,2,3
```
Assim, quando houver a instrução `LEIT`, será lido o próximo valor da lista (ou o ultimo vai ser repetido, caso já tenham sido todos lidos).

Quando input não for especificado, `LEIT` vai pedir entrada pelo stdin.

#### Compilação de lote

É possível indicar uma pasta, e todos os arquivos dentro serão compilados:

```
$ cargo run -- compile samples/ipt 
```
Nesse modo, se for especificado -o, ele será sequencialmente sobreescrito.