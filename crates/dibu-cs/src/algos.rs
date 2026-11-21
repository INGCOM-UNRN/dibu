//! Renderizadores de structs, TADs y algoritmos de `dibu-cs` (`struct`, `adt`, `complexity`, `flowchart`, `cost_table`, `trace_table`).

use crate::common::{
    ancho_panel, ancho_texto, encabezado, entero, entero_b, es_truthy, fmt_hex, hex_b, lista_de,
    lista_o_vacia, panel, str_o, tabla, texto, Bloque, ErrorDiagrama, Lienzo, Punto, TablaOpts,
    MONO, SANS,
};
use dibu_core::Registro;
use serde_json::Value;
use std::collections::BTreeMap;

fn envolver_tipo(tipo: &str, res: Result<Punto, ErrorDiagrama>) -> Result<Punto, ErrorDiagrama> {
    res.map_err(|e| {
        if e.0.strip_prefix(tipo).is_some_and(|rest| rest.starts_with(": ")) {
            e
        } else {
            ErrorDiagrama::new(format!("{tipo}: {}", e.0))
        }
    })
}

/// Registra los 6 renderizadores de structs, TADs y algoritmos en el registro global.
pub fn registrar(reg: &mut Registro) {
    reg.registrar("struct", |c, b| envolver_tipo("struct", render_struct(c, b)));
    reg.registrar("adt", |c, b| envolver_tipo("adt", render_adt(c, b)));
    reg.registrar("complexity", |c, b| {
        envolver_tipo("complexity", render_complexity(c, b))
    });
    reg.registrar("flowchart", |c, b| {
        envolver_tipo("flowchart", render_flowchart(c, b))
    });
    reg.registrar("cost_table", |c, b| {
        envolver_tipo("cost_table", render_cost_table(c, b))
    });
    reg.registrar("trace_table", |c, b| {
        envolver_tipo("trace_table", render_trace_table(c, b))
    });
}

// ---------------------------------------------------------------------------
// Struct: disposición en memoria (LP64 GCC x86_64) y tipo opaco
// ---------------------------------------------------------------------------

const MAX_CAMPOS: usize = 16;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CampoStruct {
    pub nombre: String,
    pub tipo: String,
    pub dims: String,
    pub valor: String,
    pub tamano: usize,
    pub alineacion: usize,
    pub offset: usize,
    pub en_blanco: bool,
}

impl CampoStruct {
    /// Parsea y mide una declaración C simple (p. ej. `"char nombre[10]"` o `"int edad = 30"`).
    pub fn desde_str(decl: &str) -> Result<Self, ErrorDiagrama> {
        let mut campo = parsear_campo_str(decl, "campo 0")?;
        medir_campo(&mut campo, None, None, "campo 0")?;
        Ok(campo)
    }

