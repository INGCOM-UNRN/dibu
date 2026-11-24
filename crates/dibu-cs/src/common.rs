//! Utilidades y primitivas compartidas para los diagramas de estructuras de datos y algoritmos.

pub use dibu_core::{
    ancho_texto, bool_opt, encabezado, esc, fmt_num_clean, str_opt, valor_str, Bloque,
    ErrorDiagrama, Lienzo, Paleta, Punto, MONO, SANS,
};

use serde_json::Value;
use std::collections::{BTreeMap, HashSet};
use std::num::IntErrorKind;

pub const FILA_CABECERA: f64 = 26.0;
pub const ALTO_ZONA: f64 = 20.0;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CampoCabecera {
    pub nombre: String,
    pub valor: String,
    pub destino: Option<String>,
}

#[derive(Debug, Clone, Copy)]
pub struct TablaOpts<'a> {
    pub izquierda: &'a [usize],
    pub tenues: &'a [usize],
    pub fila_h: f64,
    pub min_w: f64,
    pub anchos: Option<&'a [f64]>,
}

impl Default for TablaOpts<'_> {
    fn default() -> Self {
        Self {
            izquierda: &[],
            tenues: &[],
            fila_h: 28.0,
            min_w: 46.0,
            anchos: None,
        }
    }
}

pub fn es_truthy(v: &Value) -> bool {
    match v {
        Value::Null => false,
        Value::Bool(b) => *b,
        Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                i != 0
            } else if let Some(u) = n.as_u64() {
                u != 0
            } else {
                n.as_f64().is_some_and(|x| x != 0.0)
            }
        }
        Value::String(s) => !s.is_empty(),
        Value::Array(a) => !a.is_empty(),
        Value::Object(o) => !o.is_empty(),
    }
}

pub fn texto(valor: &Value) -> String {
    match valor {
        Value::Array(arr) => {
            let items: Vec<String> = arr.iter().map(texto).collect();
            format!("[{}]", items.join(", "))
        }
        _ => valor_str(valor),
    }
}

pub fn str_o(b: &Bloque, clave: &str, defecto: &str) -> String {
    b.get(clave)
        .filter(|v| es_truthy(v))
        .map(texto)
        .unwrap_or_else(|| defecto.to_string())
}

pub fn texto_opt(b: &Bloque, clave: &str) -> Option<String> {
    b.get(clave).filter(|v| !v.is_null()).map(texto)
}

pub fn hex(valor: Option<&Value>, defecto: i64) -> i64 {
    match valor {
        Some(Value::Number(n)) => {
            if let Some(i) = n.as_i64() {
                i
            } else if let Some(u) = n.as_u64() {
                i64::try_from(u).unwrap_or(i64::MAX)
            } else {
                defecto
            }
        }
        Some(Value::String(s)) => {
            let trimmed = s.trim();
            let rest = trimmed
                .strip_prefix("0x")
                .or_else(|| trimmed.strip_prefix("0X"))
                .unwrap_or(trimmed);
            if rest.is_empty() {
                return defecto;
            }
            i64::from_str_radix(rest, 16).unwrap_or(defecto)
        }
        _ => defecto,
    }
}

pub fn hex_b(b: &Bloque, clave: &str, defecto: i64) -> i64 {
    hex(b.get(clave), defecto)
}

pub fn fmt_hex(numero: i64) -> String {
    let n = numero.max(0);
    format!("0x{n:04X}")
}

pub fn entero(valor: Option<&Value>, defecto: i64, minimo: i64, maximo: i64) -> i64 {
    let (lo, hi) = if minimo <= maximo {
        (minimo, maximo)
    } else {
        (maximo, minimo)
    };
    let clamp = |x: i64| x.clamp(lo, hi);

    match valor {
        None | Some(Value::Null | Value::Array(_) | Value::Object(_)) => clamp(defecto),
        Some(Value::Bool(b)) => clamp(if *b { 1 } else { 0 }),
        Some(Value::Number(n)) => {
            if let Some(i) = n.as_i64() {
                clamp(i)
            } else if let Some(u) = n.as_u64() {
                match i64::try_from(u) {
                    Ok(i) => clamp(i),
                    Err(_) => hi,
                }
            } else if let Some(f) = n.as_f64() {
                if !f.is_finite() {
                    return clamp(defecto);
                }
                let t = f.trunc();
                if t >= hi as f64 {
                    hi
                } else if t <= lo as f64 {
                    lo
                } else {
                    clamp(t as i64)
                }
            } else {
                clamp(defecto)
            }
        }
        Some(Value::String(s)) => match s.trim().parse::<i64>() {
            Ok(n) => clamp(n),
            Err(e) => match e.kind() {
                IntErrorKind::PosOverflow => hi,
                IntErrorKind::NegOverflow => lo,
                _ => clamp(defecto),
            },
        },
    }
}

