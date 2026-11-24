//! Señales y sistemas: `signal`, `convolution`, `fourier` y `sampling`.

use crate::expr::{evaluar, parsear};
use dibu_comp::comun::*;
use dibu_plot::{leyenda, Ejes, EjesConfig};
use serde_json::Value;
use std::f64::consts::PI;

pub const ML: f64 = 56.0; // margen izquierdo de los ejes
pub const ANCHO_PLOT: f64 = 520.0;

/// Rango (lo, hi) con margen; si es plano, lo abre ±1.
pub fn rango_y(vals: impl Iterator<Item = f64>, incluir_cero: bool) -> (f64, f64) {
    let (mut lo, mut hi) = (f64::INFINITY, f64::NEG_INFINITY);
    for v in vals.filter(|v| v.is_finite()) {
        lo = lo.min(v);
        hi = hi.max(v);
    }
    if !lo.is_finite() {
        return (-1.0, 1.0);
    }
    if incluir_cero {
        lo = lo.min(0.0);
        hi = hi.max(0.0);
    }
    if (hi - lo).abs() < 1e-12 {
        return (lo - 1.0, hi + 1.0);
    }
    let m = (hi - lo) * 0.12;
    (lo - if lo < 0.0 { m } else { 0.0 }, hi + m)
}

#[allow(clippy::too_many_arguments)]
pub fn ejes(
    c: &mut Lienzo,
    y: f64,
    w: f64,
    h: f64,
    xr: Punto,
    yr: Punto,
    xlabel: &str,
    ylabel: &str,
) -> Ejes {
    Ejes::dibujar(
        c,
        EjesConfig {
            x0: ML,
            y0: y,
            w,
            h,
            xr,
            yr,
            xlabel,
            ylabel,
            log: false,
            grilla: true,
            origen: true,
            ypaso: None,
        },
    )
}

/// Gráfico de varillas (señal discreta).
pub fn varillas(c: &mut Lienzo, e: &Ejes, pts: &[Punto], color: &str, resaltar: &[i64]) {
    let y0 = e.py(0.0_f64.clamp(e.yr.0.min(e.yr.1), e.yr.0.max(e.yr.1)));
    for &(x, y) in pts {
        if !y.is_finite() {
            continue;
        }
        let (px, py) = (e.px(x), e.py(y));
        if !(px.is_finite() && py.is_finite()) || px < e.x0 - 0.5 || px > e.x0 + e.w + 0.5 {
            continue;
        }
        let (ymin, ymax) = (e.y0, e.y0 + e.h);
        let py = py.clamp(ymin, ymax);
        let fuerte = resaltar.contains(&(x.round() as i64));
        c.linea(
            px,
            y0,
            px,
            py,
            color,
            if fuerte { 2.4 } else { 1.6 },
            None,
            "",
        );
        circulo(c, px, py, if fuerte { 4.2 } else { 3.2 }, color, color, 1.0);
    }
}

fn rango_x(b: &Bloque, defecto: Punto) -> Result<Punto, ErrorDiagrama> {
    match b.get("range") {
        None | Some(Value::Null) => Ok(defecto),
        Some(Value::Array(a)) if a.len() == 2 => {
            let lo = num(&a[0]).ok_or_else(|| err("'range' es [desde, hasta]"))?;
            let hi = num(&a[1]).ok_or_else(|| err("'range' es [desde, hasta]"))?;
            if hi <= lo || hi - lo > 1e6 {
                return Err(err("'range' necesita desde < hasta"));
            }
            Ok((lo, hi))
        }
        _ => Err(err("'range' es [desde, hasta]")),
    }
}

