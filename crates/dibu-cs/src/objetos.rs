//! Modelo de memoria de un lenguaje con referencias (`object_heap`): los marcos activos en el
//! stack, los objetos en el heap y las referencias entre ellos, como se enseña en Java.
//!
//! Cada objeto tiene un ID; las variables, los campos y las celdas guardan un valor (`value`) o
//! una referencia (`ref`, que puede ser `null`). dibu calcula qué objetos son alcanzables desde el
//! stack y marca los demás. Con `snapshots`, varios instantes apilados (o lado a lado).
//!
//! La variante `c` agrega lo propio de C: punteros a variables (con `id`), a campos (`B.sig`) y a
//! celdas (`A[2]`, y `A[n]`, el límite excluido); objetos automáticos dentro de su marco, estáticos
//! y del heap; bloques con la vida terminada (`ended`), punteros colgantes, valores
//! indeterminados, fugas y direcciones simbólicas.

use crate::common::{
    agregar_sub_lienzo, ancho_texto, encabezado, es_truthy, nombres_de_zonas, sub_lienzo, texto,
    zona, Bloque, ErrorDiagrama, Lienzo, Punto, ALTO_ZONA, MONO, SANS,
};
use dibu_core::Registro;
use serde_json::Value;
use std::collections::{HashMap, HashSet, VecDeque};

const TIPO: &str = "object_heap";
const MAX_VISTAS: usize = 6;
const MAX_MARCOS: usize = 8;
const MAX_OBJETOS: usize = 16;
const MAX_RANURAS: usize = 12;

const ALTO_CABECERA: f64 = 22.0;
const ALTO_FILA: f64 = 32.0;
const ALTO_CAJA: f64 = 24.0;
const ANCHO_CELDA: f64 = 46.0;
const ALTO_CELDA: f64 = 30.0;
const SEP_MARCOS: f64 = 10.0;
const SEP_OBJETOS: f64 = 22.0;
const PASO_CANAL: f64 = 9.0;
/// Lugar debajo de un objeto automático, dentro de su marco, para los carriles de las flechas.
const BAJO_AUTOMATICO: f64 = 16.0;

const VIDA_TERMINADA: &str = "vida terminada";
const FUGA: &str = "fuga";
const COLGANTE: &str = "colgante";
const INDETERMINADO: &str = "indeterminado";
const ESTATICO: &str = "estático";

pub fn registrar(reg: &mut Registro) {
    reg.registrar("object_heap", render_object_heap);
}

// ---------------------------------------------------------------------------
// Modelo
// ---------------------------------------------------------------------------

#[derive(Clone, Copy, Debug, PartialEq)]
enum Lenguaje {
    Java,
    Python,
    C,
}

#[derive(Clone, Debug, PartialEq)]
enum Valor {
    Dato(String),
    /// `None` es una referencia nula.
    Ref(Option<String>),
}

#[derive(Clone, Debug)]
struct Ranura {
    nombre: String,
    tipo: Option<String>,
    valor: Valor,
    en_blanco: bool,
    /// En C, una variable con `id` es un objeto al que puede apuntar un puntero.
    id: Option<String>,
    /// En C, una variable sin inicializar.
    indeterminado: bool,
}

impl Ranura {
    fn rotulo(&self) -> String {
        match &self.tipo {
            Some(t) if !t.is_empty() => declaracion(t, &self.nombre),
            _ => self.nombre.clone(),
        }
    }
}

/// `int` y `x` dan `int x`; `char *` y `p`, `char *p`; `int[3]` y `a`, `int a[3]`.
fn declaracion(tipo: &str, nombre: &str) -> String {
    if lenguaje() == Lenguaje::C {
        if let Some(i) = tipo.find('[') {
            return format!("{} {nombre}{}", tipo[..i].trim_end(), &tipo[i..]);
        }
        if tipo.ends_with('*') {
            return format!("{tipo}{nombre}");
        }
    }
    format!("{tipo} {nombre}")
}

#[derive(Debug)]
struct Marco {
    funcion: String,
    ranuras: Vec<Ranura>,
}

#[derive(Debug)]
enum Contenido {
    Campos(Vec<Ranura>),
    Elementos(Vec<Ranura>),
    Texto(String),
}

/// Dónde vive un objeto de C: en el heap, en almacenamiento estático o en un marco.
#[derive(Clone, Debug, PartialEq)]
enum Almacen {
    Heap,
    Estatico,
    /// El índice del marco (0 es `main`); `None` si el marco ya terminó.
    Automatico(Option<usize>, String),
}

#[derive(Debug)]
struct Objeto {
    id: String,
    tipo: String,
    contenido: Contenido,
    grupo: Option<String>,
    almacen: Almacen,
    terminado: bool,
    direccion: Option<String>,
}

impl Objeto {
    fn ranuras(&self) -> &[Ranura] {
        match &self.contenido {
            Contenido::Campos(r) | Contenido::Elementos(r) => r,
            Contenido::Texto(_) => &[],
        }
    }

    fn rotulo(&self) -> String {
        if self.tipo.is_empty() {
            self.id.clone()
        } else if self.almacen != Almacen::Heap {
            // Un objeto automático o estático de C es una variable: se rotula con su declaración.
            declaracion(&self.tipo, &self.id)
        } else {
            format!("{} : {}", self.id, self.tipo)
        }
    }

    fn en_stack(&self) -> bool {
        matches!(self.almacen, Almacen::Automatico(..))
    }
}

#[derive(Debug)]
struct Vista {
    rotulo: Option<String>,
    marcos: Vec<Marco>,
    objetos: Vec<Objeto>,
    /// Los marcos que ya terminaron y conservan objetos con la vida terminada.
    terminados: Vec<String>,
}

// ---------------------------------------------------------------------------
// Lectura
// ---------------------------------------------------------------------------

fn error(msg: impl Into<String>) -> ErrorDiagrama {
    ErrorDiagrama::new(msg)
}

/// Una variable, un campo o una celda: `{name, type, value | ref, blank}`; en las celdas, también
/// un valor suelto (`3`, `"x"`). En C, además `id` e `indeterminate`.
fn leer_ranura(dato: &Value, nombre_por_omision: &str, donde: &str) -> Result<Ranura, ErrorDiagrama> {
    let Some(obj) = dato.as_object() else {
        return Ok(Ranura {
            nombre: nombre_por_omision.to_string(),
            tipo: None,
            valor: Valor::Dato(if dato.is_null() { dato_nulo() } else { mostrar_celda(dato) }),
            en_blanco: false,
            id: None,
            indeterminado: false,
        });
    };
    let nombre = obj.get("name").map(texto).unwrap_or_else(|| nombre_por_omision.to_string());
    if nombre.is_empty() {
        return Err(error(format!("{donde}: falta 'name'")));
    }
    let tipo = obj.get("type").filter(|v| !v.is_null()).map(texto);
    let c = lenguaje() == Lenguaje::C;
    let indeterminado = c && obj.get("indeterminate").is_some_and(es_truthy);
    let valor = match (obj.get("value"), obj.contains_key("ref")) {
        (Some(_), true) => {
            return Err(error(format!(
                "{donde}: '{nombre}' tiene 'value' y 'ref' (un valor o una referencia, no los dos)"
            )));
        }
        _ if indeterminado && (obj.contains_key("value") || obj.contains_key("ref")) => {
            return Err(error(format!(
                "{donde}: '{nombre}' es 'indeterminate': no lleva 'value' ni 'ref'"
            )));
        }
        (_, true) => match obj.get("ref") {
            Some(Value::Null) | None => Valor::Ref(None),
            Some(r) => Valor::Ref(Some(texto(r))),
        },
        (Some(v), false) => Valor::Dato(if v.is_null() { dato_nulo() } else { mostrar_dato(v) }),
        (None, false) => Valor::Dato(String::new()),
    };
    let id = if c { obj.get("id").filter(|v| !v.is_null()).map(texto).filter(|s| !s.is_empty()) } else { None };
    Ok(Ranura {
        nombre,
        tipo,
        valor,
        en_blanco: obj.get("blank").is_some_and(es_truthy),
        id,
        indeterminado,
    })
}

thread_local! {
    /// La variante: en Python los valores se escriben como en Python (`'texto'`, `True`, `None`);
    /// en C, tal cual y la referencia nula es `NULL`.
    static LENGUAJE: std::cell::Cell<Lenguaje> = const { std::cell::Cell::new(Lenguaje::Java) };
}

fn lenguaje() -> Lenguaje {
    LENGUAJE.with(|p| p.get())
}

fn es_python() -> bool {
    lenguaje() == Lenguaje::Python
}

/// Un texto se muestra entre comillas, como en el código; los números y booleanos, tal cual. En
/// C, todo tal cual (`'a'`, `0x10`, `"hola"`): el valor se escribe como en el código.
fn mostrar_dato(v: &Value) -> String {
    match lenguaje() {
        Lenguaje::Python => match v {
            Value::Null => "None".to_string(),
            Value::Bool(true) => "True".to_string(),
            Value::Bool(false) => "False".to_string(),
            Value::String(s) if s.contains('\'') && !s.contains('"') => format!("\"{s}\""),
            Value::String(s) => format!("'{}'", s.replace('\'', "\\'")),
            otro => texto(otro),
        },
        Lenguaje::C => texto(v),
        Lenguaje::Java => match v {
            Value::String(s) => format!("\"{s}\""),
            otro => texto(otro),
        },
    }
}

/// Un valor suelto de una celda (`elements: [1, "a"]`): tal cual en Java y C, como `repr` en Python.
fn mostrar_celda(v: &Value) -> String {
    if es_python() { mostrar_dato(v) } else { texto(v) }
}

