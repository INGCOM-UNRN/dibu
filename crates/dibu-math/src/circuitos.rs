//! Circuitos eléctricos: `circuit` (redes serie/paralelo con fuente, impedancia equivalente)
//! y `phasor` (diagrama fasorial con suma).

use crate::expr::valor_si;
use dibu_comp::comun::*;
use serde_json::Value;
use std::f64::consts::PI;

// ---------------------------------------------------------------------------
// Números complejos mínimos
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Cx(pub f64, pub f64);

impl Cx {
    pub fn suma(self, o: Cx) -> Cx {
        Cx(self.0 + o.0, self.1 + o.1)
    }
    pub fn inv(self) -> Cx {
        let d = self.0 * self.0 + self.1 * self.1;
        Cx(self.0 / d, -self.1 / d)
    }
    pub fn por(self, o: Cx) -> Cx {
        Cx(self.0 * o.0 - self.1 * o.1, self.0 * o.1 + self.1 * o.0)
    }
    pub fn modulo(self) -> f64 {
        self.0.hypot(self.1)
    }
    pub fn angulo_deg(self) -> f64 {
        self.1.atan2(self.0).to_degrees()
    }
}

pub fn fmt_cx(z: Cx, unidad: &str) -> String {
    let re = if z.0.abs() < 1e-12 { 0.0 } else { z.0 };
    let im = if z.1.abs() < 1e-12 { 0.0 } else { z.1 };
    let rect = if im == 0.0 {
        fmt_si(re, unidad)
    } else if re == 0.0 {
        format!(
            "{}j{}",
            if im < 0.0 { "−" } else { "" },
            fmt_si(im.abs(), unidad)
        )
    } else {
        format!(
            "{} {} j{}",
            fmt_si(re, ""),
            if im < 0.0 { "−" } else { "+" },
            fmt_si(im.abs(), unidad)
        )
    };
    if im == 0.0 {
        rect
    } else {
        format!(
            "{rect} = {}∠{}°",
            fmt_si(z.modulo(), unidad),
            fmt(z.angulo_deg(), 2)
        )
    }
}

/// 4700 → "4,7 k", 0.0001 → "100 µ".
pub fn fmt_si(x: f64, unidad: &str) -> String {
    if x == 0.0 || !x.is_finite() {
        return format!(
            "{}{}{unidad}",
            fmt(x, 3),
            if unidad.is_empty() { "" } else { " " }
        );
    }
    let pref = [
        (1e9, "G"),
        (1e6, "M"),
        (1e3, "k"),
        (1.0, ""),
        (1e-3, "m"),
        (1e-6, "µ"),
        (1e-9, "n"),
        (1e-12, "p"),
    ];
    let a = x.abs();
    let (f, p) = pref
        .iter()
        .find(|(f, _)| a >= *f * 0.9995)
        .copied()
        .unwrap_or((1e-12, "p"));
    let sep = if unidad.is_empty() && p.is_empty() {
        ""
    } else {
        " "
    };
    format!("{}{sep}{p}{unidad}", fmt(x / f, 3))
}

// ---------------------------------------------------------------------------
// circuit
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub enum Red {
    Comp {
        tipo: char,
        nombre: String,
        valor: Option<f64>,
        texto: String,
    },
    Serie(Vec<Red>),
    Paralelo(Vec<Red>),
}

