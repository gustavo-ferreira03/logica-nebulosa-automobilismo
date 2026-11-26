//! Sistema de inferência nebuloso (Mamdani) para o comportamento de um carro
//! numa curva: saindo de frente (subesterço), saindo de traseira
//! (sobresterço) ou equilibrado.
//!
//! Etapas: fuzzificação, base de regras (E = mínimo, OU = máximo),
//! implicação (recorte do conjunto de saída), agregação (máximo) e
//! defuzzificação (centroide).
//!
//! Todas as funções de pertinência e regras foram definidas à mão.

/// Medidas que o módulo nebuloso recebe em cada amostra.
#[derive(Clone, Copy, Debug)]
pub struct Entradas {
    /// Ângulo de escorregamento médio do eixo dianteiro, em graus.
    pub frente: f64,
    /// Ângulo de escorregamento médio do eixo traseiro, em graus.
    pub traseira: f64,
    /// Aceleração lateral, em m/s².
    pub aceleracao_lateral: f64,
    /// Velocidade de rotação do carro (guinada), em rad/s. Positiva à direita.
    pub rotacao: f64,
    /// Posição do volante, de -1 a 1. Positiva à direita.
    pub volante: f64,
    /// Velocidade com que o acelerador está sendo solto, em curso por segundo.
    pub alivio_acelerador: f64,
}

/// Graus de pertinência das entradas em cada termo linguístico.
#[derive(Clone, Copy, Debug)]
pub struct Graus {
    pub em_curva: f64,
    pub frente_escorrega_mais: f64,
    pub traseira_escorrega_mais: f64,
    pub traseira_escorrega_um_pouco_mais: f64,
    pub escorregamentos_parecidos: f64,
    pub frente_alto: f64,
    pub traseira_alto: f64,
    pub maior_moderado: f64,
    pub volante_contra_a_curva: f64,
    pub acelerador_aliviado: f64,
}

/// Ativação de cada regra.
#[derive(Clone, Copy, Debug)]
pub struct Regras {
    pub r1: f64,
    pub r2: f64,
    pub r3: f64,
    pub r4: f64,
    pub r5: f64,
}

impl Regras {
    /// Conclusão da R1.
    pub fn sai_de_frente(&self) -> f64 {
        self.r1
    }

    /// R2, R3 e R4 têm a mesma conclusão: OU pelo máximo.
    pub fn sai_de_traseira(&self) -> f64 {
        self.r2.max(self.r3).max(self.r4)
    }

    /// Conclusão da R5.
    pub fn equilibrado(&self) -> f64 {
        self.r5
    }
}

/// Resultado de uma inferência.
#[derive(Clone, Copy, Debug)]
pub struct Decisao {
    pub graus: Graus,
    pub regras: Regras,
    /// Centroide da saída: -1 (sai de traseira) a +1 (sai de frente).
    /// `None` quando nenhuma regra dispara, como numa reta.
    pub comportamento: Option<f64>,
}

/// Rampa: 0 até `inicio`, 1 a partir de `fim`, linear entre os dois.
pub fn rampa(valor: f64, inicio: f64, fim: f64) -> f64 {
    ((valor - inicio) / (fim - inicio)).clamp(0.0, 1.0)
}

/// Triângulo: 1 no centro, 0 a partir de `meia_largura` de distância dele.
pub fn triangulo(valor: f64, centro: f64, meia_largura: f64) -> f64 {
    (1.0 - (valor - centro).abs() / meia_largura).max(0.0)
}

/// Etapa 1: cada medida vira graus de pertinência.
pub fn fuzzificar(e: &Entradas) -> Graus {
    let diferenca = e.frente - e.traseira;
    Graus {
        // Exige aceleração lateral E rotação.
        em_curva: rampa(e.aceleracao_lateral.abs(), 2.0, 5.0).min(rampa(
            e.rotacao.abs(),
            0.04,
            0.12,
        )),
        frente_escorrega_mais: rampa(diferenca, 0.8, 3.0),
        traseira_escorrega_mais: rampa(-diferenca, 0.8, 3.0),
        traseira_escorrega_um_pouco_mais: rampa(-diferenca, 0.4, 2.0),
        escorregamentos_parecidos: 1.0 - rampa(diferenca.abs(), 0.3, 1.2),
        frente_alto: rampa(e.frente, 2.0, 5.0),
        traseira_alto: rampa(e.traseira, 2.0, 5.0),
        maior_moderado: 1.0 - rampa(e.frente.max(e.traseira), 5.0, 10.0),
        // Volante e rotação com sinais opostos: o motorista corrige a traseira.
        volante_contra_a_curva: if e.volante * e.rotacao < 0.0 {
            rampa(e.volante.abs(), 0.015, 0.12)
        } else {
            0.0
        },
        acelerador_aliviado: rampa(e.alivio_acelerador, 0.3, 1.5),
    }
}