/// Una referencia nula: `null` en Java, `None` en Python, `NULL` en C.
fn nulo() -> &'static str {
    match lenguaje() {
        Lenguaje::Java => "null",
        Lenguaje::Python => "None",
        Lenguaje::C => "NULL",
    }
}

/// Un valor `null` declarado como dato: vacío en Java y C, `None` en Python.
fn dato_nulo() -> String {
    if es_python() { "None".to_string() } else { String::new() }
}

fn leer_ranuras(dato: Option<&Value>, donde: &str, celdas: bool) -> Result<Vec<Ranura>, ErrorDiagrama> {
    let Some(v) = dato.filter(|v| !v.is_null()) else {
        return Ok(Vec::new());
    };
    let ranuras: Vec<Ranura> = match v {
        Value::Array(lista) => lista
            .iter()
            .enumerate()
            .map(|(i, d)| leer_ranura(d, &if celdas { format!("[{i}]") } else { String::new() }, donde))
            .collect::<Result<_, _>>()?,
        // `vars: {a: 3, b: 4}`: sólo valores, como en call_stack.
        Value::Object(mapa) if !celdas => mapa
            .iter()
            .map(|(k, val)| {
                Ok(Ranura {
                    nombre: k.clone(),
                    tipo: None,
                    valor: Valor::Dato(if val.is_null() { dato_nulo() } else { mostrar_dato(val) }),
                    en_blanco: false,
                    id: None,
                    indeterminado: false,
                })
            })
            .collect::<Result<_, ErrorDiagrama>>()?,
        _ => {
            return Err(error(format!(
                "{donde}: se espera una lista{}",
                if celdas { "" } else { " de {name, value | ref}" }
            )));
        }
    };
    if ranuras.len() > MAX_RANURAS {
        return Err(error(format!("{donde}: hay {} entradas; el máximo es {MAX_RANURAS}", ranuras.len())));
    }
    Ok(ranuras)
}

fn leer_marcos(dato: Option<&Value>, donde: &str) -> Result<Vec<Marco>, ErrorDiagrama> {
    let Some(v) = dato.filter(|v| !v.is_null()) else {
        return Ok(Vec::new());
    };
    let Some(lista) = v.as_array() else {
        return Err(error(format!("{donde}: 'frames' tiene que ser una lista de marcos (de main al tope)")));
    };
    if lista.len() > MAX_MARCOS {
        return Err(error(format!("{donde}: hay {} marcos; el máximo es {MAX_MARCOS}", lista.len())));
    }
    lista
        .iter()
        .enumerate()
        .map(|(i, m)| {
            let Some(obj) = m.as_object() else {
                return Err(error(format!("{donde}: el marco {i} tiene que ser un objeto con 'function'")));
            };
            let funcion = obj.get("function").map(texto).unwrap_or_default();
            if funcion.is_empty() {
                return Err(error(format!("{donde}: el marco {i} no tiene 'function'")));
            }
            let lugar = format!("{donde}, marco {funcion}");
            let mut ranuras = leer_ranuras(obj.get("params"), &lugar, false)?;
            ranuras.extend(leer_ranuras(obj.get("vars"), &lugar, false)?);
            if ranuras.len() > MAX_RANURAS {
                return Err(error(format!("{lugar}: hay {} variables; el máximo es {MAX_RANURAS}", ranuras.len())));
            }
            Ok(Marco { funcion, ranuras })
        })
        .collect()
}

/// `storage` y `frame` de un objeto de C. El marco se busca por su índice (0 es `main`) o por el
/// nombre de la función; un marco que no está es uno que ya terminó.
fn leer_almacen(obj: &serde_json::Map<String, Value>, marcos: &[Marco], lugar: &str) -> Result<Almacen, ErrorDiagrama> {
    let almacen = obj.get("storage").filter(|v| !v.is_null()).map(texto);
    match almacen.as_deref() {
        None | Some("heap") => {
            if obj.contains_key("frame") {
                return Err(error(format!("{lugar}: 'frame' es sólo para 'storage: auto'")));
            }
            Ok(Almacen::Heap)
        }
        Some("static") => Ok(Almacen::Estatico),
        Some("auto") => {
            let Some(m) = obj.get("frame").filter(|v| !v.is_null()) else {
                return Err(error(format!("{lugar}: un objeto 'storage: auto' necesita 'frame' (la función o su índice)")));
            };
            if let Some(i) = m.as_u64() {
                let i = i as usize;
                if i >= marcos.len() {
                    return Err(error(format!("{lugar}: no hay un marco {i} (hay {})", marcos.len())));
                }
                return Ok(Almacen::Automatico(Some(i), marcos[i].funcion.clone()));
            }
            let nombre = texto(m);
            // El de más arriba, si la función está más de una vez (recursión).
            let indice = marcos.iter().rposition(|mc| mc.funcion == nombre);
            Ok(Almacen::Automatico(indice, nombre))
        }
        Some(otro) => Err(error(format!("{lugar}: 'storage' es heap, static o auto, no '{otro}'"))),
    }
}

fn leer_objetos(dato: Option<&Value>, marcos: &[Marco], donde: &str) -> Result<Vec<Objeto>, ErrorDiagrama> {
    let Some(v) = dato.filter(|v| !v.is_null()) else {
        return Ok(Vec::new());
    };
    let Some(lista) = v.as_array() else {
        return Err(error(format!("{donde}: 'objects' tiene que ser una lista de objetos con 'id'")));
    };
    if lista.len() > MAX_OBJETOS {
        return Err(error(format!("{donde}: hay {} objetos; el máximo es {MAX_OBJETOS}", lista.len())));
    }
    let c = lenguaje() == Lenguaje::C;
    let mut objetos = Vec::with_capacity(lista.len());
    for (i, o) in lista.iter().enumerate() {
        let Some(obj) = o.as_object() else {
            return Err(error(format!("{donde}: el objeto {i} tiene que tener 'id'")));
        };
        let id = obj.get("id").filter(|v| !v.is_null()).map(texto).unwrap_or_default();
        if id.is_empty() {
            return Err(error(format!("{donde}: el objeto {i} no tiene 'id'")));
        }
        let lugar = format!("{donde}, objeto {id}");
        let claves = ["fields", "elements", "value"].iter().filter(|k| obj.contains_key(**k)).count();
        if claves > 1 {
            return Err(error(format!("{lugar}: usar sólo uno de 'fields', 'elements' o 'value'")));
        }
        let contenido = if obj.contains_key("elements") {
            Contenido::Elementos(leer_ranuras(obj.get("elements"), &lugar, true)?)
        } else if let Some(v) = obj.get("value") {
            Contenido::Texto(if v.is_null() { dato_nulo() } else { mostrar_dato(v) })
        } else {
            Contenido::Campos(leer_ranuras(obj.get("fields"), &lugar, false)?)
        };
        let almacen = if c { leer_almacen(obj, marcos, &lugar)? } else { Almacen::Heap };
        let terminado = c && obj.get("ended").is_some_and(es_truthy);
        if let Almacen::Automatico(None, f) = &almacen {
            if !terminado {
                return Err(error(format!(
                    "{lugar}: el marco '{f}' no está en 'frames'; si ya terminó, el objeto lleva 'ended: true'"
                )));
            }
        }
        let grupo = obj.get("group").filter(|v| es_truthy(v)).map(texto);
        objetos.push(Objeto {
            id,
            tipo: obj.get("type").filter(|v| !v.is_null()).map(texto).unwrap_or_default(),
            contenido,
            grupo: if almacen == Almacen::Estatico { grupo.or_else(|| Some(ESTATICO.to_string())) } else { grupo },
            almacen,
            terminado,
            direccion: if c { obj.get("address").filter(|v| !v.is_null()).map(texto) } else { None },
        });
    }
    Ok(objetos)
}

fn leer_vista(obj: &Bloque, donde: &str) -> Result<Vista, ErrorDiagrama> {
    let marcos = leer_marcos(obj.get("frames"), donde)?;
    let objetos = leer_objetos(obj.get("objects"), &marcos, donde)?;
    let mut terminados: Vec<String> = Vec::new();
    for o in &objetos {
        if let Almacen::Automatico(None, f) = &o.almacen {
            if !terminados.contains(f) {
                terminados.push(f.clone());
            }
        }
    }
    let vista = Vista {
        rotulo: obj.get("label").filter(|v| es_truthy(v)).map(texto),
        marcos,
        objetos,
        terminados,
    };
    if vista.marcos.is_empty() && vista.objetos.is_empty() {
        return Err(error(format!("{donde}: no hay 'frames' ni 'objects'")));
    }
    validar(&vista, donde)?;
    Ok(vista)
}

/// A qué apunta un destino de C: un objeto o una variable entera, un campo, una celda o el límite
/// excluido de un arreglo (`A[n]`).
#[derive(Clone, Debug, PartialEq)]
enum Parte {
    Entero,
    Campo(String),
    Celda(usize),
}

/// `B`, `B.sig` o `A[2]`.
fn partir_destino(destino: &str) -> (&str, Option<Parte>) {
    if let Some((base, resto)) = destino.split_once('[') {
        if let Some(n) = resto.strip_suffix(']').and_then(|n| n.trim().parse::<usize>().ok()) {
            return (base.trim(), Some(Parte::Celda(n)));
        }
        return (destino, None);
    }
    if let Some((base, campo)) = destino.split_once('.') {
        return (base.trim(), Some(Parte::Campo(campo.trim().to_string())));
    }
    (destino.trim(), Some(Parte::Entero))
}

