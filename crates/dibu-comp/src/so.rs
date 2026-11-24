//! Sistemas operativos: `cpu_scheduling`, `page_table` y `page_replacement`.

use crate::comun::*;
use serde_json::Value;

// ---------------------------------------------------------------------------
// cpu_scheduling
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct Proceso {
    pub nombre: String,
    pub llegada: f64,
    pub rafaga: f64,
    pub prioridad: f64,
}

/// Tramo del diagrama de Gantt: proceso (o None = CPU ociosa), inicio, fin.
pub type Tramo = (Option<usize>, f64, f64);

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Algoritmo {
    Fcfs,
    Sjf,
    Srtf,
    Rr,
    Prioridad,
    PrioridadExp,
}

impl Algoritmo {
    pub fn nombre(self) -> &'static str {
        match self {
            Self::Fcfs => "FCFS",
            Self::Sjf => "SJF (no expropiativo)",
            Self::Srtf => "SRTF (SJF expropiativo)",
            Self::Rr => "Round Robin",
            Self::Prioridad => "Prioridad (no expropiativo)",
            Self::PrioridadExp => "Prioridad (expropiativo)",
        }
    }
    fn expropiativo(self) -> bool {
        matches!(self, Self::Srtf | Self::PrioridadExp)
    }
}

const EPS: f64 = 1e-9;

/// Simula la planificación. Empates: menor llegada y después orden de la lista.
pub fn planificar(procs: &[Proceso], alg: Algoritmo, quantum: f64) -> Vec<Tramo> {
    let n = procs.len();
    let mut restante: Vec<f64> = procs.iter().map(|p| p.rafaga).collect();
    let mut terminado = vec![false; n];
    let mut tramos: Vec<Tramo> = Vec::new();
    // El diagrama siempre empieza en t = 0 (si nadie llegó, la CPU está ociosa).
    let mut t = 0.0_f64;
    let agregar = |tramos: &mut Vec<Tramo>, p: Option<usize>, a: f64, b: f64| {
        if b - a < EPS {
            return;
        }
        if let Some(u) = tramos.last_mut() {
            if u.0 == p && (u.2 - a).abs() < EPS {
                u.2 = b;
                return;
            }
        }
        tramos.push((p, a, b));
    };

    if alg == Algoritmo::Rr {
        let q = quantum.max(EPS);
        let mut cola: std::collections::VecDeque<usize> = std::collections::VecDeque::new();
        let mut admitido = vec![false; n];
        let mut orden: Vec<usize> = (0..n).collect();
        orden.sort_by(|&a, &b| {
            procs[a]
                .llegada
                .partial_cmp(&procs[b].llegada)
                .unwrap()
                .then(a.cmp(&b))
        });
        let admitir =
            |hasta: f64, cola: &mut std::collections::VecDeque<usize>, admitido: &mut [bool]| {
                for &i in &orden {
                    if !admitido[i] && procs[i].llegada <= hasta + EPS {
                        admitido[i] = true;
                        cola.push_back(i);
                    }
                }
            };
        admitir(t, &mut cola, &mut admitido);
        let mut guardia = 0;
        while terminado.iter().any(|x| !x) && guardia < 100_000 {
            guardia += 1;
            let Some(i) = cola.pop_front() else {
                let prox = (0..n)
                    .filter(|&i| !admitido[i])
                    .map(|i| procs[i].llegada)
                    .fold(f64::INFINITY, f64::min);
                if !prox.is_finite() {
                    break;
                }
                agregar(&mut tramos, None, t, prox);
                t = prox;
                admitir(t, &mut cola, &mut admitido);
                continue;
            };
            let corre = restante[i].min(q);
            agregar(&mut tramos, Some(i), t, t + corre);
            t += corre;
            restante[i] -= corre;
            // Los que llegaron durante el quantum entran antes que el expropiado.
            admitir(t, &mut cola, &mut admitido);
            if restante[i] > EPS {
                cola.push_back(i);
            } else {
                terminado[i] = true;
            }
        }
        return tramos;
    }

    let mut guardia = 0;
    while terminado.iter().any(|x| !x) && guardia < 100_000 {
        guardia += 1;
        let listos: Vec<usize> = (0..n)
            .filter(|&i| !terminado[i] && procs[i].llegada <= t + EPS)
            .collect();
        if listos.is_empty() {
            let prox = (0..n)
                .filter(|&i| !terminado[i])
                .map(|i| procs[i].llegada)
                .fold(f64::INFINITY, f64::min);
            agregar(&mut tramos, None, t, prox);
            t = prox;
            continue;
        }
        let clave = |i: usize| -> (f64, f64, usize) {
            let k = match alg {
                Algoritmo::Fcfs => procs[i].llegada,
                Algoritmo::Sjf => procs[i].rafaga,
                Algoritmo::Srtf => restante[i],
                Algoritmo::Prioridad | Algoritmo::PrioridadExp => procs[i].prioridad,
                Algoritmo::Rr => unreachable!(),
            };
            (k, procs[i].llegada, i)
        };
        let i = *listos
            .iter()
            .min_by(|&&a, &&b| clave(a).partial_cmp(&clave(b)).unwrap())
            .unwrap();
        let mut fin = t + restante[i];
        if alg.expropiativo() {
            // Corre hasta terminar o hasta la próxima llegada (que podría expropiarlo).
            let prox = (0..n)
                .filter(|&j| !terminado[j] && procs[j].llegada > t + EPS)
                .map(|j| procs[j].llegada)
                .fold(f64::INFINITY, f64::min);
            fin = fin.min(prox);
        }
        agregar(&mut tramos, Some(i), t, fin);
        restante[i] -= fin - t;
        t = fin;
        if restante[i] <= EPS {
            terminado[i] = true;
        }
    }
    tramos
}

