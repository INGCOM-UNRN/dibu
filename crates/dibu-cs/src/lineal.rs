//! Renderizadores lineales de `dibu-cs` (`array`, `matrix`, `linked_list`, `stack`, `queue`).

use crate::common::{
    ancho_cabecera, ancho_texto, cabecera, campos_cabecera, encabezado, entero_b, enteros_set, es_truthy,
    fmt_hex, fmt_num_clean, hex, hex_b, lista_o_vacia, nombres_de_zonas, punteros, str_o,
    sub_lienzo, agregar_sub_lienzo, texto, texto_opt, valores, zona, Bloque, ErrorDiagrama,
    Lienzo, Punto, ALTO_ZONA, FILA_CABECERA, MONO, SANS,
};
use dibu_core::Registro;
use serde_json::Value;
use std::collections::{HashMap, HashSet};
use std::f64::consts::{FRAC_PI_2, PI};
use std::num::IntErrorKind;

const MAX_FILAS: usize = 10;
const MAX_COLUMNAS: usize = 10;
const MAX_CELDAS_PLANA: usize = 24;

pub fn registrar(reg: &mut Registro) {
    reg.registrar("array", render_array);
    reg.registrar("matrix", render_matrix);
    reg.registrar("linked_list", render_linked_list);
    reg.registrar("stack", render_stack);
    reg.registrar("queue", render_queue);
}

#[inline]
fn flag_f(b: &Bloque, clave: &str) -> bool {
    b.get(clave).is_some_and(es_truthy)
}

fn envolver_tipo(tipo: &str, res: Result<Punto, ErrorDiagrama>) -> Result<Punto, ErrorDiagrama> {
    res.map_err(|e| {
        if e.0.strip_prefix(tipo).is_some_and(|r| r.starts_with(": ")) {
            e
        } else {
            ErrorDiagrama::new(format!("{tipo}: {}", e.0))
        }
    })
}

#[inline]
fn flag_t(b: &Bloque, clave: &str) -> bool {
    b.get(clave).map(es_truthy).unwrap_or(true)
}

fn capacidad(b: &Bloque, len_valores: usize, defecto: i64, minimo: i64, maximo: i64) -> usize {
    if b.contains_key("capacity") {
        entero_b(b, "capacity", defecto, minimo, maximo) as usize
    } else {
        let base = if len_valores > 0 {
            len_valores as i64
        } else {
            defecto
        };
        base.clamp(minimo, maximo) as usize
    }
}

fn valor_py(v: &Value) -> String {
    if v.is_null() {
        "None".to_string()
    } else {
        texto(v)
    }
}

fn repr_val(v: &Value) -> String {
    match v {
        Value::Null => "None".to_string(),
        Value::String(s) => format!("'{s}'"),
        Value::Bool(true) => "True".to_string(),
        Value::Bool(false) => "False".to_string(),
        _ => v.to_string(),
    }
}

// ---------------------------------------------------------------------------
// Arreglo (`array`)
// ---------------------------------------------------------------------------

fn render_array(c: &mut Lienzo, b: &Bloque) -> Result<Punto, ErrorDiagrama> {
    envolver_tipo("array", render_array_inner(c, b))
}

fn render_array_inner(c: &mut Lienzo, b: &Bloque) -> Result<Punto, ErrorDiagrama> {
    match b.get("variant") {
        None | Some(Value::Null) => {}
        Some(Value::String(v)) if v == "python_sequence" => return render_secuencia_python(c, b),
        Some(Value::String(v)) if v == "c_string" => return render_cadena_c(c, b),
        Some(otro) => {
            return Err(ErrorDiagrama::new(format!(
                "variante de arreglo no soportada: {} (python_sequence o c_string)",
                repr_val(otro)
            )));
        }
    }
    let vals = valores(b)?;
    let cap = capacidad(b, vals.len(), 8, 1, 16);
    let base = hex_b(b, "baseAddr", 0x1000);
    let paso = entero_b(b, "typeSize", 4, 1, 64);
    let direcciones = flag_f(b, "showAddresses");
    let offsets = flag_f(b, "showOffsets");
    let indices = flag_t(b, "showIndices");
    let sin_valores = flag_f(b, "blankValues");
    let sin_dirs = flag_f(b, "blankAddrs");

    let extra = if direcciones {
        format!("Base: {} | sizeof: {paso}B", fmt_hex(base))
    } else {
        String::new()
    };
    let top = encabezado(c, b, &extra);
    // Un puntero en `n` señala el límite excluido: el borde derecho, sin una celda `a[n]`.
    let ptrs = punteros(b, cap + 1);
    let rango = leer_rango(b, cap)?;

    let x0 = 78.0_f64;
    let celda_w = 64.0_f64;
    let celda_h = 54.0_f64;
    let y = top + if ptrs.is_empty() { 30.0 } else { 64.0 };

    if indices {
        c.t(x0 - 10.0, y - 8.0, "Índice [i]:", 10.0, 600, "textSub", "end", MONO);
    }
    c.t(
        x0 - 10.0,
        y + celda_h / 2.0 + 4.0,
        "Valor:",
        10.5,
        700,
        "textMain",
        "end",
        MONO,
    );
    if direcciones {
        c.t(
            x0 - 10.0,
            y + celda_h + 17.0,
            "Dirección:",
            10.0,
            600,
            "textSub",
            "end",
            MONO,
        );
    }
    if offsets {
        c.t(
            x0 - 10.0,
            y + celda_h + 32.0,
            "Offset:",
            9.5,
            400,
            "textMuted",
            "end",
            MONO,
        );
    }

    for i in 0..cap {
        let cx = x0 + i as f64 * celda_w;
        let centro = cx + celda_w / 2.0;
        if let Some(etiquetas) = ptrs.get(&i) {
            insignia_puntero(c, centro, y, &etiquetas.join(", "));
        }
        if indices {
            c.t(
                centro,
                y - 7.0,
                &format!("[{i}]"),
                11.0,
                700,
                "textSub",
                "middle",
                MONO,
            );
        }
        c.r(cx, y, celda_w, celda_h, "fillCell", 1.75, 0.0);
        let valor = vals.get(i).map(String::as_str).unwrap_or("");
        if !valor.is_empty() && !sin_valores {
            c.t(
                centro,
                y + celda_h / 2.0 + 5.0,
                valor,
                15.0,
                700,
                "textMain",
                "middle",
                MONO,
            );
        } else {
            c.vacio(cx + 10.0, cx + celda_w - 10.0, y + celda_h - 12.0);
        }
        if direcciones {
            let dir_txt = if sin_dirs {
                "0x____".to_string()
            } else {
                fmt_hex(base.saturating_add((i as i64).saturating_mul(paso)))
            };
            c.t(
                centro,
                y + celda_h + 17.0,
                &dir_txt,
                10.0,
                600,
                "textSub",
                "middle",
                MONO,
            );
        }
        if offsets {
            let off = (i as i64).saturating_mul(paso);
            c.t(
                centro,
                y + celda_h + 32.0,
                &format!("+{off}B"),
                9.0,
                400,
                "textMuted",
                "middle",
                MONO,
            );
        }
    }

    let mut ancho = x0 + cap as f64 * celda_w + 20.0;
    if let Some(etiquetas) = ptrs.get(&cap) {
        ancho = ancho.max(limite_excluido(c, x0 + cap as f64 * celda_w, y, celda_h, &etiquetas.join(", ")));
    }
    let mut alto = y + celda_h + if offsets { 42.0 } else if direcciones { 26.0 } else { 12.0 };
    if let Some(r) = &rango {
        let (a, w) = dibujar_rango(c, r, x0, celda_w, alto + 4.0);
        alto = a;
        ancho = ancho.max(w);
    }
    Ok((ancho, alto))
}

/// La insignia de un puntero sobre la celda centrada en `centro`, con su flecha.
fn insignia_puntero(c: &mut Lienzo, centro: f64, y: f64, txt: &str) -> f64 {
    let ancho = 40.0_f64.max(txt.chars().count() as f64 * 7.5 + 12.0);
    c.r(centro - ancho / 2.0, y - 48.0, ancho, 18.0, "fillBadge", 1.2, 3.0);
    c.t(centro, y - 35.0, txt, 10.0, 700, "textMain", "middle", MONO);
    c.linea(centro, y - 30.0, centro, y - 19.0, "accentPrimary", 1.5, Some("main"), "");
    centro + ancho / 2.0
}

/// Un puntero al límite excluido: la insignia sobre el borde derecho y una marca punteada, sin
/// celda. Devuelve hasta dónde llega a la derecha.
fn limite_excluido(c: &mut Lienzo, x: f64, y: f64, alto: f64, txt: &str) -> f64 {
    c.linea(x, y - 16.0, x, y + alto + 6.0, "strokeMain", 1.6, None, "4 2");
    insignia_puntero(c, x, y, txt) + 6.0
}

/// Un intervalo semiabierto `[start, stop)` sobre un arreglo, con un orden de visita opcional.
struct Rango {
    inicio: usize,
    fin: usize,
    rotulo: Option<String>,
    orden: Option<bool>,
    en_blanco: bool,
}

fn leer_rango(b: &Bloque, n: usize) -> Result<Option<Rango>, ErrorDiagrama> {
    let Some(v) = b.get("range").filter(|v| !v.is_null()) else {
        return Ok(None);
    };
    let Some(r) = v.as_object() else {
        return Err(ErrorDiagrama::new("'range' tiene que ser {start, stop, label, order, blank}"));
    };
    let limite = |clave: &str, defecto: usize| -> Result<usize, ErrorDiagrama> {
        match r.get(clave).filter(|v| !v.is_null()) {
            None => Ok(defecto),
            Some(v) => match v.as_i64() {
                Some(i) if i >= 0 && i as usize <= n => Ok(i as usize),
                _ => Err(ErrorDiagrama::new(format!(
                    "'range.{clave}' tiene que ser un entero entre 0 y {n} (el límite excluido): {}",
                    texto(v)
                ))),
            },
        }
    };
    let inicio = limite("start", 0)?;
    let fin = limite("stop", n)?;
    if inicio > fin {
        return Err(ErrorDiagrama::new(format!("'range': start ({inicio}) es mayor que stop ({fin})")));
    }
    let orden = match r.get("order").filter(|v| !v.is_null()) {
        None => None,
        Some(Value::Bool(false)) => None,
        Some(Value::Bool(true)) => Some(true),
        Some(Value::String(s)) if s == "forward" => Some(true),
        Some(Value::String(s)) if s == "reverse" => Some(false),
        Some(otro) => {
            return Err(ErrorDiagrama::new(format!(
                "'range.order' es true, forward o reverse: {}",
                texto(otro)
            )));
        }
    };
    Ok(Some(Rango {
        inicio,
        fin,
        rotulo: r.get("label").filter(|v| es_truthy(v)).map(texto),
        orden,
        en_blanco: r.get("blank").is_some_and(es_truthy),
    }))
}

/// Dibuja el intervalo debajo de las celdas, desde `y`; devuelve dónde termina, abajo y a la
/// derecha.
fn dibujar_rango(c: &mut Lienzo, r: &Rango, x0: f64, celda_w: f64, y: f64) -> Punto {
    let xa = x0 + r.inicio as f64 * celda_w;
    let xb = x0 + r.fin as f64 * celda_w;
    let mut y = y;
    if let (Some(adelante), false) = (r.orden, r.en_blanco) {
        let k = r.fin - r.inicio;
        for (n, i) in (r.inicio..r.fin).enumerate() {
            let centro = x0 + i as f64 * celda_w + celda_w / 2.0;
            let numero = if adelante { n + 1 } else { k - n };
            c.add(format!(
                r#"<circle cx="{centro:.1}" cy="{:.1}" r="8" fill="none" stroke="{}" stroke-width="1.1"/>"#,
                y + 8.0,
                c.pal.stroke_sub
            ));
            c.t(centro, y + 11.5, &numero.to_string(), 9.5, 700, "textSub", "middle", MONO);
        }
        if k > 0 {
            y += 20.0;
        }
    }
    let ym = y + 10.0;
    if r.en_blanco {
        c.vacio(x0, x0 + celda_w * 2.0, ym + 4.0);
    } else {
        // `[` en el inicio (incluido) y `)` en el fin (excluido); vacío si coinciden.
        c.linea(xa, ym, xb.max(xa), ym, "strokeMain", 1.4, None, "");
        c.t(xa, ym + 5.0, "[", 15.0, 700, "textMain", "middle", MONO);
        c.t(xb + if xa == xb { 7.0 } else { 0.0 }, ym + 5.0, ")", 15.0, 700, "textMain", "middle", MONO);
    }
    let rotulo = match (&r.rotulo, r.inicio == r.fin && !r.en_blanco) {
        _ if r.en_blanco => r.rotulo.clone().unwrap_or_default(),
        (Some(t), true) => format!("{t} (vacío)"),
        (Some(t), false) => t.clone(),
        (None, true) => format!("[{}, {}) vacío", r.inicio, r.fin),
        (None, false) => format!("[{}, {})", r.inicio, r.fin),
    };
    let centro = if r.en_blanco { x0 + celda_w } else { (xa + xb) / 2.0 };
    c.t(centro, ym + 22.0, &rotulo, 10.0, 600, "textSub", "middle", MONO);
    (ym + 30.0, (centro + ancho_texto(&rotulo, 10.0) / 2.0).max(xb + 8.0) + 8.0)
}

// ---------------------------------------------------------------------------
// Matriz (`matrix`: static, dynamic, pointers, jagged, flat)
// ---------------------------------------------------------------------------

fn dimension(b: &Bloque, clave: &str, defecto: usize, maximo: usize) -> Result<usize, ErrorDiagrama> {
    let Some(valor) = b.get(clave).filter(|v| !v.is_null()) else {
        let n = defecto as i64;
        if !(1..=maximo as i64).contains(&n) {
            return Err(ErrorDiagrama::new(format!(
                "'{clave}' de una matriz tiene que estar entre 1 y {maximo}: {n}"
            )));
        }
        return Ok(defecto);
    };

    let n: i64 = match valor {
        Value::Bool(v) => i64::from(*v),
        Value::Number(num) => {
            if let Some(i) = num.as_i64() {
                i
            } else if let Some(u) = num.as_u64() {
                if u <= maximo as u64 {
                    u as i64
                } else {
                    return Err(ErrorDiagrama::new(format!(
                        "'{clave}' de una matriz tiene que estar entre 1 y {maximo}: {u}"
                    )));
                }
            } else if let Some(f) = num.as_f64() {
                if !f.is_finite() {
                    return Err(ErrorDiagrama::new(format!(
                        "'{clave}' de una matriz tiene que ser un entero: {valor}"
                    )));
                }
                let t = f.trunc();
                if t < 1.0 || t > maximo as f64 {
                    return Err(ErrorDiagrama::new(format!(
                        "'{clave}' de una matriz tiene que estar entre 1 y {maximo}: {t}"
                    )));
                }
                t as i64
            } else {
                return Err(ErrorDiagrama::new(format!(
                    "'{clave}' de una matriz tiene que ser un entero: {valor}"
                )));
            }
        }
        Value::String(s) => match s.trim().parse::<i64>() {
            Ok(i) => i,
            Err(e) if matches!(e.kind(), IntErrorKind::PosOverflow | IntErrorKind::NegOverflow) => {
                return Err(ErrorDiagrama::new(format!(
                    "'{clave}' de una matriz tiene que estar entre 1 y {maximo}: {}",
                    s.trim()
                )));
            }
            Err(_) => {
                return Err(ErrorDiagrama::new(format!(
                    "'{clave}' de una matriz tiene que ser un entero: '{}'",
                    s
                )));
            }
        },
        _ => {
            return Err(ErrorDiagrama::new(format!(
                "'{clave}' de una matriz tiene que ser un entero: {}",
                repr_val(valor)
            )));
        }
    };

    if !(1..=maximo as i64).contains(&n) {
        return Err(ErrorDiagrama::new(format!(
            "'{clave}' de una matriz tiene que estar entre 1 y {maximo}: {n}"
        )));
    }
    Ok(n as usize)
}

fn filas_matriz(b: &Bloque) -> Result<Vec<Option<Vec<String>>>, ErrorDiagrama> {
    let Some(raw) = b.get("values").filter(|v| es_truthy(v)) else {
        return Ok(Vec::new());
    };
    let Some(arr) = raw.as_array() else {
        return Err(ErrorDiagrama::new(
            "'values' de una matriz tiene que ser una lista de filas",
        ));
    };

    if !arr.is_empty() && !arr.iter().any(Value::is_array) {
        let cols = dimension(b, "cols", arr.len().min(MAX_COLUMNAS), MAX_COLUMNAS)?;
        let num_filas = arr.len().div_ceil(cols);
        if num_filas > MAX_FILAS {
            return Err(ErrorDiagrama::new(format!(
                "la matriz tiene {num_filas} filas; el máximo es {MAX_FILAS}"
            )));
        }
        let mut filas = Vec::with_capacity(num_filas);
        for chunk in arr.chunks(cols) {
            let fila = chunk
                .iter()
                .map(|v| if v.is_null() { String::new() } else { texto(v) })
                .collect();
            filas.push(Some(fila));
        }
        return Ok(filas);
    }

    if arr.len() > MAX_FILAS {
        return Err(ErrorDiagrama::new(format!(
            "la matriz tiene {} filas; el máximo es {MAX_FILAS}",
            arr.len()
        )));
    }

    let mut filas = Vec::with_capacity(arr.len());
    for (i, item) in arr.iter().enumerate() {
        match item {
            Value::Null => filas.push(None),
            Value::Array(fila) => {
                if fila.len() > MAX_COLUMNAS {
                    return Err(ErrorDiagrama::new(format!(
                        "la fila {i} de la matriz tiene {} columnas; el máximo es {MAX_COLUMNAS}",
                        fila.len()
                    )));
                }
                filas.push(Some(
                    fila.iter()
                        .map(|v| if v.is_null() { String::new() } else { texto(v) })
                        .collect(),
                ));
            }
            _ => {
                return Err(ErrorDiagrama::new(
                    "cada fila de una matriz tiene que ser una lista (o null)",
                ));
            }
        }
    }
    Ok(filas)
}

fn parse_int_par(v: &Value) -> Option<i64> {
    match v {
        Value::Number(n) => n.as_i64().or_else(|| n.as_u64().and_then(|u| i64::try_from(u).ok())),
        Value::String(s) => s.trim().parse::<i64>().ok(),
        _ => None,
    }
}

fn pares_highlight(b: &Bloque) -> Result<HashSet<(i64, i64)>, ErrorDiagrama> {
    let mut celdas = HashSet::new();
    let Some(valor) = b.get("highlight").filter(|v| es_truthy(v)) else {
        return Ok(celdas);
    };
    match valor {
        Value::String(s) => {
            for seg in s.split(';').filter(|p| !p.trim().is_empty()) {
                let partes: Vec<&str> = seg.split(',').collect();
                if partes.len() != 2 {
                    return Err(ErrorDiagrama::new(format!(
                        "celda de 'highlight' inválida: {seg:?} (se espera [fila, columna])"
                    )));
                }
                let Ok(i) = partes[0].trim().parse::<i64>() else {
                    return Err(ErrorDiagrama::new(format!(
                        "celda de 'highlight' inválida: {seg:?} (se espera [fila, columna])"
                    )));
                };
                let Ok(j) = partes[1].trim().parse::<i64>() else {
                    return Err(ErrorDiagrama::new(format!(
                        "celda de 'highlight' inválida: {seg:?} (se espera [fila, columna])"
                    )));
                };
                celdas.insert((i, j));
            }
        }
        Value::Array(arr) => {
            for par in arr {
                let Some(items) = par.as_array().filter(|a| a.len() == 2) else {
                    return Err(ErrorDiagrama::new(format!(
                        "celda de 'highlight' inválida: {par} (se espera [fila, columna])"
                    )));
                };
                let (Some(i), Some(j)) = (parse_int_par(&items[0]), parse_int_par(&items[1])) else {
                    return Err(ErrorDiagrama::new(format!(
                        "celda de 'highlight' inválida: {par} (se espera [fila, columna])"
                    )));
                };
                celdas.insert((i, j));
            }
        }
        otro => {
            return Err(ErrorDiagrama::new(format!(
                "celda de 'highlight' inválida: {} (se espera [fila, columna])",
                repr_val(otro)
            )));
        }
    }
    Ok(celdas)
}

fn render_matrix(c: &mut Lienzo, b: &Bloque) -> Result<Punto, ErrorDiagrama> {
    envolver_tipo("matrix", render_matrix_inner(c, b))
}