/// IDs únicos y referencias a objetos que existen (en C, también a variables, campos y celdas).
fn validar(v: &Vista, donde: &str) -> Result<(), ErrorDiagrama> {
    let c = lenguaje() == Lenguaje::C;
    let mut ids = HashSet::new();
    for o in &v.objetos {
        if !ids.insert(o.id.as_str()) {
            return Err(error(format!("{donde}: el ID '{}' está repetido", o.id)));
        }
    }
    let mut variables: HashSet<&str> = HashSet::new();
    for r in v.marcos.iter().flat_map(|m| &m.ranuras) {
        if let Some(id) = &r.id {
            if !ids.insert(id.as_str()) {
                return Err(error(format!("{donde}: el ID '{id}' está repetido")));
            }
            variables.insert(id.as_str());
        }
    }
    if c {
        // Dos objetos vivos no ocupan la misma dirección; un bloque terminado sí puede compartirla
        // con el que la reutiliza, y siguen siendo dos objetos.
        let mut direcciones: HashMap<&str, &str> = HashMap::new();
        for o in v.objetos.iter().filter(|o| !o.terminado) {
            if let Some(d) = &o.direccion {
                if let Some(otro) = direcciones.insert(d.as_str(), o.id.as_str()) {
                    return Err(error(format!(
                        "{donde}: '{otro}' y '{}' están vivos en la misma dirección {d}",
                        o.id
                    )));
                }
            }
        }
    }
    let por_id: HashMap<&str, &Objeto> = v.objetos.iter().map(|o| (o.id.as_str(), o)).collect();
    let ranuras = v
        .marcos
        .iter()
        .flat_map(|m| m.ranuras.iter().map(move |r| (format!("{}.{}", m.funcion, r.nombre), r)))
        .chain(v.objetos.iter().flat_map(|o| o.ranuras().iter().map(move |r| (format!("{}.{}", o.id, r.nombre), r))));
    for (lugar, r) in ranuras {
        let Valor::Ref(Some(destino)) = &r.valor else {
            continue;
        };
        if !c {
            if !por_id.contains_key(destino.as_str()) {
                return Err(error(format!("{donde}: {lugar} refiere a '{destino}', que no está en 'objects'")));
            }
            continue;
        }
        let (base, parte) = partir_destino(destino);
        let Some(parte) = parte else {
            return Err(error(format!("{donde}: {lugar} apunta a '{destino}': se espera ID, ID.campo o ID[i]")));
        };
        if variables.contains(base) {
            if parte != Parte::Entero {
                return Err(error(format!(
                    "{donde}: {lugar} apunta a '{destino}', pero '{base}' es una variable simple; declararla como objeto 'storage: auto'"
                )));
            }
            continue;
        }
        let Some(o) = por_id.get(base) else {
            return Err(error(format!(
                "{donde}: {lugar} apunta a '{destino}', y '{base}' no es un objeto ni una variable con 'id'"
            )));
        };
        match (&parte, &o.contenido) {
            (Parte::Entero, _) => {}
            (Parte::Campo(campo), Contenido::Campos(campos)) => {
                if !campos.iter().any(|r| &r.nombre == campo) {
                    return Err(error(format!("{donde}: {lugar} apunta a '{destino}', y '{base}' no tiene el campo '{campo}'")));
                }
            }
            (Parte::Celda(k), Contenido::Elementos(celdas)) => {
                if *k > celdas.len() {
                    return Err(error(format!(
                        "{donde}: {lugar} apunta a '{destino}', fuera de '{base}' (tiene {} celdas; {} es el límite)",
                        celdas.len(),
                        celdas.len()
                    )));
                }
            }
            (Parte::Campo(_), _) => {
                return Err(error(format!("{donde}: {lugar} apunta a '{destino}', pero '{base}' no tiene campos")));
            }
            (Parte::Celda(_), _) => {
                return Err(error(format!("{donde}: {lugar} apunta a '{destino}', pero '{base}' no es un arreglo")));
            }
        }
    }
    Ok(())
}

/// Los objetos a los que se llega desde el stack (y, en C, desde los objetos estáticos y
/// automáticos), siguiendo referencias. Un bloque con la vida terminada no lleva a ningún lado.
fn alcanzables(v: &Vista) -> HashSet<String> {
    let por_id: HashMap<&str, &Objeto> = v.objetos.iter().map(|o| (o.id.as_str(), o)).collect();
    let base = |d: &str| -> String { partir_destino(d).0.to_string() };
    let mut vistos = HashSet::new();
    let mut cola: VecDeque<String> = v
        .marcos
        .iter()
        .flat_map(|m| &m.ranuras)
        .chain(v.objetos.iter().filter(|o| o.almacen != Almacen::Heap && !o.terminado).flat_map(|o| o.ranuras()))
        .filter_map(|r| match &r.valor {
            Valor::Ref(Some(d)) => Some(base(d)),
            _ => None,
        })
        .collect();
    while let Some(id) = cola.pop_front() {
        if !vistos.insert(id.clone()) {
            continue;
        }
        if let Some(o) = por_id.get(id.as_str()).filter(|o| !o.terminado) {
            for r in o.ranuras() {
                if let Valor::Ref(Some(d)) = &r.valor {
                    cola.push_back(base(d));
                }
            }
        }
    }
    vistos
}

// ---------------------------------------------------------------------------
// Dibujo
// ---------------------------------------------------------------------------

struct Opciones {
    sin_valores: bool,
    sin_refs: bool,
    sin_estados: bool,
    marcar: bool,
    zonas: Option<(String, String)>,
}

fn ancho_valor(r: &Ranura) -> f64 {
    if r.indeterminado {
        return ancho_texto(INDETERMINADO, 9.5) + 14.0;
    }
    match &r.valor {
        Valor::Dato(d) => ancho_texto(d, 11.5) + 16.0,
        Valor::Ref(None) => ancho_texto(nulo(), 11.0) + 16.0,
        Valor::Ref(Some(_)) => 30.0,
    }
}

/// Dibuja el contenido de una caja de valor; devuelve el punto de salida de la referencia, si hay.
fn dibujar_valor(c: &mut Lienzo, r: &Ranura, x: f64, y: f64, w: f64, h: f64, op: &Opciones) -> Option<(Punto, String)> {
    let cx = x + w / 2.0;
    let cy = y + h / 2.0;
    if r.indeterminado {
        if r.en_blanco || op.sin_valores {
            c.vacio(x + 6.0, x + w - 6.0, y + h - 6.0);
        } else {
            // En una celda angosta no entra la palabra entera.
            let rotulo = if ancho_texto(INDETERMINADO, 9.5) + 8.0 > w { "indet." } else { INDETERMINADO };
            c.t(cx, cy + 3.5, rotulo, 9.5, 400, "textMuted", "middle", SANS);
        }
        return None;
    }
    match &r.valor {
        Valor::Dato(d) => {
            if d.is_empty() || r.en_blanco || op.sin_valores {
                c.vacio(x + 6.0, x + w - 6.0, y + h - 6.0);
            } else {
                c.t(cx, cy + 4.5, d, 11.5, 700, "textMain", "middle", MONO);
            }
            None
        }
        _ if r.en_blanco || op.sin_refs => {
            c.vacio(x + 6.0, x + w - 6.0, y + h - 6.0);
            None
        }
        Valor::Ref(None) => {
            c.t(cx, cy + 4.0, nulo(), 11.0, 700, "textMuted", "middle", MONO);
            None
        }
        Valor::Ref(Some(destino)) => {
            c.punto(cx, cy, "accentPrimary", 3.2);
            Some(((cx, cy), destino.clone()))
        }
    }
}

/// Una referencia a dibujar: desde dónde, a qué objeto y si sale del stack o de otro objeto.
struct Flecha {
    desde: Punto,
    destino: String,
    del_stack: bool,
    /// En una celda de un arreglo, la flecha baja antes de salir hacia la derecha.
    bajo: Option<f64>,
}

/// Dónde está un destino posible de una flecha.
#[derive(Clone, Debug)]
struct Destino {
    x: f64,
    y: f64,
    w: f64,
    h: f64,
    parte: Parte,
    en_stack: bool,
    /// El objeto (o la variable) al que pertenece, y el borde inferior de su caja.
    base: String,
    fondo: f64,
}

fn medir_marco(m: &Marco) -> (f64, f64) {
    let nombre = titulo_marco(&m.funcion);
    let rotulos = m.ranuras.iter().map(|r| ancho_texto(&r.rotulo(), 10.5)).fold(0.0, f64::max);
    let valores = m.ranuras.iter().map(ancho_valor).fold(36.0, f64::max);
    let w = (rotulos + valores + 34.0).max(ancho_texto(&nombre, 11.0) + 20.0).max(110.0);
    (w, valores)
}

fn titulo_marco(funcion: &str) -> String {
    // `main` pasa a `main()`; `<módulo>` y `f(x)` quedan como están.
    if funcion.contains('(') || funcion.starts_with('<') {
        funcion.to_string()
    } else {
        format!("{funcion}()")
    }
}

fn medir_objeto(o: &Objeto) -> (f64, f64) {
    let direccion = o.direccion.as_ref().map(|d| ancho_texto(d, 9.5) + 12.0).unwrap_or(0.0);
    let cabecera = ancho_texto(&o.rotulo(), 10.5) + 20.0 + direccion;
    match &o.contenido {
        Contenido::Elementos(e) => {
            let w = (e.len().max(1) as f64 * ANCHO_CELDA + 16.0).max(cabecera).max(70.0);
            (w, ALTO_CABECERA + 18.0 + ALTO_CELDA + 8.0)
        }
        Contenido::Campos(campos) => {
            let rotulos = campos.iter().map(|r| ancho_texto(&r.rotulo(), 10.5)).fold(0.0, f64::max);
            let valores = campos.iter().map(ancho_valor).fold(36.0, f64::max);
            let w = (rotulos + valores + 34.0).max(cabecera).max(90.0);
            (w, ALTO_CABECERA + campos.len().max(1) as f64 * ALTO_FILA + 4.0)
        }
        Contenido::Texto(t) => {
            let w = (ancho_texto(t, 11.5) + 28.0).max(cabecera).max(70.0);
            (w, ALTO_CABECERA + ALTO_FILA + 4.0)
        }
    }
}