/// Etapa 2: a base de regras. O E é o mínimo.
pub fn avaliar_regras(g: &Graus) -> Regras {
    Regras {
        // SE a frente escorrega mais E o escorregamento da frente é alto
        // ENTÃO sai de frente.
        r1: g.frente_escorrega_mais.min(g.frente_alto).min(g.em_curva),
        // SE a traseira escorrega mais E o de trás é alto
        // ENTÃO sai de traseira.
        r2: g
            .traseira_escorrega_mais
            .min(g.traseira_alto)
            .min(g.em_curva),
        // SE a traseira escorrega um pouco mais E o de trás é alto
        // E o volante está contra a curva ENTÃO sai de traseira.
        r3: g
            .traseira_escorrega_um_pouco_mais
            .min(g.traseira_alto)
            .min(g.volante_contra_a_curva)
            .min(g.em_curva),
        // SE a traseira escorrega um pouco mais E o de trás é alto
        // E o acelerador foi aliviado de repente ENTÃO sai de traseira.
        r4: g
            .traseira_escorrega_um_pouco_mais
            .min(g.traseira_alto)
            .min(g.acelerador_aliviado)
            .min(g.em_curva),
        // SE os escorregamentos são parecidos E o maior deles é moderado
        // ENTÃO equilibrado.
        r5: g
            .escorregamentos_parecidos
            .min(g.maior_moderado)
            .min(g.em_curva),
    }
}

/// Altura do corte do conjunto "leve" para uma ativação.
pub fn corte_leve(ativacao: f64) -> f64 {
    (ativacao * 2.0).min(1.0)
}

/// Altura do corte do conjunto "forte" para uma ativação.
pub fn corte_forte(ativacao: f64) -> f64 {
    rampa(ativacao, 0.4, 1.0)
}

/// Etapas 3 e 4: valor da saída agregada no ponto `x` da régua de -1 a +1.
///
/// Cada conjunto de saída é recortado (mínimo) e os recortes são unidos
/// (máximo).
pub fn saida_agregada(x: f64, frente: f64, traseira: f64, equilibrado: f64) -> f64 {
    let frente_leve = corte_leve(frente).min(triangulo(x, 0.4, 0.4));
    let frente_forte = corte_forte(frente).min(triangulo(x, 0.85, 0.3));
    let traseira_leve = corte_leve(traseira).min(triangulo(x, -0.4, 0.4));
    let traseira_forte = corte_forte(traseira).min(triangulo(x, -0.85, 0.3));
    let neutro = equilibrado.min(triangulo(x, 0.0, 0.25));
    frente_leve
        .max(frente_forte)
        .max(traseira_leve)
        .max(traseira_forte)
        .max(neutro)
}

/// Etapa 5: centroide da área agregada, por integração numérica (regra do
/// trapézio) em 201 pontos, com passo de 0,01.
pub fn centroide(frente: f64, traseira: f64, equilibrado: f64) -> Option<f64> {
    let mut area = 0.0;
    let mut momento = 0.0;
    for i in 0..=200 {
        let x = f64::from(i) / 100.0 - 1.0;
        let altura = saida_agregada(x, frente, traseira, equilibrado);
        let peso = if i == 0 || i == 200 { 0.5 } else { 1.0 };
        area += altura * peso;
        momento += altura * peso * x;
    }
    (area > 1e-9).then(|| momento / area)
}

/// Executa as cinco etapas para uma amostra.
pub fn inferir(entradas: &Entradas) -> Decisao {
    let graus = fuzzificar(entradas);
    let regras = avaliar_regras(&graus);
    let comportamento = centroide(
        regras.sai_de_frente(),
        regras.sai_de_traseira(),
        regras.equilibrado(),
    );
    Decisao {
        graus,
        regras,
        comportamento,
    }
}
