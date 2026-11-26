//! Disposición de octetos (`byte_layout`): los campos de un formato binario con su ancho, su
//! offset y su orden de bytes, y las filas de píxeles de un BMP con su stride y su relleno.
//!
//! El búfer que se dibuja sale de los campos declarados (o de los píxeles): dibu calcula los
//! octetos, los offsets, el relleno y el stride; no supone el layout en memoria de un `struct`.

use crate::common::{ancho_texto, encabezado, es_truthy, tabla, texto, Bloque, ErrorDiagrama, Lienzo, Punto, TablaOpts, MONO, SANS};
use dibu_core::Registro;
use serde_json::Value;
use std::collections::HashSet;

const MAX_OCTETOS: usize = 32;
const MAX_CAMPOS: usize = 12;
const MAX_ANCHO_BMP: u64 = 8;
const MAX_ALTO_BMP: u64 = 6;

const ANCHO_OCTETO: f64 = 34.0;
const ALTO_OCTETO: f64 = 34.0;

pub fn registrar(reg: &mut Registro) {
    reg.registrar("byte_layout", |c, b| {
        render_byte_layout(c, b).map_err(|e| {
            if e.0.starts_with("byte_layout: ") {
                e
            } else {
                ErrorDiagrama::new(format!("byte_layout: {}", e.0))
            }
        })
    });
}

fn error_octetos(msg: impl Into<String>) -> ErrorDiagrama {
    ErrorDiagrama::new(msg)
}

fn render_byte_layout(c: &mut Lienzo, b: &Bloque) -> Result<Punto, ErrorDiagrama> {
    match b.get("variant").and_then(Value::as_str) {
        None | Some("fields") => render_campos(c, b),
        Some("bmp_rows") => render_bmp(c, b),
        Some(otra) => Err(error_octetos(format!("variante no soportada: '{otra}' (fields o bmp_rows)"))),
    }
}

/// Qué es un octeto del dibujo; cada clase tiene su rótulo, que sobrevive en monocromo.
#[derive(Clone, Copy, Debug, PartialEq)]
enum Clase {
    Dato,
    Relleno,
    Truncado,
    Invalido,
}

/// `{blank: [...]}` a nivel del bloque: las partes que se dejan en blanco.
fn blancos(b: &Bloque, admitidos: &[&str]) -> Result<HashSet<String>, ErrorDiagrama> {
    let mut salida = HashSet::new();
    let Some(v) = b.get("blank").filter(|v| !v.is_null()) else {
        return Ok(salida);
    };
    let Some(lista) = v.as_array() else {
        return Err(error_octetos(format!("'blank' es una lista con {}", admitidos.join(", "))));
    };
    for v in lista {
        let t = texto(v);
        if !admitidos.contains(&t.as_str()) {
            return Err(error_octetos(format!("'blank' admite {}: no '{t}'", admitidos.join(", "))));
        }
        salida.insert(t);
    }
    Ok(salida)
}

/// Un entero de YAML: `4660`, `"0x1234"` o `-1`.
fn entero_octeto(v: &Value, donde: &str) -> Result<i128, ErrorDiagrama> {
    match v {
        Value::Number(n) => n
            .as_i64()
            .map(i128::from)
            .or_else(|| n.as_u64().map(i128::from))
            .ok_or_else(|| error_octetos(format!("{donde}: se espera un entero, no {n}"))),
        Value::String(s) => {
            let t = s.trim();
            let (neg, t) = t.strip_prefix('-').map_or((false, t), |r| (true, r));
            let r = if let Some(h) = t.strip_prefix("0x").or_else(|| t.strip_prefix("0X")) {
                i128::from_str_radix(h, 16)
            } else {
                t.parse::<i128>()
            };
            r.map(|x| if neg { -x } else { x }).map_err(|_| error_octetos(format!("{donde}: se espera un entero, no {s:?}")))
        }
        otro => Err(error_octetos(format!("{donde}: se espera un entero, no {}", texto(otro)))),
    }
}

