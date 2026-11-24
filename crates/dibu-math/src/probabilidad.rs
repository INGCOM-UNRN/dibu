//! Probabilidad: `distribution` (densidades y masas con área sombreada) y `venn`
//! (2 o 3 conjuntos con regiones sombreadas por una expresión).

use crate::senales::{ejes, ML};
use dibu_comp::comun::*;
use serde_json::Value;
use std::f64::consts::PI;

// ---------------------------------------------------------------------------
// Funciones especiales
// ---------------------------------------------------------------------------

pub fn ln_gamma(x: f64) -> f64 {
    // Lanczos (g = 7, n = 9).
    const C: [f64; 9] = [
        0.999_999_999_999_809_9,
        676.520_368_121_885_1,
        -1_259.139_216_722_402_8,
        771.323_428_777_653_1,
        -176.615_029_162_140_6,
        12.507_343_278_686_905,
        -0.138_571_095_265_720_12,
        9.984_369_578_019_572e-6,
        1.505_632_735_149_311_6e-7,
    ];
    if x < 0.5 {
        return (PI / (PI * x).sin()).ln() - ln_gamma(1.0 - x);
    }
    let x = x - 1.0;
    let mut a = C[0];
    let t = x + 7.5;
    for (i, c) in C.iter().enumerate().skip(1) {
        a += c / (x + i as f64);
    }
    0.5 * (2.0 * PI).ln() + (x + 0.5) * t.ln() - t + a.ln()
}

/// Gamma incompleta regularizada inferior P(a, x).
pub fn gamma_p(a: f64, x: f64) -> f64 {
    if x <= 0.0 {
        return 0.0;
    }
    if x < a + 1.0 {
        let (mut suma, mut del, mut ap) = (1.0 / a, 1.0 / a, a);
        for _ in 0..1000 {
            ap += 1.0;
            del *= x / ap;
            suma += del;
            if del.abs() < suma.abs() * 1e-15 {
                break;
            }
        }
        (suma.ln() - x + a * x.ln() - ln_gamma(a)).exp()
    } else {
        1.0 - gamma_q_fc(a, x)
    }
}

fn gamma_q_fc(a: f64, x: f64) -> f64 {
    let tiny = 1e-300;
    let mut b = x + 1.0 - a;
    let mut c = 1.0 / tiny;
    let mut d = 1.0 / b;
    let mut h = d;
    for i in 1..1000 {
        let an = -(i as f64) * (i as f64 - a);
        b += 2.0;
        d = an * d + b;
        if d.abs() < tiny {
            d = tiny;
        }
        c = b + an / c;
        if c.abs() < tiny {
            c = tiny;
        }
        d = 1.0 / d;
        let del = d * c;
        h *= del;
        if (del - 1.0).abs() < 1e-15 {
            break;
        }
    }
    (-x + a * x.ln() - ln_gamma(a)).exp() * h
}

/// Beta incompleta regularizada I_x(a, b).
pub fn beta_i(a: f64, b: f64, x: f64) -> f64 {
    if x <= 0.0 {
        return 0.0;
    }
    if x >= 1.0 {
        return 1.0;
    }
    let bt = (ln_gamma(a + b) - ln_gamma(a) - ln_gamma(b) + a * x.ln() + b * (1.0 - x).ln()).exp();
    if x < (a + 1.0) / (a + b + 2.0) {
        bt * beta_fc(a, b, x) / a
    } else {
        1.0 - bt * beta_fc(b, a, 1.0 - x) / b
    }
}

fn beta_fc(a: f64, b: f64, x: f64) -> f64 {
    let tiny = 1e-300;
    let (qab, qap, qam) = (a + b, a + 1.0, a - 1.0);
    let mut c = 1.0;
    let mut d = 1.0 - qab * x / qap;
    if d.abs() < tiny {
        d = tiny;
    }
    d = 1.0 / d;
    let mut h = d;
    for m in 1..1000 {
        let m = m as f64;
        let m2 = 2.0 * m;
        let aa = m * (b - m) * x / ((qam + m2) * (a + m2));
        d = 1.0 + aa * d;
        if d.abs() < tiny {
            d = tiny;
        }
        c = 1.0 + aa / c;
        if c.abs() < tiny {
            c = tiny;
        }
        d = 1.0 / d;
        h *= d * c;
        let aa = -(a + m) * (qab + m) * x / ((a + m2) * (qap + m2));
        d = 1.0 + aa * d;
        if d.abs() < tiny {
            d = tiny;
        }
        c = 1.0 + aa / c;
        if c.abs() < tiny {
            c = tiny;
        }
        d = 1.0 / d;
        let del = d * c;
        h *= del;
        if (del - 1.0).abs() < 1e-15 {
            break;
        }
    }
    h
}