/// Los objetos en el orden de dibujo: los de un mismo grupo, juntos, donde aparece el primero.
fn orden_objetos(objetos: &[&Objeto]) -> Vec<usize> {
    let mut orden = Vec::with_capacity(objetos.len());
    let mut grupos_vistos = HashSet::new();
    for (i, o) in objetos.iter().enumerate() {
        match &o.grupo {
            None => orden.push(i),
            Some(g) => {
                if grupos_vistos.insert(g.clone()) {
                    orden.extend(objetos.iter().enumerate().filter(|(_, p)| p.grupo.as_ref() == Some(g)).map(|(j, _)| j));
                }
            }
        }
    }
    orden
}

/// La ubicación de los objetos, común a todos los instantes: un mismo ID queda en el mismo
/// lugar, así sólo cambian las flechas que cambiaron.
struct Disposicion {
    /// Desplazamiento vertical de cada ID desde el comienzo del heap.
    y: HashMap<String, f64>,
    grupos: Vec<(String, f64, f64, HashSet<String>)>,
    ancho_stack: f64,
    ancho_objetos: f64,
}

/// Lo que ocupan en un marco sus objetos automáticos.
fn alto_automaticos<'a>(objetos: impl Iterator<Item = &'a Objeto>) -> f64 {
    objetos.map(|o| medir_objeto(o).1 + BAJO_AUTOMATICO + 6.0).sum()
}

fn automaticos_de<'a>(v: &'a Vista, marco: Option<usize>, nombre: &'a str) -> impl Iterator<Item = &'a Objeto> + 'a {
    v.objetos.iter().filter(move |o| match &o.almacen {
        Almacen::Automatico(Some(i), _) => marco == Some(*i),
        Almacen::Automatico(None, f) => marco.is_none() && f == nombre,
        _ => false,
    })
}

fn disponer(vistas: &[Vista]) -> Disposicion {
    let mut unicos: Vec<&Objeto> = Vec::new();
    let mut medida: HashMap<&str, (f64, f64)> = HashMap::new();
    let mut ancho_stack = 0.0_f64;
    for v in vistas {
        for m in &v.marcos {
            ancho_stack = ancho_stack.max(medir_marco(m).0);
        }
        for f in &v.terminados {
            ancho_stack = ancho_stack.max(ancho_texto(&rotulo_terminado(f), 11.0) + 20.0);
        }
    }
    for o in vistas.iter().flat_map(|v| &v.objetos) {
        let (w, h) = medir_objeto(o);
        if o.en_stack() {
            let estado = if o.terminado { ancho_texto(VIDA_TERMINADA, 9.5) + 12.0 } else { 0.0 };
            ancho_stack = ancho_stack.max(w + estado + 16.0);
            continue;
        }
        let m = medida.entry(o.id.as_str()).or_insert((0.0, 0.0));
        *m = (m.0.max(w), m.1.max(h));
        if !unicos.iter().any(|u| u.id == o.id) {
            unicos.push(o);
        }
    }
    let mut y_de = HashMap::new();
    let mut grupos = Vec::new();
    let mut y = 0.0_f64;
    let mut actual: Option<(String, f64, HashSet<String>)> = None;
    for i in orden_objetos(&unicos) {
        let o = unicos[i];
        if actual.as_ref().map(|g| &g.0) != o.grupo.as_ref() {
            if let Some((g, y0, ids)) = actual.take() {
                grupos.push((g, y0, y - SEP_OBJETOS + 8.0, ids));
                y += 6.0;
            }
            if let Some(g) = &o.grupo {
                actual = Some((g.clone(), y, HashSet::new()));
                y += 20.0;
            }
        }
        if let Some((_, _, ids)) = actual.as_mut() {
            ids.insert(o.id.clone());
        }
        y_de.insert(o.id.clone(), y);
        y += medida[o.id.as_str()].1 + SEP_OBJETOS;
    }
    if let Some((g, y0, ids)) = actual.take() {
        grupos.push((g, y0, y - SEP_OBJETOS + 8.0, ids));
    }
    Disposicion {
        y: y_de,
        grupos,
        ancho_stack,
        ancho_objetos: medida.values().map(|m| m.0).fold(0.0, f64::max),
    }
}

fn rotulo_terminado(funcion: &str) -> String {
    format!("{} (terminó)", titulo_marco(funcion))
}

/// Qué se dibuja junto a la cabecera de un objeto: su estado, si tiene uno.
fn estado_objeto(o: &Objeto, vivos: &HashSet<String>, v: &Vista, op: &Opciones) -> Option<&'static str> {
    let c = lenguaje() == Lenguaje::C;
    if o.terminado {
        return Some(VIDA_TERMINADA);
    }
    if op.marcar && !v.marcos.is_empty() && o.almacen == Almacen::Heap && !vivos.contains(&o.id) {
        return Some(if c { FUGA } else { "inalcanzable" });
    }
    None
}

/// Dibuja un objeto en su caja, registra sus destinos y devuelve las referencias que salen de él.
#[allow(clippy::too_many_arguments)]
fn dibujar_objeto(
    c: &mut Lienzo,
    o: &Objeto,
    (x, y, w, h): (f64, f64, f64, f64),
    estado: Option<&str>,
    op: &Opciones,
    destinos: &mut HashMap<String, Destino>,
    flechas: &mut Vec<Flecha>,
    carriles_bajo: &mut HashMap<String, usize>,
) {
    let en_stack = o.en_stack();
    let terminado = o.terminado;
    let fondo = y + h;
    let destino = |parte: Parte, (dx, dy, dw, dh): (f64, f64, f64, f64)| Destino {
        x: dx,
        y: dy,
        w: dw,
        h: dh,
        parte,
        en_stack,
        base: o.id.clone(),
        fondo,
    };
    destinos.insert(o.id.clone(), destino(Parte::Entero, (x, y, w, h)));
    let estilo_terminado = terminado && !op.sin_estados;
    if estilo_terminado {
        c.add(format!(
            r#"<rect x="{x:.1}" y="{y:.1}" width="{w:.1}" height="{h:.1}" rx="3" fill="none" stroke="{}" stroke-width="1.6" stroke-dasharray="6 3"/>"#,
            c.pal.stroke_muted
        ));
        c.add(format!(
            r#"<rect x="{x:.1}" y="{y:.1}" width="{w:.1}" height="{ALTO_CABECERA:.1}" rx="3" fill="none" stroke="{}" stroke-width="1.2" stroke-dasharray="6 3"/>"#,
            c.pal.stroke_muted
        ));
    } else {
        c.r(x, y, w, h, "fillCell", 1.6, 3.0);
        c.r(x, y, w, ALTO_CABECERA, "fillHeader", 1.6, 3.0);
    }
    let color_rotulo = if estilo_terminado { "textMuted" } else { "textMain" };
    c.t(x + 8.0, y + 15.5, &o.rotulo(), 10.5, 700, color_rotulo, "start", MONO);
    if let Some(d) = &o.direccion {
        c.t(x + w - 8.0, y + 15.0, d, 9.5, 400, "textSub", "end", MONO);
    }
    if op.sin_estados && lenguaje() == Lenguaje::C {
        // Para completar: un renglón de estado en cada objeto fuera del stack.
        if !en_stack {
            c.t(x + w + 8.0, y + 15.0, "estado:", 9.5, 700, "textMuted", "start", SANS);
            let x0 = x + w + 12.0 + ancho_texto("estado:", 9.5);
            c.vacio(x0, x0 + 52.0, y + 17.0);
        }
    } else if let Some(e) = estado {
        if e != VIDA_TERMINADA {
            c.add(format!(
                r#"<rect x="{:.1}" y="{:.1}" width="{:.1}" height="{:.1}" rx="5" fill="none" stroke="{}" stroke-width="1.4" stroke-dasharray="5 3"/>"#,
                x - 4.0,
                y - 4.0,
                w + 8.0,
                h + 8.0,
                c.pal.stroke_muted
            ));
        }
        if en_stack {
            // Dentro del marco no hay lugar a la derecha: el estado va en la cabecera.
            let xr = x + w - 8.0 - o.direccion.as_ref().map(|d| ancho_texto(d, 9.5) + 8.0).unwrap_or(0.0);
            c.t(xr, y + 15.0, e, 9.5, 700, "textMuted", "end", SANS);
        } else {
            c.t(x + w + 8.0, y + 15.0, e, 9.5, 700, "textMuted", "start", SANS);
        }
    }
    // Una caja sin datos legibles: la memoria terminada no tiene un valor que leer.
    let vacia = |c: &mut Lienzo, bx: f64, by: f64, bw: f64, bh: f64, radio: f64| {
        if estilo_terminado {
            c.add(format!(
                r#"<rect x="{bx:.1}" y="{by:.1}" width="{bw:.1}" height="{bh:.1}" fill="none" stroke="{}" stroke-width="1.1" stroke-dasharray="3 3"/>"#,
                c.pal.stroke_muted
            ));
        } else {
            c.r(bx, by, bw, bh, "fillPointerBox", 1.4, radio);
        }
    };
    match &o.contenido {
        Contenido::Elementos(e) => {
            // Los índices van arriba: abajo salen las referencias de las celdas.
            let cy = y + ALTO_CABECERA + 18.0;
            if e.is_empty() {
                c.t(x + 10.0, cy + 19.0, "length 0", 10.0, 400, "textSub", "start", MONO);
            }
            for (k, r) in e.iter().enumerate() {
                let cx = x + 8.0 + k as f64 * ANCHO_CELDA;
                destinos.insert(format!("{}[{k}]", o.id), destino(Parte::Celda(k), (cx, cy, ANCHO_CELDA, ALTO_CELDA)));
                vacia(c, cx, cy, ANCHO_CELDA, ALTO_CELDA, 0.0);
                c.t(cx + ANCHO_CELDA / 2.0, cy - 5.0, &format!("[{k}]"), 9.0, 400, "textSub", "middle", MONO);
                if terminado {
                    continue;
                }
                if let Some((desde, destino)) = dibujar_valor(c, r, cx, cy, ANCHO_CELDA, ALTO_CELDA, op) {
                    let n = carriles_bajo.entry(o.id.clone()).or_insert(0);
                    *n += 1;
                    let bajo = fondo + 2.0 + *n as f64 * 4.0;
                    flechas.push(Flecha { desde, destino, del_stack: en_stack, bajo: Some(bajo) });
                }
            }
            let fin = x + 8.0 + e.len() as f64 * ANCHO_CELDA;
            destinos.insert(format!("{}[{}]", o.id, e.len()), destino(Parte::Celda(e.len()), (fin, cy, 0.0, ALTO_CELDA)));
        }
        Contenido::Campos(campos) => {
            let ancho_valores = campos.iter().map(ancho_valor).fold(36.0, f64::max);
            for (k, r) in campos.iter().enumerate() {
                let fy = y + ALTO_CABECERA + k as f64 * ALTO_FILA + (ALTO_FILA - ALTO_CAJA) / 2.0;
                destinos.insert(
                    format!("{}.{}", o.id, r.nombre),
                    destino(Parte::Campo(r.nombre.clone()), (x, fy, w, ALTO_CAJA)),
                );
                c.t(x + 8.0, fy + ALTO_CAJA / 2.0 + 4.0, &r.rotulo(), 10.5, 400, color_rotulo, "start", MONO);
                let vx = x + w - ancho_valores - 8.0;
                vacia(c, vx, fy, ancho_valores, ALTO_CAJA, 2.0);
                if terminado {
                    continue;
                }
                if let Some((desde, destino)) = dibujar_valor(c, r, vx, fy, ancho_valores, ALTO_CAJA, op) {
                    flechas.push(Flecha { desde, destino, del_stack: en_stack, bajo: None });
                }
            }
        }
        Contenido::Texto(t) => {
            let fy = y + ALTO_CABECERA + (ALTO_FILA - ALTO_CAJA) / 2.0 + 2.0;
            if terminado {
            } else if t.is_empty() || op.sin_valores {
                c.vacio(x + 12.0, x + w - 12.0, fy + ALTO_CAJA - 4.0);
            } else {
                c.t(x + w / 2.0, fy + ALTO_CAJA / 2.0 + 4.0, t, 11.5, 700, "textMain", "middle", MONO);
            }
        }
    }
}

