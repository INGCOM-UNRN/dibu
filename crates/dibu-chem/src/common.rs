//! Helpers compartidos de fórmulas químicas, primitivas SVG, geometrías TREPEV y extractores
//! seguros YAML/JSON para `dibu-chem`.

use crate::chem;
use dibu_core::{esc, fmt_num_clean, Bloque, ErrorDiagrama, Lienzo, SANS};
use serde_json::Value;
use std::fmt::Write as _;

// --- Fórmulas químicas y texto con sub/superíndices ------------------------------------------------------

fn es_carga(ch: char) -> bool {
    matches!(
        ch,
        '0'..='9' | '+' | '-' | '−' | '*' | '•' | '·' | '‡'
    )
}

fn es_sub_simple(ch: char) -> bool {
    ch.is_ascii_alphanumeric() || matches!(ch, 'α' | 'β' | 'γ')
}

fn normalizar_signo(ch: char) -> char {
    if ch == '-' {
        '−'
    } else {
        ch
    }
}

fn poner_parte(partes: &mut Vec<(String, &'static str)>, ch: char, modo: &'static str) {
    if let Some(ultima) = partes.last_mut() {
        if ultima.1 == modo {
            ultima.0.push(ch);
            return;
        }
    }
    let mut s = String::new();
    s.push(ch);
    partes.push((s, modo));
}

/// Convierte una fórmula en segmentos `(texto, modo)` donde `modo` es `"n"`, `"sub"` o `"sup"`.
/// Ej.: `"SO4^2-"` → `[("SO", "n"), ("4", "sub"), ("2−", "sup")]`.
pub fn partes_formula(texto: &str) -> Vec<(String, &'static str)> {
    let chars: Vec<char> = texto.chars().collect();
    let mut partes: Vec<(String, &'static str)> = Vec::new();
    let mut i = 0usize;
    let mut previo = '\0';

    while i < chars.len() {
        let ch = chars[i];
        if ch == '\\' && i + 1 < chars.len() {
            poner_parte(&mut partes, chars[i + 1], "n");
            previo = ' ';
            i += 2;
            continue;
        }
        if ch == '^' || ch == '_' {
            let modo = if ch == '^' { "sup" } else { "sub" };
            i += 1;
            if i < chars.len() && chars[i] == '{' {
                let fin = chars[i..]
                    .iter()
                    .position(|&c| c == '}')
                    .map_or(chars.len(), |rel| i + rel);
                for &x in &chars[i + 1..fin] {
                    poner_parte(&mut partes, normalizar_signo(x), modo);
                }
                i = if fin < chars.len() {
                    fin + 1
                } else {
                    chars.len()
                };
            } else {
                while i < chars.len()
                    && if modo == "sup" {
                        es_carga(chars[i])
                    } else {
                        es_sub_simple(chars[i])
                    }
                {
                    poner_parte(&mut partes, normalizar_signo(chars[i]), modo);
                    i += 1;
                }
            }
            previo = 'x';
            continue;
        }
        if ch.is_ascii_digit()
            && (previo.is_alphabetic() || previo == ')' || previo == ']')
            && previo != '\0'
            && previo != ' '
        {
            poner_parte(&mut partes, ch, "sub");
            i += 1;
            while i < chars.len() && chars[i].is_ascii_digit() {
                poner_parte(&mut partes, chars[i], "sub");
                i += 1;
            }
            previo = '0';
            continue;
        }
        poner_parte(&mut partes, ch, "n");
        previo = ch;
        i += 1;
    }
    partes
}

pub fn ancho_formula(texto: &str, size: f64) -> f64 {
    partes_formula(texto)
        .into_iter()
        .map(|(t, m)| t.chars().count() as f64 * size * if m == "n" { 0.66 } else { 0.48 })
        .sum()
}

#[allow(clippy::too_many_arguments)]
pub fn tq(
    c: &mut Lienzo,
    x: f64,
    y: f64,
    texto: &str,
    size: f64,
    peso: u16,
    color: &str,
    ancla: &str,
    fuente: &str,
    fondo: bool,
) -> f64 {
    let ancho = ancho_formula(texto, size);
    if fondo {
        let x0 = match ancla {
            "start" => x,
            "middle" => x - ancho / 2.0,
            _ => x - ancho,
        };
        c.add(format!(
            r#"<rect x="{:.1}" y="{:.1}" width="{:.1}" height="{:.1}" fill="{}" opacity="0.88"/>"#,
            x0 - 2.0,
            y - size,
            ancho + 4.0,
            size * 1.45,
            c.pal.fill_cell
        ));
    }
    let mut piezas = String::new();
    let mut desplazado = 0.0_f64;
    let mut pendiente = 0usize;
    for (t, modo) in partes_formula(texto) {
        let t_chars = t.chars().count();
        let sin_izq = t.trim_start_matches(' ');
        let sin_izq_chars = sin_izq.chars().count();
        pendiente += t_chars - sin_izq_chars;
        let rec = sin_izq.trim_end_matches(' ');
        let rec_chars = rec.chars().count();
        let finales = sin_izq_chars - rec_chars;
        if rec.is_empty() {
            continue;
        }
        let objetivo = match modo {
            "sub" => size * 0.3,
            "sup" => -size * 0.4,
            _ => 0.0,
        };
        let mut dy = objetivo - desplazado;
        if dy == 0.0 {
            dy = 0.0;
        }
        desplazado = objetivo;
        let tamano = if modo == "n" { size } else { size * 0.72 };
        let dx = if pendiente > 0 {
            format!(r#" dx="{:.1}""#, pendiente as f64 * size * 0.32)
        } else {
            String::new()
        };
        pendiente = finales;
        let contenido = esc(&rec.replace(' ', "\u{00A0}"));
        let _ = write!(
            &mut piezas,
            r#"<tspan{dx} dy="{dy:.1}" font-size="{tamano:.1}">{contenido}</tspan>"#
        );
    }
    c.add(format!(
        r#"<text x="{x:.1}" y="{y:.1}" text-anchor="{ancla}" font-family="{fuente}" font-size="{}" font-weight="{peso}" fill="{}">{piezas}</text>"#,
        fmt_num_clean(size),
        c.pal.color(color)
    ));
    ancho
}

// --- Primitivas SVG y formateo compartidos ---------------------------------------------------------------

pub fn encabezado_q(c: &mut Lienzo, bloque: &Bloque) -> f64 {
    let titulo = opt_texto(bloque, "title");
    let subtitulo = opt_texto(bloque, "subtitle");
    if titulo.is_none() && subtitulo.is_none() {
        return 0.0;
    }
    if let Some(ref t) = titulo {
        c.ancho_min = c
            .ancho_min
            .max(tq(c, 0.0, 16.0, t, 14.0, 700, "textMain", "start", SANS, false) + 4.0);
    }
    if let Some(ref s) = subtitulo {
        let y = if titulo.is_some() { 32.0 } else { 14.0 };
        c.ancho_min = c.ancho_min.max(
            tq(
                c,
                0.0,
                y,
                s,
                10.5,
                400,
                "textMuted",
                "start",
                SANS,
                false,
            ) + 4.0,
        );
    }
    if titulo.is_some() {
        40.0
    } else {
        22.0
    }
}

pub fn caja_vacia(c: &mut Lienzo, x: f64, y: f64, w: f64, h: f64) {
    c.add(format!(
        r#"<rect x="{:.1}" y="{:.1}" width="{:.1}" height="{:.1}" rx="2" fill="none" stroke="{}" stroke-width="1" stroke-dasharray="3 2"/>"#,
        x - w / 2.0,
        y - h / 2.0,
        w,
        h,
        c.pal.stroke_muted
    ));
}

pub fn flecha_doble(c: &mut Lienzo, x: f64, y1: f64, y2: f64, color: &str) {
    c.add(format!(
        r#"<line x1="{x:.1}" y1="{y1:.1}" x2="{x:.1}" y2="{y2:.1}" stroke="{}" stroke-width="1.2" marker-start="url(#flecha-sub-{})" marker-end="url(#flecha-sub-{})"/>"#,
        c.pal.color(color),
        c.sufijo,
        c.sufijo
    ));
}

pub fn incrustar(c: &mut Lienzo, sub: &Lienzo, x: f64, y: f64) {
    c.add(format!(r#"<g transform="translate({x:.1} {y:.1})">"#));
    c.partes.extend(sub.partes.iter().cloned());
    c.add("</g>");
}

#[allow(clippy::too_many_arguments)]
pub fn rotulo_q(
    c: &mut Lienzo,
    x: f64,
    y: f64,
    nombre: &str,
    valor: Option<&str>,
    blanco: bool,
    ancla: &str,
    size: f64,
) -> f64 {
    if blanco {
        let texto = format!("{nombre} =");
        let w_txt = ancho_formula(&texto, size);
        let ancho = w_txt + 50.0;
        let x0 = match ancla {
            "start" => x,
            "middle" => x - ancho / 2.0,
            _ => x - ancho,
        };
        tq(c, x0, y, &texto, size, 700, "textMain", "start", SANS, true);
        c.vacio(x0 + w_txt + 4.0, x0 + ancho, y + 2.0);
        return ancho;
    }
    let texto = format!("{nombre} = {}", valor.unwrap_or(""));
    tq(c, x, y, &texto, size, 700, "textMain", ancla, SANS, true)
}

pub fn coma(x: f64, decimales: usize) -> String {
    format!("{x:.decimales$}").replace('.', ",")
}

pub fn leyenda_q(c: &mut Lienzo, x: f64, y: f64, etiquetas: &[String]) -> f64 {
    if etiquetas.is_empty() {
        return 0.0;
    }
    let max_w = etiquetas
        .iter()
        .map(|e| ancho_formula(e, 10.0))
        .fold(0.0_f64, f64::max);
    let ancho = 40.0 + max_w;
    c.add(format!(
        r#"<rect x="{:.1}" y="{:.1}" width="{:.1}" height="{:.1}" fill="{}" opacity="0.9"/>"#,
        x - 4.0,
        y - 9.0,
        ancho,
        etiquetas.len() as f64 * 16.0 + 4.0,
        c.pal.fill_cell
    ));
    dibu_plot::leyenda(c, x, y, etiquetas)
}

pub fn flechita(c: &mut Lienzo, x: f64, y: f64, arriba: bool, alto: f64) {
    let trazo = c.pal.stroke_main;
    let (y1, y2) = if arriba {
        (y + alto / 2.0, y - alto / 2.0)
    } else {
        (y - alto / 2.0, y + alto / 2.0)
    };
    let signo = if arriba { -1.0 } else { 1.0 };
    c.add(format!(
        r#"<line x1="{x:.1}" y1="{y1:.1}" x2="{x:.1}" y2="{y2:.1}" stroke="{trazo}" stroke-width="1.4"/><path d="M {:.1} {:.1} L {x:.1} {y2:.1} L {:.1} {:.1}" fill="none" stroke="{trazo}" stroke-width="1.4"/>"#,
        x - 3.0,
        y2 - signo * 4.0,
        x + 3.0,
        y2 - signo * 4.0
    ));
}

pub fn cajas(
    c: &mut Lienzo,
    x: f64,
    y: f64,
    electrones: usize,
    n_cajas: usize,
    lado: f64,
    vacio: bool,
) -> f64 {
    let llenas = if vacio {
        vec![0u8; n_cajas]
    } else {
        chem::llenar_cajas(electrones, n_cajas)
    };
    for (i, &n) in llenas.iter().enumerate() {
        let bx = x + i as f64 * lado;
        c.add(format!(
            r#"<rect x="{bx:.1}" y="{y:.1}" width="{lado:.1}" height="{lado:.1}" fill="{}" stroke="{}" stroke-width="1.2"/>"#,
            c.pal.fill_cell, c.pal.stroke_main
        ));
        if n >= 1 {
            let x_f = bx + if n == 2 { lado / 2.0 - 4.0 } else { lado / 2.0 };
            flechita(c, x_f, y + lado / 2.0, true, 14.0);
        }
        if n == 2 {
            flechita(c, bx + lado / 2.0 + 4.0, y + lado / 2.0, false, 14.0);
        }
    }
    n_cajas as f64 * lado
}

// --- Geometrías TREPEV -----------------------------------------------------------------------------------

#[derive(Debug, Clone, Copy)]
pub struct Geometria {
    pub clave: &'static str,
    pub axe: &'static str,
    pub nombre: &'static str,
    pub angulo: &'static str,
    pub hibridacion: &'static str,
    pub ligandos: &'static [(f64, &'static str)],
    pub pares_libres: &'static [f64],
    pub par_angulo: (usize, usize),
}

pub const GEOMETRIAS: &[Geometria] = &[
    Geometria {
        clave: "linear",
        axe: "AX2",
        nombre: "lineal",
        angulo: "180°",
        hibridacion: "sp",
        ligandos: &[(180.0, "p"), (0.0, "p")],
        pares_libres: &[],
        par_angulo: (0, 1),
    },
    Geometria {
        clave: "trigonal_planar",
        axe: "AX3",
        nombre: "trigonal plana",
        angulo: "120°",
        hibridacion: "sp²",
        ligandos: &[(90.0, "p"), (210.0, "p"), (330.0, "p")],
        pares_libres: &[],
        par_angulo: (1, 2),
    },
    Geometria {
        clave: "bent_trigonal",
        axe: "AX2E",
        nombre: "angular",
        angulo: "≈ 117°",
        hibridacion: "sp²",
        ligandos: &[(210.0, "p"), (330.0, "p")],
        pares_libres: &[90.0],
        par_angulo: (0, 1),
    },
    Geometria {
        clave: "tetrahedral",
        axe: "AX4",
        nombre: "tetraédrica",
        angulo: "109,5°",
        hibridacion: "sp³",
        ligandos: &[(90.0, "p"), (210.0, "p"), (330.0, "w"), (15.0, "h")],
        pares_libres: &[],
        par_angulo: (0, 1),
    },
    Geometria {
        clave: "trigonal_pyramidal",
        axe: "AX3E",
        nombre: "piramidal trigonal",
        angulo: "≈ 107°",
        hibridacion: "sp³",
        ligandos: &[(210.0, "p"), (330.0, "w"), (290.0, "h")],
        pares_libres: &[90.0],
        par_angulo: (0, 1),
    },
    Geometria {
        clave: "bent",
        axe: "AX2E2",
        nombre: "angular",
        angulo: "≈ 104,5°",
        hibridacion: "sp³",
        ligandos: &[(215.0, "p"), (325.0, "p")],
        pares_libres: &[60.0, 120.0],
        par_angulo: (0, 1),
    },
    Geometria {
        clave: "trigonal_bipyramidal",
        axe: "AX5",
        nombre: "bipirámide trigonal",
        angulo: "90° y 120°",
        hibridacion: "sp³d",
        ligandos: &[
            (90.0, "p"),
            (270.0, "p"),
            (180.0, "p"),
            (330.0, "w"),
            (30.0, "h"),
        ],
        pares_libres: &[],
        par_angulo: (0, 2),
    },
    Geometria {
        clave: "seesaw",
        axe: "AX4E",
        nombre: "balancín",
        angulo: "< 90° y < 120°",
        hibridacion: "sp³d",
        ligandos: &[(90.0, "p"), (270.0, "p"), (330.0, "w"), (30.0, "h")],
        pares_libres: &[180.0],
        par_angulo: (1, 2),
    },
    Geometria {
        clave: "t_shaped",
        axe: "AX3E2",
        nombre: "forma de T",
        angulo: "< 90°",
        hibridacion: "sp³d",
        ligandos: &[(90.0, "p"), (270.0, "p"), (0.0, "p")],
        pares_libres: &[210.0, 150.0],
        par_angulo: (0, 2),
    },
    Geometria {
        clave: "linear_tbp",
        axe: "AX2E3",
        nombre: "lineal",
        angulo: "180°",
        hibridacion: "sp³d",
        ligandos: &[(90.0, "p"), (270.0, "p")],
        pares_libres: &[180.0, 330.0, 30.0],
        par_angulo: (0, 1),
    },
    Geometria {
        clave: "octahedral",
        axe: "AX6",
        nombre: "octaédrica",
        angulo: "90°",
        hibridacion: "sp³d²",
        ligandos: &[
            (90.0, "p"),
            (270.0, "p"),
            (180.0, "p"),
            (0.0, "p"),
            (225.0, "w"),
            (45.0, "h"),
        ],
        pares_libres: &[],
        par_angulo: (0, 2),
    },
    Geometria {
        clave: "square_pyramidal",
        axe: "AX5E",
        nombre: "pirámide cuadrada",
        angulo: "< 90°",
        hibridacion: "sp³d²",
        ligandos: &[
            (90.0, "p"),
            (180.0, "p"),
            (0.0, "p"),
            (225.0, "w"),
            (45.0, "h"),
        ],
        pares_libres: &[270.0],
        par_angulo: (0, 1),
    },
    Geometria {
        clave: "square_planar",
        axe: "AX4E2",
        nombre: "cuadrada plana",
        angulo: "90°",
        hibridacion: "sp³d²",
        ligandos: &[(180.0, "p"), (0.0, "p"), (225.0, "w"), (45.0, "h")],
        pares_libres: &[90.0, 270.0],
        par_angulo: (0, 2),
    },
];

pub fn geometria(clave: &str) -> Option<&'static Geometria> {
    GEOMETRIAS.iter().find(|g| g.clave == clave)
}

pub fn por_pares(ligandos: usize, lone_pairs: usize) -> Option<&'static str> {
    match (ligandos, lone_pairs) {
        (2, 0) => Some("linear"),
        (3, 0) => Some("trigonal_planar"),
        (2, 1) => Some("bent_trigonal"),
        (4, 0) => Some("tetrahedral"),
        (3, 1) => Some("trigonal_pyramidal"),
        (2, 2) => Some("bent"),
        (5, 0) => Some("trigonal_bipyramidal"),
        (4, 1) => Some("seesaw"),
        (3, 2) => Some("t_shaped"),
        (2, 3) => Some("linear_tbp"),
        (6, 0) => Some("octahedral"),
        (5, 1) => Some("square_pyramidal"),
        (4, 2) => Some("square_planar"),
        _ => None,
    }
}

// --- Helpers compartidos de inorgánica / general ---------------------------------------------------------

pub fn romano(mut n: i64) -> String {
    if n <= 0 || n > 39 {
        return n.to_string();
    }
    let mut salida = String::new();
    for (v, s) in [(10, "X"), (9, "IX"), (5, "V"), (4, "IV"), (1, "I")] {
        while n >= v {
            salida.push_str(s);
            n -= v;
        }
    }
    salida
}

pub fn romano_signo(n: f64) -> String {
    let entero = n.round();
    if (n - entero).abs() > 1e-9 {
        dibu_plot::fmt_g(Some(n), 3)
    } else {
        let e = entero as i64;
        if e > 0 {
            format!("+{e}")
        } else {
            format!("{e}")
        }
    }
}

pub fn carga_sup(carga: i64) -> String {
    if carga == 0 {
        return String::new();
    }
    let abs_c = carga.unsigned_abs();
    let mag = if abs_c > 1 {
        abs_c.to_string()
    } else {
        String::new()
    };
    let signo = if carga > 0 { '+' } else { '-' };
    format!("^{mag}{signo}")
}

// --- Extractores seguros YAML/JSON y envoltorio de errores -----------------------------------------------

pub fn num(val: &Value, donde: &str) -> Result<f64, ErrorDiagrama> {
    let x = match val {
        Value::Number(n) => n
            .as_f64()
            .ok_or_else(|| ErrorDiagrama::new(format!("{donde}: {val} tiene que ser un número")))?,
        Value::String(s) => s
            .trim()
            .parse::<f64>()
            .map_err(|_| ErrorDiagrama::new(format!("{donde}: '{s}' tiene que ser un número")))?,
        _ => {
            return Err(ErrorDiagrama::new(format!(
                "{donde}: {val} tiene que ser un número"
            )));
        }
    };
    if !x.is_finite() {
        return Err(ErrorDiagrama::new(format!(
            "{donde}: tiene que ser un número finito"
        )));
    }
    Ok(x)
}

pub fn opt_num(b: &Bloque, clave: &str, donde: &str) -> Result<Option<f64>, ErrorDiagrama> {
    match b.get(clave) {
        None | Some(Value::Null) => Ok(None),
        Some(v) => num(v, donde).map(Some),
    }
}

pub fn entero(val: &Value, donde: &str) -> Result<i64, ErrorDiagrama> {
    match val {
        Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                Ok(i)
            } else if let Some(u) = n.as_u64() {
                i64::try_from(u)
                    .map_err(|_| ErrorDiagrama::new(format!("{donde}: {val} fuera de rango entero")))
            } else if let Some(f) = n.as_f64() {
                if f.is_finite()
                    && (f - f.round()).abs() < 1e-12
                    && f >= i64::MIN as f64
                    && f <= i64::MAX as f64
                {
                    Ok(f.round() as i64)
                } else {
                    Err(ErrorDiagrama::new(format!(
                        "{donde}: {val} tiene que ser un entero"
                    )))
                }
            } else {
                Err(ErrorDiagrama::new(format!(
                    "{donde}: {val} tiene que ser un entero"
                )))
            }
        }
        Value::String(s) => s
            .trim()
            .parse::<i64>()
            .map_err(|_| ErrorDiagrama::new(format!("{donde}: '{s}' tiene que ser un entero"))),
        _ => Err(ErrorDiagrama::new(format!(
            "{donde}: {val} tiene que ser un entero"
        ))),
    }
}

