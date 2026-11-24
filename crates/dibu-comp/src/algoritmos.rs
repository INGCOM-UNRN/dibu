//! Algoritmos: `dp_table` (programación dinámica), `dijkstra` (tabla de iteraciones) y
//! `turing` (traza de una máquina de Turing).

use crate::comun::*;
use serde_json::Value;
use std::collections::BTreeMap;

// ---------------------------------------------------------------------------
// dp_table
// ---------------------------------------------------------------------------

pub fn tabla_lcs(a: &[char], b: &[char]) -> Vec<Vec<i64>> {
    let mut t = vec![vec![0i64; b.len() + 1]; a.len() + 1];
    for i in 1..=a.len() {
        for j in 1..=b.len() {
            t[i][j] = if a[i - 1] == b[j - 1] {
                t[i - 1][j - 1] + 1
            } else {
                t[i - 1][j].max(t[i][j - 1])
            };
        }
    }
    t
}

pub fn tabla_edicion(a: &[char], b: &[char]) -> Vec<Vec<i64>> {
    let mut t = vec![vec![0i64; b.len() + 1]; a.len() + 1];
    for (i, fila) in t.iter_mut().enumerate() {
        fila[0] = i as i64;
    }
    for (j, celda) in t[0].iter_mut().enumerate() {
        *celda = j as i64;
    }
    for i in 1..=a.len() {
        for j in 1..=b.len() {
            let costo = if a[i - 1] == b[j - 1] { 0 } else { 1 };
            t[i][j] = (t[i - 1][j] + 1)
                .min(t[i][j - 1] + 1)
                .min(t[i - 1][j - 1] + costo);
        }
    }
    t
}

pub fn tabla_mochila(pesos: &[usize], valores: &[f64], cap: usize) -> Vec<Vec<f64>> {
    let n = pesos.len();
    let mut t = vec![vec![0.0_f64; cap + 1]; n + 1];
    for i in 1..=n {
        for w in 0..=cap {
            t[i][w] = t[i - 1][w];
            if pesos[i - 1] <= w {
                t[i][w] = t[i][w].max(t[i - 1][w - pesos[i - 1]] + valores[i - 1]);
            }
        }
    }
    t
}

pub fn tabla_monedas(monedas: &[usize], monto: usize) -> Vec<Option<usize>> {
    let mut t = vec![None; monto + 1];
    t[0] = Some(0);
    for m in 1..=monto {
        t[m] = monedas
            .iter()
            .filter(|&&c| c <= m && c > 0)
            .filter_map(|&c| t[m - c].map(|x| x + 1))
            .min();
    }
    t
}

fn texto_seq(b: &Bloque, clave: &str) -> Result<Vec<char>, ErrorDiagrama> {
    let s = str_b(b, clave, "");
    if s.is_empty() {
        return Err(err(format!("falta '{clave}' (texto)")));
    }
    let v: Vec<char> = s.chars().collect();
    if v.len() > 24 {
        return Err(err(format!("'{clave}' admite como máximo 24 caracteres")));
    }
    Ok(v)
}

