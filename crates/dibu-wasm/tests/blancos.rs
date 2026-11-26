//! Valores en blanco (`!blank X`, `X.blank`) en todos los tipos: cada valor de cada ejemplo del
//! kitchen sink, de a uno, se pone en blanco. El diagrama se tiene que poder dibujar (el valor se
//! sigue usando para calcular), sin dejar marcas visibles en el SVG.
//!
//! `cobertura_de_valores_en_blanco` informa, por tipo, en cuántos lugares aparece el renglón para
//! completar (`cargo test -p dibu-wasm --test blancos -- --nocapture`).

use dibu_core::{leer_bloques, Bloque, BLANCO_ESCALAR, BLANCO_FIN, BLANCO_TEXTO};
use serde_json::Value;
use std::collections::BTreeMap;

const OMITIR: [&str; 7] = ["type", "id", "title", "subtitle", "theme", "inkSaver", "variant"];

fn ejemplos() -> Vec<Bloque> {
    leer_bloques(include_str!("../../../fixtures/kitchen_sink/kitchen_sink.yaml")).expect("kitchen sink válido")
}

/// Las rutas de los valores escalares (números, textos y booleanos) de un bloque.
fn hojas(v: &Value, base: &str, raiz: bool, salida: &mut Vec<String>) {
    match v {
        Value::Object(m) => {
            for (k, x) in m {
                if raiz && (OMITIR.contains(&k.as_str()) || k.starts_with("blank")) {
                    continue;
                }
                hojas(x, &format!("{base}/{}", k.replace('~', "~0").replace('/', "~1")), false, salida);
            }
        }
        Value::Array(l) => {
            for (i, x) in l.iter().enumerate() {
                hojas(x, &format!("{base}/{i}"), false, salida);
            }
        }
        Value::Null => {}
        // Un valor que el ejemplo ya deja en blanco no se vuelve a marcar.
        otro if dibu_core::es_blanco(otro) => {}
        _ => salida.push(base.to_string()),
    }
}

fn marcar(v: &Value) -> Value {
    match v {
        Value::String(s) => Value::String(format!("{BLANCO_TEXTO}{s}{BLANCO_FIN}")),
        otro => Value::String(format!("{BLANCO_ESCALAR}{}{BLANCO_FIN}", dibu_core::valor_str(otro))),
    }
}

fn renglones(svg: &str) -> usize {
    svg.matches(r#"stroke-dasharray="3 2""#).count()
}

#[derive(Default)]
struct Cuenta {
    ok: usize,
    /// El diagrama no dibuja ese valor como texto (se avisa con un error).
    sin_efecto: usize,
    /// El lector de ese campo no admite un valor en blanco (se avisa con un error).
    error: usize,
    errores: Vec<String>,
}

fn recorrer() -> (BTreeMap<String, Cuenta>, Vec<String>, Vec<String>) {
    let reg = dibu_wasm::registro();
    let mut por_tipo: BTreeMap<String, Cuenta> = BTreeMap::new();
    let mut marcas = Vec::new();
    let mut dudosos = Vec::new();
    for b in ejemplos() {
        let tipo = b["type"].as_str().unwrap().to_string();
        let Ok(original) = reg.renderizar(&b, true) else { continue };
        let base = renglones(&original);
        let raiz = Value::Object(b.clone());
        let mut lista = Vec::new();
        hojas(&raiz, "", true, &mut lista);
        let cuenta = por_tipo.entry(tipo.clone()).or_default();
        for ruta in lista {
            let mut v = raiz.clone();
            let hoja = v.pointer_mut(&ruta).unwrap();
            *hoja = marcar(hoja);
            let bloque = v.as_object().unwrap().clone();
            match reg.renderizar(&bloque, true) {
                Err(e) if e.0.contains("no dibuja ese valor") => cuenta.sin_efecto += 1,
                Err(e) => {
                    cuenta.error += 1;
                    cuenta.errores.push(format!("{} {ruta}: {}", b.get("id").and_then(Value::as_str).unwrap_or("?"), e.0));
                }
                Ok(svg) => {
                    if svg.chars().any(|c| matches!(c, BLANCO_TEXTO | BLANCO_FIN | BLANCO_ESCALAR)) {
                        marcas.push(format!("{tipo} {ruta}"));
                    }
                    // El valor se ve como renglón o como `____`; si no, se filtró.
                    if renglones(&svg) > base || svg.matches("____").count() > original.matches("____").count() {
                        cuenta.ok += 1;
                    } else {
                        dudosos.push(format!("{tipo} {ruta}"));
                    }
                }
            }
        }
    }
    (por_tipo, marcas, dudosos)
}

#[test]
fn cobertura_de_valores_en_blanco() {
    let (por_tipo, marcas, dudosos) = recorrer();
    let (mut ok, mut sin, mut err) = (0, 0, 0);
    for (t, c) in &por_tipo {
        ok += c.ok;
        sin += c.sin_efecto;
        err += c.error;
        eprintln!("{t:24} en blanco {:4}  no se dibuja {:4}  no admite {:4}", c.ok, c.sin_efecto, c.error);
        for e in c.errores.iter().take(4) {
            eprintln!("      {e}");
        }
    }
    eprintln!("TOTAL en blanco {ok}  no se dibuja {sin}  no admite {err}  dudosos {}", dudosos.len());
    // DIBU_BLANCOS_LOG=archivo: todos los errores, para revisarlos.
    if let Ok(ruta) = std::env::var("DIBU_BLANCOS_LOG") {
        let todo: Vec<String> = por_tipo.iter().flat_map(|(t, c)| c.errores.iter().map(move |e| format!("{t}\t{e}"))).collect();
        std::fs::write(ruta, todo.join("\n")).unwrap();
    }
    assert!(marcas.is_empty(), "marcas visibles en el SVG:\n{}", marcas.join("\n"));
    assert!(dudosos.is_empty(), "se dibujaron sin renglón ni ____:\n{}", dudosos.join("\n"));
}

#[test]
fn sintaxis_de_blank() {
    let reg = dibu_wasm::registro();
    for texto in [
        "type: array\nvalues: [3, !blank 4, 5]",
        "type: array\nvalues: [3, 4.blank, 5]",
        "{\"type\": \"array\", \"values\": [3, \"4.blank\", 5]}",
    ] {
        let b = &leer_bloques(texto).unwrap()[0];
        let svg = reg.renderizar(b, true).unwrap();
        assert!(!svg.contains(">4<"), "{texto}: el 4 no tiene que verse");
        assert!(svg.contains(">3<") && svg.contains(">5<"));
        assert!(svg.contains(r#"stroke-dasharray="3 2""#), "{texto}: falta el renglón");
    }
    // Por la API JSON, sin pasar por leer_bloques.
    let b: Bloque = serde_json::from_str(r#"{"type": "array", "values": [3, "4.blank", 5]}"#).unwrap();
    assert!(!reg.renderizar(&b, true).unwrap().contains(">4<"));
    // `!blank` sobre una lista deja en blanco todos sus valores.
    let b = &leer_bloques("type: array\nvalues: !blank [3, 4]").unwrap()[0];
    let svg = reg.renderizar(b, true).unwrap();
    assert!(!svg.contains(">3<") && !svg.contains(">4<"));
}