pub struct Metricas {
    pub fin: f64,
    pub retorno: f64,
    pub espera: f64,
    pub respuesta: f64,
}

pub fn metricas(procs: &[Proceso], tramos: &[Tramo]) -> Vec<Metricas> {
    procs
        .iter()
        .enumerate()
        .map(|(i, p)| {
            let propios: Vec<&Tramo> = tramos.iter().filter(|t| t.0 == Some(i)).collect();
            let fin = propios.iter().map(|t| t.2).fold(0.0, f64::max);
            let inicio = propios.iter().map(|t| t.1).fold(f64::INFINITY, f64::min);
            let retorno = fin - p.llegada;
            Metricas {
                fin,
                retorno,
                espera: retorno - p.rafaga,
                respuesta: inicio - p.llegada,
            }
        })
        .collect()
}

fn leer_procesos(b: &Bloque) -> Result<Vec<Proceso>, ErrorDiagrama> {
    let mut out = Vec::new();
    for (k, v) in lista(b, "processes", false)?.iter().enumerate() {
        let m = v.as_object().ok_or_else(|| {
            err(format!(
                "processes[{k}] tiene que ser {{name, arrival, burst}}"
            ))
        })?;
        let campo = |c: &str, def: Option<f64>| -> Result<f64, ErrorDiagrama> {
            match m.get(c).and_then(num) {
                Some(x) => Ok(x),
                None => def.ok_or_else(|| err(format!("processes[{k}] necesita '{c}'"))),
            }
        };
        let p = Proceso {
            nombre: m
                .get("name")
                .map(valor_str)
                .unwrap_or_else(|| format!("P{}", k + 1)),
            llegada: campo("arrival", Some(0.0))?,
            rafaga: campo("burst", None)?,
            prioridad: campo("priority", Some(0.0))?,
        };
        if p.rafaga <= 0.0 || p.llegada < 0.0 || p.rafaga > 10_000.0 || p.llegada > 10_000.0 {
            return Err(err(format!(
                "processes[{k}]: ráfaga > 0 y llegada ≥ 0 (hasta 10000)"
            )));
        }
        out.push(p);
    }
    if out.is_empty() || out.len() > 26 {
        return Err(err("'processes' necesita entre 1 y 26 procesos"));
    }
    Ok(out)
}