fn render_matrix_inner(c: &mut Lienzo, b: &Bloque) -> Result<Punto, ErrorDiagrama> {
    let variante = match b.get("variant") {
        None => "static",
        Some(Value::String(s)) => s.as_str(),
        Some(otro) => {
            return Err(ErrorDiagrama::new(format!(
                "variante de matriz no soportada: {} (static, dynamic, jagged, flat, row_pointers o java)",
                repr_val(otro)
            )));
        }
    };

    match variante {
        "dynamic" | "pointers" => render_matrix_dinamica(c, b, false),
        "jagged" => render_matrix_dinamica(c, b, true),
        "flat" => render_matrix_plana(c, b),
        "static" => render_matrix_estatica(c, b),
        "java" => render_matrix_java(c, b),
        "row_pointers" => render_matrix_punteros_bloque(c, b),
        otra => Err(ErrorDiagrama::new(format!(
            "variante de matriz no soportada: '{otra}' (static, dynamic, jagged, flat, row_pointers o java)"
        ))),
    }
}

/// Las filas de una matriz de Java: `None` es una referencia `null`; `Some(vec![])`, un arreglo de
/// largo cero. Sin `values`, `rows` filas de `cols` celdas vacías.
fn filas_matriz_java(b: &Bloque) -> Result<Vec<Option<Vec<String>>>, ErrorDiagrama> {
    let Some(raw) = b.get("values").filter(|v| !v.is_null()) else {
        let nf = dimension(b, "rows", 3, MAX_FILAS)?;
        let nc = dimension(b, "cols", 3, MAX_COLUMNAS)?;
        return Ok(vec![Some(vec![String::new(); nc]); nf]);
    };
    let Some(arr) = raw.as_array() else {
        return Err(ErrorDiagrama::new(
            "'values' de una matriz de Java tiene que ser una lista de filas (listas o null)",
        ));
    };
    if arr.len() > MAX_FILAS {
        return Err(ErrorDiagrama::new(format!(
            "la matriz tiene {} filas; el máximo es {MAX_FILAS}",
            arr.len()
        )));
    }
    arr.iter()
        .enumerate()
        .map(|(i, item)| match item {
            Value::Null => Ok(None),
            Value::Array(fila) if fila.len() > MAX_COLUMNAS => Err(ErrorDiagrama::new(format!(
                "la fila {i} de la matriz tiene {} columnas; el máximo es {MAX_COLUMNAS}",
                fila.len()
            ))),
            Value::Array(fila) => Ok(Some(
                fila.iter()
                    .map(|v| if v.is_null() { String::new() } else { texto(v) })
                    .collect(),
            )),
            _ => Err(ErrorDiagrama::new(format!(
                "la fila {i} de una matriz de Java tiene que ser una lista (o null)"
            ))),
        })
        .collect()
}

/// Una matriz de C con una tabla de punteros a filas y un solo bloque de datos: `m[i]` apunta al
/// índice `i * C` del bloque. Son dos reservas, y se liberan las dos.
fn render_matrix_punteros_bloque(c: &mut Lienzo, b: &Bloque) -> Result<Punto, ErrorDiagrama> {
    let filas = filas_matriz(b)?;
    let nf = dimension(b, "rows", if filas.is_empty() { 2 } else { filas.len() }, MAX_FILAS)?;
    let nc_datos = filas.iter().flatten().map(Vec::len).max().unwrap_or(0);
    let nc = dimension(b, "cols", if nc_datos == 0 { 3 } else { nc_datos }, MAX_COLUMNAS)?;
    if let Some((i, f)) = filas.iter().enumerate().find(|(_, f)| f.as_ref().is_some_and(|f| f.len() > nc)) {
        return Err(ErrorDiagrama::new(format!(
            "la fila {i} tiene {} valores y la matriz {nc} columnas",
            f.as_ref().map_or(0, Vec::len)
        )));
    }
    if nf * nc > MAX_CELDAS_PLANA {
        return Err(ErrorDiagrama::new(format!(
            "{nf} × {nc} = {} celdas; el bloque se dibuja con hasta {MAX_CELDAS_PLANA}",
            nf * nc
        )));
    }
    let nombre = str_o(b, "name", "m");
    let tipo = str_o(b, "elementType", "int");
    let sin_valores = flag_f(b, "blankValues");
    let sin_punteros = flag_f(b, "blankPointers");
    let resaltadas = pares_highlight(b)?;
    let top = encabezado(c, b, &format!("{tipo} **{nombre}: {nf} punteros a filas y un bloque de {nf} × {nc}"));

    let x0 = 20.0_f64;
    let celda_w = 44.0_f64;
    let celda_h = 36.0_f64;
    let ptr_w = 64.0_f64;
    // La variable, la tabla de punteros (reserva 1) y el bloque de datos (reserva 2).
    let y_var = top + 10.0;
    c.t(x0, y_var + 16.0, &format!("{tipo} **{nombre}"), 11.0, 700, "textMain", "start", MONO);
    let x_var = x0 + ancho_texto(&format!("{tipo} **{nombre}"), 11.0) + 10.0;
    c.r(x_var, y_var, 30.0, 24.0, "fillPointerBox", 1.4, 2.0);
    let y_ptr = y_var + 62.0;
    let x_ptr = x0 + 120.0;
    c.punto(x_var + 15.0, y_var + 12.0, "accentPrimary", 3.2);
    c.ruta(&[(x_var + 15.0, y_var + 12.0), (x_var + 15.0, y_ptr + celda_h / 2.0), (x_ptr - 3.0, y_ptr + celda_h / 2.0)]);
    c.t(x_ptr, y_ptr - 22.0, &format!("reserva 1: {nf} × sizeof({tipo} *)"), 9.5, 700, "textSub", "start", SANS);
    let y_dat = y_ptr + celda_h + 74.0;
    let x_dat = x0;
    for i in 0..nf {
        let x = x_ptr + i as f64 * ptr_w;
        c.r(x, y_ptr, ptr_w, celda_h, "fillPointerBox", 1.5, 0.0);
        c.t(x + ptr_w / 2.0, y_ptr - 6.0, &format!("{nombre}[{i}]"), 9.5, 700, "textSub", "middle", MONO);
        if sin_punteros {
            c.vacio(x + 8.0, x + ptr_w - 8.0, y_ptr + celda_h - 9.0);
            continue;
        }
        let desde = (x + ptr_w / 2.0, y_ptr + celda_h / 2.0);
        c.punto(desde.0, desde.1, "accentPrimary", 3.2);
        let hasta = x_dat + (i * nc) as f64 * celda_w + 6.0;
        c.ruta(&[desde, (desde.0, y_ptr + celda_h + 16.0 + i as f64 * 6.0), (hasta, y_ptr + celda_h + 16.0 + i as f64 * 6.0), (hasta, y_dat - 3.0)]);
    }
    let rotulo_datos = format!("reserva 2: {nf} × {nc} × sizeof({tipo}), contiguos");
    for i in 0..nf {
        for j in 0..nc {
            let k = i * nc + j;
            let x = x_dat + k as f64 * celda_w;
            let marcada = resaltadas.contains(&(i as i64, j as i64));
            c.r(x, y_dat, celda_w, celda_h, if marcada { "fillBadge" } else { "fillCell" }, if j == 0 { 2.0 } else { 1.3 }, 0.0);
            let valor = filas.get(i).and_then(|f| f.as_ref()).and_then(|f| f.get(j)).map(String::as_str).unwrap_or("");
            if sin_valores || valor.is_empty() {
                c.vacio(x + 8.0, x + celda_w - 8.0, y_dat + celda_h - 9.0);
            } else {
                c.t(x + celda_w / 2.0, y_dat + celda_h / 2.0 + 5.0, valor, 13.0, 700, "textMain", "middle", MONO);
            }
            c.t(x + celda_w / 2.0, y_dat + celda_h + 13.0, &format!("[{k}]"), 9.0, 400, "textMuted", "middle", MONO);
        }
        // La fila i ocupa del índice i*C al (i+1)*C - 1 del bloque.
        let xa = x_dat + (i * nc) as f64 * celda_w + 3.0;
        let xz = x_dat + ((i + 1) * nc) as f64 * celda_w - 3.0;
        let yb = y_dat + celda_h + 20.0;
        c.linea(xa, yb, xa, yb + 5.0, "strokeSub", 1.1, None, "");
        c.linea(xz, yb, xz, yb + 5.0, "strokeSub", 1.1, None, "");
        c.linea(xa, yb + 5.0, xz, yb + 5.0, "strokeSub", 1.1, None, "");
        c.t((xa + xz) / 2.0, yb + 18.0, &format!("fila {i}: desde [{}]", i * nc), 9.0, 600, "textSub", "middle", SANS);
    }
    c.t(x_dat, y_dat + celda_h + 60.0, &rotulo_datos, 9.5, 700, "textSub", "start", SANS);
    let y_pie = y_dat + celda_h + 84.0;
    let pie = format!("liberar las dos reservas: free({nombre}[0]); free({nombre});");
    c.t(x0, y_pie, &pie, 10.5, 700, "textMain", "start", MONO);
    let ancho = (x_dat + (nf * nc) as f64 * celda_w)
        .max(x_ptr + nf as f64 * ptr_w)
        .max(x0 + ancho_texto(&pie, 10.5))
        .max(x_ptr + ancho_texto(&format!("reserva 1: {nf} × sizeof({tipo} *)"), 9.5));
    Ok((ancho + 20.0, y_pie + 10.0))
}

/// `int[][]` de Java: la variable en el stack refiere al arreglo exterior, que guarda una
/// referencia por fila; cada fila es un objeto aparte, con su propio largo.
fn render_matrix_java(c: &mut Lienzo, b: &Bloque) -> Result<Punto, ErrorDiagrama> {
    let filas = filas_matriz_java(b)?;
    let nf = filas.len();
    let nombre = str_o(b, "name", "matriz");
    let tipo = str_o(b, "elementType", "int");
    let indices = flag_t(b, "showIndices");
    let sin_valores = flag_f(b, "blankValues");
    let sin_largos = flag_f(b, "blankLengths");
    let resaltadas = pares_highlight(b)?;
    let zonas = nombres_de_zonas(b);

    let inicio = encabezado(c, b, &format!("{tipo}[][] {nombre}"));
    let top = inicio + if zonas.is_some() { ALTO_ZONA } else { 0.0 };

    let celda_w = 50.0_f64;
    let celda_h = 40.0_f64;
    let paso_fila = 54.0_f64;
    let vacio_w = 14.0_f64;
    let y0 = top + 34.0;

    // La variable, en el stack.
    let var_w = 56.0_f64.max((nombre.chars().count() as f64 * 8.5 + 14.0).ceil());
    c.t(var_w / 2.0, y0 - 8.0, &nombre, 12.0, 700, "textMain", "middle", MONO);
    c.r(0.0, y0, var_w, celda_h, "fillPointerBox", 1.75, 3.0);
    c.punto(var_w / 2.0, y0 + celda_h / 2.0, "accentPrimary", 3.2);

    // El arreglo exterior: una referencia por fila.
    let x_ext = var_w + if zonas.is_some() { 76.0 } else { 56.0 };
    let ext_w = 52.0_f64;
    let ext_rotulo = format!("{tipo}[][] · length {}", if sin_largos { "__".to_string() } else { nf.to_string() });
    c.t(x_ext, y0 - 8.0, &ext_rotulo, 10.0, 700, "textSub", "start", MONO);
    let alto_ext = if nf == 0 { celda_h } else { nf as f64 * celda_h };
    c.linea(
        var_w / 2.0,
        y0 + celda_h / 2.0,
        x_ext - 3.0,
        y0 + celda_h / 2.0,
        "accentPrimary",
        1.6,
        Some("main"),
        "",
    );
    if nf == 0 {
        c.r(x_ext, y0, vacio_w, celda_h, "fillCell", 2.0, 0.0);
    }
    let ancho_rotulo = ext_rotulo.chars().count() as f64 * 6.2;
    let x_filas = x_ext + (ext_w + 84.0).max(ancho_rotulo + 28.0);
    let mut ancho = x_ext + ancho_rotulo + 10.0;

    if indices {
        let max_l = filas.iter().flatten().map(Vec::len).max().unwrap_or(0);
        for j in 0..max_l {
            c.t(
                x_filas + j as f64 * celda_w + celda_w / 2.0,
                y0 - 8.0,
                &format!("[{j}]"),
                10.5,
                700,
                "textSub",
                "middle",
                MONO,
            );
        }
    }

    for (i, fila) in filas.iter().enumerate() {
        let y = y0 + i as f64 * celda_h;
        let cy = y + celda_h / 2.0;
        c.r(x_ext, y, ext_w, celda_h, "fillPointerBox", 1.5, 0.0);
        c.t(x_ext + 6.0, y + 12.0, &format!("[{i}]"), 9.0, 400, "textSub", "start", MONO);
        let Some(fila) = fila else {
            c.t(x_ext + ext_w / 2.0, cy + 7.0, "null", 10.5, 700, "textMuted", "middle", MONO);
            continue;
        };
        // Cada fila es un objeto aparte: la posición en el dibujo no es la de la memoria.
        let yf = y0 + i as f64 * paso_fila;
        let cyf = yf + celda_h / 2.0;
        c.punto(x_ext + ext_w / 2.0, cy + 5.0, "accentPrimary", 3.2);
        c.linea(
            x_ext + ext_w / 2.0,
            cy + 5.0,
            x_filas - 3.0,
            cyf,
            "accentPrimary",
            1.6,
            Some("main"),
            "",
        );
        let largo = fila.len();
        if largo == 0 {
            c.r(x_filas, yf, vacio_w, celda_h, "fillCell", 2.0, 0.0);
        }
        for (j, v) in fila.iter().enumerate() {
            let x = x_filas + j as f64 * celda_w;
            let marcada = resaltadas.contains(&(i as i64, j as i64));
            c.r(x, yf, celda_w, celda_h, if marcada { "fillHighlight" } else { "fillCell" }, 1.5, 0.0);
            if !v.is_empty() && !sin_valores {
                c.t(x + celda_w / 2.0, cyf + 5.0, v, 14.0, 700, "textMain", "middle", MONO);
            } else {
                c.vacio(x + 9.0, x + celda_w - 9.0, yf + celda_h - 9.0);
            }
        }
        let fin = x_filas + if largo == 0 { vacio_w } else { largo as f64 * celda_w };
        if largo > 0 {
            c.add(format!(
                r#"<rect x="{}" y="{yf:.1}" width="{}" height="{}" fill="none" stroke="{}" stroke-width="2"/>"#,
                fmt_num_clean(x_filas),
                fmt_num_clean(largo as f64 * celda_w),
                fmt_num_clean(celda_h),
                c.pal.stroke_main
            ));
        }
        let rotulo = format!("{tipo}[] · length {}", if sin_largos { "__".to_string() } else { largo.to_string() });
        c.t(fin + 8.0, cyf + 4.0, &rotulo, 9.5, 400, "textSub", "start", MONO);
        ancho = ancho.max(fin + 8.0 + rotulo.chars().count() as f64 * 6.0 + 10.0);
    }
    if nf > 0 {
        c.add(format!(
            r#"<rect x="{}" y="{y0:.1}" width="{}" height="{}" fill="none" stroke="{}" stroke-width="2"/>"#,
            fmt_num_clean(x_ext),
            fmt_num_clean(ext_w),
            fmt_num_clean(alto_ext),
            c.pal.stroke_main
        ));
    }

    let ultima_fila = filas.iter().rposition(Option::is_some).map_or(0.0, |i| i as f64 * paso_fila + celda_h);
    let mut alto = y0 + alto_ext.max(celda_h).max(ultima_fila) + 10.0;
    if let Some((z_stack, z_heap)) = zonas {
        zona(c, -6.0, var_w + 10.0, inicio + 2.0, alto + 4.0, &z_stack);
        zona(c, x_ext - 16.0, ancho, inicio + 2.0, alto + 4.0, &z_heap);
        alto += 8.0;
    }
    Ok((ancho, alto))
}

fn render_matrix_estatica(c: &mut Lienzo, b: &Bloque) -> Result<Punto, ErrorDiagrama> {
    let filas: Vec<Vec<String>> = filas_matriz(b)?
        .into_iter()
        .map(|f| f.unwrap_or_default())
        .collect();
    let def_rows = if filas.is_empty() { 3 } else { filas.len() };
    let nf = dimension(b, "rows", def_rows, MAX_FILAS)?;
    let max_cols = filas.iter().map(Vec::len).max().unwrap_or(0);
    let def_cols = if max_cols == 0 { 3 } else { max_cols };
    let nc = dimension(b, "cols", def_cols, MAX_COLUMNAS)?;

    let base = hex_b(b, "baseAddr", 0x1000);
    let paso = entero_b(b, "typeSize", 4, 1, 64);
    let nombre = str_o(b, "name", "m");
    let tipo = str_o(b, "elementType", "int");
    let direcciones = flag_f(b, "showAddresses");
    let indices = flag_t(b, "showIndices");
    let sin_valores = flag_f(b, "blankValues");
    let sin_dirs = flag_f(b, "blankAddrs");
    let resaltadas = pares_highlight(b)?;
    let filas_r = enteros_set(b, "highlightRows")?;
    let cols_r = enteros_set(b, "highlightCols")?;

    let mut extra = format!("{tipo} {nombre}[{nf}][{nc}]");
    if direcciones {
        extra.push_str(&format!(" | Base: {} | sizeof: {paso}B", fmt_hex(base)));
    }
    let top = encabezado(c, b, &extra);

    let x0 = 46.0_f64;
    let celda_w = 64.0_f64;
    let celda_h = if direcciones { 50.0 } else { 42.0 };
    let y0 = top + if indices { 26.0 } else { 8.0 };

    let valor_celda = |i: usize, j: usize| -> &str {
        filas
            .get(i)
            .and_then(|f| f.get(j))
            .map(String::as_str)
            .unwrap_or("")
    };

    if indices {
        for j in 0..nc {
            c.t(
                x0 + j as f64 * celda_w + celda_w / 2.0,
                y0 - 8.0,
                &format!("[{j}]"),
                11.0,
                700,
                "textSub",
                "middle",
                MONO,
            );
        }
    }

    for i in 0..nf {
        let y = y0 + i as f64 * celda_h;
        if indices {
            c.t(
                x0 - 8.0,
                y + celda_h / 2.0 + 4.0,
                &format!("[{i}]"),
                11.0,
                700,
                "textSub",
                "end",
                MONO,
            );
        }
        for j in 0..nc {
            let x = x0 + j as f64 * celda_w;
            let marcada = resaltadas.contains(&(i as i64, j as i64))
                || filas_r.contains(&(i as i64))
                || cols_r.contains(&(j as i64));
            c.r(
                x,
                y,
                celda_w,
                celda_h,
                if marcada { "fillHighlight" } else { "fillCell" },
                1.5,
                0.0,
            );
            let v = valor_celda(i, j);
            let centro_v = y + if direcciones {
                celda_h / 2.0 - 2.0
            } else {
                celda_h / 2.0 + 5.0
            };
            if !v.is_empty() && !sin_valores {
                c.t(
                    x + celda_w / 2.0,
                    centro_v,
                    v,
                    14.0,
                    700,
                    "textMain",
                    "middle",
                    MONO,
                );
            } else {
                c.vacio(x + 10.0, x + celda_w - 10.0, centro_v + 3.0);
            }
            if direcciones {
                let dir_txt = if sin_dirs {
                    "0x____".to_string()
                } else {
                    let offset_celdas = (i * nc + j) as i64;
                    fmt_hex(base.saturating_add(offset_celdas.saturating_mul(paso)))
                };
                c.t(
                    x + celda_w / 2.0,
                    y + celda_h - 7.0,
                    &dir_txt,
                    9.0,
                    400,
                    "textMain",
                    "middle",
                    MONO,
                );
            }
        }
    }

    c.add(format!(
        r#"<rect x="{}" y="{y0:.1}" width="{}" height="{}" fill="none" stroke="{}" stroke-width="2.25"/>"#,
        fmt_num_clean(x0),
        fmt_num_clean(nc as f64 * celda_w),
        fmt_num_clean(nf as f64 * celda_h),
        c.pal.stroke_main
    ));
    let mut ancho = x0 + nc as f64 * celda_w + 12.0;
    let mut alto = y0 + nf as f64 * celda_h + 8.0;

    if flag_f(b, "showLinear") {
        let lw = 52.0_f64;
        let lh = 36.0_f64;
        let y = alto + 34.0;
        c.t(
            0.0,
            y - 22.0,
            &format!("En memoria (fila por fila): {nombre}[i][j] = *(&{nombre}[0][0] + i*{nc} + j)"),
            10.0,
            400,
            "textMuted",
            "start",
            MONO,
        );
        for k in 0..(nf * nc) {
            let i = k / nc;
            let j = k % nc;
            let x = k as f64 * lw;
            let marcada = resaltadas.contains(&(i as i64, j as i64))
                || filas_r.contains(&(i as i64))
                || cols_r.contains(&(j as i64));
            c.t(
                x + lw / 2.0,
                y - 5.0,
                &format!("[{i}][{j}]"),
                9.0,
                400,
                "textMain",
                "middle",
                MONO,
            );
            c.r(
                x,
                y,
                lw,
                lh,
                if marcada { "fillHighlight" } else { "fillCell" },
                1.25,
                0.0,
            );
            let v = valor_celda(i, j);
            if !v.is_empty() && !sin_valores {
                c.t(
                    x + lw / 2.0,
                    y + lh / 2.0 + 5.0,
                    v,
                    13.0,
                    700,
                    "textMain",
                    "middle",
                    MONO,
                );
            } else {
                c.vacio(x + 8.0, x + lw - 8.0, y + lh - 9.0);
            }
            if direcciones {
                let dir_txt = if sin_dirs {
                    "0x____".to_string()
                } else {
                    fmt_hex(base.saturating_add((k as i64).saturating_mul(paso)))
                };
                c.t(
                    x + lw / 2.0,
                    y + lh + 13.0,
                    &dir_txt,
                    8.5,
                    400,
                    "textSub",
                    "middle",
                    MONO,
                );
            }
        }
        for i in 1..nf {
            let sep_x = i as f64 * nc as f64 * lw;
            c.linea(sep_x, y - 3.0, sep_x, y + lh + 3.0, "strokeMain", 2.5, None, "");
        }
        ancho = ancho.max((nf * nc) as f64 * lw + 4.0);
        alto = y + lh + if direcciones { 20.0 } else { 6.0 };
    }

    Ok((ancho, alto))
}