pub fn entero_b(b: &Bloque, clave: &str, defecto: i64, minimo: i64, maximo: i64) -> i64 {
    entero(b.get(clave), defecto, minimo, maximo)
}

pub fn lista_o_vacia<'a>(b: &'a Bloque, clave: &str) -> Result<&'a [Value], ErrorDiagrama> {
    match b.get(clave) {
        None => Ok(&[]),
        Some(v) if !es_truthy(v) => Ok(&[]),
        Some(Value::Array(arr)) => Ok(arr.as_slice()),
        Some(_) => Err(ErrorDiagrama::new(format!(
            "'{clave}' tiene que ser una lista"
        ))),
    }
}

pub fn valores(b: &Bloque) -> Result<Vec<String>, ErrorDiagrama> {
    let items = lista_o_vacia(b, "values")?;
    Ok(items
        .iter()
        .map(|v| if v.is_null() { String::new() } else { texto(v) })
        .collect())
}

pub fn lista_de<'a>(
    b: &'a Bloque,
    clave: &str,
    tipo: &str,
) -> Result<&'a [Value], ErrorDiagrama> {
    match b.get(clave) {
        Some(Value::Array(arr)) if !arr.is_empty() => Ok(arr.as_slice()),
        _ => Err(ErrorDiagrama::new(format!(
            "'{clave}' de {tipo} tiene que ser una lista con al menos un elemento"
        ))),
    }
}

pub fn punteros(b: &Bloque, limite: usize) -> BTreeMap<usize, Vec<String>> {
    let mut salida: BTreeMap<usize, Vec<String>> = BTreeMap::new();
    let Some(raw) = b.get("pointers").filter(|v| es_truthy(v)).map(texto) else {
        return salida;
    };
    for par in raw.split(',') {
        let Some((etiqueta, idx_raw)) = par.split_once(':') else {
            continue;
        };
        let idx_s = idx_raw.trim();
        if idx_s.is_empty() || !idx_s.chars().all(|c| c.is_ascii_digit()) {
            continue;
        }
        if let Ok(idx) = idx_s.parse::<usize>() {
            if idx < limite {
                salida.entry(idx).or_default().push(etiqueta.trim().to_string());
            }
        }
    }
    salida
}

pub fn enteros_set(b: &Bloque, clave: &str) -> Result<HashSet<i64>, ErrorDiagrama> {
    let mut set = HashSet::new();
    for v in lista_o_vacia(b, clave)? {
        if let Some(i) = v.as_i64() {
            set.insert(i);
        } else if let Some(u) = v.as_u64() {
            if let Ok(i) = i64::try_from(u) {
                set.insert(i);
            }
        }
    }
    Ok(set)
}

pub fn strings_set(b: &Bloque, clave: &str) -> Result<HashSet<String>, ErrorDiagrama> {
    let mut set = HashSet::new();
    for v in lista_o_vacia(b, clave)? {
        set.insert(texto(v));
    }
    Ok(set)
}

