//! Ámbitos y visibilidad (`scopes`), como en Python Tutor: el código con la línea que se va a
//! ejecutar y la recién ejecutada, los ámbitos marcados sobre el código y, al lado, cada ámbito
//! con sus nombres. dibu calcula la cadena de búsqueda desde el ámbito actual, qué nombres quedan
//! ocultos por otro más cercano, qué marcos activos no son visibles y dónde se resuelve cada nombre
//! de `lookup`, con las reglas de C, Python (LEGB, `global`, `nonlocal`) o Java (`this.x`).

use crate::common::{
    ancho_texto, encabezado, es_truthy, tabla, texto, Bloque, ErrorDiagrama, Lienzo, Punto, TablaOpts, MONO, SANS,
};
use dibu_core::Registro;
use serde_json::Value;

const MAX_LINEAS: usize = 40;
const MAX_ALCANCES: usize = 10;
const MAX_NOMBRES: usize = 12;
const MAX_BUSQUEDAS: usize = 12;

const ALTO_LINEA: f64 = 18.0;
const ALTO_FILA: f64 = 24.0;
const ALTO_CABECERA: f64 = 24.0;
const SEP_CAJAS: f64 = 22.0;
const PASO_BARRA: f64 = 9.0;

pub fn registrar(reg: &mut Registro) {
    reg.registrar("scopes", |c, b| {
        render_alcances(c, b).map_err(|e| {
            if e.0.starts_with("scopes: ") {
                e
            } else {
                ErrorDiagrama::new(format!("scopes: {}", e.0))
            }
        })
    });
}

