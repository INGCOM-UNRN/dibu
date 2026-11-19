//! Diagrama de bloques (`block_diagram`): cadena directa, realimentación, perturbación y lazos anidados.

use dibu_core::{ancho_texto, bool_opt, encabezado, str_opt, valor_str, Bloque, ErrorDiagrama, Lienzo, Punto, MONO};
use serde_json::Value;

const ALTO_BLOQUE: f64 = 36.0;
const FLECHA: f64 = 38.0;
const RADIO: f64 = 11.0;
const CARRIL: f64 = 30.0;

struct BloqueNodo {
    texto: String,
    blanco: bool,
    salida: String,
    w: f64,
    arriba: f64,
    abajo: f64,
}

impl BloqueNodo {
    fn parse(dato: &Value, donde: &str) -> Result<Self, ErrorDiagrama> {
        let (texto, blanco, salida) = if let Some(obj) = dato.as_object() {
            (
                obj.get("label").map(valor_str).unwrap_or_default(),
                obj.get("blank").and_then(Value::as_bool).unwrap_or(false),
                obj.get("out").map(valor_str).unwrap_or_default(),
            )
        } else {
            (valor_str(dato), false, String::new())
        };
        if texto.is_empty() && !blanco {
            return Err(ErrorDiagrama::new(format!(
                "{donde}: un bloque necesita label (o blank: true)"
            )));
        }
        let base_t = if texto.is_empty() { "XXXXXX" } else { &texto };
        let w = (ancho_texto(base_t, 11.0) + 24.0).max(64.0);
        Ok(Self {
            texto,
            blanco,
            salida,
            w,
            arriba: ALTO_BLOQUE / 2.0,
            abajo: ALTO_BLOQUE / 2.0,
        })
    }

    fn dibujar(&self, c: &mut Lienzo, x: f64, y: f64) {
        c.r(x, y - ALTO_BLOQUE / 2.0, self.w, ALTO_BLOQUE, "fillCell", 1.6, 2.0);
        if self.blanco {
            c.vacio(x + 12.0, x + self.w - 12.0, y + 6.0);
        } else {
            c.t(x + self.w / 2.0, y + 4.0, &self.texto, 11.0, 600, "textMain", "middle", MONO);
        }
    }
}

fn sumador(c: &mut Lienzo, x: f64, y: f64, signos: &[(&str, &str)], blanco: bool) {
    c.add(format!(
        r##"<circle cx="{x:.1}" cy="{y:.1}" r="{}" fill="#ffffff" stroke="{}" stroke-width="1.6"/>"##,
        dibu_core::fmt_num_clean(RADIO),
        c.pal.stroke_main
    ));
    for &(lado, signo) in signos {
        let (sx, sy) = match lado {
            "izq" => (x - RADIO - 7.0, y - 6.0),
            "abajo" => (x - 11.0, y + RADIO + 11.0),
            _ => (x - 11.0, y - RADIO - 3.0),
        };
        if blanco {
            c.add(format!(
                r#"<rect x="{:.1}" y="{:.1}" width="10" height="11" fill="none" stroke="{}" stroke-width="1" stroke-dasharray="2 2"/>"#,
                sx - 5.0,
                sy - 9.0,
                c.pal.stroke_muted
            ));
        } else {
            let ch = if signo == "-" { "−" } else { "+" };
            c.t(sx, sy, ch, 12.0, 700, "textMain", "middle", MONO);
        }
    }
}

struct PerturbacionNodo {
    at: usize,
    texto: String,
    signo: String,
    bloque: Option<BloqueNodo>,
    w: f64,
    arriba: f64,
    abajo: f64,
}