pub fn render_cpu_scheduling(c: &mut Lienzo, b: &Bloque) -> Result<Punto, ErrorDiagrama> {
    let procs = leer_procesos(b)?;
    let alg = match opcion(
        b,
        "algorithm",
        &[
            "fcfs",
            "sjf",
            "srtf",
            "rr",
            "priority",
            "priority_preemptive",
        ],
        "fcfs",
    )?
    .as_str()
    {
        "sjf" => Algoritmo::Sjf,
        "srtf" => Algoritmo::Srtf,
        "rr" => Algoritmo::Rr,
        "priority" => Algoritmo::Prioridad,
        "priority_preemptive" => Algoritmo::PrioridadExp,
        _ => Algoritmo::Fcfs,
    };
    let quantum = num_b(b, "quantum", 2.0);
    if alg == Algoritmo::Rr && !(quantum > 0.0 && quantum <= 1000.0) {
        return Err(err("'quantum' tiene que ser > 0"));
    }
    let blanco = bool_opt(b, "blank", false);
    let tramos = planificar(&procs, alg, quantum);
    if tramos.len() > 400 {
        return Err(err("la planificación tiene demasiados tramos (máximo 400)"));
    }
    let met = metricas(&procs, &tramos);
    let fin_total = tramos.last().map(|t| t.2).unwrap_or(0.0);
    let inicio = tramos.first().map(|t| t.1).unwrap_or(0.0);
    let mut extra = alg.nombre().to_string();
    if alg == Algoritmo::Rr {
        extra.push_str(&format!(", q = {}", fmt(quantum, 2)));
    }
    let y0 = encabezado(c, b, &extra);

    // Gantt.
    let escala = (560.0 / (fin_total - inicio).max(1.0)).clamp(8.0, 40.0);
    let x0 = 6.0;
    let gy = y0 + 10.0;
    let gh = 32.0;
    let px = |t: f64| x0 + (t - inicio) * escala;
    let ancho_gantt = px(fin_total) + 6.0;
    if blanco {
        c.r(x0, gy, ancho_gantt - x0 - 6.0, gh, "fillCell", 1.4, 0.0);
        let paso = if fin_total - inicio > 40.0 { 5.0 } else { 1.0 };
        let mut t = inicio;
        while t <= fin_total + EPS {
            c.linea(
                px(t),
                gy + gh,
                px(t),
                gy + gh + 4.0,
                "strokeMuted",
                1.0,
                None,
                "",
            );
            c.t(
                px(t),
                gy + gh + 15.0,
                &fmt(t, 1),
                9.0,
                400,
                "textMuted",
                "middle",
                MONO,
            );
            t += paso;
        }
    } else {
        let mut marcas: Vec<f64> = Vec::new();
        for &(p, a, z) in &tramos {
            let (x1, x2) = (px(a), px(z));
            match p {
                Some(i) => {
                    c.r(
                        x1,
                        gy,
                        x2 - x1,
                        gh,
                        if i % 2 == 0 {
                            "fillPointerBox"
                        } else {
                            "fillHeader"
                        },
                        1.4,
                        0.0,
                    );
                    let nombre = &procs[i].nombre;
                    if ancho_texto(nombre, 11.0) < x2 - x1 - 2.0 {
                        c.t(
                            (x1 + x2) / 2.0,
                            gy + gh / 2.0 + 4.0,
                            nombre,
                            11.0,
                            700,
                            "textMain",
                            "middle",
                            MONO,
                        );
                    }
                }
                None => {
                    c.add(format!(
                        r#"<rect x="{x1:.1}" y="{gy:.1}" width="{:.1}" height="{gh:.1}" fill="none" stroke="{}" stroke-width="1" stroke-dasharray="3 2"/>"#,
                        x2 - x1,
                        c.pal.color("strokeMuted")
                    ));
                    c.t(
                        (x1 + x2) / 2.0,
                        gy + gh / 2.0 + 4.0,
                        "ocioso",
                        9.0,
                        400,
                        "textMuted",
                        "middle",
                        SANS,
                    );
                }
            }
            for t in [a, z] {
                if !marcas.iter().any(|m| (m - t).abs() < EPS) {
                    marcas.push(t);
                }
            }
        }
        let mut ultimo_x = f64::NEG_INFINITY;
        for t in marcas {
            let x = px(t);
            if x - ultimo_x < 14.0 {
                continue;
            }
            ultimo_x = x;
            c.t(
                x,
                gy + gh + 14.0,
                &fmt(t, 2),
                9.5,
                500,
                "textMain",
                "middle",
                MONO,
            );
        }
    }
    // Llegadas.
    let ya = gy - 3.0;
    for p in &procs {
        let x = px(p.llegada);
        c.linea(x, ya - 9.0, x, ya, "strokeSub", 1.0, Some("sub"), "");
    }
    let mut y = gy + gh + 30.0;

    // Métricas.
    let con_prio = matches!(alg, Algoritmo::Prioridad | Algoritmo::PrioridadExp);
    let mut enc: Vec<String> = vec!["Proceso".into(), "Llegada".into(), "Ráfaga".into()];
    if con_prio {
        enc.push("Prioridad".into());
    }
    enc.extend(
        ["Fin", "Retorno", "Espera", "Respuesta"]
            .iter()
            .map(|s| s.to_string()),
    );
    let mut filas = Vec::new();
    for (p, m) in procs.iter().zip(&met) {
        let mut f = vec![
            celda(&p.nombre).fuerte(true),
            celda(fmt(p.llegada, 2)),
            celda(fmt(p.rafaga, 2)),
        ];
        if con_prio {
            f.push(celda(fmt(p.prioridad, 2)));
        }
        for v in [m.fin, m.retorno, m.espera, m.respuesta] {
            f.push(celda(fmt(v, 2)).en_blanco(blanco));
        }
        filas.push(f);
    }
    let n = procs.len() as f64;
    let prom = |g: fn(&Metricas) -> f64| met.iter().map(g).sum::<f64>() / n;
    let mut f = vec![celda("Promedio").fuerte(true), celda(""), celda("")];
    if con_prio {
        f.push(celda(""));
    }
    f.push(celda(""));
    for v in [
        prom(|m| m.retorno),
        prom(|m| m.espera),
        prom(|m| m.respuesta),
    ] {
        f.push(
            celda(fmt(v, 2))
                .en_blanco(blanco)
                .fuerte(true)
                .resaltada(!blanco),
        );
    }
    filas.push(f);
    let (w, h) = tabla(c, 0.0, y, &enc, &filas, &OpcTabla::default());
    y += h;
    Ok((w.max(ancho_gantt), y + 6.0))
}

