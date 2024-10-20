# Documentação do Compilador IPT

## Introdução

Este documento descreve a linguagem de programação *ipt* (**I**nteiro e **P**on**T**eiro), uma linguagem de tipagem simples inspirada em C, e o funcionamento de seu compilador. A linguagem foi projetada para ser usada em um otimizador de linguagem intermediária MEPA, e pode produzir todos os cenários que ele aborda.

A linguagem *ipt* oferece suporte a variáveis inteiras (`int`), ponteiros (`ptr`), arrays estáticos e permite o uso de estruturas de controle como `if` e `while`, além de funções que retornam valores obrigatoriamente consumidos.

Para mais detalhes sobre a gramática da linguagem, consulte o arquivo [gramatica.md](./gramatica.md).

---

## Índice

1. [Estrutura da Linguagem](#estrutura-da-linguagem)
2. [Tipos de Dados](#tipos-de-dados)
3. [Sintaxe de Funções](#sintaxe-de-funções)
4. [Estruturas de Controle](#estruturas-de-controle)
5. [Operações de Entrada e Saída](#operações-de-entrada-e-saída)
6. [Arrays e Ponteiros](#arrays-e-ponteiros)
7. [Geração de Código MEPA](#geração-de-código-mepa)

---

## Estrutura da Linguagem

A linguagem *ipt* segue uma estrutura simples, inspirada em C. O código é composto por funções, que devem sempre retornar um valor. Todas as variáveis, tanto no escopo global quanto no local, devem ser declaradas no início de um bloco de código.

Exemplo de uma função `bubble_sort` na linguagem *ipt*:

```c
fn bubble_sort(ptr arr, int n){
    int i, j, temp;
    i = 0;
    while(i<n){
        j = i + 1;
        while(j<n){
            if(arr[i] > arr[j]){
                temp = arr[i];
                arr[i] = arr[j];
                arr[j] = temp;                
            }
            j = j + 1;
        }
        i = i+1;
    }
    return 0;
}
```
Para ser válido, um programa em *ipt* deve possuir uma função `main`. Assim, o menor programa válido possível é:
```c
fn main(){return 0;}
```

---

## Tipos de Dados

A linguagem *ipt* suporta os seguintes tipos de dados:

1. **`int`**: Inteiro de 32 bits.
2. **`ptr`**: Ponteiro, que pode referenciar arrays de inteiros.
3. **Arrays**: Arrays estáticos de inteiros, que podem ser passados como ponteiros.

### Declaração de Variáveis

As variáveis devem ser declaradas no início de cada função ou no início do programa. É possível declarar múltiplas variáveis do mesmo tipo separadas por vírgula.

```c
int a, b, c;
int arr[10];  // Array estático de 10 inteiros
```

---

## Sintaxe de Funções

Funções na linguagem *ipt* são declaradas com `fn`. Toda função deve retornar um valor, e o retorno precisa ser consumido (armazenado ou utilizado em uma expressão).

### Exemplo de Função

```c
fn nome_da_funcao(tipo1 param1, tipo2 param2){
    // Declaração de variáveis
    int var1;
    
    // Corpo da função
    // ...

    return valor;
}
```

### Chamadas de Função

As funções são chamadas passando argumentos por valor. Arrays devem ser passados com o uso de ponteiros.

```c
x = bubble_sort(&arr, tamanho);
```

---

## Estruturas de Controle

A linguagem *ipt* possui duas estruturas de controle principais:

### Condicionais `if`

O `if` pode ser usado para testar condições:

```c
if(condição){
    // Bloco de código
}
```

### Laços `while`

O `while` repete um bloco de código enquanto a condição for verdadeira:

```c
while(condição){
    // Bloco de código
}
```

---

## Operações de Entrada e Saída

A linguagem *ipt* possui dois comandos de entrada e saída:

### Comando `print`

Imprime uma ou mais expressões separadas por vírgula. O compilador irá gerar uma sequência de operações de saída no código intermediário.

```c
print(a, b, arr[i]);
```

### Comando `read`

Lê um inteiro para dentro de uma variável ou da posição de um array.

```c
read(a);
read(arr[i]);
```

---

## Arrays e Ponteiros

A linguagem *ipt* suporta arrays estáticos de inteiros. Os arrays são indexados de forma similar a C, e a indexação de ponteiros faz automaticamente a desreferenciação.

### Declaração de Arrays

Arrays são declarados especificando o tamanho fixo no momento da declaração:

```c
int arr[5];
```

### Uso de Ponteiros

Ponteiros podem ser passados como argumentos de função ou usados para acessar arrays.

```c
int x;
ptr p;
p = &arr;
p[0] = 1;
x = *p //x = 1
```

---

## Geração de Código MEPA

O compilador gera código intermediário MEPA. MEPA é uma linguagem de pilha, e para mais informações sobre ela, consulte o [readme principal](../../readme.md). A maioria das traduções é bastante simples, como o exemplo a seguir de uma condicional:
```
if(2<3){            CRCT 2
    print(2);       CRCT 3
}                   CMME
                    DSVF L1
                    CRCT 2
                    IMPR
                L1: NADA
```
Mas as chamadas de funções e a indexação são mais complexas. 

### Funções
Ao chamar uma função, primeiro precisamos reservar uma posição na pilha para o retorno. Se a função possui argumentos, eles são empilhados e serão acessados dentro do corpo da função usando endereços negativos (usando o nível léxico, eles vão se tornar positivos. Consulte a [tabela](../../readme.md) que especifica as instruções). Dentro do corpo da função (iniciado por `ENPR`), são reservadas duas variáveis, que são usadas na indexação.

### Indexação
Como essa versão da MEPA não tem instruções específicas para acesso relativo (a menos não diretamente), é preciso fazer um caminho alternativo. Primeiro calculamos o endereço do array; se estamos indexando um `int` usamos `CREN`, se estamos indexando um `ptr` usamos `CRVL`, para fazer a dereferencia. Tendo esse endereço, somamos ao índice e salvamos numa das variáveis reservadas para indexação. São reservadas duas: uma para conter o endereço de rvalue, outra para o endereço de lvalue. 

Isso porque, no cenário:
```
arr[i] = arr[j];
``` 
se tivessemos uma variável só iria acontecer a seguinte situação:
1. Primeiro calcula o endereço do lvalue, o destino da atribuição, salvando no lugar reservado
2. Calcular o endereço do rvalue, o valor da atribuição, salvando no lugar reservado (sobreescrevendo o anterior)
3. O valor é armazenado no endereço errado

Poderíamos resolver isso calculando primeiro o rvalue e depois o lvalue, mas isso iria causar uma ordem de execução menos clara (o que poderia causar efeitos inesperados ao usar resultado de funções como índices, por exemplo).

Tendo o endereço com o índice já somado, usamos as instruções de acesso indireto (`CRVI` ou `ARMI`) para salvar ou obter o valor.
