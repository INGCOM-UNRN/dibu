//! Modelo de memoria de un lenguaje con referencias (`object_heap`): los marcos activos en el
//! stack, los objetos en el heap y las referencias entre ellos, como se enseña en Java.
//!
//! Cada objeto tiene un ID; las variables, los campos y las celdas guardan un valor (`value`) o
//! una referencia (`ref`, que puede ser `null`). dibu calcula qué objetos son alcanzables desde el
//! stack y marca los demás. Con `snapshots`, varios instantes apilados (o lado a lado).

use crate::common::{
    agregar_sub_lienzo, ancho_texto, encabezado, es_truthy, nombres_de_zonas, sub_lienzo, texto,
    zona, Bloque, ErrorDiagrama, Lienzo, Punto, ALTO_ZONA, MONO, SANS,
};
use dibu_core::Registro;
use serde_json::Value;
use std::collections::{BTreeMap, HashMap, HashSet, VecDeque};

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

pub fn registrar(reg: &mut Registro) {
    reg.registrar("object_heap", render_object_heap);
}

// ---------------------------------------------------------------------------
// Modelo
// ---------------------------------------------------------------------------

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
}

impl Ranura {
    fn rotulo(&self) -> String {
        match &self.tipo {
            Some(t) if !t.is_empty() => format!("{t} {}", self.nombre),
            _ => self.nombre.clone(),
        }
    }
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

#[derive(Debug)]
struct Objeto {
    id: String,
    tipo: String,
    contenido: Contenido,
    grupo: Option<String>,
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
        } else {
            format!("{} : {}", self.id, self.tipo)
        }
    }
}

#[derive(Debug)]
struct Vista {
    rotulo: Option<String>,
    marcos: Vec<Marco>,
    objetos: Vec<Objeto>,
}

// ---------------------------------------------------------------------------
// Lectura
// ---------------------------------------------------------------------------

fn error(msg: impl Into<String>) -> ErrorDiagrama {
    ErrorDiagrama::new(msg)
}

/// Una variable, un campo o una celda: `{name, type, value | ref, blank}`; en las celdas, también
/// un valor suelto (`3`, `"x"`).
fn leer_ranura(dato: &Value, nombre_por_omision: &str, donde: &str) -> Result<Ranura, ErrorDiagrama> {
    let Some(obj) = dato.as_object() else {
        return Ok(Ranura {
            nombre: nombre_por_omision.to_string(),
            tipo: None,
            valor: Valor::Dato(if dato.is_null() { dato_nulo() } else { mostrar_celda(dato) }),
            en_blanco: false,
        });
    };
    let nombre = obj.get("name").map(texto).unwrap_or_else(|| nombre_por_omision.to_string());
    if nombre.is_empty() {
        return Err(error(format!("{donde}: falta 'name'")));
    }
    let tipo = obj.get("type").filter(|v| !v.is_null()).map(texto);
    let valor = match (obj.get("value"), obj.contains_key("ref")) {
        (Some(_), true) => {
            return Err(error(format!(
                "{donde}: '{nombre}' tiene 'value' y 'ref' (un valor o una referencia, no los dos)"
            )));
        }
        (_, true) => match obj.get("ref") {
            Some(Value::Null) | None => Valor::Ref(None),
            Some(r) => Valor::Ref(Some(texto(r))),
        },
        (Some(v), false) => Valor::Dato(if v.is_null() { dato_nulo() } else { mostrar_dato(v) }),
        (None, false) => Valor::Dato(String::new()),
    };
    Ok(Ranura {
        nombre,
        tipo,
        valor,
        en_blanco: obj.get("blank").is_some_and(es_truthy),
    })
}

thread_local! {
    /// `variant: python`: los valores se escriben como en Python (`'texto'`, `True`, `None`).
    static PYTHON: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
}

fn es_python() -> bool {
    PYTHON.with(|p| p.get())
}