/// Un rectángulo de octeto según su clase.
fn caja(c: &mut Lienzo, x: f64, y: f64, w: f64, clase: Clase) {
    match clase {
        Clase::Dato => c.r(x, y, w, ALTO_OCTETO, "fillCell", 1.5, 0.0),
        Clase::Invalido => {
            c.r(x, y, w, ALTO_OCTETO, "fillCell", 1.5, 0.0);
            c.linea(x + 4.0, y + ALTO_OCTETO - 4.0, x + w - 4.0, y + 4.0, "strokeSub", 1.2, None, "");
        }
        Clase::Relleno => {
            c.r(x, y, w, ALTO_OCTETO, "fillHeader", 1.1, 0.0);
            c.add(format!(
                r#"<path d="M {:.1} {:.1} L {:.1} {:.1} M {:.1} {:.1} L {:.1} {:.1}" stroke="{}" stroke-width="0.8" fill="none"/>"#,
                x + 2.0,
                y + ALTO_OCTETO - 2.0,
                x + w / 2.0,
                y + 2.0,
                x + w / 2.0,
                y + ALTO_OCTETO - 2.0,
                x + w - 2.0,
                y + 2.0,
                c.pal.stroke_muted
            ));
        }
        Clase::Truncado => c.add(format!(
            r#"<rect x="{x:.1}" y="{y:.1}" width="{w:.1}" height="{ALTO_OCTETO:.1}" fill="none" stroke="{}" stroke-width="1.2" stroke-dasharray="3 3"/>"#,
            c.pal.stroke_muted
        )),
    }
}

// ---------------------------------------------------------------------------
// Campos de un formato binario
// ---------------------------------------------------------------------------

#[derive(Debug)]
enum Pieza {
    Campo { nombre: String, octetos: Vec<u8>, rotulo: Option<String>, en_blanco: bool },
    Relleno(usize),
}

fn leer_piezas(b: &Bloque) -> Result<Vec<Pieza>, ErrorDiagrama> {
    let Some(lista) = b.get("fields").and_then(Value::as_array).filter(|l| !l.is_empty()) else {
        return Err(error_octetos("falta 'fields': la lista de campos, cada uno con name y size y value, o bytes"));
    };
    if lista.len() > MAX_CAMPOS {
        return Err(error_octetos(format!("hay {} campos; el máximo es {MAX_CAMPOS}", lista.len())));
    }
    let orden_general = orden_bytes(b.get("endian"), "endian")?.unwrap_or(false);
    let mut piezas = Vec::new();
    let mut offset = 0usize;
    for (i, f) in lista.iter().enumerate() {
        let Some(obj) = f.as_object() else {
            return Err(error_octetos(format!("campo {i}: se espera {{name, size, value}}, {{name, bytes}} o {{padding}}")));
        };
        if let Some(p) = obj.get("padding") {
            let n = entero_octeto(p, &format!("campo {i}, padding"))?;
            if !(1..=MAX_OCTETOS as i128).contains(&n) {
                return Err(error_octetos(format!("campo {i}: el relleno va de 1 a {MAX_OCTETOS} bytes")));
            }
            piezas.push(Pieza::Relleno(n as usize));
            offset += n as usize;
            continue;
        }
        let nombre = obj.get("name").filter(|v| es_truthy(v)).map(texto).unwrap_or_default();
        if nombre.is_empty() {
            return Err(error_octetos(format!("campo {i}: falta 'name'")));
        }
        let donde = format!("campo {nombre}");
        if let Some(o) = obj.get("offset").filter(|v| !v.is_null()) {
            let o = entero_octeto(o, &donde)?;
            if o < offset as i128 {
                return Err(error_octetos(format!("{donde}: el offset {o} se superpone con el campo anterior, que termina en {offset}")));
            }
            let hueco = (o - offset as i128) as usize;
            if hueco > MAX_OCTETOS {
                return Err(error_octetos(format!("{donde}: el offset {o} está fuera del rango dibujable")));
            }
            if hueco > 0 {
                piezas.push(Pieza::Relleno(hueco));
                offset += hueco;
            }
        }
        let big = orden_bytes(obj.get("endian"), &format!("{donde}, endian"))?.unwrap_or(orden_general);
        let (octetos, rotulo) = if let Some(v) = obj.get("bytes").filter(|v| !v.is_null()) {
            let Some(l) = v.as_array() else {
                return Err(error_octetos(format!("{donde}: 'bytes' es una lista de octetos")));
            };
            let octetos = l
                .iter()
                .map(|x| {
                    let n = entero_octeto(x, &donde)?;
                    u8::try_from(n).map_err(|_| error_octetos(format!("{donde}: un octeto va de 0 a 255 (0xFF), no {n}")))
                })
                .collect::<Result<Vec<u8>, _>>()?;
            (octetos, None)
        } else {
            let Some(tam) = obj.get("size").filter(|v| !v.is_null()) else {
                return Err(error_octetos(format!("{donde}: falta 'size' (en bytes) o 'bytes'")));
            };
            let tam = entero_octeto(tam, &donde)?;
            if !(1..=8).contains(&tam) {
                return Err(error_octetos(format!("{donde}: 'size' va de 1 a 8 bytes, no {tam}")));
            }
            let tam = tam as u32;
            let maximo: i128 = 1i128 << (8 * tam);
            let (octetos, rotulo) = match obj.get("value").filter(|v| !v.is_null()) {
                None => (vec![0u8; tam as usize], None),
                Some(v) => {
                    let n = entero_octeto(v, &donde)?;
                    if n >= maximo || n < -(maximo / 2) {
                        return Err(error_octetos(format!(
                            "{donde}: {} no entra en {tam} byte{} (va de {} a {})",
                            texto(v),
                            if tam == 1 { "" } else { "s" },
                            -(maximo / 2),
                            maximo - 1
                        )));
                    }
                    let u = if n < 0 { n + maximo } else { n } as u128;
                    let mut octetos: Vec<u8> = (0..tam).map(|k| (u >> (8 * k)) as u8).collect();
                    if big {
                        octetos.reverse();
                    }
                    let hex = format!("0x{:0ancho$X}", u, ancho = 2 * tam as usize);
                    let orden = if tam == 1 { String::new() } else if big { ", big-endian".to_string() } else { ", little-endian".to_string() };
                    (octetos, Some(format!("{hex}{orden}")))
                }
            };
            (octetos, rotulo)
        };
        if octetos.is_empty() {
            return Err(error_octetos(format!("{donde}: no tiene bytes")));
        }
        offset += octetos.len();
        piezas.push(Pieza::Campo { nombre, octetos, rotulo, en_blanco: obj.get("blank").is_some_and(es_truthy) });
    }
    if offset > MAX_OCTETOS {
        return Err(error_octetos(format!("el registro ocupa {offset} bytes; se dibujan hasta {MAX_OCTETOS}")));
    }
    Ok(piezas)
}

