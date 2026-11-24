//! Utilidades compartidas: lectura de parámetros, tablas con celdas resaltadas o en blanco,
//! campos de bits y cotas de tamaño.
//!
//! Sólo usa la API estable de `Lienzo` (`t`, `r`, `linea`, `punto`, `add`) y lee los colores con
//! `c.pal.color(..)`, así funciona igual con cualquier paleta o tema.

pub use dibu_core::{
    ancho_texto, bool_opt, encabezado, esc, fmt_num_clean, str_opt, valor_str, Bloque,
    ErrorDiagrama, Lienzo, Punto, MONO, SANS,
};
use serde_json::Value;

/// Cotas para que un bloque malicioso o con errores no genere SVG gigantes.
pub const MAX_FILAS: usize = 400;
pub const MAX_ITEMS: usize = 256;

pub fn err(msg: impl Into<String>) -> ErrorDiagrama {
    ErrorDiagrama::new(msg)
}

/// Prefija el mensaje de error con el tipo (`tipo: …`) si todavía no lo tiene.
pub fn envolver(tipo: &str, res: Result<Punto, ErrorDiagrama>) -> Result<Punto, ErrorDiagrama> {
    res.map_err(|e| {
        if e.0.starts_with(&format!("{tipo}: ")) {
            e
        } else {
            ErrorDiagrama::new(format!("{tipo}: {}", e.0))
        }
    })
}

pub fn num(v: &Value) -> Option<f64> {
    match v {
        Value::Number(n) => n.as_f64().filter(|x| x.is_finite()),
        Value::String(s) => s
            .trim()
            .replace(',', ".")
            .parse::<f64>()
            .ok()
            .filter(|x| x.is_finite()),
        _ => None,
    }
}

pub fn num_b(b: &Bloque, clave: &str, defecto: f64) -> f64 {
    b.get(clave).and_then(num).unwrap_or(defecto)
}

/// Número obligatorio dentro de un rango.
pub fn num_req(b: &Bloque, clave: &str, lo: f64, hi: f64) -> Result<f64, ErrorDiagrama> {
    let v = b
        .get(clave)
        .and_then(num)
        .ok_or_else(|| err(format!("falta '{clave}' (número)")))?;
    if v < lo || v > hi {
        return Err(err(format!(
            "'{clave}' tiene que estar entre {} y {}",
            fmt_num_clean(lo),
            fmt_num_clean(hi)
        )));
    }
    Ok(v)
}

pub fn entero_b(
    b: &Bloque,
    clave: &str,
    defecto: i64,
    lo: i64,
    hi: i64,
) -> Result<i64, ErrorDiagrama> {
    match b.get(clave) {
        None | Some(Value::Null) => Ok(defecto),
        Some(v) => {
            let x =
                num(v).ok_or_else(|| err(format!("'{clave}' tiene que ser un número entero")))?;
            if x.fract() != 0.0 {
                return Err(err(format!("'{clave}' tiene que ser un número entero")));
            }
            let x = x as i64;
            if x < lo || x > hi {
                return Err(err(format!("'{clave}' tiene que estar entre {lo} y {hi}")));
            }
            Ok(x)
        }
    }
}

pub fn texto_v(v: &Value) -> String {
    valor_str(v)
}

pub fn str_b(b: &Bloque, clave: &str, defecto: &str) -> String {
    b.get(clave)
        .filter(|v| !v.is_null())
        .map(valor_str)
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| defecto.to_string())
}

/// Lista obligatoria (o vacía si `opcional`), con cota de largo.
pub fn lista<'a>(b: &'a Bloque, clave: &str, opcional: bool) -> Result<&'a [Value], ErrorDiagrama> {
    match b.get(clave) {
        Some(Value::Array(a)) => {
            if a.len() > MAX_ITEMS {
                return Err(err(format!(
                    "'{clave}' admite como máximo {MAX_ITEMS} elementos"
                )));
            }
            Ok(a.as_slice())
        }
        None | Some(Value::Null) if opcional => Ok(&[]),
        None | Some(Value::Null) => Err(err(format!("falta '{clave}' (lista)"))),
        _ => Err(err(format!("'{clave}' tiene que ser una lista"))),
    }
}

pub fn lista_num(b: &Bloque, clave: &str, opcional: bool) -> Result<Vec<f64>, ErrorDiagrama> {
    lista(b, clave, opcional)?
        .iter()
        .map(|v| num(v).ok_or_else(|| err(format!("'{clave}' tiene que contener números"))))
        .collect()
}

pub fn lista_str(b: &Bloque, clave: &str, opcional: bool) -> Result<Vec<String>, ErrorDiagrama> {
    Ok(lista(b, clave, opcional)?.iter().map(valor_str).collect())
}