impl PerturbacionNodo {
    fn parse(dato: &Value, donde: &str) -> Result<Self, ErrorDiagrama> {
        let Some(obj) = dato.as_object() else {
            return Err(ErrorDiagrama::new(format!(
                "{donde}: disturbance necesita at (antes de qué bloque entra)"
            )));
        };
        let Some(at_v) = obj.get("at").and_then(Value::as_u64) else {
            return Err(ErrorDiagrama::new(format!(
                "{donde}: disturbance necesita at (antes de qué bloque entra)"
            )));
        };
        let texto = obj
            .get("label")
            .and_then(Value::as_str)
            .filter(|s| !s.is_empty())
            .unwrap_or("D(s)")
            .to_string();
        let signo = obj.get("sign").and_then(Value::as_str).unwrap_or("+").to_string();
        let bloque = match obj.get("block") {
            Some(v) if !v.is_null() => Some(BloqueNodo::parse(v, &format!("{donde}.block"))?),
            _ => None,
        };
        let arriba = RADIO + 34.0 + if bloque.is_some() { ALTO_BLOQUE + 24.0 } else { 0.0 };
        Ok(Self {
            at: at_v as usize,
            texto,
            signo,
            bloque,
            w: 2.0 * RADIO,
            arriba,
            abajo: RADIO,
        })
    }

    fn dibujar(&self, c: &mut Lienzo, x: f64, y: f64, blanco: bool) {
        let cx = x + RADIO;
        sumador(c, cx, y, &[("izq", "+"), ("arriba", &self.signo)], blanco);
        let tope = y - self.arriba;
        c.t(cx, tope + 10.0, &self.texto, 11.0, 700, "textMain", "middle", MONO);
        if let Some(bl) = &self.bloque {
            let by = tope + 18.0 + ALTO_BLOQUE / 2.0 + 6.0;
            c.linea(cx, tope + 14.0, cx, by - ALTO_BLOQUE / 2.0, "accentPrimary", 1.6, Some("main"), "");
            bl.dibujar(c, cx - bl.w / 2.0, by);
            c.linea(cx, by + ALTO_BLOQUE / 2.0, cx, y - RADIO, "accentPrimary", 1.6, Some("main"), "");
        } else {
            c.linea(cx, tope + 14.0, cx, y - RADIO, "accentPrimary", 1.6, Some("main"), "");
        }
    }
}

enum Elemento {
    Bloque(BloqueNodo),
    Perturbacion(PerturbacionNodo),
    Lazo(Box<LazoNodo>),
}

impl Elemento {
    fn w(&self) -> f64 {
        match self {
            Self::Bloque(b) => b.w,
            Self::Perturbacion(p) => p.w,
            Self::Lazo(l) => l.w,
        }
    }
    fn arriba(&self) -> f64 {
        match self {
            Self::Bloque(b) => b.arriba,
            Self::Perturbacion(p) => p.arriba,
            Self::Lazo(l) => l.arriba,
        }
    }
    fn abajo(&self) -> f64 {
        match self {
            Self::Bloque(b) => b.abajo,
            Self::Perturbacion(p) => p.abajo,
            Self::Lazo(l) => l.abajo,
        }
    }
    fn salida(&self) -> &str {
        match self {
            Self::Bloque(b) => &b.salida,
            _ => "",
        }
    }
    fn es_perturbacion(&self) -> bool {
        matches!(self, Self::Perturbacion(_))
    }
}

struct LazoNodo {
    cerrado: bool,
    signo: String,
    error: String,
    medida: String,
    cadena: Vec<Elemento>,
    realim: Vec<BloqueNodo>,
    blanco: bool,
    inicio: f64,
    w: f64,
    arriba: f64,
    abajo: f64,
    carril: f64,
}