fn rango_y_fijo(b: &Bloque) -> Result<Option<Punto>, ErrorDiagrama> {
    match b.get("yrange") {
        None | Some(Value::Null) => Ok(None),
        Some(Value::Array(a)) if a.len() == 2 => {
            let lo = num(&a[0]).ok_or_else(|| err("'yrange' es [mín, máx]"))?;
            let hi = num(&a[1]).ok_or_else(|| err("'yrange' es [mín, máx]"))?;
            if hi <= lo {
                return Err(err("'yrange' necesita mín < máx"));
            }
            Ok(Some((lo, hi)))
        }
        _ => Err(err("'yrange' es [mín, máx]")),
    }
}

// ---------------------------------------------------------------------------
// signal
// ---------------------------------------------------------------------------

pub fn render_signal(c: &mut Lienzo, b: &Bloque) -> Result<Punto, ErrorDiagrama> {
    let discreta = opcion(b, "domain", &["continuous", "discrete"], "continuous")? == "discrete";
    let var = if discreta { "n" } else { "t" };
    let var = str_b(b, "variable", var);
    let mut senales: Vec<(String, String)> = Vec::new();
    match b.get("signals") {
        Some(Value::Array(a)) => {
            for (k, v) in a.iter().enumerate() {
                match v {
                    Value::String(s) => senales.push((s.clone(), String::new())),
                    Value::Object(m) => senales.push((
                        m.get("expr")
                            .map(valor_str)
                            .ok_or_else(|| err(format!("signals[{k}] necesita 'expr'")))?,
                        m.get("label").map(valor_str).unwrap_or_default(),
                    )),
                    _ => return Err(err("cada señal es un texto o {expr, label}")),
                }
            }
        }
        _ => senales.push((
            b.get("expr")
                .map(valor_str)
                .ok_or_else(|| err("falta 'expr' o 'signals'"))?,
            str_b(b, "label", ""),
        )),
    }
    if senales.is_empty() || senales.len() > 6 {
        return Err(err("entre 1 y 6 señales"));
    }
    let exprs = senales
        .iter()
        .map(|(e, _)| parsear(e, &var))
        .collect::<Result<Vec<_>, _>>()?;
    let (lo, hi) = rango_x(b, if discreta { (-5.0, 10.0) } else { (-2.0, 5.0) })?;
    if discreta && hi - lo > 400.0 {
        return Err(err("una señal discreta admite como máximo 400 muestras"));
    }
    let blanco = bool_opt(b, "blank", false);
    let apilar = bool_opt(b, "stack", discreta);
    let etiqueta = |k: usize| -> String {
        let (e, l) = &senales[k];
        if !l.is_empty() {
            l.clone()
        } else {
            e.clone()
        }
    };
    let muestras: Vec<Vec<Punto>> = exprs
        .iter()
        .map(|e| {
            if discreta {
                (lo.ceil() as i64..=hi.floor() as i64)
                    .map(|n| (n as f64, evaluar(e, n as f64)))
                    .collect()
            } else {
                let n = 900;
                (0..=n)
                    .map(|i| {
                        let t = lo + (hi - lo) * i as f64 / n as f64;
                        (t, evaluar(e, t))
                    })
                    .collect()
            }
        })
        .collect();
    let yfijo = rango_y_fijo(b)?;
    let y0 = encabezado(c, b, "");
    let mut y = y0 + 8.0;
    let h = num_b(
        b,
        "height",
        if apilar && senales.len() > 1 {
            140.0
        } else {
            220.0
        },
    )
    .clamp(60.0, 600.0);
    let grupos: Vec<Vec<usize>> = if apilar {
        (0..senales.len()).map(|k| vec![k]).collect()
    } else {
        vec![(0..senales.len()).collect()]
    };
    for g in &grupos {
        let yr = yfijo.unwrap_or_else(|| {
            rango_y(
                g.iter().flat_map(|&k| muestras[k].iter().map(|p| p.1)),
                true,
            )
        });
        let ylabel = if g.len() == 1 {
            etiqueta(g[0])
        } else {
            String::new()
        };
        let ylabel_corto = if ylabel.chars().count() <= 14 {
            ylabel.clone()
        } else {
            String::new()
        };
        let e = ejes(c, y, ANCHO_PLOT, h, (lo, hi), yr, &var, &ylabel_corto);
        if !blanco {
            for (j, &k) in g.iter().enumerate() {
                if discreta {
                    varillas(
                        c,
                        &e,
                        &muestras[k],
                        if j == 0 { "accentPrimary" } else { "strokeSub" },
                        &[],
                    );
                } else {
                    e.curva(c, &muestras[k], j, 2.0, "accentPrimary");
                }
            }
        }
        // Rótulo con la expresión arriba a la derecha.
        if g.len() == 1 {
            let txt = if senales[g[0]].1.is_empty() {
                senales[g[0]].0.clone()
            } else {
                format!("{} = {}", senales[g[0]].1, senales[g[0]].0)
            };
            c.t(
                ML + ANCHO_PLOT,
                y - 4.0,
                &txt,
                10.0,
                600,
                "textSub",
                "end",
                MONO,
            );
        } else {
            let etiquetas: Vec<String> = g.iter().map(|&k| etiqueta(k)).collect();
            if !blanco {
                leyenda(c, ML + ANCHO_PLOT + 14.0, y + 10.0, &etiquetas);
                let lw = etiquetas
                    .iter()
                    .map(|s| ancho_texto(s, 10.0))
                    .fold(0.0, f64::max);
                c.ancho_min = c.ancho_min.max(ML + ANCHO_PLOT + 50.0 + lw);
            }
        }
        y += h + 46.0;
    }
    Ok((ML + ANCHO_PLOT + 10.0, y - 8.0))
}