fn render_matrix_plana(c: &mut Lienzo, b: &Bloque) -> Result<Punto, ErrorDiagrama> {
    let filas: Vec<Vec<String>> = filas_matriz(b)?
        .into_iter()
        .map(|f| f.unwrap_or_default())
        .collect();
    let def_rows = if filas.is_empty() { 3 } else { filas.len() };
    let nf = dimension(b, "rows", def_rows, MAX_FILAS)?;
    let max_cols = filas.iter().map(Vec::len).max().unwrap_or(0);
    let def_cols = if max_cols == 0 { 3 } else { max_cols };
    let nc = dimension(b, "cols", def_cols, MAX_COLUMNAS)?;

    if nf * nc > MAX_CELDAS_PLANA {
        return Err(ErrorDiagrama::new(format!(
            "la matriz plana tiene {} celdas; el máximo es {MAX_CELDAS_PLANA}",
            nf * nc
        )));
    }

    let base = hex_b(b, "baseAddr", 0x1000);
    let paso = entero_b(b, "typeSize", 4, 1, 64);
    let nombre = str_o(b, "name", "m");
    let tipo = str_o(b, "elementType", "int");
    let direcciones = flag_f(b, "showAddresses");
    let sin_valores = flag_f(b, "blankValues");
    let sin_dirs = flag_f(b, "blankAddrs");
    let indices = flag_t(b, "showIndices");
    let coordenadas = flag_t(b, "showCoords");
    let resaltadas = pares_highlight(b)?;
    let filas_r = enteros_set(b, "highlightRows")?;
    let cols_r = enteros_set(b, "highlightCols")?;

    let top = encabezado(
        c,
        b,
        &format!("{tipo} *{nombre} = malloc({nf} * {nc} * sizeof({tipo}))"),
    );

    let m_w = 56.0_f64.max((nombre.chars().count() as f64 * 8.5 + 14.0).ceil());
    let x0 = m_w + 56.0;
    let lw = 52.0_f64;
    let lh = 40.0_f64;
    let y = top + 32.0;

    c.t(m_w / 2.0, y - 8.0, &nombre, 12.0, 700, "textMain", "middle", MONO);
    c.r(0.0, y, m_w, lh, "fillPointerBox", 1.75, 3.0);
    c.punto(m_w / 2.0, y + lh / 2.0, "accentPrimary", 3.2);
    c.linea(
        m_w / 2.0,
        y + lh / 2.0,
        x0 - 3.0,
        y + lh / 2.0,
        "accentPrimary",
        1.6,
        Some("main"),
        "",
    );

    for k in 0..(nf * nc) {
        let i = k / nc;
        let j = k % nc;
        let x = x0 + k as f64 * lw;
        let marcada = resaltadas.contains(&(i as i64, j as i64))
            || filas_r.contains(&(i as i64))
            || cols_r.contains(&(j as i64));
        if indices {
            c.t(
                x + lw / 2.0,
                y - 7.0,
                &format!("[{k}]"),
                10.0,
                700,
                "textSub",
                "middle",
                MONO,
            );
        }
        c.r(
            x,
            y,
            lw,
            lh,
            if marcada { "fillHighlight" } else { "fillCell" },
            1.25,
            0.0,
        );
        let v = filas
            .get(i)
            .and_then(|f| f.get(j))
            .map(String::as_str)
            .unwrap_or("");
        if !v.is_empty() && !sin_valores {
            c.t(
                x + lw / 2.0,
                y + lh / 2.0 + 5.0,
                v,
                13.0,
                700,
                "textMain",
                "middle",
                MONO,
            );
        } else {
            c.vacio(x + 8.0, x + lw - 8.0, y + lh - 9.0);
        }
        if coordenadas {
            c.t(
                x + lw / 2.0,
                y + lh + 13.0,
                &format!("({i},{j})"),
                9.0,
                400,
                "textMain",
                "middle",
                MONO,
            );
        }
        if direcciones {
            let dir_txt = if sin_dirs {
                "0x____".to_string()
            } else {
                fmt_hex(base.saturating_add((k as i64).saturating_mul(paso)))
            };
            c.t(
                x + lw / 2.0,
                y + lh + if coordenadas { 26.0 } else { 13.0 },
                &dir_txt,
                8.5,
                400,
                "textSub",
                "middle",
                MONO,
            );
        }
    }

    c.add(format!(
        r#"<rect x="{}" y="{y:.1}" width="{}" height="{}" fill="none" stroke="{}" stroke-width="2.25"/>"#,
        fmt_num_clean(x0),
        fmt_num_clean((nf * nc) as f64 * lw),
        fmt_num_clean(lh),
        c.pal.stroke_main
    ));
    for i in 1..nf {
        let sep_x = x0 + i as f64 * nc as f64 * lw;
        c.linea(sep_x, y - 3.0, sep_x, y + lh + 3.0, "strokeMain", 2.5, None, "");
    }

    let mut alto = y
        + lh
        + if coordenadas { 13.0 } else { 0.0 }
        + if direcciones { 13.0 } else { 0.0 }
        + 18.0;
    let mut ancho = x0 + (nf * nc) as f64 * lw + 6.0;

    if flag_t(b, "showFormula") {
        c.t(
            x0,
            alto,
            &format!("{nombre}[i * {nc} + j] es la celda (i, j)"),
            10.0,
            400,
            "textMuted",
            "start",
            MONO,
        );
        alto += 6.0;
    }

    if flag_f(b, "showLogical") {
        let mut sub_bloque = Bloque::new();
        for clave in [
            "values",
            "rows",
            "cols",
            "highlight",
            "highlightRows",
            "highlightCols",
            "blankValues",
        ] {
            if let Some(v) = b.get(clave) {
                sub_bloque.insert(clave.to_string(), v.clone());
            }
        }
        sub_bloque.insert(
            "subtitle".to_string(),
            Value::String("Vista lógica".to_string()),
        );
        let mut sub = sub_lienzo(c);
        let (ancho_m, alto_m) = render_matrix(&mut sub, &sub_bloque)?;
        agregar_sub_lienzo(c, 0.0, alto + 10.0, sub);
        ancho = ancho.max(ancho_m);
        alto = alto + 10.0 + alto_m;
    }

    Ok((ancho, alto))
}

fn fmt_direccion_simbolica(valor: Option<&Value>, hexadecimal: i64, sin_dirs: bool) -> String {
    if sin_dirs {
        return "0x____".to_string();
    }
    if let Some(Value::String(s)) = valor {
        if s.starts_with('@') {
            return s.clone();
        }
    }
    fmt_hex(hexadecimal)
}

fn render_matrix_dinamica(
    c: &mut Lienzo,
    b: &Bloque,
    dentada: bool,
) -> Result<Punto, ErrorDiagrama> {
    let mut filas = filas_matriz(b)?;
    let def_rows = if filas.is_empty() { 3 } else { filas.len() };
    let nf = dimension(b, "rows", def_rows, MAX_FILAS)?;
    while filas.len() < nf {
        filas.push(Some(Vec::new()));
    }
    let nc_fijo = if matches!(b.get("cols"), Some(v) if !v.is_null()) {
        Some(dimension(b, "cols", 1, MAX_COLUMNAS)?)
    } else {
        None
    };

    let nombre = str_o(b, "name", "m");
    let tipo = str_o(b, "elementType", "int");
    let direcciones = flag_f(b, "showAddresses");
    let indices = flag_t(b, "showIndices");
    let sin_valores = flag_f(b, "blankValues");
    let sin_dirs = flag_f(b, "blankAddrs");
    let resaltadas = pares_highlight(b)?;
    let base = hex_b(b, "baseAddr", 0x1000);
    let dirs_filas = lista_o_vacia(b, "rowAddresses")?;
    let base_filas = hex_b(b, "rowsBaseAddr", base.saturating_add(0x40));

    let centinela = if dentada {
        texto_opt(b, "sentinel")
    } else {
        None
    };
    let con_largos = dentada
        && b.get("showLengths")
            .map(es_truthy)
            .unwrap_or_else(|| centinela.is_none());
    let nombre_largos = str_o(b, "lengthsName", "largos");
    let tipo_largos = str_o(b, "lengthsType", "size_t");

    let mut extra = format!("{tipo} **{nombre}");
    if con_largos {
        extra.push_str(&format!(" | {tipo_largos} {nombre_largos}[{nf}]"));
    }
    if let Some(ref sent) = centinela {
        extra.push_str(&format!(" | centinela: {sent}"));
    }

    let zonas = nombres_de_zonas(b);
    let inicio = encabezado(c, b, &extra);
    let top = inicio + if zonas.is_some() { ALTO_ZONA } else { 0.0 };

    let dir_fila = |i: usize| -> String {
        if let Some(v) = dirs_filas.get(i) {
            fmt_direccion_simbolica(Some(v), hex(Some(v), 0x1000), sin_dirs)
        } else {
            fmt_direccion_simbolica(
                None,
                base_filas.saturating_add((i as i64).saturating_mul(0x40)),
                sin_dirs,
            )
        }
    };

    let dir_base = fmt_direccion_simbolica(b.get("baseAddr"), base, sin_dirs);
    let dir_largos_val = b.get("lengthsAddr");
    let dir_largos_str = dir_largos_val.filter(|v| es_truthy(v)).map(texto);

    let largos: Vec<usize> = filas[..nf]
        .iter()
        .map(|f| match f {
            None => 0,
            Some(fila) => nc_fijo.unwrap_or(fila.len()),
        })
        .collect();

    let celda_w = 56.0_f64;
    let celda_h = 40.0_f64;
    let paso_y = 54.0_f64;

    let ptr_lbl = format!("{nombre}[{}]", nf - 1);
    let ptr_w = (if direcciones { 72.0_f64 } else { 52.0_f64 })
        .max((ptr_lbl.chars().count() as f64 * 6.2 + 14.0).ceil());
    let mut m_w = 56.0_f64.max((nombre.chars().count() as f64 * 8.5 + 14.0).ceil());
    let largos_w = if con_largos {
        64.0_f64.max((nombre_largos.chars().count() as f64 * 8.5 + 14.0).ceil())
    } else {
        0.0
    };
    let mut x_m = if con_largos { largos_w + 36.0 } else { 0.0 };

    let dueno_val = b.get("owner").filter(|v| !v.is_null());
    let mut campos = Vec::new();
    let mut valores_dueno = Vec::new();
    if let Some(dueno) = dueno_val {
        campos = campos_cabecera(dueno, "owner")?;
        let Some(dueno_obj) = dueno.as_object() else {
            return Err(ErrorDiagrama::new(
                "'owner' necesita 'fields', la lista de sus campos",
            ));
        };
        for campo in &campos {
            match campo.destino.as_deref() {
                None => valores_dueno.push(campo.valor.clone()),
                Some("rows") => {
                    valores_dueno.push(if direcciones {
                        dir_base.clone()
                    } else {
                        String::new()
                    });
                }
                Some("lengths") if con_largos => {
                    if let Some(dl_val) = dir_largos_val.filter(|v| es_truthy(v)) {
                        valores_dueno.push(fmt_direccion_simbolica(
                            Some(dl_val),
                            hex(Some(dl_val), 0x1000),
                            sin_dirs,
                        ));
                    } else {
                        valores_dueno.push(String::new());
                    }
                }
                Some(otro) => {
                    return Err(ErrorDiagrama::new(format!(
                        "'owner': {} apunta a '{otro}' (se espera rows, o lengths con largos)",
                        campo.nombre
                    )));
                }
            }
        }
        m_w = ancho_cabecera(dueno_obj, &campos, &valores_dueno).ceil();
        x_m = 0.0;
    }

    let x_ptr = x_m + m_w + if dueno_val.is_some() { 72.0 } else { 56.0 };
    let x_filas = x_ptr + ptr_w + 70.0;
    let hay_largos = largos.iter().any(|&l| l > 0);
    let mut y0 = top + if indices && hay_largos { 44.0 } else { 30.0 };
    let largo_w = 44.0_f64;
    let largo_h = 30.0_f64;
    let y_largos = top + 22.0;
    if dueno_val.is_some() && con_largos {
        y0 = y_largos + largo_h + 56.0;
    }

    let declarados = lista_o_vacia(b, "lengths")?;
    let blank_lengths = flag_f(b, "blankLengths");

    let mut y_dueno_saved = top + 22.0;
    if let Some(dueno_obj) = dueno_val.and_then(Value::as_object) {
        if con_largos {
            let mut partes_titulo = Vec::new();
            if let Some(ref dl) = dir_largos_str {
                if !dl.is_empty() {
                    partes_titulo.push(dl.clone());
                }
            }
            partes_titulo.push(format!("({nombre_largos}: {tipo_largos}[{nf}])"));
            c.t(
                x_ptr,
                y_largos - 6.0,
                &partes_titulo.join(" "),
                10.5,
                700,
                "textMain",
                "start",
                MONO,
            );
            for (i, &largo_calc) in largos.iter().enumerate() {
                let x = x_ptr + i as f64 * largo_w;
                c.r(x, y_largos, largo_w, largo_h, "fillCell", 1.5, 0.0);
                c.t(
                    x + largo_w / 2.0,
                    y_largos + largo_h + 13.0,
                    &format!("[{i}]"),
                    10.0,
                    400,
                    "textMain",
                    "middle",
                    MONO,
                );
                if blank_lengths {
                    c.vacio(x + 8.0, x + largo_w - 8.0, y_largos + largo_h - 8.0);
                } else {
                    let largo_txt = declarados
                        .get(i)
                        .map(valor_py)
                        .unwrap_or_else(|| largo_calc.to_string());
                    c.t(
                        x + largo_w / 2.0,
                        y_largos + 20.0,
                        &largo_txt,
                        13.0,
                        700,
                        "textMain",
                        "middle",
                        MONO,
                    );
                }
            }
        }
        let fila_filas = campos
            .iter()
            .position(|campo| campo.destino.as_deref() == Some("rows"))
            .unwrap_or(0);
        let y_dueno = (top + 22.0)
            .max(y0 + celda_h / 2.0 - FILA_CABECERA / 2.0 - fila_filas as f64 * FILA_CABECERA);
        y_dueno_saved = y_dueno;
        let centros = cabecera(c, 0.0, y_dueno, m_w, dueno_obj, &campos, &valores_dueno)?;
        for (campo, centro) in campos.iter().zip(centros.iter().copied()) {
            match campo.destino.as_deref() {
                Some("rows") => c.ruta(&[
                    (m_w, centro),
                    (m_w + 30.0, centro),
                    (m_w + 30.0, y0 + celda_h / 2.0),
                    (x_ptr - 3.0, y0 + celda_h / 2.0),
                ]),
                Some("lengths") => c.ruta(&[
                    (m_w, centro),
                    (m_w + 16.0, centro),
                    (m_w + 16.0, y_largos + largo_h / 2.0),
                    (x_ptr - 3.0, y_largos + largo_h / 2.0),
                ]),
                _ => {}
            }
        }
    } else if con_largos {
        c.t(
            largos_w / 2.0,
            y0 - 8.0,
            &nombre_largos,
            12.0,
            700,
            "textMain",
            "middle",
            MONO,
        );
        for (i, &largo_calc) in largos.iter().enumerate() {
            let y = y0 + i as f64 * paso_y;
            c.r(0.0, y, largos_w, celda_h, "fillCell", 1.5, 0.0);
            c.t(
                largos_w / 2.0,
                y + 12.0,
                &format!("[{i}]"),
                9.5,
                400,
                "textMain",
                "middle",
                MONO,
            );
            if blank_lengths {
                c.vacio(10.0, largos_w - 10.0, y + celda_h - 8.0);
            } else {
                let largo_txt = declarados
                    .get(i)
                    .map(valor_py)
                    .unwrap_or_else(|| largo_calc.to_string());
                c.t(
                    largos_w / 2.0,
                    y + 32.0,
                    &largo_txt,
                    13.0,
                    700,
                    "textMain",
                    "middle",
                    MONO,
                );
            }
        }
    }

    if dueno_val.is_none() {
        c.t(
            x_m + m_w / 2.0,
            y0 - 8.0,
            &nombre,
            12.0,
            700,
            "textMain",
            "middle",
            MONO,
        );
        c.r(x_m, y0, m_w, celda_h, "fillPointerBox", 1.75, 3.0);
        if direcciones {
            c.t(
                x_m + m_w / 2.0,
                y0 + celda_h / 2.0 + 4.0,
                &dir_base,
                9.0,
                400,
                "textMain",
                "middle",
                MONO,
            );
            c.linea(
                x_m + m_w,
                y0 + celda_h / 2.0,
                x_ptr - 3.0,
                y0 + celda_h / 2.0,
                "accentPrimary",
                1.6,
                Some("main"),
                "",
            );
        } else {
            c.punto(x_m + m_w / 2.0, y0 + celda_h / 2.0, "accentPrimary", 3.2);
            c.linea(
                x_m + m_w / 2.0,
                y0 + celda_h / 2.0,
                x_ptr - 3.0,
                y0 + celda_h / 2.0,
                "accentPrimary",
                1.6,
                Some("main"),
                "",
            );
        }
    }
    if direcciones {
        c.t(
            x_ptr + ptr_w / 2.0,
            y0 - 8.0,
            &dir_base,
            9.0,
            400,
            "textSub",
            "middle",
            MONO,
        );
    }

    if indices && hay_largos {
        let max_l = largos.iter().copied().max().unwrap_or(0);
        for j in 0..max_l {
            c.t(
                x_filas + j as f64 * celda_w + celda_w / 2.0,
                y0 - 8.0,
                &format!("[{j}]"),
                11.0,
                700,
                "textSub",
                "middle",
                MONO,
            );
        }
    }

    let mut ancho = x_ptr + ptr_w + 60.0;
    for i in 0..nf {
        let y = y0 + i as f64 * paso_y;
        let cy = y + celda_h / 2.0;
        c.r(x_ptr, y, ptr_w, celda_h, "fillPointerBox", 1.5, 0.0);
        c.t(
            x_ptr + ptr_w / 2.0,
            y + 12.0,
            &format!("{nombre}[{i}]"),
            9.5,
            400,
            "textMain",
            "middle",
            MONO,
        );
        let Some(ref fila) = filas[i] else {
            c.linea(
                x_ptr + 5.0,
                y + celda_h - 5.0,
                x_ptr + ptr_w - 5.0,
                y + 16.0,
                "strokeMain",
                1.5,
                None,
                "",
            );
            c.t(
                x_ptr + ptr_w + 10.0,
                cy + 4.0,
                "NULL",
                10.0,
                700,
                "textMuted",
                "start",
                MONO,
            );
            continue;
        };

        let df = dir_fila(i);
        if direcciones {
            c.t(
                x_ptr + ptr_w / 2.0,
                y + 31.0,
                &df,
                9.0,
                400,
                "textMain",
                "middle",
                MONO,
            );
            c.linea(
                x_ptr + ptr_w,
                cy + 5.0,
                x_filas - 3.0,
                cy + 5.0,
                "accentPrimary",
                1.6,
                Some("main"),
                "",
            );
        } else {
            c.punto(x_ptr + ptr_w / 2.0, cy + 5.0, "accentPrimary", 3.2);
            c.linea(
                x_ptr + ptr_w / 2.0,
                cy + 5.0,
                x_filas - 3.0,
                cy + 5.0,
                "accentPrimary",
                1.6,
                Some("main"),
                "",
            );
        }

        let largo = largos[i];
        for j in 0..largo {
            let x = x_filas + j as f64 * celda_w;
            let marcada = resaltadas.contains(&(i as i64, j as i64));
            c.r(
                x,
                y,
                celda_w,
                celda_h,
                if marcada { "fillHighlight" } else { "fillCell" },
                1.5,
                0.0,
            );
            let v = fila.get(j).map(String::as_str).unwrap_or("");
            if !v.is_empty() && !sin_valores {
                c.t(
                    x + celda_w / 2.0,
                    cy + 5.0,
                    v,
                    14.0,
                    700,
                    "textMain",
                    "middle",
                    MONO,
                );
            } else {
                c.vacio(x + 9.0, x + celda_w - 9.0, y + celda_h - 9.0);
            }
        }
        if let Some(ref sent) = centinela {
            let x = x_filas + largo as f64 * celda_w;
            c.r(x, y, celda_w, celda_h, "fillHeader", 1.5, 0.0);
            c.add(format!(
                r#"<rect x="{:.1}" y="{:.1}" width="{}" height="{}" fill="none" stroke="{}" stroke-width="1" stroke-dasharray="3 2"/>"#,
                x + 3.0,
                y + 3.0,
                fmt_num_clean(celda_w - 6.0),
                fmt_num_clean(celda_h - 6.0),
                c.pal.stroke_muted
            ));
            c.t(
                x + celda_w / 2.0,
                cy + 5.0,
                sent,
                13.0,
                700,
                "textSub",
                "middle",
                MONO,
            );
        }
        let total = largo + usize::from(centinela.is_some());
        if total > 0 {
            c.add(format!(
                r#"<rect x="{}" y="{y:.1}" width="{}" height="{}" fill="none" stroke="{}" stroke-width="2"/>"#,
                fmt_num_clean(x_filas),
                fmt_num_clean(total as f64 * celda_w),
                fmt_num_clean(celda_h),
                c.pal.stroke_main
            ));
        }
        if direcciones && !sin_dirs {
            c.t(
                x_filas + total as f64 * celda_w + 8.0,
                cy + 4.0,
                &format!("← {df}"),
                9.0,
                400,
                "textSub",
                "start",
                MONO,
            );
        }
        ancho = ancho.max(
            x_filas + total as f64 * celda_w + if direcciones { 70.0 } else { 16.0 },
        );
    }

    let mut alto = y0 + (nf - 1) as f64 * paso_y + celda_h + 10.0;
    if dueno_val.is_some() {
        alto = alto.max(y_dueno_saved + campos.len() as f64 * FILA_CABECERA + 10.0);
        if con_largos {
            ancho = ancho.max(x_ptr + nf as f64 * largo_w + 12.0);
        }
    }
    if let Some((z_stack, z_heap)) = zonas {
        zona(c, -6.0, x_m + m_w + 8.0, inicio + 2.0, alto + 4.0, &z_stack);
        zona(c, x_ptr - 14.0, ancho, inicio + 2.0, alto + 4.0, &z_heap);
        alto += 8.0;
    }

    Ok((ancho, alto))
}