impl LazoNodo {
    fn parse(dato: &Bloque, donde: &str, cerrado: bool) -> Result<Self, ErrorDiagrama> {
        let signo = str_opt(dato, "sign").unwrap_or("-").to_string();
        if signo != "-" && signo != "+" {
            return Err(ErrorDiagrama::new(format!("{donde}.sign: es '-' o '+'")));
        }
        let error = str_opt(dato, "error").unwrap_or("").to_string();
        let medida = str_opt(dato, "measured").unwrap_or("").to_string();
        let Some(directa) = dato.get("forward").and_then(Value::as_array) else {
            return Err(ErrorDiagrama::new(format!(
                "{donde}.forward: tiene que ser una lista de bloques"
            )));
        };
        if directa.is_empty() {
            return Err(ErrorDiagrama::new(format!(
                "{donde}.forward: tiene que ser una lista de bloques"
            )));
        }
        let mut cadena = Vec::with_capacity(directa.len());
        for (i, e) in directa.iter().enumerate() {
            let sub = format!("{donde}.forward[{i}]");
            if let Some(obj) = e.as_object() {
                if let Some(lp) = obj.get("loop").and_then(Value::as_object) {
                    cadena.push(Elemento::Lazo(Box::new(LazoNodo::parse(
                        lp,
                        &format!("{sub}.loop"),
                        true,
                    )?)));
                    continue;
                }
            }
            cadena.push(Elemento::Bloque(BloqueNodo::parse(e, &sub)?));
        }
        if let Some(dist) = dato.get("disturbance").filter(|v| !v.is_null()) {
            let p = PerturbacionNodo::parse(dist, &format!("{donde}.disturbance"))?;
            if p.at > cadena.len() {
                return Err(ErrorDiagrama::new(format!(
                    "{donde}.disturbance.at: va de 0 a {}",
                    cadena.len()
                )));
            }
            let idx = p.at;
            cadena.insert(idx, Elemento::Perturbacion(p));
        }
        let mut realim = Vec::new();
        if let Some(rv) = dato.get("feedback").filter(|v| !v.is_null()) {
            let Some(arr) = rv.as_array() else {
                return Err(ErrorDiagrama::new(format!(
                    "{donde}.feedback: tiene que ser una lista de bloques ([] es unitaria)"
                )));
            };
            for (i, e) in arr.iter().enumerate() {
                realim.push(BloqueNodo::parse(e, &format!("{donde}.feedback[{i}]"))?);
            }
        }
        let blanco = bool_opt(dato, "blankSigns", false);
        let flechas: f64 = (0..cadena.len().saturating_sub(1))
            .map(|i| FLECHA + if cadena[i + 1].es_perturbacion() { 14.0 } else { 0.0 })
            .sum();
        let ancho_cadena: f64 = cadena.iter().map(Elemento::w).sum::<f64>() + flechas;
        let inicio = if cerrado { 2.0 * RADIO + FLECHA } else { 0.0 };
        let w = inicio + ancho_cadena + if cerrado { FLECHA } else { 0.0 };
        let arriba = cadena.iter().map(Elemento::arriba).fold(0.0_f64, f64::max)
            + if error.is_empty() { 0.0 } else { 14.0 };
        let abajo_cadena = cadena.iter().map(Elemento::abajo).fold(0.0_f64, f64::max);
        let (carril, abajo) = if cerrado {
            let extra = if realim.is_empty() { 0.0 } else { ALTO_BLOQUE / 2.0 };
            let c_y = abajo_cadena + CARRIL + extra;
            let ab = c_y + extra + if medida.is_empty() { 6.0 } else { 14.0 };
            (c_y, ab)
        } else {
            (0.0, abajo_cadena)
        };
        Ok(Self {
            cerrado,
            signo,
            error,
            medida,
            cadena,
            realim,
            blanco,
            inicio,
            w,
            arriba,
            abajo,
            carril,
        })
    }

    fn flecha(&self, i: usize) -> f64 {
        FLECHA + if self.cadena[i + 1].es_perturbacion() { 14.0 } else { 0.0 }
    }