// ---------------------------------------------------------------------------
// page_table
// ---------------------------------------------------------------------------

fn dir(v: &Value) -> Result<u64, ErrorDiagrama> {
    match v {
        Value::String(s) => {
            let t = s.trim().replace('_', "");
            if let Some(h) = t.strip_prefix("0x").or_else(|| t.strip_prefix("0X")) {
                u64::from_str_radix(h, 16).map_err(|_| err(format!("dirección inválida {s:?}")))
            } else {
                t.parse()
                    .map_err(|_| err(format!("dirección inválida {s:?}")))
            }
        }
        _ => num(v)
            .filter(|x| x.fract() == 0.0 && *x >= 0.0)
            .map(|x| x as u64)
            .ok_or_else(|| err("las direcciones son enteros ≥ 0")),
    }
}

pub fn render_page_table(c: &mut Lienzo, b: &Bloque) -> Result<Punto, ErrorDiagrama> {
    let pag = num_req(b, "pageSize", 1.0, 1e12)? as u64;
    if !pag.is_power_of_two() {
        return Err(err("'pageSize' tiene que ser potencia de 2"));
    }
    let tabla_v = lista(b, "table", false)?;
    let marcos: Vec<Option<u64>> = tabla_v
        .iter()
        .map(|v| match v {
            Value::Null => Ok(None),
            Value::String(s) if matches!(s.trim(), "" | "-" | "—" | "x" | "X" | "invalid") => {
                Ok(None)
            }
            _ => num(v)
                .filter(|x| *x >= 0.0 && x.fract() == 0.0)
                .map(|x| Some(x as u64))
                .ok_or_else(|| {
                    err("'table' contiene números de marco o null (página no presente)")
                }),
        })
        .collect::<Result<_, _>>()?;
    let direcciones: Vec<u64> = lista(b, "addresses", false)?
        .iter()
        .map(dir)
        .collect::<Result<_, _>>()?;
    let hexa = opcion(b, "base", &["dec", "hex"], "dec")? == "hex";
    let blanco = bool_opt(b, "blank", false);
    let bits_off = pag.trailing_zeros();
    let fmt_d = |x: u64| {
        if hexa {
            format!("0x{x:X}")
        } else {
            x.to_string()
        }
    };
    let extra = format!("páginas de {pag} B ({bits_off} bits de offset)");
    let y0 = encabezado(c, b, &extra);

    // Tabla de páginas.
    let filas_pt: Vec<Vec<Celda>> = marcos
        .iter()
        .enumerate()
        .map(|(p, m)| {
            vec![
                celda(p.to_string()).atenuada(true),
                celda(m.map(|x| x.to_string()).unwrap_or_else(|| "—".into())),
                celda(if m.is_some() { "1" } else { "0" }),
            ]
        })
        .collect();
    let enc_pt: Vec<String> = ["Página", "Marco", "V"]
        .iter()
        .map(|s| s.to_string())
        .collect();
    c.t(
        0.0,
        y0 + 12.0,
        "Tabla de páginas",
        11.0,
        700,
        "textSub",
        "start",
        SANS,
    );
    let (w1, h1) = tabla(
        c,
        0.0,
        y0 + 18.0,
        &enc_pt,
        &filas_pt,
        &OpcTabla {
            fila_h: 21.0,
            ..Default::default()
        },
    );

    // Traducciones.
    let x2 = w1 + 30.0;
    let filas: Vec<Vec<Celda>> = direcciones
        .iter()
        .map(|&a| {
            let p = a >> bits_off;
            let off = a & (pag - 1);
            let m = marcos.get(p as usize).copied().flatten();
            let fisica = m.map(|mm| fmt_d((mm << bits_off) | off));
            vec![
                celda(fmt_d(a)).fuerte(true),
                celda(p.to_string()).en_blanco(blanco),
                celda(fmt_d(off)).en_blanco(blanco),
                celda(m.map(|x| x.to_string()).unwrap_or_else(|| "—".into())).en_blanco(blanco),
                celda(fisica.unwrap_or_else(|| {
                    if (p as usize) < marcos.len() {
                        "fallo de página".into()
                    } else {
                        "fuera de rango".into()
                    }
                }))
                .en_blanco(blanco)
                .resaltada(!blanco && m.is_none()),
            ]
        })
        .collect();
    let enc: Vec<String> = [
        "Dirección lógica",
        "Página",
        "Offset",
        "Marco",
        "Dirección física",
    ]
    .iter()
    .map(|s| s.to_string())
    .collect();
    c.t(
        x2,
        y0 + 12.0,
        "Traducción",
        11.0,
        700,
        "textSub",
        "start",
        SANS,
    );
    let (w2, h2) = tabla(c, x2, y0 + 18.0, &enc, &filas, &OpcTabla::default());
    let mut alto = y0 + 18.0 + h1.max(h2);
    if bool_opt(b, "formula", true) {
        alto += 18.0;
        c.t(0.0, alto, &format!("página = dirección div {pag} · offset = dirección mod {pag} · física = marco × {pag} + offset"), 10.0, 500, "textMuted", "start", MONO);
        c.ancho_min = c.ancho_min.max(ancho_texto("página = dirección div 0000 · offset = dirección mod 0000 · física = marco × 0000 + offset", 10.0));
    }
    Ok((x2 + w2, alto + 6.0))
}