// ---------------------------------------------------------------------------
// Secuencia de Python (`array` con `variant: python_sequence`): índices y cortes
// ---------------------------------------------------------------------------

const MAX_SECUENCIA: usize = 16;

/// Un valor como lo escribe `repr()` de Python.
fn repr_python(v: &Value) -> String {
    match v {
        Value::Null => "None".to_string(),
        Value::Bool(true) => "True".to_string(),
        Value::Bool(false) => "False".to_string(),
        Value::String(s) => repr_cadena_python(s),
        Value::Array(a) => format!("[{}]", a.iter().map(repr_python).collect::<Vec<_>>().join(", ")),
        otro => texto(otro),
    }
}

fn repr_cadena_python(s: &str) -> String {
    if s.contains('\'') && !s.contains('"') {
        format!("\"{}\"", s.replace('\\', "\\\\"))
    } else {
        format!("'{}'", s.replace('\\', "\\\\").replace('\'', "\\'"))
    }
}

/// Un límite de un corte: entero o ausente (`null`).
fn limite_corte(corte: &serde_json::Map<String, Value>, clave: &str) -> Result<Option<i64>, ErrorDiagrama> {
    match corte.get(clave) {
        None | Some(Value::Null) => Ok(None),
        Some(Value::Number(n)) if n.as_i64().is_some() => Ok(n.as_i64()),
        Some(Value::String(t)) if t.trim().is_empty() => Ok(None),
        Some(Value::String(t)) => t
            .trim()
            .parse::<i64>()
            .map(Some)
            .map_err(|_| ErrorDiagrama::new(format!("'{clave}' del corte tiene que ser un entero: '{t}'"))),
        Some(otro) => Err(ErrorDiagrama::new(format!(
            "'{clave}' del corte tiene que ser un entero: {}",
            repr_val(otro)
        ))),
    }
}

/// `slice(inicio, fin, paso).indices(n)` de Python y los índices que selecciona.
fn indices_corte(
    n: usize,
    inicio: Option<i64>,
    fin: Option<i64>,
    paso: Option<i64>,
) -> Result<(i64, i64, i64, Vec<usize>), ErrorDiagrama> {
    let paso = paso.unwrap_or(1);
    if paso == 0 {
        return Err(ErrorDiagrama::new("el paso de un corte no puede ser 0"));
    }
    let n = n as i64;
    let (inferior, superior) = if paso > 0 { (0, n) } else { (-1, n - 1) };
    let ajustar = |v: Option<i64>, defecto: i64| match v {
        None => defecto,
        Some(v) if v < 0 => (v + n).max(inferior),
        Some(v) => v.min(superior),
    };
    let i0 = ajustar(inicio, if paso > 0 { inferior } else { superior });
    let i1 = ajustar(fin, if paso > 0 { superior } else { inferior });
    let mut elegidos = Vec::new();
    let mut i = i0;
    while (paso > 0 && i < i1) || (paso < 0 && i > i1) {
        elegidos.push(i as usize);
        i += paso;
    }
    Ok((i0, i1, paso, elegidos))
}

fn render_secuencia_python(c: &mut Lienzo, b: &Bloque) -> Result<Punto, ErrorDiagrama> {
    let tipo = str_o(b, "kind", "list");
    let (abre, cierra, nombre_defecto) = match tipo.as_str() {
        "list" => ("[", "]", "a"),
        "tuple" => ("(", ")", "t"),
        "str" => ("'", "'", "s"),
        otro => {
            return Err(ErrorDiagrama::new(format!(
                "'kind' de una secuencia de Python: '{otro}' (list, tuple o str)"
            )));
        }
    };
    // Los elementos, como Value; en una cadena, cada punto de código es un elemento.
    let elementos: Vec<Value> = match b.get("values") {
        None | Some(Value::Null) => Vec::new(),
        Some(Value::String(t)) if tipo == "str" => t.chars().map(|ch| Value::String(ch.to_string())).collect(),
        Some(Value::Array(a)) => a.clone(),
        Some(otro) => {
            return Err(ErrorDiagrama::new(format!(
                "'values' de una secuencia tiene que ser una lista{}: {}",
                if tipo == "str" { " o un texto" } else { "" },
                repr_val(otro)
            )));
        }
    };
    if elementos.len() > MAX_SECUENCIA {
        return Err(ErrorDiagrama::new(format!(
            "la secuencia tiene {} elementos; el máximo es {MAX_SECUENCIA}",
            elementos.len()
        )));
    }
    let n = elementos.len();
    let nombre = str_o(b, "name", nombre_defecto);
    let indices = flag_t(b, "showIndices");
    let negativos = flag_f(b, "showNegativeIndices");
    let fronteras = flag_f(b, "showBoundaries");
    let sin_valores = flag_f(b, "blankValues");

    let celdas: Vec<String> = elementos
        .iter()
        .map(|v| match (tipo.as_str(), v) {
            ("str", Value::String(t)) => t.clone(),
            ("str", otro) => texto(otro),
            (_, v) => repr_python(v),
        })
        .collect();
    let literal = |seleccion: &[usize]| -> String {
        if tipo == "str" {
            let t: String = seleccion.iter().map(|&i| celdas[i].as_str()).collect();
            repr_cadena_python(&t)
        } else {
            let partes: Vec<&str> = seleccion.iter().map(|&i| celdas[i].as_str()).collect();
            let coma = if tipo == "tuple" && partes.len() == 1 { "," } else { "" };
            format!("{abre}{}{coma}{cierra}", partes.join(", "))
        }
    };
    let todos: Vec<usize> = (0..n).collect();
    let declaracion = if sin_valores { String::new() } else { format!("{nombre} = {}", literal(&todos)) };

    // El corte, si hay.
    let corte = match b.get("slice") {
        None | Some(Value::Null) => None,
        Some(Value::Object(obj)) => {
            let (inicio, fin, paso) = (limite_corte(obj, "start")?, limite_corte(obj, "stop")?, limite_corte(obj, "step")?);
            let (i0, i1, p, elegidos) = indices_corte(n, inicio, fin, paso)?;
            let fmt = |v: Option<i64>| v.map(|v| v.to_string()).unwrap_or_default();
            let mut expr = format!("{nombre}[{}:{}", fmt(inicio), fmt(fin));
            if paso.is_some() {
                expr.push_str(&format!(":{}", fmt(paso)));
            }
            expr.push(']');
            Some((expr, i0, i1, p, elegidos, obj.get("blankSelection").is_some_and(es_truthy)))
        }
        Some(otro) => {
            return Err(ErrorDiagrama::new(format!(
                "'slice' tiene que ser un objeto con start, stop y step: {}",
                repr_val(otro)
            )));
        }
    };

    let top = encabezado(c, b, &declaracion);
    let cw = 46.0_f64;
    let ch = 40.0_f64;
    let x0 = 64.0_f64;
    let y_c = top + if indices { 26.0 } else { 10.0 };
    let elegidas: HashSet<usize> = match &corte {
        Some((_, _, _, _, el, false)) => el.iter().copied().collect(),
        _ => HashSet::new(),
    };
    let orden: HashMap<usize, usize> = match &corte {
        Some((_, _, _, p, el, false)) if *p != 1 => el.iter().enumerate().map(|(k, &i)| (i, k + 1)).collect(),
        _ => HashMap::new(),
    };

    if indices {
        c.t(x0 - 10.0, y_c - 9.0, "índice", 9.5, 400, "textSub", "end", SANS);
    }
    if n == 0 {
        c.r(x0, y_c, 16.0, ch, "fillCell", 2.0, 0.0);
        c.t(x0 + 26.0, y_c + ch / 2.0 + 4.0, &format!("{abre}{cierra} · len 0"), 10.0, 400, "textSub", "start", MONO);
    }
    for (i, celda) in celdas.iter().enumerate() {
        let x = x0 + i as f64 * cw;
        c.r(x, y_c, cw, ch, if elegidas.contains(&i) { "fillHighlight" } else { "fillCell" }, 1.5, 0.0);
        if indices {
            c.t(x + cw / 2.0, y_c - 9.0, &i.to_string(), 10.5, 700, "textSub", "middle", MONO);
        }
        if sin_valores {
            c.vacio(x + 8.0, x + cw - 8.0, y_c + ch - 9.0);
        } else if tipo == "str" && celda == " " {
            // Un espacio se ve: no se confunde con una celda para completar.
            c.t(x + cw / 2.0, y_c + ch / 2.0 + 5.0, "␣", 14.0, 700, "textMuted", "middle", MONO);
        } else {
            c.t(x + cw / 2.0, y_c + ch / 2.0 + 5.0, celda, 13.0, 700, "textMain", "middle", MONO);
        }
        if let Some(k) = orden.get(&i) {
            c.t(x + cw - 4.0, y_c + 11.0, &format!("{k}º"), 8.5, 700, "accentPrimary", "end", SANS);
        }
    }
    if n > 0 {
        c.add(format!(
            r#"<rect x="{x0:.1}" y="{y_c:.1}" width="{:.1}" height="{ch:.1}" fill="none" stroke="{}" stroke-width="2"/>"#,
            n as f64 * cw,
            c.pal.stroke_main
        ));
    }
    let mut y = y_c + ch;
    if negativos && n > 0 {
        c.t(x0 - 10.0, y + 14.0, "negativo", 9.5, 400, "textSub", "end", SANS);
        for i in 0..n {
            c.t(x0 + i as f64 * cw + cw / 2.0, y + 14.0, &format!("-{}", n - i), 10.5, 700, "textMuted", "middle", MONO);
        }
        y += 18.0;
    }
    let mut y_fronteras = None;
    if fronteras {
        y += 6.0;
        c.t(x0 - 10.0, y + 18.0, "frontera", 9.5, 400, "textSub", "end", SANS);
        c.linea(x0, y + 4.0, x0 + n as f64 * cw, y + 4.0, "strokeMuted", 1.0, None, "");
        for k in 0..=n {
            let x = x0 + k as f64 * cw;
            c.linea(x, y, x, y + 8.0, "strokeMain", 1.4, None, "");
        }
        y_fronteras = Some(y + 20.0);
        y += 24.0;
    }
    let mut ancho = x0 + n.max(1) as f64 * cw + 20.0;

    if let Some((expr, i0, i1, p, elegidos, en_blanco)) = &corte {
        // Las fronteras donde empieza y termina el recorrido: con paso negativo, a la derecha
        // de cada índice.
        let frontera = |i: i64| -> f64 { x0 + (if *p > 0 { i } else { i + 1 }) as f64 * cw };
        y += 10.0;
        if !en_blanco {
            let (xa, xb) = (frontera(*i0), frontera(*i1));
            // El fin es el límite excluido: una línea punteada atraviesa las celdas.
            c.linea(xb, y_c - 4.0, xb, y + 4.0, "accentPrimary", 1.4, None, "4 3");
            c.linea(xa, y_c + ch, xa, y + 4.0, "strokeSub", 1.0, None, "2 2");
            if (xa - xb).abs() > 1.0 {
                c.linea(xa, y + 4.0, xb + if *p > 0 { -3.0 } else { 3.0 }, y + 4.0, "accentPrimary", 1.6, Some("main"), "");
            }
            c.t(xa, y + 18.0, "inicio", 9.5, 700, "textSub", "middle", SANS);
            c.t(xb, y + if (xa - xb).abs() > 1.0 { 18.0 } else { 30.0 }, "fin (excluido)", 9.5, 700, "accentPrimary", "middle", SANS);
            ancho = ancho.max(xb.max(xa) + 50.0);
            y += 30.0;
        }
        y += 14.0;
        c.t(x0, y + 4.0, expr, 12.0, 700, "textMain", "start", MONO);
        let x_res = x0 + ancho_texto(expr, 12.0) + 12.0;
        c.t(x_res, y + 4.0, "→", 12.0, 700, "textMain", "start", MONO);
        if *en_blanco {
            c.vacio(x_res + 18.0, x_res + 118.0, y + 6.0);
            ancho = ancho.max(x_res + 130.0);
        } else {
            let resultado = literal(elegidos);
            c.t(x_res + 18.0, y + 4.0, &resultado, 12.0, 700, "accentPrimary", "start", MONO);
            ancho = ancho.max(x_res + 18.0 + ancho_texto(&resultado, 12.0) + 12.0);
        }
        y += 12.0;
    }
    // Los números de las fronteras van encima de las líneas del corte, con un halo del fondo.
    if let Some(yf) = y_fronteras {
        let fondo = c.pal.color("background").to_string();
        let color = c.pal.color("textSub").to_string();
        let fuente = c.font_mono().to_string();
        for k in 0..=n {
            c.add(format!(
                r#"<text x="{:.1}" y="{yf:.1}" text-anchor="middle" font-family="{fuente}" font-size="{}" font-weight="700" fill="{color}" stroke="{fondo}" stroke-width="3" paint-order="stroke">{k}</text>"#,
                x0 + k as f64 * cw,
                fmt_num_clean(c.font_size(10.0))
            ));
        }
    }
    Ok((ancho, y + 8.0))
}

// ---------------------------------------------------------------------------
// Cadena de C (`array`, `variant: c_string`)
// ---------------------------------------------------------------------------

const MAX_CADENA_C: usize = 24;

/// Un byte de una cadena de C: un carácter, el terminador o una posición sin dato declarado.
#[derive(Clone, Debug, PartialEq)]
enum ByteC {
    Caracter(String),
    Nul,
    SinDato,
}

/// `'a'`, `"\\0"` o `0` (el terminador), `"\\n"`, un número (el código) o `null` (sin dato).
fn byte_c(v: &Value, i: usize) -> Result<ByteC, ErrorDiagrama> {
    let caracter = |codigo: u32| -> ByteC {
        match codigo {
            0 => ByteC::Nul,
            0x0a => ByteC::Caracter("\\n".to_string()),
            0x09 => ByteC::Caracter("\\t".to_string()),
            0x20..=0x7e => ByteC::Caracter(char::from_u32(codigo).unwrap().to_string()),
            _ => ByteC::Caracter(format!("\\x{codigo:02X}")),
        }
    };
    match v {
        Value::Null => Ok(ByteC::SinDato),
        Value::Number(n) => match n.as_u64() {
            Some(k) if k <= 255 => Ok(caracter(k as u32)),
            _ => Err(ErrorDiagrama::new(format!("'bytes[{i}]': un byte va de 0 a 255, no {n}"))),
        },
        Value::String(t) => {
            let t = t.strip_prefix('\'').and_then(|r| r.strip_suffix('\'')).unwrap_or(t);
            match t {
                "\\0" | "\0" => Ok(ByteC::Nul),
                "\\n" | "\n" => Ok(caracter(0x0a)),
                "\\t" | "\t" => Ok(caracter(0x09)),
                "\\\\" => Ok(ByteC::Caracter("\\\\".to_string())),
                _ => {
                    let mut letras = t.chars();
                    match (letras.next(), letras.next()) {
                        (Some(ch), None) if (ch as u32) < 0x80 => Ok(caracter(ch as u32)),
                        _ => Err(ErrorDiagrama::new(format!(
                            "'bytes[{i}]': se espera un carácter ASCII, '\\0' o un número, no {t:?}"
                        ))),
                    }
                }
            }
        }
        otro => Err(ErrorDiagrama::new(format!("'bytes[{i}]': se espera un carácter, no {}", texto(otro)))),
    }
}