pub fn cadena(val: &Value, donde: &str) -> Result<String, ErrorDiagrama> {
    match val {
        Value::String(s) => Ok(s.clone()),
        Value::Number(n) => Ok(n.to_string()),
        _ => Err(ErrorDiagrama::new(format!(
            "{donde}: {val} tiene que ser un texto"
        ))),
    }
}

pub fn opt_texto(b: &Bloque, clave: &str) -> Option<String> {
    match b.get(clave)? {
        Value::Null | Value::Bool(false) => None,
        Value::Bool(true) => Some("true".to_string()),
        Value::String(s) if s.is_empty() => None,
        Value::String(s) => Some(s.clone()),
        Value::Number(n) => Some(n.to_string()),
        Value::Array(a) if a.is_empty() => None,
        Value::Object(m) if m.is_empty() => None,
        otro => Some(otro.to_string()),
    }
}

pub fn lista<'a>(val: &'a Value, donde: &str) -> Result<&'a Vec<Value>, ErrorDiagrama> {
    val.as_array()
        .ok_or_else(|| ErrorDiagrama::new(format!("{donde}: tiene que ser una lista")))
}

pub fn opt_lista<'a>(
    b: &'a Bloque,
    clave: &str,
    donde: &str,
) -> Result<Option<&'a Vec<Value>>, ErrorDiagrama> {
    match b.get(clave) {
        None | Some(Value::Null) => Ok(None),
        Some(v) => lista(v, donde).map(Some),
    }
}