pub fn campos_cabecera(cabecera: &Value, donde: &str) -> Result<Vec<CampoCabecera>, ErrorDiagrama> {
    let Some(obj) = cabecera.as_object() else {
        return Err(ErrorDiagrama::new(format!(
            "'{donde}' necesita 'fields', la lista de sus campos"
        )));
    };
    let Some(fields) = obj.get("fields").and_then(Value::as_array) else {
        return Err(ErrorDiagrama::new(format!(
            "'{donde}' necesita 'fields', la lista de sus campos"
        )));
    };
    if fields.is_empty() {
        return Err(ErrorDiagrama::new(format!(
            "'{donde}' necesita 'fields', la lista de sus campos"
        )));
    }
    let mut campos = Vec::with_capacity(fields.len());
    for dato in fields {
        if let Some(d) = dato.as_object() {
            if let Some(nombre) = d.get("name").filter(|v| es_truthy(v)) {
                let valor = d.get("value").map(texto).unwrap_or_default();
                let destino = d.get("to").filter(|v| !v.is_null()).map(texto);
                campos.push(CampoCabecera {
                    nombre: texto(nombre),
                    valor,
                    destino,
                });
                continue;
            }
        }
        if let Some(s) = dato.as_str() {
            if let Some((nombre, destino)) = s.split_once("->") {
                campos.push(CampoCabecera {
                    nombre: nombre.trim().to_string(),
                    valor: String::new(),
                    destino: Some(destino.trim().to_string()),
                });
                continue;
            }
            if let Some((nombre, valor)) = s.split_once('=') {
                campos.push(CampoCabecera {
                    nombre: nombre.trim().to_string(),
                    valor: valor.trim().to_string(),
                    destino: None,
                });
                continue;
            }
        }
        return Err(ErrorDiagrama::new(format!(
            "'{donde}': campo inválido {dato} (se espera 'x = 5', 'p -> destino' o name/value/to)"
        )));
    }
    Ok(campos)
}

pub fn titulo_cabecera(cabecera: &Bloque) -> String {
    let nombre = cabecera
        .get("name")
        .filter(|v| es_truthy(v))
        .map(texto)
        .unwrap_or_default();
    match cabecera.get("type").filter(|v| es_truthy(v)).map(texto) {
        Some(tipo) => format!("{nombre} ({tipo})"),
        None => nombre,
    }
}

pub fn ancho_cabecera(
    cabecera: &Bloque,
    campos: &[CampoCabecera],
    valores: &[String],
) -> f64 {
    let max_nombre = campos
        .iter()
        .map(|campo| campo.nombre.chars().count())
        .max()
        .unwrap_or(0) as f64;
    let nombres = max_nombre * 7.4 + 16.0;
    let max_valor = valores
        .iter()
        .map(|v| v.chars().count())
        .max()
        .unwrap_or(0) as f64;
    let titulo = titulo_cabecera(cabecera);
    (nombres + (max_valor * 8.0 + 18.0).max(44.0)).max(titulo.chars().count() as f64 * 7.0 + 8.0)
}

pub fn cabecera(
    c: &mut Lienzo,
    x: f64,
    y: f64,
    ancho: f64,
    cab: &Bloque,
    campos: &[CampoCabecera],
    valores: &[String],
) -> Result<Vec<f64>, ErrorDiagrama> {
    if campos.len() != valores.len() {
        return Err(ErrorDiagrama::new(format!(
            "cabecera: la cantidad de campos ({}) no coincide con la de valores ({})",
            campos.len(),
            valores.len()
        )));
    }
    let titulo = titulo_cabecera(cab);
    c.t(x, y - 6.0, &titulo, 10.5, 700, "textMain", "start", MONO);
    let max_nombre = campos
        .iter()
        .map(|campo| campo.nombre.chars().count())
        .max()
        .unwrap_or(0) as f64;
    let nombres = max_nombre * 7.4 + 16.0;
    let mut centros = Vec::with_capacity(campos.len());
    for (i, (campo, valor)) in campos.iter().zip(valores.iter()).enumerate() {
        let fila = y + i as f64 * FILA_CABECERA;
        let relleno = if campo.destino.is_some() {
            "fillPointerBox"
        } else {
            "fillCell"
        };
        c.r(x, fila, ancho, FILA_CABECERA, relleno, 1.5, 0.0);
        c.linea(
            x + nombres,
            fila,
            x + nombres,
            fila + FILA_CABECERA,
            "strokeMain",
            1.0,
            None,
            "",
        );
        c.t(
            x + 8.0,
            fila + 17.0,
            &campo.nombre,
            11.0,
            400,
            "textMain",
            "start",
            MONO,
        );
        c.t(
            x + (nombres + ancho) / 2.0,
            fila + 17.0,
            valor,
            11.5,
            700,
            "textMain",
            "middle",
            MONO,
        );
        centros.push(fila + FILA_CABECERA / 2.0);
    }
    Ok(centros)
}