pub fn phi(z: f64) -> f64 {
    if z >= 0.0 {
        0.5 + 0.5 * gamma_p(0.5, z * z / 2.0)
    } else {
        0.5 - 0.5 * gamma_p(0.5, z * z / 2.0)
    }
}

// ---------------------------------------------------------------------------
// distribution
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy)]
pub enum Dist {
    Normal(f64, f64),
    Uniforme(f64, f64),
    Exponencial(f64),
    T(f64),
    Chi2(f64),
    Binomial(u64, f64),
    Poisson(f64),
    Geometrica(f64),
}

impl Dist {
    pub fn discreta(&self) -> bool {
        matches!(
            self,
            Self::Binomial(..) | Self::Poisson(_) | Self::Geometrica(_)
        )
    }
    pub fn densidad(&self, x: f64) -> f64 {
        match *self {
            Self::Normal(m, s) => {
                (-(x - m).powi(2) / (2.0 * s * s)).exp() / (s * (2.0 * PI).sqrt())
            }
            Self::Uniforme(a, b) => {
                if x >= a && x <= b {
                    1.0 / (b - a)
                } else {
                    0.0
                }
            }
            Self::Exponencial(l) => {
                if x >= 0.0 {
                    l * (-l * x).exp()
                } else {
                    0.0
                }
            }
            Self::T(v) => (ln_gamma((v + 1.0) / 2.0)
                - ln_gamma(v / 2.0)
                - 0.5 * (v * PI).ln()
                - (v + 1.0) / 2.0 * (1.0 + x * x / v).ln())
            .exp(),
            Self::Chi2(k) => {
                if x <= 0.0 {
                    0.0
                } else {
                    ((k / 2.0 - 1.0) * x.ln() - x / 2.0 - (k / 2.0) * 2f64.ln() - ln_gamma(k / 2.0))
                        .exp()
                }
            }
            Self::Binomial(n, p) => {
                let k = x.round();
                if (x - k).abs() > 1e-9 || k < 0.0 || k > n as f64 {
                    return 0.0;
                }
                let nf = n as f64;
                let lc = ln_gamma(nf + 1.0) - ln_gamma(k + 1.0) - ln_gamma(nf - k + 1.0);
                if p == 0.0 {
                    return if k == 0.0 { 1.0 } else { 0.0 };
                }
                if p == 1.0 {
                    return if k == nf { 1.0 } else { 0.0 };
                }
                (lc + k * p.ln() + (nf - k) * (1.0 - p).ln()).exp()
            }
            Self::Poisson(l) => {
                let k = x.round();
                if (x - k).abs() > 1e-9 || k < 0.0 {
                    return 0.0;
                }
                (k * l.ln() - l - ln_gamma(k + 1.0)).exp()
            }
            Self::Geometrica(p) => {
                let k = x.round();
                if (x - k).abs() > 1e-9 || k < 1.0 {
                    return 0.0;
                }
                (1.0 - p).powf(k - 1.0) * p
            }
        }
    }
    /// F(x) = P(X ≤ x).
    pub fn acumulada(&self, x: f64) -> f64 {
        match *self {
            Self::Normal(m, s) => phi((x - m) / s),
            Self::Uniforme(a, b) => ((x - a) / (b - a)).clamp(0.0, 1.0),
            Self::Exponencial(l) => {
                if x <= 0.0 {
                    0.0
                } else {
                    1.0 - (-l * x).exp()
                }
            }
            Self::T(v) => {
                let p = 0.5 * beta_i(v / 2.0, 0.5, v / (v + x * x));
                if x >= 0.0 {
                    1.0 - p
                } else {
                    p
                }
            }
            Self::Chi2(k) => gamma_p(k / 2.0, x.max(0.0) / 2.0),
            _ => {
                let (lo, _) = self.soporte();
                let hasta = x.floor();
                let mut s = 0.0;
                let mut k = lo.max(0.0);
                while k <= hasta && k <= 1e6 {
                    s += self.densidad(k);
                    k += 1.0;
                }
                s.min(1.0)
            }
        }
    }
    /// Rango de dibujo.
    pub fn soporte(&self) -> (f64, f64) {
        match *self {
            Self::Normal(m, s) => (m - 4.0 * s, m + 4.0 * s),
            Self::Uniforme(a, b) => (a - 0.25 * (b - a), b + 0.25 * (b - a)),
            Self::Exponencial(l) => (0.0, 5.0 / l),
            Self::T(v) => {
                let w = if v < 3.0 { 6.0 } else { 4.5 };
                (-w, w)
            }
            Self::Chi2(k) => (0.0, k + 5.0 * (2.0 * k).sqrt() + 2.0),
            Self::Binomial(n, _) => (0.0, n as f64),
            Self::Poisson(l) => (0.0, (l + 5.0 * l.sqrt() + 3.0).ceil()),
            Self::Geometrica(p) => (1.0, (5.0 / p).ceil().max(6.0)),
        }
    }
    pub fn media_var(&self) -> (f64, f64) {
        match *self {
            Self::Normal(m, s) => (m, s * s),
            Self::Uniforme(a, b) => ((a + b) / 2.0, (b - a).powi(2) / 12.0),
            Self::Exponencial(l) => (1.0 / l, 1.0 / (l * l)),
            Self::T(v) => (
                0.0,
                if v > 2.0 {
                    v / (v - 2.0)
                } else {
                    f64::INFINITY
                },
            ),
            Self::Chi2(k) => (k, 2.0 * k),
            Self::Binomial(n, p) => (n as f64 * p, n as f64 * p * (1.0 - p)),
            Self::Poisson(l) => (l, l),
            Self::Geometrica(p) => (1.0 / p, (1.0 - p) / (p * p)),
        }
    }
    pub fn nombre(&self) -> String {
        match *self {
            Self::Normal(m, s) => format!("X ~ N(μ = {}, σ = {})", fmt(m, 4), fmt(s, 4)),
            Self::Uniforme(a, b) => format!("X ~ U({}, {})", fmt(a, 4), fmt(b, 4)),
            Self::Exponencial(l) => format!("X ~ Exp(λ = {})", fmt(l, 4)),
            Self::T(v) => format!("T ~ t de Student ({} g.l.)", fmt(v, 2)),
            Self::Chi2(k) => format!("X ~ χ² ({} g.l.)", fmt(k, 2)),
            Self::Binomial(n, p) => format!("X ~ Bin(n = {n}, p = {})", fmt(p, 4)),
            Self::Poisson(l) => format!("X ~ Poisson(λ = {})", fmt(l, 4)),
            Self::Geometrica(p) => format!("X ~ Geom(p = {})", fmt(p, 4)),
        }
    }
}