/// Elige una opción de un conjunto cerrado (sin distinguir mayúsculas).
pub fn opcion(
    b: &Bloque,
    clave: &str,
    opciones: &[&str],
    defecto: &str,
) -> Result<String, ErrorDiagrama> {
    let v = str_b(b, clave, defecto).to_lowercase();
    if opciones.contains(&v.as_str()) {
        Ok(v)
    } else {
        Err(err(format!(
            "'{clave}' no admite {v:?} (opciones: {})",
            opciones.join(", ")
        )))
    }
}

/// Formato numérico corto con coma decimal.
pub fn fmt(x: f64, decimales: usize) -> String {
    if !x.is_finite() {
        return "—".into();
    }
    if (x - x.round()).abs() < 1e-9 {
        return format!("{}", x.round() as i64);
    }
    let s = format!("{x:.decimales$}");
    let s = s.trim_end_matches('0').trim_end_matches('.').to_string();
    s.replace('.', ",")
}

// ---------------------------------------------------------------------------
// Tablas
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Default)]
pub struct Celda {
    pub texto: String,
    pub resaltar: bool,
    pub blanco: bool,
    pub tenue: bool,
    pub negrita: bool,
}

pub fn celda(t: impl Into<String>) -> Celda {
    Celda {
        texto: t.into(),
        ..Default::default()
    }
}

impl Celda {
    pub fn resaltada(mut self, si: bool) -> Self {
        self.resaltar = si;
        self
    }
    pub fn en_blanco(mut self, si: bool) -> Self {
        self.blanco = si;
        self
    }
    pub fn atenuada(mut self, si: bool) -> Self {
        self.tenue = si;
        self
    }
    pub fn fuerte(mut self, si: bool) -> Self {
        self.negrita = si;
        self
    }
}

pub struct OpcTabla<'a> {
    pub fila_h: f64,
    pub min_w: f64,
    pub size: f64,
    pub izquierda: &'a [usize],
    /// Columnas tras las que se dibuja un separador grueso.
    pub separadores: &'a [usize],
}

impl Default for OpcTabla<'_> {
    fn default() -> Self {
        Self {
            fila_h: 24.0,
            min_w: 34.0,
            size: 10.5,
            izquierda: &[],
            separadores: &[],
        }
    }
}

pub fn anchos_tabla(enc: &[String], filas: &[Vec<Celda>], o: &OpcTabla<'_>) -> Vec<f64> {
    let mut anchos: Vec<f64> = enc
        .iter()
        .map(|e| o.min_w.max(ancho_texto(e, o.size) + 16.0))
        .collect();
    for f in filas {
        for (j, cel) in f.iter().enumerate() {
            if j < anchos.len() {
                anchos[j] = anchos[j].max(ancho_texto(&cel.texto, o.size) + 16.0);
            }
        }
    }
    anchos
}

/// Dibuja una tabla y devuelve (ancho, alto).
pub fn tabla(
    c: &mut Lienzo,
    x: f64,
    y: f64,
    enc: &[String],
    filas: &[Vec<Celda>],
    o: &OpcTabla<'_>,
) -> Punto {
    let anchos = anchos_tabla(enc, filas, o);
    tabla_con_anchos(c, x, y, enc, filas, &anchos, o)
}

pub fn tabla_con_anchos(
    c: &mut Lienzo,
    x: f64,
    y: f64,
    enc: &[String],
    filas: &[Vec<Celda>],
    anchos: &[f64],
    o: &OpcTabla<'_>,
) -> Punto {
    let cab_h = if enc.is_empty() { 0.0 } else { o.fila_h + 2.0 };
    let mut cx = x;
    for (j, e) in enc.iter().enumerate() {
        c.r(cx, y, anchos[j], cab_h, "fillHeader", 1.3, 0.0);
        c.t(
            cx + anchos[j] / 2.0,
            y + cab_h / 2.0 + 4.0,
            e,
            o.size,
            700,
            "textMain",
            "middle",
            MONO,
        );
        cx += anchos[j];
    }
    for (i, fila) in filas.iter().enumerate() {
        let fy = y + cab_h + i as f64 * o.fila_h;
        let mut cx = x;
        for (j, &w) in anchos.iter().enumerate() {
            let cel = fila.get(j).cloned().unwrap_or_default();
            let relleno = if cel.resaltar {
                "fillHighlight"
            } else {
                "fillCell"
            };
            c.r(cx, fy, w, o.fila_h, relleno, 1.0, 0.0);
            if cel.blanco {
                c.vacio(cx + 6.0, cx + w - 6.0, fy + o.fila_h - 6.0);
            } else if !cel.texto.is_empty() {
                let color = if cel.tenue { "textMuted" } else { "textMain" };
                let peso = if cel.negrita { 700 } else { 500 };
                if o.izquierda.contains(&j) {
                    c.t(
                        cx + 8.0,
                        fy + o.fila_h / 2.0 + 4.0,
                        &cel.texto,
                        o.size,
                        peso,
                        color,
                        "start",
                        MONO,
                    );
                } else {
                    c.t(
                        cx + w / 2.0,
                        fy + o.fila_h / 2.0 + 4.0,
                        &cel.texto,
                        o.size,
                        peso,
                        color,
                        "middle",
                        MONO,
                    );
                }
            }
            cx += w;
        }
    }
    let ancho: f64 = anchos.iter().sum();
    let alto = cab_h + filas.len() as f64 * o.fila_h;
    let mut sx = x;
    for (j, w) in anchos.iter().enumerate() {
        sx += w;
        if o.separadores.contains(&j) {
            c.linea(sx, y, sx, y + alto, "strokeMain", 2.4, None, "");
        }
    }
    (ancho, alto)
}