/// Lleva una flecha hasta una celda (o el límite de un arreglo) por debajo de su objeto, desde
/// la columna `gx`.
fn entrar_por_abajo(puntos: &mut Vec<Punto>, gx: f64, d: &Destino, carriles_bajo: &mut HashMap<String, usize>) {
    let n = carriles_bajo.entry(d.base.clone()).or_insert(0);
    *n += 1;
    let yb = d.fondo + 2.0 + *n as f64 * 4.0;
    let cx = d.x + d.w / 2.0;
    let ultimo = *puntos.last().unwrap();
    if ultimo.0 != gx {
        puntos.push((gx, ultimo.1));
    }
    puntos.push((gx, yb));
    puntos.push((cx, yb));
    puntos.push((cx, d.y + d.h + 2.0));
}

/// Una flecha por los puntos dados; las colgantes, con guiones y el rótulo en su tramo más largo.
fn trazar(c: &mut Lienzo, puntos: &[Punto], colgante: bool) {
    if puntos.len() < 2 {
        return;
    }
    if !colgante {
        if puntos.len() == 2 {
            c.linea(puntos[0].0, puntos[0].1, puntos[1].0, puntos[1].1, "accentPrimary", 1.6, Some("main"), "");
        } else {
            c.ruta(puntos);
        }
        return;
    }
    let camino: String = puntos.iter().map(|(x, y)| format!("{x:.1} {y:.1}")).collect::<Vec<_>>().join(" L ");
    c.add(format!(
        r#"<path d="M {camino}" fill="none" stroke="{}" stroke-width="1.5" stroke-dasharray="5 3" marker-end="url(#flecha-sub-{})" data-colgante="1"/>"#,
        c.pal.stroke_sub, c.sufijo
    ));
    let (a, b) = puntos
        .windows(2)
        .map(|p| (p[0], p[1]))
        .max_by(|p, q| {
            let l = |(a, b): &(Punto, Punto)| (a.0 - b.0).hypot(a.1 - b.1);
            l(p).total_cmp(&l(q))
        })
        .unwrap();
    if a.0 == b.0 {
        c.t(a.0 + 4.0, (a.1 + b.1) / 2.0 + 3.0, COLGANTE, 9.0, 700, "textMuted", "start", SANS);
    } else {
        c.t((a.0 + b.0) / 2.0, (a.1 + b.1) / 2.0 - 4.0, COLGANTE, 9.0, 700, "textMuted", "middle", SANS);
    }
}