pub fn render_dp_table(c: &mut Lienzo, b: &Bloque) -> Result<Punto, ErrorDiagrama> {
    let problema = opcion(b, "problem", &["lcs", "edit", "knapsack", "coins"], "lcs")?;
    let blanco = bool_opt(b, "blank", false);
    let camino_on = bool_opt(b, "path", true) && !blanco;
    let y0 = encabezado(c, b, "");
    let y = y0 + 4.0;
    let (enc, filas, resultado): (Vec<String>, Vec<Vec<Celda>>, String) =
        match problema.as_str() {
            "lcs" | "edit" => {
                let (a, bb) = (texto_seq(b, "a")?, texto_seq(b, "b")?);
                let es_lcs = problema == "lcs";
                let t = if es_lcs {
                    tabla_lcs(&a, &bb)
                } else {
                    tabla_edicion(&a, &bb)
                };
                // Reconstrucción.
                let mut en_camino = vec![vec![false; bb.len() + 1]; a.len() + 1];
                let (mut i, mut j) = (a.len(), bb.len());
                let mut lcs = String::new();
                let mut ops: Vec<String> = Vec::new();
                loop {
                    en_camino[i][j] = true;
                    if i == 0 && j == 0 {
                        break;
                    }
                    if es_lcs {
                        if i == 0 || j == 0 {
                            break;
                        }
                        if a[i - 1] == bb[j - 1] {
                            lcs.insert(0, a[i - 1]);
                            i -= 1;
                            j -= 1;
                        } else if t[i - 1][j] >= t[i][j - 1] {
                            i -= 1;
                        } else {
                            j -= 1;
                        }
                    } else if i > 0
                        && j > 0
                        && t[i][j] == t[i - 1][j - 1] + if a[i - 1] == bb[j - 1] { 0 } else { 1 }
                    {
                        if a[i - 1] != bb[j - 1] {
                            ops.insert(0, format!("sustituir {}→{}", a[i - 1], bb[j - 1]));
                        }
                        i -= 1;
                        j -= 1;
                    } else if i > 0 && t[i][j] == t[i - 1][j] + 1 {
                        ops.insert(0, format!("borrar {}", a[i - 1]));
                        i -= 1;
                    } else {
                        ops.insert(0, format!("insertar {}", bb[j - 1]));
                        j -= 1;
                    }
                }
                let mut enc = vec![String::new(), "ε".to_string()];
                enc.extend(bb.iter().map(|ch| ch.to_string()));
                let filas: Vec<Vec<Celda>> = (0..=a.len())
                    .map(|i| {
                        let mut f = vec![celda(if i == 0 {
                            "ε".into()
                        } else {
                            a[i - 1].to_string()
                        })
                        .fuerte(true)];
                        for j in 0..=bb.len() {
                            let borde = i == 0 || j == 0;
                            f.push(
                                celda(t[i][j].to_string())
                                    .en_blanco(blanco && !borde)
                                    .resaltada(camino_on && en_camino[i][j]),
                            );
                        }
                        f
                    })
                    .collect();
                let a_s: String = a.iter().collect();
                let b_s: String = bb.iter().collect();
                let res = if es_lcs {
                    format!(
                        "LCS(\"{a_s}\", \"{b_s}\") = {} → \"{lcs}\"",
                        t[a.len()][bb.len()]
                    )
                } else {
                    format!(
                        "distancia(\"{a_s}\", \"{b_s}\") = {}{}",
                        t[a.len()][bb.len()],
                        if ops.is_empty() {
                            String::new()
                        } else {
                            format!(" → {}", ops.join(", "))
                        }
                    )
                };
                (enc, filas, res)
            }
            "knapsack" => {
                let pesos: Vec<usize> = lista_num(b, "weights", false)?
                    .iter()
                    .map(|&w| w.max(0.0) as usize)
                    .collect();
                let valores = lista_num(b, "values", false)?;
                let cap = entero_b(b, "capacity", 10, 1, 40)? as usize;
                if pesos.len() != valores.len() || pesos.is_empty() || pesos.len() > 15 {
                    return Err(err(
                        "'weights' y 'values' tienen el mismo largo (1 a 15 objetos)",
                    ));
                }
                let t = tabla_mochila(&pesos, &valores, cap);
                let mut en_camino = vec![vec![false; cap + 1]; pesos.len() + 1];
                let mut elegidos = Vec::new();
                let mut w = cap;
                for i in (1..=pesos.len()).rev() {
                    en_camino[i][w] = true;
                    if t[i][w] != t[i - 1][w] {
                        elegidos.insert(0, i);
                        w -= pesos[i - 1];
                    }
                }
                en_camino[0][w] = true;
                let mut enc = vec!["objeto (p, v)".to_string()];
                enc.extend((0..=cap).map(|w| w.to_string()));
                let filas: Vec<Vec<Celda>> = (0..=pesos.len())
                    .map(|i| {
                        let nombre = if i == 0 {
                            "—".into()
                        } else {
                            format!("{i} ({}, {})", pesos[i - 1], fmt(valores[i - 1], 2))
                        };
                        let mut f = vec![celda(nombre).fuerte(true)];
                        for w in 0..=cap {
                            f.push(
                                celda(fmt(t[i][w], 2))
                                    .en_blanco(blanco && i > 0)
                                    .resaltada(camino_on && en_camino[i][w]),
                            );
                        }
                        f
                    })
                    .collect();
                let res = format!(
                    "valor máximo = {} · objetos elegidos: {}",
                    fmt(t[pesos.len()][cap], 2),
                    if elegidos.is_empty() {
                        "ninguno".into()
                    } else {
                        elegidos
                            .iter()
                            .map(|i| i.to_string())
                            .collect::<Vec<_>>()
                            .join(", ")
                    }
                );
                (enc, filas, res)
            }
            _ => {
                let monedas: Vec<usize> = lista_num(b, "coins", false)?
                    .iter()
                    .map(|&x| x.max(0.0) as usize)
                    .collect();
                let monto = entero_b(b, "amount", 10, 1, 60)? as usize;
                if monedas.is_empty() || monedas.contains(&0) {
                    return Err(err("'coins' es una lista de valores enteros positivos"));
                }
                let t = tabla_monedas(&monedas, monto);
                let mut usados = Vec::new();
                let mut m = monto;
                let mut en_camino = vec![false; monto + 1];
                en_camino[m] = true;
                while m > 0 {
                    let Some(cm) = monedas.iter().copied().find(|&cm| {
                        cm <= m && t[m - cm].is_some() && t[m - cm].map(|x| x + 1) == t[m]
                    }) else {
                        break;
                    };
                    usados.push(cm);
                    m -= cm;
                    en_camino[m] = true;
                }
                let mut enc = vec!["monto".to_string()];
                enc.extend((0..=monto).map(|x| x.to_string()));
                let fila = std::iter::once(celda("monedas").fuerte(true))
                    .chain((0..=monto).map(|x| {
                        celda(t[x].map(|v| v.to_string()).unwrap_or_else(|| "∞".into()))
                            .en_blanco(blanco && x > 0)
                            .resaltada(camino_on && en_camino[x])
                    }))
                    .collect();
                let res = match t[monto] {
                    Some(v) => format!(
                        "mínimo de monedas para {monto} = {v} → {}",
                        usados
                            .iter()
                            .map(|x| x.to_string())
                            .collect::<Vec<_>>()
                            .join(" + ")
                    ),
                    None => format!("no se puede formar {monto} con esas monedas"),
                };
                (enc, vec![fila], res)
            }
        };
    let (w, h) = tabla(
        c,
        0.0,
        y,
        &enc,
        &filas,
        &OpcTabla {
            fila_h: 24.0,
            min_w: 28.0,
            separadores: &[0],
            ..Default::default()
        },
    );
    let mut yy = y + h + 20.0;
    let txt = if blanco {
        "Resultado: ____".to_string()
    } else {
        resultado
    };
    c.t(0.0, yy, &txt, 10.5, 700, "textMain", "start", MONO);
    yy += 6.0;
    Ok((w.max(ancho_texto(&txt, 10.5)), yy))
}