// ---------------------------------------------------------------------------
// convolution
// ---------------------------------------------------------------------------

fn secuencia(b: &Bloque, clave: &str, inicio: &str) -> Result<(Vec<f64>, i64), ErrorDiagrama> {
    let v = lista_num(b, clave, false)?;
    if v.is_empty() || v.len() > 64 {
        return Err(err(format!("'{clave}' necesita entre 1 y 64 valores")));
    }
    let n0 = entero_b(b, inicio, 0, -1000, 1000)?;
    Ok((v, n0))
}

pub fn convolucion(x: &[f64], h: &[f64]) -> Vec<f64> {
    let mut y = vec![0.0; x.len() + h.len() - 1];
    for (i, a) in x.iter().enumerate() {
        for (j, bb) in h.iter().enumerate() {
            y[i + j] += a * bb;
        }
    }
    y
}

pub fn render_convolution(c: &mut Lienzo, b: &Bloque) -> Result<Punto, ErrorDiagrama> {
    let (x, nx) = secuencia(b, "x", "xStart")?;
    let (h, nh) = secuencia(b, "h", "hStart")?;
    let y = convolucion(&x, &h);
    let ny = nx + nh;
    let blanco = bool_opt(b, "blank", false);
    let paso = match b.get("step") {
        None | Some(Value::Null) => None,
        Some(v) => Some(
            num(v)
                .filter(|z| z.fract() == 0.0)
                .ok_or_else(|| err("'step' es un entero n"))? as i64,
        ),
    };
    let fin_y = ny + y.len() as i64 - 1;
    let lo = (nx.min(nh).min(ny) - 2) as f64;
    let mut hi = (fin_y.max(nx + x.len() as i64).max(nh + h.len() as i64) + 1) as f64;
    if let Some(n) = paso {
        hi = hi.max(n as f64 + 1.0);
    }
    let pts = |v: &[f64], n0: i64| -> Vec<Punto> {
        v.iter()
            .enumerate()
            .map(|(k, val)| ((n0 + k as i64) as f64, *val))
            .collect()
    };
    let px = pts(&x, nx);
    let ph = pts(&h, nh);
    let py = pts(&y, ny);
    let y0 = encabezado(c, b, "y[n] = Σ x[k]·h[n−k]");
    let alto = 110.0;
    let mut yy = y0 + 8.0;
    let panel = |c: &mut Lienzo,
                 puntos: &[Punto],
                 extra: &[Punto],
                 etiqueta: &str,
                 var: &str,
                 color: &str,
                 ocultar: bool,
                 resaltar: &[i64],
                 yy: &mut f64|
     -> Ejes {
        let yr = rango_y(puntos.iter().chain(extra).map(|p| p.1), true);
        let e = ejes(c, *yy, ANCHO_PLOT, alto, (lo, hi), yr, var, etiqueta);
        if !ocultar {
            varillas(c, &e, puntos, color, resaltar);
            for &(n, v) in puntos {
                if v != 0.0 && puntos.len() <= 24 {
                    c.t(
                        e.px(n),
                        e.py(v) - 8.0,
                        &fmt(v, 3),
                        8.5,
                        600,
                        "textSub",
                        "middle",
                        MONO,
                    );
                }
            }
        }
        *yy += alto + 44.0;
        e
    };
    panel(
        c,
        &px,
        &[],
        "x[k]",
        "k",
        "accentPrimary",
        false,
        &[],
        &mut yy,
    );
    panel(c, &ph, &[], "h[k]", "k", "strokeSub", false, &[], &mut yy);
    let mut lineas_paso = Vec::new();
    if let Some(n) = paso {
        // h[n−k] volteada y desplazada sobre x[k].
        let pv: Vec<Punto> = h
            .iter()
            .enumerate()
            .map(|(j, v)| ((n - (nh + j as i64)) as f64, *v))
            .collect();
        let e = panel(
            c,
            &pv,
            &px,
            &format!("x[k] · h[{n}−k]"),
            "k",
            "strokeMuted",
            false,
            &[],
            &mut yy,
        );
        varillas(c, &e, &px, "accentPrimary", &[]);
        c.t(
            ML + ANCHO_PLOT,
            e.y0 - 4.0,
            &format!("x[k] (oscuro) sobre h[{n}−k] (gris)"),
            9.5,
            500,
            "textMuted",
            "end",
            MONO,
        );
        let mut terminos = Vec::new();
        let mut suma = 0.0;
        for (k, xv) in px.iter() {
            if let Some((_, hv)) = pv.iter().find(|(kk, _)| kk == k) {
                if *xv != 0.0 && *hv != 0.0 {
                    terminos.push(format!("{}·{}", fmt(*xv, 3), fmt(*hv, 3)));
                    suma += xv * hv;
                }
            }
        }
        lineas_paso.push(if blanco {
            format!("y[{n}] = Σ x[k]·h[{n}−k] = ____")
        } else if terminos.is_empty() {
            format!("y[{n}] = 0 (no hay superposición)")
        } else {
            format!("y[{n}] = {} = {}", terminos.join(" + "), fmt(suma, 4))
        });
    }
    let resaltar: Vec<i64> = paso.into_iter().collect();
    panel(
        c,
        &py,
        &[],
        "y[n]",
        "n",
        "accentPrimary",
        blanco,
        &resaltar,
        &mut yy,
    );
    let mut ancho = ML + ANCHO_PLOT + 10.0;
    for l in lineas_paso {
        c.t(0.0, yy, &l, 10.5, 700, "textMain", "start", MONO);
        ancho = ancho.max(ancho_texto(&l, 10.5));
        yy += 16.0;
    }
    if bool_opt(b, "values", true) {
        let txt = if blanco {
            format!("y[n] = {{ ____ }}  (n = {ny} … {fin_y})")
        } else {
            format!(
                "y[n] = {{ {} }}  (n = {ny} … {fin_y})",
                y.iter().map(|v| fmt(*v, 3)).collect::<Vec<_>>().join(", ")
            )
        };
        c.t(0.0, yy, &txt, 10.0, 500, "textSub", "start", MONO);
        ancho = ancho.max(ancho_texto(&txt, 10.0));
        yy += 16.0;
    }
    Ok((ancho, yy - 8.0))
}

