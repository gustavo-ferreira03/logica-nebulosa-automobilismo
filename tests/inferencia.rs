use logica_nebulosa_automobilismo::{centroide, inferir, Decisao, Entradas};

/// Meio de uma curva, volante a favor dela e acelerador parado.
fn na_curva(frente: f64, traseira: f64) -> Decisao {
    inferir(&Entradas {
        frente,
        traseira,
        aceleracao_lateral: 8.0,
        rotacao: 0.3,
        volante: 0.2,
        alivio_acelerador: 0.0,
    })
}

fn perto(obtido: f64, esperado: f64) {
    assert!(
        (obtido - esperado).abs() < 0.005,
        "obtido {obtido}, esperado {esperado}"
    );
}

#[test]
fn exemplo_numerico() {
    let decisao = na_curva(4.0, 2.0);
    perto(decisao.graus.frente_escorrega_mais, 0.55);
    perto(decisao.graus.frente_alto, 0.67);
    perto(decisao.graus.em_curva, 1.0);
    perto(decisao.regras.sai_de_frente(), 0.55);
    perto(decisao.regras.sai_de_traseira(), 0.0);
    perto(decisao.comportamento.expect("a R1 dispara"), 0.46);
}

#[test]
fn centroide_cresce_com_a_ativacao() {
    for (ativacao, esperado) in [
        (0.2, 0.40),
        (0.4, 0.40),
        (0.55, 0.46),
        (0.7, 0.52),
        (1.0, 0.56),
    ] {
        perto(centroide(ativacao, 0.0, 0.0).expect("há área"), esperado);
    }
}

#[test]
fn traseira_e_o_espelho_da_frente() {
    let frente = na_curva(4.0, 2.0).comportamento.expect("a R1 dispara");
    let traseira = na_curva(2.0, 4.0).comportamento.expect("a R2 dispara");
    perto(traseira, -frente);
}

#[test]
fn numa_reta_nenhuma_regra_dispara() {
    let decisao = inferir(&Entradas {
        frente: 4.0,
        traseira: 2.0,
        aceleracao_lateral: 0.5,
        rotacao: 0.01,
        volante: 0.0,
        alivio_acelerador: 0.0,
    });
    assert!(decisao.comportamento.is_none());
}

#[test]
fn centroide_zero_nao_significa_equilibrio() {
    // Frente e traseira com ativação máxima se anulam no centroide, por isso as
    // duas ativações também são guardadas.
    perto(centroide(1.0, 1.0, 0.0).expect("há área"), 0.0);
}

#[test]
fn volante_contra_a_curva_antecipa_a_saida_de_traseira() {
    let amostra = Entradas {
        frente: 3.0,
        traseira: 4.0,
        aceleracao_lateral: 8.0,
        rotacao: 0.3,
        volante: 0.2,
        alivio_acelerador: 0.0,
    };
    let a_favor = inferir(&amostra).regras.sai_de_traseira();
    let contra = inferir(&Entradas {
        volante: -0.2,
        ..amostra
    })
    .regras
    .sai_de_traseira();
    assert!(contra > a_favor);
}