fn leer_dist(b: &Bloque) -> Result<Dist, ErrorDiagrama> {
    let tipo = opcion(
        b,
        "dist",
        &[
            "normal",
            "uniform",
            "exponential",
            "t",
            "chi2",
            "binomial",
            "poisson",
            "geometric",
        ],
        "normal",
    )?;
    let pos = |clave: &str, def: f64| -> Result<f64, ErrorDiagrama> {
        let v = num_b(b, clave, def);
        if v > 0.0 && v.is_finite() {
            Ok(v)
        } else {
            Err(err(format!("'{clave}' tiene que ser > 0")))
        }
    };
    let prob = |clave: &str| -> Result<f64, ErrorDiagrama> {
        let p = num_b(b, clave, 0.5);
        if (0.0..=1.0).contains(&p) {
            Ok(p)
        } else {
            Err(err(format!("'{clave}' es una probabilidad entre 0 y 1")))
        }
    };
    Ok(match tipo.as_str() {
        "normal" => Dist::Normal(num_b(b, "mean", 0.0), pos("sd", 1.0)?),
        "uniform" => {
            let (a, bb) = (num_b(b, "a", 0.0), num_b(b, "b", 1.0));
            if bb <= a {
                return Err(err("en una uniforme hace falta a < b"));
            }
            Dist::Uniforme(a, bb)
        }
        "exponential" => Dist::Exponencial(pos("lambda", 1.0)?),
        "t" => Dist::T(pos("df", 5.0)?.min(1e6)),
        "chi2" => Dist::Chi2(pos("df", 3.0)?.min(1e4)),
        "binomial" => {
            let n = entero_b(b, "n", 10, 1, 200)? as u64;
            Dist::Binomial(n, prob("p")?)
        }
        "poisson" => Dist::Poisson(pos("lambda", 3.0)?.min(200.0)),
        _ => {
            let p = prob("p")?;
            if p <= 0.0 {
                return Err(err("en una geométrica hace falta p > 0"));
            }
            Dist::Geometrica(p.max(0.02))
        }
    })
}