pub fn mapa<'a>(val: &'a Value, donde: &str) -> Result<&'a Bloque, ErrorDiagrama> {
    val.as_object()
        .ok_or_else(|| ErrorDiagrama::new(format!("{donde}: tiene que ser un diccionario")))
}

pub fn opt_mapa<'a>(
    b: &'a Bloque,
    clave: &str,
    donde: &str,
) -> Result<Option<&'a Bloque>, ErrorDiagrama> {
    match b.get(clave) {
        None | Some(Value::Null) => Ok(None),
        Some(v) => mapa(v, donde).map(Some),
    }
}

pub fn booleano(b: &Bloque, clave: &str, defecto: bool) -> bool {
    match b.get(clave) {
        None => defecto,
        Some(Value::Bool(x)) => *x,
        Some(Value::Null) => false,
        Some(Value::Number(n)) => n.as_f64().is_some_and(|x| x != 0.0),
        Some(Value::String(s)) => !s.is_empty(),
        Some(Value::Array(a)) => !a.is_empty(),
        Some(Value::Object(m)) => !m.is_empty(),
    }
}

pub fn es_bool_true(b: &Bloque, clave: &str) -> bool {
    matches!(b.get(clave), Some(Value::Bool(true)))
}

pub fn envolver_error<T>(
    tipo: &str,
    res: Result<T, chem::ErrorQuimica>,
) -> Result<T, ErrorDiagrama> {
    res.map_err(|e| {
        let prefijo = format!("{tipo}: ");
        if e.0.starts_with(&prefijo) {
            ErrorDiagrama::new(e.0)
        } else {
            ErrorDiagrama::new(format!("{prefijo}{}", e.0))
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use dibu_core::Paleta;
    use serde_json::json;

    #[test]
    fn test_partes_formula_casos_referencia() {
        let casos: &[(&str, &[(&str, &str)])] = &[
            (
                "H2SO4",
                &[("H", "n"), ("2", "sub"), ("SO", "n"), ("4", "sub")],
            ),
            ("SO4^2-", &[("SO", "n"), ("4", "sub"), ("2−", "sup")]),
            (
                "[Cu(NH3)4]^2+",
                &[
                    ("[Cu(NH", "n"),
                    ("3", "sub"),
                    (")", "n"),
                    ("4", "sub"),
                    ("]", "n"),
                    ("2+", "sup"),
                ],
            ),
            ("2 H2O", &[("2 H", "n"), ("2", "sub"), ("O", "n")]),
            ("pK_a1 = 4", &[("pK", "n"), ("a1", "sub"), (" = 4", "n")]),
            ("g/cm³", &[("g/cm³", "n")]),
        ];
        for &(texto, esperado) in casos {
            let got = partes_formula(texto);
            let got_ref: Vec<(&str, &str)> = got.iter().map(|(s, m)| (s.as_str(), *m)).collect();
            assert_eq!(got_ref.as_slice(), esperado, "texto={texto}");
        }
    }

    #[test]
    fn test_partes_formula_bordes_y_utf8() {
        let esc_g = partes_formula("S_N\\2");
        assert_eq!(
            esc_g,
            vec![
                ("S".to_string(), "n"),
                ("N".to_string(), "sub"),
                ("2".to_string(), "n")
            ]
        );
        // Trailing backslash no panickea
        assert_eq!(partes_formula("A\\"), vec![("A\\".to_string(), "n")]);
        // Llaves y Unicode
        assert_eq!(
            partes_formula("σ^*_{2p}"),
            vec![
                ("σ".to_string(), "n"),
                ("*".to_string(), "sup"),
                ("2p".to_string(), "sub")
            ]
        );
        // Llave sin cerrar no panickea
        assert_eq!(
            partes_formula("H_{2"),
            vec![("H".to_string(), "n"), ("2".to_string(), "sub")]
        );
    }

    #[test]
    fn test_ancho_formula_utf8() {
        assert!((ancho_formula("g/cm³", 10.0) - 33.0).abs() < 1e-9);
        let esperado = 2.0 * 10.0 * 0.66 + 1.0 * 10.0 * 0.48 + 2.0 * 10.0 * 0.48;
        assert!((ancho_formula("SO4^2-", 10.0) - esperado).abs() < 1e-9);
    }

    #[test]
    fn test_tq_emite_fondo_dx_dy_y_escape() {
        let mut c = Lienzo::new(Paleta::para(true), "abc".to_string());
        tq(
            &mut c,
            50.0,
            20.0,
            "2 H2O < &",
            11.0,
            700,
            "textMain",
            "middle",
            SANS,
            true,
        );
        let joined = c.partes.join("");
        assert!(joined.contains(r#"<rect x="#));
        assert!(joined.contains(r#"opacity="0.88"/>"#));
        assert!(joined.contains(r#"text-anchor="middle""#));
        assert!(joined.contains(r#"dy="3.3""#));
        assert!(!joined.contains(r#"dy="-0.0""#));
        assert!(joined.contains("&lt;\u{00A0}&amp;"));
    }

    #[test]
    fn test_geometrias_y_helpers_texto() {
        assert_eq!(GEOMETRIAS.len(), 13);
        assert_eq!(geometria("tetrahedral").unwrap().axe, "AX4");
        assert_eq!(por_pares(3, 1), Some("trigonal_pyramidal"));
        assert_eq!(por_pares(7, 0), None);

        assert_eq!(romano(0), "0");
        assert_eq!(romano(3), "III");
        assert_eq!(romano(4), "IV");
        assert_eq!(romano(9), "IX");
        assert_eq!(romano_signo(7.0), "+7");
        assert_eq!(romano_signo(0.0), "0");
        assert_eq!(romano_signo(-1.0), "-1");
        assert_eq!(romano_signo(2.5), "2,5");

        assert_eq!(carga_sup(0), "");
        assert_eq!(carga_sup(1), "^+");
        assert_eq!(carga_sup(-1), "^-");
        assert_eq!(carga_sup(2), "^2+");
        assert_eq!(carga_sup(-3), "^3-");
        assert_eq!(coma(1.234, 2), "1,23");
    }

    #[test]
    fn test_extractores_seguros_y_envolver_error() {
        assert!((num(&json!(3), "x").unwrap() - 3.0).abs() < 1e-12);
        assert!((num(&json!(" -1.25 "), "x").unwrap() + 1.25).abs() < 1e-12);
        assert!(num(&json!(true), "x").is_err());
        assert!(num(&json!(null), "x").is_err());
        assert!(num(&json!("NaN"), "x").is_err());
        assert!(num(&json!("inf"), "x").is_err());
        assert!(num(&json!("abc"), "x").is_err());

        assert_eq!(entero(&json!(4), "n").unwrap(), 4);
        assert_eq!(entero(&json!(4.0), "n").unwrap(), 4);
        assert_eq!(entero(&json!(" -7 "), "n").unwrap(), -7);
        assert!(entero(&json!(false), "n").is_err());
        assert!(entero(&json!(4.5), "n").is_err());
        assert!(entero(&json!("4.5"), "n").is_err());

        let b: Bloque = serde_json::from_value(json!({
            "a": 1.5,
            "nulo": null,
            "arr": [1, 2],
            "obj": {"k": "v"},
            "flag": 1,
            "blank": true,
            "texto": ""
        }))
        .unwrap();

        assert_eq!(opt_num(&b, "a", "a").unwrap(), Some(1.5));
        assert_eq!(opt_num(&b, "nulo", "nulo").unwrap(), None);
        assert_eq!(opt_num(&b, "falta", "falta").unwrap(), None);
        assert_eq!(opt_texto(&b, "texto"), None);
        assert_eq!(opt_texto(&b, "a"), Some("1.5".to_string()));
        assert_eq!(opt_lista(&b, "arr", "arr").unwrap().unwrap().len(), 2);
        assert!(opt_lista(&b, "a", "a").is_err());
        assert_eq!(opt_mapa(&b, "obj", "obj").unwrap().unwrap().len(), 1);
        assert!(opt_mapa(&b, "arr", "arr").is_err());
        assert!(booleano(&b, "flag", false));
        assert!(!booleano(&b, "texto", true));
        assert!(booleano(&b, "ausente", true));
        assert!(es_bool_true(&b, "blank"));
        assert!(!es_bool_true(&b, "flag"));
        assert_eq!(cadena(&json!(26), "el").unwrap(), "26");
        assert!(cadena(&json!(true), "el").is_err());

        let err = envolver_error::<()>("orbitals", Err(chem::ErrorQuimica("falló".to_string())))
            .unwrap_err();
        assert_eq!(err.0, "orbitals: falló");

        let err_sin_dup = envolver_error::<()>(
            "unit_cell",
            Err(chem::ErrorQuimica("unit_cell: red 'hcp'".to_string())),
        )
        .unwrap_err();
        assert_eq!(err_sin_dup.0, "unit_cell: red 'hcp'");

        let e_num = num(&json!("abc"), "campo").unwrap_err().0;
        assert!(e_num.contains("'abc' tiene que ser un número"), "{e_num}");
        let e_ent = entero(&json!("xyz"), "campo").unwrap_err().0;
        assert!(e_ent.contains("'xyz' tiene que ser un entero"), "{e_ent}");
    }

    #[test]
    fn test_romano_acotado_en_o1_y_bordes() {
        assert_eq!(romano(0), "0");
        assert_eq!(romano(-3), "-3");
        assert_eq!(romano(1), "I");
        assert_eq!(romano(4), "IV");
        assert_eq!(romano(9), "IX");
        assert_eq!(romano(39), "XXXIX");
        assert_eq!(romano(40), "40");
        assert_eq!(romano(i64::MAX), i64::MAX.to_string());
        assert_eq!(romano(i64::MIN), i64::MIN.to_string());
    }
}