// ---------------------------------------------------------------------------
// dijkstra
// ---------------------------------------------------------------------------

/// (origen, destino, peso)
pub type Arista = (usize, usize, f64);

/// Aristas "A-B:4" (no dirigida) o "A->B:4" (dirigida), o {from, to, w}.
pub fn leer_aristas(b: &Bloque) -> Result<(Vec<String>, Vec<Arista>), ErrorDiagrama> {
    let mut vertices: Vec<String> = lista_str(b, "vertices", true)?;
    let dirigido_global = bool_opt(b, "directed", false);
    let mut aristas = Vec::new();
    let idx = |n: &str, vs: &mut Vec<String>| -> usize {
        match vs.iter().position(|v| v == n) {
            Some(i) => i,
            None => {
                vs.push(n.to_string());
                vs.len() - 1
            }
        }
    };
    for (k, v) in lista(b, "edges", false)?.iter().enumerate() {
        let (a, z, w, dir) = match v {
            Value::String(s) => {
                let (par, peso) = s
                    .rsplit_once(':')
                    .ok_or_else(|| err(format!("edges[{k}]: formato \"A-B:4\" o \"A->B:4\"")))?;
                let w = num(&Value::String(peso.into()))
                    .ok_or_else(|| err(format!("edges[{k}]: peso inválido")))?;
                let (a, z, dir) = if let Some((a, z)) = par.split_once("->") {
                    (a, z, true)
                } else if let Some((a, z)) = par.split_once('-') {
                    (a, z, dirigido_global)
                } else {
                    return Err(err(format!("edges[{k}]: falta '-' o '->'")));
                };
                (a.trim().to_string(), z.trim().to_string(), w, dir)
            }
            Value::Object(m) => (
                m.get("from")
                    .map(valor_str)
                    .ok_or_else(|| err(format!("edges[{k}] necesita 'from'")))?,
                m.get("to")
                    .map(valor_str)
                    .ok_or_else(|| err(format!("edges[{k}] necesita 'to'")))?,
                m.get("w")
                    .or_else(|| m.get("weight"))
                    .and_then(num)
                    .unwrap_or(1.0),
                dirigido_global,
            ),
            _ => return Err(err("cada arista es \"A-B:4\" o {from, to, w}")),
        };
        if w < 0.0 {
            return Err(err(format!(
                "edges[{k}]: Dijkstra no admite pesos negativos (usar Bellman-Ford)"
            )));
        }
        let (ia, iz) = (idx(&a, &mut vertices), idx(&z, &mut vertices));
        aristas.push((ia, iz, w));
        if !dir {
            aristas.push((iz, ia, w));
        }
    }
    if vertices.len() > 20 {
        return Err(err("como máximo 20 vértices"));
    }
    Ok((vertices, aristas))
}