// ---------------------------------------------------------------------------
// fourier
// ---------------------------------------------------------------------------

pub fn dft(x: &[f64]) -> Vec<(f64, f64)> {
    let n = x.len();
    (0..n)
        .map(|k| {
            let (mut re, mut im) = (0.0, 0.0);
            for (m, v) in x.iter().enumerate() {
                let a = -2.0 * PI * (k * m) as f64 / n as f64;
                re += v * a.cos();
                im += v * a.sin();
            }
            (re, im)
        })
        .collect()
}

/// Coeficientes trigonométricos (a_k, b_k) de ondas clásicas de amplitud A y período T.
pub fn coef_onda(onda: &str, k: usize, a: f64) -> (f64, f64) {
    let kf = k as f64;
    match onda {
        // Cuadrada impar ±A: b_k = 4A/(kπ) para k impar.
        "square" => (0.0, if k % 2 == 1 { 4.0 * a / (kf * PI) } else { 0.0 }),
        // Diente de sierra de −A a A: b_k = 2A(−1)^(k+1)/(kπ).
        "sawtooth" => (
            0.0,
            2.0 * a * if k % 2 == 1 { 1.0 } else { -1.0 } / (kf * PI),
        ),
        // Triangular par de pico A: a_k = 8A/(kπ)² para k impar.
        _ => (
            if k % 2 == 1 {
                8.0 * a / (kf * kf * PI * PI)
            } else {
                0.0
            },
            0.0,
        ),
    }
}