pub fn nombres_de_zonas(b: &Bloque) -> Option<(String, String)> {
    let zonas = b.get("zones").filter(|v| es_truthy(v))?;
    if let Some(propios) = zonas.as_object() {
        let stack = propios
            .get("stack")
            .filter(|v| es_truthy(v))
            .map(texto)
            .unwrap_or_else(|| "Stack".to_string());
        let heap = propios
            .get("heap")
            .filter(|v| es_truthy(v))
            .map(texto)
            .unwrap_or_else(|| "Heap".to_string());
        Some((stack, heap))
    } else {
        Some(("Stack".to_string(), "Heap".to_string()))
    }
}

pub fn zona(c: &mut Lienzo, x1: f64, x2: f64, y1: f64, y2: f64, titulo: &str) {
    let w = x2 - x1;
    let h = y2 - y1;
    c.add(format!(
        r#"<rect x="{x1:.1}" y="{y1:.1}" width="{w:.1}" height="{h:.1}" rx="4" fill="none" stroke="{}" stroke-width="1" stroke-dasharray="5 3"/>"#,
        c.pal.stroke_muted
    ));
    c.t(
        x1 + 6.0,
        y1 + 13.0,
        titulo,
        10.0,
        700,
        "textSub",
        "start",
        SANS,
    );
}

pub fn codigo(c: &mut Lienzo, x: f64, y: f64, texto_linea: &str, size: f64, color: &str) {
    let resto = texto_linea.trim_start_matches(' ');
    let sangria = texto_linea.chars().count() - resto.chars().count();
    c.t(
        x + dibu_core::avance_mono(sangria as f64, size),
        y,
        resto,
        size,
        400,
        color,
        "start",
        MONO,
    );
}

pub fn tabla(
    c: &mut Lienzo,
    x: f64,
    y: f64,
    encabezados: &[&str],
    filas: &[Vec<String>],
    opts: TablaOpts<'_>,
) -> Punto {
    let cab_h = 26.0_f64;
    let size = 10.5_f64;
    let columnas = encabezados.len();
    let mut anchos: Vec<f64> = Vec::with_capacity(columnas);
    for (j, enc) in encabezados.iter().enumerate() {
        if let Some(a) = opts.anchos.and_then(|s| s.get(j).copied()) {
            anchos.push(a);
        } else {
            let mut w = opts.min_w.max(ancho_texto(enc, size) + 16.0);
            for f in filas {
                if let Some(celda) = f.get(j) {
                    w = w.max(ancho_texto(celda, size) + 18.0);
                }
            }
            anchos.push(w);
        }
    }

    let mut cx = x;
    for (j, titulo) in encabezados.iter().enumerate() {
        let w = anchos[j];
        c.r(cx, y, w, cab_h, "fillHeader", 1.5, 0.0);
        c.t(
            cx + w / 2.0,
            y + 17.0,
            titulo,
            size,
            700,
            "textMain",
            "middle",
            MONO,
        );
        cx += w;
    }

    for (i, fila) in filas.iter().enumerate() {
        let fy = y + cab_h + i as f64 * opts.fila_h;
        let mut cx = x;
        let tenue = opts.tenues.contains(&i);
        let color = if tenue { "textMuted" } else { "textMain" };
        for (j, &w) in anchos.iter().enumerate() {
            let texto_celda = fila.get(j).map(String::as_str).unwrap_or("");
            if tenue {
                c.add(format!(
                    r#"<rect x="{cx:.1}" y="{fy:.1}" width="{w:.1}" height="{}" fill="none" stroke="{}" stroke-width="1" stroke-dasharray="4 3"/>"#,
                    fmt_num_clean(opts.fila_h),
                    c.pal.stroke_muted
                ));
            } else {
                c.r(cx, fy, w, opts.fila_h, "fillCell", 1.2, 0.0);
            }
            if texto_celda.is_empty() {
                c.vacio(cx + 8.0, cx + w - 8.0, fy + opts.fila_h - 8.0);
            } else if opts.izquierda.contains(&j) {
                codigo(
                    c,
                    cx + 9.0,
                    fy + opts.fila_h / 2.0 + 4.0,
                    texto_celda,
                    size,
                    color,
                );
            } else {
                c.t(
                    cx + w / 2.0,
                    fy + opts.fila_h / 2.0 + 4.0,
                    texto_celda,
                    size,
                    600,
                    color,
                    "middle",
                    MONO,
                );
            }
            cx += w;
        }
    }

    (anchos.iter().sum(), cab_h + filas.len() as f64 * opts.fila_h)
}

