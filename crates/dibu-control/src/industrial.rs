//! Diagramas industriales: `instrumentation` (P&ID ISA-5.1), `ladder` (PLC) y `grafcet`.

use crate::math::numero;
use dibu_core::{
    ancho_texto, bool_opt, encabezado, esc, str_opt, valor_str, Bloque, ErrorDiagrama, Lienzo,
    Punto, MONO, SANS,
};
use serde_json::Value;
use std::collections::{BTreeSet, HashMap};

// --- P&ID (ISA-5.1) ------------------------------------------------------------------------

const EQUIPOS: &[&str] = &[
    "tank",
    "vessel",
    "reactor",
    "column",
    "pump",
    "control_valve",
    "valve",
    "heat_exchanger",
];
const UBICACIONES: &[&str] = &["field", "panel", "panel_back", "dcs", "plc"];
const LINEAS: &[&str] = &["process", "electric", "pneumatic", "data", "capillary"];

struct Pieza {
    x: f64,
    y: f64,
    mw: f64,
    mh: f64,
    circular: bool,
    descarga: Option<Punto>,
    actuador: Option<Punto>,
}

impl Pieza {
    fn borde(&self, hacia: Punto) -> Punto {
        let (dx, dy) = (hacia.0 - self.x, hacia.1 - self.y);
        if dx == 0.0 && dy == 0.0 {
            return (self.x, self.y);
        }
        if self.circular {
            let n = dx.hypot(dy);
            return (self.x + dx / n * self.mw, self.y + dy / n * self.mw);
        }
        let tx = if dx != 0.0 { self.mw / dx.abs() } else { f64::INFINITY };
        let ty = if dy != 0.0 { self.mh / dy.abs() } else { f64::INFINITY };
        let t = tx.min(ty);
        (self.x + dx * t, self.y + dy * t)
    }
}

fn par_coords(val: &Value, g: f64, donde: &str) -> Result<Punto, ErrorDiagrama> {
    let Some(arr) = val.as_array() else {
        return Err(ErrorDiagrama::new(format!("{donde}: se espera [x, y]")));
    };
    if arr.len() != 2 {
        return Err(ErrorDiagrama::new(format!("{donde}: se espera [x, y]")));
    }
    let x = numero(&arr[0], donde)? * g;
    let y = numero(&arr[1], donde)? * g;
    if !x.is_finite() || !y.is_finite() || x.abs() > 1_000_000.0 || y.abs() > 1_000_000.0 {
        return Err(ErrorDiagrama::new(format!(
            "{donde}: coordenadas fuera de rango finito"
        )));
    }
    Ok((x, y))
}