/// Un texto se muestra entre comillas, como en el código; los números y booleanos, tal cual.
fn mostrar_dato(v: &Value) -> String {
    if es_python() {
        return match v {
            Value::Null => "None".to_string(),
            Value::Bool(true) => "True".to_string(),
            Value::Bool(false) => "False".to_string(),
            Value::String(s) if s.contains('\'') && !s.contains('"') => format!("\"{s}\""),
            Value::String(s) => format!("'{}'", s.replace('\'', "\\'")),
            otro => texto(otro),
        };
    }
    match v {
        Value::String(s) => format!("\"{s}\""),
        otro => texto(otro),
    }
}

/// Un valor suelto de una celda (`elements: [1, "a"]`): tal cual en Java, como `repr` en Python.
fn mostrar_celda(v: &Value) -> String {
    if es_python() { mostrar_dato(v) } else { texto(v) }
}

/// Una referencia nula: `null` en Java, `None` en Python.
fn nulo() -> &'static str {
    if es_python() { "None" } else { "null" }
}

/// Un valor `null` declarado como dato: vacío en Java, `None` en Python.
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

fn leer_objetos(dato: Option<&Value>, donde: &str) -> Result<Vec<Objeto>, ErrorDiagrama> {
    let Some(v) = dato.filter(|v| !v.is_null()) else {
        return Ok(Vec::new());
    };
    let Some(lista) = v.as_array() else {
        return Err(error(format!("{donde}: 'objects' tiene que ser una lista de objetos con 'id'")));
    };
    if lista.len() > MAX_OBJETOS {
        return Err(error(format!("{donde}: hay {} objetos; el máximo es {MAX_OBJETOS}", lista.len())));
    }
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
        objetos.push(Objeto {
            id,
            tipo: obj.get("type").filter(|v| !v.is_null()).map(texto).unwrap_or_default(),
            contenido,
            grupo: obj.get("group").filter(|v| es_truthy(v)).map(texto),
        });
    }
    Ok(objetos)
}

fn leer_vista(obj: &Bloque, donde: &str) -> Result<Vista, ErrorDiagrama> {
    let vista = Vista {
        rotulo: obj.get("label").filter(|v| es_truthy(v)).map(texto),
        marcos: leer_marcos(obj.get("frames"), donde)?,
        objetos: leer_objetos(obj.get("objects"), donde)?,
    };
    if vista.marcos.is_empty() && vista.objetos.is_empty() {
        return Err(error(format!("{donde}: no hay 'frames' ni 'objects'")));
    }
    validar(&vista, donde)?;
    Ok(vista)
}

/// IDs únicos y referencias a objetos que existen.
fn validar(v: &Vista, donde: &str) -> Result<(), ErrorDiagrama> {
    let mut ids = HashSet::new();
    for o in &v.objetos {
        if !ids.insert(o.id.as_str()) {
            return Err(error(format!("{donde}: el ID '{}' está repetido", o.id)));
        }
    }
    let ranuras = v
        .marcos
        .iter()
        .flat_map(|m| m.ranuras.iter().map(move |r| (format!("{}.{}", m.funcion, r.nombre), r)))
        .chain(v.objetos.iter().flat_map(|o| o.ranuras().iter().map(move |r| (format!("{}.{}", o.id, r.nombre), r))));
    for (lugar, r) in ranuras {
        if let Valor::Ref(Some(destino)) = &r.valor {
            if !ids.contains(destino.as_str()) {
                return Err(error(format!("{donde}: {lugar} refiere a '{destino}', que no está en 'objects'")));
            }
        }
    }
    Ok(())
}