pub fn panel(
    c: &mut Lienzo,
    x: f64,
    y: f64,
    w: f64,
    titulo: &str,
    lineas: &[String],
    alto: Option<f64>,
) -> f64 {
    let cab_h = 24.0_f64;
    let renglon = 17.0_f64;
    let h = alto.unwrap_or_else(|| cab_h + 8.0 + lineas.len().max(1) as f64 * renglon);
    c.r(x, y, w, h, "fillCell", 1.6, 4.0);
    c.add(format!(
        r#"<rect x="{:.1}" y="{:.1}" width="{:.1}" height="{}" rx="3" fill="{}"/>"#,
        x + 1.0,
        y + 1.0,
        w - 2.0,
        fmt_num_clean(cab_h - 1.0),
        c.pal.fill_header
    ));
    c.linea(x, y + cab_h, x + w, y + cab_h, "strokeMain", 1.0, None, "");
    c.t(
        x + 9.0,
        y + 16.0,
        titulo,
        11.0,
        700,
        "textMain",
        "start",
        SANS,
    );
    for (i, linea) in lineas.iter().enumerate() {
        if !linea.is_empty() {
            codigo(
                c,
                x + 10.0,
                y + cab_h + 17.0 + i as f64 * renglon,
                linea,
                10.5,
                "textMain",
            );
        } else {
            c.vacio(
                x + 10.0,
                x + w - 10.0,
                y + cab_h + 19.0 + i as f64 * renglon,
            );
        }
    }
    h
}

pub fn ancho_panel(titulo: &str, lineas: &[String]) -> f64 {
    let mut max_w = ancho_texto(titulo, 11.0) + 20.0;
    for linea in lineas {
        max_w = max_w.max(ancho_texto(linea, 10.5) + 22.0);
    }
    max_w
}

pub fn sub_lienzo(c: &Lienzo) -> Lienzo {
    c.sub_lienzo()
}

pub fn agregar_sub_lienzo(c: &mut Lienzo, dx: f64, dy: f64, sub: Lienzo) {
    let tx = if dx == 0.0 {
        "0".to_string()
    } else {
        format!("{dx:.1}")
    };
    let ty = format!("{dy:.1}");
    c.ancho_min = c.ancho_min.max(dx + sub.ancho_min);
    c.add(format!(
        r#"<g transform="translate({tx}, {ty})">{}</g>"#,
        sub.partes.concat()
    ));
}

#[cfg(test)]
mod tests {
    use super::*;
    use dibu_core::PALETA_TINTA;
    use serde_json::json;

    #[test]
    fn hex_casos_borde() {
        assert_eq!(hex(None, 0x1000), 0x1000);
        assert_eq!(hex(Some(&json!(null)), 0x1000), 0x1000);
        assert_eq!(hex(Some(&json!(true)), 0x1000), 0x1000);
        assert_eq!(hex(Some(&json!(false)), 0x1000), 0x1000);
        assert_eq!(hex(Some(&json!(0x7FF0)), 0x1000), 0x7FF0);
        assert_eq!(hex(Some(&json!(4096.0)), 0x1000), 0x1000);
        assert_eq!(hex(Some(&json!(" 0X7ffc ")), 0x1000), 0x7FFC);
        assert_eq!(hex(Some(&json!("3b20")), 0x1000), 0x3B20);
        assert_eq!(hex(Some(&json!("-10")), 0x1000), -16);
        assert_eq!(hex(Some(&json!("-0x10")), 0x1000), 0x1000);
        assert_eq!(hex(Some(&json!("@M")), 0x1000), 0x1000);
        assert_eq!(hex(Some(&json!("0x")), 0x1000), 0x1000);
        assert_eq!(hex(Some(&json!("")), 0x1000), 0x1000);
        assert_eq!(
            hex(Some(&json!("0xffffffffffffffffffffffffffff")), 0x1000),
            0x1000
        );

        let mut b = Bloque::new();
        b.insert("baseAddr".to_string(), json!("0x2000"));
        assert_eq!(hex_b(&b, "baseAddr", 0x1000), 0x2000);
        assert_eq!(hex_b(&b, "otra", 0x1000), 0x1000);
    }