// ---------------------------------------------------------------------------
// Campos de bits
// ---------------------------------------------------------------------------

pub struct CampoBits<'a> {
    pub nombre: &'a str,
    pub bits: &'a str,
    pub relleno: &'a str,
    pub blanco: bool,
}

/// Dibuja una fila de bits agrupada en campos con su nombre arriba y el ancho abajo.
/// Devuelve (ancho, alto).
pub fn fila_bits(c: &mut Lienzo, x: f64, y: f64, campos: &[CampoBits<'_>], celda_w: f64) -> Punto {
    let h = 24.0;
    let mut cx = x;
    for campo in campos {
        let n = campo.bits.chars().count().max(1);
        let w = n as f64 * celda_w;
        c.t(
            cx + w / 2.0,
            y + 11.0,
            campo.nombre,
            10.5,
            700,
            "textSub",
            "middle",
            SANS,
        );
        for (i, b) in campo.bits.chars().enumerate() {
            let bx = cx + i as f64 * celda_w;
            c.r(bx, y + 16.0, celda_w, h, campo.relleno, 1.0, 0.0);
            if !campo.blanco {
                c.t(
                    bx + celda_w / 2.0,
                    y + 16.0 + h / 2.0 + 4.0,
                    &b.to_string(),
                    11.0,
                    600,
                    "textMain",
                    "middle",
                    MONO,
                );
            }
        }
        c.add(format!(
            r#"<rect x="{cx:.1}" y="{:.1}" width="{w:.1}" height="{h:.1}" fill="none" stroke="{}" stroke-width="2.2"/>"#,
            y + 16.0,
            c.pal.color("strokeMain")
        ));
        c.t(
            cx + w / 2.0,
            y + 16.0 + h + 13.0,
            &format!("{n} bit{}", if n == 1 { "" } else { "s" }),
            9.5,
            400,
            "textMuted",
            "middle",
            MONO,
        );
        cx += w;
    }
    (cx - x, 16.0 + h + 18.0)
}

/// Llave horizontal bajo un rango con un rótulo centrado.
pub fn llave(c: &mut Lienzo, x1: f64, x2: f64, y: f64, rotulo: &str) {
    let m = (x1 + x2) / 2.0;
    c.add(format!(
        r#"<path d="M {x1:.1} {y:.1} q 0 6 6 6 L {:.1} {:.1} q 6 0 6 6 q 0 -6 6 -6 L {:.1} {:.1} q 6 0 6 -6" fill="none" stroke="{}" stroke-width="1.2"/>"#,
        m - 6.0,
        y + 6.0,
        x2 - 6.0,
        y + 6.0,
        c.pal.color("strokeSub")
    ));
    c.t(m, y + 26.0, rotulo, 10.5, 600, "textSub", "middle", MONO);
}

/// Texto con nota pequeña a la derecha (p. ej. "Promedio: 4,5 ms").
pub fn nota(c: &mut Lienzo, x: f64, y: f64, texto: &str) -> f64 {
    c.t(x, y, texto, 10.5, 500, "textSub", "start", MONO);
    ancho_texto(texto, 10.5)
}

/// Polígono o camino arbitrario con colores de la paleta.
pub fn camino(c: &mut Lienzo, d: &str, relleno: &str, trazo: &str, ancho: f64) {
    let fill = if relleno == "none" {
        "none".to_string()
    } else {
        c.pal.color(relleno).to_string()
    };
    c.add(format!(
        r#"<path d="{d}" fill="{fill}" stroke="{}" stroke-width="{}" stroke-linejoin="round"/>"#,
        c.pal.color(trazo),
        fmt_num_clean(ancho)
    ));
}

pub fn circulo(c: &mut Lienzo, x: f64, y: f64, r: f64, relleno: &str, trazo: &str, ancho: f64) {
    let fill = if relleno == "none" {
        "none".to_string()
    } else {
        c.pal.color(relleno).to_string()
    };
    c.add(format!(
        r#"<circle cx="{x:.1}" cy="{y:.1}" r="{r:.1}" fill="{fill}" stroke="{}" stroke-width="{}"/>"#,
        c.pal.color(trazo),
        fmt_num_clean(ancho)
    ));
}