fn leer_comp(v: &Value, prof: usize) -> Result<Red, ErrorDiagrama> {
    if prof > 12 {
        return Err(err("la red está anidada demasiado"));
    }
    match v {
        Value::String(s) => {
            // "R1", "R1 = 10k", "C2: 100uF"
            let (nombre, valor) = match s.split_once(['=', ':']) {
                Some((n, v)) => (n.trim().to_string(), Some(v.trim().to_string())),
                None => (s.trim().to_string(), None),
            };
            let tipo = nombre.chars().next().map(|c| c.to_ascii_uppercase()).unwrap_or('Z');
            let tipo = if "RLCVIZ".contains(tipo) { tipo } else { 'Z' };
            Ok(Red::Comp { tipo, nombre, valor: valor.as_deref().and_then(valor_si), texto: valor.unwrap_or_default() })
        }
        Value::Object(m) => {
            if let Some(Value::Array(a)) = m.get("series") {
                return Ok(Red::Serie(a.iter().map(|x| leer_comp(x, prof + 1)).collect::<Result<_, _>>()?));
            }
            if let Some(Value::Array(a)) = m.get("parallel") {
                if a.len() < 2 {
                    return Err(err("'parallel' necesita al menos 2 ramas"));
                }
                return Ok(Red::Paralelo(a.iter().map(|x| leer_comp(x, prof + 1)).collect::<Result<_, _>>()?));
            }
            let nombre = m.get("name").map(valor_str).unwrap_or_default();
            let tipo = m
                .get("type")
                .map(valor_str)
                .or_else(|| nombre.chars().next().map(String::from))
                .unwrap_or_else(|| "Z".into())
                .to_uppercase()
                .chars()
                .next()
                .unwrap_or('Z');
            if !"RLCVIZ".contains(tipo) {
                return Err(err(format!("tipo de componente {tipo:?} desconocido (R, L, C, V, I, Z)")));
            }
            let texto = m.get("value").map(valor_str).unwrap_or_default();
            let valor = m.get("value").and_then(|v| num(v).or_else(|| valor_si(&valor_str(v))));
            Ok(Red::Comp { tipo, nombre, valor, texto })
        }
        _ => Err(err("cada elemento es un texto (\"R1 = 10k\"), un objeto {type, name, value} o {series|parallel: […]}")),
    }
}

fn contar(r: &Red) -> usize {
    match r {
        Red::Comp { .. } => 1,
        Red::Serie(v) | Red::Paralelo(v) => v.iter().map(contar).sum(),
    }
}

/// Impedancia a la pulsación ω (None si falta algún valor o hay fuentes dentro de la red).
pub fn impedancia(r: &Red, w: f64) -> Option<Cx> {
    match r {
        Red::Comp { tipo, valor, .. } => {
            let v = (*valor)?;
            match tipo {
                'R' => Some(Cx(v, 0.0)),
                'L' => Some(Cx(0.0, w * v)),
                'C' => {
                    if w == 0.0 {
                        Some(Cx(f64::INFINITY, 0.0))
                    } else {
                        Some(Cx(0.0, -1.0 / (w * v)))
                    }
                }
                'Z' => Some(Cx(v, 0.0)),
                _ => None,
            }
        }
        Red::Serie(v) => v
            .iter()
            .try_fold(Cx(0.0, 0.0), |acc, x| impedancia(x, w).map(|z| acc.suma(z))),
        Red::Paralelo(v) => {
            let mut y = Cx(0.0, 0.0);
            for x in v {
                let z = impedancia(x, w)?;
                if z.0.is_infinite() {
                    continue; // rama abierta
                }
                if z.modulo() == 0.0 {
                    return Some(Cx(0.0, 0.0)); // cortocircuito
                }
                y = y.suma(z.inv());
            }
            if y.modulo() == 0.0 {
                Some(Cx(f64::INFINITY, 0.0))
            } else {
                Some(y.inv())
            }
        }
    }
}

const COMP_W: f64 = 64.0;
const COMP_H: f64 = 44.0;
const SEP_PAR: f64 = 14.0;
const PUNTA: f64 = 16.0;

/// (ancho, alto, desplazamiento vertical del cable de entrada/salida).
fn medir(r: &Red) -> (f64, f64, f64) {
    match r {
        Red::Comp { nombre, texto, .. } => {
            let w = COMP_W.max(ancho_texto(nombre, 10.0).max(ancho_texto(texto, 9.5)) + 10.0);
            (w, COMP_H, COMP_H / 2.0)
        }
        Red::Serie(v) => {
            let medidas: Vec<_> = v.iter().map(medir).collect();
            let arriba = medidas.iter().map(|m| m.2).fold(0.0, f64::max);
            let abajo = medidas.iter().map(|m| m.1 - m.2).fold(0.0, f64::max);
            (
                medidas.iter().map(|m| m.0).sum::<f64>(),
                arriba + abajo,
                arriba,
            )
        }
        Red::Paralelo(v) => {
            let medidas: Vec<_> = v.iter().map(medir).collect();
            let w = medidas.iter().map(|m| m.0).fold(0.0, f64::max) + 2.0 * PUNTA;
            let h = medidas.iter().map(|m| m.1).sum::<f64>() + SEP_PAR * (medidas.len() - 1) as f64;
            (w, h, medidas[0].2)
        }
    }
}