    #[test]
    fn fmt_hex_rango() {
        assert_eq!(fmt_hex(-5), "0x0000");
        assert_eq!(fmt_hex(0), "0x0000");
        assert_eq!(fmt_hex(0x400), "0x0400");
        assert_eq!(fmt_hex(0x7ffc), "0x7FFC");
        assert_eq!(fmt_hex(0x12345), "0x12345");
    }

    #[test]
    fn entero_casos_borde_y_clamp() {
        assert_eq!(entero(None, 8, 1, 16), 8);
        assert_eq!(entero(Some(&json!(null)), 8, 1, 16), 8);
        assert_eq!(entero(Some(&json!([])), 8, 1, 16), 8);
        assert_eq!(entero(Some(&json!({})), 8, 1, 16), 8);
        assert_eq!(entero(Some(&json!(true)), 8, 0, 16), 1);
        assert_eq!(entero(Some(&json!(false)), 8, 0, 16), 0);
        assert_eq!(entero(Some(&json!(false)), 8, 2, 16), 2);
        assert_eq!(entero(Some(&json!(3.9)), 8, 1, 16), 3);
        assert_eq!(entero(Some(&json!(-3.9)), 8, -10, 16), -3);
        assert_eq!(entero(Some(&json!(1e308)), 8, 1, 16), 16);
        assert_eq!(entero(Some(&json!(-1e308)), 8, -5, 16), -5);
        assert_eq!(entero(Some(&json!(" 12 ")), 8, 1, 16), 12);
        assert_eq!(entero(Some(&json!("3.5")), 8, 1, 16), 8);
        assert_eq!(entero(Some(&json!("0x10")), 8, 1, 16), 8);
        assert_eq!(
            entero(Some(&json!("99999999999999999999999999")), 8, 1, 16),
            16
        );
        assert_eq!(
            entero(Some(&json!("-99999999999999999999999999")), 8, -5, 16),
            -5
        );

        let mut b = Bloque::new();
        b.insert("capacity".to_string(), json!(100));
        assert_eq!(entero_b(&b, "capacity", 8, 1, 16), 16);
    }

    #[test]
    fn texto_recursivo_y_valores() {
        assert_eq!(texto(&json!(null)), "");
        assert_eq!(texto(&json!(true)), "true");
        assert_eq!(texto(&json!(false)), "false");
        assert_eq!(texto(&json!(42)), "42");
        assert_eq!(texto(&json!([1, null, [true, "a"]])), "[1, , [true, a]]");

        let mut b = Bloque::new();
        assert_eq!(valores(&b).unwrap(), Vec::<String>::new());

        b.insert("values".to_string(), json!([10, null, "x", [1, 2]]));
        assert_eq!(valores(&b).unwrap(), vec!["10", "", "x", "[1, 2]"]);

        b.insert("values".to_string(), json!(false));
        assert_eq!(valores(&b).unwrap(), Vec::<String>::new());

        b.insert("values".to_string(), json!(5));
        assert!(valores(&b).is_err());
    }

    #[test]
    fn lista_de_y_punteros() {
        let mut b = Bloque::new();
        let err = lista_de(&b, "fields", "struct").unwrap_err();
        assert_eq!(
            err.to_string(),
            "'fields' de struct tiene que ser una lista con al menos un elemento"
        );

        b.insert("fields".to_string(), json!([]));
        assert!(lista_de(&b, "fields", "struct").is_err());

        b.insert("fields".to_string(), json!("no_lista"));
        assert!(lista_de(&b, "fields", "struct").is_err());

        b.insert("fields".to_string(), json!(["int x"]));
        assert_eq!(lista_de(&b, "fields", "struct").unwrap().len(), 1);

        b.insert(
            "pointers".to_string(),
            json!("p:0, q:2, r:99, s:-1, bad, t:0"),
        );
        let ptrs = punteros(&b, 4);
        assert_eq!(ptrs.get(&0).unwrap(), &vec!["p".to_string(), "t".to_string()]);
        assert_eq!(ptrs.get(&2).unwrap(), &vec!["q".to_string()]);
        assert!(!ptrs.contains_key(&99));
    }

