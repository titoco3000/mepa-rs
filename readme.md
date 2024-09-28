# Mepa - rs

Interpretador de código MEPA feito em Rust. 


## MEPA

MEPA (Máquina de Execução de PAscal) é uma linguagem intermediária criada pelo prof. Tomasz Kowatoski.

## Instruções
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

## Formatação

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
    P: ENPR k # isso é um procedimento 

## Como executar

- Instale Rust e Cargo no seu computador: https://rustup.rs/

- Baixe este repositório

- No terminal:

```
$ cargo run
```