pub fn probabilidad(d: &Dist, a: Option<f64>, bb: Option<f64>) -> f64 {
    if d.discreta() {
        let (lo, hi) = d.soporte();
        let desde = a.map(f64::ceil).unwrap_or(lo).max(lo.min(0.0));
        let hasta = bb.map(f64::floor).unwrap_or(f64::INFINITY);
        // Para la cola derecha, 1 − F(desde − 1).
        if bb.is_none() {
            return (1.0 - d.acumulada(desde - 1.0)).clamp(0.0, 1.0);
        }
        let _ = hi;
        return (d.acumulada(hasta) - d.acumulada(desde - 1.0)).clamp(0.0, 1.0);
    }
    let fa = a.map(|x| d.acumulada(x)).unwrap_or(0.0);
    let fb = bb.map(|x| d.acumulada(x)).unwrap_or(1.0);
    (fb - fa).clamp(0.0, 1.0)
}

pub fn render_distribution(c: &mut Lienzo, b: &Bloque) -> Result<Punto, ErrorDiagrama> {
    let d = leer_dist(b)?;
    let blanco = bool_opt(b, "blank", false);
    let intervalo: Option<(Option<f64>, Option<f64>)> = match b.get("interval") {
        None | Some(Value::Null) => None,
        Some(Value::Array(a)) if a.len() == 2 => Some((num(&a[0]), num(&a[1]))),
        _ => return Err(err("'interval' es [a, b] (null para −∞ o +∞)")),
    };
    let (mut lo, mut hi) = d.soporte();
    if let Some((a, bb)) = intervalo {
        if let (Some(a), Some(bb)) = (a, bb) {
            if bb < a {
                return Err(err("'interval' necesita a ≤ b"));
            }
        }
        // Que el intervalo quede a la vista.
        for v in [a, bb].into_iter().flatten() {
            lo = lo.min(v - 0.05 * (hi - lo));
            hi = hi.max(v + 0.05 * (hi - lo));
        }
    }
    let y0 = encabezado(c, b, &d.nombre());
    let h = 220.0;
    let w = 520.0;
    let y = y0 + 8.0;
    if d.discreta() {
        let (k0, k1) = (lo.ceil() as i64, hi.floor() as i64);
        if k1 - k0 > 220 {
            return Err(err("demasiados valores para dibujar"));
        }
        let ps: Vec<(f64, f64)> = (k0..=k1)
            .map(|k| (k as f64, d.densidad(k as f64)))
            .collect();
        let ymax = ps.iter().map(|p| p.1).fold(0.0, f64::max) * 1.15;
        let e = ejes(
            c,
            y,
            w,
            h,
            (k0 as f64 - 0.7, k1 as f64 + 0.7),
            (0.0, ymax.max(1e-9)),
            "x",
            "P(X = x)",
        );
        let ancho_barra = (e.px(1.0) - e.px(0.0)) * 0.7;
        for &(k, p) in &ps {
            let dentro = intervalo.is_some_and(|(a, bb)| {
                a.is_none_or(|a| k >= a - 1e-9) && bb.is_none_or(|bb| k <= bb + 1e-9)
            });
            let (x1, y1) = (e.px(k) - ancho_barra / 2.0, e.py(p));
            let relleno = if dentro && !blanco {
                "fillHighlight"
            } else {
                "fillCell"
            };
            c.r(
                x1,
                y1,
                ancho_barra,
                e.py(0.0) - y1,
                relleno,
                if dentro && !blanco { 1.6 } else { 1.0 },
                0.0,
            );
        }
    } else {
        let ys: Vec<Punto> = (0..=600)
            .map(|i| {
                let x = lo + (hi - lo) * i as f64 / 600.0;
                (x, d.densidad(x))
            })
            .collect();
        let ymax = ys
            .iter()
            .map(|p| p.1)
            .filter(|v| v.is_finite())
            .fold(0.0, f64::max)
            * 1.15;
        let e = ejes(c, y, w, h, (lo, hi), (0.0, ymax.max(1e-9)), "x", "f(x)");
        if let (Some((a, bb)), false) = (intervalo, blanco) {
            let a = a.unwrap_or(lo).max(lo);
            let bb = bb.unwrap_or(hi).min(hi);
            if bb > a {
                let mut d_path = format!("M {:.1} {:.1}", e.px(a), e.py(0.0));
                for i in 0..=200 {
                    let x = a + (bb - a) * i as f64 / 200.0;
                    d_path.push_str(&format!(
                        " L {:.1} {:.1}",
                        e.px(x),
                        e.py(d.densidad(x).min(ymax))
                    ));
                }
                d_path.push_str(&format!(" L {:.1} {:.1} Z", e.px(bb), e.py(0.0)));
                camino(c, &d_path, "fillHighlight", "strokeSub", 1.0);
            }
        }
        e.curva(c, &ys, 0, 2.0, "accentPrimary");
        if bool_opt(b, "showMean", false) {
            let (m, _) = d.media_var();
            e.guia(c, m, 0.0, m, ymax, "5 3", "strokeSub");
            c.t(
                e.px(m),
                e.py(ymax) - 3.0,
                "μ",
                10.0,
                700,
                "textSub",
                "middle",
                MONO,
            );
        }
        if let Some((a, bb)) = intervalo {
            for v in [a, bb].into_iter().flatten() {
                c.t(
                    e.px(v),
                    y + h + 26.0,
                    &fmt(v, 4),
                    9.5,
                    700,
                    "accentPrimary",
                    "middle",
                    MONO,
                );
            }
        }
    }
    let mut yy = y + h + 52.0;
    let (m, var) = d.media_var();
    let mut lineas = vec![format!(
        "E[X] = {} · Var(X) = {}",
        fmt(m, 4),
        if var.is_finite() {
            fmt(var, 4)
        } else {
            "∞".into()
        }
    )];
    if let Some((a, bb)) = intervalo {
        let sim = if d.discreta() { "≤" } else { "<" };
        let desc = match (a, bb) {
            (Some(a), Some(bb)) => format!("P({} {sim} X {sim} {})", fmt(a, 4), fmt(bb, 4)),
            (Some(a), None) => format!(
                "P(X {} {})",
                if d.discreta() { "≥" } else { ">" },
                fmt(a, 4)
            ),
            (None, Some(bb)) => format!("P(X {sim} {})", fmt(bb, 4)),
            (None, None) => "P(−∞ < X < ∞)".into(),
        };
        let p = probabilidad(&d, a, bb);
        lineas.insert(
            0,
            if blanco {
                format!("{desc} = ____")
            } else {
                format!("{desc} = {}", fmt(p, 4))
            },
        );
    }
    let mut ancho = ML + w + 10.0;
    for (k, l) in lineas.iter().enumerate() {
        c.t(
            0.0,
            yy,
            l,
            if k == 0 { 11.5 } else { 10.0 },
            if k == 0 { 700 } else { 500 },
            if k == 0 { "textMain" } else { "textSub" },
            "start",
            MONO,
        );
        ancho = ancho.max(ancho_texto(l, 11.5));
        yy += 17.0;
    }
    Ok((ancho, yy - 8.0))
}

