//! Renderizadores de recursión y árboles de `dibu-cs` (`call_stack`, `call_tree`, `tree`).

use crate::common::{
    agregar_sub_lienzo, ancho_texto, encabezado, entero, entero_b, es_truthy, fmt_hex,
    fmt_num_clean, hex, hex_b, strings_set, sub_lienzo, texto, valores, Bloque, ErrorDiagrama,
    Lienzo, Punto, MONO, SANS,
};
use dibu_core::Registro;
use serde_json::Value;
use std::cmp::Ordering;
use std::collections::BTreeMap;

const MAX_MARCOS: usize = 10;
const MAX_PASOS: usize = 8;
const MAX_LLAMADAS: usize = 40;
const MAX_NODOS_ARBOL: usize = 512;

fn envolver_tipo(tipo: &str, res: Result<Punto, ErrorDiagrama>) -> Result<Punto, ErrorDiagrama> {
    res.map_err(|e| {
        if e.0
            .strip_prefix(tipo)
            .is_some_and(|rest| rest.starts_with(": "))
        {
            e
        } else {
            ErrorDiagrama::new(format!("{tipo}: {}", e.0))
        }
    })
}

pub fn registrar(reg: &mut Registro) {
    reg.registrar("call_stack", |c, b| {
        envolver_tipo("call_stack", pila_llamadas(c, b))
    });
    reg.registrar("call_tree", |c, b| {
        envolver_tipo("call_tree", arbol_llamadas(c, b))
    });
    reg.registrar("tree", |c, b| envolver_tipo("tree", arbol(c, b)));
}

// ---------------------------------------------------------------------------
// Pila de llamadas (`call_stack`): marcos de activación y su evolución
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
struct Var {
    nombre: String,
    valor: String,
    tipo: String,
    direccion: Option<i64>,
    en_blanco: bool,
    param: bool,
    tamano: Option<i64>,
}

#[derive(Debug, Clone)]
struct Marco {
    funcion: String,
    variables: Vec<Var>,
    retorno: Option<String>,
    terminado: bool,
    linea: Option<String>,
}

type Flecha = (usize, usize, usize, usize);

fn repr_simple(v: &Value) -> String {
    match v {
        Value::String(s) => format!("'{s}'"),
        _ => v.to_string(),
    }
}

fn parsear_variables(
    dato: Option<&Value>,
    param: bool,
    donde: &str,
) -> Result<Vec<Var>, ErrorDiagrama> {
    let Some(val) = dato else {
        return Ok(Vec::new());
    };
    match val {
        Value::Null => Ok(Vec::new()),
        Value::Object(map) => Ok(map
            .iter()
            .map(|(k, v)| Var {
                nombre: k.clone(),
                valor: texto(v),
                tipo: String::new(),
                direccion: None,
                en_blanco: false,
                param,
                tamano: None,
            })
            .collect()),
        Value::Array(arr) => {
            let mut salida = Vec::with_capacity(arr.len());
            for v in arr {
                if let Some(s) = v.as_str() {
                    if let Some((nombre, valor)) = s.split_once('=') {
                        salida.push(Var {
                            nombre: nombre.trim().to_string(),
                            valor: valor.trim().to_string(),
                            tipo: String::new(),
                            direccion: None,
                            en_blanco: false,
                            param,
                            tamano: None,
                        });
                        continue;
                    }
                }
                if let Some(obj) = v.as_object() {
                    if let Some(nombre_val) = obj.get("name") {
                        let direccion = match obj.get("addr") {
                            None | Some(Value::Null) => None,
                            Some(Value::String(s)) if s.is_empty() => None,
                            Some(addr) => Some(hex(Some(addr), 0x1000).max(0)),
                        };
                        let tamano = obj
                            .get("size")
                            .filter(|s| !s.is_null())
                            .map(|s| entero(Some(s), 4, 1, 4096));
                        salida.push(Var {
                            nombre: texto(nombre_val),
                            valor: obj.get("value").map(texto).unwrap_or_default(),
                            tipo: obj
                                .get("type")
                                .filter(|t| es_truthy(t))
                                .map(texto)
                                .unwrap_or_default(),
                            direccion,
                            en_blanco: obj.get("blank").is_some_and(es_truthy),
                            param,
                            tamano,
                        });
                        continue;
                    }
                }
                return Err(ErrorDiagrama::new(format!(
                    "{donde}: variable inválida {} (se espera 'x = 5' o name/value)",
                    repr_simple(v)
                )));
            }
            Ok(salida)
        }
        _ => Err(ErrorDiagrama::new(format!(
            "{donde}: las variables van como diccionario o lista"
        ))),
    }
}

fn parsear_marcos(dato: Option<&Value>, donde: &str) -> Result<Vec<Marco>, ErrorDiagrama> {
    let Some(arr) = dato.and_then(Value::as_array) else {
        return Err(ErrorDiagrama::new(format!(
            "{donde}: 'frames' tiene que ser una lista de marcos (de main hacia el tope)"
        )));
    };
    if arr.is_empty() {
        return Err(ErrorDiagrama::new(format!(
            "{donde}: 'frames' tiene que ser una lista de marcos (de main hacia el tope)"
        )));
    }
    if arr.len() > MAX_MARCOS {
        return Err(ErrorDiagrama::new(format!(
            "{donde}: {} marcos; el máximo es {MAX_MARCOS}",
            arr.len()
        )));
    }
    let mut marcos = Vec::with_capacity(arr.len());
    for (i, f) in arr.iter().enumerate() {
        let Some(obj) = f.as_object() else {
            return Err(ErrorDiagrama::new(format!(
                "{donde}: el marco {i} necesita 'function'"
            )));
        };
        let Some(fn_val) = obj.get("function").filter(|v| es_truthy(v)) else {
            return Err(ErrorDiagrama::new(format!(
                "{donde}: el marco {i} necesita 'function'"
            )));
        };
        let funcion = texto(fn_val);
        let donde_var = format!("{donde}, {funcion}");
        let mut variables = parsear_variables(obj.get("params"), true, &donde_var)?;
        variables.extend(parsear_variables(obj.get("vars"), false, &donde_var)?);
        let retorno = obj.get("returns").filter(|v| !v.is_null()).map(texto);
        let terminado = obj.get("popped").is_some_and(es_truthy);
        let linea = obj.get("returnTo").filter(|v| es_truthy(v)).map(texto);
        marcos.push(Marco {
            funcion,
            variables,
            retorno,
            terminado,
            linea,
        });
    }
    Ok(marcos)
}

fn ubicar_ref(ref_s: &str, marcos: &[Marco], donde: &str) -> Result<(usize, usize), ErrorDiagrama> {
    let trimmed = ref_s.trim();
    let Some((marco, var)) = trimmed.rsplit_once('.') else {
        return Err(ErrorDiagrama::new(format!(
            "{donde}: referencia '{ref_s}' inválida (se espera marco.variable)"
        )));
    };
    if marco.is_empty() || var.is_empty() {
        return Err(ErrorDiagrama::new(format!(
            "{donde}: referencia '{ref_s}' inválida (se espera marco.variable)"
        )));
    }
    let im = if marco.chars().all(|c| c.is_ascii_digit()) {
        marco.parse::<usize>().ok().filter(|&idx| idx < marcos.len())
    } else {
        marcos.iter().rposition(|m| m.funcion == marco)
    };
    let Some(im) = im else {
        return Err(ErrorDiagrama::new(format!(
            "{donde}: no hay marco '{marco}'"
        )));
    };
    let Some(iv) = marcos[im].variables.iter().position(|v| v.nombre == var) else {
        return Err(ErrorDiagrama::new(format!(
            "{donde}: el marco {} no tiene la variable '{var}'",
            marcos[im].funcion
        )));
    };
    Ok((im, iv))
}

fn parsear_flechas(
    dato: Option<&Value>,
    marcos: &[Marco],
    donde: &str,
) -> Result<Vec<Flecha>, ErrorDiagrama> {
    let Some(val) = dato.filter(|v| es_truthy(v)) else {
        return Ok(Vec::new());
    };
    let partes: Vec<String> = match val {
        Value::String(s) => s
            .split(';')
            .filter(|p| !p.trim().is_empty())
            .map(str::to_string)
            .collect(),
        Value::Array(arr) => arr
            .iter()
            .map(texto)
            .filter(|p| !p.trim().is_empty())
            .collect(),
        _ => {
            return Err(ErrorDiagrama::new(format!(
                "{donde}: 'pointers' tiene que ser un texto o una lista de flechas"
            )));
        }
    };
    let mut flechas = Vec::with_capacity(partes.len());
    for p in partes {
        let Some((origen, destino)) = p.split_once("->") else {
            return Err(ErrorDiagrama::new(format!(
                "{donde}: flecha '{p}' inválida (se espera 'marco.var -> marco.var')"
            )));
        };
        let (im, iv) = ubicar_ref(origen, marcos, donde)?;
        let (jm, jv) = ubicar_ref(destino, marcos, donde)?;
        flechas.push((im, iv, jm, jv));
    }
    Ok(flechas)
}

fn rotulo_var(v: &Var, con_tipos: bool) -> String {
    if con_tipos {
        format!("{} {}", v.tipo, v.nombre).trim().to_string()
    } else {
        v.nombre.clone()
    }
}