fn error_alcances(msg: impl Into<String>) -> ErrorDiagrama {
    ErrorDiagrama::new(msg)
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum Lenguaje {
    C,
    Python,
    Java,
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum TipoAlcance {
    Integradas,
    Global,
    Funcion,
    Clase,
    Bloque,
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum Declaracion {
    Propia,
    /// `global x` en Python: el nombre es el del ámbito global.
    Global,
    /// `nonlocal x` en Python: el nombre es el de la función envolvente.
    NoLocal,
}

#[derive(Debug)]
struct Nombre {
    nombre: String,
    tipo: Option<String>,
    valor: String,
    declaracion: Declaracion,
    /// Declarado pero todavía sin valor (Python: `UnboundLocalError`; C y Java: sin inicializar).
    sin_valor: bool,
    en_blanco: bool,
}

#[derive(Debug)]
struct Alcance {
    nombre: String,
    clase: TipoAlcance,
    padre: Option<usize>,
    lineas: Option<(usize, usize)>,
    nombres: Vec<Nombre>,
}

/// Dónde se resolvió un nombre: el ámbito y el nombre, o el error.
#[derive(Debug, PartialEq)]
enum Resolucion {
    En(usize, usize),
    SinValor(usize, usize),
    Error(String),
}

fn lenguaje(b: &Bloque) -> Result<Lenguaje, ErrorDiagrama> {
    match b.get("variant").filter(|v| !v.is_null()).map(texto).as_deref() {
        None | Some("python") => Ok(Lenguaje::Python),
        Some("c") => Ok(Lenguaje::C),
        Some("java") => Ok(Lenguaje::Java),
        Some(otra) => Err(error_alcances(format!("variante no soportada: '{otra}' (python, c o java)"))),
    }
}

fn clase_de(t: &str, donde: &str) -> Result<TipoAlcance, ErrorDiagrama> {
    Ok(match t {
        "builtins" => TipoAlcance::Integradas,
        "global" | "module" | "file" => TipoAlcance::Global,
        "function" | "method" => TipoAlcance::Funcion,
        "class" => TipoAlcance::Clase,
        "block" => TipoAlcance::Bloque,
        otro => {
            return Err(error_alcances(format!(
                "{donde}: 'kind' es builtins, global, module, file, function, method, class o block; no '{otro}'"
            )));
        }
    })
}

/// Un rango `[desde, hasta]` de líneas (desde 1), o una sola línea.
fn rango(v: &Value, n: usize, donde: &str) -> Result<(usize, usize), ErrorDiagrama> {
    let linea = |x: &Value| -> Result<usize, ErrorDiagrama> {
        match x.as_u64() {
            Some(k) if k >= 1 && (k as usize) <= n => Ok(k as usize),
            _ => Err(error_alcances(format!("{donde}: las líneas van de 1 a {n}: {}", texto(x)))),
        }
    };
    match v {
        Value::Array(l) if l.len() == 2 => {
            let (a, z) = (linea(&l[0])?, linea(&l[1])?);
            if a > z {
                return Err(error_alcances(format!("{donde}: el rango {a}–{z} está al revés")));
            }
            Ok((a, z))
        }
        Value::Number(_) => {
            let k = linea(v)?;
            Ok((k, k))
        }
        _ => Err(error_alcances(format!("{donde}: 'lines' es [desde, hasta]"))),
    }
}

fn leer_alcances(b: &Bloque, n_lineas: usize, leng: Lenguaje) -> Result<Vec<Alcance>, ErrorDiagrama> {
    let Some(lista) = b.get("scopes").and_then(Value::as_array).filter(|l| !l.is_empty()) else {
        return Err(error_alcances("falta 'scopes': los ámbitos, del más externo al actual, cada uno con name, kind y vars"));
    };
    if lista.len() > MAX_ALCANCES {
        return Err(error_alcances(format!("hay {} ámbitos; el máximo es {MAX_ALCANCES}", lista.len())));
    }
    let mut alcances: Vec<Alcance> = Vec::new();
    for (i, a) in lista.iter().enumerate() {
        let Some(o) = a.as_object() else {
            return Err(error_alcances(format!("ámbito {i}: se espera {{name, kind, parent, lines, vars}}")));
        };
        let nombre = o.get("name").filter(|v| es_truthy(v)).map(texto).unwrap_or_default();
        if nombre.is_empty() {
            return Err(error_alcances(format!("ámbito {i}: falta 'name'")));
        }
        if alcances.iter().any(|x| x.nombre == nombre) {
            return Err(error_alcances(format!("el ámbito '{nombre}' está repetido; usar nombres distintos (f#1, f#2)")));
        }
        let donde = format!("ámbito {nombre}");
        let clase = match o.get("kind").filter(|v| !v.is_null()) {
            Some(k) => clase_de(&texto(k), &donde)?,
            None if i == 0 => TipoAlcance::Global,
            None => TipoAlcance::Funcion,
        };
        if clase == TipoAlcance::Integradas && leng != Lenguaje::Python {
            return Err(error_alcances(format!("{donde}: 'builtins' es un ámbito de Python")));
        }
        // Sin `parent`, el ámbito anterior de la lista (los bloques y funciones anidados).
        let padre = match o.get("parent").filter(|v| !v.is_null()) {
            None => (i > 0).then(|| i - 1),
            Some(Value::Bool(false)) => None,
            Some(p) => {
                let p = texto(p);
                match alcances.iter().position(|x| x.nombre == p) {
                    Some(k) => Some(k),
                    None => {
                        return Err(error_alcances(format!(
                            "{donde}: 'parent' es '{p}', que no es un ámbito anterior"
                        )));
                    }
                }
            }
        };
        let lineas = o.get("lines").filter(|v| !v.is_null()).map(|v| rango(v, n_lineas, &donde)).transpose()?;
        let mut nombres: Vec<Nombre> = Vec::new();
        let datos = o.get("vars").filter(|v| !v.is_null());
        let lista_nombres: Vec<(String, Value)> = match datos {
            None => Vec::new(),
            Some(Value::Array(l)) => l.iter().map(|v| (String::new(), v.clone())).collect(),
            Some(Value::Object(m)) => m.iter().map(|(k, v)| (k.clone(), v.clone())).collect(),
            Some(_) => return Err(error_alcances(format!("{donde}: 'vars' es una lista de {{name, value}} o un mapa"))),
        };
        if lista_nombres.len() > MAX_NOMBRES {
            return Err(error_alcances(format!("{donde}: hay {} nombres; el máximo es {MAX_NOMBRES}", lista_nombres.len())));
        }
        for (clave, v) in lista_nombres {
            let nombre_var;
            let mut tipo = None;
            let mut valor = String::new();
            let mut declaracion = Declaracion::Propia;
            let mut sin_valor = false;
            let mut en_blanco = false;
            if !clave.is_empty() {
                nombre_var = clave;
                valor = if v.is_null() { String::new() } else { texto(&v) };
            } else if let Some(m) = v.as_object() {
                nombre_var = m.get("name").filter(|x| es_truthy(x)).map(texto).unwrap_or_default();
                tipo = m.get("type").filter(|x| es_truthy(x)).map(texto);
                valor = m.get("value").filter(|x| !x.is_null()).map(texto).unwrap_or_default();
                // `indeterminate` (como en object_heap) es lo mismo: sin valor todavía.
                sin_valor = m.get("unbound").is_some_and(es_truthy) || m.get("indeterminate").is_some_and(es_truthy);
                en_blanco = m.get("blank").is_some_and(es_truthy);
                declaracion = match m.get("declare").filter(|x| !x.is_null()).map(texto).as_deref() {
                    None => Declaracion::Propia,
                    Some("global") if leng == Lenguaje::Python => Declaracion::Global,
                    Some("nonlocal") if leng == Lenguaje::Python => Declaracion::NoLocal,
                    Some(d) => {
                        return Err(error_alcances(format!(
                            "{donde}: 'declare: {d}' no existe en esta variante (en Python: global o nonlocal)"
                        )));
                    }
                };
                if declaracion != Declaracion::Propia && (m.contains_key("value") || sin_valor) {
                    return Err(error_alcances(format!(
                        "{donde}: '{nombre_var}' declarado {d} no tiene valor propio: el valor está en el otro ámbito",
                        d = if declaracion == Declaracion::Global { "global" } else { "nonlocal" }
                    )));
                }
            } else if let Some(s) = v.as_str() {
                nombre_var = s.to_string();
            } else {
                return Err(error_alcances(format!("{donde}: un nombre es {{name, value}}, no {}", texto(&v))));
            }
            if nombre_var.is_empty() {
                return Err(error_alcances(format!("{donde}: un nombre sin 'name'")));
            }
            if nombres.iter().any(|x| x.nombre == nombre_var) {
                return Err(error_alcances(format!("{donde}: '{nombre_var}' está dos veces en el mismo ámbito")));
            }
            nombres.push(Nombre { nombre: nombre_var, tipo, valor, declaracion, sin_valor, en_blanco });
        }
        alcances.push(Alcance { nombre, clase, padre, lineas, nombres });
    }
    Ok(alcances)
}

/// La cadena de búsqueda desde `actual`: el ámbito y sus padres. En Python, el cuerpo de una clase
/// no es envolvente para las funciones definidas adentro: se saltea.
fn cadena(alcances: &[Alcance], actual: usize, leng: Lenguaje) -> Vec<usize> {
    let mut salida = vec![actual];
    let mut k = alcances[actual].padre;
    while let Some(p) = k {
        if salida.contains(&p) {
            break;
        }
        let saltear = leng == Lenguaje::Python && alcances[p].clase == TipoAlcance::Clase && alcances[actual].clase != TipoAlcance::Clase;
        if !saltear {
            salida.push(p);
        }
        k = alcances[p].padre;
    }
    salida
}

fn no_encontrado(leng: Lenguaje, nombre: &str) -> String {
    match leng {
        Lenguaje::Python => format!("NameError: name '{nombre}' is not defined"),
        Lenguaje::C => format!("error: '{nombre}' undeclared"),
        Lenguaje::Java => format!("error: cannot find symbol {nombre}"),
    }
}

fn resolver(alcances: &[Alcance], cad: &[usize], nombre: &str, leng: Lenguaje) -> Resolucion {
    let encontrado = |a: usize, k: usize| {
        if alcances[a].nombres[k].sin_valor {
            Resolucion::SinValor(a, k)
        } else {
            Resolucion::En(a, k)
        }
    };
    // `this.x` en Java: el campo de la clase más cercana, aunque un local lo oculte.
    if let Some(campo) = nombre.strip_prefix("this.") {
        if leng != Lenguaje::Java {
            return Resolucion::Error(format!("'this.' es de Java: {}", no_encontrado(leng, nombre)));
        }
        return match cad.iter().find(|&&a| alcances[a].clase == TipoAlcance::Clase) {
            None => Resolucion::Error("error: non-static variable this cannot be referenced from a static context".to_string()),
            Some(&a) => match alcances[a].nombres.iter().position(|n| n.nombre == campo) {
                Some(k) => encontrado(a, k),
                None => Resolucion::Error(no_encontrado(leng, nombre)),
            },
        };
    }
    for (pos, &a) in cad.iter().enumerate() {
        let Some(k) = alcances[a].nombres.iter().position(|n| n.nombre == nombre) else {
            continue;
        };
        return match alcances[a].nombres[k].declaracion {
            Declaracion::Propia => encontrado(a, k),
            Declaracion::Global => {
                let global = cad.iter().copied().find(|&g| alcances[g].clase == TipoAlcance::Global);
                match global.and_then(|g| alcances[g].nombres.iter().position(|n| n.nombre == nombre).map(|k| (g, k))) {
                    Some((g, k)) => encontrado(g, k),
                    None => Resolucion::Error(no_encontrado(leng, nombre)),
                }
            }
            Declaracion::NoLocal => {
                let envolvente = cad[pos + 1..]
                    .iter()
                    .copied()
                    .filter(|&e| alcances[e].clase == TipoAlcance::Funcion)
                    .find_map(|e| alcances[e].nombres.iter().position(|n| n.nombre == nombre && n.declaracion == Declaracion::Propia).map(|k| (e, k)));
                match envolvente {
                    Some((e, k)) => encontrado(e, k),
                    None => Resolucion::Error(format!("SyntaxError: no binding for nonlocal '{nombre}' found")),
                }
            }
        };
    }
    Resolucion::Error(no_encontrado(leng, nombre))
}

/// El rótulo de la clase de un ámbito según el lenguaje y su lugar en la cadena. `neutro` no
/// dice el lugar en la cadena (local o envolvente), para el modo ejercicio.
fn rotulo_clase(a: &Alcance, leng: Lenguaje, es_actual: bool, neutro: bool) -> &'static str {
    match (leng, a.clase) {
        (Lenguaje::Python, TipoAlcance::Integradas) if neutro => "integrados",
        (Lenguaje::Python, TipoAlcance::Global) if neutro => "global",
        (Lenguaje::Python, TipoAlcance::Funcion) if neutro => "función",
        (Lenguaje::Python, TipoAlcance::Integradas) => "integrados (B)",
        (Lenguaje::Python, TipoAlcance::Global) => "global (G)",
        (Lenguaje::Python, TipoAlcance::Funcion) if es_actual => "local (L)",
        (Lenguaje::Python, TipoAlcance::Funcion) => "envolvente (E)",
        (Lenguaje::Python, TipoAlcance::Clase) => "cuerpo de clase",
        (Lenguaje::Python, TipoAlcance::Bloque) => "bloque",
        (Lenguaje::C, TipoAlcance::Global) => "archivo (global)",
        (Lenguaje::C, TipoAlcance::Funcion) => "función",
        (Lenguaje::C, TipoAlcance::Bloque) => "bloque",
        (Lenguaje::C, TipoAlcance::Clase) => "struct",
        (Lenguaje::Java, TipoAlcance::Clase) => "clase (campos)",
        (Lenguaje::Java, TipoAlcance::Funcion) => "método",
        (Lenguaje::Java, TipoAlcance::Bloque) => "bloque",
        (Lenguaje::Java, TipoAlcance::Global) => "paquete",
        (_, TipoAlcance::Integradas) => "integrados",
    }
}

/// Los espacios del código se conservan: SVG colapsa los que se repiten.
fn espacios_duros(s: &str) -> String {
    s.replace('\t', "    ").replace(' ', "\u{a0}")
}

/// Una línea de código: la sangría corre la `x` (algunos renderizadores descartan los espacios
/// del principio, aun los duros) y los espacios de adentro quedan como espacios duros.
fn linea_codigo(c: &mut Lienzo, x: f64, y: f64, linea: &str) {
    let linea = linea.replace('\t', "    ");
    let resto = linea.trim_start_matches(' ');
    let sangria = (linea.len() - resto.len()) as f64;
    c.t(x + dibu_core::avance_mono(sangria, 11.0), y, &espacios_duros(resto), 11.0, 400, "textMain", "start", MONO);
}

fn numero(k: usize) -> String {
    format!("[{k}]")
}

fn render_alcances(c: &mut Lienzo, b: &Bloque) -> Result<Punto, ErrorDiagrama> {
    let leng = lenguaje(b)?;
    let codigo = b.get("code").filter(|v| es_truthy(v)).map(texto).unwrap_or_default();
    let mut lineas: Vec<&str> = codigo.lines().collect();
    while lineas.last().is_some_and(|l| l.trim().is_empty()) {
        lineas.pop();
    }
    if lineas.len() > MAX_LINEAS {
        return Err(error_alcances(format!("el código tiene {} líneas; el máximo es {MAX_LINEAS}", lineas.len())));
    }
    let n = lineas.len();
    let linea_de = |clave: &str| -> Result<Option<usize>, ErrorDiagrama> {
        match b.get(clave).filter(|v| !v.is_null()) {
            None => Ok(None),
            Some(v) => match v.as_u64() {
                Some(k) if k >= 1 && (k as usize) <= n => Ok(Some(k as usize)),
                _ => Err(error_alcances(format!("'{clave}' es una línea del código, de 1 a {n}: {}", texto(v)))),
            },
        }
    };
    let actual_linea = linea_de("line")?;
    let previa = linea_de("prev")?;
    let alcances = leer_alcances(b, n, leng)?;
    let actual = match b.get("current").filter(|v| !v.is_null()) {
        None => alcances.len() - 1,
        Some(v) => {
            let t = texto(v);
            alcances
                .iter()
                .position(|a| a.nombre == t)
                .ok_or_else(|| error_alcances(format!("'current' es '{t}', que no es un ámbito")))?
        }
    };
    if let (Some(l), Some((a, z))) = (actual_linea, alcances[actual].lineas) {
        if l < a || l > z {
            return Err(error_alcances(format!(
                "la línea {l} está fuera del ámbito actual '{}' (líneas {a}–{z})",
                alcances[actual].nombre
            )));
        }
    }
    let cad = cadena(&alcances, actual, leng);
    let busquedas: Vec<String> = match b.get("lookup").filter(|v| !v.is_null()) {
        None => Vec::new(),
        Some(Value::Array(l)) => l.iter().map(texto).filter(|s| !s.is_empty()).collect(),
        Some(v) => texto(v).split(',').map(|s| s.trim().to_string()).filter(|s| !s.is_empty()).collect(),
    };
    if busquedas.len() > MAX_BUSQUEDAS {
        return Err(error_alcances(format!("hay {} búsquedas; el máximo es {MAX_BUSQUEDAS}", busquedas.len())));
    }
    let sin_valores = b.get("blankValues").is_some_and(es_truthy);
    let sin_visibilidad = b.get("blankVisibility").is_some_and(es_truthy);
    let sin_busquedas = b.get("blankLookup").is_some_and(es_truthy);

    // Qué nombres de la cadena quedan ocultos por otro del mismo nombre más cerca.
    let mut oculto_por: Vec<Vec<Option<usize>>> = alcances.iter().map(|a| vec![None; a.nombres.len()]).collect();
    let mut vistos: Vec<(String, usize)> = Vec::new();
    for &a in &cad {
        for (k, nm) in alcances[a].nombres.iter().enumerate() {
            if let Some((_, quien)) = vistos.iter().find(|(v, _)| *v == nm.nombre) {
                oculto_por[a][k] = Some(*quien);
            } else if nm.declaracion == Declaracion::Propia {
                vistos.push((nm.nombre.clone(), a));
            } else {
                // `global x` o `nonlocal x` no ocultan: el nombre es el de afuera.
                vistos.push((nm.nombre.clone(), usize::MAX));
            }
        }
    }
    for fila in oculto_por.iter_mut() {
        for o in fila.iter_mut() {
            if *o == Some(usize::MAX) {
                *o = None;
            }
        }
    }

    let top = encabezado(c, b, "");
    // ---------- Código, con las barras de los ámbitos y las flechas de línea ----------
    let con_rango: Vec<usize> = (0..alcances.len()).filter(|&a| alcances[a].lineas.is_some()).collect();
    let x_barras = 4.0;
    let x_flecha = x_barras + con_rango.len() as f64 * PASO_BARRA + 8.0;
    let x_num = x_flecha + 30.0;
    let x_cod = x_num + 10.0;
    let y0 = top + 22.0;
    let ancho_cod = lineas.iter().map(|l| ancho_texto(&espacios_duros(l), 11.0)).fold(160.0, f64::max);
    let fin_cod = x_cod + ancho_cod + 14.0;
    c.r(x_num - 26.0, y0 - 6.0, fin_cod - x_num + 26.0, n as f64 * ALTO_LINEA + 12.0, "fillCell", 1.2, 4.0);
    for (i, l) in lineas.iter().enumerate() {
        let y = y0 + i as f64 * ALTO_LINEA;
        let ln = i + 1;
        if Some(ln) == actual_linea {
            c.add(format!(
                r#"<rect x="{:.1}" y="{:.1}" width="{:.1}" height="{ALTO_LINEA:.1}" fill="{}"/>"#,
                x_num - 25.0,
                y,
                fin_cod - x_num + 24.0,
                c.pal.fill_badge
            ));
        }
        c.t(x_num, y + 13.0, &ln.to_string(), 9.5, 400, "textMuted", "end", MONO);
        linea_codigo(c, x_cod, y + 13.0, l);
        if Some(ln) == previa {
            c.linea(x_flecha, y + 9.0, x_num - 28.0, y + 9.0, "strokeSub", 1.6, Some("sub"), "4 2");
        }
        if Some(ln) == actual_linea {
            c.linea(x_flecha, y + 9.0, x_num - 28.0, y + 9.0, "accentPrimary", 2.2, Some("main"), "");
        }
    }
    for (pos, &a) in con_rango.iter().enumerate() {
        let (desde, hasta) = alcances[a].lineas.unwrap();
        let x = x_barras + pos as f64 * PASO_BARRA;
        let ya = y0 + (desde - 1) as f64 * ALTO_LINEA + 2.0;
        let yz = y0 + hasta as f64 * ALTO_LINEA - 2.0;
        let visible = cad.contains(&a);
        let guiones = if visible || sin_visibilidad { "" } else { "3 2" };
        c.linea(x, ya, x, yz, "strokeMain", if a == actual { 2.4 } else { 1.3 }, None, guiones);
        c.linea(x, ya, x + 4.0, ya, "strokeMain", 1.3, None, "");
        c.linea(x, yz, x + 4.0, yz, "strokeMain", 1.3, None, "");
    }
    // Números de ámbito sobre sus barras.
    for (pos, &a) in con_rango.iter().enumerate() {
        let (desde, _) = alcances[a].lineas.unwrap();
        let x = x_barras + pos as f64 * PASO_BARRA;
        let y = y0 + (desde - 1) as f64 * ALTO_LINEA + 1.0;
        c.t(x + 1.0, y - 1.0, &(a + 1).to_string(), 7.5, 700, "textSub", "middle", SANS);
    }
    let mut y_ley = y0 + n as f64 * ALTO_LINEA + 22.0;
    let leyendas = ["flecha llena: la próxima línea", "punteada: la recién ejecutada", "barras: los ámbitos [1], [2]…"];
    let mut fin_izq = fin_cod;
    for l in leyendas {
        c.t(x_flecha, y_ley, l, 8.5, 400, "textMuted", "start", SANS);
        fin_izq = fin_izq.max(x_flecha + ancho_texto(l, 8.5));
        y_ley += 12.0;
    }
    let mut ancho = fin_izq;

    // ---------- Los ámbitos: primero la cadena de búsqueda, después los que no se ven ----------
    let x_caja = fin_izq + 36.0;
    // En el modo ejercicio, del último de la lista al primero: el orden de la cadena la delataría.
    let invisibles: Vec<usize> = (0..alcances.len()).rev().filter(|a| !cad.contains(a)).collect();
    let orden: Vec<usize> = if sin_visibilidad {
        (0..alcances.len()).rev().collect()
    } else {
        cad.iter().chain(&invisibles).copied().collect()
    };
    let ancho_caja = alcances
        .iter()
        .map(|a| {
            let cab = ancho_texto(&format!("[9] {} — {}", a.nombre, "envolvente (E)"), 10.5) + 20.0;
            let filas = a
                .nombres
                .iter()
                .map(|nm| {
                    let izq = ancho_texto(&rotulo_nombre(nm), 10.5);
                    let der = ancho_texto(&mostrar_valor(nm, leng), 11.0).max(36.0);
                    izq + der + 40.0
                })
                .fold(0.0, f64::max);
            cab.max(filas)
        })
        .fold(200.0, f64::max);
    let ancho_marca = ancho_texto("oculta por [9]", 9.0) + 14.0;
    let mut y = top + 16.0;
    if !sin_visibilidad {
        c.t(x_caja, y, "Búsqueda desde el ámbito actual", 10.0, 700, "textSub", "start", SANS);
        y += 12.0;
    }
    for (i, &a) in orden.iter().enumerate() {
        let al = &alcances[a];
        let visible = cad.contains(&a);
        if i == cad.len() && !invisibles.is_empty() && !sin_visibilidad {
            y += 6.0;
            c.t(x_caja, y, "Activos pero no visibles desde aquí", 10.0, 700, "textSub", "start", SANS);
            y += 12.0;
        }
        let alto = ALTO_CABECERA + al.nombres.len().max(1) as f64 * ALTO_FILA + 6.0;
        let destacado = a == actual;
        if visible || sin_visibilidad {
            c.r(x_caja, y, ancho_caja, alto, "fillCell", if destacado { 2.4 } else { 1.4 }, 4.0);
            c.r(x_caja, y, ancho_caja, ALTO_CABECERA, "fillHeader", if destacado { 2.4 } else { 1.4 }, 4.0);
        } else {
            c.add(format!(
                r#"<rect x="{x_caja:.1}" y="{y:.1}" width="{ancho_caja:.1}" height="{alto:.1}" rx="4" fill="none" stroke="{}" stroke-width="1.4" stroke-dasharray="6 3"/>"#,
                c.pal.stroke_muted
            ));
        }
        let clase = rotulo_clase(al, leng, destacado, sin_visibilidad);
        let cab = format!("{} {} — {clase}", numero(a + 1), al.nombre);
        c.t(x_caja + 8.0, y + 16.0, &cab, 10.5, 700, if visible || sin_visibilidad { "textMain" } else { "textMuted" }, "start", MONO);
        if destacado && !sin_visibilidad {
            c.t(x_caja + ancho_caja - 8.0, y + 16.0, "actual", 9.0, 700, "textSub", "end", SANS);
        }
        if al.nombres.is_empty() {
            c.t(x_caja + 10.0, y + ALTO_CABECERA + 16.0, "(sin nombres)", 9.5, 400, "textMuted", "start", SANS);
        }
        let ancho_valor = al.nombres.iter().map(|nm| ancho_texto(&mostrar_valor(nm, leng), 11.0)).fold(36.0, f64::max) + 14.0;
        for (k, nm) in al.nombres.iter().enumerate() {
            let fy = y + ALTO_CABECERA + k as f64 * ALTO_FILA + 3.0;
            let oculto = oculto_por[a][k].filter(|_| !sin_visibilidad);
            let tenue = oculto.is_some() || (!visible && !sin_visibilidad);
            c.t(x_caja + 10.0, fy + 14.0, &rotulo_nombre(nm), 10.5, 400, if tenue { "textMuted" } else { "textMain" }, "start", MONO);
            if oculto.is_some() {
                let w = ancho_texto(&rotulo_nombre(nm), 10.5);
                c.linea(x_caja + 9.0, fy + 10.0, x_caja + 11.0 + w, fy + 10.0, "strokeSub", 1.1, None, "");
            }
            if nm.declaracion != Declaracion::Propia {
                let txt = if nm.declaracion == Declaracion::Global { "→ el global" } else { "→ el envolvente" };
                c.t(x_caja + ancho_caja - 10.0, fy + 14.0, txt, 9.5, 400, "textSub", "end", SANS);
                continue;
            }
            let vx = x_caja + ancho_caja - ancho_valor - 8.0;
            c.r(vx, fy, ancho_valor, ALTO_FILA - 6.0, "fillPointerBox", 1.2, 2.0);
            if sin_valores || nm.en_blanco {
                c.vacio(vx + 5.0, vx + ancho_valor - 5.0, fy + ALTO_FILA - 10.0);
            } else {
                let v = mostrar_valor(nm, leng);
                let (tam, color, fuente) = if nm.sin_valor { (9.0, "textMuted", SANS) } else { (11.0, "textMain", MONO) };
                c.t(vx + ancho_valor / 2.0, fy + 13.0, &v, tam, 700, color, "middle", fuente);
            }
            if let Some(q) = oculto {
                c.t(x_caja + ancho_caja + 8.0, fy + 13.0, &format!("oculta por {}", numero(q + 1)), 9.0, 700, "textMuted", "start", SANS);
            }
        }
        // Entre dos ámbitos de la cadena: si no está, se sigue buscando en el de abajo.
        let siguiente_en_cadena = i + 1 < cad.len();
        if siguiente_en_cadena && !sin_visibilidad {
            let cx = x_caja + 26.0;
            c.linea(cx, y + alto + 2.0, cx, y + alto + SEP_CAJAS - 3.0, "strokeSub", 1.3, Some("sub"), "");
            c.t(cx + 8.0, y + alto + 15.0, "si no está, sigue en", 8.5, 400, "textMuted", "start", SANS);
        }
        y += alto + SEP_CAJAS;
    }
    ancho = ancho.max(x_caja + ancho_caja + ancho_marca);
    let mut alto_total = y.max(y_ley);

    // ---------- Resolución de cada nombre de `lookup` ----------
    if !busquedas.is_empty() {
        let mut filas: Vec<Vec<String>> = Vec::new();
        let mut tenues: Vec<usize> = Vec::new();
        for (i, nombre) in busquedas.iter().enumerate() {
            let r = resolver(&alcances, &cad, nombre, leng);
            let (donde, valor) = if sin_busquedas {
                (String::new(), String::new())
            } else {
                match &r {
                    Resolucion::En(a, k) => (
                        format!("{} {}", numero(a + 1), alcances[*a].nombre),
                        if sin_valores { String::new() } else { mostrar_valor(&alcances[*a].nombres[*k], leng) },
                    ),
                    Resolucion::SinValor(a, k) => {
                        let msj = match leng {
                            Lenguaje::Python => format!("UnboundLocalError: '{}' sin valor todavía", alcances[*a].nombres[*k].nombre),
                            Lenguaje::C => "sin inicializar: valor indeterminado".to_string(),
                            Lenguaje::Java => "error: variable might not have been initialized".to_string(),
                        };
                        (format!("{} {}", numero(a + 1), alcances[*a].nombre), msj)
                    }
                    Resolucion::Error(e) => {
                        tenues.push(i);
                        ("— (no se encuentra)".to_string(), e.clone())
                    }
                }
            };
            filas.push(vec![nombre.clone(), donde, valor]);
        }
        let titulo_y = alto_total + 6.0;
        let desde = actual_linea.map(|l| format!(" en la línea {l}")).unwrap_or_default();
        c.t(0.0, titulo_y, &format!("¿A qué se refiere cada nombre{desde}?"), 10.5, 700, "textMain", "start", SANS);
        let (w, h) = tabla(
            c,
            0.0,
            titulo_y + 10.0,
            &["nombre", "se resuelve en", "valor"],
            &filas,
            TablaOpts { izquierda: &[0, 1, 2], tenues: &tenues, fila_h: 22.0, min_w: 110.0, ..TablaOpts::default() },
        );
        ancho = ancho.max(w);
        alto_total = titulo_y + 10.0 + h + 6.0;
    }
    Ok((ancho + 6.0, alto_total + 4.0))
}

fn rotulo_nombre(nm: &Nombre) -> String {
    match nm.declaracion {
        Declaracion::Global => format!("global {}", nm.nombre),
        Declaracion::NoLocal => format!("nonlocal {}", nm.nombre),
        Declaracion::Propia => match &nm.tipo {
            Some(t) if t.ends_with('*') => format!("{t}{}", nm.nombre),
            Some(t) => format!("{t} {}", nm.nombre),
            None => nm.nombre.clone(),
        },
    }
}

fn mostrar_valor(nm: &Nombre, leng: Lenguaje) -> String {
    if nm.sin_valor {
        return match leng {
            Lenguaje::C => "indeterminado".to_string(),
            _ => "sin valor".to_string(),
        };
    }
    nm.valor.clone()
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn svg(b: Value) -> Result<String, ErrorDiagrama> {
        let mut reg = Registro::new();
        registrar(&mut reg);
        reg.renderizar(b.as_object().unwrap(), true)
    }

    fn python() -> Value {
        json!({
            "type": "scopes", "variant": "python",
            "code": "x = 10\ndef f(y):\n    x = y + 1\n    return x\nprint(f(3))",
            "line": 4, "prev": 3,
            "scopes": [
                {"name": "builtins", "kind": "builtins", "parent": false, "vars": {"print": "función"}},
                {"name": "global", "kind": "global", "parent": "builtins", "lines": [1, 5], "vars": [{"name": "x", "value": 10}, {"name": "f", "value": "función"}]},
                {"name": "f", "kind": "function", "lines": [2, 4], "vars": [{"name": "y", "value": 3}, {"name": "x", "value": 4}]}
            ],
            "lookup": ["x", "y", "print", "z"]
        })
    }

    #[test]
    fn python_legb_y_ocultamiento() {
        let s = svg(python()).unwrap();
        assert!(s.contains("local (L)") && s.contains("global (G)") && s.contains("integrados (B)"));
        assert!(s.contains(">oculta por [3]<"), "el x global queda oculto por el de f");
        assert!(s.contains("NameError: name &#x27;z&#x27; is not defined"));
        // La sangría corre la línea a la derecha; los espacios de adentro son duros.
        let x_de = |txt: &str| -> f64 {
            let i = s.find(&format!(">{txt}<")).unwrap();
            let t = &s[s[..i].rfind("<text x=\"").unwrap() + 9..];
            t[..t.find('"').unwrap()].parse().unwrap()
        };
        assert!(x_de("x\u{a0}=\u{a0}y\u{a0}+\u{a0}1") > x_de("x\u{a0}=\u{a0}10") + 20.0, "la sangría se conserva");
    }

    #[test]
    fn python_global_nonlocal_y_unbound() {
        let base = |vars_g: Value, vars_f: Value, vars_h: Value| {
            let a = vec![
                json!({"name": "global", "kind": "global", "vars": vars_g}),
                json!({"name": "f", "kind": "function", "vars": vars_f}),
                json!({"name": "g", "kind": "function", "vars": vars_h}),
            ];
            let alcances: Vec<Alcance> = {
                let b = json!({"scopes": a});
                leer_alcances(b.as_object().unwrap(), 1, Lenguaje::Python).unwrap()
            };
            alcances
        };
        let al = base(json!([{"name": "n", "value": 1}]), json!([{"name": "n", "value": 2}]), json!([{"name": "n", "declare": "nonlocal"}]));
        let cad = cadena(&al, 2, Lenguaje::Python);
        assert_eq!(resolver(&al, &cad, "n", Lenguaje::Python), Resolucion::En(1, 0));
        let al = base(json!([{"name": "n", "value": 1}]), json!([{"name": "n", "value": 2}]), json!([{"name": "n", "declare": "global"}]));
        assert_eq!(resolver(&al, &cadena(&al, 2, Lenguaje::Python), "n", Lenguaje::Python), Resolucion::En(0, 0));
        let al = base(json!([{"name": "n", "value": 1}]), json!([]), json!([{"name": "n", "unbound": true}]));
        assert_eq!(resolver(&al, &cadena(&al, 2, Lenguaje::Python), "n", Lenguaje::Python), Resolucion::SinValor(2, 0));
        let al = base(json!([]), json!([]), json!([{"name": "n", "declare": "nonlocal"}]));
        assert!(matches!(resolver(&al, &cadena(&al, 2, Lenguaje::Python), "n", Lenguaje::Python), Resolucion::Error(e) if e.contains("nonlocal")));
    }

    #[test]
    fn python_el_cuerpo_de_clase_no_es_envolvente() {
        let b = json!({"scopes": [
            {"name": "global", "kind": "global", "vars": [{"name": "v", "value": 1}]},
            {"name": "C", "kind": "class", "vars": [{"name": "v", "value": 2}]},
            {"name": "m", "kind": "function", "vars": []}
        ]});
        let al = leer_alcances(b.as_object().unwrap(), 1, Lenguaje::Python).unwrap();
        assert_eq!(resolver(&al, &cadena(&al, 2, Lenguaje::Python), "v", Lenguaje::Python), Resolucion::En(0, 0));
        // En Java, la clase sí es envolvente (los campos).
        let al = leer_alcances(b.as_object().unwrap(), 1, Lenguaje::Java).unwrap();
        assert_eq!(resolver(&al, &cadena(&al, 2, Lenguaje::Java), "v", Lenguaje::Java), Resolucion::En(1, 0));
    }

    #[test]
    fn c_bloques_y_marcos_no_visibles() {
        let s = svg(json!({
            "type": "scopes", "variant": "c",
            "code": "int g = 1;\nvoid f(int a) {\n  int g = a;\n}\nint main(void) {\n  int x = 2;\n  { int x = 3; f(x); }\n}",
            "line": 3,
            "scopes": [
                {"name": "archivo", "kind": "file", "lines": [1, 8], "vars": [{"name": "g", "type": "int", "value": 1}]},
                {"name": "main", "kind": "function", "parent": "archivo", "lines": [5, 8], "vars": [{"name": "x", "type": "int", "value": 2}]},
                {"name": "bloque", "kind": "block", "parent": "main", "lines": [7, 7], "vars": [{"name": "x", "type": "int", "value": 3}]},
                {"name": "f", "kind": "function", "parent": "archivo", "lines": [2, 4], "vars": [{"name": "a", "type": "int", "value": 3}, {"name": "g", "type": "int", "indeterminate": true}]}
            ],
            "lookup": ["g", "a", "x"]
        }))
        .unwrap();
        assert!(s.contains("Activos pero no visibles desde aquí"), "main y su bloque están activos pero no se ven desde f");
        assert!(s.contains("error: &#x27;x&#x27; undeclared"));
        assert!(s.contains("sin inicializar"));
        assert!(s.contains(">oculta por [4]<"));
    }

    #[test]
    fn java_this_y_ocultamiento_de_un_campo() {
        let s = svg(json!({
            "type": "scopes", "variant": "java",
            "code": "class Punto {\n  int x = 1;\n  void mover(int x) {\n    this.x = x;\n  }\n}",
            "line": 4,
            "scopes": [
                {"name": "Punto", "kind": "class", "lines": [1, 6], "vars": [{"name": "x", "type": "int", "value": 1}]},
                {"name": "mover", "kind": "method", "lines": [3, 5], "vars": [{"name": "x", "type": "int", "value": 5}]}
            ],
            "lookup": ["x", "this.x", "y"]
        }))
        .unwrap();
        assert!(s.contains("clase (campos)") && s.contains("método"));
        assert!(s.contains(">oculta por [2]<"));
        assert!(s.contains("cannot find symbol y"));
        // this.x llega al campo aunque el parámetro lo oculte.
        let i = s.find(">this.x<").unwrap();
        assert!(s[i..].contains("[1] Punto"));
    }

    #[test]
    fn modo_ejercicio_y_errores() {
        let mut b = python();
        b["blankLookup"] = json!(true);
        b["blankVisibility"] = json!(true);
        let s = svg(b).unwrap();
        assert!(!s.contains("NameError") && !s.contains("oculta por") && !s.contains("no visibles"));
        assert!(!s.contains("local (L)") && !s.contains("envolvente (E)") && !s.contains("sigue en"));
        let mut b = python();
        b["line"] = json!(1);
        assert!(svg(b).unwrap_err().0.contains("fuera del ámbito actual"));
        let casos = [
            (json!({"type": "scopes", "code": "x"}), "falta 'scopes'"),
            (json!({"type": "scopes", "variant": "c", "scopes": [{"name": "b", "kind": "builtins"}]}), "ámbito de Python"),
            (json!({"type": "scopes", "scopes": [{"name": "a", "vars": ["x", "x"]}]}), "dos veces"),
            (json!({"type": "scopes", "scopes": [{"name": "a", "parent": "z"}]}), "no es un ámbito anterior"),
            (json!({"type": "scopes", "code": "x", "line": 3, "scopes": [{"name": "a"}]}), "de 1 a 1"),
            (json!({"type": "scopes", "variant": "c", "scopes": [{"name": "a", "vars": [{"name": "x", "declare": "global"}]}]}), "no existe en esta variante"),
        ];
        for (b, esperado) in casos {
            let e = svg(b.clone()).unwrap_err().0;
            assert!(e.starts_with("scopes: ") && e.contains(esperado), "{b}: {e}");
        }
    }
}