// ---------------------------------------------------------------------------
// venn
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
enum Conj {
    Set(usize),
    Universo,
    Vacio,
    Comp(Box<Conj>),
    Union(Box<Conj>, Box<Conj>),
    Inter(Box<Conj>, Box<Conj>),
    Dif(Box<Conj>, Box<Conj>),
    SimDif(Box<Conj>, Box<Conj>),
}

fn parsear_conj(s: &str, nombres: &[String]) -> Result<Conj, ErrorDiagrama> {
    #[derive(Debug, Clone, PartialEq)]
    enum T {
        Id(String),
        U,
        I,
        D,
        S,
        C,
        No,
        Ab,
        Ce,
    }
    let cs: Vec<char> = s.chars().collect();
    let mut toks = Vec::new();
    let mut i = 0;
    while i < cs.len() {
        let ch = cs[i];
        if ch.is_whitespace() {
            i += 1;
            continue;
        }
        let es_id = |c: char| (c.is_alphanumeric() || c == '_') && !matches!(c, 'ᶜ' | 'Δ');
        if es_id(ch) {
            let ini = i;
            while i < cs.len() && es_id(cs[i]) {
                i += 1;
            }
            let id: String = cs[ini..i].iter().collect();
            toks.push(match id.to_lowercase().as_str() {
                "union" | "or" => T::U,
                "inter" | "and" => T::I,
                _ => T::Id(id),
            });
            continue;
        }
        toks.push(match ch {
            '∪' | '|' | '+' => T::U,
            '∩' | '&' | '·' | '*' => T::I,
            '−' | '-' | '\\' => T::D,
            'Δ' | '△' | '^' | '⊕' => T::S,
            '\'' | 'ᶜ' | '’' => T::C,
            '¬' | '!' | '~' => T::No,
            '(' => T::Ab,
            ')' => T::Ce,
            _ => return Err(err(format!("carácter inesperado {ch:?} en {s:?}"))),
        });
        i += 1;
    }
    struct P<'a> {
        t: Vec<T>,
        i: usize,
        n: &'a [String],
        s: String,
    }
    impl P<'_> {
        fn union(&mut self) -> Result<Conj, ErrorDiagrama> {
            let mut e = self.dif()?;
            loop {
                match self.t.get(self.i) {
                    Some(T::U) => {
                        self.i += 1;
                        e = Conj::Union(Box::new(e), Box::new(self.dif()?));
                    }
                    Some(T::S) => {
                        self.i += 1;
                        e = Conj::SimDif(Box::new(e), Box::new(self.dif()?));
                    }
                    _ => return Ok(e),
                }
            }
        }
        fn dif(&mut self) -> Result<Conj, ErrorDiagrama> {
            let mut e = self.inter()?;
            while self.t.get(self.i) == Some(&T::D) {
                self.i += 1;
                e = Conj::Dif(Box::new(e), Box::new(self.inter()?));
            }
            Ok(e)
        }
        fn inter(&mut self) -> Result<Conj, ErrorDiagrama> {
            let mut e = self.unario()?;
            loop {
                match self.t.get(self.i) {
                    Some(T::I) => {
                        self.i += 1;
                        e = Conj::Inter(Box::new(e), Box::new(self.unario()?));
                    }
                    Some(T::Id(_)) | Some(T::Ab) | Some(T::No) => {
                        e = Conj::Inter(Box::new(e), Box::new(self.unario()?));
                    }
                    _ => return Ok(e),
                }
            }
        }
        fn unario(&mut self) -> Result<Conj, ErrorDiagrama> {
            let tok = self.t.get(self.i).cloned();
            self.i += 1;
            let mut e = match tok {
                Some(T::No) => Conj::Comp(Box::new(self.unario()?)),
                Some(T::Ab) => {
                    let e = self.union()?;
                    if self.t.get(self.i) != Some(&T::Ce) {
                        return Err(err(format!("falta ')' en {:?}", self.s)));
                    }
                    self.i += 1;
                    e
                }
                Some(T::Id(id)) => match self.n.iter().position(|x| *x == id) {
                    Some(k) => Conj::Set(k),
                    None if id == "U" => Conj::Universo,
                    None if id == "Ø" || id == "0" => Conj::Vacio,
                    None => {
                        return Err(err(format!(
                            "conjunto {id:?} desconocido (conjuntos: {})",
                            self.n.join(", ")
                        )))
                    }
                },
                otro => {
                    return Err(err(format!(
                        "expresión de conjuntos mal formada {:?} (llegó {otro:?})",
                        self.s
                    )))
                }
            };
            while self.t.get(self.i) == Some(&T::C) {
                self.i += 1;
                e = Conj::Comp(Box::new(e));
            }
            Ok(e)
        }
    }
    if toks.is_empty() {
        return Err(err("expresión de conjuntos vacía"));
    }
    let mut p = P {
        t: toks,
        i: 0,
        n: nombres,
        s: s.to_string(),
    };
    let e = p.union()?;
    if p.i < p.t.len() {
        return Err(err(format!("sobra texto en {s:?}")));
    }
    Ok(e)
}