fn render_cadena_c(c: &mut Lienzo, b: &Bloque) -> Result<Punto, ErrorDiagrama> {
    let mut bytes: Vec<ByteC> = match (b.get("text").filter(|v| !v.is_null()), b.get("bytes").filter(|v| !v.is_null())) {
        (Some(_), Some(_)) => return Err(ErrorDiagrama::new("usar 'text' o 'bytes', no los dos")),
        (Some(t), None) => {
            let t = texto(t);
            if let Some(ch) = t.chars().find(|ch| !ch.is_ascii() || ch.is_ascii_control()) {
                return Err(ErrorDiagrama::new(format!(
                    "'text' admite caracteres ASCII imprimibles; para {ch:?}, usar 'bytes'"
                )));
            }
            let mut v: Vec<ByteC> = t.chars().map(|ch| ByteC::Caracter(ch.to_string())).collect();
            v.push(ByteC::Nul);
            v
        }
        (None, Some(Value::Array(lista))) => lista.iter().enumerate().map(|(i, v)| byte_c(v, i)).collect::<Result<_, _>>()?,
        (None, Some(_)) => return Err(ErrorDiagrama::new("'bytes' tiene que ser una lista")),
        (None, None) => return Err(ErrorDiagrama::new("falta 'text' (la cadena) o 'bytes' (los bytes del arreglo)")),
    };
    let texto_dado = b.contains_key("text");
    let cap = match b.get("capacity").filter(|v| !v.is_null()) {
        None => bytes.len(),
        Some(v) => match v.as_u64() {
            Some(k) if (1..=MAX_CADENA_C as u64).contains(&k) => k as usize,
            _ => {
                return Err(ErrorDiagrama::new(format!(
                    "'capacity' tiene que ser un entero entre 1 y {MAX_CADENA_C}: {}",
                    texto(v)
                )));
            }
        },
    };
    if texto_dado && cap + 1 == bytes.len() {
        // `char s[4] = "hola";` es válido en C: el terminador no entra y el arreglo no lo tiene.
        bytes.pop();
    }
    if bytes.len() > cap {
        return Err(ErrorDiagrama::new(format!(
            "hay {} bytes y la capacidad es {cap}{}",
            bytes.len(),
            if texto_dado { " (el texto más su terminador no entran)" } else { "" }
        )));
    }
    if cap > MAX_CADENA_C {
        return Err(ErrorDiagrama::new(format!("hasta {MAX_CADENA_C} bytes; hay {cap}")));
    }
    bytes.resize(cap, ByteC::SinDato);
    // Hasta el último byte con dato: lo que sigue es capacidad sin datos declarados.
    let declarados = bytes.iter().rposition(|x| *x != ByteC::SinDato).map_or(0, |k| k + 1);
    let nul = bytes.iter().position(|x| *x == ByteC::Nul);
    let nombre = str_o(b, "name", "s");
    let en_blanco = enteros_set(b, "blank")?;
    let sin_valores = flag_f(b, "blankValues");
    let sin_largo = flag_f(b, "blankLength");
    let indices = flag_t(b, "showIndices");

    let top = encabezado(c, b, &format!("char {nombre}[{cap}]"));
    let ptrs = punteros(b, cap + 1);
    let rango = leer_rango(b, cap)?;
    let x0 = 20.0_f64;
    let celda_w = 40.0_f64;
    let celda_h = 44.0_f64;
    let y = top + if ptrs.is_empty() { 26.0 } else { 60.0 };

    for (i, byte) in bytes.iter().enumerate() {
        let cx = x0 + i as f64 * celda_w;
        let centro = cx + celda_w / 2.0;
        if let Some(etiquetas) = ptrs.get(&i) {
            insignia_puntero(c, centro, y, &etiquetas.join(", "));
        }
        if indices {
            c.t(centro, y - 7.0, &format!("[{i}]"), 10.0, 700, "textSub", "middle", MONO);
        }
        let despues = nul.is_some_and(|k| i > k);
        let oculto = sin_valores || en_blanco.contains(&(i as i64));
        match byte {
            ByteC::SinDato => {
                c.add(format!(
                    r#"<rect x="{cx:.1}" y="{y:.1}" width="{celda_w:.1}" height="{celda_h:.1}" fill="none" stroke="{}" stroke-width="1.2" stroke-dasharray="3 3"/>"#,
                    c.pal.stroke_muted
                ));
                if oculto {
                    c.vacio(cx + 8.0, cx + celda_w - 8.0, y + celda_h - 10.0);
                }
            }
            _ => {
                let relleno = if *byte == ByteC::Nul && !oculto { "fillBadge" } else { "fillCell" };
                c.r(cx, y, celda_w, celda_h, relleno, if despues { 1.1 } else { 1.6 }, 0.0);
                if oculto {
                    c.vacio(cx + 8.0, cx + celda_w - 8.0, y + celda_h - 10.0);
                } else {
                    let (txt, color) = match byte {
                        ByteC::Nul => ("'\\0'".to_string(), "textMain"),
                        ByteC::Caracter(ch) => (format!("'{ch}'"), if despues { "textMuted" } else { "textMain" }),
                        ByteC::SinDato => unreachable!(),
                    };
                    let tam = if txt.chars().count() > 4 { 10.5 } else { 13.0 };
                    c.t(centro, y + celda_h / 2.0 + 4.5, &txt, tam, 700, color, "middle", MONO);
                }
            }
        }
    }
    let mut ancho = x0 + cap as f64 * celda_w + 20.0;
    if let Some(etiquetas) = ptrs.get(&cap) {
        ancho = ancho.max(limite_excluido(c, x0 + cap as f64 * celda_w, y, celda_h, &etiquetas.join(", ")));
    }

    // Las bandas: la cadena, el terminador, lo que sigue después y la capacidad sin datos.
    let mut bandas: Vec<(usize, usize, String)> = Vec::new();
    let oculta_largo = sin_largo || sin_valores;
    if let Some(k) = nul {
        if !oculta_largo {
            if k > 0 {
                bandas.push((0, k, format!("strlen = {k}")));
            }
            bandas.push((k, k + 1, "fin".to_string()));
            if declarados > k + 1 {
                bandas.push((k + 1, declarados, "después del terminador".to_string()));
            }
        }
    }
    if declarados < cap {
        bandas.push((declarados, cap, "sin datos".to_string()));
    }
    let yb = y + celda_h + 8.0;
    for (a, z, rotulo) in &bandas {
        let xa = x0 + *a as f64 * celda_w + 3.0;
        let xz = x0 + *z as f64 * celda_w - 3.0;
        c.linea(xa, yb, xa, yb + 5.0, "strokeSub", 1.1, None, "");
        c.linea(xz, yb, xz, yb + 5.0, "strokeSub", 1.1, None, "");
        c.linea(xa, yb + 5.0, xz, yb + 5.0, "strokeSub", 1.1, None, "");
        let ancho_rotulo = ancho_texto(rotulo, 9.0);
        // Un rótulo que no entra en su banda baja un renglón.
        let dy = if ancho_rotulo > xz - xa + 6.0 { 30.0 } else { 18.0 };
        c.t((xa + xz) / 2.0, yb + dy, rotulo, 9.0, 600, "textSub", "middle", SANS);
        ancho = ancho.max((xa + xz) / 2.0 + ancho_rotulo / 2.0 + 6.0);
    }
    let mut y_fin = yb + if bandas.is_empty() { 4.0 } else { 36.0 };
    if let Some(r) = &rango {
        let (a, w) = dibujar_rango(c, r, x0, celda_w, y_fin);
        y_fin = a;
        ancho = ancho.max(w);
    }

    let pie = match nul {
        _ if oculta_largo => format!("strlen({nombre}) = ______"),
        Some(k) => format!("strlen({nombre}) = {k} · capacidad = {cap}"),
        None => "sin terminador en la capacidad indicada: strlen no está definido".to_string(),
    };
    c.t(x0, y_fin + 14.0, &pie, 11.0, 700, "textMain", "start", MONO);
    ancho = ancho.max(x0 + ancho_texto(&pie, 11.0) + 10.0);
    Ok((ancho, y_fin + 22.0))
}

// ---------------------------------------------------------------------------
// Lista enlazada (`linked_list`: singly, doubly, circular, circular_doubly)
// ---------------------------------------------------------------------------

fn render_lista_vacia(c: &mut Lienzo, b: &Bloque) -> Punto {
    let top = encabezado(c, b, "");
    let y = top + 28.0;
    let cabeza_w = 72.0_f64;
    let head_label = str_o(b, "headLabel", "HEAD");
    c.t(
        cabeza_w / 2.0,
        y,
        &head_label,
        10.0,
        700,
        "textMain",
        "middle",
        MONO,
    );
    c.r(0.0, y + 7.0, cabeza_w, 40.0, "fillPointerBox", 1.75, 3.0);
    c.linea(
        5.0,
        y + 42.0,
        cabeza_w - 5.0,
        y + 12.0,
        "strokeMain",
        1.5,
        None,
        "",
    );
    c.linea(
        cabeza_w,
        y + 27.0,
        cabeza_w + 22.0,
        y + 27.0,
        "accentPrimary",
        1.5,
        None,
        "",
    );
    c.null(cabeza_w + 22.0, y + 27.0);
    (cabeza_w + 80.0, y + 56.0)
}

fn render_linked_list(c: &mut Lienzo, b: &Bloque) -> Result<Punto, ErrorDiagrama> {
    envolver_tipo("linked_list", render_linked_list_inner(c, b))
}

fn render_linked_list_inner(c: &mut Lienzo, b: &Bloque) -> Result<Punto, ErrorDiagrama> {
    if matches!(b.get("values"), Some(Value::Array(arr)) if arr.is_empty()) {
        return Ok(render_lista_vacia(c, b));
    }

    let mut vals = valores(b)?;
    if vals.is_empty() {
        vals.push(String::new());
    }
    let n = vals.len().min(7);

    let variante = b.get("variant").and_then(Value::as_str).unwrap_or("singly");
    let doble = matches!(variante, "doubly" | "circular_doubly");
    let circular = matches!(variante, "circular" | "circular_doubly");
    let sin_dirs = flag_f(b, "blankNextAddrs");
    let sin_flechas = b.get("drawArrows") == Some(&Value::Bool(false));
    let sin_valores = flag_f(b, "blankValues");

    let zonas = nombres_de_zonas(b);
    let inicio = encabezado(c, b, "");
    let top = inicio + if zonas.is_some() { ALTO_ZONA } else { 0.0 };

    let raw_dirs = lista_o_vacia(b, "addresses")?;
    let mut direcciones: Vec<String> = raw_dirs
        .iter()
        .map(|d| {
            if let Some(i) = d.as_i64() {
                fmt_hex(i)
            } else if let Some(u) = d.as_u64() {
                fmt_hex(i64::try_from(u).unwrap_or(i64::MAX))
            } else {
                texto(d)
            }
        })
        .collect();
    while direcciones.len() < n {
        let i = direcciones.len() as i64;
        direcciones.push(fmt_hex(0x4000 + i * 0x180));
    }

    let ptrs = punteros(b, n);
    let y = top + if ptrs.is_empty() { 28.0 } else { 62.0 };
    let alto_nodo = 54.0_f64;
    let mut cabeza_w = 72.0_f64;
    let mut x0 = 104.0_f64;
    let nodo_w = if doble { 126.0_f64 } else { 102.0_f64 };
    let gap = 44.0_f64;

    let cabecera_val = b.get("header").filter(|v| !v.is_null());
    let mut campos = Vec::new();
    let mut valores_cabecera = Vec::new();
    if let Some(cab_val) = cabecera_val {
        campos = campos_cabecera(cab_val, "header")?;
        let Some(cab_obj) = cab_val.as_object() else {
            return Err(ErrorDiagrama::new(
                "'header' necesita 'fields', la lista de sus campos",
            ));
        };
        for campo in &campos {
            let Some(ref dest) = campo.destino else {
                valores_cabecera.push(campo.valor.clone());
                continue;
            };
            let ok_digits = !dest.is_empty() && dest.chars().all(|ch| ch.is_ascii_digit());
            let idx = if ok_digits {
                dest.parse::<usize>().ok()
            } else {
                None
            };
            let Some(idx) = idx.filter(|&idx| idx < n) else {
                return Err(ErrorDiagrama::new(format!(
                    "'header': {} apunta al nodo '{}', y hay {} nodos (de 0 a {})",
                    campo.nombre,
                    dest,
                    n,
                    n - 1
                )));
            };
            valores_cabecera.push(if sin_dirs {
                String::new()
            } else {
                direcciones[idx].clone()
            });
        }
        cabeza_w = ancho_cabecera(cab_obj, &campos, &valores_cabecera);
        x0 = cabeza_w + 48.0;
    }

    let tipo_nodo = b.get("nodeType").filter(|v| es_truthy(v)).map(texto);

    for (i, etiquetas) in &ptrs {
        let centro = x0 + *i as f64 * (nodo_w + gap) + nodo_w / 2.0;
        let txt = etiquetas.join(", ");
        let ancho_cursor = 40.0_f64.max(txt.chars().count() as f64 * 7.5 + 12.0);
        c.r(
            centro - ancho_cursor / 2.0,
            y - 50.0,
            ancho_cursor,
            18.0,
            "fillBadge",
            1.2,
            3.0,
        );
        c.t(centro, y - 37.0, &txt, 10.0, 700, "textMain", "middle", MONO);
        c.linea(
            centro,
            y - 32.0,
            centro,
            y - 21.0,
            "accentPrimary",
            1.5,
            Some("main"),
            "",
        );
    }

    let mut abajo = y + alto_nodo + if circular { 50.0 } else { 28.0 };
    if let Some(cab_obj) = cabecera_val.and_then(Value::as_object) {
        let centros = cabecera(
            c,
            0.0,
            y + 27.0 - FILA_CABECERA / 2.0,
            cabeza_w,
            cab_obj,
            &campos,
            &valores_cabecera,
        )?;
        for (k, (campo, centro)) in campos.iter().zip(centros.iter().copied()).enumerate() {
            let Some(ref dest_str) = campo.destino else {
                continue;
            };
            if sin_flechas {
                continue;
            }
            let Ok(destino) = dest_str.parse::<usize>() else {
                continue;
            };
            if k == 0 && destino == 0 {
                c.linea(
                    cabeza_w,
                    centro,
                    x0 - 3.0,
                    centro,
                    "accentPrimary",
                    1.75,
                    Some("main"),
                    "",
                );
                continue;
            }
            let codo = cabeza_w + 12.0 + 8.0 * k as f64;
            let x_nodo = x0 + destino as f64 * (nodo_w + gap) + 16.0;
            c.ruta(&[
                (cabeza_w, centro),
                (codo, centro),
                (codo, abajo),
                (x_nodo, abajo),
                (x_nodo, y + alto_nodo + 3.0),
            ]);
            abajo += 10.0;
        }
    } else {
        let head_label = str_o(b, "headLabel", "HEAD");
        c.t(
            cabeza_w / 2.0,
            y,
            &head_label,
            10.0,
            700,
            "textMain",
            "middle",
            MONO,
        );
        c.r(0.0, y + 7.0, cabeza_w, 40.0, "fillPointerBox", 1.75, 3.0);
        if !sin_dirs {
            c.t(
                cabeza_w / 2.0,
                y + 31.0,
                &direcciones[0],
                10.0,
                600,
                "textSub",
                "middle",
                MONO,
            );
        }
        if !sin_flechas {
            c.linea(
                cabeza_w,
                y + 27.0,
                x0 - 3.0,
                y + 27.0,
                "accentPrimary",
                1.75,
                Some("main"),
                "",
            );
        }
    }

    for i in 0..n {
        let nx = x0 + i as f64 * (nodo_w + gap);
        let valor = &vals[i];
        let siguiente = if i < n - 1 {
            direcciones[i + 1].as_str()
        } else if circular {
            direcciones[0].as_str()
        } else {
            "NULL"
        };
        let anterior = if i > 0 {
            direcciones[i - 1].as_str()
        } else if circular {
            direcciones[n - 1].as_str()
        } else {
            "NULL"
        };
        let rotulo = match tipo_nodo {
            Some(ref tn) => format!("{} ({tn})", direcciones[i]),
            None => format!("Dir: {}", direcciones[i]),
        };
        c.t(
            nx + nodo_w / 2.0,
            y - 7.0,
            &rotulo,
            10.0,
            700,
            "textSub",
            "middle",
            MONO,
        );

        if !doble {
            let dato_w = 62.0_f64;
            let ptr_w = nodo_w - dato_w;
            c.r(nx, y, nodo_w, alto_nodo, "fillCell", 1.75, 3.0);
            c.r(nx + dato_w, y, ptr_w, alto_nodo, "fillPointerBox", 1.5, 0.0);
            c.t(
                nx + dato_w / 2.0,
                y + 12.0,
                "dato",
                9.5,
                400,
                "textMain",
                "middle",
                MONO,
            );
            c.t(
                nx + dato_w + ptr_w / 2.0,
                y + 12.0,
                "*sig",
                9.5,
                400,
                "textMain",
                "middle",
                MONO,
            );
            if !valor.is_empty() && !sin_valores {
                c.t(
                    nx + dato_w / 2.0,
                    y + 36.0,
                    valor,
                    14.0,
                    700,
                    "textMain",
                    "middle",
                    MONO,
                );
            } else {
                c.vacio(nx + 8.0, nx + dato_w - 8.0, y + 42.0);
            }
            let px = nx + dato_w + ptr_w / 2.0;
            let py = y + 33.0;
            if !sin_flechas {
                if i < n - 1 {
                    c.punto(px, py, "accentPrimary", 3.2);
                    c.linea(
                        px,
                        py,
                        nx + nodo_w + gap - 3.0,
                        py,
                        "accentPrimary",
                        1.75,
                        Some("main"),
                        "",
                    );
                } else if circular {
                    let vuelta = y + alto_nodo + 28.0;
                    c.punto(px, py, "accentPrimary", 3.2);
                    c.ruta(&[
                        (px, py),
                        (nx + nodo_w + 16.0, py),
                        (nx + nodo_w + 16.0, vuelta),
                        (x0 + 20.0, vuelta),
                        (x0 + 20.0, y + alto_nodo + 3.0),
                    ]);
                } else {
                    c.linea(
                        nx + dato_w + 5.0,
                        y + alto_nodo - 5.0,
                        nx + nodo_w - 5.0,
                        y + 16.0,
                        "strokeMain",
                        1.5,
                        None,
                        "",
                    );
                    c.punto(px, py, "accentPrimary", 3.0);
                    c.linea(
                        px,
                        py,
                        nx + nodo_w + 22.0,
                        py,
                        "accentPrimary",
                        1.5,
                        None,
                        "",
                    );
                    c.null(nx + nodo_w + 22.0, py);
                }
            }
            let sig_txt = if sin_dirs { "______" } else { siguiente };
            c.t(
                px,
                y + alto_nodo + 14.0,
                &format!("sig:{sig_txt}"),
                9.5,
                400,
                "textMain",
                "middle",
                MONO,
            );
        } else {
            let ptr_w = 32.0_f64;
            let dato_w = nodo_w - 2.0 * ptr_w;
            c.r(nx, y, nodo_w, alto_nodo, "fillCell", 1.75, 3.0);
            c.r(nx, y, ptr_w, alto_nodo, "fillPointerBox", 1.5, 0.0);
            c.r(
                nx + ptr_w + dato_w,
                y,
                ptr_w,
                alto_nodo,
                "fillPointerBox",
                1.5,
                0.0,
            );
            c.t(
                nx + ptr_w / 2.0,
                y + 12.0,
                "*ant",
                9.0,
                400,
                "textMain",
                "middle",
                MONO,
            );
            c.t(
                nx + ptr_w + dato_w / 2.0,
                y + 12.0,
                "dato",
                9.5,
                400,
                "textMain",
                "middle",
                MONO,
            );
            c.t(
                nx + ptr_w + dato_w + ptr_w / 2.0,
                y + 12.0,
                "*sig",
                9.0,
                400,
                "textMain",
                "middle",
                MONO,
            );
            if !valor.is_empty() && !sin_valores {
                c.t(
                    nx + ptr_w + dato_w / 2.0,
                    y + 36.0,
                    valor,
                    14.0,
                    700,
                    "textMain",
                    "middle",
                    MONO,
                );
            } else {
                c.vacio(nx + ptr_w + 8.0, nx + ptr_w + dato_w - 8.0, y + 42.0);
            }
            if !sin_flechas {
                if i == 0 && !circular {
                    c.linea(
                        nx + 4.0,
                        y + alto_nodo - 5.0,
                        nx + ptr_w - 4.0,
                        y + 16.0,
                        "strokeMain",
                        1.4,
                        None,
                        "",
                    );
                }
                if i == n - 1 && !circular {
                    c.linea(
                        nx + ptr_w + dato_w + 4.0,
                        y + alto_nodo - 5.0,
                        nx + nodo_w - 4.0,
                        y + 16.0,
                        "strokeMain",
                        1.4,
                        None,
                        "",
                    );
                }
                if i < n - 1 {
                    let siguiente_x = nx + nodo_w + gap;
                    let sig_x = nx + ptr_w + dato_w + ptr_w / 2.0;
                    c.punto(sig_x, y + 26.0, "accentPrimary", 2.8);
                    c.linea(
                        sig_x,
                        y + 26.0,
                        siguiente_x - 3.0,
                        y + 26.0,
                        "accentPrimary",
                        1.6,
                        Some("main"),
                        "",
                    );
                    c.punto(siguiente_x + ptr_w / 2.0, y + 40.0, "strokeSub", 2.8);
                    c.linea(
                        siguiente_x + ptr_w / 2.0,
                        y + 40.0,
                        nx + nodo_w + 3.0,
                        y + 40.0,
                        "strokeSub",
                        1.5,
                        Some("sub"),
                        "3 1.5",
                    );
                } else if circular {
                    let vuelta = y + alto_nodo + 28.0;
                    let sig_x = nx + ptr_w + dato_w + ptr_w / 2.0;
                    c.punto(sig_x, y + 26.0, "accentPrimary", 2.8);
                    c.ruta(&[
                        (sig_x, y + 26.0),
                        (nx + nodo_w + 16.0, y + 26.0),
                        (nx + nodo_w + 16.0, vuelta),
                        (x0 + 16.0, vuelta),
                        (x0 + 16.0, y + alto_nodo + 3.0),
                    ]);
                }
            }
            if !sin_dirs {
                c.t(
                    nx + nodo_w / 2.0,
                    y + alto_nodo + 14.0,
                    &format!("ant:{anterior} | sig:{siguiente}"),
                    9.5,
                    400,
                    "textMain",
                    "middle",
                    MONO,
                );
            }
        }
    }

    if circular && !sin_flechas {
        c.t(
            x0 + (n as f64 * (nodo_w + gap)) / 2.0,
            y + alto_nodo + 42.0,
            &format!("Enlace circular: el último apunta a HEAD ({})", direcciones[0]),
            9.0,
            600,
            "textSub",
            "middle",
            MONO,
        );
    }

    let ancho = x0 + n as f64 * (nodo_w + gap) - gap + if circular { 30.0 } else { 80.0 };
    let mut alto = y + alto_nodo + if circular { 50.0 } else { 24.0 };
    if cabecera_val.is_some() {
        alto = alto.max(
            y + 27.0 - FILA_CABECERA / 2.0 + campos.len() as f64 * FILA_CABECERA + 6.0,
        );
        if campos.iter().skip(1).any(|campo| campo.destino.is_some()) {
            alto = alto.max(abajo);
        }
    }
    if let Some((z_stack, z_heap)) = zonas {
        zona(c, -6.0, cabeza_w + 8.0, inicio + 2.0, alto + 6.0, &z_stack);
        zona(c, x0 - 14.0, ancho, inicio + 2.0, alto + 6.0, &z_heap);
        alto += 10.0;
    }

    Ok((ancho, alto))
}