fn nota_marco(m: &Marco) -> String {
    if let Some(ref ret) = m.retorno {
        format!("devolvió {ret}")
    } else if let Some(ref linea) = m.linea {
        format!("vuelve a {linea}")
    } else {
        String::new()
    }
}

fn titulo_marco(m: &Marco) -> String {
    let params: Vec<&str> = m
        .variables
        .iter()
        .filter(|v| v.param)
        .map(|v| v.nombre.as_str())
        .collect();
    format!("{}({})", m.funcion, params.join(", "))
}

fn pila_llamadas(c: &mut Lienzo, b: &Bloque) -> Result<Punto, ErrorDiagrama> {
    let pasos: Vec<(String, Vec<Marco>, Vec<Flecha>)> =
        if let Some(steps_val) = b.get("steps").filter(|v| !v.is_null()) {
            let Some(pasos_dato) = steps_val.as_array() else {
                return Err(ErrorDiagrama::new(
                    "'steps' tiene que ser una lista de momentos, cada uno con 'frames'",
                ));
            };
            if pasos_dato.is_empty() {
                return Err(ErrorDiagrama::new(
                    "'steps' tiene que ser una lista de momentos, cada uno con 'frames'",
                ));
            }
            if pasos_dato.len() > MAX_PASOS {
                return Err(ErrorDiagrama::new(format!(
                    "{} pasos; el máximo es {MAX_PASOS}",
                    pasos_dato.len()
                )));
            }
            let mut acc = Vec::with_capacity(pasos_dato.len());
            for (k, p) in pasos_dato.iter().enumerate() {
                let Some(p_obj) = p.as_object() else {
                    return Err(ErrorDiagrama::new(format!(
                        "el paso {k} tiene que tener 'frames'"
                    )));
                };
                let donde = format!("paso {k}");
                let marcos = parsear_marcos(p_obj.get("frames"), &donde)?;
                let label = p_obj
                    .get("label")
                    .filter(|v| es_truthy(v))
                    .map(texto)
                    .unwrap_or_else(|| format!("Paso {}", k + 1));
                let flechas = parsear_flechas(p_obj.get("pointers"), &marcos, &donde)?;
                acc.push((label, marcos, flechas));
            }
            acc
        } else {
            let marcos = parsear_marcos(b.get("frames"), "pila de llamadas")?;
            let label = b
                .get("label")
                .filter(|v| es_truthy(v))
                .map(texto)
                .unwrap_or_default();
            let flechas = parsear_flechas(b.get("pointers"), &marcos, "pila de llamadas")?;
            vec![(label, marcos, flechas)]
        };

    let direcciones = b.get("showAddresses").is_some_and(es_truthy);
    let base = hex_b(b, "baseAddr", 0x7FFC);
    let paso_dir = entero_b(b, "typeSize", 4, 1, 64);
    let sin_valores = b.get("blankValues").is_some_and(es_truthy);
    let con_tipos = b.get("showTypes").is_some_and(es_truthy);
    let blank_addrs = b.get("blankAddrs").is_some_and(es_truthy);
    let top = encabezado(c, b, "");

    let max_nombre_chars = pasos
        .iter()
        .flat_map(|(_, ms, _)| ms.iter())
        .flat_map(|m| m.variables.iter())
        .map(|v| rotulo_var(v, con_tipos).chars().count())
        .max()
        .unwrap_or(0)
        .max(4);
    let nombre_w = max_nombre_chars as f64 * 7.4 + 18.0;

    let max_valor_chars = pasos
        .iter()
        .flat_map(|(_, ms, _)| ms.iter())
        .flat_map(|m| m.variables.iter())
        .map(|v| v.valor.chars().count())
        .max()
        .unwrap_or(0)
        .max(1);
    let valor_w = (max_valor_chars as f64 * 8.2 + 22.0).max(58.0);
    let dir_w = if direcciones { 62.0 } else { 0.0 };
    let cab_h = 24.0_f64;
    let fila_h = 26.0_f64;
    let sep = 6.0_f64;

    let mut marco_w = dir_w + nombre_w + valor_w + 8.0;
    for (_, ms, _) in &pasos {
        for m in ms {
            let tw = titulo_marco(m).chars().count() as f64 * 7.6
                + 24.0
                + nota_marco(m).chars().count() as f64 * 6.4;
            marco_w = marco_w.max(tw);
        }
    }

    let max_flechas = pasos.iter().map(|(_, _, f)| f.len()).max().unwrap_or(0);
    let margen_flechas = if max_flechas > 0 {
        16.0 + 10.0 * max_flechas as f64
    } else {
        0.0
    };

    let alto_marco = |m: &Marco| -> f64 { cab_h + m.variables.len().max(1) as f64 * fila_h + 6.0 };

    let alto_pila = pasos
        .iter()
        .map(|(_, ms, _)| ms.iter().map(|m| alto_marco(m) + sep).sum::<f64>())
        .fold(0.0_f64, f64::max);

    let hay_etiquetas = pasos.iter().any(|(et, _, _)| !et.is_empty());
    let y_base = top + if hay_etiquetas { 24.0 } else { 8.0 } + alto_pila;
    let rotulo_w = pasos
        .iter()
        .map(|(et, _, _)| et.chars().count())
        .max()
        .unwrap_or(0) as f64
        * 7.0;
    let col_w = (marco_w + margen_flechas + 40.0).max(rotulo_w + 24.0);
    c.ancho_min = c
        .ancho_min
        .max((pasos.len().saturating_sub(1)) as f64 * col_w + rotulo_w);

    for (col, (etiqueta, marcos, flechas)) in pasos.iter().enumerate() {
        let x0 = col as f64 * col_w;
        if !etiqueta.is_empty() {
            c.t(
                x0,
                top + 14.0,
                etiqueta,
                11.0,
                700,
                "textSub",
                "start",
                SANS,
            );
        }
        let activo = marcos.iter().rposition(|m| !m.terminado);
        let mut filas_y: BTreeMap<(usize, usize), f64> = BTreeMap::new();
        let mut dir_actual: Option<i64> = None;
        let mut y = y_base;
        for (k, m) in marcos.iter().enumerate() {
            let h = alto_marco(m);
            y -= h;
            let es_activo = activo == Some(k);
            let borde = if es_activo { 2.5 } else { 1.5 };
            if m.terminado {
                c.add(format!(
                    r#"<rect x="{x0:.1}" y="{y:.1}" width="{marco_w:.1}" height="{h:.1}" rx="4" fill="none" stroke="{}" stroke-width="1.5" stroke-dasharray="5 3"/>"#,
                    c.pal.stroke_muted
                ));
            } else {
                c.r(x0, y, marco_w, h, "fillCell", borde, 4.0);
            }
            let cab_fill = if es_activo {
                c.pal.fill_highlight
            } else {
                c.pal.fill_header
            };
            c.add(format!(
                r#"<rect x="{:.1}" y="{:.1}" width="{:.1}" height="{}" rx="3" fill="{cab_fill}"/>"#,
                x0 + 1.0,
                y + 1.0,
                marco_w - 2.0,
                fmt_num_clean(cab_h - 1.0)
            ));
            let div_color = if m.terminado {
                "strokeMuted"
            } else {
                "strokeMain"
            };
            c.linea(x0, y + cab_h, x0 + marco_w, y + cab_h, div_color, 1.0, None, "");
            let color_txt = if m.terminado { "textMuted" } else { "textMain" };
            let titulo = titulo_marco(m);
            c.t(x0 + 8.0, y + 16.0, &titulo, 11.5, 700, color_txt, "start", MONO);
            let nota = nota_marco(m);
            if !nota.is_empty() {
                let peso_nota = if m.retorno.is_some() { 700 } else { 400 };
                c.t(
                    x0 + marco_w - 8.0,
                    y + 16.0,
                    &nota,
                    9.5,
                    peso_nota,
                    "textSub",
                    "end",
                    MONO,
                );
            }
            if m.variables.is_empty() {
                c.t(
                    x0 + 10.0,
                    y + cab_h + 18.0,
                    "(sin variables)",
                    9.5,
                    400,
                    "textMuted",
                    "start",
                    MONO,
                );
            }
            for (iv, v) in m.variables.iter().enumerate() {
                let fy = y + cab_h + 3.0 + iv as f64 * fila_h;
                if direcciones {
                    let tam = v
                        .tamano
                        .unwrap_or_else(|| if v.tipo.contains('*') { 8 } else { paso_dir });
                    let siguiente = match dir_actual {
                        None => base,
                        Some(d) => d.saturating_sub(tam),
                    };
                    let val_dir = v.direccion.unwrap_or(siguiente);
                    dir_actual = Some(val_dir);
                    let dir_str = if blank_addrs {
                        "0x____".to_string()
                    } else {
                        fmt_hex(val_dir)
                    };
                    c.t(x0 + 8.0, fy + 16.0, &dir_str, 8.5, 400, "textSub", "start", MONO);
                }
                let rot = rotulo_var(v, con_tipos);
                let peso_var = if v.param { 600 } else { 400 };
                c.t(
                    x0 + dir_w + 10.0,
                    fy + 16.0,
                    &rot,
                    11.0,
                    peso_var,
                    color_txt,
                    "start",
                    MONO,
                );
                let vx = x0 + dir_w + nombre_w;
                c.r(vx, fy + 2.0, valor_w - 8.0, fila_h - 6.0, "fillCell", 1.2, 2.0);
                if !v.valor.is_empty() && !(sin_valores || v.en_blanco) {
                    c.t(
                        vx + (valor_w - 8.0) / 2.0,
                        fy + 17.0,
                        &v.valor,
                        12.0,
                        700,
                        color_txt,
                        "middle",
                        MONO,
                    );
                } else {
                    c.vacio(vx + 6.0, vx + valor_w - 14.0, fy + fila_h - 7.0);
                }
                filas_y.insert((k, iv), fy + fila_h / 2.0 - 1.0);
            }
            y -= sep;
        }
        c.linea(
            x0 - 6.0,
            y_base + 2.0,
            x0 + marco_w + 6.0,
            y_base + 2.0,
            "strokeMain",
            2.5,
            None,
            "",
        );
        let borde_valor = x0 + dir_w + nombre_w + valor_w - 8.0;
        for (n, &(im, iv, jm, jv)) in flechas.iter().enumerate() {
            let (Some(&y1), Some(&y2)) = (filas_y.get(&(im, iv)), filas_y.get(&(jm, jv))) else {
                continue;
            };
            let xm = x0 + marco_w + 12.0 + n as f64 * 10.0;
            c.punto(borde_valor - 8.0, y1, "accentPrimary", 2.8);
            c.ruta(&[
                (borde_valor - 8.0, y1),
                (xm, y1),
                (xm, y2),
                (borde_valor + 3.0, y2),
            ]);
        }
    }
    c.t(
        0.0,
        y_base + 18.0,
        "base de la pila",
        9.0,
        400,
        "textMuted",
        "start",
        MONO,
    );
    Ok((pasos.len() as f64 * col_w - 40.0 + 4.0, y_base + 24.0))
}