fn dibujar_equipo(c: &mut Lienzo, e: &Bloque, g: f64, donde: &str) -> Result<Pieza, ErrorDiagrama> {
    let tipo = str_opt(e, "kind").unwrap_or("");
    if !EQUIPOS.contains(&tipo) {
        return Err(ErrorDiagrama::new(format!(
            "{donde}.kind: {tipo:?} (valen {})",
            EQUIPOS.join(", ")
        )));
    }
    let (x, y) = par_coords(&e["at"], g, &format!("{donde}.at"))?;
    let etiqueta = str_opt(e, "label").unwrap_or("");
    let bg = c.pal.background.clone();
    let trazo = format!(r#"stroke="{}" stroke-width="1.8""#, c.pal.stroke_main);
    match tipo {
        "tank" | "vessel" | "reactor" | "column" => {
            let def = match tipo {
                "tank" => (2.0, 2.4),
                "vessel" => (1.4, 2.6),
                "reactor" => (1.8, 2.4),
                _ => (1.0, 4.0),
            };
            let (w, h) = match e.get("size") {
                Some(v) => par_coords(v, g, &format!("{donde}.size"))?,
                None => (def.0 * g, def.1 * g),
            };
            if w < 0.0 || h < 0.0 {
                return Err(ErrorDiagrama::new(format!(
                    "{donde}.size: las dimensiones no pueden ser negativas"
                )));
            }
            if tipo == "tank" {
                c.add(format!(
                    r##"<path d="M {:.1} {:.1} L {:.1} {:.1} L {:.1} {:.1} L {:.1} {:.1}" fill="{bg}" {trazo}/>"##,
                    x - w / 2.0, y - h / 2.0,
                    x - w / 2.0, y + h / 2.0,
                    x + w / 2.0, y + h / 2.0,
                    x + w / 2.0, y - h / 2.0
                ));
                if let Some(lv_val) = e.get("level").filter(|v| !v.is_null()) {
                    let lv = numero(lv_val, &format!("{donde}.level"))?;
                    if !(0.0..=1.0).contains(&lv) {
                        return Err(ErrorDiagrama::new(format!(
                            "{donde}.level: tiene que estar entre 0 y 1"
                        )));
                    }
                    let yl = y + h / 2.0 - lv * h;
                    if !yl.is_finite() {
                        return Err(ErrorDiagrama::new(format!(
                            "{donde}.level: posición de nivel fuera de rango finito"
                        )));
                    }
                    c.linea(x - w / 2.0 + 4.0, yl, x + w / 2.0 - 4.0, yl, "strokeSub", 1.0, None, "5 3");
                }
            } else {
                let radio = (w / 2.0).min(h / 4.0);
                c.add(format!(
                    r##"<rect x="{:.1}" y="{:.1}" width="{w:.1}" height="{h:.1}" rx="{:.1}" ry="{radio:.1}" fill="{bg}" {trazo}/>"##,
                    x - w / 2.0,
                    y - h / 2.0,
                    w / 2.0
                ));
                if tipo == "column" {
                    for k in 1..8 {
                        let yk = y - h / 2.0 + radio + k as f64 * (h - 2.0 * radio) / 8.0;
                        c.linea(
                            x - w / 2.0,
                            yk,
                            x + w / 2.0 - w * 0.25 * (k % 2) as f64,
                            yk,
                            "strokeSub",
                            0.9,
                            None,
                            "",
                        );
                    }
                }
                if tipo == "reactor" {
                    let tope = y - h / 2.0;
                    c.add(format!(
                        r##"<circle cx="{x:.1}" cy="{:.1}" r="10" fill="{bg}" {trazo}/>"##,
                        tope - 16.0
                    ));
                    c.t(x, tope - 12.0, "M", 10.0, 700, "textMain", "middle", MONO);
                    c.linea(x, tope - 6.0, x, y + h * 0.25, "strokeMain", 1.5, None, "");
                    c.add(format!(
                        r#"<path d="M {:.1} {:.1} L {:.1} {:.1} M {:.1} {:.1} L {:.1} {:.1}" {trazo}/>"#,
                        x - w * 0.28, y + h * 0.18,
                        x + w * 0.28, y + h * 0.32,
                        x - w * 0.28, y + h * 0.32,
                        x + w * 0.28, y + h * 0.18
                    ));
                }
            }
            if !etiqueta.is_empty() {
                c.t(x, y + h / 2.0 + 14.0, etiqueta, 10.0, 700, "textMain", "middle", MONO);
            }
            Ok(Pieza {
                x,
                y,
                mw: w / 2.0,
                mh: h / 2.0,
                circular: false,
                descarga: None,
                actuador: None,
            })
        }
        "pump" => {
            let r = 0.42 * g;
            c.add(format!(
                r#"<path d="M {:.1} {:.1} L {:.1} {:.1} M {:.1} {:.1} L {:.1} {:.1} M {:.1} {:.1} L {:.1} {:.1}" {trazo}/>"#,
                x - r * 0.8, y + r * 1.25, x - r * 0.45, y + r * 0.7,
                x + r * 0.8, y + r * 1.25, x + r * 0.45, y + r * 0.7,
                x - r, y + r * 1.25, x + r, y + r * 1.25
            ));
            c.add(format!(r##"<circle cx="{x:.1}" cy="{y:.1}" r="{r:.1}" fill="{bg}" {trazo}/>"##));
            c.linea(x, y - r, x + r * 1.3, y - r, "strokeMain", 1.8, None, "");
            if !etiqueta.is_empty() {
                c.t(x, y + r * 1.25 + 14.0, etiqueta, 10.0, 700, "textMain", "middle", MONO);
            }
            Ok(Pieza {
                x,
                y,
                mw: r,
                mh: r,
                circular: true,
                descarga: Some((x + r * 1.3, y - r)),
                actuador: None,
            })
        }
        "control_valve" | "valve" => {
            let a = 0.36 * g;
            let vertical = str_opt(e, "orientation") == Some("v");
            if vertical {
                c.add(format!(
                    r##"<path d="M {:.1} {:.1} L {:.1} {:.1} L {:.1} {:.1} L {:.1} {:.1} Z" fill="{bg}" {trazo}/>"##,
                    x - a * 0.6, y - a, x + a * 0.6, y - a, x - a * 0.6, y + a, x + a * 0.6, y + a
                ));
            } else {
                c.add(format!(
                    r##"<path d="M {:.1} {:.1} L {:.1} {:.1} L {:.1} {:.1} L {:.1} {:.1} Z" fill="{bg}" {trazo}/>"##,
                    x - a, y - a * 0.6, x - a, y + a * 0.6, x + a, y - a * 0.6, x + a, y + a * 0.6
                ));
            }
            let mut tope = y;
            let mut actuador = (x, y);
            if tipo == "control_valve" {
                if vertical {
                    c.linea(x, y, x + a * 1.6, y, "strokeMain", 1.5, None, "");
                    c.add(format!(
                        r##"<path d="M {:.1} {:.1} A {:.1} {:.1} 0 0 1 {:.1} {:.1} Z" fill="{bg}" {trazo}/>"##,
                        x + a * 1.6, y - a * 0.8, a * 0.6, a * 0.8, x + a * 1.6, y + a * 0.8
                    ));
                    actuador = (x + a * 2.2, y);
                } else {
                    c.linea(x, y, x, y - a * 1.6, "strokeMain", 1.5, None, "");
                    c.add(format!(
                        r##"<path d="M {:.1} {:.1} A {:.1} {:.1} 0 0 1 {:.1} {:.1} Z" fill="{bg}" {trazo}/>"##,
                        x - a * 0.8, y - a * 1.6, a * 0.8, a * 0.6, x + a * 0.8, y - a * 1.6
                    ));
                    tope = y - a * 2.2;
                    actuador = (x, tope);
                }
            }
            if !etiqueta.is_empty() {
                if vertical {
                    c.t(x - a - 4.0, y + 4.0, etiqueta, 10.0, 700, "textMain", "end", MONO);
                } else {
                    let ly = if tipo == "control_valve" { tope + 2.0 } else { y - a - 4.0 };
                    c.t(x + a + 4.0, ly, etiqueta, 10.0, 700, "textMain", "start", MONO);
                }
            }
            Ok(Pieza {
                x,
                y,
                mw: if vertical { a * 0.6 } else { a },
                mh: if vertical { a } else { a * 0.6 },
                circular: false,
                descarga: None,
                actuador: Some(actuador),
            })
        }
        _ => {
            // heat_exchanger
            let r = 0.45 * g;
            c.add(format!(r##"<circle cx="{x:.1}" cy="{y:.1}" r="{r:.1}" fill="{bg}" {trazo}/>"##));
            c.add(format!(
                r#"<path d="M {:.1} {y:.1} L {:.1} {y:.1} L {:.1} {:.1} L {:.1} {:.1} L {:.1} {y:.1} L {:.1} {y:.1}" fill="none" {trazo}/>"#,
                x - r, x - r * 0.5, x - r * 0.25, y - r * 0.45, x + r * 0.25, y + r * 0.45, x + r * 0.5, x + r
            ));
            if !etiqueta.is_empty() {
                c.t(x, y + r + 14.0, etiqueta, 10.0, 700, "textMain", "middle", MONO);
            }
            Ok(Pieza {
                x,
                y,
                mw: r,
                mh: r,
                circular: true,
                descarga: None,
                actuador: None,
            })
        }
    }
}

fn dibujar_instrumento(c: &mut Lienzo, e: &Bloque, g: f64, donde: &str) -> Result<Pieza, ErrorDiagrama> {
    let ubicacion = str_opt(e, "location").unwrap_or("field");
    if !UBICACIONES.contains(&ubicacion) {
        return Err(ErrorDiagrama::new(format!(
            "{donde}.location: {ubicacion:?} (valen {})",
            UBICACIONES.join(", ")
        )));
    }
    let (x, y) = par_coords(&e["at"], g, &format!("{donde}.at"))?;
    let r = 0.4 * g;
    let bg = c.pal.background.clone();
    let trazo = format!(r#"stroke="{}" stroke-width="1.5""#, c.pal.stroke_main);
    if matches!(ubicacion, "dcs" | "plc") {
        c.add(format!(
            r##"<rect x="{:.1}" y="{:.1}" width="{:.1}" height="{:.1}" fill="{bg}" {trazo}/>"##,
            x - r, y - r, 2.0 * r, 2.0 * r
        ));
    }
    if ubicacion == "plc" {
        c.add(format!(
            r##"<path d="M {x:.1} {:.1} L {:.1} {y:.1} L {x:.1} {:.1} L {:.1} {y:.1} Z" fill="{bg}" {trazo}/>"##,
            y - r, x + r, y + r, x - r
        ));
    } else {
        c.add(format!(r##"<circle cx="{x:.1}" cy="{y:.1}" r="{r:.1}" fill="{bg}" {trazo}/>"##));
    }
    if matches!(ubicacion, "panel" | "dcs") {
        c.linea(x - r, y, x + r, y, "strokeMain", 1.2, None, "");
    } else if ubicacion == "panel_back" {
        c.linea(x - r, y, x + r, y, "strokeMain", 1.2, None, "3 2");
    }
    let tag = str_opt(e, "tag").unwrap_or("");
    let (letras, num) = if let Some((a, b)) = tag.split_once(' ') {
        (a, b.trim())
    } else {
        let l = tag.trim_end_matches(|ch: char| ch.is_ascii_digit() || ch == '-');
        (l, tag[l.len()..].trim_matches('-'))
    };
    if bool_opt(e, "blank", false) {
        c.vacio(x - r * 0.6, x + r * 0.6, y - 4.0);
        c.vacio(x - r * 0.6, x + r * 0.6, y + r * 0.6);
    } else {
        c.t(x, y - 4.0, letras, 10.0, 700, "textMain", "middle", SANS);
        c.t(x, y + r * 0.55, num, 9.0, 400, "textMain", "middle", SANS);
    }
    Ok(Pieza {
        x,
        y,
        mw: r,
        mh: r,
        circular: !matches!(ubicacion, "dcs" | "plc"),
        descarga: None,
        actuador: None,
    })
}

fn marcas_linea(c: &mut Lienzo, puntos: &[Punto], tipo: &str) {
    let mut resto = 13.0;
    for win in puntos.windows(2) {
        let ((x1, y1), (x2, y2)) = (win[0], win[1]);
        let largo = (x2 - x1).hypot(y2 - y1);
        if largo == 0.0 {
            continue;
        }
        let (ux, uy) = ((x2 - x1) / largo, (y2 - y1) / largo);
        let mut s = resto;
        while s < largo - 6.0 {
            let (px, py) = (x1 + ux * s, y1 + uy * s);
            match tipo {
                "pneumatic" => {
                    for corrido in [-3.0, 3.0] {
                        let (qx, qy) = (px + ux * corrido, py + uy * corrido);
                        c.linea(
                            qx - uy * 5.0 + ux * 3.0,
                            qy + ux * 5.0 + uy * 3.0,
                            qx + uy * 5.0 - ux * 3.0,
                            qy - ux * 5.0 - uy * 3.0,
                            "strokeMain",
                            1.0,
                            None,
                            "",
                        );
                    }
                }
                "data" => {
                    c.add(format!(
                        r#"<circle cx="{px:.1}" cy="{py:.1}" r="2.6" fill="{}" stroke="{}" stroke-width="1"/>"#,
                        c.pal.background,
                        c.pal.stroke_main
                    ));
                }
                "capillary" => {
                    c.linea(px - 3.0, py - 3.0, px + 3.0, py + 3.0, "strokeMain", 1.0, None, "");
                    c.linea(px - 3.0, py + 3.0, px + 3.0, py - 3.0, "strokeMain", 1.0, None, "");
                }
                _ => {}
            }
            s += 26.0;
        }
        resto = s - largo;
    }
}

pub fn render_instrumentation(c: &mut Lienzo, b: &Bloque) -> Result<Punto, ErrorDiagrama> {
    let g = match b.get("grid") {
        None | Some(Value::Null) => 50.0,
        Some(v) => numero(v, "instrumentation.grid")?,
    };
    if g < 0.0 {
        return Err(ErrorDiagrama::new(
            "instrumentation: grid no puede ser negativo",
        ));
    }
    let top = encabezado(c, b, "") + 10.0;
    c.add(format!(r#"<g transform="translate(0 {top})">"#));
    let equipos = b.get("equipment").and_then(Value::as_array).cloned().unwrap_or_default();
    let instrumentos = b.get("instruments").and_then(Value::as_array).cloned().unwrap_or_default();
    if equipos.is_empty() && instrumentos.is_empty() {
        return Err(ErrorDiagrama::new("instrumentation: falta equipment o instruments"));
    }
    let mut medidas = c.sub_lienzo();
    let mut piezas: HashMap<String, Pieza> = HashMap::new();
    for (i, e) in equipos.iter().enumerate() {
        let Some(obj) = e.as_object() else {
            return Err(ErrorDiagrama::new(format!("equipment[{i}]: necesita id, kind y at: [x, y]")));
        };
        if !obj.contains_key("id") || !obj.contains_key("at") {
            return Err(ErrorDiagrama::new(format!("equipment[{i}]: necesita id, kind y at: [x, y]")));
        }
        let id = valor_str(&obj["id"]);
        piezas.insert(id, dibujar_equipo(&mut medidas, obj, g, &format!("equipment[{i}]"))?);
    }
    for (i, e) in instrumentos.iter().enumerate() {
        let Some(obj) = e.as_object() else {
            return Err(ErrorDiagrama::new(format!("instruments[{i}]: necesita id, tag y at: [x, y]")));
        };
        if !obj.contains_key("id") || !obj.contains_key("at") {
            return Err(ErrorDiagrama::new(format!("instruments[{i}]: necesita id, tag y at: [x, y]")));
        }
        let id = valor_str(&obj["id"]);
        piezas.insert(id, dibujar_instrumento(&mut medidas, obj, g, &format!("instruments[{i}]"))?);
    }

    let mut ancho = piezas.values().map(|p| p.x + p.mw).fold(0.0_f64, f64::max) + 60.0;
    let mut alto = piezas.values().map(|p| p.y + p.mh).fold(0.0_f64, f64::max) + 30.0;
    let lineas = b.get("lines").and_then(Value::as_array).cloned().unwrap_or_default();

    for (i, lv) in lineas.iter().enumerate() {
        let Some(linea) = lv.as_object() else { continue };
        let donde = format!("lines[{i}]");
        let tipo = str_opt(linea, "kind").unwrap_or("process");
        if !LINEAS.contains(&tipo) {
            return Err(ErrorDiagrama::new(format!(
                "{donde}.kind: {tipo:?} (valen {})",
                LINEAS.join(", ")
            )));
        }
        let lugar = |ref_v: Option<&Value>, clave: &str| -> Result<(Option<&Pieza>, Punto), ErrorDiagrama> {
            let Some(rv) = ref_v else {
                return Err(ErrorDiagrama::new(format!("{donde}.{clave}: falta referencia")));
            };
            if rv.is_array() {
                return Ok((None, par_coords(rv, g, &format!("{donde}.{clave}"))?));
            }
            let id = valor_str(rv);
            let Some(pieza) = piezas.get(&id) else {
                return Err(ErrorDiagrama::new(format!(
                    "{donde}.{clave}: no hay un equipo ni un instrumento {id:?}"
                )));
            };
            if clave == "from" {
                if let Some(d) = pieza.descarga {
                    return Ok((None, d));
                }
                if bool_opt(linea, "fromActuator", false) {
                    if let Some(a) = pieza.actuador {
                        return Ok((None, a));
                    }
                }
            }
            if clave == "to" && bool_opt(linea, "toActuator", tipo != "process") {
                if let Some(a) = pieza.actuador {
                    return Ok((None, a));
                }
            }
            Ok((Some(pieza), (pieza.x, pieza.y)))
        };

        let (a, pa) = lugar(linea.get("from"), "from")?;
        let (z, pz) = lugar(linea.get("to"), "to")?;
        let mut intermedios = Vec::new();
        if let Some(arr) = linea.get("via").and_then(Value::as_array) {
            for p in arr {
                intermedios.push(par_coords(p, g, &format!("{donde}.via"))?);
            }
        }
        if intermedios.is_empty() && pa.0 != pz.0 && pa.1 != pz.1 {
            if str_opt(linea, "route").unwrap_or("hv") == "hv" {
                intermedios.push((pz.0, pa.1));
            } else {
                intermedios.push((pa.0, pz.1));
            }
        }
        let mut puntos = Vec::with_capacity(intermedios.len() + 2);
        puntos.push(pa);
        puntos.extend(intermedios);
        puntos.push(pz);
        if let Some(pa_ref) = a {
            puntos[0] = pa_ref.borde(puntos[1]);
        }
        if let Some(pz_ref) = z {
            let n = puntos.len();
            puntos[n - 1] = pz_ref.borde(puntos[n - 2]);
        }
        let camino: String = puntos
            .iter()
            .map(|(x, y)| format!("{x:.1} {y:.1}"))
            .collect::<Vec<_>>()
            .join(" L ");
        let estilo = match tipo {
            "process" => r#"stroke-width="2.2""#,
            "electric" => r#"stroke-width="1.2" stroke-dasharray="7 4""#,
            _ => r#"stroke-width="1.2""#,
        };
        let flecha = bool_opt(linea, "arrow", tipo == "process");
        let marca = if flecha {
            format!(r#" marker-end="url(#flecha-main-{})""#, c.sufijo)
        } else {
            String::new()
        };
        c.add(format!(
            r#"<path d="M {camino}" fill="none" stroke="{}" {estilo}{marca}/>"#,
            c.pal.stroke_main
        ));
        if matches!(tipo, "pneumatic" | "data" | "capillary") {
            marcas_linea(c, &puntos, tipo);
        }
        if let Some(lbl) = str_opt(linea, "label") {
            let seg = puntos
                .windows(2)
                .max_by(|a, b| {
                    let la = (a[1].0 - a[0].0).hypot(a[1].1 - a[0].1);
                    let lb = (b[1].0 - b[0].0).hypot(b[1].1 - b[0].1);
                    la.total_cmp(&lb)
                })
                .unwrap();
            let ((x1, y1), (x2, y2)) = (seg[0], seg[1]);
            let vertical = (x2 - x1).abs() < (y2 - y1).abs();
            let (lx, ly) = if vertical {
                ((x1 + x2) / 2.0 + 7.0, (y1 + y2) / 2.0 + 4.0)
            } else {
                ((x1 + x2) / 2.0, (y1 + y2) / 2.0 - 6.0)
            };
            c.add(format!(
                r#"<text x="{lx:.1}" y="{ly:.1}" text-anchor="{}" font-family="{}" font-size="{}" fill="{}">{}</text>"#,
                if vertical { "start" } else { "middle" },
                c.font_sans(),
                dibu_core::fmt_num_clean(c.font_size(10.0)),
                c.pal.text_sub,
                esc(lbl)
            ));
        }
        for &(x, y) in &puntos {
            ancho = ancho.max(x + 40.0);
            alto = alto.max(y + 20.0);
        }
    }

    for (i, e) in equipos.iter().enumerate() {
        dibujar_equipo(c, e.as_object().unwrap(), g, &format!("equipment[{i}]"))?;
    }
    for (i, e) in instrumentos.iter().enumerate() {
        dibujar_instrumento(c, e.as_object().unwrap(), g, &format!("instruments[{i}]"))?;
    }

    if bool_opt(b, "legend", true) {
        let mut presentes: BTreeSet<&str> = BTreeSet::new();
        for lv in &lineas {
            if let Some(obj) = lv.as_object() {
                let t = str_opt(obj, "kind").unwrap_or("process");
                if t != "process" {
                    presentes.insert(t);
                }
            }
        }
        for (k, tipo) in presentes.iter().enumerate() {
            let ly = alto + 6.0 + k as f64 * 16.0;
            let estilo = if *tipo == "electric" {
                r#"stroke-dasharray="7 4""#
            } else {
                ""
            };
            c.add(format!(
                r#"<line x1="10" y1="{ly:.1}" x2="62" y2="{ly:.1}" stroke="{}" stroke-width="1.2" {estilo}/>"#,
                c.pal.stroke_main
            ));
            if *tipo != "electric" {
                marcas_linea(c, &[(10.0, ly), (62.0, ly)], tipo);
            }
            let nombre = match *tipo {
                "electric" => "señal eléctrica",
                "pneumatic" => "señal neumática",
                "data" => "enlace de datos",
                _ => "tubo capilar",
            };
            c.t(70.0, ly + 4.0, nombre, 9.5, 400, "textSub", "start", SANS);
        }
        alto += 6.0 + presentes.len() as f64 * 16.0;
    }
    c.add("</g>");
    Ok((ancho, alto + top))
}

// --- Escalera (PLC) ------------------------------------------------------------------------

const CONTACTOS: &[&str] = &["no", "nc", "p", "n"];
const SALIDAS: &[&str] = &["coil", "set", "reset", "ncoil"];
const CAJAS: &[&str] = &["ton", "tof", "tp", "ctu", "ctd"];
const FILA: f64 = 58.0;
const CONTACTO: f64 = 64.0;
const CAJA_W: f64 = 96.0;
const CAJA_H: f64 = 48.0;

fn clase_ladder(e: &Bloque) -> Result<(&'static str, String), ErrorDiagrama> {
    if let Some(v) = e.get("no").or_else(|| e.get("na")).or_else(|| e.get("false")) {
        return Ok(("no", valor_str(v)));
    }
    for &k in CONTACTOS.iter().chain(SALIDAS).chain(CAJAS) {
        if let Some(v) = e.get(k) {
            return Ok((k, valor_str(v)));
        }
    }
    if e.contains_key("or") {
        return Ok(("or", String::new()));
    }
    Err(ErrorDiagrama::new(
        "ladder: elemento desconocido (valen no, nc, p, n, coil, set, reset, ncoil, ton, tof, tp, ctu, ctd y or)",
    ))
}

fn medir_escalon(serie: &[Value]) -> Result<Punto, ErrorDiagrama> {
    let mut ancho = 0.0;
    let mut alto = FILA;
    for item in serie {
        let Some(e) = item.as_object() else {
            return Err(ErrorDiagrama::new("ladder: cada elemento tiene que ser un objeto"));
        };
        let (k, _) = clase_ladder(e)?;
        if k == "or" {
            let Some(arr) = e["or"].as_array() else {
                return Err(ErrorDiagrama::new("ladder: 'or' tiene que ser una lista de ramas"));
            };
            let mut max_w = 0.0_f64;
            let mut sum_h = 0.0_f64;
            for r in arr {
                let Some(ra) = r.as_array() else {
                    return Err(ErrorDiagrama::new("ladder: cada rama de 'or' tiene que ser una lista"));
                };
                let (rw, rh) = medir_escalon(ra)?;
                max_w = max_w.max(rw);
                sum_h += rh;
            }
            ancho += max_w + 24.0;
            alto = alto.max(sum_h);
        } else if CAJAS.contains(&k) {
            ancho += CAJA_W + 24.0;
            alto = alto.max(CAJA_H + 30.0);
        } else {
            ancho += CONTACTO;
        }
    }
    Ok((ancho, alto))
}

fn dibujar_simbolo(c: &mut Lienzo, e: &Bloque, x: f64, y: f64, w: f64) -> Result<(), ErrorDiagrama> {
    let (k, tag) = clase_ladder(e)?;
    let cx = x + w / 2.0;
    let blank = bool_opt(e, "blank", false);
    if blank {
        c.linea(x, y, cx - 14.0, y, "strokeMain", 1.5, None, "");
        c.linea(cx + 14.0, y, x + w, y, "strokeMain", 1.5, None, "");
        c.add(format!(
            r#"<rect x="{:.1}" y="{:.1}" width="28" height="26" fill="none" stroke="{}" stroke-width="1.2" stroke-dasharray="3 2"/>"#,
            cx - 14.0, y - 13.0, c.pal.stroke_muted
        ));
    } else if CONTACTOS.contains(&k) {
        c.linea(x, y, cx - 6.0, y, "strokeMain", 1.5, None, "");
        c.linea(cx + 6.0, y, x + w, y, "strokeMain", 1.5, None, "");
        for dx in [-6.0, 6.0] {
            c.linea(cx + dx, y - 11.0, cx + dx, y + 11.0, "strokeMain", 2.0, None, "");
        }
        if k == "nc" {
            c.linea(cx - 9.0, y + 10.0, cx + 9.0, y - 10.0, "strokeMain", 1.5, None, "");
        } else if matches!(k, "p" | "n") {
            c.t(cx, y + 4.0, &k.to_uppercase(), 9.0, 700, "textMain", "middle", MONO);
        }
    } else if SALIDAS.contains(&k) {
        c.linea(x, y, cx - 11.0, y, "strokeMain", 1.5, None, "");
        c.linea(cx + 11.0, y, x + w, y, "strokeMain", 1.5, None, "");
        c.add(format!(
            r#"<path d="M {:.1} {:.1} A 12 12 0 0 0 {:.1} {:.1} M {:.1} {:.1} A 12 12 0 0 1 {:.1} {:.1}" fill="none" stroke="{}" stroke-width="2"/>"#,
            cx - 6.0, y - 11.0, cx - 6.0, y + 11.0,
            cx + 6.0, y - 11.0, cx + 6.0, y + 11.0,
            c.pal.stroke_main
        ));
        let letra = match k {
            "set" => Some("S"),
            "reset" => Some("R"),
            "ncoil" => Some("/"),
            _ => None,
        };
        if let Some(l) = letra {
            c.t(cx, y + 4.0, l, 10.0, 700, "textMain", "middle", MONO);
        }
    } else {
        let bx = x + 12.0;
        c.linea(x, y, bx, y, "strokeMain", 1.5, None, "");
        c.linea(bx + CAJA_W, y, x + w, y, "strokeMain", 1.5, None, "");
        c.r(bx, y - 14.0, CAJA_W, CAJA_H, "fillCell", 1.6, 0.0);
        c.t(bx + CAJA_W / 2.0, y + 2.0, &k.to_uppercase(), 11.0, 700, "textMain", "middle", MONO);
        if let Some(preset) = e.get("preset").filter(|v| !v.is_null()) {
            let pref = if matches!(k, "ton" | "tof" | "tp") { "PT" } else { "PV" };
            c.t(bx + 6.0, y + 26.0, &format!("{pref} {}", valor_str(preset)), 9.0, 400, "textSub", "start", MONO);
        }
    }
    if !CAJAS.contains(&k) || blank {
        if !tag.is_empty() {
            c.t(cx, y - 16.0, &tag, 10.0, 700, "textMain", "middle", MONO);
        }
        if let Some(lbl) = str_opt(e, "label") {
            c.t(cx, y + 26.0, lbl, 9.0, 400, "textMuted", "middle", SANS);
        }
    } else {
        c.t(x + 12.0 + CAJA_W / 2.0, y - 20.0, &tag, 10.0, 700, "textMain", "middle", MONO);
        if let Some(lbl) = str_opt(e, "label") {
            c.t(x + 12.0 + CAJA_W / 2.0, y + CAJA_H + 2.0, lbl, 9.0, 400, "textMuted", "middle", SANS);
        }
    }
    Ok(())
}

fn dibujar_serie(c: &mut Lienzo, serie: &[Value], mut x: f64, y: f64, mut ancho: f64) -> Result<(), ErrorDiagrama> {
    for item in serie {
        let e = item.as_object().unwrap();
        let (k, _) = clase_ladder(e)?;
        if k == "or" {
            let arr = e["or"].as_array().unwrap();
            let mut ramas = Vec::with_capacity(arr.len());
            for r in arr {
                let ra = r.as_array().unwrap();
                ramas.push((ra.as_slice(), medir_escalon(ra)?));
            }
            let w = ramas.iter().map(|(_, m)| m.0).fold(0.0_f64, f64::max) + 24.0;
            let mut yy = y;
            let mut ys = Vec::with_capacity(ramas.len());
            for (rama, (_, rh)) in ramas {
                c.linea(x, yy, x + 12.0, yy, "strokeMain", 1.5, None, "");
                dibujar_serie(c, rama, x + 12.0, yy, w - 24.0)?;
                c.linea(x + w - 12.0, yy, x + w, yy, "strokeMain", 1.5, None, "");
                ys.push(yy);
                yy += rh;
            }
            if let (Some(&y0), Some(&y1)) = (ys.first(), ys.last()) {
                c.linea(x, y0, x, y1, "strokeMain", 1.5, None, "");
                c.linea(x + w, y0, x + w, y1, "strokeMain", 1.5, None, "");
            }
            x += w;
            ancho -= w;
        } else {
            let w = if CAJAS.contains(&k) { CAJA_W + 24.0 } else { CONTACTO };
            dibujar_simbolo(c, e, x, y, w)?;
            x += w;
            ancho -= w;
        }
    }
    if ancho > 0.5 {
        c.linea(x, y, x + ancho, y, "strokeMain", 1.5, None, "");
    }
    Ok(())
}

pub fn render_ladder(c: &mut Lienzo, b: &Bloque) -> Result<Punto, ErrorDiagrama> {
    let Some(escalones) = b.get("rungs").and_then(Value::as_array) else {
        return Err(ErrorDiagrama::new(
            "ladder: 'rungs' tiene que ser una lista de escalones con logic",
        ));
    };
    if escalones.is_empty() {
        return Err(ErrorDiagrama::new(
            "ladder: 'rungs' tiene que ser una lista de escalones con logic",
        ));
    }
    let mut partes = Vec::with_capacity(escalones.len());
    for (i, rv) in escalones.iter().enumerate() {
        let Some(r) = rv.as_object() else {
            return Err(ErrorDiagrama::new(format!("rungs[{i}].logic: tiene que ser una lista de elementos")));
        };
        let Some(logica) = r.get("logic").and_then(Value::as_array) else {
            return Err(ErrorDiagrama::new(format!("rungs[{i}].logic: tiene que ser una lista de elementos")));
        };
        if logica.is_empty() {
            return Err(ErrorDiagrama::new(format!("rungs[{i}].logic: tiene que ser una lista de elementos")));
        }
        let mut corte = logica.len();
        while corte > 0 {
            let obj = logica[corte - 1]
                .as_object()
                .ok_or_else(|| ErrorDiagrama::new("ladder: elemento inválido"))?;
            let (k, _) = clase_ladder(obj)?;
            if SALIDAS.contains(&k) || CAJAS.contains(&k) {
                corte -= 1;
            } else {
                break;
            }
        }
        if corte == logica.len() {
            return Err(ErrorDiagrama::new(format!(
                "rungs[{i}]: el escalón tiene que terminar en una salida (coil, set, reset, un temporizador o un contador)"
            )));
        }
        if corte == 0 {
            return Err(ErrorDiagrama::new(format!(
                "rungs[{i}]: el escalón necesita al menos un contacto antes de la salida"
            )));
        }
        let entradas = &logica[..corte];
        let salidas = &logica[corte..];
        let me = medir_escalon(entradas)?;
        let ms = medir_escalon(salidas)?;
        partes.push((r, entradas, salidas, me, ms));
    }
    let ancho_util = partes.iter().map(|(_, _, _, me, ms)| me.0 + ms.0).fold(0.0_f64, f64::max) + 40.0;
    let top = encabezado(c, b, "") + 8.0;
    let (x_izq, x_der) = (30.0, 30.0 + ancho_util);
    let mut y = top;
    for (idx, (r, entradas, salidas, me, ms)) in partes.into_iter().enumerate() {
        if let Some(com) = str_opt(r, "comment") {
            c.t(x_izq + 6.0, y + 12.0, com, 9.5, 400, "textMuted", "start", SANS);
            y += 16.0;
        }
        let linea_y = y + 26.0;
        c.t(x_izq - 10.0, linea_y + 4.0, &(idx + 1).to_string(), 10.0, 700, "textSub", "end", MONO);
        dibujar_serie(c, entradas, x_izq, linea_y, x_der - x_izq - ms.0)?;
        dibujar_serie(c, salidas, x_der - ms.0, linea_y, ms.0)?;
        y += me.1.max(ms.1) + 6.0;
    }
    c.linea(x_izq, top, x_izq, y, "strokeMain", 3.0, None, "");
    c.linea(x_der, top, x_der, y, "strokeMain", 3.0, None, "");
    Ok((x_der + 10.0, y + 4.0))
}

// --- GRAFCET -------------------------------------------------------------------------------

const ETAPA: f64 = 34.0;
const ENLACE: f64 = 18.0;

fn medir_grafcet(seq: &[Value]) -> Result<(f64, f64, f64), ErrorDiagrama> {
    let (mut izq, mut der, mut alto) = (ETAPA / 2.0, ETAPA / 2.0, 0.0);
    for (i, item) in seq.iter().enumerate() {
        if i > 0 {
            alto += ENLACE;
        }
        let Some(e) = item.as_object() else {
            return Err(ErrorDiagrama::new("grafcet: elemento inválido"));
        };
        if e.contains_key("step") {
            let acciones = e.get("actions").and_then(Value::as_array);
            let ancho_acc: f64 = match acciones {
                Some(arr) if !arr.is_empty() => {
                    arr.iter()
                        .map(|a| (ancho_texto(&valor_str(a), 10.0) + 18.0).max(70.0))
                        .sum::<f64>()
                        + 24.0
                }
                _ => 0.0,
            };
            der = der.max(ETAPA / 2.0 + ancho_acc);
            alto += ETAPA;
        } else if e.contains_key("transition") {
            let txt = valor_str(&e["transition"]);
            let base = if txt.is_empty() { "XXXXXXXX" } else { &txt };
            der = der.max(18.0 + ancho_texto(base, 10.0));
            alto += 4.0;
        } else if e.contains_key("or") || e.contains_key("and") {
            let doble = e.contains_key("and");
            let ramas_v = e.get("or").or_else(|| e.get("and")).and_then(Value::as_array)
                .ok_or_else(|| ErrorDiagrama::new("grafcet: or/and requiere una lista de ramas"))?;
            if ramas_v.is_empty() {
                return Err(ErrorDiagrama::new(
                    "grafcet: or/and requiere al menos una rama",
                ));
            }
            let mut medidas = Vec::with_capacity(ramas_v.len());
            for r in ramas_v {
                let ra = r.as_array().ok_or_else(|| ErrorDiagrama::new("grafcet: rama inválida"))?;
                medidas.push(medir_grafcet(ra)?);
            }
            let ancho_total: f64 = medidas.iter().map(|(l, r, _)| l + r).sum::<f64>()
                + 30.0 * medidas.len().saturating_sub(1) as f64;
            if let Some(&(l0, _, _)) = medidas.first() {
                izq = izq.max(l0);
                der = der.max(ancho_total - l0);
            }
            let max_h = medidas.iter().map(|(_, _, h)| *h).fold(0.0_f64, f64::max);
            alto += max_h + 2.0 * if doble { 12.0 } else { 6.0 };
        } else {
            return Err(ErrorDiagrama::new(
                "grafcet: elemento desconocido (step, transition, or, and)",
            ));
        }
    }
    Ok((izq, der, alto))
}

fn dibujar_grafcet(c: &mut Lienzo, seq: &[Value], x: f64, mut y: f64) -> Result<f64, ErrorDiagrama> {
    for (i, item) in seq.iter().enumerate() {
        if i > 0 {
            c.linea(x, y, x, y + ENLACE, "strokeMain", 1.5, None, "");
            y += ENLACE;
        }
        let e = item.as_object().unwrap();
        if e.contains_key("step") {
            let s = ETAPA;
            c.r(x - s / 2.0, y, s, s, "fillCell", 1.6, 0.0);
            if bool_opt(e, "initial", false) {
                c.add(format!(
                    r#"<rect x="{:.1}" y="{:.1}" width="{}" height="{}" fill="none" stroke="{}" stroke-width="1.2"/>"#,
                    x - s / 2.0 + 4.0, y + 4.0, (s - 8.0) as i64, (s - 8.0) as i64, c.pal.stroke_main
                ));
            }
            if bool_opt(e, "blank", false) {
                c.vacio(x - 8.0, x + 8.0, y + s / 2.0 + 6.0);
            } else {
                c.t(x, y + s / 2.0 + 4.0, &valor_str(&e["step"]), 11.0, 700, "textMain", "middle", MONO);
            }
            let mut ax = x + s / 2.0;
            if let Some(acciones) = e.get("actions").and_then(Value::as_array) {
                if !acciones.is_empty() {
                    c.linea(ax, y + s / 2.0, ax + 24.0, y + s / 2.0, "strokeMain", 1.2, None, "");
                    ax += 24.0;
                    for av in acciones {
                        let accion = valor_str(av);
                        let w = (ancho_texto(&accion, 10.0) + 18.0).max(70.0);
                        c.r(ax, y + 4.0, w, s - 8.0, "fillCell", 1.2, 0.0);
                        if accion.is_empty() {
                            c.vacio(ax + 8.0, ax + w - 8.0, y + s / 2.0 + 5.0);
                        } else {
                            c.t(ax + w / 2.0, y + s / 2.0 + 4.0, &accion, 10.0, 400, "textMain", "middle", MONO);
                        }
                        ax += w;
                    }
                }
            }
            y += s;
        } else if e.contains_key("transition") {
            c.linea(x - 10.0, y + 2.0, x + 10.0, y + 2.0, "strokeMain", 3.0, None, "");
            let texto = valor_str(&e["transition"]);
            if bool_opt(e, "blank", false) || texto.is_empty() {
                c.vacio(x + 16.0, x + 90.0, y + 6.0);
            } else {
                c.t(x + 16.0, y + 6.0, &texto, 10.0, 400, "textMain", "start", MONO);
            }
            y += 4.0;
        } else {
            let doble = e.contains_key("and");
            let ramas = e
                .get("or")
                .or_else(|| e.get("and"))
                .and_then(Value::as_array)
                .ok_or_else(|| ErrorDiagrama::new("grafcet: or/and requiere una lista de ramas"))?;
            if ramas.is_empty() {
                return Err(ErrorDiagrama::new(
                    "grafcet: or/and requiere al menos una rama",
                ));
            }
            let mut medidas = Vec::with_capacity(ramas.len());
            for r in ramas {
                medidas.push(medir_grafcet(r.as_array().unwrap())?);
            }
            let mut ejes = Vec::with_capacity(ramas.len());
            let mut cx = x;
            for (k, &(l, _, _)) in medidas.iter().enumerate() {
                if k > 0 {
                    cx += medidas[k - 1].1 + 30.0 + l;
                }
                ejes.push(cx);
            }
            let sep = if doble { 12.0 } else { 6.0 };
            let barra = |c: &mut Lienzo, yb: f64| {
                let dys: &[f64] = if doble { &[-2.0, 2.0] } else { &[0.0] };
                for &dy in dys {
                    c.linea(ejes[0], yb + dy, *ejes.last().unwrap(), yb + dy, "strokeMain", 1.5, None, "");
                }
            };
            barra(c, y);
            let mut fines = Vec::with_capacity(ramas.len());
            for (&ex, rama) in ejes.iter().zip(ramas.iter()) {
                c.linea(ex, y, ex, y + sep, "strokeMain", 1.5, None, "");
                fines.push(dibujar_grafcet(c, rama.as_array().unwrap(), ex, y + sep)?);
            }
            let y_fin = fines.iter().copied().fold(0.0_f64, f64::max) + sep;
            for (&ex, &f) in ejes.iter().zip(fines.iter()) {
                c.linea(ex, f, ex, y_fin, "strokeMain", 1.5, None, "");
            }
            barra(c, y_fin);
            y = y_fin;
        }
    }
    Ok(y)
}

pub fn render_grafcet(c: &mut Lienzo, b: &Bloque) -> Result<Punto, ErrorDiagrama> {
    let Some(seq) = b.get("steps").and_then(Value::as_array) else {
        return Err(ErrorDiagrama::new(
            "grafcet: 'steps' tiene que ser una lista de step, transition, or y and",
        ));
    };
    if seq.is_empty() {
        return Err(ErrorDiagrama::new(
            "grafcet: 'steps' tiene que ser una lista de step, transition, or y and",
        ));
    }
    let (izq, der, _) = medir_grafcet(seq)?;
    let top = encabezado(c, b, "") + 18.0;
    let x = izq + 34.0;
    let mut fin = dibujar_grafcet(c, seq, x, top)?;
    if bool_opt(b, "loop", true) {
        let retorno = x - izq - 22.0;
        c.add(format!(
            r#"<path d="M {x:.1} {fin:.1} L {x:.1} {:.1} L {retorno:.1} {:.1} L {retorno:.1} {:.1} L {x:.1} {:.1} L {x:.1} {top:.1}" fill="none" stroke="{}" stroke-width="1.5" marker-end="url(#flecha-main-{})"/>"#,
            fin + 16.0, fin + 16.0, top - 12.0, top - 12.0, c.pal.stroke_main, c.sufijo
        ));
        fin += 16.0;
    }
    Ok((x + der + 10.0, fin + 10.0))
}

#[cfg(test)]
mod tests {
    use super::*;
    use dibu_core::Paleta;
    use serde_json::json;

    #[test]
    fn grafcet_rechaza_or_y_and_vacios_sin_panic() {
        for clave in ["or", "and"] {
            let val = json!({
                "type": "grafcet",
                "steps": [{ clave: [] }]
            });
            let bloque = val.as_object().unwrap();
            let mut lienzo = Lienzo::new(Paleta::para(true), "test".to_string());
            let res = render_grafcet(&mut lienzo, bloque);
            assert!(res.is_err(), "grafcet con '{clave}: []' debe devolver Err");
        }
    }

    #[test]
    fn grafcet_rechaza_or_vacio_anidado_sin_panic() {
        let val = json!({
            "type": "grafcet",
            "steps": [
                {"step": 0, "initial": true},
                {"or": [[{"step": 1}], [{"and": []}]]}
            ]
        });
        let bloque = val.as_object().unwrap();
        let mut lienzo = Lienzo::new(Paleta::para(true), "test".to_string());
        let res = render_grafcet(&mut lienzo, bloque);
        assert!(res.is_err());
    }

    #[test]
    fn instrumentation_tank_valida_level_finito_y_rango_0_1() {
        for malo in [
            json!(1e308),
            json!(-1e308),
            json!(1e305),
            json!(-0.1),
            json!(1.2),
            json!("alto"),
            json!(true),
        ] {
            let val = json!({
                "type": "instrumentation",
                "equipment": [{"id": "T1", "kind": "tank", "at": [2, 2], "level": malo}]
            });
            let mut lienzo = Lienzo::new(Paleta::para(true), "test".to_string());
            assert!(
                render_instrumentation(&mut lienzo, val.as_object().unwrap()).is_err(),
                "level={malo} debía rechazarse"
            );
        }
        for bueno in [0.0, 0.6, 1.0] {
            let val = json!({
                "type": "instrumentation",
                "equipment": [{"id": "T1", "kind": "tank", "at": [2, 2], "level": bueno}]
            });
            let mut lienzo = Lienzo::new(Paleta::para(true), "test".to_string());
            render_instrumentation(&mut lienzo, val.as_object().unwrap())
                .unwrap_or_else(|e| panic!("level={bueno} falló: {e}"));
            let svg = lienzo.partes.join("");
            assert!(!svg.contains("inf") && !svg.contains("NaN"));
        }
    }
}