    #[test]
    fn helpers_bloque_seguros() {
        let mut b = Bloque::new();
        b.insert("zero".to_string(), json!(0));
        b.insert("neg".to_string(), json!(-1));
        b.insert("vacio".to_string(), json!(""));
        b.insert("ok".to_string(), json!("hola"));
        b.insert("nums".to_string(), json!([1, 3, 5]));
        b.insert("tags".to_string(), json!(["A", 2, true]));

        assert_eq!(str_o(&b, "zero", "def"), "def");
        assert_eq!(str_o(&b, "vacio", "def"), "def");
        assert_eq!(str_o(&b, "ok", "def"), "hola");

        assert_eq!(texto_opt(&b, "zero"), Some("0".to_string()));
        assert_eq!(texto_opt(&b, "neg"), Some("-1".to_string()));
        assert_eq!(texto_opt(&b, "ausente"), None);

        let eset = enteros_set(&b, "nums").unwrap();
        assert!(eset.contains(&1) && eset.contains(&3) && eset.contains(&5));
        let sset = strings_set(&b, "tags").unwrap();
        assert!(sset.contains("A") && sset.contains("2") && sset.contains("true"));
    }

    #[test]
    fn cabecera_completa_y_utf8() {
        let cab_val = json!({
            "name": "lista",
            "type": "Lista",
            "fields": [
                "cant = 3",
                "primero -> 0",
                {"name": "último", "value": "?", "to": 0}
            ]
        });
        let campos = campos_cabecera(&cab_val, "header").unwrap();
        assert_eq!(campos.len(), 3);
        assert_eq!(campos[0].nombre, "cant");
        assert_eq!(campos[0].valor, "3");
        assert_eq!(campos[0].destino, None);
        assert_eq!(campos[1].nombre, "primero");
        assert_eq!(campos[1].destino, Some("0".to_string()));
        assert_eq!(campos[2].nombre, "último");
        assert_eq!(campos[2].destino, Some("0".to_string()));

        let cab_obj = cab_val.as_object().unwrap();
        assert_eq!(titulo_cabecera(cab_obj), "lista (Lista)");

        let vals = vec!["3".to_string(), "0x4000".to_string(), "0x4000".to_string()];
        let w = ancho_cabecera(cab_obj, &campos, &vals);
        // "primero" = 7 chars => nombres = 7*7.4 + 16 = 67.8
        // "0x4000" = 6 chars => max(6*8 + 18, 44) = 66
        // titulo "lista (Lista)" = 13 chars => 13*7 + 8 = 99
        assert!((w - (67.8 + 66.0)).abs() < 1e-6);

        let mut c = Lienzo::new(PALETA_TINTA.clone(), "test".to_string());
        let centros = cabecera(&mut c, 0.0, 20.0, w, cab_obj, &campos, &vals).unwrap();
        assert_eq!(centros, vec![33.0, 59.0, 85.0]);
        let unido = c.partes.concat();
        assert!(unido.contains(r##"fill="#ffffff""##)); // fillCell
        assert!(unido.contains(r##"fill="#f1f5f9""##)); // fillPointerBox

        // Errores esperados
        assert_eq!(
            campos_cabecera(&json!({}), "header")
                .unwrap_err()
                .to_string(),
            "'header' necesita 'fields', la lista de sus campos"
        );
        assert!(campos_cabecera(&json!({"fields": []}), "owner").is_err());
        assert!(campos_cabecera(&json!({"fields": ["sin separador"]}), "header").is_err());
    }

    #[test]
    fn zonas_variantes_y_svg() {
        let mut b = Bloque::new();
        assert_eq!(nombres_de_zonas(&b), None);
        b.insert("zones".to_string(), json!(false));
        assert_eq!(nombres_de_zonas(&b), None);
        b.insert("zones".to_string(), json!({}));
        assert_eq!(nombres_de_zonas(&b), None);

        b.insert("zones".to_string(), json!(true));
        assert_eq!(
            nombres_de_zonas(&b),
            Some(("Stack".to_string(), "Heap".to_string()))
        );

        b.insert("zones".to_string(), json!({"stack": "Pila"}));
        assert_eq!(
            nombres_de_zonas(&b),
            Some(("Pila".to_string(), "Heap".to_string()))
        );

        let mut c = Lienzo::new(PALETA_TINTA.clone(), "test".to_string());
        zona(&mut c, 0.0, 100.0, 10.0, 80.0, "Pila");
        let svg = c.partes.concat();
        assert!(svg.contains(r#"rx="4""#));
        assert!(svg.contains(r#"stroke-dasharray="5 3""#));
        assert!(svg.contains(">Pila</text>"));
    }

    #[test]
    fn codigo_tabla_panel_utf8() {
        let mut c = Lienzo::new(PALETA_TINTA.clone(), "test".to_string());
        codigo(&mut c, 10.0, 20.0, "    x = áé", 10.5, "textMain");
        let esperada_x = 10.0 + 4.0 * 10.5 * 0.602;
        assert!(c.partes[0].contains(&format!(r#"x="{esperada_x:.1}""#)));
        assert!(c.partes[0].contains(">x = áé</text>"));

        let mut ct = Lienzo::new(PALETA_TINTA.clone(), "test".to_string());
        let filas = vec![
            vec!["  áéíóú".to_string(), "".to_string()],
            vec!["fin".to_string(), "10".to_string()],
        ];
        let (tw, th) = tabla(
            &mut ct,
            0.0,
            0.0,
            &["Col", "Val"],
            &filas,
            TablaOpts {
                izquierda: &[0],
                tenues: &[1],
                ..TablaOpts::default()
            },
        );
        assert_eq!(th, 26.0 + 2.0 * 28.0);
        // "  áéíóú" son 7 chars: 7 * 10.5 * 0.62 + 18 = 63.57
        assert!((tw - (63.57 + 46.0)).abs() < 1e-6);
        let tsvg = ct.partes.concat();
        assert!(tsvg.contains(r#"stroke-dasharray="3 2""#)); // vacio
        assert!(tsvg.contains(r#"stroke-dasharray="4 3""#)); // tenue

        let lineas = vec!["  if (x)".to_string(), "".to_string()];
        let pw = ancho_panel("TAD", &lineas);
        assert!((pw - (8.0 * 10.5 * 0.62 + 22.0)).abs() < 1e-6);
        let mut cp = Lienzo::new(PALETA_TINTA.clone(), "test".to_string());
        let ph = panel(&mut cp, 0.0, 0.0, pw, "TAD", &lineas, None);
        assert_eq!(ph, 24.0 + 8.0 + 2.0 * 17.0);
        let ph_fijo = panel(&mut cp, 0.0, 0.0, pw, "TAD", &[], Some(90.0));
        assert_eq!(ph_fijo, 90.0);
    }

    #[test]
    fn sub_lienzo_y_propagacion_ancho_min() {
        let mut c = Lienzo::new(PALETA_TINTA.clone(), "test".to_string());
        c.ancho_min = 50.0;
        let mut sub = sub_lienzo(&c);
        sub.ancho_min = 120.0;
        sub.t(0.0, 14.0, "Vista lógica", 10.5, 400, "textMuted", "start", MONO);
        agregar_sub_lienzo(&mut c, 0.0, 100.0, sub);
        assert_eq!(c.ancho_min, 120.0);
        assert!(c.partes[0].starts_with(r#"<g transform="translate(0, 100.0)">"#));
        assert!(c.partes[0].ends_with("</g>"));

        let mut sub2 = sub_lienzo(&c);
        sub2.ancho_min = 90.0;
        agregar_sub_lienzo(&mut c, 50.0, 20.0, sub2);
        assert_eq!(c.ancho_min, 140.0);
        assert!(c.partes[1].starts_with(r#"<g transform="translate(50.0, 20.0)">"#));
    }
}