fn simbolo(c: &mut Lienzo, tipo: char, x: f64, y: f64, w: f64, iec: bool) {
    // Cuerpo centrado de 36 px; cables hasta los bordes.
    let cx = x + w / 2.0;
    let (a, z) = (cx - 18.0, cx + 18.0);
    c.linea(x, y, a, y, "strokeMain", 1.6, None, "");
    c.linea(z, y, x + w, y, "strokeMain", 1.6, None, "");
    match tipo {
        'R' if iec => {
            c.r(a, y - 6.0, 36.0, 12.0, "fillCell", 1.6, 0.0);
        }
        'R' => {
            let mut d = format!("M {a:.1} {y:.1}");
            for k in 0..6 {
                let xx = a + 3.0 + k as f64 * 6.0;
                let yy = if k % 2 == 0 { y - 7.0 } else { y + 7.0 };
                d.push_str(&format!(" L {xx:.1} {yy:.1}"));
            }
            d.push_str(&format!(" L {z:.1} {y:.1}"));
            camino(c, &d, "none", "strokeMain", 1.6);
        }
        'L' => {
            let mut d = format!("M {a:.1} {y:.1}");
            for k in 0..4 {
                let x1 = a + (k + 1) as f64 * 9.0;
                d.push_str(&format!(" A 4.5 4.5 0 0 1 {x1:.1} {y:.1}"));
            }
            camino(c, &d, "none", "strokeMain", 1.6);
        }
        'C' => {
            c.linea(a, y, cx - 4.0, y, "strokeMain", 1.6, None, "");
            c.linea(cx + 4.0, y, z, y, "strokeMain", 1.6, None, "");
            c.linea(
                cx - 4.0,
                y - 10.0,
                cx - 4.0,
                y + 10.0,
                "strokeMain",
                2.2,
                None,
                "",
            );
            c.linea(
                cx + 4.0,
                y - 10.0,
                cx + 4.0,
                y + 10.0,
                "strokeMain",
                2.2,
                None,
                "",
            );
        }
        'V' | 'I' => {
            c.linea(a, y, cx - 11.0, y, "strokeMain", 1.6, None, "");
            c.linea(cx + 11.0, y, z, y, "strokeMain", 1.6, None, "");
            circulo(c, cx, y, 11.0, "fillCell", "strokeMain", 1.6);
            if tipo == 'V' {
                c.t(
                    cx - 5.0,
                    y + 3.5,
                    "+",
                    10.0,
                    700,
                    "textMain",
                    "middle",
                    MONO,
                );
                c.t(
                    cx + 5.0,
                    y + 3.5,
                    "−",
                    10.0,
                    700,
                    "textMain",
                    "middle",
                    MONO,
                );
            } else {
                c.linea(
                    cx - 6.0,
                    y,
                    cx + 5.0,
                    y,
                    "strokeMain",
                    1.4,
                    Some("main"),
                    "",
                );
            }
        }
        _ => {
            c.r(a, y - 8.0, 36.0, 16.0, "fillPointerBox", 1.6, 0.0);
        }
    }
}

