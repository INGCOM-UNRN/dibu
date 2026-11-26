//! Valores imposibles en los diagramas de control: cada número, texto y lista de los ejemplos se
//! reemplaza por valores absurdos (cero, negativos, enormes, vacíos, de otro tipo). Cada render tiene
//! que terminar en un error con mensaje o en un SVG sin `NaN` ni `inf` en las coordenadas, sin
//! entrar en pánico y en poco tiempo.

use dibu_core::{leer_bloques, Bloque, Registro};
use serde_json::{json, Value};
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::time::{Duration, Instant};

const TIPOS: [&str; 9] = [
    "block_diagram",
    "time_response",
    "bode",
    "nyquist",
    "pole_zero",
    "root_locus",
    "instrumentation",
    "ladder",
    "grafcet",
];

fn ejemplos() -> Vec<Bloque> {
    let mut salida = Vec::new();
    for texto in [
        include_str!("../../../fixtures/diagramas_control.yaml"),
        include_str!("../../../fixtures/kitchen_sink/kitchen_sink.yaml"),
    ] {
        for b in leer_bloques(texto).expect("fixture YAML válido") {
            if b.get("type").and_then(Value::as_str).is_some_and(|t| TIPOS.contains(&t)) {
                salida.push(b);
            }
        }
    }
    salida
}

/// Las rutas (como punteros JSON) de todos los valores debajo de la raíz, salvo `type`.
fn rutas(v: &Value, base: &str, salida: &mut Vec<String>) {
    match v {
        Value::Object(m) => {
            for (k, x) in m {
                if base.is_empty() && (k == "type" || k == "id") {
                    continue;
                }
                let r = format!("{base}/{}", k.replace('~', "~0").replace('/', "~1"));
                salida.push(r.clone());
                rutas(x, &r, salida);
            }
        }
        Value::Array(l) => {
            for (i, x) in l.iter().enumerate() {
                let r = format!("{base}/{i}");
                salida.push(r.clone());
                rutas(x, &r, salida);
            }
        }
        _ => {}
    }
}

/// Los reemplazos para un valor según su clase.
fn reemplazos(v: &Value) -> Vec<Value> {
    let mut r = vec![Value::Null, json!(true), json!("¿?")];
    match v {
        Value::Number(_) => r.extend([
            json!(0),
            json!(-1),
            json!(-1000),
            json!(1e-12),
            json!(1e6),
            json!(1e12),
            json!(1e300),
            json!(-1e300),
            json!(u64::MAX),
            json!(i64::MIN),
        ]),
        Value::String(_) => r.extend([json!(""), json!("0"), json!("1/0"), json!("-1e308"), json!("s/0"), json!(0)]),
        Value::Array(l) => {
            r.extend([json!([]), json!({}), json!(0)]);
            if let Some(x) = l.first() {
                r.push(Value::Array(vec![x.clone(); 200]));
            }
        }
        Value::Object(_) => r.extend([json!({}), json!([]), json!(0)]),
        _ => r.extend([json!(0), json!(1e300)]),
    }
    r
}

/// Un número no finito en un atributo o en una ruta del SVG.
fn coordenada_rota(svg: &str) -> Option<String> {
    for (i, _) in svg.match_indices("NaN").chain(svg.match_indices("inf")) {
        let antes = &svg[..i];
        // Sólo dentro de una etiqueta (atributos), no en el texto visible.
        if antes.rfind('<') > antes.rfind('>') {
            let a = i.saturating_sub(40);
            return Some(svg[a..(i + 20).min(svg.len())].to_string());
        }
    }
    None
}