fn dibujar_vista(c: &mut Lienzo, v: &Vista, top: f64, op: &Opciones, d: &Disposicion) -> Punto {
    let es_c = lenguaje() == Lenguaje::C;
    let alto_zona = if op.zonas.is_some() { ALTO_ZONA } else { 0.0 };
    let y_inicio = top + alto_zona + 8.0;
    let mut flechas: Vec<Flecha> = Vec::new();
    let mut destinos: HashMap<String, Destino> = HashMap::new();
    let mut carriles_bajo: HashMap<String, usize> = HashMap::new();
    let vivos = alcanzables(v);

    // Stack: del tope (arriba) a main (abajo), como en call_stack. Arriba de todo, los marcos que
    // ya terminaron y conservan objetos (en C, un local cuya dirección sobrevivió).
    let ancho_stack = d.ancho_stack;
    let mut y = y_inicio;
    for f in &v.terminados {
        let alto = ALTO_CABECERA + 6.0 + alto_automaticos(automaticos_de(v, None, f));
        c.add(format!(
            r#"<rect x="0" y="{y:.1}" width="{ancho_stack:.1}" height="{alto:.1}" rx="4" fill="none" stroke="{}" stroke-width="1.4" stroke-dasharray="6 3"/>"#,
            c.pal.stroke_muted
        ));
        c.t(8.0, y + 15.5, &rotulo_terminado(f), 11.0, 700, "textMuted", "start", MONO);
        let mut oy = y + ALTO_CABECERA + 6.0;
        for o in automaticos_de(v, None, f) {
            let (_, h) = medir_objeto(o);
            let estado = estado_objeto(o, &vivos, v, op);
            dibujar_objeto(c, o, (8.0, oy, ancho_stack - 16.0, h), estado, op, &mut destinos, &mut flechas, &mut carriles_bajo);
            oy += h + BAJO_AUTOMATICO + 6.0;
        }
        y += alto + SEP_MARCOS;
    }
    for (i, m) in v.marcos.iter().enumerate().rev() {
        let ancho_valores = medir_marco(m).1;
        let alto_filas = ALTO_CABECERA + m.ranuras.len().max(if es_c { 0 } else { 1 }) as f64 * ALTO_FILA + 4.0;
        let alto = (alto_filas + alto_automaticos(automaticos_de(v, Some(i), &m.funcion))).max(ALTO_CABECERA + ALTO_FILA + 4.0);
        let tope = i + 1 == v.marcos.len();
        c.r(0.0, y, ancho_stack, alto, "fillCell", if tope { 2.2 } else { 1.5 }, 4.0);
        c.r(0.0, y, ancho_stack, ALTO_CABECERA, "fillHeader", if tope { 2.2 } else { 1.5 }, 4.0);
        c.t(8.0, y + 15.5, &titulo_marco(&m.funcion), 11.0, 700, "textMain", "start", MONO);
        for (k, r) in m.ranuras.iter().enumerate() {
            let fy = y + ALTO_CABECERA + k as f64 * ALTO_FILA + (ALTO_FILA - ALTO_CAJA) / 2.0;
            c.t(8.0, fy + ALTO_CAJA / 2.0 + 4.0, &r.rotulo(), 10.5, 400, "textMain", "start", MONO);
            let vx = ancho_stack - ancho_valores - 8.0;
            c.r(vx, fy, ancho_valores, ALTO_CAJA, "fillPointerBox", 1.4, 2.0);
            if let Some(id) = &r.id {
                destinos.insert(
                    id.clone(),
                    Destino {
                        x: vx,
                        y: fy,
                        w: ancho_valores,
                        h: ALTO_CAJA,
                        parte: Parte::Entero,
                        en_stack: true,
                        base: id.clone(),
                        fondo: fy + ALTO_CAJA,
                    },
                );
            }
            if let Some((desde, destino)) = dibujar_valor(c, r, vx, fy, ancho_valores, ALTO_CAJA, op) {
                flechas.push(Flecha { desde, destino, del_stack: true, bajo: None });
            }
        }
        let mut oy = y + alto_filas + 2.0;
        for o in automaticos_de(v, Some(i), &m.funcion) {
            let (_, h) = medir_objeto(o);
            let estado = estado_objeto(o, &vivos, v, op);
            dibujar_objeto(c, o, (8.0, oy, ancho_stack - 16.0, h), estado, op, &mut destinos, &mut flechas, &mut carriles_bajo);
            oy += h + BAJO_AUTOMATICO + 6.0;
        }
        y += alto + SEP_MARCOS;
    }
    let fin_stack = if v.marcos.is_empty() && v.terminados.is_empty() { y_inicio } else { y - SEP_MARCOS };

    // Heap: los objetos en una columna; los de un grupo, dentro de un recuadro.
    let hueco = if op.zonas.is_some() { 110.0 } else { 90.0 };
    let x_heap = if d.ancho_stack == 0.0 { 14.0 } else { ancho_stack + hueco };
    let ancho_objetos = d.ancho_objetos;
    let mut fin_heap = y_inicio;
    let hay_grupos = !d.grupos.is_empty();
    let borde_heap = x_heap + ancho_objetos + if hay_grupos { 20.0 } else { 0.0 };
    for (g, y0, y1, ids) in &d.grupos {
        if v.objetos.iter().any(|o| ids.contains(&o.id)) {
            zona(c, x_heap - 2.0, borde_heap, y_inicio + y0, y_inicio + y1, g);
            fin_heap = fin_heap.max(y_inicio + y1);
        }
    }

    let mut ancho_marcas = 0.0_f64;
    for o in v.objetos.iter().filter(|o| !o.en_stack()) {
        let x = x_heap + if o.grupo.is_some() { 10.0 } else { 0.0 };
        let (w, h) = medir_objeto(o);
        let y = y_inicio + d.y[&o.id];
        fin_heap = fin_heap.max(y + h);
        let estado = estado_objeto(o, &vivos, v, op);
        if estado.is_some() || (op.sin_estados && es_c) {
            let rotulo = if es_c { VIDA_TERMINADA } else { "inalcanzable" };
            ancho_marcas = ancho_marcas.max(x_heap + ancho_objetos + 8.0 + ancho_texto(rotulo, 9.5) + 6.0);
        }
        dibujar_objeto(c, o, (x, y, w, h), estado, op, &mut destinos, &mut flechas, &mut carriles_bajo);
    }

    // Flechas: del stack, al borde izquierdo del objeto; entre objetos, por un canal a la derecha
    // del heap, al borde derecho. A una celda (o al límite de un arreglo) entran desde abajo; a una
    // variable del stack, por un canal en el hueco entre el stack y el heap.
    let mut entradas_izq: HashMap<String, usize> = HashMap::new();
    let mut entradas_der: HashMap<String, usize> = HashMap::new();
    let mut canal = 0usize;
    let mut canal_hueco = 0usize;
    let mut canal_heap = 0usize;
    let borde_canal = borde_heap + 14.0;
    let mut max_x = 0.0_f64;
    let mut max_y = 0.0_f64;
    let terminados: HashSet<&str> = v.objetos.iter().filter(|o| o.terminado).map(|o| o.id.as_str()).collect();
    for f in &flechas {
        let d = &destinos[&f.destino];
        let colgante = terminados.contains(d.base.as_str()) && !op.sin_estados;
        let mut puntos = vec![f.desde];
        if let (Some(bajo), true) = (f.bajo, f.del_stack) {
            // De una celda de un objeto automático: baja y sale hacia la derecha del marco.
            puntos.push((f.desde.0, bajo));
            puntos.push((ancho_stack + 6.0, bajo));
        }
        let a_celda = matches!(d.parte, Parte::Celda(_));
        let fila_destino = |k: &mut usize| -> f64 {
            let ty = match d.parte {
                Parte::Campo(_) => d.y + d.h / 2.0,
                _ => (d.y + 11.0 + *k as f64 * 7.0).min(d.y + d.h - 6.0),
            };
            *k += 1;
            ty
        };
        if d.en_stack {
            let gx = ancho_stack + 16.0 + canal_hueco as f64 * 7.0;
            canal_hueco += 1;
            if !f.del_stack {
                puntos = vec![f.desde, (d.x + d.w + 3.0, d.y + d.h / 2.0)];
            } else if a_celda {
                entrar_por_abajo(&mut puntos, gx, d, &mut carriles_bajo);
            } else {
                let k = entradas_der.entry(f.destino.clone()).or_insert(0);
                let ty = fila_destino(k);
                puntos.push((gx, puntos.last().unwrap().1));
                puntos.push((gx, ty));
                puntos.push((d.x + d.w + 3.0, ty));
            }
        } else if f.del_stack {
            if a_celda {
                let gx = x_heap - 16.0 - canal_heap as f64 * 6.0;
                canal_heap += 1;
                entrar_por_abajo(&mut puntos, gx, d, &mut carriles_bajo);
            } else {
                let k = entradas_izq.entry(f.destino.clone()).or_insert(0);
                let ty = fila_destino(k);
                puntos.push((d.x - 3.0, ty));
            }
        } else {
            let cx = borde_canal + canal as f64 * PASO_CANAL;
            canal += 1;
            if let Some(bajo) = f.bajo {
                puntos.push((f.desde.0, bajo));
                puntos.push((cx, bajo));
            } else {
                puntos.push((cx, f.desde.1));
            }
            if a_celda {
                entrar_por_abajo(&mut puntos, cx, d, &mut carriles_bajo);
            } else {
                let k = entradas_der.entry(f.destino.clone()).or_insert(0);
                let ty = fila_destino(k);
                puntos.push((cx, ty));
                puntos.push((d.x + d.w + 3.0, ty));
            }
        }
        for p in &puntos {
            max_x = max_x.max(p.0);
            max_y = max_y.max(p.1);
        }
        trazar(c, &puntos, colgante);
    }

    // El límite excluido de un arreglo al que apunta algo: una marca en el borde, sin celda.
    for f in &flechas {
        let d = &destinos[&f.destino];
        if let Parte::Celda(k) = d.parte {
            if d.w == 0.0 {
                c.linea(d.x, d.y - 12.0, d.x, d.y + d.h + 4.0, "strokeMain", 1.6, None, "4 2");
                c.t(d.x + 3.0, d.y - 5.0, &format!("[{k}]"), 9.0, 700, "textMuted", "start", MONO);
                max_x = max_x.max(d.x + 3.0 + ancho_texto(&format!("[{k}]"), 9.0));
            }
        }
    }

    let mut ancho = (borde_canal + canal as f64 * PASO_CANAL)
        .max(borde_heap + 8.0)
        .max(ancho_marcas)
        .max(max_x + 6.0);
    let bajos = v
        .objetos
        .iter()
        .filter(|o| !o.en_stack())
        .map(|o| carriles_bajo.get(&o.id).copied().unwrap_or(0) as f64 * 4.0)
        .fold(0.0, f64::max)
        + 4.0;
    let mut alto = fin_stack.max(fin_heap + bajos).max(max_y + 4.0) + 8.0;
    if let Some((z_stack, z_heap)) = &op.zonas {
        if d.ancho_stack > 0.0 {
            zona(c, -6.0, ancho_stack + 10.0, top + 2.0, alto + 4.0, z_stack);
        }
        let hay_estaticos = v.objetos.iter().any(|o| o.almacen == Almacen::Estatico);
        let titulo = if hay_estaticos && z_heap == "Heap" { "Heap y estático".to_string() } else { z_heap.clone() };
        zona(c, x_heap - 16.0, ancho + 6.0, top + 2.0, alto + 4.0, &titulo);
        ancho += 6.0;
        alto += 8.0;
    }
    (ancho, alto)
}

fn render_object_heap(c: &mut Lienzo, b: &Bloque) -> Result<Punto, ErrorDiagrama> {
    render(c, b).map_err(|e| {
        if e.0.starts_with("object_heap: ") {
            e
        } else {
            ErrorDiagrama::new(format!("object_heap: {}", e.0))
        }
    })
}

fn render(c: &mut Lienzo, b: &Bloque) -> Result<Punto, ErrorDiagrama> {
    let lenguaje = match b.get("variant") {
        None | Some(Value::Null) => Lenguaje::Java,
        Some(Value::String(v)) if v == "java" => Lenguaje::Java,
        Some(Value::String(v)) if v == "python" => Lenguaje::Python,
        Some(Value::String(v)) if v == "c" => Lenguaje::C,
        Some(otro) => return Err(error(format!("variante no soportada: {} (java, python o c)", texto(otro)))),
    };
    LENGUAJE.with(|p| p.set(lenguaje));
    let resultado = render_con_estilo(c, b);
    LENGUAJE.with(|p| p.set(Lenguaje::Java));
    resultado
}

fn render_con_estilo(c: &mut Lienzo, b: &Bloque) -> Result<Punto, ErrorDiagrama> {
    let op = Opciones {
        sin_valores: b.get("blankValues").is_some_and(es_truthy),
        sin_refs: b.get("blankRefs").is_some_and(es_truthy),
        sin_estados: lenguaje() == Lenguaje::C && b.get("blankStates").is_some_and(es_truthy),
        marcar: b.get("markUnreachable").map(es_truthy).unwrap_or(true),
        zonas: if b.contains_key("zones") {
            nombres_de_zonas(b)
        } else {
            Some(("Stack".to_string(), "Heap".to_string()))
        },
    };
    let vistas: Vec<Vista> = if let Some(inst) = b.get("snapshots").filter(|v| !v.is_null()) {
        if b.contains_key("frames") || b.contains_key("objects") {
            return Err(error("usar 'snapshots' o 'frames' y 'objects', no los dos"));
        }
        let Some(lista) = inst.as_array().filter(|l| !l.is_empty()) else {
            return Err(error("'snapshots' tiene que ser una lista de instantes, cada uno con 'frames' y 'objects'"));
        };
        if lista.len() > MAX_VISTAS {
            return Err(error(format!("hay {} instantes; el máximo es {MAX_VISTAS}", lista.len())));
        }
        lista
            .iter()
            .enumerate()
            .map(|(i, s)| {
                let Some(obj) = s.as_object() else {
                    return Err(error(format!("el instante {i} tiene que ser un objeto con 'frames' y 'objects'")));
                };
                leer_vista(obj, &format!("instante {}", i + 1))
            })
            .collect::<Result<_, _>>()?
    } else {
        vec![leer_vista(b, TIPO)?]
    };

    let top = encabezado(c, b, "");
    let d = disponer(&vistas);
    if vistas.len() == 1 {
        return Ok(dibujar_vista(c, &vistas[0], top, &op, &d));
    }
    let en_fila = b.get("direction").map(texto).as_deref() == Some("row");
    let (mut x, mut y) = (0.0_f64, top);
    let (mut ancho, mut alto) = (0.0_f64, top);
    for (i, v) in vistas.iter().enumerate() {
        let mut sub = sub_lienzo(c);
        let rotulo = v.rotulo.clone().unwrap_or_else(|| format!("Instante {}", i + 1));
        sub.t(0.0, 14.0, &rotulo, 11.5, 700, "textMain", "start", SANS);
        let (w, h) = dibujar_vista(&mut sub, v, 20.0, &op, &d);
        agregar_sub_lienzo(c, x, y, sub);
        ancho = ancho.max(x + w);
        alto = alto.max(y + h);
        if en_fila {
            x += w + 36.0;
        } else {
            y += h + 26.0;
        }
    }
    Ok((ancho, alto))
}