// ---------------------------------------------------------------------------
// Pila (`stack`: simple y `vertical_multi`)
// ---------------------------------------------------------------------------

fn render_stack(c: &mut Lienzo, b: &Bloque) -> Result<Punto, ErrorDiagrama> {
    envolver_tipo("stack", render_stack_inner(c, b))
}

fn render_stack_inner(c: &mut Lienzo, b: &Bloque) -> Result<Punto, ErrorDiagrama> {
    let vals = valores(b)?;
    let cap = capacidad(b, vals.len(), 5, 2, 10);
    let base = hex_b(b, "baseAddr", 0x1000);
    let paso = entero_b(b, "typeSize", 4, 1, 64);
    let tope_str = match b.get("topIndex") {
        Some(v) => valor_py(v),
        None => {
            let no_vacios = vals.iter().filter(|v| !v.is_empty()).count() as i64;
            (no_vacios - 1).to_string()
        }
    };
    let sin_valores = flag_f(b, "blankValues");
    let sin_ptrs = flag_f(b, "blankPointers");
    let top_label = str_o(b, "topLabel", "TOPE");
    let dirs = flag_t(b, "showAddresses");

    let top = encabezado(c, b, "");
    let celda_w = 104.0_f64;
    let celda_h = 34.0_f64;
    let y0 = top + 26.0;
    let izq = if dirs { 84.0_f64 } else { 36.0_f64 };

    let dibujar_instancia = |c: &mut Lienzo, x: f64, etiqueta: &str, inicial: bool| {
        let caja = x + izq;
        let fondo = y0 + cap as f64 * celda_h;
        if !etiqueta.is_empty() {
            c.t(x, y0 - 8.0, etiqueta, 11.0, 700, "textSub", "start", SANS);
        }
        for idx in (0..cap).rev() {
            let cy = y0 + (cap - 1 - idx) as f64 * celda_h;
            let valor = if inicial {
                vals.get(idx).map(String::as_str).unwrap_or("")
            } else {
                ""
            };
            c.t(
                caja - 8.0,
                cy + celda_h / 2.0 + 4.0,
                &format!("[{idx}]"),
                10.0,
                600,
                "textSub",
                "end",
                MONO,
            );
            if inicial && dirs {
                let dir_txt = fmt_hex(base.saturating_add((idx as i64).saturating_mul(paso)));
                c.t(
                    caja - 34.0,
                    cy + celda_h / 2.0 + 3.0,
                    &dir_txt,
                    9.0,
                    400,
                    "textMuted",
                    "end",
                    MONO,
                );
            }
            c.r(caja, cy, celda_w, celda_h, "fillCell", 1.5, 0.0);
            if !valor.is_empty() && !sin_valores {
                c.t(
                    caja + celda_w / 2.0,
                    cy + celda_h / 2.0 + 5.0,
                    valor,
                    13.0,
                    700,
                    "textMain",
                    "middle",
                    MONO,
                );
            } else {
                c.vacio(caja + 14.0, caja + celda_w - 14.0, cy + celda_h - 8.0);
            }
            if inicial && !sin_ptrs && idx.to_string() == tope_str {
                let sx = caja + celda_w + 30.0;
                let sy = cy + celda_h / 2.0;
                c.r(sx, sy - 10.0, 64.0, 20.0, "fillBadge", 1.3, 3.0);
                c.t(
                    sx + 32.0,
                    sy + 4.0,
                    &top_label,
                    9.5,
                    700,
                    "textMain",
                    "middle",
                    MONO,
                );
                c.linea(
                    sx,
                    sy,
                    caja + celda_w + 3.0,
                    sy,
                    "accentPrimary",
                    1.6,
                    Some("main"),
                    "",
                );
            }
        }
        c.add(format!(
            r#"<path d="M {} {} L {} {} L {} {} L {} {}" fill="none" stroke="{}" stroke-width="2.5"/>"#,
            fmt_num_clean(caja - 3.0),
            fmt_num_clean(y0 - 4.0),
            fmt_num_clean(caja - 3.0),
            fmt_num_clean(fondo + 3.0),
            fmt_num_clean(caja + celda_w + 3.0),
            fmt_num_clean(fondo + 3.0),
            fmt_num_clean(caja + celda_w + 3.0),
            fmt_num_clean(y0 - 4.0),
            c.pal.stroke_main
        ));
        let tope_txt = if inicial && !sin_ptrs {
            tope_str.as_str()
        } else {
            "___"
        };
        c.t(
            caja + celda_w / 2.0,
            fondo + 18.0,
            &format!("BASE | tope = {tope_txt}"),
            9.5,
            600,
            "textMuted",
            "middle",
            MONO,
        );
    };

    let ancho = if b.get("layout").and_then(Value::as_str) == Some("vertical_multi") {
        let raw_labels = lista_o_vacia(b, "traceLabels")?;
        let etiquetas: Vec<String> = raw_labels.iter().map(valor_py).collect();
        let cuantas = if etiquetas.is_empty() {
            2
        } else {
            etiquetas.len().clamp(2, 4)
        };
        let max_lbl = etiquetas
            .iter()
            .map(|e| e.chars().count())
            .max()
            .unwrap_or(0) as f64;
        let paso_x = (izq + celda_w + 110.0).max(max_lbl * 6.2 + 40.0);
        for t in 0..cuantas {
            let etiq = etiquetas
                .get(t)
                .cloned()
                .unwrap_or_else(|| format!("Paso {}", t + 1));
            dibujar_instancia(c, t as f64 * paso_x, &etiq, t == 0);
        }
        cuantas as f64 * paso_x
    } else {
        let label = str_o(b, "label", "");
        dibujar_instancia(c, 0.0, &label, true);
        izq + celda_w + 30.0 + 64.0 + 10.0
    };

    Ok((ancho, y0 + cap as f64 * celda_h + 28.0))
}

// ---------------------------------------------------------------------------
// Cola (`queue`: lineal y circular)
// ---------------------------------------------------------------------------

fn render_queue(c: &mut Lienzo, b: &Bloque) -> Result<Punto, ErrorDiagrama> {
    envolver_tipo("queue", render_queue_inner(c, b))
}

fn render_queue_inner(c: &mut Lienzo, b: &Bloque) -> Result<Punto, ErrorDiagrama> {
    match b.get("variant").and_then(Value::as_str) {
        Some("ring") => return render_cola_anillo(c, b, false),
        Some("deque") => return render_cola_anillo(c, b, true),
        // Cualquier otra variante se dibuja como siempre: lineal (o circular).
        _ => {}
    }
    let vals = valores(b)?;
    let cap = capacidad(b, vals.len(), 8, 4, 12);
    let frente = b.get("frontIndex").map(valor_py).unwrap_or_default();
    let final_ = b.get("rearIndex").map(valor_py).unwrap_or_default();
    let sin_valores = flag_f(b, "blankValues");
    let sin_ptrs = flag_f(b, "blankPointers");
    let top = encabezado(c, b, "");

    if b.get("variant").and_then(Value::as_str) == Some("circular") {
        let cx = 150.0_f64;
        let cy = top + 130.0;
        let r_ext = 96.0_f64;
        let r_int = 52.0_f64;
        let r_texto = (r_ext + r_int) / 2.0;
        let r_indice = r_int - 13.0;
        let pt = |radio: f64, ang: f64| -> String {
            format!("{:.2} {:.2}", cx + radio * ang.cos(), cy + radio * ang.sin())
        };

        for i in 0..cap {
            let a0 = (i as f64 / cap as f64) * 2.0 * PI - FRAC_PI_2;
            let a1 = ((i + 1) as f64 / cap as f64) * 2.0 * PI - FRAC_PI_2;
            let am = (a0 + a1) / 2.0;
            c.add(format!(
                r#"<path d="M {} A 96 96 0 0 1 {} L {} A 52 52 0 0 0 {} Z" fill="{}" stroke="{}" stroke-width="1.6"/>"#,
                pt(r_ext, a0),
                pt(r_ext, a1),
                pt(r_int, a1),
                pt(r_int, a0),
                c.pal.fill_cell,
                c.pal.stroke_main
            ));
            c.t(
                cx + r_indice * am.cos(),
                cy + r_indice * am.sin() + 3.5,
                &format!("[{i}]"),
                10.0,
                700,
                "textSub",
                "middle",
                MONO,
            );
            let valor = vals.get(i).map(String::as_str).unwrap_or("");
            if !valor.is_empty() && !sin_valores {
                c.t(
                    cx + r_texto * am.cos(),
                    cy + r_texto * am.sin() + 4.0,
                    valor,
                    12.5,
                    700,
                    "textMain",
                    "middle",
                    MONO,
                );
            }
            let i_str = i.to_string();
            if !sin_ptrs && (i_str == frente || i_str == final_) {
                c.linea(
                    cx + (r_ext + 28.0) * am.cos(),
                    cy + (r_ext + 28.0) * am.sin(),
                    cx + (r_ext + 4.0) * am.cos(),
                    cy + (r_ext + 4.0) * am.sin(),
                    "accentPrimary",
                    1.75,
                    Some("main"),
                    "",
                );
                let etiqueta = if frente == final_ {
                    "FRENTE=FINAL"
                } else if i_str == frente {
                    "FRENTE"
                } else {
                    "FINAL"
                };
                c.t(
                    cx + (r_ext + 42.0) * am.cos(),
                    cy + (r_ext + 42.0) * am.sin() + 4.0,
                    etiqueta,
                    10.0,
                    700,
                    "textMain",
                    "middle",
                    MONO,
                );
            }
        }
        c.t(
            cx,
            cy + 4.0,
            &format!("N = {cap}"),
            10.0,
            700,
            "textMuted",
            "middle",
            MONO,
        );
        if !flag_t(b, "showPanel") {
            return Ok((320.0, top + 270.0));
        }
        let px = 330.0_f64;
        let pw = 400.0_f64;
        c.add(format!(
            r#"<g transform="translate(330, {})">"#,
            fmt_num_clean(top + 30.0)
        ));
        c.r(0.0, 0.0, pw, 190.0, "fillHeader", 1.4, 4.0);
        c.t(
            14.0,
            24.0,
            &format!("Índices de la cola circular (módulo N = {cap})"),
            12.0,
            700,
            "textMain",
            "start",
            SANS,
        );
        c.linea(0.0, 34.0, pw, 34.0, "strokeMain", 1.0, None, "");
        let f_txt = if sin_ptrs { "______" } else { &frente };
        let r_txt = if sin_ptrs { "______" } else { &final_ };
        c.t(
            14.0,
            56.0,
            &format!("• FRENTE = {f_txt}   (próximo a salir)"),
            11.0,
            600,
            "textMain",
            "start",
            MONO,
        );
        c.t(
            14.0,
            78.0,
            &format!("• FINAL  = {r_txt}   (último insertado)"),
            11.0,
            600,
            "textMain",
            "start",
            MONO,
        );
        c.t(
            14.0,
            108.0,
            "Aritmética modular:",
            10.5,
            400,
            "textSub",
            "start",
            MONO,
        );
        c.t(
            22.0,
            126.0,
            &format!("Encolar:     final  = (final + 1) % {cap}"),
            10.0,
            400,
            "textMuted",
            "start",
            MONO,
        );
        c.t(
            22.0,
            144.0,
            &format!("Desencolar:  frente = (frente + 1) % {cap}"),
            10.0,
            400,
            "textMuted",
            "start",
            MONO,
        );
        c.t(
            22.0,
            162.0,
            &format!("Llena:       (final + 1) % {cap} == frente"),
            10.0,
            400,
            "textMuted",
            "start",
            MONO,
        );
        c.t(
            22.0,
            180.0,
            "Cantidad:    cant = _________",
            10.0,
            400,
            "textMuted",
            "start",
            MONO,
        );
        c.add("</g>");
        return Ok((px + pw + 10.0, top + 270.0));
    }

    let x0 = 110.0_f64;
    let celda_w = 60.0_f64;
    let celda_h = 50.0_f64;
    let y = top + 58.0;
    let fin_x = x0 + cap as f64 * celda_w;

    c.t(
        x0 - 14.0,
        y + 20.0,
        "DESENCOLAR",
        10.0,
        700,
        "textSub",
        "end",
        MONO,
    );
    c.t(
        x0 - 14.0,
        y + 34.0,
        "← sale frente",
        9.0,
        400,
        "textMuted",
        "end",
        MONO,
    );
    c.t(
        fin_x + 14.0,
        y + 20.0,
        "ENCOLAR(x)",
        10.0,
        700,
        "textSub",
        "start",
        MONO,
    );
    c.t(
        fin_x + 14.0,
        y + 34.0,
        "← entra final",
        9.0,
        400,
        "textMuted",
        "start",
        MONO,
    );
    c.linea(
        x0 - 6.0,
        y - 3.0,
        fin_x + 6.0,
        y - 3.0,
        "strokeMain",
        2.2,
        None,
        "",
    );
    c.linea(
        x0 - 6.0,
        y + celda_h + 3.0,
        fin_x + 6.0,
        y + celda_h + 3.0,
        "strokeMain",
        2.2,
        None,
        "",
    );

    for i in 0..cap {
        let cx = x0 + i as f64 * celda_w;
        let centro = cx + celda_w / 2.0;
        c.t(
            centro,
            y - 9.0,
            &format!("[{i}]"),
            10.5,
            700,
            "textSub",
            "middle",
            MONO,
        );
        c.r(cx, y, celda_w, celda_h, "fillCell", 1.5, 0.0);
        let valor = vals.get(i).map(String::as_str).unwrap_or("");
        if !valor.is_empty() && !sin_valores {
            c.t(
                centro,
                y + celda_h / 2.0 + 5.0,
                valor,
                14.0,
                700,
                "textMain",
                "middle",
                MONO,
            );
        } else {
            c.vacio(cx + 10.0, cx + celda_w - 10.0, y + celda_h - 12.0);
        }
        if sin_ptrs {
            continue;
        }
        let i_str = i.to_string();
        if i_str == frente {
            c.linea(
                centro,
                y + celda_h + 24.0,
                centro,
                y + celda_h + 6.0,
                "accentPrimary",
                1.6,
                Some("main"),
                "",
            );
            c.r(
                centro - 30.0,
                y + celda_h + 24.0,
                60.0,
                18.0,
                "fillBadge",
                1.2,
                3.0,
            );
            c.t(
                centro,
                y + celda_h + 37.0,
                "FRENTE",
                9.5,
                700,
                "textMain",
                "middle",
                MONO,
            );
        }
        if i_str == final_ {
            let cx_fin = if i_str == frente {
                centro + 18.0
            } else {
                centro
            };
            c.linea(
                cx_fin,
                y - 34.0,
                cx_fin,
                y - 21.0,
                "accentPrimary",
                1.6,
                Some("main"),
                "",
            );
            c.r(cx_fin - 28.0, y - 52.0, 56.0, 18.0, "fillBadge", 1.2, 3.0);
            c.t(
                cx_fin,
                y - 39.0,
                "FINAL",
                9.5,
                700,
                "textMain",
                "middle",
                MONO,
            );
        }
    }

    Ok((fin_x + 110.0, y + celda_h + 50.0))
}

/// Las partes del panel de una cola circular que se pueden dejar en blanco.
const BLANCOS_ANILLO: [&str; 6] = ["front", "last", "count", "next", "values", "order"];

/// Una cola circular (o una deque) por su capacidad, el índice del frente y los valores en orden
/// lógico: la cantidad es el largo de los valores; el último ocupado, `(frente + cantidad - 1) % N`
/// y el siguiente para agregar al final, `(frente + cantidad) % N`.
fn render_cola_anillo(c: &mut Lienzo, b: &Bloque, deque: bool) -> Result<Punto, ErrorDiagrama> {
    let vals = valores(b)?;
    let Some(cap) = b.get("capacity").filter(|v| !v.is_null()) else {
        return Err(ErrorDiagrama::new("falta 'capacity' (la cantidad de celdas del arreglo)"));
    };
    let cap = match cap.as_u64() {
        Some(k) if (1..=12).contains(&k) => k as usize,
        _ => return Err(ErrorDiagrama::new(format!("'capacity' tiene que estar entre 1 y 12: {}", texto(cap)))),
    };
    let frente = match b.get("frontIndex").filter(|v| !v.is_null()) {
        None => return Err(ErrorDiagrama::new("falta 'frontIndex' (el índice del primer elemento)")),
        Some(v) => match v.as_u64() {
            Some(k) if (k as usize) < cap => k as usize,
            _ => {
                return Err(ErrorDiagrama::new(format!(
                    "'frontIndex' tiene que estar entre 0 y {}: {}",
                    cap - 1,
                    texto(v)
                )));
            }
        },
    };
    let cant = vals.len();
    if cant > cap {
        return Err(ErrorDiagrama::new(format!(
            "hay {cant} valores y la capacidad es {cap}: no entran"
        )));
    }
    let mut blancos: HashSet<String> = HashSet::new();
    for v in lista_o_vacia(b, "blank")? {
        let t = texto(v);
        if !BLANCOS_ANILLO.contains(&t.as_str()) {
            return Err(ErrorDiagrama::new(format!(
                "'blank' admite {}: no '{t}'",
                BLANCOS_ANILLO.join(", ")
            )));
        }
        blancos.insert(t);
    }
    let oculto = |k: &str| blancos.contains(k);
    let ultimo = (cant > 0).then(|| (frente + cant - 1) % cap);
    // Llena, el siguiente coincide con el frente: no hay dónde agregar.
    let siguiente = (frente + cant) % cap;
    let llena = cant == cap;
    let mut fisico: Vec<Option<(usize, &str)>> = vec![None; cap];
    for (k, v) in vals.iter().enumerate() {
        fisico[(frente + k) % cap] = Some((k, v.as_str()));
    }

    let nombre = if deque { "Deque circular" } else { "Cola circular" };
    let top = encabezado(c, b, &format!("{nombre}: N = {cap}"));
    let x0 = 30.0_f64;
    let celda_w = 58.0_f64;
    let celda_h = 48.0_f64;
    let y = top + 72.0;
    let fin_x = x0 + cap as f64 * celda_w;

    // La vuelta: del final del arreglo se sigue en la celda 0.
    c.add(format!(
        r#"<path d="M {:.1} {:.1} C {:.1} {:.1}, {:.1} {:.1}, {:.1} {:.1}" fill="none" stroke="{}" stroke-width="1.3" stroke-dasharray="5 3" marker-end="url(#flecha-sub-{})"/>"#,
        fin_x - 10.0,
        y - 22.0,
        fin_x - 10.0,
        y - 52.0,
        x0 + 10.0,
        y - 52.0,
        x0 + 10.0,
        y - 24.0,
        c.pal.stroke_sub,
        c.sufijo
    ));
    c.t((x0 + fin_x) / 2.0, y - 58.0, &format!("índice siguiente: (i + 1) % {cap}"), 9.5, 400, "textMuted", "middle", MONO);

    for (i, celda) in fisico.iter().enumerate() {
        let cx = x0 + i as f64 * celda_w;
        let centro = cx + celda_w / 2.0;
        c.t(centro, y - 8.0, &format!("[{i}]"), 10.5, 700, "textSub", "middle", MONO);
        match celda {
            Some((k, v)) => {
                c.r(cx, y, celda_w, celda_h, "fillCell", 1.6, 0.0);
                if oculto("values") || v.is_empty() {
                    c.vacio(cx + 10.0, cx + celda_w - 10.0, y + celda_h - 12.0);
                } else {
                    c.t(centro, y + celda_h / 2.0 + 3.0, v, 14.0, 700, "textMain", "middle", MONO);
                }
                if !oculto("order") {
                    c.t(cx + celda_w - 5.0, y + celda_h - 5.0, &format!("{}º", k + 1), 8.5, 600, "textMuted", "end", SANS);
                }
            }
            None => {
                c.add(format!(
                    r#"<rect x="{cx:.1}" y="{y:.1}" width="{celda_w:.1}" height="{celda_h:.1}" fill="none" stroke="{}" stroke-width="1.2" stroke-dasharray="4 3"/>"#,
                    c.pal.stroke_muted
                ));
                if oculto("values") || oculto("order") {
                    c.vacio(cx + 10.0, cx + celda_w - 10.0, y + celda_h - 12.0);
                } else {
                    c.t(centro, y + celda_h / 2.0 + 3.0, "libre", 9.5, 400, "textMuted", "middle", SANS);
                }
            }
        }
    }

    // Los índices, debajo de su celda; varios en la misma celda se apilan.
    let mut marcas: Vec<Vec<&str>> = vec![Vec::new(); cap];
    if !oculto("front") {
        marcas[frente].push("frente");
    }
    if let (Some(u), false) = (ultimo, oculto("last")) {
        marcas[u].push("último");
    }
    if !llena && !oculto("next") {
        marcas[siguiente].push("siguiente");
    }
    let mut y_marcas = y + celda_h;
    for (i, m) in marcas.iter().enumerate() {
        if m.is_empty() {
            continue;
        }
        let centro = x0 + i as f64 * celda_w + celda_w / 2.0;
        c.linea(centro, y + celda_h + 22.0, centro, y + celda_h + 5.0, "accentPrimary", 1.6, Some("main"), "");
        for (k, rotulo) in m.iter().enumerate() {
            let yy = y + celda_h + 36.0 + k as f64 * 14.0;
            c.t(centro, yy, rotulo, 10.0, 700, "textMain", "middle", MONO);
            y_marcas = y_marcas.max(yy);
        }
    }

    // El panel: los índices y la cantidad, con lo que hace cada operación.
    let dato = |clave: &str, valor: Option<usize>, sin: &str| -> String {
        if oculto(clave) {
            "____".to_string()
        } else {
            valor.map(|v| v.to_string()).unwrap_or_else(|| sin.to_string())
        }
    };
    let mut lineas = vec![
        format!("capacidad N = {cap}"),
        format!("frente = {}", dato("front", Some(frente), "")),
        format!("cantidad = {}", dato("count", Some(cant), "")),
        format!("último = {}", dato("last", ultimo, "— (vacía)")),
        format!(
            "siguiente = {}{}",
            dato("next", Some(siguiente), ""),
            if llena && !oculto("next") && !oculto("count") { " (= frente, pero llena: no hay lugar)" } else { "" }
        ),
    ];
    lineas.push(String::new());
    if deque {
        lineas.push("agregar al frente: frente = (frente - 1 + N) % N; cantidad++".to_string());
        lineas.push("agregar al final: datos[(frente + cantidad) % N] = x; cantidad++".to_string());
        lineas.push("quitar del frente: frente = (frente + 1) % N; cantidad--".to_string());
        lineas.push("quitar del final: cantidad--".to_string());
    } else {
        lineas.push("encolar: datos[(frente + cantidad) % N] = x; cantidad++".to_string());
        lineas.push("desencolar: frente = (frente + 1) % N; cantidad--".to_string());
    }
    let py = y_marcas + 22.0;
    let ancho_panel = lineas.iter().map(|l| ancho_texto(l, 10.0)).fold(0.0, f64::max) + 28.0;
    let alto_panel = lineas.len() as f64 * 16.0 + 14.0;
    c.r(x0, py, ancho_panel, alto_panel, "fillHeader", 1.2, 4.0);
    for (k, l) in lineas.iter().enumerate() {
        let color = if k >= 5 { "textMuted" } else { "textMain" };
        c.t(x0 + 14.0, py + 20.0 + k as f64 * 16.0, l, 10.0, if k < 5 { 700 } else { 400 }, color, "start", MONO);
    }
    Ok(((fin_x + 20.0).max(x0 + ancho_panel + 10.0), py + alto_panel + 8.0))
}