// ---------------------------------------------------------------------------
// Árbol de llamadas (`call_tree`)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
struct Llamada {
    texto: String,
    retorno: String,
    hijos: Vec<Llamada>,
    x: f64,
    prof: usize,
    orden: usize,
    ancho_arbol: f64,
    en_blanco: bool,
}

fn parsear_llamada(
    dato: &Value,
    prof: usize,
    contador: &mut usize,
) -> Result<Llamada, ErrorDiagrama> {
    *contador += 1;
    if *contador > MAX_LLAMADAS {
        return Err(ErrorDiagrama::new(format!(
            "el árbol tiene más de {MAX_LLAMADAS} llamadas; el máximo es {MAX_LLAMADAS}"
        )));
    }
    if let Some(obj) = dato.as_object() {
        let Some(call_val) = obj.get("call").filter(|v| es_truthy(v)) else {
            return Err(ErrorDiagrama::new(
                "cada nodo de un árbol de llamadas tiene que tener 'call'",
            ));
        };
        let hijos_slice: &[Value] = match obj.get("children") {
            None => &[],
            Some(v) if !es_truthy(v) => &[],
            Some(Value::Array(arr)) => arr.as_slice(),
            Some(_) => {
                return Err(ErrorDiagrama::new(
                    "'children' de un árbol de llamadas tiene que ser una lista",
                ));
            }
        };
        let mut hijos = Vec::with_capacity(hijos_slice.len().min(MAX_LLAMADAS));
        for h in hijos_slice {
            hijos.push(parsear_llamada(h, prof + 1, contador)?);
        }
        Ok(Llamada {
            texto: texto(call_val),
            retorno: obj.get("returns").map(texto).unwrap_or_default(),
            hijos,
            x: 0.0,
            prof,
            orden: 0,
            ancho_arbol: 0.0,
            en_blanco: obj.get("blank").is_some_and(es_truthy),
        })
    } else {
        Ok(Llamada {
            texto: texto(dato),
            retorno: String::new(),
            hijos: Vec::new(),
            x: 0.0,
            prof,
            orden: 0,
            ancho_arbol: 0.0,
            en_blanco: false,
        })
    }
}

fn numerar_preorden(
    n: &mut Llamada,
    contador: &mut usize,
    prof_max: &mut usize,
    hay_retorno: &mut bool,
) {
    *contador += 1;
    n.orden = *contador;
    *prof_max = (*prof_max).max(n.prof);
    if !n.retorno.is_empty() {
        *hay_retorno = true;
    }
    for h in &mut n.hijos {
        numerar_preorden(h, contador, prof_max, hay_retorno);
    }
}

fn ancho_nodo_llamada(n: &Llamada) -> f64 {
    let ret = format!("→ {}", n.retorno);
    58.0_f64
        .max(ancho_texto(&n.texto, 11.0) + 18.0)
        .max(ancho_texto(&ret, 10.0) + 16.0)
}

fn medir_llamada(n: &mut Llamada, sep: f64) -> f64 {
    let mut suma_hijos = 0.0;
    for h in &mut n.hijos {
        suma_hijos += medir_llamada(h, sep);
    }
    let sep_total = sep * n.hijos.len().saturating_sub(1) as f64;
    n.ancho_arbol = ancho_nodo_llamada(n).max(suma_hijos + sep_total);
    n.ancho_arbol
}

fn ubicar_llamada(n: &mut Llamada, x: f64, sep: f64) {
    let suma_hijos: f64 = n.hijos.iter().map(|h| h.ancho_arbol).sum();
    let hijos_w = suma_hijos + sep * n.hijos.len().saturating_sub(1) as f64;
    let mut cursor = x + (n.ancho_arbol - hijos_w) / 2.0;
    for h in &mut n.hijos {
        ubicar_llamada(h, cursor, sep);
        cursor += h.ancho_arbol + sep;
    }
    n.x = match (n.hijos.first(), n.hijos.last()) {
        (Some(prim), Some(ult)) => (prim.x + ult.x) / 2.0,
        _ => x + n.ancho_arbol / 2.0,
    };
}

fn dibujar_enlaces_llamada(c: &mut Lienzo, n: &Llamada, y0: f64, nivel_h: f64, alto_nodo: f64) {
    let y_n = y0 + n.prof as f64 * nivel_h;
    for h in &n.hijos {
        let y_h = y0 + h.prof as f64 * nivel_h;
        c.linea(
            n.x,
            y_n + alto_nodo,
            h.x,
            y_h - 3.0,
            "accentPrimary",
            1.5,
            Some("main"),
            "",
        );
    }
    for h in &n.hijos {
        dibujar_enlaces_llamada(c, h, y0, nivel_h, alto_nodo);
    }
}

#[allow(clippy::too_many_arguments)]
fn dibujar_nodos_llamada(
    c: &mut Lienzo,
    n: &Llamada,
    y0: f64,
    nivel_h: f64,
    alto_nodo: f64,
    sin_llamadas: bool,
    sin_retornos: bool,
    con_retorno: bool,
    con_orden: bool,
    resaltados: &std::collections::HashSet<String>,
) {
    let w = ancho_nodo_llamada(n);
    let y = y0 + n.prof as f64 * nivel_h;
    let marcada = resaltados.contains(&n.texto);
    let relleno = if marcada { "fillHighlight" } else { "fillCell" };
    let trazo = if marcada { 2.4 } else { 1.6 };
    c.r(n.x - w / 2.0, y, w, alto_nodo, relleno, trazo, 4.0);
    if sin_llamadas || n.en_blanco {
        c.vacio(n.x - w / 2.0 + 8.0, n.x + w / 2.0 - 8.0, y + 20.0);
    } else {
        c.t(n.x, y + 18.0, &n.texto, 11.0, 700, "textMain", "middle", MONO);
    }
    if con_retorno {
        if sin_retornos || n.retorno.is_empty() {
            c.t(
                n.x - w / 2.0 + 8.0,
                y + 35.0,
                "→",
                10.0,
                400,
                "textSub",
                "start",
                MONO,
            );
            c.vacio(n.x - w / 2.0 + 22.0, n.x + w / 2.0 - 8.0, y + 37.0);
        } else {
            c.t(
                n.x,
                y + 35.0,
                &format!("→ {}", n.retorno),
                10.0,
                400,
                "textSub",
                "middle",
                MONO,
            );
        }
    }
    if con_orden {
        c.add(format!(
            r#"<circle cx="{:.1}" cy="{y:.1}" r="9" fill="{}" stroke="{}" stroke-width="1.1"/>"#,
            n.x - w / 2.0,
            c.pal.fill_badge,
            c.pal.stroke_main
        ));
        c.t(
            n.x - w / 2.0,
            y + 3.5,
            &n.orden.to_string(),
            9.0,
            700,
            "textMain",
            "middle",
            MONO,
        );
    }
    for h in &n.hijos {
        dibujar_nodos_llamada(
            c,
            h,
            y0,
            nivel_h,
            alto_nodo,
            sin_llamadas,
            sin_retornos,
            con_retorno,
            con_orden,
            resaltados,
        );
    }
}

