# Lógica nebulosa aplicada ao automobilismo virtual

Protótipo da disciplina de Lógica Nebulosa. Um sistema de inferência de Mamdani
decide, a cada amostra de um simulador de corrida, se o carro está **saindo de
frente** (subesterço), **saindo de traseira** (sobresterço) ou **equilibrado**.

O módulo nebuloso faz parte do aplicativo LMU Practice, que lê os dados do jogo
Le Mans Ultimate. Este repositório contém só a parte de lógica nebulosa,
reorganizada para ser lida e executada sozinha. O cálculo é o mesmo do
aplicativo.

## Como executar

É preciso ter o [Rust](https://www.rust-lang.org/tools/install) instalado. Não
há outras dependências, e nenhuma biblioteca de lógica nebulosa é usada.

```sh
cargo run                                    # exemplo numérico, etapa por etapa
cargo run -- valores 2 6                     # outro par: frente 2°, traseira 6°
cargo run -- trecho dados/passagem-a.csv     # decisões sucessivas sobre uma gravação
cargo test
```

## Organização

| Arquivo | Conteúdo |
|---|---|
| [src/lib.rs](src/lib.rs) | Módulo nebuloso: funções de pertinência, regras, implicação, agregação e centroide |
| [src/main.rs](src/main.rs) | Sistema externo: entrega uma amostra por vez ao módulo e registra as decisões |
| [dados/](dados) | Duas passagens gravadas no mesmo trecho de Imola |
| [tests/inferencia.rs](tests/inferencia.rs) | Exemplo numérico e propriedades do sistema |

## Entradas

A medida principal é o **ângulo de escorregamento** de cada eixo: o ângulo entre
a direção para onde o pneu aponta e a direção para onde ele anda, na média das
duas rodas.

Uma rampa "de a até b" vale 0 até a, 1 a partir de b e cresce em linha reta
entre os dois.

| Variável | Termo | Função de pertinência |
|---|---|---|
| Diferença frente − traseira | frente escorrega mais | rampa de 0,8° a 3° |
| | traseira escorrega mais | a mesma rampa, espelhada |
| | traseira escorrega um pouco mais | rampa de 0,4° a 2°, espelhada |
| | escorregamentos parecidos | 1 até 0,3° de diferença, 0 a partir de 1,2° |
| Escorregamento de um eixo | alto | rampa de 2° a 5° |
| Maior escorregamento | moderado | 1 até 5°, 0 a partir de 10° |
| Carro | em curva | mínimo entre a rampa de 2 a 5 m/s² (aceleração lateral) e a de 0,04 a 0,12 rad/s (rotação) |
| Volante | contra a curva | rampa de 0,015 a 0,12, quando volante e rotação têm sinais opostos |
| Acelerador | aliviado de repente | rampa de 0,3 a 1,5 do curso por segundo |

## Base de regras

O **E** é o mínimo e o **OU** é o máximo. Todas as regras exigem que o carro
esteja em curva.

| Regra | SE | ENTÃO |
|---|---|---|
| R1 | a frente escorrega mais **e** o escorregamento da frente é alto | sai de frente |
| R2 | a traseira escorrega mais **e** o de trás é alto | sai de traseira |
| R3 | a traseira escorrega um pouco mais **e** o de trás é alto **e** o volante está contra a curva | sai de traseira |
| R4 | a traseira escorrega um pouco mais **e** o de trás é alto **e** o acelerador foi aliviado de repente | sai de traseira |
| R5 | os escorregamentos são parecidos **e** o maior deles é moderado | equilibrado |

R2, R3 e R4 têm a mesma conclusão, então a ativação de "sai de traseira" é o
máximo das três.

## Saída

O comportamento vai de −1 (sai de traseira) a +1 (sai de frente), com cinco
conjuntos triangulares:

| Termo | Centro | Meia-largura |
|---|---|---|
| sai de traseira forte | −0,85 | 0,3 |
| sai de traseira leve | −0,40 | 0,4 |
| equilibrado | 0 | 0,25 |
| sai de frente leve | +0,40 | 0,4 |
| sai de frente forte | +0,85 | 0,3 |

- **Implicação:** o conjunto "leve" é recortado na altura `min(2 × ativação, 1)`
  e o "forte" na altura dada pela rampa de 0,4 a 1 sobre a ativação. O conjunto
  "equilibrado" é recortado na própria ativação da R5.
- **Agregação:** máximo dos conjuntos recortados.
- **Defuzzificação:** centroide, calculado numericamente em 201 pontos.

Com um recorte único no conjunto "leve", que é simétrico, o centroide seria
sempre ±0,40. O conjunto "forte" faz a resposta crescer com a ativação.

## Exemplo numérico

Frente escorregando 4° e traseira 2°, no meio de uma curva:

| Passo | Valor |
|---|---|
| frente escorrega mais: (2 − 0,8) / (3 − 0,8) | 0,55 |
| escorregamento da frente alto: (4 − 2) / (5 − 2) | 0,67 |
| carro em curva | 1,00 |
| ativação da R1 (mínimo) | 0,55 |
| recorte de "leve" | 1,00 |
| recorte de "forte": (0,55 − 0,4) / 0,6 | 0,24 |
| centroide | +0,46 |

## Dados

`passagem-a.csv` e `passagem-b.csv` são duas passagens por um mesmo trecho de
531 m em Imola, com um McLaren 720S LMGT3 Evo no Le Mans Ultimate, a 100
amostras por segundo.

Os arquivos contêm só as variáveis de entrada e só as amostras que o aplicativo
considerou utilizáveis. A leitura do jogo e o descarte de amostras (carro fora
da pista, roda sem contato, velocidade baixa, leituras extremas) são feitos pelo
aplicativo e não fazem parte deste repositório. Por isso há saltos no tempo.

Perto dos 247 m, a passagem A resulta em −0,56 (sai de traseira) e a B em
+0,56 (sai de frente).

O programa só dá nome ao comportamento quando a ativação chega a 0,35. Esse
limite serve para a exibição e não participa da inferência.

## Limitações

- As funções de pertinência e as regras foram definidas à mão, sem ajuste
  automático e sem medida de taxa de acerto.
- Um centroide zero é ambíguo: frente e traseira ativadas ao mesmo tempo dão o
  mesmo valor que "equilibrado". Por isso as duas ativações também são
  guardadas.
- O sistema decide a cada amostra, mas não atua no carro. Um próximo passo seria
  fechar a malha ajustando o acerto (setup) a partir das decisões, e ajustar os
  parâmetros com dados, por exemplo com ANFIS.

## Referências

- Zadeh, L. A. Fuzzy sets. *Information and Control*, 8(3), 338–353, 1965.
- Mamdani, E. H.; Assilian, S. An experiment in linguistic synthesis with a
  fuzzy logic controller. *International Journal of Man-Machine Studies*, 7(1),
  1–13, 1975.
- MathWorks. [Fuzzy Inference Process](https://www.mathworks.com/help/fuzzy/fuzzy-inference-process.html).
- Jang, J.-S. R. ANFIS: adaptive-network-based fuzzy inference system. *IEEE
  Transactions on Systems, Man, and Cybernetics*, 23(3), 665–685, 1993.
- Hirche, B.; Ayalew, B. A Fuzzy Inference System for Understeer/Oversteer
  Detection Towards Model-Free Stability Control. SAE Technical Paper
  2016-01-1630, 2016.