#[test]
fn valores_imposibles_no_rompen_los_diagramas_de_control() {
    let mut reg = Registro::new();
    dibu_control::registrar(&mut reg);
    let previo = std::panic::take_hook();
    std::panic::set_hook(Box::new(|_| {}));
    let mut fallas: Vec<String> = Vec::new();
    let mut renders = 0;
    for base in ejemplos() {
        let tipo = base["type"].as_str().unwrap().to_string();
        let raiz = Value::Object(base.clone());
        let mut lista = Vec::new();
        rutas(&raiz, "", &mut lista);
        for ruta in lista {
            let original = raiz.pointer(&ruta).unwrap().clone();
            for nuevo in reemplazos(&original) {
                let mut v = raiz.clone();
                *v.pointer_mut(&ruta).unwrap() = nuevo.clone();
                let b = v.as_object().unwrap().clone();
                let t0 = Instant::now();
                let res = catch_unwind(AssertUnwindSafe(|| reg.renderizar(&b, true)));
                let dt = t0.elapsed();
                renders += 1;
                let donde = format!("{tipo} {ruta} = {}", serde_json::to_string(&nuevo).unwrap().chars().take(40).collect::<String>());
                match res {
                    Err(p) => {
                        let msj = p
                            .downcast_ref::<String>()
                            .cloned()
                            .or_else(|| p.downcast_ref::<&str>().map(|s| s.to_string()))
                            .unwrap_or_default();
                        fallas.push(format!("PÁNICO {donde}: {msj}"));
                    }
                    Ok(Ok(svg)) => {
                        if let Some(trozo) = coordenada_rota(&svg) {
                            fallas.push(format!("NO FINITO {donde}: …{trozo}…"));
                        }
                    }
                    Ok(Err(e)) => {
                        if e.0.trim().is_empty() {
                            fallas.push(format!("ERROR SIN MENSAJE {donde}"));
                        }
                    }
                }
                if dt > Duration::from_secs(2) {
                    fallas.push(format!("LENTO {donde}: {dt:?}"));
                }
            }
        }
    }
    std::panic::set_hook(previo);
    eprintln!("robustez: {renders} renders de {} tipos", TIPOS.len());
    fallas.sort();
    fallas.dedup();
    assert!(
        fallas.is_empty(),
        "{} problemas en {renders} renders:\n{}",
        fallas.len(),
        fallas.join("\n")
    );
}