    fn dibujar(&self, c: &mut Lienzo, x: f64, y: f64) {
        let cx = x + RADIO;
        if self.cerrado {
            sumador(c, cx, y, &[("izq", "+"), ("abajo", &self.signo)], self.blanco);
            c.linea(x + 2.0 * RADIO, y, x + self.inicio, y, "accentPrimary", 1.6, Some("main"), "");
            if !self.error.is_empty() {
                c.t(x + 2.0 * RADIO + FLECHA / 2.0, y - 8.0, &self.error, 10.0, 600, "textMain", "middle", MONO);
            }
        }
        let mut xi = x + self.inicio;
        for (i, e) in self.cadena.iter().enumerate() {
            match e {
                Elemento::Bloque(b) => b.dibujar(c, xi, y),
                Elemento::Perturbacion(p) => p.dibujar(c, xi, y, self.blanco),
                Elemento::Lazo(l) => l.dibujar(c, xi, y),
            }
            xi += e.w();
            if i + 1 < self.cadena.len() {
                let largo = self.flecha(i);
                c.linea(xi, y, xi + largo, y, "accentPrimary", 1.6, Some("main"), "");
                let sal = e.salida();
                if !sal.is_empty() {
                    c.t(xi + FLECHA / 2.0, y - 8.0, sal, 10.0, 600, "textMain", "middle", MONO);
                }
                xi += largo;
            }
        }
        if !self.cerrado {
            return;
        }
        let toma = x + self.w;
        c.linea(xi, y, toma, y, "accentPrimary", 1.6, None, "");
        if let Some(last) = self.cadena.last() {
            let sal = last.salida();
            if !sal.is_empty() {
                c.t(xi + FLECHA / 2.0, y - 8.0, sal, 10.0, 600, "textMain", "middle", MONO);
            }
        }
        c.punto(toma, y, "accentPrimary", 3.2);
        let yf = y + self.carril;
        if !self.realim.is_empty() {
            let ancho: f64 = self.realim.iter().map(|b| b.w).sum::<f64>()
                + FLECHA * (self.realim.len() - 1) as f64;
            let bx = toma - (toma - cx - ancho) / 2.0 - ancho;
            c.add(format!(
                r#"<path d="M {toma:.1} {y:.1} L {toma:.1} {yf:.1} L {:.1} {yf:.1}" fill="none" stroke="{}" stroke-width="1.6" marker-end="url(#flecha-main-{})"/>"#,
                bx + ancho,
                c.pal.accent_primary,
                c.sufijo
            ));
            let mut xb = bx + ancho;
            for (i, bl) in self.realim.iter().rev().enumerate() {
                xb -= bl.w;
                bl.dibujar(c, xb, yf);
                if i + 1 < self.realim.len() {
                    c.linea(xb, yf, xb - FLECHA, yf, "accentPrimary", 1.6, Some("main"), "");
                    xb -= FLECHA;
                }
            }
            c.ruta(&[(bx, yf), (cx, yf), (cx, y + RADIO)]);
        } else {
            c.ruta(&[(toma, y), (toma, yf), (cx, yf), (cx, y + RADIO)]);
        }
        if !self.medida.is_empty() {
            c.t(cx + 6.0, yf + 14.0, &self.medida, 10.0, 600, "textMain", "start", MONO);
        }
    }
}

pub fn render_block_diagram(c: &mut Lienzo, b: &Bloque) -> Result<Punto, ErrorDiagrama> {
    let cerrado = b.contains_key("feedback");
    let lazo = LazoNodo::parse(b, "block_diagram", cerrado)?;
    let top = encabezado(c, b, "");
    let entrada = str_opt(b, "input").unwrap_or("R(s)");
    let salida = str_opt(b, "output").unwrap_or("Y(s)");
    let margen = (ancho_texto(entrada, 11.0) + 30.0).max(46.0);
    let y = top + lazo.arriba + 12.0;
    let x = margen;
    c.t(4.0, y - 8.0, entrada, 11.0, 700, "textMain", "start", MONO);
    c.linea(4.0, y, x, y, "accentPrimary", 1.6, Some("main"), "");
    lazo.dibujar(c, x, y);
    let fin = x + lazo.w;
    let largo = (ancho_texto(salida, 11.0) + 16.0).max(46.0);
    c.linea(fin, y, fin + largo, y, "accentPrimary", 1.6, Some("main"), "");
    c.t(fin + largo, y - 8.0, salida, 11.0, 700, "textMain", "end", MONO);
    Ok((fin + largo + 4.0, y + lazo.abajo + 6.0))
}