fn arbol_llamadas(c: &mut Lienzo, b: &Bloque) -> Result<Punto, ErrorDiagrama> {
    let Some(root_val) = b.get("root").filter(|v| es_truthy(v)) else {
        return Err(ErrorDiagrama::new(
            "el árbol de llamadas no tiene 'root' ({call, returns, children})",
        ));
    };
    let mut parseados = 0usize;
    let mut raiz = parsear_llamada(root_val, 0, &mut parseados)?;
    let mut total_nodos = 0usize;
    let mut profundidad = 0usize;
    let mut hay_retorno = false;
    numerar_preorden(&mut raiz, &mut total_nodos, &mut profundidad, &mut hay_retorno);
    if total_nodos > MAX_LLAMADAS {
        return Err(ErrorDiagrama::new(format!(
            "el árbol tiene {total_nodos} llamadas; el máximo es {MAX_LLAMADAS}"
        )));
    }

    let sin_retornos = b.get("blankReturns").is_some_and(es_truthy);
    let sin_llamadas = b.get("blankCalls").is_some_and(es_truthy);
    let con_orden = b.get("showOrder").is_some_and(es_truthy);
    let resaltados = strings_set(b, "highlight")?;
    let con_retorno = sin_retornos || hay_retorno;
    let top = encabezado(c, b, "");
    let alto_nodo = if con_retorno { 42.0 } else { 28.0 };
    let nivel_h = 76.0_f64;
    let sep = 14.0_f64;

    medir_llamada(&mut raiz, sep);
    ubicar_llamada(&mut raiz, if con_orden { 10.0 } else { 0.0 }, sep);
    let y0 = top + 12.0;

    dibujar_enlaces_llamada(c, &raiz, y0, nivel_h, alto_nodo);
    dibujar_nodos_llamada(
        c,
        &raiz,
        y0,
        nivel_h,
        alto_nodo,
        sin_llamadas,
        sin_retornos,
        con_retorno,
        con_orden,
        &resaltados,
    );

    let mut ancho = raiz.ancho_arbol + if con_orden { 10.0 } else { 0.0 };
    let mut alto = y0 + profundidad as f64 * nivel_h + alto_nodo + 8.0;

    let blank_level_costs = b.get("blankLevelCosts").is_some_and(es_truthy);
    let niveles_opt = b.get("levelCosts").filter(|v| !v.is_null());
    if niveles_opt.is_some() || blank_level_costs {
        let datos: &[Value] = match niveles_opt {
            None => &[],
            Some(v) if !es_truthy(v) => &[],
            Some(Value::Array(arr)) => arr.as_slice(),
            Some(_) => {
                return Err(ErrorDiagrama::new("'levelCosts' tiene que ser una lista"));
            }
        };
        let x = ancho + 34.0;
        let mut textos = Vec::with_capacity(profundidad + 1);
        for i in 0..=profundidad {
            if blank_level_costs {
                textos.push(String::new());
            } else {
                textos.push(datos.get(i).map(texto).unwrap_or_default());
            }
        }
        let mut col_w = ancho_texto("costo del nivel", 9.5).max(90.0);
        for t in &textos {
            col_w = col_w.max(ancho_texto(t, 10.5));
        }
        c.t(
            x,
            y0 - 8.0,
            "costo del nivel",
            9.5,
            700,
            "textSub",
            "start",
            MONO,
        );
        for (i, t) in textos.iter().enumerate() {
            let y = y0 + i as f64 * nivel_h + alto_nodo / 2.0;
            c.linea(x - 28.0, y, x - 6.0, y, "strokeMuted", 1.0, None, "3 2");
            if !t.is_empty() {
                c.t(x, y + 4.0, t, 10.5, 700, "textMain", "start", MONO);
            } else {
                c.vacio(x, x + col_w, y + 6.0);
            }
        }
        ancho = x + col_w;
    }

    let blank_total = b.get("blankTotal").is_some_and(es_truthy);
    let total_opt = b.get("total").filter(|v| !v.is_null());
    if total_opt.is_some() || blank_total {
        let txt = if blank_total {
            format!("T(n) = {}", "_".repeat(24))
        } else {
            format!("T(n) = {}", total_opt.map(texto).unwrap_or_default())
        };
        c.t(0.0, alto + 12.0, &txt, 12.0, 700, "textMain", "start", MONO);
        ancho = ancho.max(ancho_texto(&txt, 12.0));
        alto += 20.0;
    }

    Ok((ancho, alto))
}

// ---------------------------------------------------------------------------
// Árbol binario / ABB / AVL / Montículo (`tree`)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Nodo {
    pub valor: String,
    pub izq: Option<Box<Nodo>>,
    pub der: Option<Box<Nodo>>,
    pub indice: Option<usize>,
    pub altura: usize,
}

impl Nodo {
    pub fn altura(&self) -> usize {
        self.altura
    }

    fn actualizar_altura(&mut self) {
        self.altura = 1 + self
            .izq
            .as_ref()
            .map_or(0, |n| n.altura)
            .max(self.der.as_ref().map_or(0, |n| n.altura));
    }

    pub fn balance(&self) -> i32 {
        let izq_h = self.izq.as_ref().map_or(0, |n| n.altura as i32);
        let der_h = self.der.as_ref().map_or(0, |n| n.altura as i32);
        izq_h - der_h
    }
}

enum Clave<'a> {
    Num(f64),
    Txt(&'a str),
}

fn clave(valor: &str) -> Clave<'_> {
    if let Ok(n) = valor.trim().parse::<f64>() {
        if n.is_finite() {
            return Clave::Num(if n == 0.0 { 0.0 } else { n });
        }
    }
    Clave::Txt(valor)
}

fn cmp_clave(a: &str, b: &str) -> Ordering {
    match (clave(a), clave(b)) {
        (Clave::Num(x), Clave::Num(y)) => x.total_cmp(&y),
        (Clave::Num(_), Clave::Txt(_)) => Ordering::Less,
        (Clave::Txt(_), Clave::Num(_)) => Ordering::Greater,
        (Clave::Txt(x), Clave::Txt(y)) => x.cmp(y),
    }
}

fn rotar_der(mut n: Box<Nodo>) -> Box<Nodo> {
    let Some(mut raiz) = n.izq.take() else {
        return n;
    };
    n.izq = raiz.der.take();
    n.actualizar_altura();
    raiz.der = Some(n);
    raiz.actualizar_altura();
    raiz
}

fn rotar_izq(mut n: Box<Nodo>) -> Box<Nodo> {
    let Some(mut raiz) = n.der.take() else {
        return n;
    };
    n.der = raiz.izq.take();
    n.actualizar_altura();
    raiz.izq = Some(n);
    raiz.actualizar_altura();
    raiz
}

fn insertar(raiz: Option<Box<Nodo>>, valor: String, avl: bool) -> Box<Nodo> {
    let Some(mut nodo) = raiz else {
        return Box::new(Nodo {
            valor,
            izq: None,
            der: None,
            indice: None,
            altura: 1,
        });
    };
    match cmp_clave(&valor, &nodo.valor) {
        Ordering::Equal => return nodo,
        Ordering::Less => nodo.izq = Some(insertar(nodo.izq.take(), valor, avl)),
        Ordering::Greater => nodo.der = Some(insertar(nodo.der.take(), valor, avl)),
    }
    nodo.actualizar_altura();
    if !avl {
        return nodo;
    }
    let bal = nodo.balance();
    if bal > 1 {
        if nodo.izq.as_ref().is_some_and(|izq| izq.balance() < 0) {
            if let Some(izq) = nodo.izq.take() {
                nodo.izq = Some(rotar_izq(izq));
            }
        }
        return rotar_der(nodo);
    }
    if bal < -1 {
        if nodo.der.as_ref().is_some_and(|der| der.balance() > 0) {
            if let Some(der) = nodo.der.take() {
                nodo.der = Some(rotar_der(der));
            }
        }
        return rotar_izq(nodo);
    }
    nodo
}

fn desde_niveles(
    vals: &[String],
    i: usize,
    contador: &mut usize,
) -> Result<Option<Box<Nodo>>, ErrorDiagrama> {
    let Some(v) = vals.get(i) else {
        return Ok(None);
    };
    if matches!(v.as_str(), "" | "null" | "None" | "~") {
        return Ok(None);
    }
    *contador += 1;
    if *contador > MAX_NODOS_ARBOL {
        return Err(ErrorDiagrama::new(format!(
            "el árbol tiene más de {MAX_NODOS_ARBOL} nodos; el máximo es {MAX_NODOS_ARBOL}"
        )));
    }
    let izq_idx = i.checked_mul(2).and_then(|x| x.checked_add(1));
    let der_idx = i.checked_mul(2).and_then(|x| x.checked_add(2));
    let izq = match izq_idx {
        Some(idx) => desde_niveles(vals, idx, contador)?,
        None => None,
    };
    let der = match der_idx {
        Some(idx) => desde_niveles(vals, idx, contador)?,
        None => None,
    };
    let altura = 1 + izq
        .as_ref()
        .map_or(0, |n| n.altura)
        .max(der.as_ref().map_or(0, |n| n.altura));
    Ok(Some(Box::new(Nodo {
        valor: v.clone(),
        izq,
        der,
        indice: Some(i),
        altura,
    })))
}