fn onda_ideal(onda: &str, t: f64, periodo: f64, a: f64) -> f64 {
    let fase = (t / periodo).rem_euclid(1.0);
    match onda {
        "square" => {
            if fase < 0.5 {
                a
            } else {
                -a
            }
        }
        "sawtooth" => {
            let f = (t / periodo + 0.5).rem_euclid(1.0);
            a * (2.0 * f - 1.0)
        }
        _ => {
            // Triangular par: pico A en t = 0, −A en t = T/2.
            let d = fase.min(1.0 - fase);
            a * (1.0 - 4.0 * d)
        }
    }
}

pub fn render_fourier(c: &mut Lienzo, b: &Bloque) -> Result<Punto, ErrorDiagrama> {
    let blanco = bool_opt(b, "blank", false);
    let y0 = encabezado(c, b, "");
    let mut y = y0 + 8.0;
    let h = 150.0;
    if let Some(Value::String(onda)) = b.get("wave") {
        let onda = onda.to_lowercase();
        if !matches!(onda.as_str(), "square" | "sawtooth" | "triangle") {
            return Err(err("'wave' es square, sawtooth o triangle"));
        }
        let n = entero_b(b, "harmonics", 7, 1, 60)? as usize;
        let a = num_b(b, "amplitude", 1.0);
        let periodo = num_b(b, "period", 1.0).max(1e-9);
        let coef: Vec<(f64, f64)> = (1..=n).map(|k| coef_onda(&onda, k, a)).collect();
        // Espectro de amplitud de una cara: |c_k| = √(a_k² + b_k²).
        let amp: Vec<Punto> = coef
            .iter()
            .enumerate()
            .map(|(k, (ak, bk))| ((k + 1) as f64, ak.hypot(*bk)))
            .collect();
        let yr = rango_y(amp.iter().map(|p| p.1), true);
        let e = ejes(
            c,
            y,
            ANCHO_PLOT,
            h,
            (0.0, n as f64 + 1.0),
            yr,
            "armónico k",
            "amplitud",
        );
        if !blanco {
            varillas(c, &e, &amp, "accentPrimary", &[]);
        }
        y += h + 48.0;
        // Suma parcial frente a la onda ideal.
        let pts_ideal: Vec<Punto> = (0..=800)
            .map(|i| {
                let t = -0.5 * periodo + 2.0 * periodo * i as f64 / 800.0;
                (t, onda_ideal(&onda, t, periodo, a))
            })
            .collect();
        let w0 = 2.0 * PI / periodo;
        let parcial: Vec<Punto> = pts_ideal
            .iter()
            .map(|&(t, _)| {
                let s: f64 = coef
                    .iter()
                    .enumerate()
                    .map(|(k, (ak, bk))| {
                        let kw = (k + 1) as f64 * w0 * t;
                        ak * kw.cos() + bk * kw.sin()
                    })
                    .sum();
                (t, s)
            })
            .collect();
        let yr = rango_y(parcial.iter().chain(&pts_ideal).map(|p| p.1), true);
        let e = ejes(
            c,
            y,
            ANCHO_PLOT,
            h,
            (-0.5 * periodo, 1.5 * periodo),
            yr,
            "t",
            "",
        );
        e.curva(c, &pts_ideal, 2, 1.2, "strokeMuted");
        if !blanco {
            e.curva(c, &parcial, 0, 2.0, "accentPrimary");
        }
        c.t(
            ML + ANCHO_PLOT,
            y - 4.0,
            &format!(
                "suma parcial con {n} armónico{}",
                if n == 1 { "" } else { "s" }
            ),
            10.0,
            600,
            "textSub",
            "end",
            MONO,
        );
        y += h + 46.0;
        let formula = match onda.as_str() {
            "square" => "x(t) = (4A/π) Σ sin(kω₀t)/k, k impar",
            "sawtooth" => "x(t) = (2A/π) Σ (−1)^(k+1) sin(kω₀t)/k",
            _ => "x(t) = (8A/π²) Σ cos(kω₀t)/k², k impar",
        };
        c.t(0.0, y, formula, 10.5, 600, "textMain", "start", MONO);
        return Ok((ML + ANCHO_PLOT + 10.0, y + 8.0));
    }
    let x = lista_num(b, "sequence", false)?;
    if x.is_empty() || x.len() > 128 {
        return Err(err(
            "'sequence' necesita entre 1 y 128 muestras (o usar 'wave')",
        ));
    }
    let xx = dft(&x);
    let n = x.len();
    let mag: Vec<Punto> = xx
        .iter()
        .enumerate()
        .map(|(k, (re, im))| (k as f64, re.hypot(*im)))
        .collect();
    let pts_x: Vec<Punto> = x.iter().enumerate().map(|(k, v)| (k as f64, *v)).collect();
    let e = ejes(
        c,
        y,
        ANCHO_PLOT,
        h * 0.8,
        (-0.5, n as f64 - 0.5),
        rango_y(x.iter().copied(), true),
        "n",
        "x[n]",
    );
    varillas(c, &e, &pts_x, "strokeSub", &[]);
    y += h * 0.8 + 46.0;
    let e = ejes(
        c,
        y,
        ANCHO_PLOT,
        h,
        (-0.5, n as f64 - 0.5),
        rango_y(mag.iter().map(|p| p.1), true),
        "k",
        "|X[k]|",
    );
    if !blanco {
        varillas(c, &e, &mag, "accentPrimary", &[]);
    }
    y += h + 46.0;
    if bool_opt(b, "phase", false) && !blanco {
        let fase: Vec<Punto> = xx
            .iter()
            .enumerate()
            .map(|(k, (re, im))| {
                (
                    k as f64,
                    if re.hypot(*im) < 1e-9 {
                        0.0
                    } else {
                        im.atan2(*re)
                    },
                )
            })
            .collect();
        let e = ejes(
            c,
            y,
            ANCHO_PLOT,
            h * 0.8,
            (-0.5, n as f64 - 0.5),
            (-PI * 1.1, PI * 1.1),
            "k",
            "∠X[k] (rad)",
        );
        varillas(c, &e, &fase, "strokeSub", &[]);
        y += h * 0.8 + 46.0;
    }
    let mut ancho = ML + ANCHO_PLOT + 10.0;
    if bool_opt(b, "table", n <= 16) {
        let enc: Vec<String> = ["k", "X[k]", "|X[k]|"]
            .iter()
            .map(|s| s.to_string())
            .collect();
        let filas: Vec<Vec<Celda>> = xx
            .iter()
            .enumerate()
            .map(|(k, (re, im))| {
                let re = if re.abs() < 1e-9 { 0.0 } else { *re };
                let im = if im.abs() < 1e-9 { 0.0 } else { *im };
                let z = if im == 0.0 {
                    fmt(re, 3)
                } else if re == 0.0 {
                    format!("{}j", fmt(im, 3))
                } else {
                    format!(
                        "{} {} {}j",
                        fmt(re, 3),
                        if im < 0.0 { "−" } else { "+" },
                        fmt(im.abs(), 3)
                    )
                };
                vec![
                    celda(k.to_string()),
                    celda(z).en_blanco(blanco),
                    celda(fmt(re.hypot(im), 3)).en_blanco(blanco),
                ]
            })
            .collect();
        let (w, hh) = tabla(
            c,
            0.0,
            y - 26.0,
            &enc,
            &filas,
            &OpcTabla {
                fila_h: 20.0,
                ..Default::default()
            },
        );
        ancho = ancho.max(w);
        y += hh;
    }
    Ok((ancho, y - 30.0))
}