// ---------------------------------------------------------------------------
// page_replacement
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct PasoReemplazo {
    pub marcos: Vec<Option<i64>>,
    pub fallo: bool,
    pub cambiado: Option<usize>,
    pub bits: Vec<bool>,
    pub puntero: usize,
}

pub fn reemplazo(refs: &[i64], n: usize, alg: &str) -> Vec<PasoReemplazo> {
    let mut marcos: Vec<Option<i64>> = vec![None; n];
    let mut carga = vec![0usize; n];
    let mut uso = vec![0usize; n];
    let mut bits = vec![false; n];
    let mut puntero = 0usize;
    let mut out = Vec::new();
    for (t, &r) in refs.iter().enumerate() {
        if let Some(k) = marcos.iter().position(|m| *m == Some(r)) {
            uso[k] = t;
            bits[k] = true;
            out.push(PasoReemplazo {
                marcos: marcos.clone(),
                fallo: false,
                cambiado: None,
                bits: bits.clone(),
                puntero,
            });
            continue;
        }
        let k = if let Some(libre) = marcos.iter().position(Option::is_none) {
            if alg == "clock" {
                puntero = (libre + 1) % n;
            }
            libre
        } else {
            match alg {
                "lru" => (0..n).min_by_key(|&k| uso[k]).unwrap(),
                "opt" => (0..n)
                    .max_by_key(|&k| {
                        let pag = marcos[k].unwrap();
                        // Próximo uso (o infinito); empate: el que se cargó antes.
                        (
                            refs[t + 1..]
                                .iter()
                                .position(|&x| x == pag)
                                .unwrap_or(usize::MAX),
                            usize::MAX - carga[k],
                        )
                    })
                    .unwrap(),
                "clock" => loop {
                    if bits[puntero] {
                        bits[puntero] = false;
                        puntero = (puntero + 1) % n;
                    } else {
                        let k = puntero;
                        puntero = (puntero + 1) % n;
                        break k;
                    }
                },
                _ => (0..n).min_by_key(|&k| carga[k]).unwrap(),
            }
        };
        marcos[k] = Some(r);
        carga[k] = t;
        uso[k] = t;
        bits[k] = alg != "clock" || bits[k];
        if alg == "clock" {
            bits[k] = true;
        }
        out.push(PasoReemplazo {
            marcos: marcos.clone(),
            fallo: true,
            cambiado: Some(k),
            bits: bits.clone(),
            puntero,
        });
    }
    out
}