/// `big` (orden de red) es `true`; `little`, `false`.
fn orden_bytes(v: Option<&Value>, donde: &str) -> Result<Option<bool>, ErrorDiagrama> {
    match v.filter(|v| !v.is_null()).map(texto).as_deref() {
        None => Ok(None),
        Some("big") | Some("network") => Ok(Some(true)),
        Some("little") => Ok(Some(false)),
        Some(otro) => Err(error_octetos(format!("'{donde}' es big o little, no '{otro}'"))),
    }
}

fn render_campos(c: &mut Lienzo, b: &Bloque) -> Result<Punto, ErrorDiagrama> {
    let piezas = leer_piezas(b)?;
    let blanco = blancos(b, &["fields", "offsets", "bytes", "values"])?;
    let total: usize = piezas
        .iter()
        .map(|p| match p {
            Pieza::Campo { octetos, .. } => octetos.len(),
            Pieza::Relleno(n) => *n,
        })
        .sum();
    let disponibles = match b.get("length").filter(|v| !v.is_null()) {
        None => total,
        Some(v) => {
            let n = entero_octeto(v, "length")?;
            if n < 0 || n > total as i128 {
                return Err(error_octetos(format!("'length' va de 0 a {total} (lo que ocupan los campos): {n}")));
            }
            n as usize
        }
    };
    let top = encabezado(c, b, &format!("{total} bytes"));
    let x0 = 20.0_f64;
    let y = top + 30.0;
    let mut x = x0;
    let mut i = 0usize;
    let yb = y + ALTO_OCTETO + 6.0;
    let mut ancho = x0;
    let mut filas: Vec<Vec<String>> = Vec::new();
    let mut tenues: Vec<usize> = Vec::new();
    let mut llaves: Vec<(f64, f64, String)> = Vec::new();
    for p in &piezas {
        let (n, octetos, clase_base) = match p {
            Pieza::Campo { octetos, .. } => (octetos.len(), Some(octetos), Clase::Dato),
            Pieza::Relleno(n) => (*n, None, Clase::Relleno),
        };
        let x_ini = x;
        let mut truncados = 0;
        for k in 0..n {
            let clase = if i >= disponibles { Clase::Truncado } else { clase_base };
            if clase == Clase::Truncado {
                truncados += 1;
            }
            caja(c, x, y, ANCHO_OCTETO, clase);
            let rotulo_offset = if blanco.contains("offsets") { String::new() } else { format!("+{i}") };
            if !rotulo_offset.is_empty() {
                c.t(x + ANCHO_OCTETO / 2.0, y - 6.0, &rotulo_offset, 9.0, 400, "textSub", "middle", MONO);
            }
            let oculto = blanco.contains("bytes") || matches!(p, Pieza::Campo { en_blanco: true, .. });
            match (clase, octetos) {
                (Clase::Dato, Some(o)) if !oculto => {
                    c.t(x + ANCHO_OCTETO / 2.0, y + 21.0, &format!("{:02X}", o[k]), 12.0, 700, "textMain", "middle", MONO);
                }
                (Clase::Dato, _) => c.vacio(x + 6.0, x + ANCHO_OCTETO - 6.0, y + ALTO_OCTETO - 8.0),
                _ => {}
            }
            x += ANCHO_OCTETO;
            i += 1;
        }
        let nombre = match p {
            Pieza::Relleno(_) => "relleno".to_string(),
            Pieza::Campo { .. } if blanco.contains("fields") => "____".to_string(),
            Pieza::Campo { nombre, .. } => nombre.clone(),
        };
        let detalle = match p {
            Pieza::Relleno(_) => "relleno".to_string(),
            _ if truncados == n => "truncado".to_string(),
            _ if truncados > 0 => format!("truncado ({} de {n} B)", n - truncados),
            Pieza::Campo { rotulo: Some(r), en_blanco: false, .. } if !blanco.contains("values") => r.clone(),
            _ => "dato".to_string(),
        };
        let ofs = if blanco.contains("offsets") { String::new() } else { format!("+{}", i - n) };
        filas.push(vec![ofs, nombre.clone(), format!("{n} B"), detalle]);
        if matches!(p, Pieza::Relleno(_)) || truncados > 0 {
            tenues.push(filas.len() - 1);
        }
        llaves.push((x_ini, x, nombre));
    }
    // Los nombres en las llaves; si alguno no entra en la suya, se alternan en dos alturas.
    let alternar = llaves.iter().any(|(a, z, t)| ancho_texto(t, 10.0) > z - a - 4.0);
    for (k, (a, z, t)) in llaves.iter().enumerate() {
        let nivel = if alternar && k % 2 == 1 { 1.0 } else { 0.0 };
        c.linea(a + 2.0, yb, a + 2.0, yb + 5.0, "strokeSub", 1.1, None, "");
        c.linea(z - 2.0, yb, z - 2.0, yb + 5.0, "strokeSub", 1.1, None, "");
        c.linea(a + 2.0, yb + 5.0, z - 2.0, yb + 5.0, "strokeSub", 1.1, None, "");
        if nivel > 0.0 {
            c.linea((a + z) / 2.0, yb + 5.0, (a + z) / 2.0, yb + 20.0, "strokeMuted", 0.8, None, "");
        }
        c.t((a + z) / 2.0, yb + 18.0 + nivel * 15.0, t, 10.0, 700, "textMain", "middle", MONO);
        ancho = ancho.max((a + z) / 2.0 + ancho_texto(t, 10.0) / 2.0);
    }
    let y_tabla = yb + if alternar { 44.0 } else { 30.0 };
    let (w_tabla, h_tabla) = tabla(
        c,
        x0,
        y_tabla,
        &["offset", "campo", "tamaño", "valor"],
        &filas,
        TablaOpts { izquierda: &[1, 3], tenues: &tenues, fila_h: 22.0, ..TablaOpts::default() },
    );
    ancho = ancho.max(x0 + w_tabla);
    ancho = ancho.max(x);
    let pie = if disponibles < total {
        format!("se esperaban {total} bytes y hay {disponibles}: la entrada es corta")
    } else {
        format!("total: {total} bytes")
    };
    let y_pie = y_tabla + h_tabla + 18.0;
    c.t(x0, y_pie, &pie, 10.5, 700, "textMain", "start", MONO);
    ancho = ancho.max(x0 + ancho_texto(&pie, 10.5));
    Ok((ancho + 16.0, y_pie + 10.0))
}