/// Los objetos a los que se llega desde el stack, siguiendo referencias.
fn alcanzables(v: &Vista) -> HashSet<String> {
    let por_id: HashMap<&str, &Objeto> = v.objetos.iter().map(|o| (o.id.as_str(), o)).collect();
    let mut vistos = HashSet::new();
    let mut cola: VecDeque<&str> = VecDeque::new();
    for r in v.marcos.iter().flat_map(|m| &m.ranuras) {
        if let Valor::Ref(Some(d)) = &r.valor {
            cola.push_back(d);
        }
    }
    while let Some(id) = cola.pop_front() {
        if !vistos.insert(id.to_string()) {
            continue;
        }
        if let Some(o) = por_id.get(id) {
            for r in o.ranuras() {
                if let Valor::Ref(Some(d)) = &r.valor {
                    cola.push_back(d);
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
    marcar: bool,
    zonas: Option<(String, String)>,
}

fn ancho_valor(r: &Ranura) -> f64 {
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

struct Caja {
    x: f64,
    y: f64,
    w: f64,
    h: f64,
}

/// Una referencia a dibujar: desde dónde, a qué objeto y si sale del stack o de otro objeto.
struct Flecha {
    desde: Punto,
    destino: String,
    del_stack: bool,
    /// En una celda de un arreglo, la flecha baja antes de salir hacia la derecha.
    bajo: Option<f64>,
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
    let cabecera = ancho_texto(&o.rotulo(), 10.5) + 20.0;
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

fn disponer(vistas: &[Vista]) -> Disposicion {
    let mut unicos: Vec<&Objeto> = Vec::new();
    let mut medida: HashMap<&str, (f64, f64)> = HashMap::new();
    for o in vistas.iter().flat_map(|v| &v.objetos) {
        let (w, h) = medir_objeto(o);
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
        ancho_stack: vistas.iter().flat_map(|v| &v.marcos).map(|m| medir_marco(m).0).fold(0.0, f64::max),
        ancho_objetos: medida.values().map(|m| m.0).fold(0.0, f64::max),
    }
}

fn dibujar_vista(c: &mut Lienzo, v: &Vista, top: f64, op: &Opciones, d: &Disposicion) -> Punto {
    let alto_zona = if op.zonas.is_some() { ALTO_ZONA } else { 0.0 };
    let y_inicio = top + alto_zona + 8.0;
    let mut flechas: Vec<Flecha> = Vec::new();

    // Stack: del tope (arriba) a main (abajo), como en call_stack.
    let medidas: Vec<(f64, f64)> = v.marcos.iter().map(medir_marco).collect();
    let ancho_stack = d.ancho_stack;
    let mut y = y_inicio;
    for (i, m) in v.marcos.iter().enumerate().rev() {
        let (_, ancho_valores) = medidas[i];
        let alto = ALTO_CABECERA + m.ranuras.len().max(1) as f64 * ALTO_FILA + 4.0;
        let tope = i + 1 == v.marcos.len();
        c.r(0.0, y, ancho_stack, alto, "fillCell", if tope { 2.2 } else { 1.5 }, 4.0);
        c.r(0.0, y, ancho_stack, ALTO_CABECERA, "fillHeader", if tope { 2.2 } else { 1.5 }, 4.0);
        c.t(8.0, y + 15.5, &titulo_marco(&m.funcion), 11.0, 700, "textMain", "start", MONO);
        for (k, r) in m.ranuras.iter().enumerate() {
            let fy = y + ALTO_CABECERA + k as f64 * ALTO_FILA + (ALTO_FILA - ALTO_CAJA) / 2.0;
            c.t(8.0, fy + ALTO_CAJA / 2.0 + 4.0, &r.rotulo(), 10.5, 400, "textMain", "start", MONO);
            let vx = ancho_stack - ancho_valores - 8.0;
            c.r(vx, fy, ancho_valores, ALTO_CAJA, "fillPointerBox", 1.4, 2.0);
            if let Some((desde, destino)) = dibujar_valor(c, r, vx, fy, ancho_valores, ALTO_CAJA, op) {
                flechas.push(Flecha { desde, destino, del_stack: true, bajo: None });
            }
        }
        y += alto + SEP_MARCOS;
    }
    let fin_stack = if v.marcos.is_empty() { y_inicio } else { y - SEP_MARCOS };

    // Heap: los objetos en una columna; los de un grupo, dentro de un recuadro.
    let hueco = if op.zonas.is_some() { 110.0 } else { 90.0 };
    let x_heap = if d.ancho_stack == 0.0 { 14.0 } else { ancho_stack + hueco };
    let mut cajas: BTreeMap<String, Caja> = BTreeMap::new();
    let ancho_objetos = d.ancho_objetos;
    let mut fin_heap = y_inicio;
    for o in &v.objetos {
        let x = x_heap + if o.grupo.is_some() { 10.0 } else { 0.0 };
        let (w, h) = medir_objeto(o);
        let y = y_inicio + d.y[&o.id];
        fin_heap = fin_heap.max(y + h);
        cajas.insert(o.id.clone(), Caja { x, y, w, h });
    }
    let hay_grupos = !d.grupos.is_empty();
    let borde_heap = x_heap + ancho_objetos + if hay_grupos { 20.0 } else { 0.0 };
    for (g, y0, y1, ids) in &d.grupos {
        if v.objetos.iter().any(|o| ids.contains(&o.id)) {
            zona(c, x_heap - 2.0, borde_heap, y_inicio + y0, y_inicio + y1, g);
            fin_heap = fin_heap.max(y_inicio + y1);
        }
    }

    let vivos = alcanzables(v);
    let mut salidas_bajo: HashMap<String, usize> = HashMap::new();
    for o in &v.objetos {
        let caja = &cajas[&o.id];
        let (x, y, w) = (caja.x, caja.y, caja.w);
        c.r(x, y, w, caja.h, "fillCell", 1.6, 3.0);
        c.r(x, y, w, ALTO_CABECERA, "fillHeader", 1.6, 3.0);
        c.t(x + 8.0, y + 15.5, &o.rotulo(), 10.5, 700, "textMain", "start", MONO);
        if op.marcar && !v.marcos.is_empty() && !vivos.contains(&o.id) {
            c.add(format!(
                r#"<rect x="{:.1}" y="{:.1}" width="{:.1}" height="{:.1}" rx="5" fill="none" stroke="{}" stroke-width="1.4" stroke-dasharray="5 3"/>"#,
                x - 4.0,
                y - 4.0,
                w + 8.0,
                caja.h + 8.0,
                c.pal.stroke_muted
            ));
            c.t(x + w + 8.0, y + 15.0, "inalcanzable", 9.5, 700, "textMuted", "start", SANS);
        }
        match &o.contenido {
            Contenido::Elementos(e) => {
                // Los índices van arriba: abajo salen las referencias de las celdas.
                let cy = y + ALTO_CABECERA + 18.0;
                if e.is_empty() {
                    c.t(x + 10.0, cy + 19.0, "length 0", 10.0, 400, "textSub", "start", MONO);
                }
                for (k, r) in e.iter().enumerate() {
                    let cx = x + 8.0 + k as f64 * ANCHO_CELDA;
                    c.r(cx, cy, ANCHO_CELDA, ALTO_CELDA, "fillPointerBox", 1.4, 0.0);
                    c.t(cx + ANCHO_CELDA / 2.0, cy - 5.0, &format!("[{k}]"), 9.0, 400, "textSub", "middle", MONO);
                    if let Some((desde, destino)) = dibujar_valor(c, r, cx, cy, ANCHO_CELDA, ALTO_CELDA, op) {
                        let n = salidas_bajo.entry(o.id.clone()).or_insert(0);
                        *n += 1;
                        let bajo = y + caja.h + 2.0 + *n as f64 * 4.0;
                        flechas.push(Flecha { desde, destino, del_stack: false, bajo: Some(bajo) });
                    }
                }
            }
            Contenido::Campos(campos) => {
                let ancho_valores = campos.iter().map(ancho_valor).fold(36.0, f64::max);
                for (k, r) in campos.iter().enumerate() {
                    let fy = y + ALTO_CABECERA + k as f64 * ALTO_FILA + (ALTO_FILA - ALTO_CAJA) / 2.0;
                    c.t(x + 8.0, fy + ALTO_CAJA / 2.0 + 4.0, &r.rotulo(), 10.5, 400, "textMain", "start", MONO);
                    let vx = x + w - ancho_valores - 8.0;
                    c.r(vx, fy, ancho_valores, ALTO_CAJA, "fillPointerBox", 1.4, 2.0);
                    if let Some((desde, destino)) = dibujar_valor(c, r, vx, fy, ancho_valores, ALTO_CAJA, op) {
                        flechas.push(Flecha { desde, destino, del_stack: false, bajo: None });
                    }
                }
            }
            Contenido::Texto(t) => {
                let fy = y + ALTO_CABECERA + (ALTO_FILA - ALTO_CAJA) / 2.0 + 2.0;
                if t.is_empty() || op.sin_valores {
                    c.vacio(x + 12.0, x + w - 12.0, fy + ALTO_CAJA - 4.0);
                } else {
                    c.t(x + w / 2.0, fy + ALTO_CAJA / 2.0 + 4.0, t, 11.5, 700, "textMain", "middle", MONO);
                }
            }
        }
    }

    // Flechas: del stack, al borde izquierdo del objeto; entre objetos, por un canal a la derecha
    // del heap, al borde derecho.
    let mut entradas_izq: HashMap<String, usize> = HashMap::new();
    let mut entradas_der: HashMap<String, usize> = HashMap::new();
    let mut canal = 0usize;
    let borde_canal = borde_heap + 14.0;
    for f in &flechas {
        let d = &cajas[&f.destino];
        if f.del_stack {
            let k = entradas_izq.entry(f.destino.clone()).or_insert(0);
            let ty = (d.y + 11.0 + *k as f64 * 7.0).min(d.y + d.h - 6.0);
            *k += 1;
            c.linea(f.desde.0, f.desde.1, d.x - 3.0, ty, "accentPrimary", 1.6, Some("main"), "");
        } else {
            let k = entradas_der.entry(f.destino.clone()).or_insert(0);
            let ty = (d.y + 11.0 + *k as f64 * 7.0).min(d.y + d.h - 6.0);
            *k += 1;
            let cx = borde_canal + canal as f64 * PASO_CANAL;
            canal += 1;
            let mut puntos = vec![f.desde];
            if let Some(bajo) = f.bajo {
                puntos.push((f.desde.0, bajo));
                puntos.push((cx, bajo));
            } else {
                puntos.push((cx, f.desde.1));
            }
            puntos.push((cx, ty));
            puntos.push((d.x + d.w + 3.0, ty));
            c.ruta(&puntos);
        }
    }

    let ancho_marcas = if op.marcar && v.objetos.iter().any(|o| !vivos.contains(&o.id)) && !v.marcos.is_empty() {
        x_heap + ancho_objetos + 8.0 + ancho_texto("inalcanzable", 9.5) + 6.0
    } else {
        0.0
    };
    let mut ancho = (borde_canal + canal as f64 * PASO_CANAL).max(borde_heap + 8.0).max(ancho_marcas);
    let bajos = salidas_bajo.values().copied().max().unwrap_or(0) as f64 * 4.0 + 4.0;
    let mut alto = fin_stack.max(fin_heap + bajos) + 8.0;
    if let Some((z_stack, z_heap)) = &op.zonas {
        if d.ancho_stack > 0.0 {
            zona(c, -6.0, ancho_stack + 10.0, top + 2.0, alto + 4.0, z_stack);
        }
        zona(c, x_heap - 16.0, ancho + 6.0, top + 2.0, alto + 4.0, z_heap);
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
    let python = match b.get("variant") {
        None | Some(Value::Null) => false,
        Some(Value::String(v)) if v == "java" => false,
        Some(Value::String(v)) if v == "python" => true,
        Some(otro) => return Err(error(format!("variante no soportada: {} (java o python)", texto(otro)))),
    };
    PYTHON.with(|p| p.set(python));
    let resultado = render_con_estilo(c, b);
    PYTHON.with(|p| p.set(false));
    resultado
}

fn render_con_estilo(c: &mut Lienzo, b: &Bloque) -> Result<Punto, ErrorDiagrama> {
    let op = Opciones {
        sin_valores: b.get("blankValues").is_some_and(es_truthy),
        sin_refs: b.get("blankRefs").is_some_and(es_truthy),
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
        assert!(error_de(json!({"type": "object_heap", "variant": "c", "objects": [{"id": "A"}]})).contains("java o python"));
    }
}
