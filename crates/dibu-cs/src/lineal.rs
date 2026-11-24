//! Renderizadores lineales de `dibu-cs` (`array`, `matrix`, `linked_list`, `stack`, `queue`).

use crate::common::{
    ancho_cabecera, cabecera, campos_cabecera, encabezado, entero_b, enteros_set, es_truthy,
    fmt_hex, fmt_num_clean, hex, hex_b, lista_o_vacia, nombres_de_zonas, punteros, str_o,
    sub_lienzo, agregar_sub_lienzo, texto, texto_opt, valores, zona, Bloque, ErrorDiagrama,
    Lienzo, Punto, ALTO_ZONA, FILA_CABECERA, MONO, SANS,
};
use dibu_core::Registro;
use serde_json::Value;
use std::collections::HashSet;
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
    let ptrs = punteros(b, cap);

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
            let txt = etiquetas.join(", ");
            let ancho = 40.0_f64.max(txt.chars().count() as f64 * 7.5 + 12.0);
            c.r(
                centro - ancho / 2.0,
                y - 48.0,
                ancho,
                18.0,
                "fillBadge",
                1.2,
                3.0,
            );
            c.t(centro, y - 35.0, &txt, 10.0, 700, "textMain", "middle", MONO);
            c.linea(
                centro,
                y - 30.0,
                centro,
                y - 19.0,
                "accentPrimary",
                1.5,
                Some("main"),
                "",
            );
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

    let alto = y + celda_h + if offsets { 42.0 } else if direcciones { 26.0 } else { 12.0 };
    Ok((x0 + cap as f64 * celda_w + 20.0, alto))
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
                "variante de matriz no soportada: {} (static, dynamic, jagged o flat)",
                repr_val(otro)
            )));
        }
    };

    match variante {
        "dynamic" | "pointers" => render_matrix_dinamica(c, b, false),
        "jagged" => render_matrix_dinamica(c, b, true),
        "flat" => render_matrix_plana(c, b),
        "static" => render_matrix_estatica(c, b),
        otra => Err(ErrorDiagrama::new(format!(
            "variante de matriz no soportada: '{otra}' (static, dynamic, jagged o flat)"
        ))),
    }
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
}