fn desde_anidado(
    dato: &Value,
    contador: &mut usize,
) -> Result<Option<Box<Nodo>>, ErrorDiagrama> {
    match dato {
        Value::Null => Ok(None),
        Value::Object(obj) => {
            *contador += 1;
            if *contador > MAX_NODOS_ARBOL {
                return Err(ErrorDiagrama::new(format!(
                    "el árbol tiene más de {MAX_NODOS_ARBOL} nodos; el máximo es {MAX_NODOS_ARBOL}"
                )));
            }
            let izq = match obj.get("left") {
                Some(v) => desde_anidado(v, contador)?,
                None => None,
            };
            let der = match obj.get("right") {
                Some(v) => desde_anidado(v, contador)?,
                None => None,
            };
            let altura = 1 + izq
                .as_ref()
                .map_or(0, |n| n.altura)
                .max(der.as_ref().map_or(0, |n| n.altura));
            Ok(Some(Box::new(Nodo {
                valor: obj.get("value").map(texto).unwrap_or_default(),
                izq,
                der,
                indice: None,
                altura,
            })))
        }
        other => {
            *contador += 1;
            if *contador > MAX_NODOS_ARBOL {
                return Err(ErrorDiagrama::new(format!(
                    "el árbol tiene más de {MAX_NODOS_ARBOL} nodos; el máximo es {MAX_NODOS_ARBOL}"
                )));
            }
            Ok(Some(Box::new(Nodo {
                valor: texto(other),
                izq: None,
                der: None,
                indice: None,
                altura: 1,
            })))
        }
    }
}

pub fn construir_arbol(b: &Bloque) -> Result<Option<Box<Nodo>>, ErrorDiagrama> {
    let variante = b
        .get("variant")
        .filter(|v| !v.is_null())
        .map(texto)
        .unwrap_or_else(|| "binary".to_string());
    if let Some(ins) = b.get("insert").filter(|v| !v.is_null()) {
        let Some(arr) = ins.as_array() else {
            return Err(ErrorDiagrama::new("'insert' tiene que ser una lista"));
        };
        if arr.len() > MAX_NODOS_ARBOL {
            return Err(ErrorDiagrama::new(format!(
                "el árbol tiene {} nodos en 'insert'; el máximo es {MAX_NODOS_ARBOL}",
                arr.len()
            )));
        }
        let es_avl = variante == "avl";
        let mut raiz: Option<Box<Nodo>> = None;
        for v in arr {
            raiz = Some(insertar(raiz, texto(v), es_avl));
        }
        return Ok(raiz);
    }
    let mut contador = 0usize;
    if let Some(root_val) = b.get("root").filter(|v| !v.is_null()) {
        return desde_anidado(root_val, &mut contador);
    }
    let vals = valores(b)?;
    if vals.len() > MAX_NODOS_ARBOL * 4 {
        return Err(ErrorDiagrama::new(format!(
            "el árbol tiene {} posiciones en 'values'; el máximo es {}",
            vals.len(),
            MAX_NODOS_ARBOL * 4
        )));
    }
    desde_niveles(&vals, 0, &mut contador)
}

struct NodoPos {
    valor: String,
    balance: i32,
    indice: Option<usize>,
    x: f64,
    y: f64,
    izq: Option<usize>,
    der: Option<usize>,
}

fn recolectar_inorden(
    nodo: &Nodo,
    prof: usize,
    paso_x: f64,
    nivel_h: f64,
    margen: f64,
    top: f64,
    out: &mut Vec<NodoPos>,
) -> usize {
    let izq_idx = nodo
        .izq
        .as_deref()
        .map(|h| recolectar_inorden(h, prof + 1, paso_x, nivel_h, margen, top, out));
    let mi_idx = out.len();
    let x = margen + mi_idx as f64 * paso_x;
    let y = top + 34.0 + prof as f64 * nivel_h;
    out.push(NodoPos {
        valor: nodo.valor.clone(),
        balance: nodo.balance(),
        indice: nodo.indice,
        x,
        y,
        izq: izq_idx,
        der: None,
    });
    let der_idx = nodo
        .der
        .as_deref()
        .map(|h| recolectar_inorden(h, prof + 1, paso_x, nivel_h, margen, top, out));
    if let Some(slot) = out.get_mut(mi_idx) {
        slot.der = der_idx;
    }
    mi_idx
}