#[cfg(test)]
mod tests {
    use super::*;
    use dibu_core::Registro;
    use serde_json::json;

    fn svg(bloque: Value) -> Result<String, ErrorDiagrama> {
        let mut reg = Registro::new();
        registrar(&mut reg);
        reg.renderizar(bloque.as_object().unwrap(), true)
    }

    fn error_de(bloque: Value) -> String {
        svg(bloque).unwrap_err().0
    }

    /// Las flechas (líneas y caminos con punta) del SVG.
    fn flechas(svg: &str) -> usize {
        svg.matches("marker-end=\"url(#flecha-main").count()
    }

    fn compartido() -> Value {
        json!({
            "type": "object_heap",
            "frames": [{"function": "main", "vars": [
                {"name": "a", "type": "int[]", "ref": "A"},
                {"name": "b", "type": "int[]", "ref": "A"},
                {"name": "copia", "type": "int[]", "ref": "B"},
                {"name": "cantidad", "type": "int", "value": 3}
            ]}],
            "objects": [
                {"id": "A", "type": "int[]", "elements": [{"value": 1}, {"value": 2}, {"value": 3}]},
                {"id": "B", "type": "int[]", "elements": [1, 2, 3]}
            ]
        })
    }

    #[test]
    fn alias_y_objetos_con_el_mismo_contenido() {
        let s = svg(compartido()).unwrap();
        assert_eq!(flechas(&s), 3, "a, b y copia");
        assert!(s.contains("A : int[]") && s.contains("B : int[]"));
        assert!(s.contains(">int cantidad<") && s.contains(">3<"));
        assert!(!s.contains("inalcanzable"));
    }

    #[test]
    fn las_dos_referencias_a_un_objeto_llegan_a_la_misma_caja() {
        let v = leer_vista(compartido().as_object().unwrap(), "t").unwrap();
        let destinos: Vec<_> = v.marcos[0]
            .ranuras
            .iter()
            .filter_map(|r| match &r.valor {
                Valor::Ref(Some(d)) => Some(d.as_str()),
                _ => None,
            })
            .collect();
        assert_eq!(destinos, ["A", "A", "B"]);
    }

    #[test]
    fn referencia_nula_y_entre_objetos() {
        let s = svg(json!({
            "type": "object_heap",
            "frames": [{"function": "main", "vars": [
                {"name": "p", "type": "Nodo", "ref": "N1"},
                {"name": "q", "type": "Nodo", "ref": null}
            ]}],
            "objects": [
                {"id": "N1", "type": "Nodo", "fields": [{"name": "dato", "value": 1}, {"name": "sig", "ref": "N2"}]},
                {"id": "N2", "type": "Nodo", "fields": [{"name": "dato", "value": 2}, {"name": "sig", "ref": "N2"}]}
            ]
        }))
        .unwrap();
        assert!(s.contains(">null<"));
        // p → N1, N1.sig → N2, y N2.sig → N2 (a sí mismo).
        assert_eq!(flechas(&s), 3);
        let dibujo = s.split("</defs>").nth(1).unwrap();
        assert_eq!(dibujo.matches("<path d=\"M ").count(), 2, "las referencias entre objetos van por el canal");
    }

    #[test]
    fn alcanzabilidad() {
        let s = svg(json!({
            "type": "object_heap",
            "frames": [{"function": "main", "vars": [{"name": "p", "type": "Persona", "ref": null}]}],
            "objects": [{"id": "P1", "type": "Persona", "fields": [{"name": "nombre", "value": "Ana"}]}]
        }))
        .unwrap();
        assert!(s.contains("inalcanzable"));
        assert!(s.contains("&quot;Ana&quot;"), "los textos van entre comillas");
        let sin = svg(json!({
            "type": "object_heap", "markUnreachable": false,
            "frames": [{"function": "main", "vars": [{"name": "p", "ref": null}]}],
            "objects": [{"id": "P1", "fields": []}]
        }))
        .unwrap();
        assert!(!sin.contains("inalcanzable"));
    }

    #[test]
    fn el_string_pool_agrupa_sus_objetos() {
        let s = svg(json!({
            "type": "object_heap",
            "frames": [{"function": "main", "vars": [
                {"name": "s1", "type": "String", "ref": "H"},
                {"name": "s2", "type": "String", "ref": "H"},
                {"name": "s3", "type": "String", "ref": "N"}
            ]}],
            "objects": [
                {"id": "N", "type": "String", "value": "Hola"},
                {"id": "H", "type": "String", "value": "Hola", "group": "String Pool"}
            ]
        }))
        .unwrap();
        assert!(s.contains(">String Pool<"));
        assert_eq!(s.matches("&quot;Hola&quot;").count(), 2);
    }

    #[test]
    fn instantes_conservan_la_identidad() {
        let s = svg(json!({
            "type": "object_heap",
            "snapshots": [
                {"label": "Al llamar", "frames": [
                    {"function": "main", "vars": [{"name": "numeros", "type": "int[]", "ref": "A"}]},
                    {"function": "modificar", "params": [{"name": "arr", "type": "int[]", "ref": "A"}]}
                ], "objects": [{"id": "A", "type": "int[]", "elements": [1, 2]}]},
                {"label": "Después de arr = new int[] {...}", "frames": [
                    {"function": "main", "vars": [{"name": "numeros", "type": "int[]", "ref": "A"}]},
                    {"function": "modificar", "params": [{"name": "arr", "type": "int[]", "ref": "B"}]}
                ], "objects": [
                    {"id": "A", "type": "int[]", "elements": [1, 2]},
                    {"id": "B", "type": "int[]", "elements": [7, 8]}
                ]}
            ]
        }))
        .unwrap();
        assert!(s.contains(">Al llamar<"));
        assert_eq!(flechas(&s), 4);
        assert_eq!(s.matches("A : int[]").count(), 2, "el mismo objeto en los dos instantes");
    }

    #[test]
    fn modo_ejercicio() {
        let mut b = compartido();
        b["blankRefs"] = json!(true);
        let s = svg(b).unwrap();
        assert_eq!(flechas(&s), 0, "sin flechas: hay que dibujarlas");
        let mut b = compartido();
        b["blankValues"] = json!(true);
        assert!(!svg(b).unwrap().contains(">3<"));
    }

    #[test]
    fn errores() {
        let casos = [
            (json!({"type": "object_heap"}), "no hay 'frames' ni 'objects'"),
            (
                json!({"type": "object_heap", "objects": [{"id": "A"}, {"id": "A"}]}),
                "el ID 'A' está repetido",
            ),
            (
                json!({"type": "object_heap", "frames": [{"function": "main", "vars": [{"name": "x", "ref": "Z"}]}]}),
                "refiere a 'Z'",
            ),
            (
                json!({"type": "object_heap", "frames": [{"function": "main", "vars": [{"name": "x", "value": 1, "ref": "A"}]}], "objects": [{"id": "A"}]}),
                "tiene 'value' y 'ref'",
            ),
            (
                json!({"type": "object_heap", "objects": [{"id": "A", "fields": [], "elements": []}]}),
                "sólo uno de",
            ),
            (json!({"type": "object_heap", "objects": [{"type": "X"}]}), "no tiene 'id'"),
            (
                json!({"type": "object_heap", "snapshots": [], "frames": []}),
                "no los dos",
            ),
        ];
        for (bloque, esperado) in casos {
            let e = error_de(bloque.clone());
            assert!(e.starts_with("object_heap: "), "{e}");
            assert!(e.contains(esperado), "{bloque}: {e}");
        }
    }

    #[test]
    fn determinismo_y_paletas_de_los_diagramas_para_java() {
        let mut reg = Registro::new();
        crate::registrar(&mut reg);
        let casos = [
            compartido(),
            json!({"type": "matrix", "variant": "java", "values": [[1, 2], null, []]}),
            json!({"type": "flowchart", "steps": [{"doWhile": "i < 3", "do": ["i = i + 1"]}]}),
        ];
        for caso in casos {
            let b = caso.as_object().unwrap();
            let tinta = reg.renderizar(b, true).unwrap();
            assert_eq!(tinta, reg.renderizar(b, true).unwrap(), "no determinista: {caso}");
            assert_ne!(tinta, reg.renderizar(b, false).unwrap(), "tinta y color iguales: {caso}");
            assert!(tinta.starts_with("<svg ") && tinta.ends_with("</svg>") && !tinta.contains("NaN"));
        }
    }