fn pertenece(e: &Conj, region: u32) -> bool {
    match e {
        Conj::Set(k) => region >> k & 1 == 1,
        Conj::Universo => true,
        Conj::Vacio => false,
        Conj::Comp(a) => !pertenece(a, region),
        Conj::Union(a, b) => pertenece(a, region) || pertenece(b, region),
        Conj::Inter(a, b) => pertenece(a, region) && pertenece(b, region),
        Conj::Dif(a, b) => pertenece(a, region) && !pertenece(b, region),
        Conj::SimDif(a, b) => pertenece(a, region) != pertenece(b, region),
    }
}

/// Regiones (bitmask de conjuntos) que forman la expresión.
pub fn regiones(expr: &str, nombres: &[String]) -> Result<Vec<u32>, ErrorDiagrama> {
    let e = parsear_conj(expr, nombres)?;
    Ok((0..(1u32 << nombres.len()))
        .filter(|&r| pertenece(&e, r))
        .collect())
}

struct Disposicion {
    centros: Vec<Punto>,
    radio: f64,
    universo: (f64, f64, f64, f64),
    etiquetas: Vec<Punto>,
    puntos: Vec<Punto>, // posición del contenido de cada región (índice = bitmask)
}

fn disposicion(n: usize) -> Disposicion {
    if n == 2 {
        Disposicion {
            centros: vec![(110.0, 100.0), (190.0, 100.0)],
            radio: 70.0,
            universo: (10.0, 14.0, 290.0, 186.0),
            etiquetas: vec![(62.0, 40.0), (238.0, 40.0)],
            puntos: vec![(32.0, 170.0), (80.0, 104.0), (220.0, 104.0), (150.0, 104.0)],
        }
    } else {
        Disposicion {
            centros: vec![(118.0, 92.0), (192.0, 92.0), (155.0, 156.0)],
            radio: 62.0,
            universo: (10.0, 10.0, 290.0, 240.0),
            etiquetas: vec![(66.0, 40.0), (244.0, 40.0), (155.0, 236.0)],
            puntos: vec![
                (30.0, 232.0),
                (92.0, 80.0),
                (218.0, 80.0),
                (155.0, 72.0),
                (155.0, 192.0),
                (118.0, 140.0),
                (192.0, 140.0),
                (155.0, 118.0),
            ],
        }
    }
}