#[cfg(test)]
mod tests {
    use super::*;
    use dibu_core::PALETA_TINTA;
    use serde_json::json;

    fn reg_lineal() -> Registro {
        let mut r = Registro::new();
        registrar(&mut r);
        r
    }

    fn render(val: Value, tinta: bool) -> String {
        let reg = reg_lineal();
        let bloque = val.as_object().expect("objeto JSON de prueba");
        reg.renderizar(bloque, tinta).expect("render válido")
    }

    fn render_err(val: Value) -> String {
        let reg = reg_lineal();
        let bloque = val.as_object().expect("objeto JSON de prueba");
        reg.renderizar(bloque, true)
            .expect_err("se esperaba ErrorDiagrama")
            .to_string()
    }

    fn textos_svg(svg: &str) -> Vec<String> {
        let mut out = Vec::new();
        let mut rest = svg;
        while let Some(pos) = rest.find("</text>") {
            let antes = &rest[..pos];
            if let Some(gt) = antes.rfind('>') {
                out.push(antes[gt + 1..].to_string());
            }
            rest = &rest[pos + "</text>".len()..];
        }
        out
    }

    fn attr_int(svg: &str, attr: &str) -> i64 {
        let pat = format!(r#"{attr}=""#);
        let idx = svg.find(&pat).expect("atributo presente");
        let resto = &svg[idx + pat.len()..];
        let fin = resto.find('"').expect("cierre comilla");
        resto[..fin].parse::<i64>().expect("entero")
    }

    #[test]
    fn registro_expone_exactamente_los_cinco_tipos() {
        let reg = reg_lineal();
        assert_eq!(
            reg.tipos(),
            vec!["array", "linked_list", "matrix", "queue", "stack"]
        );
    }

    #[test]
    fn array_basico_direcciones_offsets_y_blank() {
        let svg = render(
            json!({
                "type": "array",
                "values": [3, 1, 4],
                "pointers": "p:0, q:2, r:0",
                "showAddresses": true,
                "showOffsets": true
            }),
            true,
        );
        assert!(svg.starts_with("<svg "));
        assert!(attr_int(&svg, "width") > 50);
        assert!(attr_int(&svg, "height") > 50);
        let ts = textos_svg(&svg);
        assert!(ts.contains(&"3".to_string()));
        assert!(ts.contains(&"1".to_string()));
        assert!(ts.contains(&"4".to_string()));
        assert!(ts.contains(&"p, r".to_string()));
        assert!(ts.contains(&"q".to_string()));
        assert!(ts.contains(&"0x1000".to_string()));
        assert!(ts.contains(&"0x1008".to_string()));
        assert!(ts.contains(&"+8B".to_string()));

        let svg_blank = render(
            json!({
                "type": "array",
                "values": [41, 42],
                "blankValues": true,
                "blankAddrs": true,
                "showAddresses": true,
                "showIndices": false
            }),
            true,
        );
        assert!(!svg_blank.contains(">41<"));
        assert!(svg_blank.contains("stroke-dasharray"));
        assert!(svg_blank.contains(">0x____<"));
        assert!(!svg_blank.contains(">[0]<"));

        let titulo_largo = "A".repeat(50);
        let svg_titulo = render(
            json!({
                "type": "array",
                "values": [1],
                "title": titulo_largo
            }),
            true,
        );
        assert!(attr_int(&svg_titulo, "width") as f64 >= 50.0 * 8.6);
    }

    #[test]
    fn array_utf8_y_errores_malformados() {
        let svg = render(
            json!({
                "type": "array",
                "values": ["ñ", "áé"],
                "pointers": "índ_α:0, índice_β:1"
            }),
            true,
        );
        let ts = textos_svg(&svg);
        assert!(ts.contains(&"índ_α".to_string()));
        assert!(ts.contains(&"índice_β".to_string()));

        assert!(render_err(json!({"type": "array", "values": 5})).contains("values"));

        // Extremos numéricos: cero panics
        let svg_ext = render(
            json!({
                "type": "array",
                "capacity": 1e308,
                "typeSize": -50,
                "baseAddr": "no_hex",
                "showIndices": null
            }),
            true,
        );
        assert!(svg_ext.starts_with("<svg "));
        assert!(!svg_ext.contains(">[0]<"));
    }

    #[test]
    fn matrix_static_show_linear_y_resaltado() {
        let svg = render(
            json!({
                "type": "matrix",
                "values": [1, 2, 3, 4, 5, 6],
                "cols": 3,
                "baseAddr": "0x1000",
                "typeSize": 4,
                "showAddresses": true,
                "showLinear": true
            }),
            true,
        );
        let ts = textos_svg(&svg);
        assert!(ts.contains(&"int m[2][3] | Base: 0x1000 | sizeof: 4B".to_string()));
        assert!(ts.contains(&"0x1010".to_string()));
        assert!(ts.contains(&"[1][2]".to_string()));
        assert!(svg.contains("fila por fila"));

        let gris = PALETA_TINTA.fill_highlight.as_ref();
        let base_svg = render(json!({"type": "matrix", "values": [[1, 2], [3, 4]]}), true);
        assert_eq!(base_svg.matches(gris).count(), 0);
        assert!(!base_svg.contains("0x1000"));
        assert!(!base_svg.contains("fila por fila"));

        let hl_str = render(
            json!({"type": "matrix", "values": [[1, 2], [3, 4]], "highlight": "1,0"}),
            true,
        );
        assert_eq!(hl_str.matches(gris).count(), 1);

        let hl_arr = render(
            json!({"type": "matrix", "values": [[1, 2], [3, 4]], "highlight": [[0, 0], [1, 1]]}),
            true,
        );
        assert_eq!(hl_arr.matches(gris).count(), 2);

        let hl_row = render(
            json!({"type": "matrix", "values": [[1, 2], [3, 4]], "highlightRows": [0]}),
            true,
        );
        assert_eq!(hl_row.matches(gris).count(), 2);

        let hl_col_lin = render(
            json!({"type": "matrix", "values": [[1, 2], [3, 4]], "highlightCols": [1], "showLinear": true}),
            true,
        );
        assert_eq!(hl_col_lin.matches(gris).count(), 4);

        let blank = render(
            json!({"type": "matrix", "values": [[1, 2], [3, 4]], "blankValues": true}),
            true,
        );
        assert!(!blank.contains(">3<"));
        assert!(blank.contains("stroke-dasharray"));

        let diez = render(
            json!({"type": "matrix", "values": vec![vec![0,1,2,3,4,5,6,7,8,9]; 10]}),
            true,
        );
        assert_eq!(diez.matches(">9<").count(), 10);
    }

    #[test]
    fn matrix_dynamic_pointers_jagged_owner_zones_y_flat() {
        let dyn_svg = render(
            json!({
                "type": "matrix",
                "variant": "dynamic",
                "name": "t",
                "values": [[1, 2, 3], null, [4]],
                "showAddresses": true,
                "baseAddr": "0x2000",
                "rowAddresses": ["0x3000", "", "0x5000"]
            }),
            true,
        );
        let ts = textos_svg(&dyn_svg);
        assert!(ts.contains(&"int **t".to_string()));
        assert!(ts.contains(&"t[1]".to_string()));
        assert!(ts.contains(&"NULL".to_string()));
        assert!(ts.contains(&"0x3000".to_string()));
        assert!(ts.contains(&"0x5000".to_string()));
        assert!(dyn_svg.contains(r#"width="168" height="40" fill="none""#));
        assert!(dyn_svg.contains(r#"width="56" height="40" fill="none""#));
        assert!(!ts.contains(&"largos".to_string()));

        // `pointers` es alias de `dynamic`
        let ptr_svg = render(
            json!({"type": "matrix", "variant": "pointers", "values": [[1]]}),
            true,
        );
        assert!(ptr_svg.contains(">int **m<"));

        // `jagged` con largos por defecto y personalizados
        let jag = render(
            json!({"type": "matrix", "variant": "jagged", "values": [[1, 2, 3], [4], null, [7, 8]]}),
            true,
        );
        let ts_jag = textos_svg(&jag);
        assert!(ts_jag.contains(&"int **m | size_t largos[4]".to_string()));
        assert!(ts_jag.contains(&"largos".to_string()));
        assert!(ts_jag.contains(&"3".to_string()));
        assert!(ts_jag.contains(&"1".to_string()));
        assert!(ts_jag.contains(&"0".to_string()));
        assert!(ts_jag.contains(&"2".to_string()));

        let jag_custom = render(
            json!({"type": "matrix", "variant": "jagged", "values": [[1, 2]], "lengths": [5], "lengthsName": "tam", "blankLengths": false}),
            true,
        );
        let ts_jc = textos_svg(&jag_custom);
        assert!(ts_jc.contains(&"tam".to_string()));
        assert!(ts_jc.contains(&"5".to_string()));

        let jag_blank_len = render(
            json!({"type": "matrix", "variant": "jagged", "values": [[1, 2, 3]], "blankLengths": true}),
            true,
        );
        assert!(jag_blank_len.contains(">3<"));
        assert_eq!(jag_blank_len.matches("stroke-dasharray").count(), 1);

        // `jagged` con centinela
        let jag_sent = render(
            json!({"type": "matrix", "variant": "jagged", "sentinel": -1, "values": [[3, 1], [2]]}),
            true,
        );
        assert!(jag_sent.contains("centinela: -1"));
        assert_eq!(jag_sent.matches(">-1<").count(), 2);
        assert!(!jag_sent.contains(">largos<"));

        let jag_sent_len = render(
            json!({"type": "matrix", "variant": "jagged", "sentinel": -1, "values": [[3, 1], [2]], "showLengths": true}),
            true,
        );
        assert!(jag_sent_len.contains(">largos<"));

        // Direcciones simbólicas
        let simb = render(
            json!({
                "type": "matrix",
                "variant": "jagged",
                "name": "mat",
                "values": [[1], [2, 3]],
                "showAddresses": true,
                "baseAddr": "@M",
                "rowAddresses": ["@F0", "@F1"],
                "lengthsType": "int"
            }),
            true,
        );
        let ts_simb = textos_svg(&simb);
        assert!(ts_simb.contains(&"@M".to_string()));
        assert!(ts_simb.contains(&"@F1".to_string()));
        assert!(ts_simb.contains(&"← @F0".to_string()));
        assert!(ts_simb.contains(&"int **mat | int largos[2]".to_string()));
        assert!(!ts_simb.iter().any(|t| t.starts_with("0x")));

        // `owner` + `zones`
        let dentada_duena = json!({
            "type": "matrix",
            "variant": "jagged",
            "name": "filas",
            "values": [[4, 8], [9]],
            "showAddresses": true,
            "baseAddr": "@F",
            "rowAddresses": ["@R0", "@R1"],
            "lengthsType": "int",
            "lengthsAddr": "@L",
            "zones": true,
            "owner": {
                "name": "d",
                "type": "dentada_t",
                "fields": ["n = 2", "largos -> lengths", "filas -> rows"]
            }
        });
        let svg_duena = render(dentada_duena.clone(), true);
        let ts_d = textos_svg(&svg_duena);
        for esperado in ["d (dentada_t)", "@L (largos: int[2])", "@F", "@L", "n"] {
            assert!(ts_d.contains(&esperado.to_string()), "falta {esperado}");
        }
        assert!(svg_duena.matches("<path").count() >= 2);

        let mut sin_dirs_duena = dentada_duena;
        sin_dirs_duena["blankAddrs"] = json!(true);
        let ts_sd = textos_svg(&render(sin_dirs_duena, true));
        assert!(!ts_sd.contains(&"@L".to_string()));

        // `flat` + `showLogical`
        let flat = render(
            json!({
                "type": "matrix",
                "variant": "flat",
                "values": [[1, 2, 3], [4, 5, 6]],
                "baseAddr": "0x1000",
                "showAddresses": true
            }),
            true,
        );
        let ts_flat = textos_svg(&flat);
        assert!(ts_flat.contains(&"[5]".to_string()));
        assert!(ts_flat.contains(&"(1,2)".to_string()));
        assert!(ts_flat.contains(&"0x1014".to_string()));
        assert!(flat.contains("malloc(2 * 3 * sizeof(int))"));
        assert!(flat.contains("m[i * 3 + j]"));

        let flat_sin = render(
            json!({
                "type": "matrix",
                "variant": "flat",
                "values": [[1, 2], [3, 4]],
                "showIndices": false,
                "showFormula": false
            }),
            true,
        );
        assert!(!flat_sin.contains(">[3]<"));
        assert!(flat_sin.contains(">(1,1)<"));
        assert!(!flat_sin.contains("i * 2"));

        let flat_log = render(
            json!({
                "type": "matrix",
                "variant": "flat",
                "values": [[1, 2]],
                "showLogical": true
            }),
            true,
        );
        let ts_log = textos_svg(&flat_log);
        assert_eq!(ts_log.iter().filter(|t| t.as_str() == "1").count(), 2);
        assert!(flat_log.contains("int m[1][2]"));
        assert!(flat_log.contains("<g transform="));
    }

    #[test]
    fn matrix_todos_los_errores_de_referencia() {
        assert!(render_err(json!({"type": "matrix", "variant": "triangular"})).contains("variante de matriz"));
        assert!(render_err(json!({"type": "matrix", "values": "1 2 3"})).contains("lista de filas"));
        assert!(render_err(json!({"type": "matrix", "highlight": "a,b"})).contains("highlight"));
        assert!(render_err(json!({"type": "matrix", "values": vec![vec![1]; 11]})).contains("11 filas; el máximo es 10"));
        assert!(render_err(json!({"type": "matrix", "values": [vec![1, 2], (0..11).collect::<Vec<i32>>()]})).contains("fila 1 de la matriz tiene 11 columnas"));
        assert!(render_err(json!({"type": "matrix", "values": (0..30).collect::<Vec<i32>>(), "cols": 2})).contains("15 filas"));
        assert!(render_err(json!({"type": "matrix", "values": [[1], 5]})).contains("cada fila de una matriz"));
        assert!(render_err(json!({"type": "matrix", "rows": 11})).contains("'rows' de una matriz tiene que estar entre 1 y 10: 11"));
        assert!(render_err(json!({"type": "matrix", "cols": 12})).contains("'cols' de una matriz tiene que estar entre 1 y 10: 12"));
        assert!(render_err(json!({"type": "matrix", "cols": 0})).contains("'cols'"));
        assert!(render_err(json!({"type": "matrix", "rows": "muchas"})).contains("tiene que ser un entero"));
        assert!(render_err(json!({"type": "matrix", "variant": "dynamic", "values": vec![vec![1]; 11]})).contains("11 filas"));
        assert!(render_err(json!({"type": "matrix", "variant": "dynamic", "cols": 11})).contains("'cols'"));
        assert!(render_err(json!({"type": "matrix", "variant": "flat", "rows": 5, "cols": 5})).contains("máximo"));
        assert!(render_err(json!({
            "type": "matrix",
            "variant": "jagged",
            "values": [[1]],
            "owner": {"name": "d", "fields": ["x -> otra"]}
        })).contains("se espera rows"));
    }

    #[test]
    fn linked_list_todas_las_variantes_cabecera_zonas_y_paletas() {
        for (variante, vals) in [
            ("singly", json!([10, 20, 30])),
            ("doubly", json!(["a", "b"])),
            ("circular", json!([1, 2, 3])),
            ("circular_doubly", json!([1, 2, 3])),
        ] {
            let svg = render(json!({"type": "linked_list", "variant": variante, "values": vals}), true);
            assert!(svg.starts_with("<svg "));
            assert!(attr_int(&svg, "width") > 50);
            assert!(attr_int(&svg, "height") > 50);
        }

        // Paleta tinta vs color
        let tinta = render(json!({"type": "linked_list", "values": [1, 2]}), true);
        let color = render(json!({"type": "linked_list", "values": [1, 2]}), false);
        assert!(!tinta.contains("#1d4ed8"));
        assert!(color.contains("#1d4ed8"));

        // Direcciones hex desde YAML/JSON
        let hex_svg = render(json!({"type": "linked_list", "values": [1, 2], "addresses": [0x3B20, 0x7E10]}), true);
        assert!(hex_svg.contains("0x3B20"));
        assert!(!hex_svg.contains("15136"));

        // Lista vacía
        let vacia = render(json!({"type": "linked_list", "headLabel": "cabeza", "values": []}), true);
        let ts_v = textos_svg(&vacia);
        assert!(ts_v.contains(&"cabeza".to_string()));
        assert!(ts_v.contains(&"NULL".to_string()));
        assert!(!ts_v.contains(&"dato".to_string()));

        // Cursores + drawArrows: false + blankNextAddrs + blankValues
        let cur = render(
            json!({
                "type": "linked_list",
                "values": [10, 20, 30],
                "pointers": "ant:0, actual:1, final:2"
            }),
            true,
        );
        let ts_cur = textos_svg(&cur);
        for p in ["ant", "actual", "final"] {
            assert!(ts_cur.contains(&p.to_string()));
        }

        let blank_l = render(
            json!({
                "type": "linked_list",
                "values": [10, 20],
                "blankNextAddrs": true,
                "blankValues": true,
                "drawArrows": false
            }),
            true,
        );
        assert!(!blank_l.contains(">10<"));
        assert!(blank_l.contains(">sig:______<"));
        assert!(!blank_l.contains("marker-end="));

        // Sin claves nuevas: Dir: 0x4000
        let simple = render(json!({"type": "linked_list", "values": [1]}), true);
        assert!(textos_svg(&simple).contains(&"Dir: 0x4000".to_string()));
        assert!(!textos_svg(&simple).contains(&"Heap".to_string()));

        // Header + zones
        let con_cab = json!({
            "type": "linked_list",
            "values": [12, 25, 8],
            "addresses": ["@N1", "@N2", "@N3"],
            "nodeType": "nodo_t",
            "zones": {"stack": "Stack: main", "heap": "Heap"},
            "header": {
                "name": "mi_lista",
                "type": "lista_t",
                "fields": ["primero -> 0", "ultimo -> 2", "cant = 3"]
            }
        });
        let svg_cab = render(con_cab.clone(), true);
        let ts_cab = textos_svg(&svg_cab);
        assert!(ts_cab.contains(&"mi_lista (lista_t)".to_string()));
        assert!(!ts_cab.contains(&"HEAD".to_string()));
        for x in ["primero", "ultimo", "cant", "3", "@N1 (nodo_t)", "Stack: main", "Heap"] {
            assert!(ts_cab.contains(&x.to_string()), "falta {x}");
        }
        assert_eq!(ts_cab.iter().filter(|t| t.as_str() == "@N3").count(), 1);

        let mut z_true = con_cab.clone();
        z_true["zones"] = json!(true);
        let ts_zt = textos_svg(&render(z_true, true));
        assert!(ts_zt.contains(&"Stack".to_string()) && ts_zt.contains(&"Heap".to_string()));

        // Errores de header
        let mut cab_fuera = con_cab.clone();
        cab_fuera["header"] = json!({"name": "l", "fields": ["ultimo -> 3"]});
        assert!(render_err(cab_fuera).contains("ultimo apunta al nodo"));

        for cab_mal in [
            json!({"name": "l"}),
            json!({"fields": []}),
            json!({"fields": ["sin separador"]}),
            json!("texto"),
        ] {
            let mut b = con_cab.clone();
            b["header"] = cab_mal;
            assert!(!render_err(b).is_empty());
        }
    }

    #[test]
    fn stack_simple_vertical_multi_blank_y_errores() {
        let s1 = render(
            json!({"type": "stack", "values": [5, 9], "capacity": 4}),
            true,
        );
        let ts1 = textos_svg(&s1);
        assert!(ts1.contains(&"5".to_string()));
        assert!(ts1.contains(&"9".to_string()));
        assert!(ts1.contains(&"TOPE".to_string()));
        assert!(ts1.contains(&"0x1000".to_string()));
        assert!(ts1.contains(&"BASE | tope = 1".to_string()));

        let s_multi = render(
            json!({
                "type": "stack",
                "layout": "vertical_multi",
                "values": [1],
                "traceLabels": ["Inicio", "push(2)", "pop()"]
            }),
            true,
        );
        let ts_m = textos_svg(&s_multi);
        assert!(ts_m.contains(&"Inicio".to_string()));
        assert!(ts_m.contains(&"push(2)".to_string()));
        assert!(ts_m.contains(&"pop()".to_string()));
        assert_eq!(ts_m.iter().filter(|t| t.as_str() == "BASE | tope = ___").count(), 2);

        let s_blank = render(
            json!({
                "type": "stack",
                "values": [5, 9],
                "blankValues": true,
                "blankPointers": true,
                "showAddresses": false
            }),
            true,
        );
        assert!(!s_blank.contains(">5<"));
        assert!(!s_blank.contains(">TOPE<"));
        assert!(!s_blank.contains("0x1000"));
        assert!(s_blank.contains(">BASE | tope = ___<"));

        // Título largo no se corta
        let titulo_largo = "P".repeat(50);
        let s_tit = render(
            json!({"type": "stack", "values": [1], "title": titulo_largo}),
            true,
        );
        assert!(attr_int(&s_tit, "width") as f64 >= 50.0 * 8.6);

        // Errores y tipos inválidos
        assert!(render_err(json!({"type": "stack", "values": 5})).contains("values"));
        assert!(render_err(json!({"type": "stack", "layout": "vertical_multi", "traceLabels": 5})).contains("traceLabels"));
    }

    #[test]
    fn queue_linear_y_circular_con_y_sin_panel() {
        let q_lin = render(
            json!({
                "type": "queue",
                "variant": "linear",
                "capacity": 6,
                "frontIndex": 1,
                "rearIndex": 3,
                "values": ["", "a", "b", "c"]
            }),
            true,
        );
        let ts_ql = textos_svg(&q_lin);
        assert!(ts_ql.contains(&"FRENTE".to_string()));
        assert!(ts_ql.contains(&"FINAL".to_string()));
        assert!(ts_ql.contains(&"a".to_string()));
        assert!(ts_ql.contains(&"c".to_string()));

        // frontIndex == rearIndex en linear no falla
        let q_eq = render(
            json!({
                "type": "queue",
                "variant": "linear",
                "frontIndex": 2,
                "rearIndex": 2,
                "values": ["", "", "x"]
            }),
            true,
        );
        assert!(q_eq.contains(">FRENTE<") && q_eq.contains(">FINAL<"));

        let q_circ = render(
            json!({
                "type": "queue",
                "variant": "circular",
                "capacity": 8,
                "frontIndex": 5,
                "rearIndex": 1,
                "values": ["z", "y"]
            }),
            true,
        );
        let ts_qc = textos_svg(&q_circ);
        assert!(ts_qc.contains(&"N = 8".to_string()));
        assert!(ts_qc.contains(&"FRENTE".to_string()));
        assert!(ts_qc.contains(&"FINAL".to_string()));
        assert!(q_circ.contains("Aritmética modular:"));
        assert_eq!(attr_int(&q_circ, "width"), 740 + 16);

        let q_circ_sin_panel = render(
            json!({
                "type": "queue",
                "variant": "circular",
                "capacity": 6,
                "frontIndex": 2,
                "rearIndex": 2,
                "showPanel": false
            }),
            true,
        );
        assert!(q_circ_sin_panel.contains(">FRENTE=FINAL<"));
        assert!(!q_circ_sin_panel.contains("Aritmética modular:"));
        assert_eq!(attr_int(&q_circ_sin_panel, "width"), 320 + 16);

        // Variante desconocida hace fallback a lineal
        let q_fallback = render(json!({"type": "queue", "variant": "priority"}), true);
        assert!(q_fallback.contains(">DESENCOLAR<"));

        assert!(render_err(json!({"type": "queue", "values": "abc"})).contains("values"));
    }

    #[test]
    fn errores_con_prefijo_de_tipo_e_idempotencia() {
        let casos = [
            ("array", json!({"type": "array", "values": 5})),
            ("matrix", json!({"type": "matrix", "values": 5})),
            ("matrix", json!({"type": "matrix", "variant": "triangular"})),
            ("matrix", json!({"type": "matrix", "highlightRows": 5})),
            (
                "matrix",
                json!({"type": "matrix", "variant": "dynamic", "rowAddresses": 5}),
            ),
            (
                "matrix",
                json!({"type": "matrix", "variant": "jagged", "lengths": 5}),
            ),
            (
                "matrix",
                json!({"type": "matrix", "variant": "flat", "showLogical": true, "highlightRows": 5}),
            ),
            ("linked_list", json!({"type": "linked_list", "values": 5})),
            (
                "linked_list",
                json!({"type": "linked_list", "values": [1, 2], "addresses": 5}),
            ),
            (
                "linked_list",
                json!({"type": "linked_list", "values": [1, 2], "header": "mal"}),
            ),
            ("stack", json!({"type": "stack", "values": 5})),
            (
                "stack",
                json!({"type": "stack", "layout": "vertical_multi", "traceLabels": 5}),
            ),
            ("queue", json!({"type": "queue", "values": 5})),
            ("queue", json!({"type": "queue", "values": "abc"})),
        ];

        for (tipo, bloque) in casos {
            let err = render_err(bloque.clone());
            let prefijo = format!("{tipo}: ");
            assert!(
                err.starts_with(&prefijo),
                "se esperaba prefijo '{prefijo}' para {bloque}, obtenido: {err}"
            );
            let doble = format!("{prefijo}{prefijo}");
            assert!(
                !err.starts_with(&doble),
                "prefijo duplicado inesperado en {bloque}: {err}"
            );
        }

        // Recursión `showLogical` en `matrix` (caso válido + propagación sin doble prefijo)
        let flat_log = render(
            json!({
                "type": "matrix",
                "variant": "flat",
                "values": [[1, 2]],
                "showLogical": true
            }),
            true,
        );
        assert!(flat_log.contains("Vista lógica"));

        let mut lienzo = Lienzo::new(dibu_core::PALETA_TINTA.clone(), String::new());
        let sub_bloque = json!({"rows": 11})
            .as_object()
            .expect("objeto JSON")
            .clone();
        let err_recursivo = envolver_tipo("matrix", render_matrix(&mut lienzo, &sub_bloque))
            .expect_err("se esperaba ErrorDiagrama")
            .to_string();
        assert!(err_recursivo.starts_with("matrix: "));
        assert!(!err_recursivo.starts_with("matrix: matrix: "));
    }

    /// Las flechas (líneas con punta) de un SVG.
    fn flechas(svg: &str) -> usize {
        svg.matches("marker-end=\"url(#flecha-main").count()
    }

    #[test]
    fn matriz_java_irregular_distingue_null_de_largo_cero() {
        let svg = render(
            json!({"type": "matrix", "variant": "java", "name": "matriz",
                   "values": [[1, 2, 3], [4], null, []], "zones": true}),
            true,
        );
        let textos = textos_svg(&svg);
        assert!(svg.contains("int[][] matriz"), "declaración con sintaxis de Java");
        assert!(textos.iter().any(|t| t == "null"), "la tercera fila es null");
        assert!(textos.iter().any(|t| t == "int[] · length 0"), "la cuarta es un arreglo de largo 0");
        assert!(textos.iter().any(|t| t == "int[] · length 3") && textos.iter().any(|t| t == "int[] · length 1"));
        assert!(textos.iter().any(|t| t == "int[][] · length 4"));
        // La variable y las tres filas no nulas: null no tiene flecha.
        assert_eq!(flechas(&svg), 4);
        assert!(textos.iter().any(|t| t == "Stack") && textos.iter().any(|t| t == "Heap"));
    }

    #[test]
    fn matriz_java_rectangular_y_exterior_vacio() {
        let svg = render(json!({"type": "matrix", "variant": "java", "values": [[1, 2], [3, 4], [5, 6]]}), true);
        assert_eq!(flechas(&svg), 4);
        assert_eq!(textos_svg(&svg).iter().filter(|t| *t == "int[] · length 2").count(), 3);
        let vacia = render(json!({"type": "matrix", "variant": "java", "name": "v", "values": []}), true);
        assert!(textos_svg(&vacia).iter().any(|t| t == "int[][] · length 0"));
        assert_eq!(flechas(&vacia), 1, "sólo la variable refiere al arreglo vacío");
        let sin_valores = render(json!({"type": "matrix", "variant": "java", "rows": 2, "cols": 3}), true);
        assert_eq!(flechas(&sin_valores), 3);
    }

    #[test]
    fn matriz_java_enlaces_salen_de_su_indice() {
        let svg = render(json!({"type": "matrix", "variant": "java", "values": [[1], null, [2]]}), true);
        // Las flechas de las filas salen de la celda [0] y de la [2] del arreglo exterior.
        let ys: Vec<f64> = svg
            .split("<line ")
            .skip(1)
            .filter(|l| l.contains("flecha-main"))
            .map(|l| {
                let y1 = l.split("y1=\"").nth(1).unwrap();
                y1[..y1.find('"').unwrap()].parse().unwrap()
            })
            .collect();
        assert_eq!(ys.len(), 3);
        assert!(ys[2] - ys[1] > 70.0, "la fila 1 es null: no sale nada de su celda");
    }

    #[test]
    fn matriz_java_errores_y_ejercicio() {
        assert!(render_err(json!({"type": "matrix", "variant": "java", "values": [1, 2]})).contains("tiene que ser una lista (o null)"));
        let svg = render(json!({"type": "matrix", "variant": "java", "values": [[1, 2]], "blankValues": true, "blankLengths": true}), true);
        let textos = textos_svg(&svg);
        assert!(!textos.iter().any(|t| t == "1"));
        assert!(textos.iter().any(|t| t == "int[] · length __"));
    }

    fn corte(n: usize, i: Option<i64>, f: Option<i64>, p: Option<i64>) -> Vec<usize> {
        indices_corte(n, i, f, p).unwrap().3
    }

    #[test]
    fn indices_de_corte_como_python() {
        // Comparados con `list(range(*slice(i, f, p).indices(n)))` de Python.
        assert_eq!(corte(5, Some(1), Some(4), None), [1, 2, 3]);
        assert_eq!(corte(5, Some(-4), Some(-1), Some(2)), [1, 3]);
        assert_eq!(corte(5, None, None, Some(-1)), [4, 3, 2, 1, 0]);
        assert_eq!(corte(5, Some(2), Some(2), None), Vec::<usize>::new());
        assert_eq!(corte(5, Some(-10), Some(10), None), [0, 1, 2, 3, 4]);
        assert_eq!(corte(5, Some(10), Some(-10), Some(-2)), [4, 2, 0]);
        assert_eq!(corte(5, Some(3), Some(0), Some(-1)), [3, 2, 1]);
        assert_eq!(corte(0, None, None, Some(-1)), Vec::<usize>::new());
        assert!(indices_corte(5, None, None, Some(0)).unwrap_err().0.contains("no puede ser 0"));
    }

    #[test]
    fn secuencia_python_dibuja_el_corte() {
        let svg = render(
            json!({"type": "array", "variant": "python_sequence", "values": [2, 4, 7, 1, 8],
                   "showNegativeIndices": true, "showBoundaries": true,
                   "slice": {"start": -4, "stop": -1, "step": 2}}),
            true,
        );
        let textos = textos_svg(&svg);
        for t in ["a = [2, 4, 7, 1, 8]", "a[-4:-1:2]", "[4, 1]", "fin (excluido)", "inicio", "-5", "1º", "2º"] {
            assert!(textos.iter().any(|x| x == t), "falta {t:?}: {textos:?}");
        }
        assert!(svg.contains(r#">5</text>"#), "la frontera n queda visible");
    }

    #[test]
    fn secuencia_python_en_blanco_no_da_pistas() {
        let svg = render(
            json!({"type": "array", "variant": "python_sequence", "values": [2, 4, 7, 1, 8],
                   "showBoundaries": true, "slice": {"start": -4, "stop": -1, "step": 2, "blankSelection": true}}),
            true,
        );
        let textos = textos_svg(&svg);
        assert!(textos.iter().any(|t| t == "a[-4:-1:2]"));
        for pista in ["[4, 1]", "fin (excluido)", "inicio", "1º"] {
            assert!(!textos.iter().any(|t| t == pista), "pista {pista:?} en el ejercicio");
        }
        let resuelto = render(json!({"type": "array", "variant": "python_sequence", "values": [2, 4], "slice": {"stop": 1}}), true);
        assert!(resuelto.matches(PALETA_TINTA.fill_highlight.as_ref()).count() > svg.matches(PALETA_TINTA.fill_highlight.as_ref()).count());
    }

    #[test]
    fn secuencia_python_cadenas_tuplas_y_errores() {
        let s = render(json!({"type": "array", "variant": "python_sequence", "kind": "str", "values": "a b", "slice": {"step": -1}}), true);
        let textos = textos_svg(&s);
        assert!(textos.iter().any(|t| t == "␣"), "el espacio se ve");
        assert!(textos.iter().any(|t| t == "&#x27;b a&#x27;"));
        let t = render(json!({"type": "array", "variant": "python_sequence", "kind": "tuple", "values": [3, 4], "slice": {"start": 1}}), true);
        assert!(textos_svg(&t).iter().any(|x| x == "(4,)"));
        let vacia = render(json!({"type": "array", "variant": "python_sequence", "values": [], "slice": {"step": -1}}), true);
        assert!(textos_svg(&vacia).iter().any(|x| x == "[]"));
        assert!(render_err(json!({"type": "array", "variant": "python_sequence", "values": [1], "slice": {"step": 0}})).contains("no puede ser 0"));
        assert!(render_err(json!({"type": "array", "variant": "python_sequence", "kind": "set"})).contains("list, tuple o str"));
        assert!(render_err(json!({"type": "array", "variant": "otra"})).contains("python_sequence"));
    }

    #[test]
    fn array_limite_excluido_e_intervalo() {
        let s = render(json!({"type": "array", "values": [1, 2, 3], "pointers": "p:0, fin:3", "range": {"start": 1, "stop": 3, "order": "reverse"}}), true);
        assert!(s.contains(">fin<"), "el puntero al límite ya no se descarta");
        assert_eq!(s.matches(">[3]<").count(), 0, "sin una celda a[3]");
        assert!(s.contains(">[1, 3)<") && s.contains(">2<"));
        let vacio = render(json!({"type": "array", "values": [1, 2], "range": {"start": 2, "stop": 2}}), true);
        assert!(vacio.contains("vacío"));
        let oculto = render(json!({"type": "array", "values": [1, 2], "range": {"start": 0, "stop": 1, "blank": true}}), true);
        assert!(!oculto.contains(">[0, 1)<"));
        assert!(render_err(json!({"type": "array", "values": [1], "range": {"stop": 2}})).contains("entre 0 y 1"));
    }

    #[test]
    fn cadena_c_terminador_y_capacidad() {
        let s = render(json!({"type": "array", "variant": "c_string", "bytes": ["a", "\\0", "b", 0], "capacity": 6}), true);
        assert!(s.contains("strlen(s) = 1 · capacidad = 6"), "el primer NUL fija la longitud");
        assert_eq!(s.matches("&#x27;\\0&#x27;").count(), 2, "los dos NUL se ven");
        assert!(s.contains(">después del terminador<") && s.contains(">sin datos<"));
        let sin = render(json!({"type": "array", "variant": "c_string", "text": "hola", "capacity": 4}), true);
        assert!(sin.contains("sin terminador") && !sin.contains("strlen(s) ="));
        let blanco = render(json!({"type": "array", "variant": "c_string", "text": "ab", "blankLength": true}), true);
        assert!(blanco.contains("strlen(s) = ______") && !blanco.contains("strlen = 2"));
        assert!(render_err(json!({"type": "array", "variant": "c_string", "text": "hola", "capacity": 3})).contains("no entran"));
    }

    #[test]
    fn cola_circular_por_frente_y_cantidad() {
        let s = render(json!({"type": "queue", "variant": "ring", "capacity": 5, "frontIndex": 4, "values": [7, 9]}), true);
        // El 7 en la celda 4 y el 9 en la 0: el 9 se dibuja antes (más a la izquierda).
        assert!(s.find(">9<").unwrap() < s.find(">7<").unwrap());
        assert!(s.contains(">último = 0<") && s.contains(">siguiente = 1<") && s.contains(">cantidad = 2<"));
        let llena = render(json!({"type": "queue", "variant": "deque", "capacity": 3, "frontIndex": 1, "values": [1, 2, 3], "blank": ["count", "last"]}), true);
        assert!(llena.contains(">cantidad = ____<") && llena.contains(">último = ____<") && llena.contains("quitar del final"));
        let vacia = render(json!({"type": "queue", "variant": "ring", "capacity": 3, "frontIndex": 2, "values": []}), true);
        assert!(vacia.contains("— (vacía)"));
        assert!(render_err(json!({"type": "queue", "variant": "ring", "capacity": 2, "frontIndex": 0, "values": [1, 2, 3]})).contains("no entran"));
    }

    #[test]
    fn matriz_con_punteros_a_un_bloque() {
        let s = render(json!({"type": "matrix", "variant": "row_pointers", "values": [[1, 2, 3], [4, 5, 6]]}), true);
        assert!(s.contains("fila 1: desde [3]") && s.contains("free(m[0]); free(m);"));
        let blanco = render(json!({"type": "matrix", "variant": "row_pointers", "rows": 2, "cols": 2, "blankPointers": true}), true);
        assert_eq!(blanco.split("</defs>").nth(1).unwrap().matches("<path d=\"M ").count(), 1, "sólo m → la tabla");
    }
}