/// Combinaciones imposibles a propósito, que no salen de cambiar un solo valor.
#[test]
fn sistemas_imposibles_dan_un_error_o_un_dibujo_finito() {
    let sistemas = [
        json!({"num": [1], "den": [0, 0]}),
        json!({"num": [1], "den": []}),
        json!({"num": [], "den": [1, 1]}),
        json!({"num": [0], "den": [1, 1]}),
        json!({"num": [1, 0, 0], "den": [1, 1]}),
        json!({"num": [1], "den": [1, -10]}),
        json!({"num": [1], "den": [1, 0, 0, 0]}),
        json!({"gain": 1, "zeta": -0.5, "wn": 2}),
        json!({"gain": 1, "zeta": 0, "wn": 0}),
        json!({"gain": 1, "zeta": 0.5, "wn": -3}),
        json!({"gain": 1, "zeta": 1e300, "wn": 1e300}),
        json!({"gain": 0, "tau": 1}),
        json!({"gain": 1, "tau": 0}),
        json!({"gain": 1, "tau": -2}),
        json!({"gain": 1, "tau": 1, "delay": -1}),
        json!({"gain": 1, "tau": 1, "delay": 1e9}),
        json!({"k": 1, "poles": [0, 0, 0]}),
        json!({"k": 1, "poles": [], "zeros": [0]}),
        json!({"k": 1, "poles": [[0, 0]], "zeros": [[0, 0]]}),
        json!({"k": 0, "poles": [-1, -1]}),
        json!({"k": 1, "poles": [-1, -1], "zeros": [-1, -1]}),
        json!({"kp": 0, "ki": 0, "kd": 0}),
        json!({"kp": 1, "ki": 0, "kd": 1e12}),
    ];
    let mut bloques: Vec<Value> = Vec::new();
    for s in &sistemas {
        bloques.push(json!({"type": "time_response", "system": s, "annotations": "all"}));
        bloques.push(json!({"type": "time_response", "system": s, "input": "impulse", "tEnd": 1e6}));
        bloques.push(json!({"type": "time_response", "system": s, "input": "ramp", "tEnd": 1e-300, "settlingBand": 2}));
        bloques.push(json!({"type": "time_response", "systems": [{"label": "a", "num": [1], "den": [1, 1]}, s]}));
        bloques.push(json!({"type": "time_response", "closedLoop": {"plant": s, "controller": {"kp": 1e300}}, "annotations": "all"}));
        bloques.push(json!({"type": "bode", "system": s, "asymptotes": true, "margins": true}));
        bloques.push(json!({"type": "bode", "system": s, "wMin": 0, "wMax": 0}));
        bloques.push(json!({"type": "nyquist", "system": s, "limit": 0, "showNegative": true}));
        bloques.push(json!({"type": "nyquist", "system": s, "wMin": -1, "wMax": 1e300}));
        bloques.push(json!({"type": "pole_zero", "system": s, "dampingLines": [2, -1, 0], "wnCircles": [-1, 0]}));
        bloques.push(json!({"type": "root_locus", "system": s, "asymptotes": true, "showCrossing": true, "marks": [-1, 0, 1e300]}));
        bloques.push(json!({"type": "root_locus", "system": s, "kMax": -5}));
        bloques.push(json!({"type": "root_locus", "system": s, "kMax": 1e308}));
    }
    // Un lazo con 1 + G·H = 0: el denominador del lazo cerrado se anula.
    let nulo = json!({"plant": {"num": [-1], "den": [1]}, "feedback": {"num": [1], "den": [1]}, "sign": "-"});
    for t in ["time_response", "bode", "nyquist", "pole_zero", "root_locus"] {
        bloques.push(json!({"type": t, "closedLoop": nulo}));
    }
    bloques.extend([
        json!({"type": "time_response", "system": {"tau": 1}, "amplitude": 1e308, "showInput": true}),
        json!({"type": "time_response", "system": {"tau": 1}, "data": [[-1, 1e300], [1e300, -1e300], ["a", null]]}),
        json!({"type": "time_response", "system": {"tau": 1}, "disturbance": {"at": -5, "amplitude": 1e308, "where": "nada"}}),
        json!({"type": "time_response", "system": {"tau": 1}, "yMin": 0, "yMax": 0}),
        json!({"type": "block_diagram", "forward": ["G"], "disturbance": {"at": 1000}}),
        json!({"type": "block_diagram", "forward": ["G"], "disturbance": {"at": -1}}),
        json!({"type": "block_diagram", "forward": [], "feedback": []}),
    ]);
    // Lazos anidados muy profundos.
    let mut anidado = json!({"forward": ["G"], "feedback": ["H"]});
    for _ in 0..120 {
        anidado = json!({"forward": [{"loop": anidado}], "feedback": []});
    }
    let mut prof = anidado.as_object().unwrap().clone();
    prof.insert("type".into(), json!("block_diagram"));
    bloques.push(Value::Object(prof));

    let mut reg = Registro::new();
    dibu_control::registrar(&mut reg);
    let previo = std::panic::take_hook();
    std::panic::set_hook(Box::new(|_| {}));
    let mut fallas = Vec::new();
    for v in &bloques {
        let b = v.as_object().unwrap().clone();
        let t0 = Instant::now();
        let res = catch_unwind(AssertUnwindSafe(|| reg.renderizar(&b, true)));
        let dt = t0.elapsed();
        let donde = serde_json::to_string(v).unwrap();
        match res {
            Err(_) => fallas.push(format!("PÁNICO {donde}")),
            Ok(Ok(svg)) => {
                if let Some(trozo) = coordenada_rota(&svg) {
                    fallas.push(format!("NO FINITO {donde}: …{trozo}…"));
                }
            }
            Ok(Err(e)) if e.0.trim().is_empty() => fallas.push(format!("ERROR SIN MENSAJE {donde}")),
            Ok(Err(_)) => {}
        }
        if dt > Duration::from_secs(2) {
            fallas.push(format!("LENTO {donde}: {dt:?}"));
        }
    }
    std::panic::set_hook(previo);
    assert!(fallas.is_empty(), "{} problemas en {} bloques:\n{}", fallas.len(), bloques.len(), fallas.join("\n"));
}
