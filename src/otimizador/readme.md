Para o mapeamento de variaveis, tive que impor algumas regras que não eram contempladas em MEPA:
1. Cada alocação com AMEM deve possuir uma unica liberação DMEM correspondente, de mesmo tamanho
2. Essa liberação deve ser atingida determinísticamente (em todos os casos possíveis)