fn dibujar_red(c: &mut Lienzo, r: &Red, x: f64, y: f64, iec: bool, blanco_val: bool) {
    // (x, y) = esquina superior izquierda del bloque medido.
    match r {
        Red::Comp {
            tipo,
            nombre,
            texto,
            ..
        } => {
            let (w, _, off) = medir(r);
            let yc = y + off;
            simbolo(c, *tipo, x, yc, w, iec);
            c.t(
                x + w / 2.0,
                yc - 13.0,
                nombre,
                10.0,
                700,
                "textMain",
                "middle",
                MONO,
            );
            if !texto.is_empty() && !blanco_val {
                c.t(
                    x + w / 2.0,
                    yc + 21.0,
                    texto,
                    9.5,
                    500,
                    "textSub",
                    "middle",
                    MONO,
                );
            }
        }
        Red::Serie(v) => {
            let (_, _, off) = medir(r);
            let mut xx = x;
            for hijo in v {
                let (w, _, o) = medir(hijo);
                dibujar_red(c, hijo, xx, y + off - o, iec, blanco_val);
                xx += w;
            }
        }
        Red::Paralelo(v) => {
            let (w, _, _) = medir(r);
            let mut yy = y;
            let mut ys = Vec::new();
            for hijo in v {
                let (wh, hh, o) = medir(hijo);
                let yc = yy + o;
                ys.push(yc);
                // Centrar la rama y completar con cable.
                let x0 = x + PUNTA + (w - 2.0 * PUNTA - wh) / 2.0;
                c.linea(x, yc, x0, yc, "strokeMain", 1.6, None, "");
                c.linea(x0 + wh, yc, x + w, yc, "strokeMain", 1.6, None, "");
                dibujar_red(c, hijo, x0, yy, iec, blanco_val);
                yy += hh + SEP_PAR;
            }
            let (y1, y2) = (ys[0], *ys.last().unwrap());
            c.linea(x, y1, x, y2, "strokeMain", 1.6, None, "");
            c.linea(x + w, y1, x + w, y2, "strokeMain", 1.6, None, "");
            c.punto(x, y1, "strokeMain", 2.6);
            c.punto(x + w, y1, "strokeMain", 2.6);
        }
    }
}