// ---------------------------------------------------------------------------
// Filas de un BMP
// ---------------------------------------------------------------------------

/// Un píxel de 24 bits como `"FF0000"`, `"#FF0000"` o `[255, 0, 0]` (RGB); en el archivo va BGR.
fn pixel_24(v: &Value, donde: &str) -> Result<[u8; 3], ErrorDiagrama> {
    match v {
        Value::String(s) => {
            let h = s.trim().trim_start_matches('#');
            if h.len() != 6 || !h.chars().all(|ch| ch.is_ascii_hexdigit()) {
                return Err(error_octetos(format!("{donde}: un color es RRGGBB en hexadecimal, no {s:?}")));
            }
            let c = |k: usize| u8::from_str_radix(&h[k..k + 2], 16).unwrap();
            Ok([c(4), c(2), c(0)])
        }
        Value::Array(l) if l.len() == 3 => {
            let mut rgb = [0u8; 3];
            for (k, x) in l.iter().enumerate() {
                rgb[k] = x
                    .as_u64()
                    .and_then(|n| u8::try_from(n).ok())
                    .ok_or_else(|| error_octetos(format!("{donde}: cada componente va de 0 a 255")))?;
            }
            Ok([rgb[2], rgb[1], rgb[0]])
        }
        otro => Err(error_octetos(format!("{donde}: un píxel de 24 bits es \"RRGGBB\" o [r, g, b], no {}", texto(otro)))),
    }
}