pub fn render_page_replacement(c: &mut Lienzo, b: &Bloque) -> Result<Punto, ErrorDiagrama> {
    let n = entero_b(b, "frames", 3, 1, 16)? as usize;
    let refs: Vec<i64> = lista(b, "references", false)?
        .iter()
        .map(|v| {
            num(v)
                .filter(|x| x.fract() == 0.0)
                .map(|x| x as i64)
                .ok_or_else(|| err("'references' contiene números de página"))
        })
        .collect::<Result<_, _>>()?;
    if refs.is_empty() || refs.len() > 60 {
        return Err(err("'references' necesita entre 1 y 60 referencias"));
    }
    let algs: Vec<String> = match b.get("algorithms").or_else(|| b.get("algorithm")) {
        Some(Value::Array(a)) => a.iter().map(|v| valor_str(v).to_lowercase()).collect(),
        Some(v) => vec![valor_str(v).to_lowercase()],
        None => vec!["fifo".into()],
    };
    for a in &algs {
        if !matches!(a.as_str(), "fifo" | "lru" | "opt" | "clock") {
            return Err(err(format!(
                "algoritmo {a:?} desconocido (fifo, lru, opt, clock)"
            )));
        }
    }
    let blanco = bool_opt(b, "blank", false);
    let y0 = encabezado(c, b, &format!("{n} marcos, {} referencias", refs.len()));
    let cw = 26.0;
    let fh = 22.0;
    let x0 = 64.0;
    let mut y = y0 + 4.0;
    let mut ancho: f64 = x0 + refs.len() as f64 * cw;
    for alg in &algs {
        let pasos = reemplazo(&refs, n, alg);
        let fallos = pasos.iter().filter(|p| p.fallo).count();
        let nombre = match alg.as_str() {
            "lru" => "LRU",
            "opt" => "Óptimo (OPT)",
            "clock" => "Reloj (segunda oportunidad)",
            _ => "FIFO",
        };
        c.t(0.0, y + 13.0, nombre, 11.0, 700, "textSub", "start", SANS);
        let resumen = if blanco {
            "fallos: ____".to_string()
        } else {
            format!(
                "fallos: {fallos} / {} · tasa de fallos {} %",
                refs.len(),
                fmt(100.0 * fallos as f64 / refs.len() as f64, 1)
            )
        };
        let xr = ancho_texto(nombre, 11.0) + 16.0;
        c.t(
            xr,
            y + 13.0,
            &resumen,
            10.0,
            500,
            "textMuted",
            "start",
            MONO,
        );
        c.ancho_min = c.ancho_min.max(xr + ancho_texto(&resumen, 10.0));
        y += 20.0;
        // Fila de referencias.
        c.t(
            x0 - 8.0,
            y + fh / 2.0 + 4.0,
            "ref",
            10.0,
            700,
            "textMuted",
            "end",
            MONO,
        );
        for (j, r) in refs.iter().enumerate() {
            let x = x0 + j as f64 * cw;
            c.r(x, y, cw, fh, "fillHeader", 1.2, 0.0);
            c.t(
                x + cw / 2.0,
                y + fh / 2.0 + 4.0,
                &r.to_string(),
                10.5,
                700,
                "textMain",
                "middle",
                MONO,
            );
        }
        y += fh + 3.0;
        for k in 0..n {
            c.t(
                x0 - 8.0,
                y + fh / 2.0 + 4.0,
                &format!("M{k}"),
                10.0,
                600,
                "textMuted",
                "end",
                MONO,
            );
            for (j, p) in pasos.iter().enumerate() {
                let x = x0 + j as f64 * cw;
                let nuevo = p.cambiado == Some(k);
                c.r(
                    x,
                    y,
                    cw,
                    fh,
                    if nuevo && !blanco {
                        "fillHighlight"
                    } else {
                        "fillCell"
                    },
                    1.0,
                    0.0,
                );
                if !blanco {
                    if let Some(v) = p.marcos[k] {
                        let txt = if alg == "clock" && p.bits[k] {
                            format!("{v}*")
                        } else {
                            v.to_string()
                        };
                        c.t(
                            x + cw / 2.0,
                            y + fh / 2.0 + 4.0,
                            &txt,
                            10.5,
                            if nuevo { 700 } else { 400 },
                            if nuevo { "textMain" } else { "textSub" },
                            "middle",
                            MONO,
                        );
                    }
                }
            }
            y += fh;
        }
        // Fila de fallos.
        c.t(
            x0 - 8.0,
            y + 15.0,
            "fallo",
            10.0,
            600,
            "textMuted",
            "end",
            MONO,
        );
        for (j, p) in pasos.iter().enumerate() {
            let x = x0 + j as f64 * cw;
            if blanco {
                c.vacio(x + 5.0, x + cw - 5.0, y + 17.0);
            } else if p.fallo {
                c.t(
                    x + cw / 2.0,
                    y + 15.0,
                    "F",
                    10.5,
                    700,
                    "accentPrimary",
                    "middle",
                    MONO,
                );
            }
        }
        y += 30.0;
        ancho = ancho.max(x0 + refs.len() as f64 * cw);
    }
    if algs.iter().any(|a| a == "clock") && !blanco {
        c.t(
            0.0,
            y,
            "* bit de referencia en 1",
            9.5,
            400,
            "textMuted",
            "start",
            MONO,
        );
        y += 10.0;
    }
    Ok((ancho, y - 8.0))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn p(n: &str, a: f64, r: f64) -> Proceso {
        Proceso {
            nombre: n.into(),
            llegada: a,
            rafaga: r,
            prioridad: 0.0,
        }
    }

    #[test]
    fn planificacion_clasica() {
        let ps = vec![
            p("P1", 0.0, 8.0),
            p("P2", 1.0, 4.0),
            p("P3", 2.0, 9.0),
            p("P4", 3.0, 5.0),
        ];
        // SRTF (Silberschatz): P1 0-1, P2 1-5, P4 5-10, P1 10-17, P3 17-26; espera prom. 6,5.
        let t = planificar(&ps, Algoritmo::Srtf, 0.0);
        let orden: Vec<(usize, f64, f64)> = t.iter().map(|x| (x.0.unwrap(), x.1, x.2)).collect();
        assert_eq!(
            orden,
            vec![
                (0, 0.0, 1.0),
                (1, 1.0, 5.0),
                (3, 5.0, 10.0),
                (0, 10.0, 17.0),
                (2, 17.0, 26.0)
            ]
        );
        let m = metricas(&ps, &t);
        assert!((m.iter().map(|x| x.espera).sum::<f64>() / 4.0 - 6.5).abs() < 1e-9);
        // RR q=4 sobre P1=24, P2=3, P3=3 (todas en 0): espera prom. 17/3.
        let ps = vec![p("P1", 0.0, 24.0), p("P2", 0.0, 3.0), p("P3", 0.0, 3.0)];
        let t = planificar(&ps, Algoritmo::Rr, 4.0);
        let m = metricas(&ps, &t);
        assert!((m.iter().map(|x| x.espera).sum::<f64>() / 3.0 - 17.0 / 3.0).abs() < 1e-9);
        // CPU ociosa.
        let t = planificar(&[p("A", 2.0, 1.0)], Algoritmo::Fcfs, 0.0);
        assert_eq!(t, vec![(None, 0.0, 2.0), (Some(0), 2.0, 3.0)]);
    }

    #[test]
    fn reemplazo_clasico() {
        // Cadena de Silberschatz con 3 marcos: FIFO 15, LRU 12, OPT 9 fallos.
        let refs = [7, 0, 1, 2, 0, 3, 0, 4, 2, 3, 0, 3, 2, 1, 2, 0, 1, 7, 0, 1];
        let f = |a: &str| reemplazo(&refs, 3, a).iter().filter(|p| p.fallo).count();
        assert_eq!(f("fifo"), 15);
        assert_eq!(f("lru"), 12);
        assert_eq!(f("opt"), 9);
        // Anomalía de Belady: FIFO con 3 y 4 marcos.
        let refs = [1, 2, 3, 4, 1, 2, 5, 1, 2, 3, 4, 5];
        assert_eq!(
            reemplazo(&refs, 3, "fifo")
                .iter()
                .filter(|p| p.fallo)
                .count(),
            9
        );
        assert_eq!(
            reemplazo(&refs, 4, "fifo")
                .iter()
                .filter(|p| p.fallo)
                .count(),
            10
        );
    }
}