    /// Devuelve la declaración formateada sin el inicializador (`char nombre[10]`, `int *p`).
    pub fn declaracion(&self) -> String {
        let sep = if self.tipo.ends_with('*') { "" } else { " " };
        format!("{}{sep}{}{}", self.tipo, self.nombre, self.dims)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DisposicionStruct {
    pub rellenos: Vec<(usize, usize)>,
    pub sizeof: usize,
    pub alineacion: usize,
}

/// Parsea y mide un slice de declaraciones C usando las reglas LP64.
pub fn campos_struct_desde_strs(decls: &[&str]) -> Result<Vec<CampoStruct>, ErrorDiagrama> {
    let mut campos = Vec::with_capacity(decls.len());
    for (n, decl) in decls.iter().enumerate() {
        let donde = format!("campo {n}");
        let mut campo = parsear_campo_str(decl, &donde)?;
        medir_campo(&mut campo, None, None, &donde)?;
        campos.push(campo);
    }
    Ok(campos)
}

/// Calcula los offsets, rellenos internos/finales, `sizeof` y alineación del struct.
pub fn disposicion_struct(campos: &mut [CampoStruct], empaquetado: bool) -> DisposicionStruct {
    let mut rellenos = Vec::new();
    let mut offset: usize = 0;
    let mut alineacion: usize = 1;

    for campo in campos.iter_mut() {
        let al = if empaquetado {
            1
        } else {
            campo.alineacion.max(1)
        };
        alineacion = alineacion.max(al);
        let resto = offset % al;
        if resto != 0 {
            let pad = al - resto;
            rellenos.push((offset, pad));
            offset = offset.saturating_add(pad);
        }
        campo.offset = offset;
        offset = offset.saturating_add(campo.tamano);
    }

    let resto_final = offset % alineacion;
    if resto_final != 0 {
        let pad = alineacion - resto_final;
        rellenos.push((offset, pad));
        offset = offset.saturating_add(pad);
    }

    DisposicionStruct {
        rellenos,
        sizeof: offset,
        alineacion,
    }
}

fn tamano_tipo_c(tipo: &str) -> Option<usize> {
    match tipo {
        "char" | "bool" | "_bool" | "int8_t" | "uint8_t" => Some(1),
        "short" | "int16_t" | "uint16_t" => Some(2),
        "int" | "unsigned" | "float" | "int32_t" | "uint32_t" => Some(4),
        "long" | "long long" | "double" | "size_t" | "int64_t" | "uint64_t" => Some(8),
        _ => None,
    }
}

fn es_palabra_c(c: char) -> bool {
    c.is_alphanumeric() || c == '_'
}

fn error_no_entiendo(donde: &str, raw: &str) -> ErrorDiagrama {
    ErrorDiagrama::new(format!(
        "{donde}: no entiendo {raw:?} (se espera «char nombre[8]» o «int edad = 30»)"
    ))
}

fn parsear_campo_str(raw: &str, donde: &str) -> Result<CampoStruct, ErrorDiagrama> {
    let s = raw.trim().trim_end_matches(';');
    let (izq, valor) = match s.split_once('=') {
        Some((l, r)) => (l.trim_end(), r.trim().to_string()),
        None => (s, String::new()),
    };

    // Extraer sufijos [N] desde el final sin espacios entre grupos.
    let mut dims_rev: Vec<&str> = Vec::new();
    let mut rest = izq;
    while let Some(antes_cierre) = rest.strip_suffix(']') {
        let Some(pos_abre) = antes_cierre.rfind('[') else {
            break;
        };
        let contenido = &antes_cierre[pos_abre + 1..];
        if contenido.is_empty() || !contenido.bytes().all(|b| b.is_ascii_digit()) {
            break;
        }
        dims_rev.push(&rest[pos_abre..]);
        rest = &antes_cierre[..pos_abre];
    }
    dims_rev.reverse();
    let dims = dims_rev.concat();

    // Extraer identificador del campo (`\w+`) desde el final.
    let nombre_inicio = rest
        .char_indices()
        .rev()
        .take_while(|&(_, c)| es_palabra_c(c))
        .map(|(idx, _)| idx)
        .last();
    let Some(nombre_inicio) = nombre_inicio else {
        return Err(error_no_entiendo(donde, raw));
    };
    let nombre = rest[nombre_inicio..].to_string();
    let prefijo = rest[..nombre_inicio].trim_end();
    if prefijo.is_empty() {
        return Err(error_no_entiendo(donde, raw));
    }

    // Separar las estrellas finales (`*`) equivalentes a `(?P<estrella>\**)`.
    let sin_estrellas = prefijo.trim_end_matches('*');
    let estrellas = &prefijo[sin_estrellas.len()..];
    let tipo_base = sin_estrellas.trim_end();
    if tipo_base.is_empty() {
        return Err(error_no_entiendo(donde, raw));
    }

    // Validar que `tipo_base` cumpla `[\w ]*?[\w*]`: último char `\w` o `*`, anteriores `\w` o `' '`.
    let mut chars_tipo = tipo_base.chars();
    let Some(ultimo) = chars_tipo.next_back() else {
        return Err(error_no_entiendo(donde, raw));
    };
    if !(es_palabra_c(ultimo) || ultimo == '*') {
        return Err(error_no_entiendo(donde, raw));
    }
    if !chars_tipo.all(|c| es_palabra_c(c) || c == ' ') {
        return Err(error_no_entiendo(donde, raw));
    }

    let tipo = if estrellas.is_empty() {
        tipo_base.to_string()
    } else {
        format!("{tipo_base} {estrellas}")
    };

    Ok(CampoStruct {
        nombre,
        tipo,
        dims,
        valor,
        tamano: 0,
        alineacion: 1,
        offset: 0,
        en_blanco: false,
    })
}

fn extraer_dims_y_elementos(texto_dims: &str) -> (String, usize) {
    let mut grupos = Vec::new();
    let mut elementos: usize = 1;
    let bytes = texto_dims.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'[' {
            let mut j = i + 1;
            while j < bytes.len() && bytes[j].is_ascii_digit() {
                j += 1;
            }
            if j > i + 1 && j < bytes.len() && bytes[j] == b']' {
                let num_str = &texto_dims[i + 1..j];
                let d = num_str.parse::<usize>().unwrap_or(1 << 20);
                elementos = elementos.saturating_mul(d);
                grupos.push(&texto_dims[i..=j]);
                i = j + 1;
                continue;
            }
        }
        i += 1;
    }
    (grupos.concat(), elementos)
}

fn normalizar_tipo_c(tipo_orig: &str) -> String {
    let lower = tipo_orig.to_lowercase();
    let mut palabras = Vec::new();
    let mut inicio: Option<usize> = None;
    for (idx, c) in lower.char_indices() {
        if es_palabra_c(c) {
            if inicio.is_none() {
                inicio = Some(idx);
            }
        } else if let Some(s) = inicio.take() {
            let pal = &lower[s..idx];
            if !matches!(pal, "const" | "volatile" | "signed" | "unsigned") {
                palabras.push(pal);
            }
        }
    }
    if let Some(s) = inicio {
        let pal = &lower[s..];
        if !matches!(pal, "const" | "volatile" | "signed" | "unsigned") {
            palabras.push(pal);
        }
    }
    let unido = palabras.join(" ");
    let sin_int = unido.strip_suffix(" int").unwrap_or(&unido);
    if sin_int.is_empty() {
        if lower.contains("unsigned") {
            "unsigned".to_string()
        } else {
            "int".to_string()
        }
    } else {
        sin_int.to_string()
    }
}

fn medir_campo(
    campo: &mut CampoStruct,
    tamano_val: Option<&Value>,
    alineacion_val: Option<&Value>,
    donde: &str,
) -> Result<(), ErrorDiagrama> {
    let base = if campo.tipo.contains('*') {
        8
    } else {
        let normalizado = normalizar_tipo_c(&campo.tipo);
        match tamano_tipo_c(&normalizado) {
            Some(sz) => sz,
            None => {
                if tamano_val.is_none() {
                    return Err(ErrorDiagrama::new(format!(
                        "{donde}: no conozco el tamaño de '{}': indicalo con 'size' (y 'align')",
                        campo.tipo
                    )));
                }
                0
            }
        }
    };

    let (_, elementos) = extraer_dims_y_elementos(&campo.dims);
    campo.tamano = if let Some(v) = tamano_val {
        entero(Some(v), 0, 1, 1 << 20) as usize
    } else {
        base.saturating_mul(elementos)
    };

    campo.alineacion = if let Some(v) = alineacion_val {
        entero(Some(v), 1, 1, 64) as usize
    } else if base != 0 {
        base
    } else {
        campo.tamano.clamp(1, 8)
    };
    Ok(())
}

fn parsear_campos_struct(b: &Bloque) -> Result<Vec<CampoStruct>, ErrorDiagrama> {
    let datos = lista_de(b, "fields", "un struct")?;
    if datos.len() > MAX_CAMPOS {
        return Err(ErrorDiagrama::new(format!(
            "el struct tiene {} campos; el máximo es {MAX_CAMPOS}",
            datos.len()
        )));
    }
    let mut campos = Vec::with_capacity(datos.len());
    for (n, dato) in datos.iter().enumerate() {
        let donde = format!("campo {n}");
        if let Some(obj) = dato.as_object() {
            let nombre_raw = obj
                .get("name")
                .filter(|v| es_truthy(v))
                .map(texto)
                .unwrap_or_default();
            let tipo_opt = obj.get("type").filter(|v| es_truthy(v)).map(texto);
            let Some(tipo_raw) = tipo_opt else {
                return Err(ErrorDiagrama::new(format!(
                    "{donde}: le falta 'name' o 'type'"
                )));
            };
            if nombre_raw.is_empty() {
                return Err(ErrorDiagrama::new(format!(
                    "{donde}: le falta 'name' o 'type'"
                )));
            }
            let (dims, _) = extraer_dims_y_elementos(&nombre_raw);
            let nombre = nombre_raw.split('[').next().unwrap_or("").to_string();
            let valor = obj.get("value").map(texto).unwrap_or_default();
            let en_blanco = obj.get("blank").is_some_and(es_truthy);
            let mut campo = CampoStruct {
                nombre,
                tipo: tipo_raw.trim().to_string(),
                dims,
                valor,
                tamano: 0,
                alineacion: 1,
                offset: 0,
                en_blanco,
            };
            let tam_v = obj.get("size").filter(|v| !v.is_null());
            let al_v = obj.get("align").filter(|v| !v.is_null());
            medir_campo(&mut campo, tam_v, al_v, &donde)?;
            campos.push(campo);
        } else {
            let raw = texto(dato);
            let mut campo = parsear_campo_str(&raw, &donde)?;
            medir_campo(&mut campo, None, None, &donde)?;
            campos.push(campo);
        }
    }
    Ok(campos)
}

fn render_struct(c: &mut Lienzo, b: &Bloque) -> Result<Punto, ErrorDiagrama> {
    match b.get("variant") {
        None => {}
        Some(Value::String(s)) if s == "layout" => {}
        Some(Value::String(s)) if s == "opaque" => return render_tipo_opaco(c, b),
        Some(otro) => {
            return Err(ErrorDiagrama::new(format!(
                "variante de struct no soportada: {otro:?} (layout u opaque)"
            )));
        }
    }

    let mut campos = parsear_campos_struct(b)?;
    let empaquetado = b.get("packed").is_some_and(es_truthy);
    let disp = disposicion_struct(&mut campos, empaquetado);
    let nombre = str_o(b, "name", "struct registro");
    let alias = b.get("typedef").filter(|v| es_truthy(v)).map(texto);
    let extra = match &alias {
        Some(a) => format!("typedef {nombre} {a};"),
        None => nombre.clone(),
    };
    let top = encabezado(c, b, &extra);

    let sin_valores = b.get("blankValues").is_some_and(es_truthy);
    let con_valores = campos.iter().any(|f| !f.valor.is_empty()) || sin_valores;
    let direcciones = b.get("showAddresses").is_some_and(es_truthy);
    let base = hex_b(b, "baseAddr", 0x1000);
    let sin_offsets = b.get("blankOffsets").is_some_and(es_truthy);
    let sin_tamanos = b.get("blankSizes").is_some_and(es_truthy);
    let sin_relleno = b.get("blankPadding").is_some_and(es_truthy);
    let sin_sizeof = b.get("blankSizeof").is_some_and(es_truthy);

    let ubicacion = |off: usize| -> String {
        if sin_offsets {
            String::new()
        } else if direcciones {
            fmt_hex(base.saturating_add(i64::try_from(off).unwrap_or(i64::MAX)))
        } else {
            format!("+{off}")
        }
    };

    let mut entradas: Vec<(usize, Vec<String>, bool)> = Vec::new();
    for f in &campos {
        let mut fila = vec![ubicacion(f.offset), f.declaracion()];
        if con_valores {
            fila.push(if sin_valores || f.en_blanco {
                String::new()
            } else {
                f.valor.clone()
            });
        }
        fila.push(if sin_tamanos {
            String::new()
        } else {
            format!("{} B", f.tamano)
        });
        entradas.push((f.offset, fila, false));
    }
    if !sin_relleno {
        for &(offset, bytes_relleno) in &disp.rellenos {
            let mut fila = vec![ubicacion(offset), "(relleno)".to_string()];
            if con_valores {
                fila.push("—".to_string());
            }
            fila.push(if sin_tamanos {
                String::new()
            } else {
                format!("{bytes_relleno} B")
            });
            entradas.push((offset, fila, true));
        }
    }
    entradas.sort_by_key(|e| e.0);

    let mut encabezados = vec![
        if direcciones { "dirección" } else { "offset" },
        "campo",
    ];
    if con_valores {
        encabezados.push("valor");
    }
    encabezados.push("tamaño");

    let tenues: Vec<usize> = entradas
        .iter()
        .enumerate()
        .filter_map(|(i, e)| if e.2 { Some(i) } else { None })
        .collect();
    let filas: Vec<Vec<String>> = entradas.into_iter().map(|e| e.1).collect();

    let (ancho, alto) = tabla(
        c,
        0.0,
        top + 6.0,
        &encabezados,
        &filas,
        TablaOpts {
            izquierda: &[1],
            tenues: &tenues,
            ..TablaOpts::default()
        },
    );

    let y = top + 6.0 + alto + 20.0;
    let resultado = if sin_sizeof {
        "______".to_string()
    } else {
        format!("{} bytes", disp.sizeof)
    };
    let mut pie = format!("sizeof({}) = {resultado}", alias.unwrap_or(nombre));
    if empaquetado {
        pie.push_str(" (empaquetado)");
    } else if !sin_sizeof {
        pie.push_str(&format!(" | alineación: {}", disp.alineacion));
    }
    c.t(0.0, y, &pie, 11.0, 700, "textMain", "start", MONO);
    Ok((ancho.max(ancho_texto(&pie, 11.0)), y + 8.0))
}

fn render_tipo_opaco(c: &mut Lienzo, b: &Bloque) -> Result<Punto, ErrorDiagrama> {
    let nombre = str_o(b, "name", "struct pila");
    let etiqueta = nombre.strip_prefix("struct ").unwrap_or(&nombre).trim();
    let alias = str_o(b, "typedef", &format!("{etiqueta}_t"));
    let variable = str_o(b, "variable", "p");
    let cabecera = str_o(b, "header", &format!("{etiqueta}.h"));
    let fuente = str_o(b, "source", &format!("{etiqueta}.c"));
    let cliente = str_o(b, "client", "main.c");
    let oculto = b.get("hidden").is_some_and(es_truthy);
    let top = encabezado(c, b, "");

    let ops = lista_o_vacia(b, "operations")?;
    let mut publico = Vec::with_capacity(1 + ops.len());
    publico.push(format!("typedef {nombre} *{alias};"));
    for o in ops {
        publico.push(format!("{};", texto(o).trim_end_matches(';')));
    }
    let titulo_h = format!("{cabecera} (interfaz pública)");

    let campos = if oculto && !b.get("fields").is_some_and(es_truthy) {
        Vec::new()
    } else {
        parsear_campos_struct(b)?
    };

    let mut privado = Vec::new();
    privado.push(format!("{nombre} {{"));
    if b.get("blankValues").is_some_and(es_truthy) {
        for _ in &campos {
            privado.push(format!("    {}", "_".repeat(16)));
        }
    } else if oculto {
        privado.push("    ¿?".to_string());
    } else {
        for f in &campos {
            privado.push(format!("    {};", f.declaracion()));
        }
    }
    privado.push("};".to_string());
    let titulo_c = format!("{fuente} (implementación)");

    let uso = vec![
        format!("#include \"{cabecera}\""),
        format!("{alias} {variable};"),
    ];
    let titulo_u = format!("{cliente} (cliente)");

    let w_u = ancho_panel(&titulo_u, &uso);
    let w_c_base = ancho_panel(&titulo_c, &privado);
    let sep = 130.0_f64;
    let w_h = ancho_panel(&titulo_h, &publico).max(w_u + sep + w_c_base);
    let w_c = w_h - w_u - sep;

    let y = top + 4.0;
    let alto_h = panel(c, 0.0, y, w_h, &titulo_h, &publico, None);
    let y2 = y + alto_h + 34.0;
    let alto = (24.0 + 8.0 + uso.len() as f64 * 17.0 + 46.0)
        .max(24.0 + 8.0 + privado.len() as f64 * 17.0);

    c.linea(
        w_u / 2.0,
        y + alto_h,
        w_u / 2.0,
        y2 - 3.0,
        "strokeSub",
        1.4,
        Some("sub"),
        "4 2",
    );
    c.t(
        w_u / 2.0 + 8.0,
        y + alto_h + 21.0,
        "incluye",
        9.5,
        400,
        "textMuted",
        "start",
        MONO,
    );
    let xc = w_u + sep;
    c.linea(
        xc + w_c / 2.0,
        y + alto_h,
        xc + w_c / 2.0,
        y2 - 3.0,
        "strokeSub",
        1.4,
        Some("sub"),
        "4 2",
    );
    c.t(
        xc + w_c / 2.0 + 8.0,
        y + alto_h + 21.0,
        "incluye",
        9.5,
        400,
        "textMuted",
        "start",
        MONO,
    );
    panel(c, 0.0, y2, w_u, &titulo_u, &uso, Some(alto));
    panel(c, xc, y2, w_c, &titulo_c, &privado, Some(alto));

    let py = y2 + alto - 30.0;
    c.r(10.0, py, w_u - 20.0, 22.0, "fillPointerBox", 1.4, 3.0);
    c.t(
        18.0,
        py + 15.0,
        &variable,
        10.5,
        700,
        "textMain",
        "start",
        MONO,
    );
    c.punto(w_u - 22.0, py + 11.0, "accentPrimary", 3.2);
    c.linea(
        w_u - 22.0,
        py + 11.0,
        xc - 3.0,
        py + 11.0,
        "accentPrimary",
        1.6,
        Some("main"),
        "",
    );

    let muro = w_u + sep / 2.0;
    c.linea(
        muro,
        y2 - 4.0,
        muro,
        y2 + alto + 4.0,
        "strokeMain",
        2.0,
        None,
        "6 4",
    );
    c.t(
        muro,
        y2 + alto + 18.0,
        "el cliente no ve los campos",
        9.5,
        600,
        "textMuted",
        "middle",
        MONO,
    );
    Ok((w_h, y2 + alto + 24.0))
}

// ---------------------------------------------------------------------------
// TAD: operaciones públicas, invariante y representaciones posibles
// ---------------------------------------------------------------------------

fn render_adt(c: &mut Lienzo, b: &Bloque) -> Result<Punto, ErrorDiagrama> {
    let nombre = str_o(b, "name", "TAD");
    let operaciones = lista_de(b, "operations", "un TAD")?;
    let sin_ops = b.get("blankOperations").is_some_and(es_truthy);
    let sin_clases = b.get("blankKinds").is_some_and(es_truthy);
    let top = encabezado(c, b, "");

    let mut filas_completas: Vec<[String; 4]> = Vec::with_capacity(operaciones.len());
    let mut con_clase = false;
    let mut con_pre = false;
    let mut con_post = false;

    for op in operaciones {
        let (sig, en_blanco, clase, pre, post) = if let Some(obj) = op.as_object() {
            let Some(sig_v) = obj.get("signature").filter(|v| es_truthy(v)) else {
                return Err(ErrorDiagrama::new(
                    "cada operación de un TAD tiene que tener 'signature'",
                ));
            };
            let en_blanco = sin_ops || obj.get("blank").is_some_and(es_truthy);
            let clase = obj.get("kind").map(texto).unwrap_or_default();
            let pre = obj.get("pre").map(texto).unwrap_or_default();
            let post = obj.get("post").map(texto).unwrap_or_default();
            (texto(sig_v), en_blanco, clase, pre, post)
        } else {
            if !es_truthy(op) {
                return Err(ErrorDiagrama::new(
                    "cada operación de un TAD tiene que tener 'signature'",
                ));
            }
            (
                texto(op),
                sin_ops,
                String::new(),
                String::new(),
                String::new(),
            )
        };

        con_clase |= !clase.is_empty();
        con_pre |= !pre.is_empty();
        con_post |= !post.is_empty();

        filas_completas.push([
            if en_blanco { String::new() } else { sig },
            if sin_clases {
                String::new()
            } else if clase.is_empty() {
                "—".to_string()
            } else {
                clase
            },
            if pre.is_empty() {
                "—".to_string()
            } else {
                pre
            },
            if post.is_empty() {
                "—".to_string()
            } else {
                post
            },
        ]);
    }

    let columnas = [
        ("operación", 0usize, true),
        ("clase", 1, con_clase || sin_clases),
        ("pre", 2, con_pre),
        ("post", 3, con_post),
    ];
    let usadas: Vec<(&str, usize)> = columnas
        .iter()
        .filter(|&&(_, _, usar)| usar)
        .map(|&(t, j, _)| (t, j))
        .collect();
    let encabezados: Vec<&str> = usadas.iter().map(|&(t, _)| t).collect();
    let filas_tabla: Vec<Vec<String>> = filas_completas
        .iter()
        .map(|f| usadas.iter().map(|&(_, j)| f[j].clone()).collect())
        .collect();
    let izq_cols: Vec<usize> = (0..encabezados.len()).collect();

    let mut representaciones: Vec<(String, Vec<String>)> = Vec::new();
    for r in lista_o_vacia(b, "representations")? {
        if let Some(obj) = r.as_object() {
            let lineas = match obj.get("fields") {
                None => Vec::new(),
                Some(v) if !es_truthy(v) => Vec::new(),
                Some(Value::Array(arr)) => arr.iter().map(texto).collect(),
                Some(_) => {
                    return Err(ErrorDiagrama::new("'fields' tiene que ser una lista"));
                }
            };
            let titulo = obj
                .get("name")
                .filter(|v| es_truthy(v))
                .map(texto)
                .unwrap_or_else(|| "representación".to_string());
            representaciones.push((titulo, lineas));
        } else {
            let titulo = if es_truthy(r) {
                texto(r)
            } else {
                "representación".to_string()
            };
            representaciones.push((titulo, Vec::new()));
        }
    }

    let uso: Vec<String> = lista_o_vacia(b, "client")?.iter().map(texto).collect();
    let invariante = b.get("invariant").filter(|v| es_truthy(v)).map(texto);

    let mut y = top + 4.0;
    let ancho_rep = if representaciones.is_empty() {
        0.0
    } else {
        representaciones
            .iter()
            .map(|(t, ls)| ancho_panel(t, ls).max(130.0) + 24.0)
            .sum::<f64>()
            - 24.0
    };
    let mut ancho = ancho_rep;

    if !uso.is_empty() {
        let titulo_u = "Cliente: sólo usa las operaciones";
        let ancho_u = ancho_panel(titulo_u, &uso);
        let alto_u = panel(c, 0.0, y, ancho_u, titulo_u, &uso, None);
        ancho = ancho.max(ancho_u);
        c.linea(
            ancho_u / 2.0,
            y + alto_u,
            ancho_u / 2.0,
            y + alto_u + 26.0,
            "accentPrimary",
            1.5,
            Some("main"),
            "",
        );
        y += alto_u + 29.0;
    }

    c.t(
        0.0,
        y + 13.0,
        &format!("TAD {nombre}: interfaz"),
        11.5,
        700,
        "textMain",
        "start",
        SANS,
    );
    let (w_t, h_t) = tabla(
        c,
        0.0,
        y + 22.0,
        &encabezados,
        &filas_tabla,
        TablaOpts {
            izquierda: &izq_cols,
            min_w: 70.0,
            ..TablaOpts::default()
        },
    );
    ancho = ancho.max(w_t);
    y += 22.0 + h_t;

    if let Some(inv) = invariante {
        y += 18.0;
        let msg = format!("Invariante: {inv}");
        c.t(0.0, y, &msg, 10.5, 600, "textSub", "start", MONO);
        ancho = ancho.max(ancho_texto(&msg, 10.5));
    }

    if !representaciones.is_empty() {
        let y_rep = y + 44.0;
        c.t(
            0.0,
            y + 18.0,
            "Representaciones posibles (ocultas para el cliente):",
            10.0,
            600,
            "textMuted",
            "start",
            MONO,
        );
        let alto_rep = representaciones
            .iter()
            .map(|(_, ls)| 24.0 + 8.0 + ls.len().max(1) as f64 * 17.0)
            .fold(0.0_f64, f64::max);
        let mut x = 0.0;
        for (titulo, lineas) in &representaciones {
            let w = ancho_panel(titulo, lineas).max(130.0);
            panel(c, x, y_rep, w, titulo, lineas, Some(alto_rep));
            x += w + 24.0;
        }
        y = y_rep + alto_rep;
    }

    Ok((ancho, y + 8.0))
}

// ---------------------------------------------------------------------------
// Complejidad: curvas de crecimiento y clases anidadas
// ---------------------------------------------------------------------------

const MAX_CURVAS: usize = 9;
const MAX_CLASES: usize = 10;
const CLASES_COMUNES: [&str; 6] = ["1", "log n", "n", "n log n", "n^2", "2^n"];
const GUIONES: [&str; 4] = ["", "7 3", "2 3", "9 3 2 3"];

#[derive(Clone, Copy)]
struct CrecimientoDef {
    clave: &'static str,
    etiqueta: &'static str,
    enteras: bool,
    eval: fn(f64) -> f64,
}

fn gamma_lanczos(z: f64) -> f64 {
    if !z.is_finite() || z <= 0.0 || z > 172.0 {
        return f64::INFINITY;
    }
    const COEF: [f64; 9] = [
        0.999_999_999_999_809_9,
        676.5203681218851,
        -1259.1392167224028,
        771.323_428_777_653_1,
        -176.615_029_162_140_6,
        12.507343278686905,
        -0.13857109526572012,
        9.984_369_578_019_572e-6,
        1.5056327351493116e-7,
    ];
    let z = z - 1.0;
    let mut x = COEF[0];
    for (i, &c) in COEF.iter().enumerate().skip(1) {
        x += c / (z + i as f64);
    }
    let t = z + 7.5;
    let res = (2.0 * std::f64::consts::PI).sqrt() * t.powf(z + 0.5) * (-t).exp() * x;
    if res.is_finite() {
        res
    } else {
        f64::INFINITY
    }
}

const CRECIMIENTOS: [CrecimientoDef; 9] = [
    CrecimientoDef {
        clave: "1",
        etiqueta: "O(1)",
        enteras: true,
        eval: |_| 1.0,
    },
    CrecimientoDef {
        clave: "log n",
        etiqueta: "O(log n)",
        enteras: false,
        eval: f64::log2,
    },
    CrecimientoDef {
        clave: "sqrt n",
        etiqueta: "O(√n)",
        enteras: true,
        eval: f64::sqrt,
    },
    CrecimientoDef {
        clave: "n",
        etiqueta: "O(n)",
        enteras: true,
        eval: |n| n,
    },
    CrecimientoDef {
        clave: "n log n",
        etiqueta: "O(n log n)",
        enteras: false,
        eval: |n| n * n.log2(),
    },
    CrecimientoDef {
        clave: "n^2",
        etiqueta: "O(n²)",
        enteras: true,
        eval: |n| n * n,
    },
    CrecimientoDef {
        clave: "n^3",
        etiqueta: "O(n³)",
        enteras: true,
        eval: |n| n * n * n,
    },
    CrecimientoDef {
        clave: "2^n",
        etiqueta: "O(2ⁿ)",
        enteras: false,
        eval: |n| 2.0_f64.powf(n),
    },
    CrecimientoDef {
        clave: "n!",
        etiqueta: "O(n!)",
        enteras: false,
        eval: |n| {
            if n > 171.0 {
                f64::INFINITY
            } else {
                gamma_lanczos(n + 1.0)
            }
        },
    },
];

fn buscar_crecimiento(val: &Value) -> Result<CrecimientoDef, ErrorDiagrama> {
    let raw = texto(val);
    let clave = raw
        .trim()
        .to_lowercase()
        .replace('²', "^2")
        .replace('³', "^3")
        .replace('ⁿ', "^n")
        .replace("√n", "sqrt n");
    let interior = clave
        .strip_prefix("o(")
        .or_else(|| clave.strip_prefix("θ("))
        .or_else(|| clave.strip_prefix("ω("))
        .and_then(|s| s.strip_suffix(')'))
        .unwrap_or(&clave);
    let norm = interior.split_whitespace().collect::<Vec<_>>().join(" ");
    CRECIMIENTOS
        .iter()
        .copied()
        .find(|c| c.clave == norm)
        .ok_or_else(|| {
            ErrorDiagrama::new(format!(
                "crecimiento desconocido: {raw:?} (disponibles: 1, log n, sqrt n, n, n log n, n^2, n^3, 2^n, n!)"
            ))
        })
}

fn fmt_g_py(x: f64) -> String {
    if x == 0.0 {
        return "0".to_string();
    }
    if !x.is_finite() {
        return x.to_string();
    }
    let sci = format!("{:.5e}", x.abs());
    let (mant_part, exp_part) = match sci.split_once('e') {
        Some(p) => p,
        None => return format!("{x}"),
    };
    let exp: i32 = exp_part.parse().unwrap_or(0);
    let digits: String = mant_part.chars().filter(|c| *c != '.').collect();
    let signo = if x < 0.0 { "-" } else { "" };

    if (-4..6).contains(&exp) {
        if exp >= 5 {
            format!("{signo}{digits}")
        } else if exp >= 0 {
            let idx = (exp + 1) as usize;
            let (ent, frac) = digits.split_at(idx.min(digits.len()));
            let frac_trimmed = frac.trim_end_matches('0');
            if frac_trimmed.is_empty() {
                format!("{signo}{ent}")
            } else {
                format!("{signo}{ent}.{frac_trimmed}")
            }
        } else {
            let ceros = (-exp - 1) as usize;
            let full_frac = format!("{}{digits}", "0".repeat(ceros));
            let frac_trimmed = full_frac.trim_end_matches('0');
            format!("{signo}0.{frac_trimmed}")
        }
    } else {
        let first = &digits[..1];
        let rest = digits[1..].trim_end_matches('0');
        let mant = if rest.is_empty() {
            first.to_string()
        } else {
            format!("{first}.{rest}")
        };
        format!("{signo}{mant}e{exp:+03}")
    }
}

fn parsear_num_finito(v: &Value) -> Option<f64> {
    match v {
        Value::Number(n) => n.as_f64().filter(|x| x.is_finite()),
        Value::String(s) => s.trim().parse::<f64>().ok().filter(|x| x.is_finite()),
        _ => None,
    }
}

fn render_complexity(c: &mut Lienzo, b: &Bloque) -> Result<Punto, ErrorDiagrama> {
    match b.get("variant") {
        None => {}
        Some(Value::String(s)) if s == "growth" => {}
        Some(Value::String(s)) if s == "classes" => return render_clases(c, b),
        Some(otro) => {
            return Err(ErrorDiagrama::new(format!(
                "variante de complexity no soportada: {otro:?} (growth o classes)"
            )));
        }
    }

    let defecto_curvas: Vec<Value> = CLASES_COMUNES
        .iter()
        .map(|s| Value::String((*s).to_string()))
        .collect();
    let datos: &[Value] = match b.get("curves") {
        None => &defecto_curvas,
        Some(v) if !es_truthy(v) => &defecto_curvas,
        Some(Value::Array(arr)) if arr.len() <= MAX_CURVAS => arr.as_slice(),
        Some(_) => {
            return Err(ErrorDiagrama::new(format!(
                "'curves' tiene que ser una lista de hasta {MAX_CURVAS} curvas"
            )));
        }
    };

    let max_n = entero_b(b, "maxN", 20, 2, 10000) as f64;
    let max_y = entero_b(b, "maxY", 100, 2, 1_000_000_000) as f64;
    let top = encabezado(c, b, "");
    let x0 = 52.0_f64;
    let w = 420.0_f64;
    let h = 260.0_f64;
    let y0 = top + 38.0;
    let sx = w / max_n;
    let sy = h / max_y;

    let px = |n: f64| -> f64 { x0 + n * sx };
    let py = |valor: f64| -> f64 { y0 + h - valor * sy };

    for k in 0..5 {
        let n = max_n * k as f64 / 4.0;
        let v = max_y * k as f64 / 4.0;
        c.linea(
            px(n),
            y0 + h,
            px(n),
            y0 + h + 4.0,
            "strokeMain",
            1.0,
            None,
            "",
        );
        c.t(
            px(n),
            y0 + h + 16.0,
            &fmt_g_py(n),
            9.0,
            400,
            "textSub",
            "middle",
            MONO,
        );
        c.linea(
            x0 - 4.0,
            py(v),
            x0,
            py(v),
            "strokeMain",
            1.0,
            None,
            "",
        );
        c.t(
            x0 - 7.0,
            py(v) + 3.0,
            &fmt_g_py(v),
            9.0,
            400,
            "textSub",
            "end",
            MONO,
        );
        if k > 0 {
            c.linea(
                x0,
                py(v),
                x0 + w,
                py(v),
                "strokeMuted",
                0.5,
                None,
                "2 4",
            );
        }
    }

    c.linea(
        x0,
        y0 + h,
        x0 + w + 8.0,
        y0 + h,
        "strokeMain",
        1.6,
        Some("main"),
        "",
    );
    c.linea(
        x0,
        y0 + h,
        x0,
        y0 - 8.0,
        "strokeMain",
        1.6,
        Some("main"),
        "",
    );
    let x_label = str_o(b, "xLabel", "n (tamaño de la entrada)");
    let y_label = str_o(b, "yLabel", "operaciones");
    c.t(
        x0 + w / 2.0,
        y0 + h + 32.0,
        &x_label,
        10.0,
        600,
        "textMain",
        "middle",
        MONO,
    );
    c.t(0.0, y0 - 22.0, &y_label, 10.0, 600, "textMain", "start", MONO);

    let blank_labels = b.get("blankLabels").is_some_and(es_truthy);
    let mut derecha: Vec<(f64, String)> = Vec::new();
    let mut arriba: Vec<(f64, String)> = Vec::new();
    let mut sueltas: Vec<(f64, f64, String)> = Vec::new();

    for (i, dato) in datos.iter().enumerate() {
        let letra = char::from(b'A' + (i as u8));
        let mut etiqueta_opt = if blank_labels {
            Some(letra.to_string())
        } else {
            None
        };

        let puntos: Vec<(f64, f64)> = if let Some(obj) = dato.as_object() {
            let pts_val = obj.get("points");
            let mut pts = match pts_val {
                None => Vec::new(),
                Some(v) if !es_truthy(v) => Vec::new(),
                Some(Value::Array(arr)) => {
                    let mut out = Vec::with_capacity(arr.len());
                    for p in arr {
                        let Some(par) = p.as_array() else {
                            return Err(ErrorDiagrama::new(format!(
                                "curva {i}: 'points' tiene que ser una lista de [n, valor]"
                            )));
                        };
                        if par.len() < 2 {
                            return Err(ErrorDiagrama::new(format!(
                                "curva {i}: 'points' tiene que ser una lista de [n, valor]"
                            )));
                        }
                        let Some(pn) = parsear_num_finito(&par[0]) else {
                            return Err(ErrorDiagrama::new(format!(
                                "curva {i}: 'points' tiene que ser una lista de [n, valor]"
                            )));
                        };
                        let Some(pv) = parsear_num_finito(&par[1]) else {
                            return Err(ErrorDiagrama::new(format!(
                                "curva {i}: 'points' tiene que ser una lista de [n, valor]"
                            )));
                        };
                        out.push((pn, pv));
                    }
                    out
                }
                Some(_) => {
                    return Err(ErrorDiagrama::new(format!(
                        "curva {i}: 'points' tiene que ser una lista de [n, valor]"
                    )));
                }
            };
            if pts.is_empty() {
                return Err(ErrorDiagrama::new(format!(
                    "curva {i}: le falta 'points' (o usar un nombre, como n^2)"
                )));
            }
            pts.sort_by(|a, b| a.0.total_cmp(&b.0).then_with(|| a.1.total_cmp(&b.1)));
            if etiqueta_opt.is_none() {
                let lbl = obj
                    .get("label")
                    .filter(|v| es_truthy(v))
                    .map(texto)
                    .unwrap_or_else(|| format!("curva {}", i + 1));
                etiqueta_opt = Some(lbl);
            }
            for &(n, v) in &pts {
                if n <= max_n && v <= max_y {
                    let x = px(n);
                    let y = py(v);
                    if x.is_finite() && y.is_finite() {
                        c.punto(x, y, "accentPrimary", 2.6);
                    }
                }
            }
            pts
        } else {
            let def = buscar_crecimiento(dato)?;
            if etiqueta_opt.is_none() {
                etiqueta_opt = Some(def.etiqueta.to_string());
            }
            let mut pts = Vec::with_capacity(240);
            for k in 1..=240 {
                let n = max_n * k as f64 / 240.0;
                if n >= 1.0 || def.enteras {
                    pts.push((n, (def.eval)(n)));
                }
            }
            pts
        };

        let etiqueta = etiqueta_opt.unwrap_or_default();
        let mut trazo: Vec<(f64, f64)> = Vec::new();
        for (n, v) in puntos {
            if n > max_n {
                break;
            }
            if !px(n).is_finite() {
                continue;
            }
            if v > max_y {
                if let Some(&(n0, v0)) = trazo.last() {
                    let n_corte = if v.is_finite()
                        && (v - v0).is_finite()
                        && (max_y - v0).is_finite()
                        && (v - v0).abs() > 0.0
                        && (n - n0).is_finite()
                    {
                        let cand = n0 + (n - n0) * ((max_y - v0) / (v - v0));
                        if px(cand).is_finite() {
                            cand
                        } else {
                            n0
                        }
                    } else {
                        n0
                    };
                    trazo.push((n_corte, max_y));
                }
                break;
            }
            if !v.is_finite() || !py(v).is_finite() {
                continue;
            }
            trazo.push((n, v));
        }

        let Some(&(n_fin, v_fin)) = trazo.last() else {
            continue;
        };
        let camino: String = trazo
            .iter()
            .map(|&(n, v)| format!("{:.1},{:.1}", px(n), py(v)))
            .collect::<Vec<_>>()
            .join(" ");
        let guiones = GUIONES[i % GUIONES.len()];
        let dash = if guiones.is_empty() {
            String::new()
        } else {
            format!(r#" stroke-dasharray="{guiones}""#)
        };
        c.add(format!(
            r#"<polyline points="{camino}" fill="none" stroke="{}" stroke-width="1.8"{dash}/>"#,
            c.pal.stroke_main
        ));

        if v_fin >= max_y {
            arriba.push((px(n_fin), etiqueta));
        } else if n_fin >= max_n * 0.98 {
            derecha.push((py(v_fin), etiqueta));
        } else {
            sueltas.push((px(n_fin), py(v_fin), etiqueta));
        }
    }

    let borde = x0 + w + 8.0;
    let mut ancho = borde;
    let mut tope = y0 + h + 14.0;
    derecha.sort_by(|a, b| b.0.total_cmp(&a.0).then_with(|| b.1.cmp(&a.1)));
    for (y_fin, etiqueta) in derecha {
        tope = (y_fin + 4.0).min(tope - 14.0);
        c.t(borde, tope, &etiqueta, 10.5, 700, "textMain", "start", MONO);
        ancho = ancho.max(borde + ancho_texto(&etiqueta, 10.5));
    }
    for (x_fin, y_fin, etiqueta) in sueltas {
        c.t(
            x_fin + 8.0,
            y_fin + 4.0,
            &etiqueta,
            10.5,
            700,
            "textMain",
            "start",
            MONO,
        );
        ancho = ancho.max(x_fin + 8.0 + ancho_texto(&etiqueta, 10.5));
    }
    arriba.sort_by(|a, b| a.0.total_cmp(&b.0).then_with(|| a.1.cmp(&b.1)));
    let mut siguiente = 0.0_f64;
    for (x_fin, etiqueta) in arriba {
        let x = x_fin.max(siguiente);
        c.t(x, y0 - 6.0, &etiqueta, 10.5, 700, "textMain", "middle", MONO);
        let w_et = ancho_texto(&etiqueta, 10.5);
        siguiente = x + w_et + 8.0;
        ancho = ancho.max(x + w_et / 2.0);
    }

    Ok((ancho + 4.0, y0 + h + 40.0))
}

fn render_clases(c: &mut Lienzo, b: &Bloque) -> Result<Punto, ErrorDiagrama> {
    let defecto_clases: Vec<Value> = CLASES_COMUNES
        .iter()
        .filter_map(|k| CRECIMIENTOS.iter().find(|c| c.clave == *k))
        .map(|def| Value::String(def.etiqueta.to_string()))
        .collect();
    let datos: &[Value] = match b.get("classes") {
        None => &defecto_clases,
        Some(v) if !es_truthy(v) => &defecto_clases,
        Some(Value::Array(arr)) if !arr.is_empty() && arr.len() <= MAX_CLASES => arr.as_slice(),
        Some(_) => {
            return Err(ErrorDiagrama::new(
                "'classes' tiene que ser una lista de hasta 10 clases, de la más chica a la más grande",
            ));
        }
    };
    let clases: Vec<String> = datos.iter().map(texto).collect();

    let mut contenido: BTreeMap<String, Vec<String>> = BTreeMap::new();
    match b.get("items") {
        None => {}
        Some(v) if !es_truthy(v) => {}
        Some(Value::Object(obj)) => {
            for (clase, v) in obj {
                if !clases.iter().any(|cl| cl == clase) {
                    return Err(ErrorDiagrama::new(format!(
                        "'items' nombra una clase que no está en 'classes': {clase:?}"
                    )));
                }
                let lista = match v {
                    Value::Array(arr) => arr.iter().map(texto).collect(),
                    otro => vec![texto(otro)],
                };
                contenido.insert(clase.clone(), lista);
            }
        }
        Some(_) => {
            return Err(ErrorDiagrama::new(
                "'items' tiene que decir qué va en cada clase: {'O(n)': [búsqueda lineal]}",
            ));
        }
    }

    let sin_clases = b.get("blankClasses").is_some_and(es_truthy);
    let top = encabezado(c, b, "");
    let margen = 14.0_f64;
    let renglon = 15.0_f64;
    let cab = 24.0_f64;

    let mut medidas: Vec<(f64, f64)> = Vec::with_capacity(clases.len());
    let mut interior_w = 0.0_f64;
    let mut interior_h = 0.0_f64;
    for clase in &clases {
        let lista = contenido.get(clase).map(Vec::as_slice).unwrap_or(&[]);
        let mut max_txt = ancho_texto(clase, 11.5);
        for item in lista {
            max_txt = max_txt.max(ancho_texto(&format!("• {item}"), 10.0));
        }
        let texto_w = max_txt + 2.0 * margen;
        let banda = cab + lista.len() as f64 * renglon;
        interior_w = (interior_w + 2.0 * margen).max(texto_w).max(90.0);
        interior_h += banda + if interior_h > 0.0 { margen } else { 6.0 };
        medidas.push((interior_w, interior_h));
    }

    let (ancho_total, alto_total) = medidas.last().copied().unwrap_or((90.0, 30.0));
    let mut y = top + 4.0;
    for (idx_ext, (k, &(w, h))) in medidas.iter().enumerate().rev().enumerate() {
        let x = (ancho_total - w) / 2.0;
        let relleno = if idx_ext % 2 == 0 {
            "fillHeader"
        } else {
            "fillCell"
        };
        c.r(x, y, w, h, relleno, 1.6, 8.0);
        if sin_clases {
            c.vacio(x + margen, x + margen + 70.0, y + 18.0);
        } else {
            c.t(
                x + margen,
                y + 17.0,
                &clases[k],
                11.5,
                700,
                "textMain",
                "start",
                MONO,
            );
        }
        let lista = contenido.get(&clases[k]).map(Vec::as_slice).unwrap_or(&[]);
        for (n, item) in lista.iter().enumerate() {
            c.t(
                x + margen,
                y + cab + 8.0 + n as f64 * renglon,
                &format!("• {item}"),
                10.0,
                400,
                "textMain",
                "start",
                MONO,
            );
        }
        y += cab + lista.len() as f64 * renglon;
    }

    Ok((ancho_total, top + 4.0 + alto_total + 6.0))
}

// ---------------------------------------------------------------------------
// Diagrama de flujo: secuencia, decisión y repetición
// ---------------------------------------------------------------------------

const SEP_FLUJO: f64 = 26.0;
const ALTO_ROMBO: f64 = 60.0;
const MAX_PROF_FLUJO: usize = 64;

enum PasoFlujo {
    Proceso {
        texto: String,
        en_blanco: bool,
    },
    Io {
        texto: String,
        en_blanco: bool,
    },
    Si {
        texto: String,
        a: Vec<PasoFlujo>,
        b: Vec<PasoFlujo>,
        en_blanco: bool,
    },
    Mientras {
        texto: String,
        a: Vec<PasoFlujo>,
        en_blanco: bool,
    },
}

impl PasoFlujo {
    fn texto(&self) -> &str {
        match self {
            Self::Proceso { texto, .. }
            | Self::Io { texto, .. }
            | Self::Si { texto, .. }
            | Self::Mientras { texto, .. } => texto,
        }
    }

    fn en_blanco(&self) -> bool {
        match self {
            Self::Proceso { en_blanco, .. }
            | Self::Io { en_blanco, .. }
            | Self::Si { en_blanco, .. }
            | Self::Mientras { en_blanco, .. } => *en_blanco,
        }
    }
}

#[derive(Clone, Copy)]
struct MedidaFlujo {
    izq: f64,
    der: f64,
    alto: f64,
}

fn parsear_pasos_flujo(
    dato: Option<&Value>,
    donde: &str,
    prof: usize,
) -> Result<Vec<PasoFlujo>, ErrorDiagrama> {
    if prof > MAX_PROF_FLUJO {
        return Err(ErrorDiagrama::new(
            "el diagrama de flujo excede la profundidad máxima permitida",
        ));
    }
    let Some(v) = dato else {
        return Ok(Vec::new());
    };
    if v.is_null() {
        return Ok(Vec::new());
    }
    let Some(lista) = v.as_array() else {
        return Err(ErrorDiagrama::new(format!(
            "{donde}: los pasos van en una lista"
        )));
    };
    let mut pasos = Vec::with_capacity(lista.len());
    for (n, p) in lista.iter().enumerate() {
        let lugar = format!("{donde}, paso {n}");
        let Some(obj) = p.as_object() else {
            pasos.push(PasoFlujo::Proceso {
                texto: texto(p),
                en_blanco: false,
            });
            continue;
        };
        let en_blanco = obj.get("blank").is_some_and(es_truthy);
        if let Some(cond) = obj.get("if") {
            let entonces = parsear_pasos_flujo(obj.get("then"), &lugar, prof + 1)?;
            let sino = parsear_pasos_flujo(obj.get("else"), &lugar, prof + 1)?;
            pasos.push(PasoFlujo::Si {
                texto: texto(cond),
                a: entonces,
                b: sino,
                en_blanco,
            });
        } else if obj.contains_key("while") || obj.contains_key("for") {
            let cuerpo = parsear_pasos_flujo(obj.get("do"), &lugar, prof + 1)?;
            if cuerpo.is_empty() {
                return Err(ErrorDiagrama::new(format!(
                    "{lugar}: la repetición no tiene cuerpo ('do')"
                )));
            }
            let cond = obj
                .get("while")
                .or_else(|| obj.get("for"))
                .map(texto)
                .unwrap_or_default();
            pasos.push(PasoFlujo::Mientras {
                texto: cond,
                a: cuerpo,
                en_blanco,
            });
        } else if let Some(io_v) = obj.get("io") {
            pasos.push(PasoFlujo::Io {
                texto: texto(io_v),
                en_blanco,
            });
        } else if let Some(step_v) = obj.get("step") {
            pasos.push(PasoFlujo::Proceso {
                texto: texto(step_v),
                en_blanco,
            });
        } else {
            return Err(ErrorDiagrama::new(format!(
                "{lugar}: se espera un texto o una clave if, while, for, io o step"
            )));
        }
    }
    Ok(pasos)
}

fn ancho_caja_flujo(p: &PasoFlujo) -> f64 {
    (ancho_texto(p.texto(), 10.5) + 26.0).max(110.0)
}

fn ancho_rombo_flujo(p: &PasoFlujo) -> f64 {
    (ancho_texto(p.texto(), 10.5) * 1.7 + 30.0).max(130.0)
}

fn medir_lista_flujo(pasos: &[PasoFlujo]) -> MedidaFlujo {
    if pasos.is_empty() {
        return MedidaFlujo {
            izq: 0.0,
            der: 0.0,
            alto: 0.0,
        };
    }
    let mut izq = 0.0_f64;
    let mut der = 0.0_f64;
    let mut suma_alto = 0.0_f64;
    for p in pasos {
        let m = medir_paso_flujo(p);
        izq = izq.max(m.izq);
        der = der.max(m.der);
        suma_alto += m.alto;
    }
    MedidaFlujo {
        izq,
        der,
        alto: suma_alto + SEP_FLUJO * (pasos.len() - 1) as f64,
    }
}

fn desvios_flujo(p: &PasoFlujo) -> (f64, f64) {
    let medio = ancho_rombo_flujo(p) / 2.0;
    match p {
        PasoFlujo::Si { a, b, .. } => {
            let ma = medir_lista_flujo(a);
            let mb = medir_lista_flujo(b);
            ((medio + 12.0).max(ma.der + 16.0), (medio + 12.0).max(mb.izq + 16.0))
        }
        _ => (medio + 12.0, medio + 12.0),
    }
}

fn medir_paso_flujo(p: &PasoFlujo) -> MedidaFlujo {
    match p {
        PasoFlujo::Proceso { .. } => {
            let w = ancho_caja_flujo(p);
            MedidaFlujo {
                izq: w / 2.0,
                der: w / 2.0,
                alto: 34.0,
            }
        }
        PasoFlujo::Io { .. } => {
            let w = ancho_caja_flujo(p) + 24.0;
            MedidaFlujo {
                izq: w / 2.0,
                der: w / 2.0,
                alto: 34.0,
            }
        }
        PasoFlujo::Mientras { a, .. } => {
            let medio = ancho_rombo_flujo(p) / 2.0;
            let m = medir_lista_flujo(a);
            MedidaFlujo {
                izq: medio.max(m.izq) + 22.0,
                der: medio.max(m.der) + 34.0,
                alto: ALTO_ROMBO + SEP_FLUJO + m.alto + 34.0,
            }
        }
        PasoFlujo::Si { a, b, .. } => {
            let ma = medir_lista_flujo(a);
            let mb = medir_lista_flujo(b);
            let (dx_a, dx_b) = desvios_flujo(p);
            MedidaFlujo {
                izq: dx_a + ma.izq,
                der: dx_b + mb.der,
                alto: ALTO_ROMBO + SEP_FLUJO + ma.alto.max(mb.alto) + SEP_FLUJO,
            }
        }
    }
}

fn texto_flujo(c: &mut Lienzo, p: &PasoFlujo, x: f64, y: f64, sin_valores: bool) {
    if !p.texto().is_empty() && !(p.en_blanco() || sin_valores) {
        c.t(x, y + 4.0, p.texto(), 10.5, 600, "textMain", "middle", MONO);
    } else {
        c.vacio(x - 32.0, x + 32.0, y + 6.0);
    }
}

fn camino_flujo(c: &mut Lienzo, puntos: &[Punto], flecha: bool) {
    if puntos.is_empty() {
        return;
    }
    let trazo: String = puntos
        .iter()
        .map(|&(x, y)| format!("{x:.1} {y:.1}"))
        .collect::<Vec<_>>()
        .join(" L ");
    let marca = if flecha {
        format!(r#" marker-end="url(#flecha-main-{})""#, c.sufijo)
    } else {
        String::new()
    };
    c.add(format!(
        r#"<path d="M {trazo}" fill="none" stroke="{}" stroke-width="1.5"{marca}/>"#,
        c.pal.stroke_main
    ));
}

fn rombo_flujo(c: &mut Lienzo, p: &PasoFlujo, x: f64, y: f64, sin_valores: bool) {
    let medio = ancho_rombo_flujo(p) / 2.0;
    let alto = ALTO_ROMBO;
    c.add(format!(
        r#"<polygon points="{:.1},{:.1} {:.1},{:.1} {:.1},{:.1} {:.1},{:.1}" fill="{}" stroke="{}" stroke-width="1.6"/>"#,
        x,
        y,
        x + medio,
        y + alto / 2.0,
        x,
        y + alto,
        x - medio,
        y + alto / 2.0,
        c.pal.fill_header,
        c.pal.stroke_main
    ));
    texto_flujo(c, p, x, y + alto / 2.0, sin_valores);
}

fn dibujar_lista_flujo(
    c: &mut Lienzo,
    pasos: &[PasoFlujo],
    x: f64,
    mut y: f64,
    sin_valores: bool,
) -> f64 {
    for (i, p) in pasos.iter().enumerate() {
        if i > 0 {
            c.linea(
                x,
                y,
                x,
                y + SEP_FLUJO - 3.0,
                "strokeMain",
                1.5,
                Some("main"),
                "",
            );
            y += SEP_FLUJO;
        }
        y = dibujar_paso_flujo(c, p, x, y, sin_valores);
    }
    y
}

fn dibujar_paso_flujo(
    c: &mut Lienzo,
    p: &PasoFlujo,
    x: f64,
    y: f64,
    sin_valores: bool,
) -> f64 {
    match p {
        PasoFlujo::Proceso { .. } => {
            let w = ancho_caja_flujo(p);
            c.r(x - w / 2.0, y, w, 34.0, "fillCell", 1.6, 0.0);
            texto_flujo(c, p, x, y + 17.0, sin_valores);
            y + 34.0
        }
        PasoFlujo::Io { .. } => {
            let w = ancho_caja_flujo(p) + 24.0;
            let inc = 12.0;
            c.add(format!(
                r#"<polygon points="{:.1},{:.1} {:.1},{:.1} {:.1},{:.1} {:.1},{:.1}" fill="{}" stroke="{}" stroke-width="1.6"/>"#,
                x - w / 2.0 + inc,
                y,
                x + w / 2.0,
                y,
                x + w / 2.0 - inc,
                y + 34.0,
                x - w / 2.0,
                y + 34.0,
                c.pal.fill_cell,
                c.pal.stroke_main
            ));
            texto_flujo(c, p, x, y + 17.0, sin_valores);
            y + 34.0
        }
        PasoFlujo::Mientras { a, .. } => {
            let medio = ancho_rombo_flujo(p) / 2.0;
            let medio_y = y + ALTO_ROMBO / 2.0;
            let cuerpo_y = y + ALTO_ROMBO + SEP_FLUJO;
            let m = medir_paso_flujo(p);
            rombo_flujo(c, p, x, y, sin_valores);
            c.linea(
                x,
                y + ALTO_ROMBO,
                x,
                cuerpo_y - 3.0,
                "strokeMain",
                1.5,
                Some("main"),
                "",
            );
            c.t(
                x + 6.0,
                y + ALTO_ROMBO + 15.0,
                "Sí",
                9.5,
                700,
                "textMain",
                "start",
                SANS,
            );
            let fin = dibujar_lista_flujo(c, a, x, cuerpo_y, sin_valores);
            let vuelta = fin + 12.0;
            let salida = fin + 34.0;
            camino_flujo(
                c,
                &[
                    (x, fin),
                    (x, vuelta),
                    (x - m.izq + 4.0, vuelta),
                    (x - m.izq + 4.0, y - 11.0),
                    (x - 3.0, y - 11.0),
                ],
                true,
            );
            c.t(
                x + medio + 4.0,
                medio_y - 6.0,
                "No",
                9.5,
                700,
                "textMain",
                "start",
                SANS,
            );
            camino_flujo(
                c,
                &[
                    (x + medio, medio_y),
                    (x + m.der - 6.0, medio_y),
                    (x + m.der - 6.0, salida),
                    (x, salida),
                ],
                false,
            );
            salida
        }
        PasoFlujo::Si { a, b, .. } => {
            let medio = ancho_rombo_flujo(p) / 2.0;
            let medio_y = y + ALTO_ROMBO / 2.0;
            let cuerpo_y = y + ALTO_ROMBO + SEP_FLUJO;
            let (dx_a, dx_b) = desvios_flujo(p);
            rombo_flujo(c, p, x, y, sin_valores);
            let fin_a = if a.is_empty() {
                medio_y
            } else {
                dibujar_lista_flujo(c, a, x - dx_a, cuerpo_y, sin_valores)
            };
            let fin_b = if b.is_empty() {
                medio_y
            } else {
                dibujar_lista_flujo(c, b, x + dx_b, cuerpo_y, sin_valores)
            };
            let union = fin_a.max(fin_b).max(cuerpo_y) + SEP_FLUJO;
            c.t(
                x - medio - 4.0,
                medio_y - 6.0,
                "Sí",
                9.5,
                700,
                "textMain",
                "end",
                SANS,
            );
            c.t(
                x + medio + 4.0,
                medio_y - 6.0,
                "No",
                9.5,
                700,
                "textMain",
                "start",
                SANS,
            );

            if a.is_empty() {
                camino_flujo(
                    c,
                    &[
                        (x - medio, medio_y),
                        (x - dx_a, medio_y),
                        (x - dx_a, union),
                        (x - 3.0, union),
                    ],
                    true,
                );
            } else {
                camino_flujo(
                    c,
                    &[
                        (x - medio, medio_y),
                        (x - dx_a, medio_y),
                        (x - dx_a, cuerpo_y - 2.0),
                    ],
                    true,
                );
                camino_flujo(
                    c,
                    &[(x - dx_a, fin_a), (x - dx_a, union), (x - 3.0, union)],
                    true,
                );
            }

            if b.is_empty() {
                camino_flujo(
                    c,
                    &[
                        (x + medio, medio_y),
                        (x + dx_b, medio_y),
                        (x + dx_b, union),
                        (x + 3.0, union),
                    ],
                    true,
                );
            } else {
                camino_flujo(
                    c,
                    &[
                        (x + medio, medio_y),
                        (x + dx_b, medio_y),
                        (x + dx_b, cuerpo_y - 2.0),
                    ],
                    true,
                );
                camino_flujo(
                    c,
                    &[(x + dx_b, fin_b), (x + dx_b, union), (x + 3.0, union)],
                    true,
                );
            }

            union
        }
    }
}

fn etiqueta_terminal(b: &Bloque, clave: &str, defecto: &str) -> Option<String> {
    match b.get(clave) {
        None => Some(defecto.to_string()),
        Some(v) if es_truthy(v) => Some(texto(v)),
        Some(_) => None,
    }
}

fn render_flowchart(c: &mut Lienzo, b: &Bloque) -> Result<Punto, ErrorDiagrama> {
    let pasos_val = lista_de(b, "steps", "un diagrama de flujo")?;
    let pasos = parsear_pasos_flujo(
        Some(&Value::Array(pasos_val.to_vec())),
        "diagrama de flujo",
        0,
    )?;
    let top = encabezado(c, b, "");
    let sin_valores = b.get("blankValues").is_some_and(es_truthy);
    let inicio = etiqueta_terminal(b, "start", "Inicio");
    let fin = etiqueta_terminal(b, "end", "Fin");
    let m = medir_lista_flujo(&pasos);
    let x = m.izq.max(60.0) + 4.0;
    let mut y = top + 6.0;

    let terminal = |c: &mut Lienzo, txt: &str, ty: f64| {
        let w = (ancho_texto(txt, 10.5) + 30.0).max(80.0);
        c.r(x - w / 2.0, ty, w, 30.0, "fillHeader", 1.6, 15.0);
        c.t(x, ty + 19.0, txt, 10.5, 700, "textMain", "middle", MONO);
    };

    if let Some(ref txt) = inicio {
        terminal(c, txt, y);
        c.linea(
            x,
            y + 30.0,
            x,
            y + 30.0 + SEP_FLUJO - 3.0,
            "strokeMain",
            1.5,
            Some("main"),
            "",
        );
        y += 30.0 + SEP_FLUJO;
    }
    y = dibujar_lista_flujo(c, &pasos, x, y, sin_valores);
    if let Some(ref txt) = fin {
        c.linea(
            x,
            y,
            x,
            y + SEP_FLUJO - 3.0,
            "strokeMain",
            1.5,
            Some("main"),
            "",
        );
        y += SEP_FLUJO;
        terminal(c, txt, y);
        y += 30.0;
    }

    Ok((x + m.der.max(60.0) + 4.0, y + 4.0))
}

// ---------------------------------------------------------------------------
// Análisis de algoritmos: tabla de costos por línea
// ---------------------------------------------------------------------------

fn render_cost_table(c: &mut Lienzo, b: &Bloque) -> Result<Punto, ErrorDiagrama> {
    let lineas = lista_de(b, "lines", "una tabla de costos")?;
    let sin_costos = b.get("blankCosts").is_some_and(es_truthy);
    let sin_veces = b.get("blankTimes").is_some_and(es_truthy);

    let mut filas = Vec::with_capacity(lineas.len());
    for (idx, linea) in lineas.iter().enumerate() {
        let n = idx + 1;
        let (cod, costo, veces) = if let Some(obj) = linea.as_object() {
            (
                obj.get("code").map(texto).unwrap_or_default(),
                obj.get("cost").map(texto).unwrap_or_default(),
                obj.get("times").map(texto).unwrap_or_default(),
            )
        } else if let Some(arr) = linea.as_array() {
            if (1..=3).contains(&arr.len()) {
                (
                    arr.first().map(texto).unwrap_or_default(),
                    arr.get(1).map(texto).unwrap_or_default(),
                    arr.get(2).map(texto).unwrap_or_default(),
                )
            } else {
                (texto(linea), String::new(), String::new())
            }
        } else {
            (texto(linea), String::new(), String::new())
        };

        filas.push(vec![
            n.to_string(),
            cod,
            if sin_costos { String::new() } else { costo },
            if sin_veces { String::new() } else { veces },
        ]);
    }

    let top = encabezado(c, b, "");
    let (mut ancho, alto) = tabla(
        c,
        0.0,
        top + 6.0,
        &["#", "código", "costo", "veces"],
        &filas,
        TablaOpts {
            izquierda: &[1],
            min_w: 30.0,
            ..TablaOpts::default()
        },
    );

    let mut y = top + 6.0 + alto;
    let total = b.get("total").filter(|v| !v.is_null());
    let blank_total = b.get("blankTotal").is_some_and(es_truthy);
    if total.is_some() || blank_total {
        y += 22.0;
        let sufijo = if blank_total {
            "_".repeat(24)
        } else {
            total.map(texto).unwrap_or_default()
        };
        let msg = format!("T(n) = {sufijo}");
        c.t(0.0, y, &msg, 12.0, 700, "textMain", "start", MONO);
        ancho = ancho.max(ancho_texto(&msg, 12.0));
    }

    Ok((ancho, y + 8.0))
}

// ---------------------------------------------------------------------------
// Tabla de traza: variables o statics (pila / cola)
// ---------------------------------------------------------------------------

fn render_trace_table(c: &mut Lienzo, b: &Bloque) -> Result<Punto, ErrorDiagrama> {
    let operaciones: Vec<String> = lista_o_vacia(b, "operations")?
        .iter()
        .map(texto)
        .collect();
    let top = encabezado(c, b, "");

    if let Some(cols_val) = b.get("columns").filter(|v| !v.is_null()) {
        let Some(cols_arr) = cols_val.as_array().filter(|a| !a.is_empty()) else {
            return Err(ErrorDiagrama::new(
                "'columns' de una traza tiene que ser una lista con al menos una columna",
            ));
        };
        let mut encabezados: Vec<String> = cols_arr.iter().map(texto).collect();
        let filas_dato: &[Value] = match b.get("rows") {
            None => &[],
            Some(v) if !es_truthy(v) => &[],
            Some(Value::Array(arr)) => {
                if !arr.iter().all(Value::is_array) {
                    return Err(ErrorDiagrama::new(
                        "'rows' de una traza tiene que ser una lista de filas",
                    ));
                }
                arr.as_slice()
            }
            Some(_) => {
                return Err(ErrorDiagrama::new(
                    "'rows' de una traza tiene que ser una lista de filas",
                ));
            }
        };

        let sin_valores = b.get("blankValues").is_some_and(es_truthy);
        let row_count = entero_b(b, "rowCount", 0, 0, 30) as usize;
        let cantidad = filas_dato.len().max(operaciones.len()).max(row_count);
        if cantidad == 0 {
            return Err(ErrorDiagrama::new(
                "la traza no tiene filas (usar rows, operations o rowCount)",
            ));
        }

        let num_cols = encabezados.len();
        let mut filas = Vec::with_capacity(cantidad);
        for i in 0..cantidad {
            let dato: &[Value] = filas_dato
                .get(i)
                .and_then(Value::as_array)
                .map(Vec::as_slice)
                .unwrap_or(&[]);
            if dato.len() > num_cols {
                return Err(ErrorDiagrama::new(format!(
                    "la fila {i} de la traza tiene {} valores y hay {num_cols} columnas",
                    dato.len()
                )));
            }
            let mut fila = Vec::with_capacity(num_cols + usize::from(!operaciones.is_empty()));
            if !operaciones.is_empty() {
                fila.push(operaciones.get(i).cloned().unwrap_or_default());
            }
            for v in dato {
                fila.push(if sin_valores { String::new() } else { texto(v) });
            }
            while fila.len() < num_cols + usize::from(!operaciones.is_empty()) {
                fila.push(String::new());
            }
            filas.push(fila);
        }

        let izquierda: &[usize] = if !operaciones.is_empty() {
            encabezados.insert(0, str_o(b, "operationHeader", "Instrucción"));
            &[0]
        } else {
            &[]
        };
        let enc_refs: Vec<&str> = encabezados.iter().map(String::as_str).collect();
        let (ancho, alto) = tabla(
            c,
            0.0,
            top + 6.0,
            &enc_refs,
            &filas,
            TablaOpts {
                izquierda,
                ..TablaOpts::default()
            },
        );
        return Ok((ancho, top + 6.0 + alto + 8.0));
    }

    if operaciones.is_empty() {
        return Err(ErrorDiagrama::new(
            "la traza no tiene filas (usar operations, o columns y rows)",
        ));
    }

    let cap = entero_b(b, "capacity", 6, 3, 10) as usize;
    let pila = b.get("variant").and_then(Value::as_str) == Some("stack");
    let puntero = str_o(
        b,
        "pointerHeader",
        if pila {
            "Puntero TOPE"
        } else {
            "Punteros / Índ."
        },
    );
    let marca = str_o(
        b,
        "pointerLabel",
        if pila { "TOPE: ___" } else { "F:___ R:___" },
    );
    let mostrar_indices = b
        .get("showIndexHeader")
        .map(es_truthy)
        .unwrap_or(true);
    let indices: Vec<String> = if mostrar_indices {
        (0..cap).map(|i| format!("[{i}]")).collect()
    } else {
        vec![" ".to_string(); cap]
    };

    let mut encabezados: Vec<String> = Vec::with_capacity(cap + 2);
    encabezados.push("Operación / Instrucción C".to_string());
    encabezados.extend(indices);
    encabezados.push(puntero);

    let mut filas = Vec::with_capacity(operaciones.len());
    for op in operaciones {
        let mut fila = Vec::with_capacity(cap + 2);
        fila.push(op);
        fila.extend(std::iter::repeat_n(String::new(), cap));
        fila.push(marca.clone());
        filas.push(fila);
    }

    let enc_refs: Vec<&str> = encabezados.iter().map(String::as_str).collect();
    let izq_cols = [0, cap + 1];
    let (ancho, alto) = tabla(
        c,
        0.0,
        top + 6.0,
        &enc_refs,
        &filas,
        TablaOpts {
            izquierda: &izq_cols,
            fila_h: 36.0,
            min_w: 40.0,
            ..TablaOpts::default()
        },
    );
    Ok((ancho, top + 6.0 + alto + 8.0))
}

// ---------------------------------------------------------------------------
// Tests unitarios
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use dibu_core::leer_bloques;
    use serde_json::json;

    fn crear_registro() -> Registro {
        let mut reg = Registro::new();
        registrar(&mut reg);
        reg
    }

    fn render_val(val: Value, tinta: bool) -> Result<String, ErrorDiagrama> {
        let reg = crear_registro();
        let bloque = val.as_object().expect("objeto JSON en test");
        reg.renderizar(bloque, tinta)
    }

    fn extraer_attr_f64(svg: &str, attr: &str) -> f64 {
        let pat = format!(r#"{attr}=""#);
        let idx = svg.find(&pat).expect("atributo presente");
        let rest = &svg[idx + pat.len()..];
        let fin = rest.find('"').expect("cierre comilla");
        rest[..fin].parse::<f64>().expect("número válido")
    }

    fn extraer_textos(svg: &str) -> Vec<String> {
        let mut out = Vec::new();
        let mut rest = svg;
        while let Some(idx) = rest.find("<text ") {
            let tras_abrir = &rest[idx..];
            let Some(gt) = tras_abrir.find('>') else {
                break;
            };
            let contenido_inicio = &tras_abrir[gt + 1..];
            let Some(cierre) = contenido_inicio.find("</text>") else {
                break;
            };
            out.push(
                contenido_inicio[..cierre]
                    .replace("&lt;", "<")
                    .replace("&gt;", ">")
                    .replace("&quot;", "\"")
                    .replace("&#x27;", "'")
                    .replace("&amp;", "&"),
            );
            rest = &contenido_inicio[cierre + 7..];
        }
        out
    }

    #[test]
    fn registra_los_seis_tipos() {
        let reg = crear_registro();
        for tipo in [
            "struct",
            "adt",
            "complexity",
            "flowchart",
            "cost_table",
            "trace_table",
        ] {
            assert!(reg.contiene(tipo), "falta tipo {tipo}");
        }
    }

    #[test]
    fn disposicion_struct_coincide_con_los_6_casos_gcc() {
        struct CasoGcc<'a> {
            decls: &'a [&'a str],
            sizeof_esp: usize,
            offsets_esp: &'a [usize],
            rellenos_esp: &'a [(usize, usize)],
            align_esp: usize,
        }

        let casos = [
            CasoGcc {
                decls: &["char a", "int b", "char c"],
                sizeof_esp: 12,
                offsets_esp: &[0, 4, 8],
                rellenos_esp: &[(1, 3), (9, 3)],
                align_esp: 4,
            },
            CasoGcc {
                decls: &["char a", "double d", "short s"],
                sizeof_esp: 24,
                offsets_esp: &[0, 8, 16],
                rellenos_esp: &[(1, 7), (18, 6)],
                align_esp: 8,
            },
            CasoGcc {
                decls: &["char n[10]", "int *p", "short m[3]"],
                sizeof_esp: 32,
                offsets_esp: &[0, 16, 24],
                rellenos_esp: &[(10, 6), (30, 2)],
                align_esp: 8,
            },
            CasoGcc {
                decls: &["short a", "char b", "long c", "char d"],
                sizeof_esp: 24,
                offsets_esp: &[0, 2, 8, 16],
                rellenos_esp: &[(3, 5), (17, 7)],
                align_esp: 8,
            },
            CasoGcc {
                decls: &["float f", "char c[5]", "size_t t"],
                sizeof_esp: 24,
                offsets_esp: &[0, 4, 16],
                rellenos_esp: &[(9, 7)],
                align_esp: 8,
            },
            CasoGcc {
                decls: &["char a", "char b"],
                sizeof_esp: 2,
                offsets_esp: &[0, 1],
                rellenos_esp: &[],
                align_esp: 1,
            },
        ];

        for c in casos {
            let mut campos = campos_struct_desde_strs(c.decls).unwrap();
            let disp = disposicion_struct(&mut campos, false);
            let offsets: Vec<usize> = campos.iter().map(|f| f.offset).collect();
            assert_eq!(disp.sizeof, c.sizeof_esp, "caso {:?}", c.decls);
            assert_eq!(offsets.as_slice(), c.offsets_esp, "offsets {:?}", c.decls);
            assert_eq!(
                disp.rellenos.as_slice(),
                c.rellenos_esp,
                "rellenos {:?}",
                c.decls
            );
            assert_eq!(disp.alineacion, c.align_esp, "align {:?}", c.decls);
        }
    }

    #[test]
    fn disposicion_struct_packed_punteros_y_modificadores() {
        let mut campos = campos_struct_desde_strs(&["char a", "int b", "char c"]).unwrap();
        let packed = disposicion_struct(&mut campos, true);
        assert_eq!(packed.rellenos, vec![]);
        assert_eq!(packed.sizeof, 6);

        let mut mod_campos = campos_struct_desde_strs(&[
            "char nombre[10]",
            "int*p",
            "const unsigned long x",
            "short m[2][3]",
        ])
        .unwrap();
        assert_eq!(mod_campos[1].declaracion(), "int *p");
        let disp = disposicion_struct(&mut mod_campos, false);
        let offs: Vec<usize> = mod_campos.iter().map(|c| c.offset).collect();
        assert_eq!(offs, vec![0, 16, 24, 32]);
        assert_eq!(disp.sizeof, 48);

        let mut u_campos =
            campos_struct_desde_strs(&["unsigned char a", "unsigned b", "long long int c"]).unwrap();
        disposicion_struct(&mut u_campos, false);
        let offs_u: Vec<usize> = u_campos.iter().map(|c| c.offset).collect();
        assert_eq!(offs_u, vec![0, 4, 8]);

        let c_ptr = CampoStruct::desde_str("char* *pp").unwrap();
        assert_eq!(c_ptr.tipo, "char* *");
        assert_eq!(c_ptr.declaracion(), "char* *pp");
    }

    #[test]
    fn renderiza_los_seis_tipos_y_paletas() {
        let casos = [
            json!({"type": "struct", "name": "struct persona", "typedef": "persona_t", "fields": ["char c = 'A'", "int edad = 30"]}),
            json!({"type": "struct", "variant": "opaque", "name": "struct pila", "operations": ["void f(pila_t p)"], "fields": ["int *datos", "size_t tope"]}),
            json!({"type": "adt", "name": "Pila", "operations": ["pila_t crear(void)"], "client": ["crear();"], "invariant": "tope >= 0", "representations": ["Arreglo", {"name": "Lista", "fields": ["nodo_t *cabeza"]}]}),
            json!({"type": "complexity"}),
            json!({"type": "complexity", "variant": "classes", "items": {"O(n)": ["búsqueda lineal"]}}),
            json!({"type": "flowchart", "steps": [
                {"io": "leer n"},
                "i = 0",
                {"while": "i < n", "do": [{"if": "par(i)", "then": ["s = s + i"], "else": ["s = s - i"]}, "i = i + 1"]},
                {"if": "s > 0", "else": ["s = 0"]},
                {"if": "s == 0", "then": ["t = 1"]}
            ]}),
            json!({"type": "cost_table", "lines": [["int s = 0;", "c1", 1]], "total": "c1"}),
            json!({"type": "trace_table", "columns": ["i", "s"], "rows": [[0, 1]]}),
            json!({"type": "trace_table", "capacity": 4, "operations": ["push(1)"]}),
        ];

        for caso in casos {
            let svg_tinta = render_val(caso.clone(), true).unwrap();
            let svg_color = render_val(caso, false).unwrap();
            assert!(svg_tinta.starts_with("<svg ") && svg_tinta.ends_with("</svg>"));
            assert!(extraer_attr_f64(&svg_tinta, "width") > 50.0);
            assert!(extraer_attr_f64(&svg_tinta, "height") > 50.0);
            assert!(!svg_tinta.contains("#1d4ed8"));
            assert!(svg_color.contains("#1d4ed8"));
        }
    }

    #[test]
    fn struct_layout_y_opaque_detalles() {
        let svg = render_val(
            json!({"type": "struct", "name": "struct s", "fields": ["char a", "int b"]}),
            true,
        )
        .unwrap();
        let txts = extraer_textos(&svg);
        assert!(txts.contains(&"+4".to_string()));
        assert!(txts.contains(&"(relleno)".to_string()));
        assert!(txts.contains(&"3 B".to_string()));
        assert!(txts.iter().any(|t| t.contains("sizeof(struct s) = 8 bytes")));

        let svg_blank = render_val(
            json!({
                "type": "struct",
                "name": "struct s",
                "fields": ["char a", "int b"],
                "blankOffsets": true,
                "blankSizes": true,
                "blankPadding": true,
                "blankSizeof": true,
                "blankValues": true
            }),
            true,
        )
        .unwrap();
        let txts_b = extraer_textos(&svg_blank);
        assert!(!txts_b.contains(&"+4".to_string()));
        assert!(!txts_b.contains(&"(relleno)".to_string()));
        assert!(!txts_b.contains(&"4 B".to_string()));
        assert!(!txts_b.iter().any(|t| t.contains("8 bytes")));

        let op = render_val(
            json!({"type": "struct", "variant": "opaque", "name": "struct pila", "typedef": "pila_t", "fields": ["int *datos"]}),
            true,
        )
        .unwrap();
        assert!(extraer_textos(&op).contains(&"int *datos;".to_string()));

        let op_hidden = render_val(
            json!({"type": "struct", "variant": "opaque", "name": "struct pila", "typedef": "pila_t", "hidden": true}),
            true,
        )
        .unwrap();
        let op_txts = extraer_textos(&op_hidden);
        assert!(!op_txts.contains(&"int *datos;".to_string()));
        assert!(op_txts.contains(&"typedef struct pila *pila_t;".to_string()));
    }

    #[test]
    fn adt_detalles_y_resiliencia() {
        let svg = render_val(
            json!({
                "type": "adt",
                "name": "Pila",
                "operations": [
                    {"signature": "f()", "kind": "observador", "pre": "p"},
                    {"signature": "g()", "post": "q", "blank": true}
                ]
            }),
            true,
        )
        .unwrap();
        let txts = extraer_textos(&svg);
        assert!(txts.contains(&"f()".to_string()));
        assert!(!txts.contains(&"g()".to_string()));
        assert!(txts.contains(&"—".to_string()));

        let svg_bk = render_val(
            json!({
                "type": "adt",
                "name": "Pila",
                "blankKinds": true,
                "operations": [{"signature": "f()", "kind": "observador"}]
            }),
            true,
        )
        .unwrap();
        assert!(!extraer_textos(&svg_bk).contains(&"observador".to_string()));
        assert!(render_val(json!({"type": "adt", "operations": [1, 2], "representations": 5}), true).is_err());
    }

    #[test]
    fn complexity_recorte_curvas_y_clases() {
        let alias = render_val(json!({"type": "complexity", "curves": ["Θ(n²)"]}), true).unwrap();
        assert!(extraer_textos(&alias).contains(&"O(n²)".to_string()));

        let blk = render_val(
            json!({"type": "complexity", "curves": ["n", "n^2"], "blankLabels": true}),
            true,
        )
        .unwrap();
        let txts = extraer_textos(&blk);
        assert!(txts.contains(&"A".to_string()) && txts.contains(&"B".to_string()));
        assert!(!txts.contains(&"O(n)".to_string()));

        // maxN=10000 sin overflow/NaN
        let grande = render_val(
            json!({"type": "complexity", "curves": ["2^n", "n!", "n^3"], "maxN": 10000, "maxY": 1000}),
            true,
        )
        .unwrap();
        assert!(!grande.contains("NaN") && !grande.contains("inf"));

        // Recorte exacto en maxY: min(y) >= 37
        let recorte = render_val(
            json!({"type": "complexity", "curves": ["2^n"], "maxN": 20, "maxY": 100}),
            true,
        )
        .unwrap();
        let ini = recorte.find(r#"points=""#).unwrap() + 8;
        let fin = recorte[ini..].find('"').unwrap();
        let ys: Vec<f64> = recorte[ini..ini + fin]
            .split(' ')
            .map(|p| p.split(',').nth(1).unwrap().parse::<f64>().unwrap())
            .collect();
        let min_y = ys.into_iter().fold(f64::INFINITY, f64::min);
        assert!(min_y >= 37.0, "min_y={min_y}");

        // Mediciones empíricas
        let med = render_val(
            json!({"type": "complexity", "curves": [{"label": "medido", "points": [[2, 5], [10, 90]]}]}),
            true,
        )
        .unwrap();
        assert!(extraer_textos(&med).contains(&"medido".to_string()));
        assert_eq!(med.matches("<circle").count(), 2);

        // Clases de afuera hacia adentro y escalar en items
        let cl = render_val(
            json!({"type": "complexity", "variant": "classes", "classes": ["P", "NP"], "items": {"P": ["ordenar"]}}),
            true,
        )
        .unwrap();
        let cl_t = extraer_textos(&cl);
        let i_np = cl_t.iter().position(|t| t == "NP").unwrap();
        let i_p = cl_t.iter().position(|t| t == "P").unwrap();
        assert!(i_np < i_p);
        assert!(cl_t.contains(&"• ordenar".to_string()));

        let cl_esc = render_val(
            json!({"type": "complexity", "variant": "classes", "classes": ["P"], "items": {"P": 3}, "maxN": []}),
            true,
        )
        .unwrap();
        assert!(extraer_textos(&cl_esc).contains(&"• 3".to_string()));

        assert_eq!(fmt_g_py(1_000_000_000.0), "1e+09");
        assert_eq!(fmt_g_py(250_000.0), "250000");
        assert_eq!(fmt_g_py(0.5), "0.5");
    }

    #[test]
    fn flowchart_ramas_largas_y_terminales() {
        let svg = render_val(
            json!({
                "type": "flowchart",
                "steps": [
                    {"io": "leer n"},
                    "i = 0",
                    {"while": "i < n", "do": [{"if": "par(i)", "then": ["s = s + i"], "else": ["s = s - i"]}, "i = i + 1"]},
                    {"if": "s > 0", "else": ["s = 0"]},
                    {"if": "s == 0", "then": ["t = 1"]}
                ]
            }),
            true,
        )
        .unwrap();
        let txts = extraer_textos(&svg);
        for esp in ["Inicio", "Fin", "leer n", "i < n", "par(i)", "s = 0"] {
            assert!(txts.contains(&esp.to_string()), "falta {esp}");
        }
        assert_eq!(txts.iter().filter(|t| *t == "Sí").count(), 4);
        assert_eq!(txts.iter().filter(|t| *t == "No").count(), 4);
        assert_eq!(svg.matches("<polygon").count(), 5);

        let sin_term = render_val(
            json!({"type": "flowchart", "steps": ["a = 1", {"step": "b = 2", "blank": true}], "start": false, "end": null}),
            true,
        )
        .unwrap();
        let st = extraer_textos(&sin_term);
        assert!(st.contains(&"a = 1".to_string()));
        assert!(!st.contains(&"b = 2".to_string()));
        assert!(!st.contains(&"Inicio".to_string()));
        assert!(!st.contains(&"Fin".to_string()));

        // Ramas largas no se solapan
        let largas = render_val(
            json!({"type": "flowchart", "steps": [{"if": "c", "then": ["una instruccion bastante larga"], "else": ["una instruccion bastante larga"]}]}),
            true,
        )
        .unwrap();
        let mut cajas: Vec<(f64, f64)> = Vec::new();
        for parte in largas.split("<rect ").skip(1) {
            let tag = parte.split('>').next().unwrap_or("");
            if tag.contains(r#"height="34.0""#) {
                let x = extraer_attr_f64(tag, "x");
                let w = extraer_attr_f64(tag, "width");
                cajas.push((x, w));
            }
        }
        assert_eq!(cajas.len(), 2);
        cajas.sort_by(|a, b| a.0.total_cmp(&b.0));
        assert!(cajas[0].0 >= 0.0);
        assert!(cajas[0].0 + cajas[0].1 < cajas[1].0);
    }

    #[test]
    fn cost_table_sangria_y_total_cero() {
        let svg = render_val(
            json!({
                "type": "cost_table",
                "lines": [["for (;;)", "c1", "n"], ["    x++;", "c2", "n - 1"]],
                "total": "2n"
            }),
            true,
        )
        .unwrap();
        let txts = extraer_textos(&svg);
        assert!(txts.contains(&"T(n) = 2n".to_string()));

        let mut x_for = None;
        let mut x_cuerpo = None;
        for parte in svg.split("<text ").skip(1) {
            if parte.contains(">for (;;)</text>") {
                x_for = Some(extraer_attr_f64(parte, "x"));
            } else if parte.contains(">x++;</text>") {
                x_cuerpo = Some(extraer_attr_f64(parte, "x"));
            }
        }
        assert!(x_cuerpo.unwrap() > x_for.unwrap() + 20.0);

        let cero = render_val(
            json!({"type": "cost_table", "lines": ["return 0;"], "total": 0}),
            true,
        )
        .unwrap();
        assert!(extraer_textos(&cero).contains(&"T(n) = 0".to_string()));

        let blk = render_val(
            json!({
                "type": "cost_table",
                "lines": [["for (;;)", "c1", "n"]],
                "blankCosts": true,
                "blankTimes": true,
                "blankTotal": true
            }),
            true,
        )
        .unwrap();
        let btxts = extraer_textos(&blk);
        assert!(!btxts.contains(&"c1".to_string()));
        assert!(!btxts.contains(&"T(n) = 2n".to_string()));
    }

    #[test]
    fn trace_table_statics_variables_y_show_index_header() {
        let st = render_val(
            json!({"type": "trace_table", "variant": "stack", "capacity": 5, "pointerLabel": "TOPE: ___", "operations": ["1. push(10)", "2. pop()"]}),
            true,
        )
        .unwrap();
        let st_t = extraer_textos(&st);
        for esp in ["[0]", "[4]", "Puntero TOPE", "1. push(10)"] {
            assert!(st_t.contains(&esp.to_string()));
        }
        assert_eq!(st_t.iter().filter(|t| *t == "TOPE: ___").count(), 2);

        let no_idx = render_val(
            json!({"type": "trace_table", "capacity": 3, "showIndexHeader": false, "operations": ["push(1)"]}),
            true,
        )
        .unwrap();
        let ni_t = extraer_textos(&no_idx);
        assert!(!ni_t.contains(&"[0]".to_string()));
        assert_eq!(ni_t.iter().filter(|t| *t == " ").count(), 3);

        let vars = render_val(
            json!({"type": "trace_table", "columns": ["i", "suma"], "rows": [[0, 7], [1, null]], "rowCount": 4}),
            true,
        )
        .unwrap();
        assert!(extraer_textos(&vars).contains(&"7".to_string()));
        assert_eq!(vars.matches(r#"stroke-dasharray="3 2""#).count(), 5);

        let ops = render_val(
            json!({"type": "trace_table", "columns": ["a"], "operations": ["a = 1", "a++"], "operationHeader": "Línea"}),
            true,
        )
        .unwrap();
        let op_t = extraer_textos(&ops);
        for esp in ["Línea", "a = 1", "a++"] {
            assert!(op_t.contains(&esp.to_string()));
        }
    }

    #[test]
    fn los_21_errores_de_test_diagramas() {
        let mut diecisiete = Vec::new();
        for _ in 0..17 {
            diecisiete.push(json!("int a"));
        }
        let casos: &[(Value, &str)] = &[
            (json!({"type": "struct"}), "fields"),
            (json!({"type": "struct", "fields": ["punto_t p"]}), "tamaño"),
            (json!({"type": "struct", "fields": ["?"]}), "no entiendo"),
            (json!({"type": "struct", "fields": [{"name": "x"}]}), "'type'"),
            (json!({"type": "struct", "variant": "otra", "fields": ["int a"]}), "variante"),
            (json!({"type": "struct", "fields": diecisiete}), "máximo"),
            (json!({"type": "adt"}), "operations"),
            (json!({"type": "adt", "operations": [{"kind": "x"}]}), "signature"),
            (json!({"type": "complexity", "curves": ["n^4"]}), "desconocido"),
            (json!({"type": "complexity", "curves": [{"label": "x"}]}), "points"),
            (json!({"type": "complexity", "curves": [{"points": [[1]]}]}), "points"),
            (json!({"type": "complexity", "curves": 5}), "curves"),
            (json!({"type": "complexity", "variant": "otra"}), "variante"),
            (json!({"type": "complexity", "variant": "classes", "classes": ["P"], "items": {"NP": ["SAT"]}}), "NP"),
            (json!({"type": "flowchart"}), "steps"),
            (json!({"type": "flowchart", "steps": [{"x": 1}]}), "paso 0"),
            (json!({"type": "flowchart", "steps": [{"while": "a"}]}), "cuerpo"),
            (json!({"type": "cost_table"}), "lines"),
            (json!({"type": "trace_table"}), "filas"),
            (json!({"type": "trace_table", "columns": ["a"]}), "filas"),
            (json!({"type": "trace_table", "columns": ["a"], "rows": [[1, 2]]}), "columnas"),
        ];

        for (bloque, esperado) in casos {
            let tipo = bloque["type"].as_str().unwrap();
            let prefijo = format!("{tipo}: ");
            let err = render_val(bloque.clone(), true).unwrap_err();
            let msg = err.to_string();
            assert!(
                msg.starts_with(&prefijo),
                "esperaba prefijo {prefijo:?} en {msg:?}"
            );
            assert!(
                msg.contains(esperado),
                "esperaba {esperado:?} en {msg:?}"
            );
        }

        // Errores propagados (common::lista_de / lista_o_vacia) en los 6 tipos
        let extras: &[(Value, &str)] = &[
            (json!({"type": "struct", "variant": "opaque", "operations": 123, "fields": ["int a"]}), "operations"),
            (json!({"type": "adt", "operations": ["f()"], "representations": 5}), "representations"),
            (json!({"type": "adt", "operations": ["f()"], "client": "main();"}), "client"),
            (json!({"type": "complexity", "curves": [{"points": "bad"}]}), "points"),
            (json!({"type": "flowchart", "steps": "no_lista"}), "steps"),
            (json!({"type": "cost_table", "lines": 123}), "lines"),
            (json!({"type": "trace_table", "operations": "abc"}), "operations"),
        ];
        for (bloque, esperado) in extras {
            let tipo = bloque["type"].as_str().unwrap();
            let prefijo = format!("{tipo}: ");
            let err = render_val(bloque.clone(), true).unwrap_err();
            let msg = err.to_string();
            assert!(
                msg.starts_with(&prefijo),
                "esperaba prefijo {prefijo:?} en {msg:?}"
            );
            assert!(
                msg.contains(esperado),
                "esperaba {esperado:?} en {msg:?}"
            );
        }

        // Idempotencia de envolver_tipo y guarda strip_prefix + ": "
        let ya = envolver_tipo("struct", Err(ErrorDiagrama::new("struct: ya prefijado"))).unwrap_err();
        assert_eq!(ya.to_string(), "struct: ya prefijado");
        let otro = envolver_tipo("struct", Err(ErrorDiagrama::new("struct_extra: detalle"))).unwrap_err();
        assert_eq!(otro.to_string(), "struct: struct_extra: detalle");
    }

    #[test]
    fn complexity_puntos_extremos_sin_inf_ni_nan() {
        let casos = [
            json!({"type": "complexity", "curves": [{"points": [[-1e308, -1e308], [1, 1e308]]}]}),
            json!({"type": "complexity", "curves": [{"points": [[-1e308, 10], [5, 50], [10, 1e308]]}]}),
            json!({"type": "complexity", "curves": [{"points": [[2, 10], [10, -1e308]]}]}),
            json!({"type": "complexity", "maxN": 10000, "maxY": 1_000_000_000, "curves": [{"points": [[-1e308, -1e308], [1, 1e308]]}]}),
        ];

        for caso in casos {
            let svg = render_val(caso, true).unwrap();
            assert!(svg.starts_with("<svg ") && svg.ends_with("</svg>"));
            assert!(!svg.contains("inf"), "SVG contiene inf: {svg}");
            assert!(!svg.contains("NaN"), "SVG contiene NaN: {svg}");
            assert!(!svg.contains("nan"), "SVG contiene nan: {svg}");
            assert!(extraer_attr_f64(&svg, "width").is_finite());
            assert!(extraer_attr_f64(&svg, "height").is_finite());

            for parte in svg.split("<circle ").skip(1) {
                let tag = parte.split('>').next().unwrap_or("");
                assert!(extraer_attr_f64(tag, "cx").is_finite());
                assert!(extraer_attr_f64(tag, "cy").is_finite());
            }
            for parte in svg.split(r#"<polyline points=""#).skip(1) {
                let pts = parte.split('"').next().unwrap_or("");
                assert!(!pts.is_empty());
                for par in pts.split(' ') {
                    let mut nums = par.split(',');
                    let x = nums.next().unwrap().parse::<f64>().unwrap();
                    let y = nums.next().unwrap().parse::<f64>().unwrap();
                    assert!(x.is_finite() && y.is_finite());
                }
            }
        }
    }

    #[test]
    fn utf8_y_entradas_extremas_sin_panic() {
        // UTF-8 con tildes y emojis
        let utf8 = render_val(
            json!({
                "type": "adt",
                "name": "Árbol 🌳",
                "operations": [{"signature": "inserción(clave_ñ)", "kind": "modificadora"}],
                "invariant": "tamaño ≥ 0"
            }),
            true,
        )
        .unwrap();
        assert!(utf8.contains("Árbol 🌳"));

        // YAML con valores extremos (1e308, NaN/inf en YAML) sin panic
        let extremos_yaml = r#"
- type: complexity
  maxN: 1e308
  maxY: .nan
  curves:
    - label: "ext"
      points: [[1, 2], [5, 10]]
- type: trace_table
  columns: ["x"]
  rowCount: 1e308
"#;
        let bloques = leer_bloques(extremos_yaml).unwrap();
        let reg = crear_registro();
        for b in &bloques {
            let res = reg.renderizar(b, true);
            assert!(res.is_ok(), "falló bloque extremo: {res:?}");
        }

        // Anidamiento profundo controlado en flowchart (> 64 niveles -> ErrorDiagrama limpio)
        let mut paso = json!("fin");
        for _ in 0..70 {
            paso = json!({"while": "c", "do": [paso]});
        }
        let flujo_prof = json!({"type": "flowchart", "steps": [paso]});
        let err = render_val(flujo_prof, true).unwrap_err();
        assert!(err.to_string().contains("profundidad"));
    }

    #[test]
    fn edge_cases_adicionales_seis_tipos() {
        // struct: direcciones hex, struct anidado con size/align y errores de sintaxis C
        let st_dir = render_val(
            json!({
                "type": "struct",
                "showAddresses": true,
                "baseAddr": "0x2000",
                "fields": [
                    {"name": "c", "type": "char"},
                    {"name": "pos", "type": "struct punto", "size": 8, "align": 4}
                ]
            }),
            true,
        )
        .unwrap();
        let st_txts = extraer_textos(&st_dir);
        assert!(st_txts.contains(&"0x2000".to_string()));
        assert!(st_txts.contains(&"0x2004".to_string()));
        assert!(st_txts.iter().any(|t| t.contains("sizeof(struct registro) = 12 bytes")));
        assert!(CampoStruct::desde_str("const char * const s").is_err());
        assert!(CampoStruct::desde_str("int a[b]").is_err());

        // adt: blankOperations y falsy signature
        let adt_blk = render_val(
            json!({"type": "adt", "blankOperations": true, "operations": ["push(x)"]}),
            true,
        )
        .unwrap();
        assert!(!extraer_textos(&adt_blk).contains(&"push(x)".to_string()));
        assert!(render_val(json!({"type": "adt", "operations": [0]}), true).is_err());

        // flowchart: for y ramas vacías
        let flujo_for = render_val(
            json!({
                "type": "flowchart",
                "steps": [
                    {"for": "i = 0..n", "do": ["sum += i"]},
                    {"if": "vacio", "then": null, "else": []}
                ]
            }),
            true,
        )
        .unwrap();
        assert!(extraer_textos(&flujo_for).contains(&"i = 0..n".to_string()));

        // cost_table: mixto (dict, lista len 1, lista len 4, escalar) + total vacío
        let costos = render_val(
            json!({
                "type": "cost_table",
                "lines": [
                    {"code": "a = 1;", "cost": "c1", "times": 1},
                    ["b = 2;"],
                    ["c", "1", "2", "extra"],
                    42
                ],
                "total": ""
            }),
            true,
        )
        .unwrap();
        let c_txts = extraer_textos(&costos);
        assert!(c_txts.contains(&"a = 1;".to_string()));
        assert!(c_txts.contains(&"42".to_string()));
        assert!(c_txts.contains(&"T(n) = ".to_string()));
    }
}