fn dimension_bmp(b: &Bloque, clave: &str, maximo: u64) -> Result<u64, ErrorDiagrama> {
    let Some(v) = b.get(clave).filter(|v| !v.is_null()) else {
        return Err(error_octetos(format!("falta '{clave}'")));
    };
    let n = entero_octeto(v, clave)?;
    if n < 1 || n > maximo as i128 {
        return Err(error_octetos(format!(
            "'{clave}' = {n} está fuera del rango dibujable (de 1 a {maximo}); no se dibuja un rango imposible"
        )));
    }
    Ok(n as u64)
}

fn render_bmp(c: &mut Lienzo, b: &Bloque) -> Result<Punto, ErrorDiagrama> {
    let ancho_px = dimension_bmp(b, "width", MAX_ANCHO_BMP)? as usize;
    let alto_px = dimension_bmp(b, "height", MAX_ALTO_BMP)? as usize;
    let bpp = match b.get("bitsPerPixel").filter(|v| !v.is_null()).map(|v| entero_octeto(v, "bitsPerPixel")).transpose()? {
        None | Some(24) => 24usize,
        Some(8) => 8,
        Some(otro) => return Err(error_octetos(format!("'bitsPerPixel' es 24 u 8, no {otro}"))),
    };
    let offset = match b.get("offset").filter(|v| !v.is_null()) {
        None => 54usize,
        Some(v) => {
            let n = entero_octeto(v, "offset")?;
            usize::try_from(n).ok().filter(|n| *n <= 1 << 20).ok_or_else(|| error_octetos(format!("'offset' fuera de rango: {n}")))?
        }
    };
    let abajo_arriba = match b.get("order").filter(|v| !v.is_null()).map(texto).as_deref() {
        None | Some("bottom_up") => true,
        Some("top_down") => false,
        Some(otro) => return Err(error_octetos(format!("'order' es bottom_up o top_down, no '{otro}'"))),
    };
    let paleta = match b.get("paletteSize").filter(|v| !v.is_null()) {
        None => 256,
        Some(v) => {
            let n = entero_octeto(v, "paletteSize")?;
            if !(1..=256).contains(&n) {
                return Err(error_octetos(format!("'paletteSize' va de 1 a 256: {n}")));
            }
            n as usize
        }
    };
    let blanco = blancos(b, &["stride", "offsets", "bytes", "padding"])?;
    let utiles = ancho_px * bpp / 8;
    let stride = (ancho_px * bpp).div_ceil(32) * 4;
    let relleno = stride - utiles;
    if stride > 28 {
        return Err(error_octetos(format!("una fila de {stride} bytes no entra en el dibujo (hasta 28)")));
    }
    let total = stride * alto_px;
    let disponibles = match b.get("available").filter(|v| !v.is_null()) {
        None => total,
        Some(v) => {
            let n = entero_octeto(v, "available")?;
            if n < 0 || n > total as i128 {
                return Err(error_octetos(format!("'available' va de 0 a {total} (los bytes de las filas): {n}")));
            }
            n as usize
        }
    };

    // Las filas visuales, de arriba hacia abajo; sin `pixels`, los datos quedan para completar.
    let filas: Option<Vec<Vec<Vec<u8>>>> = match b.get("pixels").filter(|v| !v.is_null()) {
        None => None,
        Some(Value::Array(fs)) => {
            if fs.len() != alto_px {
                return Err(error_octetos(format!("'pixels' tiene {} filas y 'height' es {alto_px}", fs.len())));
            }
            let mut filas = Vec::new();
            for (r, f) in fs.iter().enumerate() {
                let Some(px) = f.as_array().filter(|p| p.len() == ancho_px) else {
                    return Err(error_octetos(format!("'pixels', fila {r}: se esperan {ancho_px} píxeles")));
                };
                let fila = px
                    .iter()
                    .enumerate()
                    .map(|(k, p)| {
                        let donde = format!("'pixels', fila {r}, píxel {k}");
                        if bpp == 24 {
                            pixel_24(p, &donde).map(|o| o.to_vec())
                        } else {
                            let n = entero_octeto(p, &donde)?;
                            u8::try_from(n).map(|o| vec![o]).map_err(|_| error_octetos(format!("{donde}: un índice va de 0 a 255")))
                        }
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                filas.push(fila);
            }
            Some(filas)
        }
        Some(_) => return Err(error_octetos("'pixels' es una lista de filas (la de arriba primero)")),
    };

    let formula = format!("stride = ((ancho × bpp + 31) / 32) × 4 = (({ancho_px} × {bpp} + 31) / 32) × 4");
    let stride_txt = if blanco.contains("stride") { "____".to_string() } else { stride.to_string() };
    let top = encabezado(c, b, &format!("{ancho_px} × {alto_px} píxeles, {bpp} bits, offset {offset}"));
    let x_rot = 20.0_f64;
    let x0 = 150.0_f64;
    c.t(x_rot, top + 14.0, &format!("{formula} = {stride_txt}"), 10.0, 700, "textMain", "start", MONO);
    let relleno_txt = if blanco.contains("stride") || blanco.contains("padding") { "____".to_string() } else { relleno.to_string() };
    let resumen = format!("datos por fila: {utiles} B · relleno: {relleno_txt} B");
    c.t(x_rot, top + 30.0, &resumen, 10.0, 400, "textSub", "start", MONO);
    let mut ancho = x_rot + ancho_texto(&format!("{formula} = {stride_txt}"), 10.0) + 10.0;

    let mut y = top + 62.0;
    let paso_fila = ALTO_OCTETO + 34.0;
    let mut invalidos = 0;
    for r in 0..alto_px {
        let visual = if abajo_arriba { alto_px - 1 - r } else { r };
        let inicio = offset + r * stride;
        let ofs = if blanco.contains("offsets") { "____".to_string() } else { inicio.to_string() };
        c.t(x_rot, y + 14.0, &format!("offset {ofs}"), 10.0, 700, "textMain", "start", MONO);
        let lugar = match (visual, alto_px) {
            (0, _) => "fila visual 0 (arriba)".to_string(),
            (v, a) if v + 1 == a => format!("fila visual {v} (abajo)"),
            (v, _) => format!("fila visual {v}"),
        };
        c.t(x_rot, y + 28.0, &lugar, 9.0, 400, "textSub", "start", SANS);
        let octetos: Option<Vec<(u8, bool)>> = filas.as_ref().map(|fs| {
            fs[visual]
                .iter()
                .flat_map(|p| {
                    let invalido = bpp == 8 && (p[0] as usize) >= paleta;
                    p.iter().map(move |o| (*o, invalido))
                })
                .collect()
        });
        for k in 0..stride {
            let x = x0 + k as f64 * ANCHO_OCTETO;
            let i = r * stride + k;
            let es_dato = k < utiles;
            let mut clase = if i >= disponibles {
                Clase::Truncado
            } else if es_dato {
                Clase::Dato
            } else {
                Clase::Relleno
            };
            let dato = octetos.as_ref().and_then(|o| o.get(k)).copied();
            if clase == Clase::Dato && dato.is_some_and(|d| d.1) && !blanco.contains("bytes") {
                clase = Clase::Invalido;
                invalidos += 1;
            }
            if clase == Clase::Relleno && blanco.contains("padding") {
                clase = Clase::Dato;
            }
            caja(c, x, y, ANCHO_OCTETO, clase);
            if r == 0 {
                c.t(x + ANCHO_OCTETO / 2.0, y - 6.0, &format!("+{k}"), 8.5, 400, "textMuted", "middle", MONO);
            }
            match clase {
                Clase::Dato | Clase::Invalido if es_dato => match dato {
                    Some((o, _)) if !blanco.contains("bytes") => {
                        c.t(x + ANCHO_OCTETO / 2.0, y + 21.0, &format!("{o:02X}"), 12.0, 700, "textMain", "middle", MONO);
                    }
                    _ => c.vacio(x + 6.0, x + ANCHO_OCTETO - 6.0, y + ALTO_OCTETO - 8.0),
                },
                Clase::Dato => c.vacio(x + 6.0, x + ANCHO_OCTETO - 6.0, y + ALTO_OCTETO - 8.0),
                _ => {}
            }
            // Los píxeles de 24 bits: una marca entre cada uno y su orden B G R.
            if bpp == 24 && es_dato && k % 3 == 0 && r == 0 {
                c.t(x + ANCHO_OCTETO * 1.5, y + ALTO_OCTETO + 11.0, &format!("píxel {} (B G R)", k / 3), 8.5, 400, "textMuted", "middle", SANS);
            }
        }
        // Rótulos por clase debajo de la fila: relleno y truncado.
        let xr = x0 + utiles as f64 * ANCHO_OCTETO;
        let fin = x0 + stride as f64 * ANCHO_OCTETO;
        let trunc_desde = disponibles.saturating_sub(r * stride).min(stride);
        if relleno > 0 && !blanco.contains("padding") && trunc_desde > utiles {
            c.t((xr + x0 + trunc_desde as f64 * ANCHO_OCTETO) / 2.0, y + ALTO_OCTETO + 11.0, "relleno", 8.5, 700, "textMuted", "middle", SANS);
        }
        if trunc_desde < stride {
            c.t((x0 + trunc_desde as f64 * ANCHO_OCTETO + fin) / 2.0, y + ALTO_OCTETO + 22.0, "truncado", 8.5, 700, "textMuted", "middle", SANS);
        }
        ancho = ancho.max(fin + 16.0);
        y += paso_fila;
    }
    let mut pie = vec![format!(
        "orden de filas: {}",
        if abajo_arriba { "de abajo hacia arriba (la primera del archivo es la de abajo)" } else { "de arriba hacia abajo" }
    )];
    if bpp == 8 {
        pie.push(format!(
            "paleta de {paleta} colores{}",
            if invalidos > 0 { ": un índice tachado está fuera de la paleta (el relleno no se valida)" } else { "" }
        ));
    }
    if disponibles < total {
        pie.push(format!("se esperaban {total} bytes de filas y hay {disponibles}"));
    }
    for (k, l) in pie.iter().enumerate() {
        c.t(x_rot, y + k as f64 * 15.0, l, 9.5, 400, "textSub", "start", SANS);
        ancho = ancho.max(x_rot + ancho_texto(l, 9.5) + 10.0);
    }
    Ok((ancho, y + pie.len() as f64 * 15.0))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn svg(bloque: Value) -> Result<String, ErrorDiagrama> {
        let mut reg = Registro::new();
        registrar(&mut reg);
        reg.renderizar(bloque.as_object().unwrap(), true)
    }

    #[test]
    fn orden_de_red_y_little_endian() {
        let s = svg(json!({"type": "byte_layout", "fields": [
            {"name": "red", "size": 2, "value": "0x1234", "endian": "big"},
            {"name": "host", "size": 2, "value": 4660}
        ]}))
        .unwrap();
        let datos: Vec<&str> = s.match_indices(">12<").chain(s.match_indices(">34<")).map(|(_, t)| t).collect();
        assert_eq!(datos.len(), 4);
        // 12 34 y después 34 12: el orden en el SVG es el de dibujo.
        let pos = |t: &str, desde: usize| s[desde..].find(t).unwrap() + desde;
        let a = pos(">12<", 0);
        let b = pos(">34<", a);
        let c2 = pos(">34<", b + 1);
        let d = pos(">12<", c2);
        assert!(a < b && b < c2 && c2 < d);
        assert!(s.contains("big-endian") && s.contains("little-endian"));
    }

    #[test]
    fn relleno_truncado_y_diagnosticos() {
        let s = svg(json!({"type": "byte_layout", "length": 4, "fields": [
            {"name": "tipo", "size": 1, "value": 2},
            {"padding": 1},
            {"name": "largo", "size": 4, "value": 9, "offset": 2}
        ]}))
        .unwrap();
        assert!(s.contains(">relleno<") && s.contains("truncado") && s.contains("la entrada es corta"));
        let e = svg(json!({"type": "byte_layout", "fields": [{"name": "x", "size": 2, "value": 70000}]})).unwrap_err().0;
        assert!(e.starts_with("byte_layout: ") && e.contains("no entra en 2 bytes"), "{e}");
        let e = svg(json!({"type": "byte_layout", "fields": [{"name": "x", "size": 2}, {"name": "y", "size": 1, "offset": 1}]})).unwrap_err().0;
        assert!(e.contains("se superpone"), "{e}");
        let blank = svg(json!({"type": "byte_layout", "blank": ["bytes", "offsets"], "fields": [{"name": "x", "size": 2, "value": "0xABCD"}]})).unwrap();
        assert!(!blank.contains(">AB<") && !blank.contains(">+1<"));
    }

    #[test]
    fn bmp_de_24_bits_stride_y_relleno() {
        let s = svg(json!({"type": "byte_layout", "variant": "bmp_rows", "width": 3, "height": 1,
            "pixels": [["FF0000", "00FF00", [0, 0, 255]]]}))
        .unwrap();
        assert!(s.contains("× 4 = 12<"), "stride 12");
        assert!(s.contains("datos por fila: 9 B · relleno: 3 B"));
        // El rojo se escribe 00 00 FF.
        let i = s.find(">FF<").unwrap();
        assert_eq!(s[..i].matches(">00<").count(), 2);
    }

    #[test]
    fn bmp_indexado_y_orden_de_filas() {
        let s = svg(json!({"type": "byte_layout", "variant": "bmp_rows", "bitsPerPixel": 8, "width": 5, "height": 1,
            "paletteSize": 4, "pixels": [[0, 1, 9, 3, 2]]}))
        .unwrap();
        assert!(s.contains("× 4 = 8<") && s.contains("fuera de la paleta"));
        let abajo = svg(json!({"type": "byte_layout", "variant": "bmp_rows", "width": 2, "height": 3, "offset": 54})).unwrap();
        let arriba = svg(json!({"type": "byte_layout", "variant": "bmp_rows", "width": 2, "height": 3, "offset": 54, "order": "top_down"})).unwrap();
        // La fila visual de arriba empieza en 70 de abajo hacia arriba y en 54 de arriba hacia abajo.
        let fila_de_arriba = |s: &str| {
            let i = s.find("fila visual 0 (arriba)").unwrap();
            let antes = &s[..i];
            let j = antes.rfind(">offset ").unwrap() + 8;
            antes[j..j + 2].to_string()
        };
        assert_eq!(fila_de_arriba(&abajo), "70");
        assert_eq!(fila_de_arriba(&arriba), "54");
        let e = svg(json!({"type": "byte_layout", "variant": "bmp_rows", "width": 0, "height": 1})).unwrap_err().0;
        assert!(e.contains("fuera del rango dibujable"), "{e}");
        let corto = svg(json!({"type": "byte_layout", "variant": "bmp_rows", "width": 2, "height": 2, "available": 10})).unwrap();
        assert!(corto.contains(">truncado<") && corto.contains("hay 10"));
    }
}