pub struct Iteracion {
    pub elegido: Option<usize>,
    pub dist: Vec<f64>,
    pub prev: Vec<Option<usize>>,
    pub visitados: Vec<bool>,
}

pub fn dijkstra(n: usize, aristas: &[(usize, usize, f64)], s: usize) -> Vec<Iteracion> {
    let mut dist = vec![f64::INFINITY; n];
    let mut prev = vec![None; n];
    let mut vis = vec![false; n];
    dist[s] = 0.0;
    let mut out = vec![Iteracion {
        elegido: None,
        dist: dist.clone(),
        prev: prev.clone(),
        visitados: vis.clone(),
    }];
    for _ in 0..n {
        let Some(u) = (0..n)
            .filter(|&v| !vis[v] && dist[v].is_finite())
            .min_by(|&a, &b| dist[a].partial_cmp(&dist[b]).unwrap().then(a.cmp(&b)))
        else {
            break;
        };
        vis[u] = true;
        for &(a, z, w) in aristas {
            if a == u && !vis[z] && dist[u] + w < dist[z] - 1e-12 {
                dist[z] = dist[u] + w;
                prev[z] = Some(u);
            }
        }
        out.push(Iteracion {
            elegido: Some(u),
            dist: dist.clone(),
            prev: prev.clone(),
            visitados: vis.clone(),
        });
    }
    out
}