    #[test]
    fn python_copia_superficial_converge_en_la_sublista() {
        let s = svg(json!({
            "type": "object_heap", "variant": "python",
            "frames": [{"function": "<módulo>", "vars": [{"name": "original", "ref": "A"}, {"name": "copia", "ref": "B"}, {"name": "x", "value": null}]}],
            "objects": [
                {"id": "A", "type": "list", "elements": [{"ref": "I"}]},
                {"id": "B", "type": "list", "elements": [{"ref": "I"}]},
                {"id": "I", "type": "list", "elements": [{"value": 4}, "siete"]}
            ]
        }))
        .unwrap();
        assert!(s.contains("><módulo><") || s.contains(">&lt;módulo&gt;<"), "el ámbito queda sin paréntesis");
        assert!(s.contains(">None<"), "None como valor rotulado");
        assert!(s.contains(">&#x27;siete&#x27;<"), "los textos como en Python");
        assert_eq!(flechas(&s), 4);
        // Las dos referencias a I entran por su borde derecho, en el mismo x.
        let dibujo = s.split("</defs>").nth(1).unwrap();
        let finales: Vec<&str> = dibujo
            .split("<path d=\"M ")
            .skip(1)
            .map(|p| p[..p.find('"').unwrap()].rsplit(" L ").next().unwrap().split(' ').next().unwrap())
            .collect();
        assert_eq!(finales.len(), 2);
        assert_eq!(finales[0], finales[1]);
    }

    #[test]
    fn python_parametro_homonimo_e_identidad_estable() {
        let b = json!({
            "type": "object_heap", "variant": "python",
            "snapshots": [
                {"label": "antes", "frames": [
                    {"function": "<módulo>", "vars": [{"name": "lista", "ref": "A"}]},
                    {"function": "agregar", "params": [{"name": "lista", "ref": "A"}]}
                ], "objects": [{"id": "A", "type": "list", "elements": [1, 2, 9]}]},
                {"label": "después", "frames": [
                    {"function": "<módulo>", "vars": [{"name": "lista", "ref": "A"}]},
                    {"function": "agregar", "params": [{"name": "lista", "ref": "B"}]}
                ], "objects": [
                    {"id": "B", "type": "list", "elements": [0]},
                    {"id": "A", "type": "list", "elements": [1, 2, 9]}
                ]}
            ]
        });
        let vistas: Vec<Vista> = b["snapshots"]
            .as_array()
            .unwrap()
            .iter()
            .map(|v| leer_vista(v.as_object().unwrap(), "t").unwrap())
            .collect();
        let destino = |v: &Vista, marco: usize| match &v.marcos[marco].ranuras[0].valor {
            Valor::Ref(Some(d)) => d.clone(),
            otro => panic!("{otro:?}"),
        };
        assert_eq!((destino(&vistas[1], 0), destino(&vistas[1], 1)), ("A".to_string(), "B".to_string()));
        // A queda en el mismo lugar en los dos instantes, aunque B se declare antes.
        let s = svg(b).unwrap();
        let ys: Vec<&str> = s
            .match_indices(">A : list<")
            .map(|(i, _)| {
                let antes = &s[..i];
                let y = antes.rfind(" y=\"").unwrap() + 4;
                &s[y..y + antes[y..].find('"').unwrap()]
            })
            .collect();
        assert_eq!(ys.len(), 2);
        assert_eq!(ys[0], ys[1]);
        assert!(error_de(json!({"type": "object_heap", "variant": "cobol", "objects": [{"id": "A"}]})).contains("java, python o c"));
    }

    fn colgantes(svg: &str) -> usize {
        svg.matches("data-colgante=\"1\"").count()
    }

    #[test]
    fn c_puntero_a_puntero_llega_a_la_variable_del_llamador() {
        let s = svg(json!({
            "type": "object_heap", "variant": "c",
            "frames": [
                {"function": "main", "vars": [{"name": "q", "type": "int *", "id": "q", "ref": "B1"}]},
                {"function": "reservar", "params": [{"name": "pp", "type": "int **", "ref": "q"}, {"name": "i", "type": "int", "indeterminate": true}]}
            ],
            "objects": [{"id": "B1", "type": "int[2]", "address": "0x5000", "elements": [0, 0]}]
        }))
        .unwrap();
        assert_eq!(flechas(&s), 2, "pp → q y q → B1");
        assert!(s.contains(">int **pp<") && s.contains(">int *q<"), "declaraciones de C");
        assert!(s.contains(">indeterminado<") && s.contains(">0x5000<"));
        // pp entra a la caja de q por la derecha, desde el canal entre el stack y el heap.
        assert_eq!(s.split("</defs>").nth(1).unwrap().matches("<path d=\"M ").count(), 1);
    }

    #[test]
    fn c_copia_superficial_converge_y_los_locales_van_en_su_marco() {
        let campos = json!([{"name": "datos", "type": "int *", "ref": "D"}, {"name": "largo", "type": "size_t", "value": 2}]);
        let s = svg(json!({
            "type": "object_heap", "variant": "c",
            "frames": [{"function": "main", "vars": [{"name": "fin", "type": "int *", "ref": "D[2]"}]}],
            "objects": [
                {"id": "a", "type": "struct vec", "storage": "auto", "frame": "main", "fields": campos},
                {"id": "b", "type": "struct vec", "storage": "auto", "frame": 0, "fields": campos},
                {"id": "D", "type": "int[2]", "elements": [7, 9]},
                {"id": "cuenta", "type": "int", "storage": "static", "value": 3}
            ]
        }))
        .unwrap();
        assert_eq!(flechas(&s), 3);
        assert!(s.contains(">struct vec a<") && s.contains(">int cuenta<"));
        assert!(s.contains(">estático<") && s.contains(">Heap y estático<"));
        assert!(s.contains("stroke-dasharray=\"4 2\""), "el límite excluido se marca sin una celda");
        assert!(!s.contains(">fuga<"), "D es alcanzable desde los locales");
    }

    #[test]
    fn c_free_con_dos_alias_y_reutilizacion_de_la_direccion() {
        let bloque = json!({
            "type": "object_heap", "variant": "c",
            "frames": [{"function": "main", "vars": [{"name": "p", "type": "int *", "ref": "B"}, {"name": "q", "type": "int *", "ref": "B[1]"}, {"name": "r", "type": "int *", "ref": "N"}]}],
            "objects": [
                {"id": "B", "type": "int[2]", "address": "0x10", "ended": true, "elements": [41, 42]},
                {"id": "N", "type": "int[2]", "address": "0x10", "elements": [5, 6]}
            ]
        });
        let s = svg(bloque.clone()).unwrap();
        assert_eq!(colgantes(&s), 2, "p y q quedan colgantes");
        assert_eq!(flechas(&s), 1, "sólo r es una referencia válida");
        assert!(s.contains(">vida terminada<") && s.matches(">colgante<").count() == 2);
        assert!(!s.contains(">41<") && !s.contains(">42<"), "la memoria liberada no muestra valores");
        assert!(s.contains(">B : int[2]<") && s.contains(">N : int[2]<"), "dos objetos aunque compartan la dirección");
        // Para completar: sin rótulos ni guiones que delaten el estado.
        let mut b = bloque;
        b["blankStates"] = json!(true);
        let s = svg(b).unwrap();
        assert_eq!(colgantes(&s), 0);
        assert!(!s.contains(">vida terminada<") && !s.contains(">colgante<"));
        assert_eq!(s.matches(">estado:<").count(), 2);
    }

    #[test]
    fn c_local_de_un_marco_terminado_y_fuga() {
        let s = svg(json!({
            "type": "object_heap", "variant": "c",
            "frames": [{"function": "main", "vars": [{"name": "r", "type": "int *", "ref": "x"}, {"name": "p", "type": "int *", "ref": null}]}],
            "objects": [
                {"id": "x", "type": "int", "storage": "auto", "frame": "crear", "ended": true, "value": 5},
                {"id": "H", "type": "int[1]", "elements": [1]}
            ]
        }))
        .unwrap();
        assert!(s.contains(">crear() (terminó)<"));
        assert_eq!(colgantes(&s), 1);
        assert!(s.contains(">fuga<") && s.contains(">NULL<"));
        assert!(!s.contains(">5<"));
    }

    #[test]
    fn c_errores() {
        let base = |objetos: Value, vars: Value| json!({"type": "object_heap", "variant": "c", "frames": [{"function": "main", "vars": vars}], "objects": objetos});
        let a = json!([{"id": "A", "type": "int[2]", "elements": [1, 2]}]);
        let casos = [
            (base(a.clone(), json!([{"name": "p", "ref": "A[3]"}])), "fuera de 'A'"),
            (base(a.clone(), json!([{"name": "p", "ref": "A.x"}])), "no tiene campos"),
            (base(a.clone(), json!([{"name": "p", "ref": "Z"}])), "no es un objeto ni una variable"),
            (base(a.clone(), json!([{"name": "p", "ref": "A[x"}])), "ID.campo o ID[i]"),
            (base(a.clone(), json!([{"name": "p", "indeterminate": true, "value": 3}])), "no lleva 'value'"),
            (base(json!([{"id": "x", "type": "int", "storage": "auto", "frame": "f", "value": 1}]), json!([])), "si ya terminó"),
            (base(json!([{"id": "x", "storage": "pila"}]), json!([])), "'storage'"),
            (
                base(json!([{"id": "A", "address": "0x1"}, {"id": "B", "address": "0x1"}]), json!([])),
                "vivos en la misma dirección",
            ),
            (base(a.clone(), json!([{"name": "p", "id": "A"}])), "repetido"),
        ];
        for (bloque, esperado) in casos {
            let e = error_de(bloque.clone());
            assert!(e.contains(esperado), "{bloque}: {e}");
        }
        // Java no acepta destinos de C.
        let java = json!({"type": "object_heap", "frames": [{"function": "main", "vars": [{"name": "p", "ref": "A[0]"}]}], "objects": [{"id": "A", "elements": [1]}]});
        assert!(error_de(java).contains("refiere a 'A[0]'"));
    }
}
