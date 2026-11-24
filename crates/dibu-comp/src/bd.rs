//! Bases de datos: `transaction_schedule` (planificación de transacciones, conflictos,
//! grafo de precedencia y serializabilidad por conflictos).

use crate::comun::*;
use serde_json::Value;

#[derive(Debug, Clone, PartialEq)]
pub struct Op {
    pub tx: String,
    /// R, W, C (commit) o A (abort).
    pub tipo: char,
    pub item: Option<String>,
}

/// Parsea `R1(A)`, `W2(B)`, `C1`, `A3`, `r1[x]`, `T1: R(A)`, `read(T1, A)`.
pub fn parsear_op(s: &str) -> Result<Op, ErrorDiagrama> {
    let t = s.trim();
    let mal = || {
        err(format!(
            "operación {s:?} no reconocida (usar R1(A), W2(B), C1)"
        ))
    };
    // Forma "T1: R(A)".
    if let Some((tx, resto)) = t.split_once(':') {
        let tx = tx.trim().trim_start_matches(['T', 't']).to_string();
        let r = resto.trim();
        let tipo = r.chars().next().ok_or_else(mal)?.to_ascii_uppercase();
        let item = r
            .find(['(', '['])
            .map(|i| r[i + 1..].trim_end_matches([')', ']']).trim().to_string());
        return Ok(Op { tx, tipo, item });
    }
    let mut cs = t.chars();
    let tipo = cs.next().ok_or_else(mal)?.to_ascii_uppercase();
    if !matches!(tipo, 'R' | 'W' | 'C' | 'A') {
        return Err(mal());
    }
    let resto: String = cs.collect();
    let (tx, item) = match resto.find(['(', '[']) {
        Some(i) => (
            resto[..i].trim().to_string(),
            Some(
                resto[i + 1..]
                    .trim_end_matches([')', ']'])
                    .trim()
                    .to_string(),
            ),
        ),
        None => (resto.trim().to_string(), None),
    };
    let tx = tx.trim_start_matches(['T', 't']).to_string();
    if tx.is_empty() || ((tipo == 'R' || tipo == 'W') && item.as_deref().is_none_or(str::is_empty))
    {
        return Err(mal());
    }
    Ok(Op { tx, tipo, item })
}

/// Pares en conflicto (i, j) con i < j.
pub fn conflictos(ops: &[Op]) -> Vec<(usize, usize)> {
    let mut out = Vec::new();
    for i in 0..ops.len() {
        for j in i + 1..ops.len() {
            let (a, b) = (&ops[i], &ops[j]);
            if a.tx != b.tx
                && a.item.is_some()
                && a.item == b.item
                && (a.tipo == 'W' || b.tipo == 'W')
                && matches!(a.tipo, 'R' | 'W')
                && matches!(b.tipo, 'R' | 'W')
            {
                out.push((i, j));
            }
        }
    }
    out
}

/// Orden topológico (serializable) o un ciclo.
pub fn orden_serial(txs: &[String], aristas: &[(usize, usize)]) -> Result<Vec<usize>, Vec<usize>> {
    let n = txs.len();
    let mut grado = vec![0; n];
    for &(_, b) in aristas {
        grado[b] += 1;
    }
    let mut orden = Vec::new();
    let mut usados = vec![false; n];
    for _ in 0..n {
        match (0..n).find(|&k| !usados[k] && grado[k] == 0) {
            Some(k) => {
                usados[k] = true;
                orden.push(k);
                for &(a, b) in aristas {
                    if a == k {
                        grado[b] -= 1;
                    }
                }
            }
            None => break,
        }
    }
    if orden.len() == n {
        return Ok(orden);
    }
    // Cada nodo pendiente tiene una arista entrante desde otro pendiente: se retrocede por
    // predecesores hasta repetir un nodo, y ese tramo invertido es un ciclo.
    let inicio = (0..n).find(|&k| !usados[k]).expect("quedan nodos");
    let mut camino = vec![inicio];
    loop {
        let u = *camino.last().unwrap();
        let Some(v) = aristas
            .iter()
            .find(|&&(a, b)| b == u && !usados[a])
            .map(|&(a, _)| a)
        else {
            return Err(camino);
        };
        if let Some(p) = camino.iter().position(|&x| x == v) {
            let mut ciclo: Vec<usize> = camino[p..].to_vec();
            ciclo.reverse();
            ciclo.insert(0, v);
            ciclo.dedup();
            if ciclo.first() != ciclo.last() {
                ciclo.push(ciclo[0]);
            }
            return Err(ciclo);
        }
        camino.push(v);
    }
}