pub fn render_dijkstra(c: &mut Lienzo, b: &Bloque) -> Result<Punto, ErrorDiagrama> {
    let (vs, aristas) = leer_aristas(b)?;
    let origen = str_b(b, "source", vs.first().map(String::as_str).unwrap_or(""));
    let s = vs
        .iter()
        .position(|v| *v == origen)
        .ok_or_else(|| err(format!("'source' {origen:?} no es un vértice")))?;
    let blanco = bool_opt(b, "blank", false);
    let its = dijkstra(vs.len(), &aristas, s);
    let y0 = encabezado(c, b, &format!("origen {origen}"));
    let mut enc = vec!["Paso".to_string(), "Visita".to_string()];
    enc.extend(vs.iter().cloned());
    let filas: Vec<Vec<Celda>> = its
        .iter()
        .enumerate()
        .map(|(k, it)| {
            let mut f = vec![
                celda(k.to_string()).atenuada(true),
                celda(
                    it.elegido
                        .map(|u| vs[u].clone())
                        .unwrap_or_else(|| "—".into()),
                )
                .fuerte(true)
                .en_blanco(blanco && k > 0),
            ];
            for v in 0..vs.len() {
                let ya = k > 0 && its[k - 1].visitados[v];
                let txt = if ya {
                    "✓".to_string()
                } else if it.dist[v].is_infinite() {
                    "∞".to_string()
                } else {
                    format!(
                        "{}, {}",
                        fmt(it.dist[v], 3),
                        it.prev[v]
                            .map(|p| vs[p].clone())
                            .unwrap_or_else(|| "—".into())
                    )
                };
                f.push(
                    celda(txt)
                        .atenuada(ya)
                        .en_blanco(blanco && k > 0 && !ya)
                        .resaltada(!blanco && it.elegido == Some(v)),
                );
            }
            f
        })
        .collect();
    let (w, h) = tabla(
        c,
        0.0,
        y0 + 4.0,
        &enc,
        &filas,
        &OpcTabla {
            fila_h: 22.0,
            min_w: 46.0,
            separadores: &[1],
            ..Default::default()
        },
    );
    let mut y = y0 + 4.0 + h + 10.0;
    c.t(
        0.0,
        y + 4.0,
        "Cada celda: distancia, predecesor · ✓ = vértice ya visitado",
        9.5,
        400,
        "textMuted",
        "start",
        MONO,
    );
    y += 14.0;
    let mut ancho = w;
    if bool_opt(b, "paths", true) {
        let fin = its.last().unwrap();
        let filas: Vec<Vec<Celda>> = (0..vs.len())
            .map(|v| {
                let mut camino = vec![vs[v].clone()];
                let mut u = v;
                while let Some(p) = fin.prev[u] {
                    camino.insert(0, vs[p].clone());
                    u = p;
                    if camino.len() > vs.len() {
                        break;
                    }
                }
                let alcanzable = fin.dist[v].is_finite();
                vec![
                    celda(&vs[v]).fuerte(true),
                    celda(if alcanzable {
                        fmt(fin.dist[v], 3)
                    } else {
                        "∞".into()
                    })
                    .en_blanco(blanco),
                    celda(if alcanzable {
                        camino.join(" → ")
                    } else {
                        "inalcanzable".into()
                    })
                    .en_blanco(blanco),
                ]
            })
            .collect();
        let enc: Vec<String> = ["Destino", "Distancia", "Camino mínimo"]
            .iter()
            .map(|s| s.to_string())
            .collect();
        y += 10.0;
        let (w2, h2) = tabla(
            c,
            0.0,
            y,
            &enc,
            &filas,
            &OpcTabla {
                fila_h: 21.0,
                izquierda: &[2],
                ..Default::default()
            },
        );
        ancho = ancho.max(w2);
        y += h2;
    }
    Ok((
        ancho.max(ancho_texto(
            "Cada celda: distancia, predecesor · ✓ = vértice ya visitado",
            9.5,
        )),
        y + 6.0,
    ))
}

// ---------------------------------------------------------------------------
// turing
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct Transicion {
    pub escribe: char,
    pub mueve: i64,
    pub siguiente: String,
}

/// "q0,1 -> q1,X,R" | "q0 1 q1 X R" | clave "q0,1": "q1,X,R".
fn parsear_transicion(izq: &str, der: &str) -> Result<((String, char), Transicion), ErrorDiagrama> {
    let sep = |s: &str| -> Vec<String> {
        s.split(|ch: char| ch == ',' || ch.is_whitespace())
            .filter(|x| !x.is_empty())
            .map(String::from)
            .collect()
    };
    let i = sep(izq);
    let d = sep(der);
    let mal = || {
        err(format!(
            "transición inválida {izq:?} → {der:?} (formato: \"q0,1 -> q1,X,R\")"
        ))
    };
    if i.len() != 2 || d.len() != 3 {
        return Err(mal());
    }
    let leer = i[1].chars().next().ok_or_else(mal)?;
    let escribe = d[1].chars().next().ok_or_else(mal)?;
    let mueve = match d[2].to_uppercase().as_str() {
        "R" | "D" | ">" => 1,
        "L" | "I" | "<" => -1,
        "S" | "N" | "-" => 0,
        _ => return Err(mal()),
    };
    Ok((
        (i[0].clone(), leer),
        Transicion {
            escribe,
            mueve,
            siguiente: d[0].clone(),
        },
    ))
}

pub struct Config {
    pub estado: String,
    pub cinta: BTreeMap<i64, char>,
    pub cabezal: i64,
}