fn dibujar_arreglo_heap(sub: &mut Lienzo, vals: &[String], sin_valores: bool) -> Punto {
    let cap = vals.len().clamp(1, 16);
    let x0 = 78.0_f64;
    let celda_w = 64.0_f64;
    let celda_h = 54.0_f64;
    let y = 30.0_f64;

    sub.t(
        x0 - 10.0,
        y - 8.0,
        "Índice [i]:",
        10.0,
        600,
        "textSub",
        "end",
        MONO,
    );
    sub.t(
        x0 - 10.0,
        y + celda_h / 2.0 + 4.0,
        "Valor:",
        10.5,
        700,
        "textMain",
        "end",
        MONO,
    );

    for i in 0..cap {
        let cx = x0 + i as f64 * celda_w;
        let centro = cx + celda_w / 2.0;
        sub.t(
            centro,
            y - 7.0,
            &format!("[{i}]"),
            11.0,
            700,
            "textSub",
            "middle",
            MONO,
        );
        sub.r(cx, y, celda_w, celda_h, "fillCell", 1.75, 0.0);
        let valor = vals.get(i).map(String::as_str).unwrap_or("");
        if !valor.is_empty() && !sin_valores {
            sub.t(
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
            sub.vacio(cx + 10.0, cx + celda_w - 10.0, y + celda_h - 12.0);
        }
    }

    (x0 + cap as f64 * celda_w + 20.0, y + celda_h + 12.0)
}

fn arbol(c: &mut Lienzo, b: &Bloque) -> Result<Punto, ErrorDiagrama> {
    let Some(raiz) = construir_arbol(b)? else {
        return Err(ErrorDiagrama::new(
            "el árbol no tiene nodos (usar values, insert o root)",
        ));
    };
    let variante = b
        .get("variant")
        .filter(|v| !v.is_null())
        .map(texto)
        .unwrap_or_else(|| "binary".to_string());
    let estructura = b.get("nodeStyle").and_then(Value::as_str) == Some("struct");
    let sin_valores = b.get("blankValues").is_some_and(es_truthy);
    let nulls = b.get("nullLeaves").map_or(estructura, es_truthy);
    let resaltados = strings_set(b, "highlight")?;
    let top = encabezado(c, b, "");

    let paso_x = if estructura {
        112.0
    } else if variante == "avl" || variante == "heap" {
        58.0
    } else {
        50.0
    };
    let nivel_h = if estructura { 78.0 } else { 66.0 };
    let margen = 30.0 + if nulls { 24.0 } else { 0.0 };
    let radio = 19.0_f64;
    let nodo_w = 100.0_f64;
    let nodo_h = 36.0_f64;

    let mut nodos = Vec::new();
    recolectar_inorden(&raiz, 0, paso_x, nivel_h, margen, top, &mut nodos);

    let mut puntos: Vec<Punto> = Vec::new();
    for n in &nodos {
        let (x, y) = (n.x, n.y);
        for (signo, hijo_opt) in [(-1.0_f64, n.izq), (1.0_f64, n.der)] {
            if estructura {
                let Some(h_idx) = hijo_opt.and_then(|idx| nodos.get(idx)) else {
                    continue;
                };
                let sx = x + signo * (nodo_w / 2.0 - 13.0);
                let sy = y + 6.0;
                puntos.push((sx, sy));
                c.linea(
                    sx,
                    sy,
                    h_idx.x,
                    h_idx.y - nodo_h / 2.0 - 2.0,
                    "accentPrimary",
                    1.5,
                    Some("main"),
                    "",
                );
            } else if let Some(h_idx) = hijo_opt.and_then(|idx| nodos.get(idx)) {
                let largo = (h_idx.x - x).hypot(h_idx.y - y);
                if largo > 0.0 {
                    let ux = (h_idx.x - x) / largo;
                    let uy = (h_idx.y - y) / largo;
                    c.linea(
                        x + ux * radio,
                        y + uy * radio,
                        h_idx.x - ux * radio,
                        h_idx.y - uy * radio,
                        "strokeMain",
                        1.5,
                        None,
                        "",
                    );
                }
            } else if nulls {
                let sx = x + signo * radio * 0.7;
                let sy = y + radio * 0.7;
                c.linea(
                    sx,
                    sy,
                    sx + signo * 16.0,
                    sy + 22.0,
                    "strokeMuted",
                    1.0,
                    None,
                    "3 2",
                );
                c.t(
                    sx + signo * 16.0,
                    sy + 32.0,
                    "NULL",
                    7.5,
                    400,
                    "textMuted",
                    "middle",
                    MONO,
                );
            }
        }
    }

    let mostrar_balance = b.get("showBalance").is_none_or(es_truthy);
    let balance_blanco = b.get("blankBalance").is_some_and(es_truthy);

    for n in &nodos {
        let (x, y) = (n.x, n.y);
        let resaltado = resaltados.contains(&n.valor);
        let relleno = if resaltado { "fillHighlight" } else { "fillCell" };
        let trazo = if resaltado { 2.6 } else { 1.6 };

        if estructura {
            let izq_x = x - nodo_w / 2.0;
            let cel = 26.0_f64;
            c.r(izq_x, y - nodo_h / 2.0, nodo_w, nodo_h, relleno, trazo, 3.0);
            c.r(
                izq_x,
                y - nodo_h / 2.0,
                cel,
                nodo_h,
                "fillPointerBox",
                1.5,
                0.0,
            );
            c.r(
                izq_x + nodo_w - cel,
                y - nodo_h / 2.0,
                cel,
                nodo_h,
                "fillPointerBox",
                1.5,
                0.0,
            );
            c.t(
                izq_x + cel / 2.0,
                y - nodo_h / 2.0 - 4.0,
                "*izq",
                7.5,
                400,
                "textMuted",
                "middle",
                MONO,
            );
            c.t(
                izq_x + nodo_w - cel / 2.0,
                y - nodo_h / 2.0 - 4.0,
                "*der",
                7.5,
                400,
                "textMuted",
                "middle",
                MONO,
            );
            for (lado_izq, hijo_opt) in [(true, n.izq), (false, n.der)] {
                if hijo_opt.is_none() {
                    let cx0 = if lado_izq { izq_x } else { izq_x + nodo_w - cel };
                    c.linea(
                        cx0 + 4.0,
                        y + nodo_h / 2.0 - 4.0,
                        cx0 + cel - 4.0,
                        y - nodo_h / 2.0 + 4.0,
                        "strokeMain",
                        1.3,
                        None,
                        "",
                    );
                }
            }
            if !n.valor.is_empty() && !sin_valores {
                c.t(x, y + 5.0, &n.valor, 13.0, 700, "textMain", "middle", MONO);
            } else {
                c.vacio(x - 18.0, x + 18.0, y + 10.0);
            }
        } else {
            c.add(format!(
                r#"<circle cx="{x:.1}" cy="{y:.1}" r="{}" fill="{}" stroke="{}" stroke-width="{}"/>"#,
                fmt_num_clean(radio),
                c.pal.color(relleno),
                c.pal.stroke_main,
                fmt_num_clean(trazo)
            ));
            if !n.valor.is_empty() && !sin_valores {
                let size = if n.valor.chars().count() < 4 {
                    12.5
                } else {
                    9.5
                };
                c.t(x, y + 4.5, &n.valor, size, 700, "textMain", "middle", MONO);
            } else {
                c.vacio(x - 9.0, x + 9.0, y + 7.0);
            }
        }

        if variante == "avl" && mostrar_balance {
            c.r(
                x + radio - 2.0,
                y - radio - 8.0,
                22.0,
                14.0,
                "fillBadge",
                1.0,
                3.0,
            );
            let txt = if balance_blanco {
                "__".to_string()
            } else if n.balance == 0 {
                "0".to_string()
            } else {
                format!("{:+}", n.balance)
            };
            c.t(
                x + radio + 9.0,
                y - radio + 2.5,
                &txt,
                8.5,
                700,
                "textSub",
                "middle",
                MONO,
            );
        }

        if variante == "heap" {
            if let Some(idx) = n.indice {
                c.t(
                    x,
                    y + radio + 13.0,
                    &format!("[{idx}]"),
                    8.5,
                    400,
                    "textMuted",
                    "middle",
                    MONO,
                );
            }
        }
    }

    for (px, py) in puntos {
        c.punto(px, py, "accentPrimary", 2.8);
    }

    let ancho = margen * 2.0
        + nodos.len().saturating_sub(1) as f64 * paso_x
        + if variante == "avl" { 24.0 } else { 0.0 };
    let alto = top
        + 34.0
        + (raiz.altura().saturating_sub(1)) as f64 * nivel_h
        + if estructura { nodo_h } else { radio }
        + if nulls { 34.0 } else { 22.0 };

    if variante == "heap"
        && b.get("showArray").is_none_or(es_truthy)
        && b.get("values").is_some_and(es_truthy)
    {
        let vals = valores(b)?;
        let mut sub = sub_lienzo(c);
        let (ancho_a, alto_a) = dibujar_arreglo_heap(&mut sub, &vals, sin_valores);
        agregar_sub_lienzo(c, 0.0, alto + 8.0, sub);
        return Ok((ancho.max(ancho_a), alto + 8.0 + alto_a));
    }

    Ok((ancho, alto))
}

// ---------------------------------------------------------------------------
// Tests unitarios
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use dibu_core::leer_bloques;
    use serde_json::json;

    fn reg_test() -> Registro {
        let mut r = Registro::new();
        registrar(&mut r);
        r
    }

    fn bloque_de(v: Value) -> Bloque {
        v.as_object().cloned().unwrap_or_default()
    }

    fn render_val(v: Value, tinta: bool) -> Result<String, ErrorDiagrama> {
        let reg = reg_test();
        reg.renderizar(&bloque_de(v), tinta)
    }

    fn extraer_textos(svg: &str) -> Vec<String> {
        let mut out = Vec::new();
        let mut rest = svg;
        while let Some(pos) = rest.find("<text ") {
            let after_tag = &rest[pos..];
            if let Some(gt) = after_tag.find('>') {
                let inner = &after_tag[gt + 1..];
                if let Some(end) = inner.find("</text>") {
                    out.push(inner[..end].to_string());
                    rest = &inner[end + 7..];
                    continue;
                }
            }
            break;
        }
        out
    }

    fn extraer_attrs(svg: &str, attr: &str) -> Option<f64> {
        let pat = format!(r#"{attr}=""#);
        let idx = svg.find(&pat)?;
        let resto = &svg[idx + pat.len()..];
        let fin = resto.find('"')?;
        resto[..fin].parse::<f64>().ok()
    }

    fn en_orden(nodo: &Option<Box<Nodo>>) -> Vec<String> {
        match nodo {
            None => Vec::new(),
            Some(n) => {
                let mut v = en_orden(&n.izq);
                v.push(n.valor.clone());
                v.extend(en_orden(&n.der));
                v
            }
        }
    }

    fn pila_llamadas_fixture() -> Value {
        json!({
            "type": "call_stack",
            "showAddresses": true,
            "showTypes": true,
            "frames": [
                {
                    "function": "main",
                    "vars": [
                        {"name": "a", "value": 3, "type": "int"},
                        {"name": "b", "value": 7, "type": "int"}
                    ]
                },
                {
                    "function": "intercambiar",
                    "returnTo": "main",
                    "params": [
                        {"name": "x", "value": "&a", "type": "int *"},
                        {"name": "y", "value": "&b", "type": "int *"}
                    ],
                    "vars": ["aux = 3"]
                }
            ],
            "pointers": "1.x -> 0.a; intercambiar.y -> main.b"
        })
    }

    fn fib3_fixture() -> Value {
        json!({
            "call": "fib(3)",
            "returns": 2,
            "children": [
                {
                    "call": "fib(2)",
                    "returns": 1,
                    "children": [
                        {"call": "fib(1)", "returns": 1},
                        {"call": "fib(0)", "returns": 0}
                    ]
                },
                {"call": "fib(1)", "returns": 1}
            ]
        })
    }

    // --- call_stack ---

    #[test]
    fn pila_de_llamadas_marcos_direcciones_y_punteros() {
        let svg = render_val(pila_llamadas_fixture(), true).unwrap();
        assert!(svg.starts_with("<svg ") && svg.ends_with("</svg>"));
        assert!(svg.contains(">intercambiar(x, y)<"));
        assert!(svg.contains(">main()<"));
        assert!(svg.contains(">vuelve a main<"));
        assert!(svg.contains(">int * x<"));
        assert!(svg.contains(">&amp;a<"));
        assert!(svg.contains(">3<"));
        let addrs: Vec<String> = extraer_textos(&svg)
            .into_iter()
            .filter(|t| t.starts_with("0x"))
            .collect();
        assert_eq!(addrs, vec!["0x7FFC", "0x7FF8", "0x7FF0", "0x7FE8", "0x7FE4"]);
        assert_eq!(svg.matches("marker-end").count(), 2);
    }

    #[test]
    fn pila_de_llamadas_en_el_tiempo() {
        let svg = render_val(
            json!({
                "type": "call_stack",
                "steps": [
                    {
                        "label": "llama a fact(2)",
                        "frames": [
                            {"function": "main", "vars": {"r": "?"}},
                            {"function": "fact", "params": {"n": 2}}
                        ]
                    },
                    {
                        "label": "fact(1) devuelve 1",
                        "frames": [
                            {"function": "main", "vars": {"r": "?"}},
                            {"function": "fact", "params": {"n": 2}},
                            {"function": "fact", "params": {"n": 1}, "returns": 1, "popped": true}
                        ]
                    }
                ]
            }),
            true,
        )
        .unwrap();
        assert!(svg.contains(">llama a fact(2)<"));
        assert!(svg.contains(">fact(1) devuelve 1<"));
        assert!(svg.contains(">devolvió 1<"));
        assert!(svg.contains(r#"stroke-dasharray="5 3""#));
        assert_eq!(svg.matches(">fact(n)<").count(), 3);
    }

    #[test]
    fn pila_de_llamadas_rotulos_largos_no_se_pisan() {
        let rotulo = "segunda llamada, antes de c++";
        let svg = render_val(
            json!({
                "type": "call_stack",
                "steps": [
                    {"label": rotulo, "frames": [{"function": "f"}]},
                    {"label": rotulo, "frames": [{"function": "f"}]}
                ]
            }),
            true,
        )
        .unwrap();
        let needle = format!(">{rotulo}<");
        let mut xs = Vec::new();
        let mut rest = svg.as_str();
        while let Some(idx) = rest.find(&needle) {
            let prefix = &rest[..idx];
            if let Some(x_pos) = prefix.rfind(r#"<text x=""#) {
                let after = &prefix[x_pos + 9..];
                if let Some(end) = after.find('"') {
                    if let Ok(x) = after[..end].parse::<f64>() {
                        xs.push(x);
                    }
                }
            }
            rest = &rest[idx + needle.len()..];
        }
        assert_eq!(xs.len(), 2);
        let min_sep = rotulo.chars().count() as f64 * 7.0;
        assert!(xs[1] - xs[0] >= min_sep);
        let width = extraer_attrs(&svg, "width").unwrap();
        assert!(width - 8.0 >= xs[1] + min_sep);
    }

    #[test]
    fn pila_de_llamadas_en_blanco() {
        let mut v = pila_llamadas_fixture();
        v["blankValues"] = json!(true);
        v["blankAddrs"] = json!(true);
        let svg = render_val(v, true).unwrap();
        assert!(!svg.contains(">7<"));
        assert!(svg.contains(">0x____<"));
    }

    #[test]
    fn pila_de_llamadas_errores_exactos() {
        let casos = [
            (json!({"type": "call_stack"}), "lista de marcos"),
            (json!({"type": "call_stack", "frames": []}), "lista de marcos"),
            (
                json!({"type": "call_stack", "frames": [123]}),
                "el marco 0 necesita 'function'",
            ),
            (
                json!({"type": "call_stack", "frames": [{"vars": {"x": 1}}]}),
                "necesita 'function'",
            ),
            (
                json!({"type": "call_stack", "frames": [{"function": "main", "vars": {"x": 1}}], "pointers": "0.x -> 0.z"}),
                "no tiene la variable 'z'",
            ),
            (
                json!({"type": "call_stack", "frames": [{"function": "main", "vars": {"x": 1}}], "pointers": "0.x"}),
                "flecha",
            ),
            (
                json!({"type": "call_stack", "frames": vec![json!({"function": "f"}); 11]}),
                "el máximo es 10",
            ),
            (
                json!({"type": "call_stack", "steps": vec![json!({"frames": [{"function": "main"}]}); 9]}),
                "el máximo es 8",
            ),
            (
                json!({"type": "call_stack", "frames": false}),
                "lista de marcos",
            ),
            (
                json!({"type": "call_stack", "steps": false}),
                "'steps'",
            ),
            (
                json!({"type": "call_stack", "steps": 5}),
                "'steps' tiene que ser una lista",
            ),
            (
                json!({"type": "call_stack", "steps": []}),
                "'steps' tiene que ser una lista",
            ),
            (
                json!({"type": "call_stack", "steps": [{"frames": []}]}),
                "paso 0: 'frames'",
            ),
            (
                json!({"type": "call_stack", "frames": [{"function": "m", "params": 5}]}),
                "diccionario o lista",
            ),
            (
                json!({"type": "call_stack", "frames": [{"function": "m", "vars": false}]}),
                "diccionario o lista",
            ),
            (
                json!({"type": "call_stack", "frames": [{"function": "m", "vars": [123]}]}),
                "variable inválida",
            ),
            (
                json!({"type": "call_stack", "frames": [{"function": "m"}], "pointers": "invalido -> 0.x"}),
                "referencia",
            ),
            (
                json!({"type": "call_stack", "frames": [{"function": "m"}], "pointers": ".x -> 0.x"}),
                "referencia",
            ),
            (
                json!({"type": "call_stack", "frames": [{"function": "m"}], "pointers": "9.x -> 0.x"}),
                "no hay marco '9'",
            ),
            (
                json!({"type": "call_stack", "frames": [{"function": "m"}], "pointers": "noexiste.x -> 0.x"}),
                "no hay marco 'noexiste'",
            ),
            (
                json!({"type": "call_stack", "frames": [{"function": "m"}], "pointers": 123}),
                "'pointers'",
            ),
            (
                json!({"type": "call_stack", "steps": [123]}),
                "el paso 0 tiene que tener 'frames'",
            ),
        ];
        for (bloque, esperado) in casos {
            let err = render_val(bloque, true).unwrap_err().to_string();
            assert!(
                err.starts_with("call_stack: "),
                "error '{err}' no empieza con 'call_stack: '"
            );
            assert!(
                err.contains(esperado),
                "error '{err}' no contiene '{esperado}'"
            );
        }
    }

    // --- call_tree ---

    #[test]
    fn arbol_de_llamadas_temas_y_paletas() {
        let bloque = json!({
            "type": "call_tree",
            "root": fib3_fixture(),
            "showOrder": true,
            "levelCosts": ["1", "2", "2"],
            "total": "2^n"
        });
        let svg_tinta = render_val(bloque.clone(), true).unwrap();
        let svg_color = render_val(bloque, false).unwrap();
        assert!(extraer_attrs(&svg_tinta, "width").unwrap() > 50.0);
        assert!(extraer_attrs(&svg_tinta, "height").unwrap() > 50.0);
        assert!(!svg_tinta.contains("#1d4ed8"));
        assert!(svg_color.contains("#1d4ed8"));
    }

    #[test]
    fn arbol_de_llamadas_orden_y_retornos() {
        let svg = render_val(
            json!({
                "type": "call_tree",
                "root": fib3_fixture(),
                "showOrder": true
            }),
            true,
        )
        .unwrap();
        let textos = extraer_textos(&svg);
        assert_eq!(textos.iter().filter(|t| *t == "fib(1)").count(), 2);
        assert!(textos.iter().any(|t| t == "→ 2"));
        let digitos: Vec<String> = textos
            .into_iter()
            .filter(|t| !t.is_empty() && t.chars().all(|ch| ch.is_ascii_digit()))
            .collect();
        assert_eq!(digitos, vec!["1", "2", "3", "4", "5"]);
    }

    #[test]
    fn arbol_de_llamadas_el_padre_queda_sobre_sus_hijos() {
        let svg = render_val(
            json!({
                "type": "call_tree",
                "root": {"call": "a", "children": ["b", "c"]}
            }),
            true,
        )
        .unwrap();
        let mut xs = BTreeMap::new();
        for nombre in ["a", "b", "c"] {
            let pat = format!(">{nombre}</text>");
            let idx = svg.find(&pat).unwrap();
            let prefix = &svg[..idx];
            let xp = prefix.rfind(r#"<text x=""#).unwrap();
            let after = &prefix[xp + 9..];
            let end = after.find('"').unwrap();
            let x: f64 = after[..end].parse().unwrap();
            xs.insert(nombre, x);
        }
        assert!(xs["b"] < xs["a"] && xs["a"] < xs["c"]);
        assert!((xs["a"] - (xs["b"] + xs["c"]) / 2.0).abs() < 1e-6);
    }

    #[test]
    fn arbol_de_llamadas_en_blanco_y_costos() {
        let bloque = json!({
            "type": "call_tree",
            "root": fib3_fixture(),
            "blankReturns": true,
            "levelCosts": ["n", "2n"],
            "total": "n^2"
        });
        let textos = extraer_textos(&render_val(bloque.clone(), true).unwrap());
        assert!(!textos.iter().any(|t| t == "→ 2"));
        assert!(textos.iter().any(|t| t == "costo del nivel"));
        assert!(textos.iter().any(|t| t == "2n"));
        assert!(textos.iter().any(|t| t == "T(n) = n^2"));

        let sin_llamadas = extraer_textos(
            &render_val(
                json!({
                    "type": "call_tree",
                    "root": fib3_fixture(),
                    "blankCalls": true
                }),
                true,
            )
            .unwrap(),
        );
        assert!(!sin_llamadas.iter().any(|t| t == "fib(3)"));

        let mut blc = bloque;
        blc["blankLevelCosts"] = json!(true);
        let sin_costos = extraer_textos(&render_val(blc, true).unwrap());
        assert!(!sin_costos.iter().any(|t| t == "n"));

        // Casos edge falsy no-nulos: levelCosts: false, total: 0, root escalar
        let falsy = render_val(
            json!({
                "type": "call_tree",
                "root": "f(0)",
                "levelCosts": false,
                "total": 0
            }),
            true,
        )
        .unwrap();
        let ft = extraer_textos(&falsy);
        assert!(ft.iter().any(|t| t == "costo del nivel"));
        assert!(ft.iter().any(|t| t == "T(n) = 0"));
    }

    #[test]
    fn arbol_de_llamadas_errores_y_tipos_malformados() {
        let casos = [
            (json!({"type": "call_tree"}), "root"),
            (json!({"type": "call_tree", "root": null}), "root"),
            (json!({"type": "call_tree", "root": ""}), "root"),
            (json!({"type": "call_tree", "root": {"returns": 1}}), "call"),
            (
                json!({"type": "call_tree", "root": {"call": "f", "children": [{"returns": 1}]}}),
                "call",
            ),
            (
                json!({"type": "call_tree", "root": {"call": "f", "children": 5}}),
                "'children' de un árbol de llamadas tiene que ser una lista",
            ),
            (
                json!({
                    "type": "call_tree",
                    "root": {"call": "f", "children": vec!["g"; 40]}
                }),
                "máximo",
            ),
            (
                json!({
                    "type": "call_tree",
                    "root": {"call": "f"},
                    "highlight": 5
                }),
                "'highlight' tiene que ser una lista",
            ),
            (
                json!({
                    "type": "call_tree",
                    "root": {"call": "f"},
                    "levelCosts": 5
                }),
                "'levelCosts'",
            ),
        ];
        for (bloque, esperado) in casos {
            let err = render_val(bloque, true).unwrap_err().to_string();
            assert!(
                err.starts_with("call_tree: "),
                "error '{err}' no empieza con 'call_tree: '"
            );
            assert!(err.contains(esperado), "error '{err}' no contiene '{esperado}'");
        }
    }

    // --- tree + construir_arbol ---

    #[test]
    fn tree_bloques_de_test_diagramas() {
        let bloques = [
            json!({"type": "tree", "variant": "bst", "insert": [8, 3, 10, 1, 6]}),
            json!({"type": "tree", "variant": "avl", "insert": [1, 2, 3, 4, 5]}),
            json!({"type": "tree", "variant": "heap", "values": [90, 70, 80, 30]}),
            json!({"type": "tree", "nodeStyle": "struct", "values": ["A", "B", "C"]}),
            json!({"type": "tree", "root": {"value": "+", "left": "x", "right": {"value": "*", "left": 2, "right": "y"}}}),
        ];
        for b in bloques {
            let svg = render_val(b.clone(), true).unwrap();
            assert!(extraer_attrs(&svg, "width").unwrap() > 50.0);
            assert!(extraer_attrs(&svg, "height").unwrap() > 50.0);
            if let Some(vals) = b.get("values").and_then(Value::as_array) {
                for v in vals {
                    let s = texto(v);
                    if !s.is_empty() {
                        assert!(svg.contains(&format!(">{s}<")));
                    }
                }
            }
        }
    }

    #[test]
    fn abb_por_inserciones() {
        let b = bloque_de(json!({
            "type": "tree",
            "variant": "bst",
            "insert": [50, 30, 70, 20, 40, 30]
        }));
        let raiz = construir_arbol(&b).unwrap().unwrap();
        assert_eq!(raiz.valor, "50");
        assert_eq!(
            en_orden(&Some(raiz)),
            vec!["20", "30", "40", "50", "70"]
        );
    }

    #[test]
    fn avl_rebalancea_y_dibuja_balance() {
        let b_val = json!({
            "type": "tree",
            "variant": "avl",
            "insert": [10, 20, 30, 40, 50, 25]
        });
        let raiz = construir_arbol(&bloque_de(b_val.clone())).unwrap().unwrap();
        assert_eq!(raiz.valor, "30");
        assert_eq!(raiz.altura(), 3);
        assert_eq!(
            en_orden(&Some(raiz)),
            vec!["10", "20", "25", "30", "40", "50"]
        );
        let svg = render_val(b_val, true).unwrap();
        assert!(svg.contains(">-1<"));
    }

    #[test]
    fn arbol_por_niveles_con_huecos() {
        let b = bloque_de(json!({
            "type": "tree",
            "values": ["A", "B", "C", "", "D", "null", "~"]
        }));
        let raiz = construir_arbol(&b).unwrap().unwrap();
        let izq = raiz.izq.as_ref().unwrap();
        assert!(izq.izq.is_none());
        assert_eq!(izq.der.as_ref().unwrap().valor, "D");
        let der = raiz.der.as_ref().unwrap();
        assert!(der.izq.is_none());
        assert!(der.der.is_none());
    }

    #[test]
    fn heap_con_show_array_y_mas_de_16_valores() {
        let vals: Vec<i32> = (1..=20).rev().collect();
        let svg_con = render_val(
            json!({
                "type": "tree",
                "variant": "heap",
                "values": vals
            }),
            true,
        )
        .unwrap();
        assert!(svg_con.contains("Índice [i]:"));
        let g_start = svg_con.find("<g transform=").unwrap();
        let sub_g = &svg_con[g_start..];
        assert_eq!(sub_g.matches("<rect ").count(), 16);
        assert!(sub_g.contains(">[15]<"));
        assert!(!sub_g.contains(">[16]<"));
        assert!(svg_con.contains(">[19]<")); // nodo del árbol sí llega a [19]

        let svg_sin = render_val(
            json!({
                "type": "tree",
                "variant": "heap",
                "values": [90, 70, 80],
                "showArray": false
            }),
            true,
        )
        .unwrap();
        assert!(!svg_sin.contains("Índice [i]:"));
    }

    #[test]
    fn tree_variantes_estilo_utf8_nan_y_errores() {
        let svg_nan = render_val(
            json!({
                "type": "tree",
                "variant": "bst",
                "insert": ["NaN", "inf", "ábaco", "ñoño", "1e308"],
                "highlight": ["ábaco"],
                "nullLeaves": true
            }),
            false,
        )
        .unwrap();
        assert!(svg_nan.contains(">ábaco<"));
        assert!(svg_nan.contains(">ñoño<"));
        assert!(svg_nan.contains(">NULL<"));
        assert!(svg_nan.contains("#fef3c7")); // fillHighlight color

        let svg_struct = render_val(
            json!({
                "type": "tree",
                "nodeStyle": "struct",
                "values": ["A", "B"],
                "blankValues": true
            }),
            true,
        )
        .unwrap();
        assert!(svg_struct.contains(">*izq<"));
        assert!(svg_struct.contains(">*der<"));
        assert!(!svg_struct.contains(">A<"));

        let err_vacio = render_val(json!({"type": "tree", "values": []}), true)
            .unwrap_err()
            .to_string();
        assert!(err_vacio.starts_with("tree: "));
        assert!(err_vacio.contains("el árbol no tiene nodos"));

        let err_ins = render_val(json!({"type": "tree", "insert": 5}), true)
            .unwrap_err()
            .to_string();
        assert!(err_ins.starts_with("tree: "));
        assert!(err_ins.contains("'insert' tiene que ser una lista"));

        // Smoke con leer_bloques YAML
        let bloques = leer_bloques("type: tree\ninsert: [2, 1]\n").unwrap();
        assert!(reg_test().renderizar(&bloques[0], true).is_ok());
    }

    #[test]
    fn tree_errores_y_envolver_tipo_idempotente() {
        let casos = [
            (json!({"type": "tree"}), "el árbol no tiene nodos"),
            (json!({"type": "tree", "values": []}), "el árbol no tiene nodos"),
            (json!({"type": "tree", "values": ["null"]}), "el árbol no tiene nodos"),
            (json!({"type": "tree", "values": 5}), "'values' tiene que ser una lista"),
            (json!({"type": "tree", "insert": 5}), "'insert' tiene que ser una lista"),
            (
                json!({"type": "tree", "values": ["A"], "highlight": 5}),
                "'highlight' tiene que ser una lista",
            ),
            (
                json!({"type": "tree", "variant": "heap", "root": {"value": "A"}, "values": 5}),
                "'values' tiene que ser una lista",
            ),
        ];
        for (bloque, esperado) in casos {
            let err = render_val(bloque, true).unwrap_err().to_string();
            assert!(
                err.starts_with("tree: "),
                "error '{err}' no empieza con 'tree: '"
            );
            assert!(err.contains(esperado), "error '{err}' no contiene '{esperado}'");
        }

        assert_eq!(
            envolver_tipo("tree", Ok((12.0, 34.0))).unwrap(),
            (12.0, 34.0)
        );
        let ya = envolver_tipo("tree", Err(ErrorDiagrama::new("tree: mensaje ya prefijado")))
            .unwrap_err()
            .to_string();
        assert_eq!(ya, "tree: mensaje ya prefijado");

        let falso_prefijo = envolver_tipo("tree", Err(ErrorDiagrama::new("tree_extra: otro")))
            .unwrap_err()
            .to_string();
        assert_eq!(falso_prefijo, "tree: tree_extra: otro");
    }

    #[test]
    fn tree_y_call_tree_acotan_dos_sin_timeout() {
        let grande: Vec<i64> = (0..15_000).collect();
        let err = render_val(
            json!({"type": "tree", "variant": "binary", "insert": grande}),
            true,
        )
        .unwrap_err()
        .to_string();
        assert!(err.contains("máximo es 512"), "{err}");

        // 500 nodos (caso usado en stress test) renderizan rápido tanto en BST como en AVL
        let medio: Vec<i64> = (0..500).collect();
        let svg_bst = render_val(
            json!({"type": "tree", "variant": "bst", "insert": medio}),
            true,
        )
        .unwrap();
        assert!(svg_bst.starts_with("<svg "));
        let svg_avl = render_val(
            json!({"type": "tree", "variant": "avl", "insert": (0..500).collect::<Vec<i64>>()}),
            true,
        )
        .unwrap();
        assert!(svg_avl.starts_with("<svg "));

        // call_tree con > 40 nodos corta en parsear_llamada
        let mut hijos = Vec::new();
        for i in 0..45 {
            hijos.push(json!({"call": format!("f({i})")}) );
        }
        let err_ct = render_val(
            json!({"type": "call_tree", "root": {"call": "root", "children": hijos}}),
            true,
        )
        .unwrap_err()
        .to_string();
        assert!(err_ct.contains("el máximo es 40"), "{err_ct}");
    }
}