fn clave_region(clave: &str, nombres: &[String]) -> Result<u32, ErrorDiagrama> {
    let t = clave.trim();
    if t.is_empty()
        || t == "U"
        || t == "-"
        || t == "∅"
        || t.eq_ignore_ascii_case("outside")
        || t.eq_ignore_ascii_case("fuera")
    {
        return Ok(0);
    }
    let mut r = 0;
    for parte in t
        .split(|ch: char| ch == ',' || ch == '∩' || ch == '&' || ch.is_whitespace())
        .filter(|s| !s.is_empty())
    {
        let k = nombres
            .iter()
            .position(|n| n == parte)
            .ok_or_else(|| err(format!("región {clave:?}: conjunto {parte:?} desconocido")))?;
        r |= 1 << k;
    }
    Ok(r)
}

pub fn render_venn(c: &mut Lienzo, b: &Bloque) -> Result<Punto, ErrorDiagrama> {
    let nombres = lista_str(b, "sets", true)?;
    let nombres = if nombres.is_empty() {
        vec!["A".to_string(), "B".to_string()]
    } else {
        nombres
    };
    if !(2..=3).contains(&nombres.len()) {
        return Err(err("'sets' tiene 2 o 3 nombres"));
    }
    let n = nombres.len();
    let exprs: Vec<String> = match b.get("shade") {
        None | Some(Value::Null) => Vec::new(),
        Some(Value::Array(a)) => a.iter().map(valor_str).collect(),
        Some(v) => vec![valor_str(v)],
    };
    if exprs.len() > 8 {
        return Err(err("como máximo 8 expresiones en 'shade'"));
    }
    let blanco = bool_opt(b, "blank", false);
    let mut contenido: Vec<Vec<String>> = vec![Vec::new(); 1 << n];
    for clave in ["regions", "counts"] {
        if let Some(Value::Object(m)) = b.get(clave) {
            for (k, v) in m {
                let r = clave_region(k, &nombres)? as usize;
                match v {
                    Value::Array(a) => contenido[r].extend(a.iter().map(valor_str)),
                    otro => contenido[r].push(valor_str(otro)),
                }
            }
        }
    }
    let y0 = encabezado(c, b, "");
    let d = disposicion(n);
    let paneles = exprs.len().max(1);
    let ancho_panel = d.universo.2 + 20.0;
    let mut alto = 0.0_f64;
    for p in 0..paneles {
        let ox = p as f64 * ancho_panel;
        let oy = y0 + 4.0;
        let sombreadas = match exprs.get(p) {
            Some(e) if !blanco => regiones(e, &nombres)?,
            Some(e) => {
                regiones(e, &nombres)?;
                Vec::new()
            }
            None => Vec::new(),
        };
        let (ux, uy, uw, uh) = d.universo;
        let suf = format!("{}-{p}", c.sufijo);
        // Definiciones: recorte (dentro de cada círculo) y máscara (fuera de cada círculo).
        let mut defs = String::from("<defs>");
        for (k, &(cx, cy)) in d.centros.iter().enumerate() {
            defs.push_str(&format!(
                r#"<clipPath id="vin-{suf}-{k}"><circle cx="{:.1}" cy="{:.1}" r="{:.1}"/></clipPath><mask id="vout-{suf}-{k}"><rect x="{:.1}" y="{:.1}" width="{uw:.1}" height="{uh:.1}" fill="white"/><circle cx="{:.1}" cy="{:.1}" r="{:.1}" fill="black"/></mask>"#,
                ox + cx, oy + cy, d.radio, ox + ux, oy + uy, ox + cx, oy + cy, d.radio
            ));
        }
        defs.push_str("</defs>");
        c.add(defs);
        c.add(format!(
            r#"<rect x="{:.1}" y="{:.1}" width="{uw:.1}" height="{uh:.1}" fill="{}" stroke="{}" stroke-width="1.6"/>"#,
            ox + ux,
            oy + uy,
            c.pal.color("fillCell"),
            c.pal.color("strokeMain")
        ));
        let relleno = c.pal.color("fillHighlight").to_string();
        let trama = c.pal.color("accentPrimary").to_string();
        for &r in &sombreadas {
            let mut abre = String::new();
            let mut cierra = String::new();
            for k in 0..n {
                if r >> k & 1 == 1 {
                    abre.push_str(&format!(r#"<g clip-path="url(#vin-{suf}-{k})">"#));
                } else {
                    abre.push_str(&format!(r#"<g mask="url(#vout-{suf}-{k})">"#));
                }
                cierra.push_str("</g>");
            }
            c.add(format!(
                r#"{abre}<rect x="{:.1}" y="{:.1}" width="{uw:.1}" height="{uh:.1}" fill="{relleno}"/><rect x="{:.1}" y="{:.1}" width="{uw:.1}" height="{uh:.1}" fill="{trama}" fill-opacity="0.18"/>{cierra}"#,
                ox + ux, oy + uy, ox + ux, oy + uy
            ));
        }
        for &(cx, cy) in &d.centros {
            circulo(c, ox + cx, oy + cy, d.radio, "none", "strokeMain", 1.8);
        }
        for (k, nombre) in nombres.iter().enumerate() {
            let (x, y) = d.etiquetas[k];
            c.t(
                ox + x,
                oy + y,
                nombre,
                14.0,
                700,
                "textMain",
                "middle",
                SANS,
            );
        }
        c.t(
            ox + ux + uw - 8.0,
            oy + uy + 16.0,
            "U",
            12.0,
            700,
            "textMuted",
            "end",
            SANS,
        );
        for (r, items) in contenido.iter().enumerate() {
            if items.is_empty() {
                continue;
            }
            let (x, y) = d.puntos[r];
            let texto = items.join(", ");
            let x = if r == 0 { ox + x - 18.0 } else { ox + x };
            c.t(
                x,
                oy + y,
                &texto,
                10.5,
                600,
                "textSub",
                if r == 0 { "start" } else { "middle" },
                MONO,
            );
        }
        if let Some(e) = exprs.get(p) {
            let rotulo = if paneles > 1 {
                format!("{}) {e}", (b'a' + p as u8) as char)
            } else {
                e.clone()
            };
            c.t(
                ox + ux + uw / 2.0,
                oy + uy + uh + 22.0,
                &rotulo,
                12.0,
                700,
                "textMain",
                "middle",
                MONO,
            );
        }
        alto = alto.max(oy + uy + uh + if exprs.is_empty() { 8.0 } else { 32.0 });
    }
    Ok((paneles as f64 * ancho_panel - 10.0, alto))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn funciones_especiales() {
        assert!((phi(1.96) - 0.975_002_1).abs() < 1e-6);
        assert!((phi(-1.0) - 0.158_655_25).abs() < 1e-7);
        // t con 10 g.l.: P(T ≤ 2,228) ≈ 0,975.
        assert!((Dist::T(10.0).acumulada(2.228) - 0.975).abs() < 2e-4);
        // χ² con 2 g.l.: F(x) = 1 − e^(−x/2).
        assert!((Dist::Chi2(2.0).acumulada(3.0) - (1.0 - (-1.5f64).exp())).abs() < 1e-10);
        // Binomial(10, 0,5): P(X ≤ 5) = 0,623046875.
        assert!((Dist::Binomial(10, 0.5).acumulada(5.0) - 0.623_046_875).abs() < 1e-9);
        assert!(
            (probabilidad(&Dist::Binomial(10, 0.5), Some(3.0), Some(5.0))
                - (0.623_046_875 - 0.054_687_5))
                .abs()
                < 1e-9
        );
        assert!(
            (probabilidad(&Dist::Poisson(2.0), Some(1.0), None) - (1.0 - (-2.0f64).exp())).abs()
                < 1e-12
        );
        assert!((ln_gamma(5.0) - 24f64.ln()).abs() < 1e-12);
    }

    #[test]
    fn regiones_de_conjuntos() {
        let n: Vec<String> = ["A", "B", "C"].iter().map(|s| s.to_string()).collect();
        assert_eq!(regiones("A ∩ B", &n).unwrap(), vec![3, 7]);
        assert_eq!(regiones("(A ∪ B)'", &n).unwrap(), vec![0, 4]);
        assert_eq!(regiones("A − B − C", &n).unwrap(), vec![1]);
        assert_eq!(regiones("A Δ B", &n).unwrap(), vec![1, 2, 5, 6]);
        assert_eq!(regiones("A B' C", &n).unwrap(), vec![5]);
        assert!(regiones("A ∩ D", &n).is_err());
    }
}