pub fn render_turing(c: &mut Lienzo, b: &Bloque) -> Result<Punto, ErrorDiagrama> {
    let blanco_sim = str_b(b, "blankSymbol", "_").chars().next().unwrap_or('_');
    let mut delta: BTreeMap<(String, char), Transicion> = BTreeMap::new();
    match b.get("transitions") {
        Some(Value::Object(m)) => {
            for (k, v) in m {
                let (clave, t) = parsear_transicion(k, &valor_str(v))?;
                delta.insert(clave, t);
            }
        }
        Some(Value::Array(a)) => {
            for v in a {
                let s = valor_str(v);
                let (izq, der) = s
                    .split_once("->")
                    .or_else(|| s.split_once('→'))
                    .ok_or_else(|| err(format!("transición {s:?}: falta '->'")))?;
                let (clave, t) = parsear_transicion(izq, der)?;
                delta.insert(clave, t);
            }
        }
        _ => return Err(err("falta 'transitions' (lista \"q0,1 -> q1,X,R\" o mapa)")),
    }
    if delta.len() > 80 {
        return Err(err("como máximo 80 transiciones"));
    }
    let inicial = str_b(b, "start", "q0");
    let aceptacion = lista_str(b, "accept", true)?;
    let entrada = str_b(b, "input", "");
    if entrada.chars().count() > 40 {
        return Err(err("'input' admite como máximo 40 símbolos"));
    }
    let max_pasos = entero_b(b, "maxSteps", 40, 1, 200)? as usize;
    let blanco = bool_opt(b, "blank", false);
    let mut cfg = Config {
        estado: inicial.clone(),
        cinta: entrada
            .chars()
            .enumerate()
            .map(|(i, ch)| (i as i64, ch))
            .collect(),
        cabezal: 0,
    };
    let mut historia: Vec<(String, BTreeMap<i64, char>, i64)> =
        vec![(cfg.estado.clone(), cfg.cinta.clone(), cfg.cabezal)];
    let mut final_ = "no se detuvo";
    for _ in 0..max_pasos {
        if aceptacion.contains(&cfg.estado) {
            final_ = "acepta";
            break;
        }
        let simbolo = *cfg.cinta.get(&cfg.cabezal).unwrap_or(&blanco_sim);
        let Some(t) = delta.get(&(cfg.estado.clone(), simbolo)) else {
            final_ = "se detiene sin transición (rechaza)";
            break;
        };
        cfg.cinta.insert(cfg.cabezal, t.escribe);
        cfg.cabezal += t.mueve;
        cfg.estado = t.siguiente.clone();
        historia.push((cfg.estado.clone(), cfg.cinta.clone(), cfg.cabezal));
    }
    if final_ == "no se detuvo" && aceptacion.contains(&cfg.estado) {
        final_ = "acepta";
    }
    let lo = historia
        .iter()
        .flat_map(|(_, cinta, h)| cinta.keys().copied().chain(std::iter::once(*h)))
        .min()
        .unwrap_or(0)
        - 1;
    let hi = historia
        .iter()
        .flat_map(|(_, cinta, h)| cinta.keys().copied().chain(std::iter::once(*h)))
        .max()
        .unwrap_or(0)
        + 1;
    if hi - lo > 80 {
        return Err(err("la cinta usada es demasiado larga para dibujarla"));
    }
    let y0 = encabezado(
        c,
        b,
        &format!("entrada \"{entrada}\" · estado inicial {inicial}"),
    );
    let cw = 22.0;
    let fh = 24.0;
    let w_paso = 30.0;
    let w_estado = historia
        .iter()
        .map(|h| ancho_texto(&h.0, 10.5))
        .fold(30.0, f64::max)
        + 14.0;
    let x_cinta = w_paso + w_estado + 8.0;
    let mut y = y0 + 6.0;
    let mostrar = if blanco { 1 } else { historia.len() };
    for (k, (estado, cinta, cab)) in historia.iter().take(mostrar).enumerate() {
        c.t(
            w_paso - 6.0,
            y + 16.0,
            &k.to_string(),
            10.0,
            500,
            "textMuted",
            "end",
            MONO,
        );
        c.r(
            w_paso,
            y + 2.0,
            w_estado,
            fh - 4.0,
            "fillPointerBox",
            1.2,
            4.0,
        );
        c.t(
            w_paso + w_estado / 2.0,
            y + 16.0,
            estado,
            10.5,
            700,
            "textMain",
            "middle",
            MONO,
        );
        for p in lo..=hi {
            let x = x_cinta + (p - lo) as f64 * cw;
            let es_cab = p == *cab;
            c.r(
                x,
                y,
                cw,
                fh,
                if es_cab { "fillHighlight" } else { "fillCell" },
                if es_cab { 1.8 } else { 1.0 },
                0.0,
            );
            let ch = *cinta.get(&p).unwrap_or(&blanco_sim);
            c.t(
                x + cw / 2.0,
                y + 16.0,
                &ch.to_string(),
                11.0,
                if es_cab { 700 } else { 500 },
                if ch == blanco_sim {
                    "textMuted"
                } else {
                    "textMain"
                },
                "middle",
                MONO,
            );
        }
        y += fh + 4.0;
    }
    let ancho_cinta = x_cinta + (hi - lo + 1) as f64 * cw;
    y += 10.0;
    let txt = if blanco {
        "Resultado: ____ (completar la traza)".to_string()
    } else {
        format!(
            "Resultado: {final_} tras {} paso{}",
            historia.len() - 1,
            if historia.len() == 2 { "" } else { "s" }
        )
    };
    c.t(0.0, y, &txt, 10.5, 700, "textMain", "start", MONO);
    let mut ancho = ancho_cinta.max(ancho_texto(&txt, 10.5));
    y += 8.0;
    if bool_opt(b, "table", true) {
        // Tabla de transiciones: filas = estados, columnas = símbolos.
        let mut estados: Vec<String> = Vec::new();
        let mut simbolos: Vec<char> = Vec::new();
        for ((q, s), t) in &delta {
            for e in [q, &t.siguiente] {
                if !estados.contains(e) {
                    estados.push(e.clone());
                }
            }
            if !simbolos.contains(s) {
                simbolos.push(*s);
            }
        }
        simbolos.sort();
        let mut enc = vec!["δ".to_string()];
        enc.extend(simbolos.iter().map(|s| s.to_string()));
        let filas: Vec<Vec<Celda>> = estados
            .iter()
            .map(|q| {
                let marca = if aceptacion.contains(q) {
                    "*"
                } else if *q == inicial {
                    "→"
                } else {
                    ""
                };
                let mut f = vec![celda(format!("{marca}{q}")).fuerte(true)];
                for s in &simbolos {
                    f.push(celda(
                        delta
                            .get(&(q.clone(), *s))
                            .map(|t| {
                                format!(
                                    "{},{},{}",
                                    t.siguiente,
                                    t.escribe,
                                    match t.mueve {
                                        1 => "R",
                                        -1 => "L",
                                        _ => "S",
                                    }
                                )
                            })
                            .unwrap_or_else(|| "—".into()),
                    ));
                }
                f
            })
            .collect();
        y += 10.0;
        let (w, h) = tabla(
            c,
            0.0,
            y,
            &enc,
            &filas,
            &OpcTabla {
                fila_h: 21.0,
                ..Default::default()
            },
        );
        ancho = ancho.max(w);
        y += h + 14.0;
        c.t(
            0.0,
            y,
            "→ estado inicial · * estado de aceptación",
            9.5,
            400,
            "textMuted",
            "start",
            MONO,
        );
        y += 6.0;
    }
    Ok((ancho, y))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn programacion_dinamica() {
        let a: Vec<char> = "ABCBDAB".chars().collect();
        let b: Vec<char> = "BDCABA".chars().collect();
        assert_eq!(tabla_lcs(&a, &b)[7][6], 4);
        let k: Vec<char> = "kitten".chars().collect();
        let s: Vec<char> = "sitting".chars().collect();
        assert_eq!(tabla_edicion(&k, &s)[6][7], 3);
        assert_eq!(
            tabla_mochila(&[1, 3, 4, 5], &[1.0, 4.0, 5.0, 7.0], 7)[4][7],
            9.0
        );
        assert_eq!(tabla_monedas(&[1, 3, 4], 6)[6], Some(2));
    }

    #[test]
    fn dijkstra_clasico() {
        // A-B:4, A-C:2, C-B:1, B-D:5, C-D:8
        let ar = vec![
            (0, 1, 4.0),
            (1, 0, 4.0),
            (0, 2, 2.0),
            (2, 0, 2.0),
            (2, 1, 1.0),
            (1, 2, 1.0),
            (1, 3, 5.0),
            (3, 1, 5.0),
            (2, 3, 8.0),
            (3, 2, 8.0),
        ];
        let it = dijkstra(4, &ar, 0);
        let fin = it.last().unwrap();
        assert_eq!(fin.dist, vec![0.0, 3.0, 2.0, 8.0]);
        assert_eq!(fin.prev[1], Some(2));
    }
}