// ---------------------------------------------------------------------------
// sampling
// ---------------------------------------------------------------------------

/// Frecuencia aparente de un tono f0 muestreado a fs (en [0, fs/2]).
pub fn alias(f0: f64, fs: f64) -> f64 {
    let r = f0.rem_euclid(fs);
    r.min(fs - r)
}

pub fn render_sampling(c: &mut Lienzo, b: &Bloque) -> Result<Punto, ErrorDiagrama> {
    let f0 = num_req(b, "frequency", 1e-6, 1e9)?;
    let fs = num_req(b, "fs", 1e-6, 1e12)?;
    let fase = num_b(b, "phase", 0.0).to_radians();
    let a = num_b(b, "amplitude", 1.0);
    let periodos = num_b(b, "periods", 3.0).clamp(0.5, 40.0);
    if periodos * fs / f0 > 400.0 {
        return Err(err(
            "demasiadas muestras para dibujar (máximo 400): bajar 'periods' o 'fs'",
        ));
    }
    let blanco = bool_opt(b, "blank", false);
    let dur = periodos / f0;
    let fa = alias(f0, fs);
    let cumple = fs > 2.0 * f0;
    let y0 = encabezado(
        c,
        b,
        &format!("f₀ = {} Hz, fs = {} Hz", fmt(f0, 4), fmt(fs, 4)),
    );
    let mut y = y0 + 8.0;
    let h = 200.0;
    let e = ejes(
        c,
        y,
        ANCHO_PLOT,
        h,
        (0.0, dur),
        (-1.25 * a.abs().max(1e-9), 1.25 * a.abs().max(1e-9)),
        "t (s)",
        "x(t)",
    );
    let cont: Vec<Punto> = (0..=1000)
        .map(|i| {
            let t = dur * i as f64 / 1000.0;
            (t, a * (2.0 * PI * f0 * t + fase).cos())
        })
        .collect();
    e.curva(c, &cont, 0, 1.4, "strokeMuted");
    let n_muestras = (dur * fs).floor() as usize;
    let muestras: Vec<Punto> = (0..=n_muestras)
        .map(|k| {
            let t = k as f64 / fs;
            (t, a * (2.0 * PI * f0 * t + fase).cos())
        })
        .collect();
    varillas(c, &e, &muestras, "accentPrimary", &[]);
    let mostrar_alias = bool_opt(b, "reconstruction", true) && !blanco;
    if mostrar_alias && !cumple {
        // Reconstrucción ideal: el tono alias que pasa por las mismas muestras.
        let k = (f0 / fs).round();
        let signo = if f0 - k * fs >= 0.0 { 1.0 } else { -1.0 };
        let rec: Vec<Punto> = cont
            .iter()
            .map(|&(t, _)| (t, a * (2.0 * PI * fa * t + signo * fase).cos()))
            .collect();
        e.curva(c, &rec, 1, 2.0, "accentPrimary");
        leyenda(
            c,
            ML + ANCHO_PLOT + 14.0,
            y + 12.0,
            &["original".into(), format!("alias {} Hz", fmt(fa, 4))],
        );
        c.ancho_min = c.ancho_min.max(ML + ANCHO_PLOT + 150.0);
    }
    y += h + 48.0;
    if bool_opt(b, "spectrum", false) {
        // Réplicas del espectro en k·fs ± f0.
        let fmax = (2.5 * fs).max(1.5 * f0);
        let e = ejes(
            c,
            y,
            ANCHO_PLOT,
            110.0,
            (0.0, fmax),
            (0.0, 1.3),
            "f (Hz)",
            "|X_s(f)|",
        );
        let mut lineas = Vec::new();
        let mut k = 0.0;
        while k * fs - f0 <= fmax {
            for f in [k * fs - f0, k * fs + f0] {
                if (0.0..=fmax).contains(&f) && !lineas.iter().any(|x: &f64| (x - f).abs() < 1e-9) {
                    lineas.push(f);
                }
            }
            k += 1.0;
        }
        let pts: Vec<Punto> = lineas.iter().map(|&f| (f, 1.0)).collect();
        varillas(c, &e, &pts, "accentPrimary", &[]);
        e.guia(c, fs / 2.0, 0.0, fs / 2.0, 1.25, "5 3", "strokeSub");
        c.t(
            e.px(fs / 2.0),
            e.py(1.25) - 3.0,
            "fs/2",
            9.0,
            600,
            "textSub",
            "middle",
            MONO,
        );
        y += 110.0 + 46.0;
    }
    let lineas = if blanco {
        vec![
            "¿Se cumple el criterio de Nyquist? ____".to_string(),
            "Frecuencia aparente: ____ Hz".to_string(),
        ]
    } else {
        vec![
            format!(
                "Nyquist: fs {} 2·f₀ = {} Hz → {}",
                if cumple { ">" } else { "≤" },
                fmt(2.0 * f0, 4),
                if cumple {
                    "sin aliasing"
                } else {
                    "hay aliasing"
                }
            ),
            format!(
                "Frecuencia aparente tras muestrear: {} Hz",
                fmt(if cumple { f0 } else { fa }, 4)
            ),
        ]
    };
    let mut ancho = ML + ANCHO_PLOT + 10.0;
    for l in lineas {
        c.t(0.0, y, &l, 10.5, 700, "textMain", "start", MONO);
        ancho = ancho.max(ancho_texto(&l, 10.5));
        y += 16.0;
    }
    Ok((ancho, y - 8.0))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn convolucion_y_dft() {
        assert_eq!(
            convolucion(&[1.0, 2.0, 3.0], &[1.0, 1.0]),
            vec![1.0, 3.0, 5.0, 3.0]
        );
        let x = dft(&[1.0, 1.0, 1.0, 1.0]);
        assert!((x[0].0 - 4.0).abs() < 1e-12 && x[1].0.abs() < 1e-12 && x[1].1.abs() < 1e-12);
        assert!((alias(7.0, 10.0) - 3.0).abs() < 1e-12);
        assert!((alias(3.0, 10.0) - 3.0).abs() < 1e-12);
        assert!((alias(12.0, 10.0) - 2.0).abs() < 1e-12);
        let (_, b1) = coef_onda("square", 1, 1.0);
        assert!((b1 - 4.0 / PI).abs() < 1e-12);
    }
}
