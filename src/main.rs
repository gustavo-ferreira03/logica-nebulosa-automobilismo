//! Sistema externo: fornece as medidas ao módulo nebuloso, uma amostra por
//! vez, e registra as decisões.

use logica_nebulosa_automobilismo::{corte_forte, corte_leve, inferir, Entradas};
use std::{env, error::Error, fs, process::ExitCode};

/// Ativação a partir da qual o programa dá nome ao comportamento. É só um
/// critério de exibição; não faz parte da inferência.
const LIMIAR: f64 = 0.35;

const USO: &str = "Uso:
  cargo run                                    exemplo numérico passo a passo
  cargo run -- valores <frente> <traseira>     um par de escorregamentos, em graus, no meio de uma curva
  cargo run -- trecho dados/passagem-a.csv     decisões sucessivas sobre uma gravação";

fn main() -> ExitCode {
    let argumentos: Vec<String> = env::args().skip(1).collect();
    let argumentos: Vec<&str> = argumentos.iter().map(String::as_str).collect();
    let resultado = match argumentos.as_slice() {
        [] => {
            exemplo(4.0, 2.0);
            Ok(())
        }
        ["valores", frente, traseira] => valores(frente, traseira),
        ["trecho", arquivo] => trecho(arquivo),
        _ => Err(USO.into()),
    };
    match resultado {
        Ok(()) => ExitCode::SUCCESS,
        Err(erro) => {
            eprintln!("{erro}");
            ExitCode::FAILURE
        }
    }
}

fn valores(frente: &str, traseira: &str) -> Result<(), Box<dyn Error>> {
    exemplo(
        frente.replace(',', ".").parse()?,
        traseira.replace(',', ".").parse()?,
    );
    Ok(())
}

/// Mostra cada etapa para um par de escorregamentos no meio de uma curva, com
/// o volante a favor dela e o acelerador parado.
fn exemplo(frente: f64, traseira: f64) {
    let decisao = inferir(&Entradas {
        frente,
        traseira,
        aceleracao_lateral: 8.0,
        rotacao: 0.3,
        volante: 0.2,
        alivio_acelerador: 0.0,
    });
    let g = decisao.graus;
    let r = decisao.regras;
    println!("Entradas: frente {frente}°, traseira {traseira}°, no meio de uma curva\n");
    println!("1. Fuzzificação");
    for (nome, grau) in [
        ("carro em curva", g.em_curva),
        ("frente escorrega mais", g.frente_escorrega_mais),
        ("traseira escorrega mais", g.traseira_escorrega_mais),
        (
            "traseira escorrega um pouco mais",
            g.traseira_escorrega_um_pouco_mais,
        ),
        ("escorregamentos parecidos", g.escorregamentos_parecidos),
        ("escorregamento da frente alto", g.frente_alto),
        ("escorregamento de trás alto", g.traseira_alto),
        ("maior escorregamento moderado", g.maior_moderado),
        ("volante contra a curva", g.volante_contra_a_curva),
        ("acelerador aliviado", g.acelerador_aliviado),
    ] {
        println!("   {nome:<34}{grau:.2}");
    }
    println!("\n2. Regras (E = mínimo)");
    println!(
        "   R1 {:.2}   R2 {:.2}   R3 {:.2}   R4 {:.2}   R5 {:.2}",
        r.r1, r.r2, r.r3, r.r4, r.r5
    );
    println!("\n3. Implicação (altura dos cortes)");
    for (nome, ativacao) in [
        ("sai de frente", r.sai_de_frente()),
        ("sai de traseira", r.sai_de_traseira()),
    ] {
        println!(
            "   {nome:<17}leve {:.2}   forte {:.2}",
            corte_leve(ativacao),
            corte_forte(ativacao)
        );
    }
    println!("   {:<17}{:.2}", "equilibrado", r.equilibrado());
    println!("\n4 e 5. Agregação (máximo) e centroide");
    match decisao.comportamento {
        Some(c) => println!("   comportamento = {c:+.3}   (-1 sai de traseira, +1 sai de frente)"),
        None => println!("   nenhuma regra disparou"),
    }
}

/// Percorre uma gravação e toma uma decisão por amostra.
fn trecho(arquivo: &str) -> Result<(), Box<dyn Error>> {
    let texto = fs::read_to_string(arquivo)?;
    let (mut frente_n, mut traseira_n, mut total) = (0, 0, 0);
    println!(
        "{:>8} {:>10} {:>8} {:>9} {:>8} {:>10} {:>9}   decisão",
        "tempo s", "posição m", "frente°", "traseira°", "S.frente", "S.traseira", "centroide"
    );
    for (numero, linha) in texto.lines().enumerate().skip(1) {
        let campos = linha
            .split(',')
            .map(str::parse)
            .collect::<Result<Vec<f64>, _>>()
            .map_err(|erro| format!("linha {}: {erro}", numero + 1))?;
        let [tempo, posicao, frente, traseira, aceleracao_lateral, rotacao, volante, alivio_acelerador] =
            campos[..]
        else {
            return Err(format!("linha {}: esperava 8 colunas", numero + 1).into());
        };
        let decisao = inferir(&Entradas {
            frente,
            traseira,
            aceleracao_lateral,
            rotacao,
            volante,
            alivio_acelerador,
        });
        let sf = decisao.regras.sai_de_frente();
        let st = decisao.regras.sai_de_traseira();
        let nome = if sf >= LIMIAR && sf > st {
            frente_n += 1;
            "sai de frente"
        } else if st >= LIMIAR && st > sf {
            traseira_n += 1;
            "sai de traseira"
        } else {
            ""
        };
        let centroide = decisao.comportamento.map_or_else(
            || "-".to_string(),
            |c| format!("{:+.3}", if c.abs() < 0.0005 { 0.0 } else { c }),
        );
        println!(
            "{tempo:>8.2} {posicao:>10.1} {frente:>8.2} {traseira:>9.2} {sf:>8.2} {st:>10.2} {centroide:>9}   {nome}"
        );
        total += 1;
    }
    println!(
        "\n{total} amostras: {frente_n} saindo de frente, {traseira_n} saindo de traseira (ativação mínima {LIMIAR})."
    );
    Ok(())
}