pub fn render_circuit(c: &mut Lienzo, b: &Bloque) -> Result<Punto, ErrorDiagrama> {
    let red = match b.get("network") {
        Some(v @ Value::Object(_)) => leer_comp(v, 0)?,
        Some(Value::Array(a)) => Red::Serie(
            a.iter()
                .map(|x| leer_comp(x, 1))
                .collect::<Result<_, _>>()?,
        ),
        _ => return Err(err("falta 'network': {series: […]} o {parallel: […]}")),
    };
    if contar(&red) > 40 {
        return Err(err("como máximo 40 componentes"));
    }
    let fuente = match b.get("source") {
        None | Some(Value::Null) => None,
        Some(v) => Some(leer_comp(v, 1)?),
    };
    let iec = opcion(b, "style", &["ansi", "iec"], "ansi")? == "iec";
    let blanco = bool_opt(b, "blank", false);
    let frecuencia = b
        .get("frequency")
        .and_then(|v| num(v).or_else(|| valor_si(&valor_str(v))));
    let y0 = encabezado(c, b, "");
    let (w, h, off) = medir(&red);
    let rotulo_fuente = match &fuente {
        Some(Red::Comp { nombre, texto, .. }) => {
            ancho_texto(nombre, 10.0).max(ancho_texto(texto, 9.5))
        }
        _ => ancho_texto("Zeq →", 9.5),
    };
    let x_fuente = 30.0_f64.max(rotulo_fuente + 32.0);
    let top = y0 + 20.0;
    let y_cable = top + off;
    let x_red = x_fuente + 40.0;
    let y_retorno = top + h + 24.0;
    let x_fin = x_red + w + 30.0;
    // Lazo: fuente (vertical) → red → retorno.
    c.linea(
        x_fuente,
        y_cable,
        x_red,
        y_cable,
        "strokeMain",
        1.6,
        None,
        "",
    );
    dibujar_red(
        c,
        &red,
        x_red,
        top,
        iec,
        blanco && bool_opt(b, "blankValues", false),
    );
    c.linea(
        x_red + w,
        y_cable,
        x_fin,
        y_cable,
        "strokeMain",
        1.6,
        None,
        "",
    );
    c.linea(
        x_fin,
        y_cable,
        x_fin,
        y_retorno,
        "strokeMain",
        1.6,
        None,
        "",
    );
    c.linea(
        x_fuente,
        y_retorno,
        x_fin,
        y_retorno,
        "strokeMain",
        1.6,
        None,
        "",
    );
    let ym = (y_cable + y_retorno) / 2.0;
    match &fuente {
        Some(Red::Comp {
            tipo,
            nombre,
            texto,
            ..
        }) => {
            c.linea(
                x_fuente,
                y_cable,
                x_fuente,
                ym - 16.0,
                "strokeMain",
                1.6,
                None,
                "",
            );
            c.linea(
                x_fuente,
                ym + 16.0,
                x_fuente,
                y_retorno,
                "strokeMain",
                1.6,
                None,
                "",
            );
            circulo(c, x_fuente, ym, 16.0, "fillCell", "strokeMain", 1.6);
            if *tipo == 'I' {
                c.linea(
                    x_fuente,
                    ym + 8.0,
                    x_fuente,
                    ym - 9.0,
                    "strokeMain",
                    1.5,
                    Some("main"),
                    "",
                );
            } else if frecuencia.is_some() {
                camino(
                    c,
                    &format!("M {:.1} {ym:.1} q 4 -9 8 0 q 4 9 8 0", x_fuente - 8.0),
                    "none",
                    "strokeMain",
                    1.4,
                );
            } else {
                c.t(
                    x_fuente,
                    ym - 4.0,
                    "+",
                    11.0,
                    700,
                    "textMain",
                    "middle",
                    MONO,
                );
                c.t(
                    x_fuente,
                    ym + 11.0,
                    "−",
                    11.0,
                    700,
                    "textMain",
                    "middle",
                    MONO,
                );
            }
            c.t(
                x_fuente - 22.0,
                ym - 3.0,
                nombre,
                10.0,
                700,
                "textMain",
                "end",
                MONO,
            );
            c.t(
                x_fuente - 22.0,
                ym + 11.0,
                texto,
                9.5,
                500,
                "textSub",
                "end",
                MONO,
            );
        }
        _ => {
            // Bornes abiertos (red sola).
            c.linea(
                x_fuente,
                y_cable,
                x_fuente,
                ym - 12.0,
                "strokeMain",
                1.6,
                None,
                "",
            );
            c.linea(
                x_fuente,
                ym + 12.0,
                x_fuente,
                y_retorno,
                "strokeMain",
                1.6,
                None,
                "",
            );
            circulo(c, x_fuente, ym - 12.0, 3.0, "fillCell", "strokeMain", 1.4);
            circulo(c, x_fuente, ym + 12.0, 3.0, "fillCell", "strokeMain", 1.4);
            c.t(
                x_fuente - 8.0,
                ym + 4.0,
                "Zeq →",
                9.5,
                600,
                "textSub",
                "end",
                MONO,
            );
        }
    }
    c.ancho_min = c.ancho_min.max(x_fin + 10.0);
    let mut y = y_retorno + 26.0;
    let mut ancho = x_fin + 10.0;
    if bool_opt(b, "results", true) {
        let w_ang = frecuencia.map(|f| 2.0 * PI * f).unwrap_or(0.0);
        let mut lineas = Vec::new();
        if let Some(f) = frecuencia {
            lineas.push(format!(
                "f = {}Hz · ω = {} rad/s",
                fmt_si(f, ""),
                fmt(w_ang, 4)
            ));
        }
        let simbolo_z = if frecuencia.is_some() { "Zeq" } else { "Req" };
        match impedancia(&red, w_ang) {
            Some(z) if z.0.is_finite() => {
                lineas.push(if blanco {
                    format!("{simbolo_z} = ____")
                } else {
                    format!("{simbolo_z} = {}", fmt_cx(z, "Ω"))
                });
                if let Some(Red::Comp {
                    tipo: 'V',
                    valor: Some(v),
                    ..
                }) = &fuente
                {
                    if z.modulo() > 0.0 {
                        let i = Cx(*v, 0.0).por(z.inv());
                        lineas.push(if blanco {
                            "I = ____".into()
                        } else {
                            format!("I = {}", fmt_cx(i, "A"))
                        });
                    }
                }
            }
            Some(_) => lineas.push(format!("{simbolo_z} = ∞ (circuito abierto en continua)")),
            None => {}
        }
        for l in lineas {
            c.t(0.0, y, &l, 10.5, 700, "textMain", "start", MONO);
            ancho = ancho.max(ancho_texto(&l, 10.5));
            y += 16.0;
        }
    }
    Ok((ancho, y - 6.0))
}

// ---------------------------------------------------------------------------
// phasor
// ---------------------------------------------------------------------------

