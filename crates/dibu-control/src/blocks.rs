//! Diagrama de bloques (`block_diagram`): cadena directa, realimentación, perturbaciones (desde
//! arriba, desde abajo o en el camino de realimentación), varias entradas y lazos anidados.

use dibu_core::{ancho_texto, bool_opt, encabezado, str_opt, valor_str, Bloque, ErrorDiagrama, Lienzo, Punto, MONO};
use serde_json::Value;

const ALTO_BLOQUE: f64 = 36.0;
const FLECHA: f64 = 38.0;
const RADIO: f64 = 11.0;
const CARRIL: f64 = 30.0;
const MAX_PERTURBACIONES: usize = 8;
const MAX_ENTRADAS: usize = 6;

/// El signo de un sumador: `+` o `-`.
fn signo_de(v: Option<&Value>, donde: &str) -> Result<String, ErrorDiagrama> {
    match v.filter(|v| !v.is_null()) {
        None => Ok("+".to_string()),
        Some(Value::String(s)) if s == "+" || s == "-" => Ok(s.clone()),
        Some(_) => Err(ErrorDiagrama::new(format!("{donde}.sign: es '+' o '-'"))),
    }
}

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
        r#"<circle cx="{x:.1}" cy="{y:.1}" r="{}" fill="{}" stroke="{}" stroke-width="1.6"/>"#,
        dibu_core::fmt_num_clean(RADIO),
        c.pal.background,
        c.pal.stroke_main
    ));
    for &(lado, signo) in signos {
        let (sx, sy) = match lado {
            "izq" => (x - RADIO - 7.0, y - 6.0),
            "der" => (x + RADIO + 7.0, y - 6.0),
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

/// Por dónde llega la señal que entra en un sumador propio: una perturbación o una entrada más.
#[derive(Clone, Copy, PartialEq)]
enum Lado {
    Arriba,
    Abajo,
}

/// En qué camino del lazo entra una perturbación.
#[derive(Clone, Copy, PartialEq)]
enum Camino {
    Directo,
    Realimentacion,
}

struct PerturbacionNodo {
    at: usize,
    camino: Camino,
    lado: Lado,
    texto: String,
    signo: String,
    bloque: Option<BloqueNodo>,
    w: f64,
    arriba: f64,
    abajo: f64,
}

impl PerturbacionNodo {
    /// Lo que ocupa la rama que llega al sumador: el rótulo y, si hay, su bloque.
    fn rama(bloque: &Option<BloqueNodo>) -> f64 {
        RADIO + 34.0 + if bloque.is_some() { ALTO_BLOQUE + 24.0 } else { 0.0 }
    }

    fn nuevo(texto: String, signo: String, bloque: Option<BloqueNodo>, lado: Lado) -> Self {
        let rama = Self::rama(&bloque);
        let (arriba, abajo) = if lado == Lado::Arriba { (rama, RADIO) } else { (RADIO, rama) };
        Self { at: 0, camino: Camino::Directo, lado, texto, signo, bloque, w: 2.0 * RADIO, arriba, abajo }
    }

    /// `rotulo` es el rótulo por omisión, que depende de cuántas perturbaciones hay.
    fn parse(dato: &Value, donde: &str, rotulo: &str) -> Result<Self, ErrorDiagrama> {
        let Some(obj) = dato.as_object() else {
            return Err(ErrorDiagrama::new(format!(
                "{donde}: una perturbación es {{at, label, sign, block, from, path}}"
            )));
        };
        let camino = match obj.get("path").filter(|v| !v.is_null()).map(valor_str).as_deref() {
            None | Some("forward") => Camino::Directo,
            Some("feedback") => Camino::Realimentacion,
            Some(otro) => {
                return Err(ErrorDiagrama::new(format!("{donde}.path: es forward o feedback, no '{otro}'")));
            }
        };
        let at = match obj.get("at").filter(|v| !v.is_null()) {
            None if camino == Camino::Realimentacion => 0,
            None => {
                return Err(ErrorDiagrama::new(format!(
                    "{donde}: una perturbación necesita at (antes de qué bloque entra, desde 0)"
                )));
            }
            Some(v) => match v.as_u64() {
                Some(k) if k <= 64 => k as usize,
                _ => {
                    return Err(ErrorDiagrama::new(format!(
                        "{donde}.at: es la posición antes de un bloque (0, 1, 2…), no {}",
                        valor_str(v)
                    )));
                }
            },
        };
        let lado = match obj.get("from").filter(|v| !v.is_null()).map(valor_str).as_deref() {
            None if camino == Camino::Realimentacion => Lado::Abajo,
            None | Some("top") => Lado::Arriba,
            Some("bottom") => Lado::Abajo,
            Some(otro) => {
                return Err(ErrorDiagrama::new(format!("{donde}.from: es top o bottom, no '{otro}'")));
            }
        };
        if camino == Camino::Realimentacion && lado == Lado::Arriba {
            return Err(ErrorDiagrama::new(format!(
                "{donde}.from: en la realimentación la perturbación entra desde abajo (bottom)"
            )));
        }
        let texto = obj
            .get("label")
            .filter(|v| !v.is_null())
            .map(valor_str)
            .filter(|s| !s.is_empty())
            .unwrap_or_else(|| rotulo.to_string());
        let signo = signo_de(obj.get("sign"), donde)?;
        let bloque = match obj.get("block") {
            Some(v) if !v.is_null() => Some(BloqueNodo::parse(v, &format!("{donde}.block"))?),
            _ => None,
        };
        let mut p = Self::nuevo(texto, signo, bloque, lado);
        p.at = at;
        p.camino = camino;
        Ok(p)
    }

    /// `entra` es el lado del sumador por el que llega la señal principal (`izq` o `der`).
    fn dibujar(&self, c: &mut Lienzo, x: f64, y: f64, blanco: bool, entra: &str) {
        let cx = x + RADIO;
        let lado = if self.lado == Lado::Arriba { "arriba" } else { "abajo" };
        sumador(c, cx, y, &[(entra, "+"), (lado, &self.signo)], blanco);
        // `d` va del sumador hacia afuera: -1 hacia arriba y +1 hacia abajo.
        let d = if self.lado == Lado::Arriba { -1.0 } else { 1.0 };
        let largo = Self::rama(&self.bloque);
        let extremo = y + d * largo;
        let y_rotulo = if d < 0.0 { extremo + 10.0 } else { extremo - 2.0 };
        c.t(cx, y_rotulo, &self.texto, 11.0, 700, "textMain", "middle", MONO);
        let desde = extremo - d * 14.0;
        let hasta = y + d * RADIO;
        if let Some(bl) = &self.bloque {
            let by = extremo - d * (18.0 + ALTO_BLOQUE / 2.0 + 6.0);
            c.linea(cx, desde, cx, by + d * ALTO_BLOQUE / 2.0, "accentPrimary", 1.6, Some("main"), "");
            bl.dibujar(c, cx - bl.w / 2.0, by);
            c.linea(cx, by - d * ALTO_BLOQUE / 2.0, cx, hasta, "accentPrimary", 1.6, Some("main"), "");
        } else {
            c.linea(cx, desde, cx, hasta, "accentPrimary", 1.6, Some("main"), "");
        }
    }
}

/// Un elemento del camino de realimentación: un bloque o el sumador de una perturbación.
enum ElementoRealim {
    Bloque(BloqueNodo),
    Perturbacion(PerturbacionNodo),
}

impl ElementoRealim {
    fn w(&self) -> f64 {
        match self {
            Self::Bloque(b) => b.w,
            Self::Perturbacion(p) => p.w,
        }
    }
    fn abajo(&self) -> f64 {
        match self {
            Self::Bloque(b) => b.abajo,
            Self::Perturbacion(p) => p.abajo,
        }
    }
    fn arriba(&self) -> f64 {
        match self {
            Self::Bloque(b) => b.arriba,
            Self::Perturbacion(p) => p.arriba,
        }
    }
}

/// Las perturbaciones de `disturbance`: un objeto o una lista, con rótulos D(s) o D1(s), D2(s)… y
/// N(s) o N1(s)… en la realimentación.
fn leer_perturbaciones(dato: &Bloque, donde: &str) -> Result<Vec<PerturbacionNodo>, ErrorDiagrama> {
    let lista: Vec<&Value> = match dato.get("disturbance").filter(|v| !v.is_null()) {
        None => return Ok(Vec::new()),
        Some(Value::Array(l)) => l.iter().collect(),
        Some(v) => vec![v],
    };
    if lista.len() > MAX_PERTURBACIONES {
        return Err(ErrorDiagrama::new(format!(
            "{donde}.disturbance: hay {} perturbaciones; el máximo es {MAX_PERTURBACIONES}",
            lista.len()
        )));
    }
    let es_realim = |v: &Value| v.get("path").and_then(Value::as_str) == Some("feedback");
    let n_directo = lista.iter().filter(|v| !es_realim(v)).count();
    let n_realim = lista.len() - n_directo;
    let (mut i_d, mut i_n) = (0, 0);
    let mut salida = Vec::with_capacity(lista.len());
    for (i, v) in lista.iter().enumerate() {
        let sub = if dato.get("disturbance").is_some_and(Value::is_array) {
            format!("{donde}.disturbance[{i}]")
        } else {
            format!("{donde}.disturbance")
        };
        let rotulo = if es_realim(v) {
            i_n += 1;
            if n_realim > 1 { format!("N{i_n}(s)") } else { "N(s)".to_string() }
        } else {
            i_d += 1;
            if n_directo > 1 { format!("D{i_d}(s)") } else { "D(s)".to_string() }
        };
        salida.push(PerturbacionNodo::parse(v, &sub, &rotulo)?);
    }
    Ok(salida)
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
    realim: Vec<ElementoRealim>,
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
        let mut bloques_realim = Vec::new();
        if let Some(rv) = dato.get("feedback").filter(|v| !v.is_null()) {
            let Some(arr) = rv.as_array() else {
                return Err(ErrorDiagrama::new(format!(
                    "{donde}.feedback: tiene que ser una lista de bloques ([] es unitaria)"
                )));
            };
            for (i, e) in arr.iter().enumerate() {
                bloques_realim.push(BloqueNodo::parse(e, &format!("{donde}.feedback[{i}]"))?);
            }
        }
        // Cada perturbación va antes del bloque `at` de su camino; varias en el mismo lugar, en el
        // orden en que se escribieron.
        let perturbaciones = leer_perturbaciones(dato, donde)?;
        let n_directo = cadena.len();
        let n_realim = bloques_realim.len();
        let mut antes_directo: Vec<Vec<PerturbacionNodo>> = (0..=n_directo).map(|_| Vec::new()).collect();
        let mut antes_realim: Vec<Vec<PerturbacionNodo>> = (0..=n_realim).map(|_| Vec::new()).collect();
        for p in perturbaciones {
            match p.camino {
                Camino::Directo if p.at > n_directo => {
                    return Err(ErrorDiagrama::new(format!("{donde}.disturbance.at: va de 0 a {n_directo}")));
                }
                Camino::Realimentacion if !cerrado => {
                    return Err(ErrorDiagrama::new(format!(
                        "{donde}.disturbance.path: un lazo abierto no tiene realimentación"
                    )));
                }
                Camino::Realimentacion if p.at > n_realim => {
                    return Err(ErrorDiagrama::new(format!(
                        "{donde}.disturbance.at: en la realimentación va de 0 a {n_realim}"
                    )));
                }
                Camino::Directo => {
                    let k = p.at;
                    antes_directo[k].push(p);
                }
                Camino::Realimentacion => {
                    let k = p.at;
                    antes_realim[k].push(p);
                }
            }
        }
        let mut con_perturbaciones = Vec::with_capacity(cadena.len());
        for (i, e) in cadena.into_iter().enumerate() {
            con_perturbaciones.extend(std::mem::take(&mut antes_directo[i]).into_iter().map(Elemento::Perturbacion));
            con_perturbaciones.push(e);
        }
        con_perturbaciones.extend(std::mem::take(&mut antes_directo[n_directo]).into_iter().map(Elemento::Perturbacion));
        let cadena = con_perturbaciones;
        let mut realim = Vec::new();
        for (i, b) in bloques_realim.into_iter().enumerate() {
            realim.extend(std::mem::take(&mut antes_realim[i]).into_iter().map(ElementoRealim::Perturbacion));
            realim.push(ElementoRealim::Bloque(b));
        }
        realim.extend(std::mem::take(&mut antes_realim[n_realim]).into_iter().map(ElementoRealim::Perturbacion));
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
            let sobre = realim.iter().map(ElementoRealim::arriba).fold(0.0_f64, f64::max);
            let bajo = realim.iter().map(ElementoRealim::abajo).fold(0.0_f64, f64::max);
            let c_y = abajo_cadena + CARRIL + sobre;
            let ab = c_y + bajo + if medida.is_empty() { 6.0 } else { 14.0 };
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
                Elemento::Perturbacion(p) => p.dibujar(c, xi, y, self.blanco, "izq"),
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
            let ancho: f64 = self.realim.iter().map(ElementoRealim::w).sum::<f64>()
                + FLECHA * (self.realim.len() - 1) as f64;
            let bx = toma - (toma - cx - ancho) / 2.0 - ancho;
            c.add(format!(
                r#"<path d="M {toma:.1} {y:.1} L {toma:.1} {yf:.1} L {:.1} {yf:.1}" fill="none" stroke="{}" stroke-width="1.6" marker-end="url(#flecha-main-{})"/>"#,
                bx + ancho,
                c.pal.accent_primary,
                c.sufijo
            ));
            // En el orden de la señal, de la toma (a la derecha) al sumador (a la izquierda).
            let mut xb = bx + ancho;
            for (i, e) in self.realim.iter().enumerate() {
                xb -= e.w();
                match e {
                    ElementoRealim::Bloque(bl) => bl.dibujar(c, xb, yf),
                    ElementoRealim::Perturbacion(p) => p.dibujar(c, xb, yf, self.blanco, "der"),
                }
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

/// Las entradas que se suman a la principal antes del comparador: `inputs`, cada una con `label`,
/// `sign`, `block` y `from`.
fn leer_entradas(b: &Bloque) -> Result<Vec<PerturbacionNodo>, ErrorDiagrama> {
    let lista = match b.get("inputs").filter(|v| !v.is_null()) {
        None => return Ok(Vec::new()),
        Some(Value::Array(l)) => l,
        Some(_) => {
            return Err(ErrorDiagrama::new(
                "block_diagram.inputs: es una lista de entradas {label, sign, block, from}",
            ));
        }
    };
    if lista.len() > MAX_ENTRADAS {
        return Err(ErrorDiagrama::new(format!(
            "block_diagram.inputs: hay {} entradas; el máximo es {MAX_ENTRADAS}",
            lista.len()
        )));
    }
    let mut salida = Vec::with_capacity(lista.len());
    for (i, v) in lista.iter().enumerate() {
        let donde = format!("block_diagram.inputs[{i}]");
        let obj = match v {
            Value::Object(o) => o.clone(),
            Value::String(s) if !s.is_empty() => {
                let mut o = serde_json::Map::new();
                o.insert("label".into(), Value::String(s.clone()));
                o
            }
            _ => return Err(ErrorDiagrama::new(format!("{donde}: es un rótulo o {{label, sign, block, from}}"))),
        };
        if obj.contains_key("at") || obj.contains_key("path") {
            return Err(ErrorDiagrama::new(format!(
                "{donde}: una entrada se suma antes del comparador; para que entre en otro lugar, usar disturbance"
            )));
        }
        let mut obj = obj;
        obj.insert("at".into(), Value::from(0));
        salida.push(PerturbacionNodo::parse(&Value::Object(obj), &donde, &format!("R{}(s)", i + 2))?);
    }
    Ok(salida)
}

pub fn render_block_diagram(c: &mut Lienzo, b: &Bloque) -> Result<Punto, ErrorDiagrama> {
    let cerrado = b.contains_key("feedback");
    let lazo = LazoNodo::parse(b, "block_diagram", cerrado)?;
    let entradas = leer_entradas(b)?;
    let top = encabezado(c, b, "");
    let entrada = str_opt(b, "input").unwrap_or("R(s)");
    let salida = str_opt(b, "output").unwrap_or("Y(s)");
    let margen = (ancho_texto(entrada, 11.0) + 30.0).max(46.0);
    // Las entradas extra van en serie antes del lazo, cada una con su sumador.
    let ancho_entradas: f64 = entradas.iter().map(|p| p.w + FLECHA + 14.0).sum();
    let arriba = entradas.iter().map(|p| p.arriba).fold(lazo.arriba, f64::max);
    let abajo = entradas.iter().map(|p| p.abajo).fold(lazo.abajo, f64::max);
    let y = top + arriba + 12.0;
    let mut x = margen;
    c.t(4.0, y - 8.0, entrada, 11.0, 700, "textMain", "start", MONO);
    c.linea(4.0, y, x, y, "accentPrimary", 1.6, Some("main"), "");
    for p in &entradas {
        p.dibujar(c, x, y, lazo.blanco, "izq");
        c.linea(x + p.w, y, x + p.w + FLECHA + 14.0, y, "accentPrimary", 1.6, Some("main"), "");
        x += p.w + FLECHA + 14.0;
    }
    debug_assert!((x - margen - ancho_entradas).abs() < 1e-6);
    lazo.dibujar(c, x, y);
    let fin = x + lazo.w;
    let largo = (ancho_texto(salida, 11.0) + 16.0).max(46.0);
    c.linea(fin, y, fin + largo, y, "accentPrimary", 1.6, Some("main"), "");
    c.t(fin + largo, y - 8.0, salida, 11.0, 700, "textMain", "end", MONO);
    Ok((fin + largo + 4.0, y + abajo + 6.0))
}

#[cfg(test)]
mod tests {
    use dibu_core::Registro;
    use serde_json::{json, Value};

    fn svg(v: Value) -> Result<String, dibu_core::ErrorDiagrama> {
        let mut reg = Registro::new();
        crate::registrar(&mut reg);
        reg.renderizar(v.as_object().unwrap(), true)
    }

    fn sumadores(s: &str) -> usize {
        s.matches("<circle").count() - s.matches(r#"r="3.2""#).count()
    }

    #[test]
    fn varias_perturbaciones_arriba_abajo_y_en_la_realimentacion() {
        let s = svg(json!({
            "type": "block_diagram",
            "forward": ["Gc", "Gp"],
            "disturbance": [{"at": 1}, {"at": 2, "from": "bottom", "block": "Gd"}, {"path": "feedback"}],
            "feedback": ["H"]
        }))
        .unwrap();
        // Rótulos por omisión numerados cuando hay varias del mismo camino.
        assert!(s.contains(">D1(s)<") && s.contains(">D2(s)<") && s.contains(">N(s)<") && s.contains(">Gd<"));
        let una = svg(json!({"type": "block_diagram", "forward": ["G"], "disturbance": {"at": 1}, "feedback": []})).unwrap();
        assert!(una.contains(">D(s)<"));
        // La de abajo y la del sensor bajan el carril de la realimentación respecto de la versión sin ellas.
        let sin = svg(json!({"type": "block_diagram", "forward": ["Gc", "Gp"], "feedback": ["H"]})).unwrap();
        let alto = |s: &str| -> f64 {
            let i = s.find("height=\"").unwrap() + 8;
            s[i..i + s[i..].find('"').unwrap()].parse().unwrap()
        };
        assert!(alto(&s) > alto(&sin) + 40.0);
    }

    #[test]
    fn varias_en_el_mismo_lugar_y_entradas_antes_del_comparador() {
        let s = svg(json!({
            "type": "block_diagram",
            "forward": ["G"],
            "disturbance": [{"at": 0, "label": "A"}, {"at": 0, "label": "B", "sign": "-"}],
            "inputs": ["R2(s)", {"label": "V", "block": "Gff", "from": "bottom", "sign": "-"}],
            "feedback": []
        }))
        .unwrap();
        // Comparador, dos perturbaciones y dos entradas.
        assert_eq!(sumadores(&s), 5);
        assert!(s.find(">R2(s)<").unwrap() < s.find(">A<").unwrap(), "las entradas van antes del lazo");
        assert!(s.contains(">−<"));
    }

    #[test]
    fn errores_de_perturbaciones_y_entradas() {
        let casos = [
            (json!({"type": "block_diagram", "forward": ["G"], "disturbance": {"at": 2}}), "va de 0 a 1"),
            (json!({"type": "block_diagram", "forward": ["G"], "disturbance": {"label": "D"}}), "necesita at"),
            (json!({"type": "block_diagram", "forward": ["G"], "disturbance": {"at": -1}}), "no -1"),
            (json!({"type": "block_diagram", "forward": ["G"], "disturbance": {"at": 0, "sign": "x"}}), "es '+' o '-'"),
            (json!({"type": "block_diagram", "forward": ["G"], "disturbance": {"at": 0, "from": "left"}}), "top o bottom"),
            (json!({"type": "block_diagram", "forward": ["G"], "disturbance": {"path": "feedback"}}), "lazo abierto"),
            (json!({"type": "block_diagram", "forward": ["G"], "feedback": ["H"], "disturbance": {"path": "feedback", "at": 2}}), "de 0 a 1"),
            (json!({"type": "block_diagram", "forward": ["G"], "feedback": [], "disturbance": {"path": "feedback", "from": "top"}}), "desde abajo"),
            (json!({"type": "block_diagram", "forward": ["G"], "disturbance": vec![json!({"at": 0}); 9]}), "el máximo es 8"),
            (json!({"type": "block_diagram", "forward": ["G"], "inputs": "R2"}), "es una lista"),
            (json!({"type": "block_diagram", "forward": ["G"], "inputs": [{"label": "R2", "at": 1}]}), "usar disturbance"),
            (json!({"type": "block_diagram", "forward": ["G"], "inputs": vec![json!("R"); 7]}), "el máximo es 6"),
        ];
        for (b, esperado) in casos {
            let e = svg(b.clone()).unwrap_err().0;
            assert!(e.contains(esperado), "{b}: {e}");
        }
    }
}