pub fn render_transaction_schedule(c: &mut Lienzo, b: &Bloque) -> Result<Punto, ErrorDiagrama> {
    let ops: Vec<Op> = match b.get("schedule") {
        Some(Value::String(s)) => s
            .split([',', ';'])
            .filter(|x| !x.trim().is_empty())
            .map(parsear_op)
            .collect::<Result<_, _>>()?,
        _ => lista(b, "schedule", false)?
            .iter()
            .map(|v| parsear_op(&valor_str(v)))
            .collect::<Result<_, _>>()?,
    };
    if ops.is_empty() || ops.len() > 80 {
        return Err(err("'schedule' necesita entre 1 y 80 operaciones"));
    }
    let mut txs: Vec<String> = Vec::new();
    for o in &ops {
        if !txs.contains(&o.tx) {
            txs.push(o.tx.clone());
        }
    }
    txs.sort_by(|a, b| {
        a.parse::<i64>()
            .ok()
            .cmp(&b.parse::<i64>().ok())
            .then(a.cmp(b))
    });
    if txs.len() > 8 {
        return Err(err("como máximo 8 transacciones"));
    }
    let blanco = bool_opt(b, "blank", false);
    let idx = |tx: &str| txs.iter().position(|t| t == tx).unwrap();
    let pares = conflictos(&ops);
    let mut aristas: Vec<(usize, usize, Vec<String>)> = Vec::new();
    for &(i, j) in &pares {
        let (a, bb) = (idx(&ops[i].tx), idx(&ops[j].tx));
        let item = ops[i].item.clone().unwrap_or_default();
        match aristas.iter_mut().find(|(x, y, _)| *x == a && *y == bb) {
            Some(e) => {
                if !e.2.contains(&item) {
                    e.2.push(item)
                }
            }
            None => aristas.push((a, bb, vec![item])),
        }
    }
    let y0 = encabezado(c, b, "");

    // Tabla temporal: una columna por transacción.
    let enc: Vec<String> = std::iter::once("t".to_string())
        .chain(txs.iter().map(|t| format!("T{t}")))
        .collect();
    let en_conflicto: Vec<bool> = (0..ops.len())
        .map(|k| pares.iter().any(|&(i, j)| i == k || j == k))
        .collect();
    let filas: Vec<Vec<Celda>> =
        ops.iter()
            .enumerate()
            .map(|(k, o)| {
                let mut f = vec![celda((k + 1).to_string()).atenuada(true)];
                for t in &txs {
                    if *t == o.tx {
                        let txt = match (o.tipo, &o.item) {
                            ('R', Some(x)) => format!("read({x})"),
                            ('W', Some(x)) => format!("write({x})"),
                            ('C', _) => "commit".into(),
                            ('A', _) => "abort".into(),
                            _ => String::new(),
                        };
                        f.push(celda(txt).fuerte(true).resaltada(
                            !blanco && en_conflicto[k] && bool_opt(b, "highlight", true),
                        ));
                    } else {
                        f.push(celda(""));
                    }
                }
                f
            })
            .collect();
    // Celdas vacías sin raya: se usan textos vacíos y se evita el modo blanco.
    let (w1, h1) = tabla(
        c,
        0.0,
        y0 + 4.0,
        &enc,
        &filas,
        &OpcTabla {
            fila_h: 22.0,
            min_w: 70.0,
            ..Default::default()
        },
    );

    // Grafo de precedencia (disposición circular).
    let gx = w1 + 50.0;
    let radio = if txs.len() <= 2 {
        50.0
    } else {
        40.0 + txs.len() as f64 * 10.0
    };
    let cx = gx + radio + 26.0;
    let cy = y0 + 30.0 + radio + 10.0;
    c.t(
        gx,
        y0 + 14.0,
        "Grafo de precedencia",
        11.0,
        700,
        "textSub",
        "start",
        SANS,
    );
    let pos: Vec<(f64, f64)> = (0..txs.len())
        .map(|k| {
            let ang =
                -std::f64::consts::FRAC_PI_2 + k as f64 * std::f64::consts::TAU / txs.len() as f64;
            if txs.len() == 1 {
                (cx, cy)
            } else if txs.len() == 2 {
                (cx - radio + 2.0 * radio * k as f64, cy)
            } else {
                (cx + radio * ang.cos(), cy + radio * ang.sin())
            }
        })
        .collect();
    let rn = 18.0;
    if !blanco {
        for (a, bb, items) in &aristas {
            let (p, q) = (pos[*a], pos[*bb]);
            let doble = aristas.iter().any(|(x, y, _)| x == bb && y == a);
            let (dx, dy) = (q.0 - p.0, q.1 - p.1);
            let d = dx.hypot(dy).max(1.0);
            let (ux, uy) = (dx / d, dy / d);
            // Curvar si hay arista en ambos sentidos.
            let curva = if doble { 22.0 } else { 0.0 };
            let (nx, ny) = (-uy * curva, ux * curva);
            let (x1, y1) = (p.0 + ux * rn, p.1 + uy * rn);
            let (x2, y2) = (q.0 - ux * (rn + 2.0), q.1 - uy * (rn + 2.0));
            let (mx, my) = ((x1 + x2) / 2.0 + nx, (y1 + y2) / 2.0 + ny);
            c.add(format!(
                r#"<path d="M {x1:.1} {y1:.1} Q {mx:.1} {my:.1} {x2:.1} {y2:.1}" fill="none" stroke="{}" stroke-width="1.6" marker-end="url(#flecha-main-{})"/>"#,
                c.pal.color("accentPrimary"),
                c.sufijo
            ));
            c.t(
                mx + nx * 0.2,
                my + ny * 0.2 - 4.0,
                &items.join(","),
                10.0,
                600,
                "textSub",
                "middle",
                MONO,
            );
        }
    }
    for (k, t) in txs.iter().enumerate() {
        circulo(
            c,
            pos[k].0,
            pos[k].1,
            rn,
            "fillPointerBox",
            "strokeMain",
            1.6,
        );
        c.t(
            pos[k].0,
            pos[k].1 + 4.5,
            &format!("T{t}"),
            11.0,
            700,
            "textMain",
            "middle",
            MONO,
        );
    }
    let alto_grafo = cy + radio + 30.0;
    let ancho = cx + radio + 40.0;
    let mut y = (y0 + 4.0 + h1).max(alto_grafo) + 6.0;

    // Veredicto.
    let simples: Vec<(usize, usize)> = aristas.iter().map(|(a, b, _)| (*a, *b)).collect();
    let veredicto = match orden_serial(&txs, &simples) {
        Ok(orden) => format!(
            "Serializable por conflictos: sí — equivale a {}",
            orden
                .iter()
                .map(|&k| format!("T{}", txs[k]))
                .collect::<Vec<_>>()
                .join(" → ")
        ),
        Err(ciclo) => format!(
            "Serializable por conflictos: no — ciclo {}",
            ciclo
                .iter()
                .map(|&k| format!("T{}", txs[k]))
                .collect::<Vec<_>>()
                .join(" → ")
        ),
    };
    if bool_opt(b, "conflicts", true) && !blanco && !pares.is_empty() {
        let lista_c: Vec<String> = pares
            .iter()
            .map(|&(i, j)| {
                let f =
                    |o: &Op| format!("{}{}({})", o.tipo, o.tx, o.item.clone().unwrap_or_default());
                format!("{} < {}", f(&ops[i]), f(&ops[j]))
            })
            .collect();
        let mut linea = String::from("Conflictos: ");
        for (k, p) in lista_c.iter().enumerate() {
            if ancho_texto(&linea, 10.0) + ancho_texto(p, 10.0) > ancho.max(w1) && linea.len() > 12
            {
                c.t(0.0, y + 12.0, &linea, 10.0, 400, "textSub", "start", MONO);
                y += 15.0;
                linea = "            ".into();
            }
            linea.push_str(p);
            if k + 1 < lista_c.len() {
                linea.push_str(", ");
            }
        }
        c.t(0.0, y + 12.0, &linea, 10.0, 400, "textSub", "start", MONO);
        y += 18.0;
    }
    y += 12.0;
    if blanco {
        c.t(
            0.0,
            y,
            "Serializable por conflictos: ____   Orden equivalente: ____",
            10.5,
            700,
            "textMain",
            "start",
            MONO,
        );
    } else {
        c.t(0.0, y, &veredicto, 10.5, 700, "textMain", "start", MONO);
        c.ancho_min = c.ancho_min.max(ancho_texto(&veredicto, 10.5));
    }
    Ok((ancho, y + 8.0))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn serializabilidad() {
        let ops: Vec<Op> = ["R1(A)", "W2(A)", "R2(B)", "W1(B)", "C1", "C2"]
            .iter()
            .map(|s| parsear_op(s).unwrap())
            .collect();
        let pares = conflictos(&ops);
        assert_eq!(pares, vec![(0, 1), (2, 3)]);
        let txs = vec!["1".to_string(), "2".to_string()];
        assert_eq!(orden_serial(&txs, &[(0, 1), (1, 0)]), Err(vec![0, 1, 0]));
        assert_eq!(orden_serial(&txs, &[(1, 0)]), Ok(vec![1, 0]));
        assert_eq!(
            parsear_op("T3: W(X)").unwrap(),
            Op {
                tx: "3".into(),
                tipo: 'W',
                item: Some("X".into())
            }
        );
        assert!(parsear_op("Q1(A)").is_err());
        // Ciclo cuyo primer nodo pendiente no tiene salida hacia el ciclo directamente.
        let t3: Vec<String> = ["1", "2", "3"].iter().map(|s| s.to_string()).collect();
        let ciclo = orden_serial(&t3, &[(1, 2), (2, 1), (2, 0)]).unwrap_err();
        assert_eq!(ciclo.first(), ciclo.last());
        assert!(ciclo.len() >= 3);
    }
}