pub fn render_phasor(c: &mut Lienzo, b: &Bloque) -> Result<Punto, ErrorDiagrama> {
    let mut fasores: Vec<(String, Cx)> = Vec::new();
    for (k, v) in lista(b, "phasors", false)?.iter().enumerate() {
        let m = v.as_object().ok_or_else(|| {
            err(format!(
                "phasors[{k}] es {{label, magnitude, angle}} o {{label, re, im}}"
            ))
        })?;
        let etiqueta = m
            .get("label")
            .map(valor_str)
            .unwrap_or_else(|| format!("F{}", k + 1));
        let z = if m.contains_key("re") || m.contains_key("im") {
            Cx(
                m.get("re").and_then(num).unwrap_or(0.0),
                m.get("im").and_then(num).unwrap_or(0.0),
            )
        } else {
            let r = m
                .get("magnitude")
                .and_then(num)
                .ok_or_else(|| err(format!("phasors[{k}] necesita 'magnitude'")))?;
            let a = m.get("angle").and_then(num).unwrap_or(0.0).to_radians();
            Cx(r * a.cos(), r * a.sin())
        };
        fasores.push((etiqueta, z));
    }
    if fasores.is_empty() || fasores.len() > 12 {
        return Err(err("entre 1 y 12 fasores"));
    }
    let suma = bool_opt(b, "sum", false);
    let encadenar = bool_opt(b, "tipToTail", false);
    let blanco = bool_opt(b, "blank", false);
    let unidad = str_b(b, "unit", "");
    let total = fasores.iter().fold(Cx(0.0, 0.0), |a, (_, z)| a.suma(*z));
    let mut alcance = fasores.iter().map(|(_, z)| z.modulo()).fold(0.0, f64::max);
    if suma || encadenar {
        let mut acc = Cx(0.0, 0.0);
        for (_, z) in &fasores {
            acc = acc.suma(*z);
            alcance = alcance.max(acc.modulo());
        }
    }
    let alcance = alcance.max(1e-9) * 1.15;
    let y0 = encabezado(c, b, "");
    let r_px = 150.0;
    let (cx, cy) = (r_px + 40.0, y0 + 20.0 + r_px);
    let esc = r_px / alcance;
    let p = |z: Cx| (cx + z.0 * esc, cy - z.1 * esc);
    // Grilla polar.
    for k in 1..=4 {
        circulo(c, cx, cy, r_px * k as f64 / 4.0, "none", "strokeMuted", 0.6);
    }
    for k in 0..12 {
        let a = k as f64 * PI / 6.0;
        c.linea(
            cx,
            cy,
            cx + r_px * a.cos(),
            cy - r_px * a.sin(),
            "strokeMuted",
            0.4,
            None,
            "2 4",
        );
    }
    c.linea(
        cx - r_px - 14.0,
        cy,
        cx + r_px + 14.0,
        cy,
        "strokeSub",
        1.2,
        Some("sub"),
        "",
    );
    c.linea(
        cx,
        cy + r_px + 14.0,
        cx,
        cy - r_px - 14.0,
        "strokeSub",
        1.2,
        Some("sub"),
        "",
    );
    c.t(
        cx + r_px + 18.0,
        cy + 4.0,
        "Re",
        10.5,
        700,
        "textSub",
        "start",
        SANS,
    );
    c.t(
        cx + 6.0,
        cy - r_px - 16.0,
        "Im",
        10.5,
        700,
        "textSub",
        "start",
        SANS,
    );
    c.t(
        cx + r_px,
        cy + 14.0,
        &fmt(alcance, 3),
        9.0,
        400,
        "textMuted",
        "middle",
        MONO,
    );
    let mut origen = Cx(0.0, 0.0);
    for (k, (et, z)) in fasores.iter().enumerate() {
        let inicio = if encadenar { origen } else { Cx(0.0, 0.0) };
        let fin = inicio.suma(*z);
        let (x1, y1) = p(inicio);
        let (x2, y2) = p(fin);
        let color = if k % 2 == 0 {
            "strokeMain"
        } else {
            "strokeSub"
        };
        c.linea(
            x1,
            y1,
            x2,
            y2,
            color,
            2.0,
            Some(if k % 2 == 0 { "main" } else { "sub" }),
            "",
        );
        let (dx, dy) = (x2 - x1, y2 - y1);
        let d = dx.hypot(dy).max(1.0);
        c.t(
            x2 + dx / d * 10.0,
            y2 + dy / d * 10.0 + 4.0,
            et,
            11.0,
            700,
            "textMain",
            "middle",
            MONO,
        );
        origen = fin;
    }
    if suma || encadenar {
        let (x2, y2) = p(total);
        if !blanco {
            c.linea(cx, cy, x2, y2, "accentPrimary", 2.6, Some("main"), "5 3");
            c.t(
                x2 + 8.0,
                y2 - 8.0,
                &str_b(b, "sumLabel", "Σ"),
                12.0,
                700,
                "accentPrimary",
                "start",
                MONO,
            );
        }
    }
    let mut y = cy + r_px + 36.0;
    let mut ancho = cx + r_px + 50.0;
    if bool_opt(b, "table", true) {
        let mut filas: Vec<Vec<Celda>> = fasores
            .iter()
            .map(|(et, z)| {
                vec![
                    celda(et).fuerte(true),
                    celda(format!(
                        "{}∠{}°",
                        fmt(z.modulo(), 4),
                        fmt(z.angulo_deg(), 2)
                    )),
                    celda(format!(
                        "{} {} j{}",
                        fmt(z.0, 4),
                        if z.1 < 0.0 { "−" } else { "+" },
                        fmt(z.1.abs(), 4)
                    )),
                ]
            })
            .collect();
        if suma || encadenar {
            filas.push(vec![
                celda(str_b(b, "sumLabel", "Σ")).fuerte(true),
                celda(format!(
                    "{}∠{}°",
                    fmt(total.modulo(), 4),
                    fmt(total.angulo_deg(), 2)
                ))
                .en_blanco(blanco)
                .resaltada(!blanco),
                celda(format!(
                    "{} {} j{}",
                    fmt(total.0, 4),
                    if total.1 < 0.0 { "−" } else { "+" },
                    fmt(total.1.abs(), 4)
                ))
                .en_blanco(blanco)
                .resaltada(!blanco),
            ]);
        }
        let u = if unidad.is_empty() {
            String::new()
        } else {
            format!(" ({unidad})")
        };
        let enc = vec![
            "Fasor".to_string(),
            format!("Polar{u}"),
            format!("Rectangular{u}"),
        ];
        let (w, h) = tabla(
            c,
            0.0,
            y - 10.0,
            &enc,
            &filas,
            &OpcTabla {
                fila_h: 21.0,
                ..Default::default()
            },
        );
        ancho = ancho.max(w);
        y += h;
    }
    Ok((ancho, y - 4.0))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn impedancias() {
        let red = Red::Serie(vec![
            Red::Comp {
                tipo: 'R',
                nombre: "R1".into(),
                valor: Some(100.0),
                texto: String::new(),
            },
            Red::Paralelo(vec![
                Red::Comp {
                    tipo: 'R',
                    nombre: "R2".into(),
                    valor: Some(200.0),
                    texto: String::new(),
                },
                Red::Comp {
                    tipo: 'R',
                    nombre: "R3".into(),
                    valor: Some(200.0),
                    texto: String::new(),
                },
            ]),
        ]);
        assert_eq!(impedancia(&red, 0.0), Some(Cx(200.0, 0.0)));
        // RL serie a 50 Hz: 10 + j·2π·50·0,1.
        let rl = Red::Serie(vec![
            Red::Comp {
                tipo: 'R',
                nombre: "R".into(),
                valor: Some(10.0),
                texto: String::new(),
            },
            Red::Comp {
                tipo: 'L',
                nombre: "L".into(),
                valor: Some(0.1),
                texto: String::new(),
            },
        ]);
        let z = impedancia(&rl, 2.0 * PI * 50.0).unwrap();
        assert!((z.1 - 31.415_926).abs() < 1e-5);
        assert_eq!(fmt_si(4700.0, "Ω"), "4,7 kΩ");
        assert_eq!(fmt_si(0.0001, "F"), "100 µF");
    }
}
