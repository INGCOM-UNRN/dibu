//! Núcleo de dibujo SVG y utilidades compartidas para todas las familias de diagramas de `dibu`.

#![warn(clippy::missing_safety_doc)]

use serde_json::Value;
use sha1::{Digest, Sha1};
use std::collections::BTreeMap;
use std::fmt::Write as _;
use thiserror::Error;

pub mod theme;
pub use theme::{
    Paleta, Theme, Tipografia, PALETA_COLOR, PALETA_DARK, PALETA_HIGH_CONTRAST, PALETA_TINTA,
    PRESETS,
};

pub const MONO: &str = "DejaVu Sans Mono, Menlo, Consolas, Courier New, monospace";
pub const SANS: &str = "DejaVu Sans, Inter, Segoe UI, Roboto, Helvetica, Arial, sans-serif";
pub const MAX_SVG_DIM: f64 = 1_000_000.0;
const MAX_SVG_DIM_I64: i64 = 1_000_000;

pub type Punto = (f64, f64);
pub type Bloque = serde_json::Map<String, Value>;
pub type RenderFn = fn(&mut Lienzo, &Bloque) -> Result<Punto, ErrorDiagrama>;

// ---------------------------------------------------------------- valores en blanco
//
// `!blank 34` (o `34.blank`) es un valor que se usa para calcular el diagrama pero se dibuja como
// un renglón para completar. Viaja como un texto entre marcas invisibles del área de uso privado:
// `BLANCO_TEXTO` o `BLANCO_ESCALAR` al principio (el valor era un texto, o un número o un booleano)
// y `BLANCO_FIN` al final. Los lectores de números lo leen con `plano`; `Lienzo::t` dibuja la parte
// marcada como un renglón del mismo ancho, así el dibujo no delata la respuesta.

/// Comienzo de un valor en blanco que era un texto.
pub const BLANCO_TEXTO: char = '\u{E000}';
/// Fin de un valor en blanco.
pub const BLANCO_FIN: char = '\u{E001}';
/// Comienzo de un valor en blanco que era un número o un booleano.
pub const BLANCO_ESCALAR: char = '\u{E002}';

const SUFIJO_BLANCO: &str = ".blank";

fn es_marca(c: char) -> bool {
    matches!(c, BLANCO_TEXTO | BLANCO_FIN | BLANCO_ESCALAR)
}

/// Si `v` es un valor en blanco (o un texto que contiene uno).
pub fn es_blanco(v: &Value) -> bool {
    matches!(v, Value::String(s) if s.chars().any(es_marca))
}

/// Un texto sin las marcas de blanco, con su contenido.
pub fn sin_marcas(s: &str) -> String {
    s.chars().filter(|c| !es_marca(*c)).collect()
}

/// El valor con el que se calcula: un valor en blanco sin sus marcas, con su tipo original
/// (`!blank 34` es el número 34); cualquier otro, tal cual.
pub fn plano(v: &Value) -> std::borrow::Cow<'_, Value> {
    let Value::String(s) = v else {
        return std::borrow::Cow::Borrowed(v);
    };
    let mut cs = s.chars();
    match (cs.next(), s.chars().last()) {
        (Some(BLANCO_ESCALAR), Some(BLANCO_FIN)) => {
            let adentro = sin_marcas(s);
            let val = serde_yaml::from_str::<serde_yaml::Value>(&adentro)
                .ok()
                .and_then(|y| serde_json::to_value(y).ok())
                .filter(|j| j.is_number() || j.is_boolean())
                .unwrap_or(Value::String(adentro));
            std::borrow::Cow::Owned(val)
        }
        (Some(BLANCO_TEXTO), Some(BLANCO_FIN)) => std::borrow::Cow::Owned(Value::String(sin_marcas(s))),
        _ => std::borrow::Cow::Borrowed(v),
    }
}

thread_local! {
    /// Los números en blanco que leyó el render en curso (se vacía al empezar cada render).
    static NUMEROS_EN_BLANCO: std::cell::RefCell<Vec<f64>> = const { std::cell::RefCell::new(Vec::new()) };
}

/// Para los lectores de números: si `v` es un número en blanco, lo anota para el render en curso
/// (así `numero_en_blanco` lo reconoce al escribirlo) y devuelve el valor sin marcas.
pub fn plano_numerico(v: &Value) -> std::borrow::Cow<'_, Value> {
    let p = plano(v);
    if let (true, Some(x)) = (es_blanco(v), p.as_f64()) {
        NUMEROS_EN_BLANCO.with(|n| n.borrow_mut().push(x));
    }
    p
}

/// Si `x` es uno de los números en blanco del render en curso. Lo usan las funciones que
/// escriben valores de datos (no las coordenadas): un número igual que aparezca en otro lugar
/// también sale en blanco, así nunca se ve la respuesta.
pub fn numero_en_blanco(x: f64) -> bool {
    x.is_finite()
        && NUMEROS_EN_BLANCO.with(|n| n.borrow().iter().any(|y| (x - y).abs() <= 1e-9 * x.abs().max(1.0)))
}

/// `texto` como un valor en blanco (para escribirlo con `Lienzo::t`).
pub fn marcar_texto(texto: &str) -> String {
    format!("{BLANCO_TEXTO}{texto}{BLANCO_FIN}")
}

/// `texto` en blanco si `x` es un número en blanco del render en curso.
pub fn numero_mostrado(x: f64, texto: String) -> String {
    if numero_en_blanco(x) {
        marcar_texto(&texto)
    } else {
        texto
    }
}

fn marcar_json(v: &Value) -> Value {
    match v {
        Value::String(s) => Value::String(format!("{BLANCO_TEXTO}{}{BLANCO_FIN}", sin_marcas(s))),
        Value::Number(n) => Value::String(format!("{BLANCO_ESCALAR}{n}{BLANCO_FIN}")),
        Value::Bool(b) => Value::String(format!("{BLANCO_ESCALAR}{b}{BLANCO_FIN}")),
        Value::Null => Value::String(format!("{BLANCO_TEXTO}{BLANCO_FIN}")),
        Value::Array(l) => Value::Array(l.iter().map(marcar_json).collect()),
        Value::Object(m) => Value::Object(m.iter().map(|(k, x)| (k.clone(), marcar_json(x))).collect()),
    }
}

/// `X.blank` → el valor `X` en blanco, con el tipo que tendría `X` en YAML (`34.blank` es el
/// número 34). Recorre listas y objetos. Las claves no se tocan.
pub fn normalizar_blancos(v: Value) -> Value {
    match v {
        Value::String(s) if s.len() > SUFIJO_BLANCO.len() && s.ends_with(SUFIJO_BLANCO) && !s.chars().any(es_marca) => {
            let adentro = &s[..s.len() - SUFIJO_BLANCO.len()];
            let val = serde_yaml::from_str::<serde_yaml::Value>(adentro)
                .ok()
                .and_then(|y| serde_json::to_value(y).ok())
                .filter(|j| j.is_number() || j.is_boolean())
                .unwrap_or_else(|| Value::String(adentro.to_string()));
            marcar_json(&val)
        }
        Value::Array(l) => Value::Array(l.into_iter().map(normalizar_blancos).collect()),
        Value::Object(m) => Value::Object(m.into_iter().map(|(k, x)| (k, normalizar_blancos(x))).collect()),
        otro => otro,
    }
}

fn contiene_sufijo_blanco(v: &Value) -> bool {
    match v {
        Value::String(s) => s.ends_with(SUFIJO_BLANCO),
        Value::Array(l) => l.iter().any(contiene_sufijo_blanco),
        Value::Object(m) => m.values().any(contiene_sufijo_blanco),
        _ => false,
    }
}

/// Las rutas (punteros JSON) de los valores en blanco de `v`.
fn rutas_en_blanco(v: &Value, base: &str, salida: &mut Vec<String>) {
    match v {
        Value::Object(m) => {
            for (k, x) in m {
                rutas_en_blanco(x, &format!("{base}/{}", k.replace('~', "~0").replace('/', "~1")), salida);
            }
        }
        Value::Array(l) => {
            for (i, x) in l.iter().enumerate() {
                rutas_en_blanco(x, &format!("{base}/{i}"), salida);
            }
        }
        otro if es_blanco(otro) => salida.push(base.to_string()),
        _ => {}
    }
}

/// `/frames/0/vars/1/value` → `frames[0].vars[1].value`.
fn ruta_legible(puntero: &str) -> String {
    let mut salida = String::new();
    for parte in puntero.split('/').skip(1) {
        let parte = parte.replace("~1", "/").replace("~0", "~");
        if parte.chars().all(|c| c.is_ascii_digit()) && !parte.is_empty() {
            salida.push_str(&format!("[{parte}]"));
        } else {
            if !salida.is_empty() {
                salida.push('.');
            }
            salida.push_str(&parte);
        }
    }
    salida
}

/// YAML → JSON, con `!blank X` como un valor en blanco. Otras etiquetas quedan como antes.
fn yaml_a_json(y: serde_yaml::Value) -> Result<Value, ErrorDiagrama> {
    use serde_yaml::Value as Y;
    Ok(match y {
        Y::Tagged(t) if t.tag == "blank" => marcar_json(&yaml_a_json(t.value)?),
        Y::Sequence(l) => Value::Array(l.into_iter().map(yaml_a_json).collect::<Result<_, _>>()?),
        Y::Mapping(m) => {
            let mut salida = serde_json::Map::new();
            for (k, x) in m {
                let k = match serde_json::to_value(k)
                    .map_err(|e| ErrorDiagrama::new(format!("el bloque no se pudo normalizar a JSON: {e}")))?
                {
                    Value::String(s) => s,
                    otra => otra.to_string(),
                };
                salida.insert(k, yaml_a_json(x)?);
            }
            Value::Object(salida)
        }
        otro => serde_json::to_value(otro)
            .map_err(|e| ErrorDiagrama::new(format!("el bloque no se pudo normalizar a JSON: {e}")))?,
    })
}

#[derive(Debug, Error)]
#[error("{0}")]
pub struct ErrorDiagrama(pub String);

impl ErrorDiagrama {
    pub fn new(msg: impl Into<String>) -> Self {
        Self(msg.into())
    }
}

pub fn esc(texto: &str) -> String {
    let mut out = String::with_capacity(texto.len());
    // Un valor en blanco que llega acá (un texto que no pasa por `Lienzo::t`, o un atributo como
    // `aria-label`) se escribe como `____`: nunca se ve su contenido.
    let mut en_blanco = false;
    for ch in texto.chars() {
        if en_blanco {
            en_blanco = ch != BLANCO_FIN;
            continue;
        }
        match ch {
            BLANCO_TEXTO | BLANCO_ESCALAR => {
                out.push_str("____");
                en_blanco = true;
            }
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&#x27;"),
            '\x00'..='\x08' | '\x0b' | '\x0c' | '\x0e'..='\x1f' => {}
            BLANCO_FIN => {}
            _ if (ch as u32 & 0xFFFE) == 0xFFFE => {}
            _ => out.push(ch),
        }
    }
    out
}

thread_local! {
    /// Escala tipográfica y factor de ancho del theme con el que se está renderizando.
    static TIPOGRAFIA_ACTIVA: std::cell::Cell<(f64, f64)> = const { std::cell::Cell::new((1.0, 0.62)) };
}

/// (escala, factor de ancho por carácter) del theme del render en curso.
pub fn tipografia_activa() -> (f64, f64) {
    TIPOGRAFIA_ACTIVA.with(std::cell::Cell::get)
}

/// Fija la tipografía activa mientras vive el guardián (la restaura al salir, aun con `?`).
pub struct GuardiaTipografia((f64, f64));

impl GuardiaTipografia {
    pub fn activar(t: &Tipografia) -> Self {
        let previa = tipografia_activa();
        TIPOGRAFIA_ACTIVA.with(|c| c.set((t.scale, t.char_width_factor)));
        Self(previa)
    }
}

impl Drop for GuardiaTipografia {
    fn drop(&mut self) {
        TIPOGRAFIA_ACTIVA.with(|c| c.set(self.0));
    }
}

/// Ancho estimado de un texto. Respeta `typography.scale` y `charWidthFactor` del theme activo,
/// así los renderizadores dimensionan celdas y cajas acordes al tamaño real de la letra.
pub fn ancho_texto(texto: &str, size: f64) -> f64 {
    let (escala, factor) = tipografia_activa();
    texto.chars().filter(|c| !es_marca(*c)).count() as f64 * size * escala * factor
}

/// Avance de `n` caracteres en la fuente monoespaciada (DejaVu Sans Mono ≈ 0,602 em), para
/// posicionar caracteres sueltos (sangrías, subrayados) dentro de un texto ya dibujado.
pub fn avance_mono(n: f64, size: f64) -> f64 {
    let (escala, factor) = tipografia_activa();
    n * size * escala * 0.602 * (factor / 0.62)
}

pub fn sha1_hex(datos: &[u8]) -> String {
    let mut hasher = Sha1::new();
    hasher.update(datos);
    let digest = hasher.finalize();
    let mut s = String::with_capacity(40);
    for b in digest {
        let _ = write!(&mut s, "{b:02x}");
    }
    s
}

/// Nombre del SVG versionado para un bloque Markdown: `estructura-<sha1(cuerpo.strip())[:12]>.svg`.
pub fn nombre_svg(cuerpo: &str) -> String {
    let hash = sha1_hex(cuerpo.trim().as_bytes());
    format!("estructura-{}.svg", &hash[..12])
}

pub fn valor_str(v: &Value) -> String {
    match v {
        Value::Null => String::new(),
        Value::String(s) => s.clone(),
        Value::Bool(b) => if *b { "true" } else { "false" }.to_string(),
        Value::Number(n) => n.to_string(),
        _ => v.to_string(),
    }
}

pub fn fmt_num_clean(x: f64) -> String {
    if (x - x.round()).abs() < 1e-12 && x.abs() < 1e15 {
        format!("{}", x.round() as i64)
    } else {
        format!("{x}")
    }
}

pub fn bool_opt(b: &Bloque, clave: &str, defecto: bool) -> bool {
    b.get(clave).and_then(Value::as_bool).unwrap_or(defecto)
}

pub fn str_opt<'a>(b: &'a Bloque, clave: &str) -> Option<&'a str> {
    b.get(clave).and_then(Value::as_str).filter(|s| !s.is_empty())
}

pub struct Lienzo {
    pub pal: Paleta,
    pub typography: Tipografia,
    pub sufijo: String,
    pub partes: Vec<String>,
    pub ancho_min: f64,
}

impl Lienzo {
    pub fn new(pal: Paleta, sufijo: String) -> Self {
        Self {
            pal,
            typography: Tipografia::default(),
            sufijo,
            partes: Vec::with_capacity(64),
            ancho_min: 0.0,
        }
    }

    pub fn con_tema(tema: &Theme, sufijo: String) -> Self {
        Self {
            pal: tema.colors.clone(),
            typography: tema.typography.clone(),
            sufijo,
            partes: Vec::with_capacity(64),
            ancho_min: 0.0,
        }
    }

    pub fn sub_lienzo(&self) -> Self {
        Self {
            pal: self.pal.clone(),
            typography: self.typography.clone(),
            sufijo: self.sufijo.clone(),
            partes: Vec::with_capacity(32),
            ancho_min: 0.0,
        }
    }

    pub fn font_sans(&self) -> &str {
        &self.typography.sans
    }

    pub fn font_mono(&self) -> &str {
        &self.typography.mono
    }

    pub fn resolver_fuente<'a>(&'a self, fuente: &'a str) -> &'a str {
        if fuente == MONO {
            &self.typography.mono
        } else if fuente == SANS {
            &self.typography.sans
        } else {
            fuente
        }
    }

    pub fn font_size(&self, size: f64) -> f64 {
        size * self.typography.scale
    }

    pub fn ancho_texto(&self, texto: &str, size: f64) -> f64 {
        texto.chars().count() as f64
            * (size * self.typography.scale)
            * self.typography.char_width_factor
    }

    pub fn add(&mut self, pieza: impl Into<String>) {
        self.partes.push(pieza.into());
    }

    #[allow(clippy::too_many_arguments)]
    pub fn t(
        &mut self,
        x: f64,
        y: f64,
        texto: &str,
        size: f64,
        peso: u16,
        color: &str,
        ancla: &str,
        fuente: &str,
    ) {
        let fam = self.resolver_fuente(fuente).to_string();
        let sz = self.font_size(size);
        let fill = self.pal.color(color).to_string();
        if !texto.chars().any(es_marca) {
            self.add(format!(
                r#"<text x="{x:.1}" y="{y:.1}" text-anchor="{ancla}" font-family="{fam}" font-size="{}" font-weight="{peso}" fill="{fill}">{}</text>"#,
                fmt_num_clean(sz),
                esc(texto)
            ));
            return;
        }
        // Con valores en blanco: el texto en tramos; cada valor en blanco es un renglón para
        // completar del ancho que tendría (al menos dos caracteres), y el resto se escribe igual.
        let mut tramos: Vec<(String, bool)> = Vec::new();
        let mut actual = String::new();
        let mut en_blanco = false;
        for ch in texto.chars() {
            match ch {
                BLANCO_TEXTO | BLANCO_ESCALAR => {
                    if !actual.is_empty() {
                        tramos.push((std::mem::take(&mut actual), en_blanco));
                    }
                    en_blanco = true;
                }
                BLANCO_FIN => {
                    tramos.push((std::mem::take(&mut actual), true));
                    en_blanco = false;
                }
                _ => actual.push(ch),
            }
        }
        if !actual.is_empty() {
            tramos.push((actual, en_blanco));
        }
        let minimo = ancho_texto("000", size);
        let anchos: Vec<f64> = tramos
            .iter()
            .map(|(t, b)| if *b { ancho_texto(t, size).max(minimo) } else { ancho_texto(t, size) })
            .collect();
        let total: f64 = anchos.iter().sum();
        let mut xi = match ancla {
            "middle" => x - total / 2.0,
            "end" => x - total,
            _ => x,
        };
        for ((t, blanco), w) in tramos.iter().zip(anchos) {
            if *blanco {
                self.vacio(xi + 1.0, xi + w - 1.0, y + 1.0);
            } else {
                self.add(format!(
                    r#"<text x="{xi:.1}" y="{y:.1}" text-anchor="start" font-family="{fam}" font-size="{}" font-weight="{peso}" fill="{fill}" xml:space="preserve">{}</text>"#,
                    fmt_num_clean(sz),
                    esc(t)
                ));
            }
            xi += w;
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub fn r(
        &mut self,
        x: f64,
        y: f64,
        w: f64,
        h: f64,
        relleno: &str,
        trazo: f64,
        rx: f64,
    ) {
        self.add(format!(
            r#"<rect x="{x:.1}" y="{y:.1}" width="{w:.1}" height="{h:.1}" rx="{}" fill="{}" stroke="{}" stroke-width="{}"/>"#,
            fmt_num_clean(rx),
            self.pal.color(relleno),
            self.pal.stroke_main,
            fmt_num_clean(trazo)
        ));
    }

    #[allow(clippy::too_many_arguments)]
    pub fn linea(
        &mut self,
        x1: f64,
        y1: f64,
        x2: f64,
        y2: f64,
        color: &str,
        ancho: f64,
        flecha: Option<&str>,
        guiones: &str,
    ) {
        let marca = match flecha {
            Some(f) if !f.is_empty() => format!(r#" marker-end="url(#flecha-{f}-{})""#, self.sufijo),
            _ => String::new(),
        };
        let dash = if guiones.is_empty() {
            String::new()
        } else {
            format!(r#" stroke-dasharray="{guiones}""#)
        };
        self.add(format!(
            r#"<line x1="{x1:.1}" y1="{y1:.1}" x2="{x2:.1}" y2="{y2:.1}" stroke="{}" stroke-width="{}"{dash}{marca}/>"#,
            self.pal.color(color),
            fmt_num_clean(ancho)
        ));
    }

    pub fn vacio(&mut self, x1: f64, x2: f64, y: f64) {
        self.linea(x1, y, x2, y, "strokeMuted", 1.0, None, "3 2");
    }

    pub fn ruta(&mut self, puntos: &[Punto]) {
        if puntos.is_empty() {
            return;
        }
        let camino: String = puntos
            .iter()
            .map(|(x, y)| format!("{x:.1} {y:.1}"))
            .collect::<Vec<_>>()
            .join(" L ");
        self.add(format!(
            r#"<path d="M {camino}" fill="none" stroke="{}" stroke-width="1.6" marker-end="url(#flecha-main-{})"/>"#,
            self.pal.accent_primary, self.sufijo
        ));
    }

    pub fn punto(&mut self, x: f64, y: f64, color: &str, radio: f64) {
        self.add(format!(
            r#"<circle cx="{x:.1}" cy="{y:.1}" r="{}" fill="{}"/>"#,
            fmt_num_clean(radio),
            self.pal.color(color)
        ));
    }

    pub fn null(&mut self, x: f64, y: f64) {
        self.linea(x, y - 10.0, x, y + 10.0, "strokeMain", 2.0, None, "");
        self.linea(x + 5.0, y - 6.0, x + 5.0, y + 6.0, "strokeMain", 1.5, None, "");
        self.t(x + 12.0, y + 4.0, "NULL", 10.0, 700, "textMuted", "start", MONO);
    }
}

/// Título y subtítulo estándar; devuelve `y` donde empieza el contenido.
pub fn encabezado(c: &mut Lienzo, bloque: &Bloque, extra: &str) -> f64 {
    let titulo = str_opt(bloque, "title");
    let subtitulo = str_opt(bloque, "subtitle");
    if titulo.is_none() && subtitulo.is_none() && extra.is_empty() {
        return 0.0;
    }
    if let Some(t) = titulo {
        c.t(0.0, 16.0, t, 14.0, 700, "textMain", "start", SANS);
        c.ancho_min = c.ancho_min.max(t.chars().count() as f64 * 8.6 * tipografia_activa().0);
    }
    let mut partes = Vec::new();
    if let Some(s) = subtitulo {
        partes.push(s);
    }
    if !extra.is_empty() {
        partes.push(extra);
    }
    if !partes.is_empty() {
        let texto = partes.join(" | ");
        let y = if titulo.is_some() { 32.0 } else { 14.0 };
        c.t(0.0, y, &texto, 10.5, 400, "textMuted", "start", MONO);
        c.ancho_min = c.ancho_min.max(texto.chars().count() as f64 * 6.4 * tipografia_activa().0);
    }
    if titulo.is_some() { 40.0 } else { 22.0 }
}

/// Parsea YAML o JSON en una lista de bloques (admite bloque único, lista o `{blocks: [...]}`).
pub fn leer_bloques(texto: &str) -> Result<Vec<Bloque>, ErrorDiagrama> {
    let texto = texto.strip_prefix('\u{feff}').unwrap_or(texto);
    let yaml_val: serde_yaml::Value = serde_yaml::from_str(texto)
        .map_err(|e| ErrorDiagrama::new(format!("el bloque no es YAML/JSON válido: {e}")))?;
    let json_val = normalizar_blancos(yaml_a_json(yaml_val)?);
    let raiz = match json_val {
        Value::Object(mut obj) if obj.contains_key("blocks") => obj.remove("blocks").unwrap(),
        otro => otro,
    };
    let lista = match raiz {
        Value::Array(arr) => arr,
        otro => vec![otro],
    };
    let mut salida = Vec::with_capacity(lista.len());
    for item in lista {
        match item {
            Value::Object(map) => salida.push(map),
            _ => {
                return Err(ErrorDiagrama::new(
                    "cada bloque tiene que ser un diccionario con 'type'",
                ))
            }
        }
    }
    Ok(salida)
}

/// Registro de renderizadores por nombre de `type`.
#[derive(Default)]
pub struct Registro {
    mapa: BTreeMap<&'static str, RenderFn>,
}

impl Registro {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn registrar(&mut self, tipo: &'static str, f: RenderFn) {
        self.mapa.insert(tipo, f);
    }

    pub fn tipos(&self) -> Vec<&'static str> {
        self.mapa.keys().copied().collect()
    }

    pub fn contiene(&self, tipo: &str) -> bool {
        self.mapa.contains_key(tipo)
    }

    pub fn renderizar(&self, bloque: &Bloque, tinta: bool) -> Result<String, ErrorDiagrama> {
        self.renderizar_con_tema(bloque, &Theme::para_tinta(tinta))
    }

    pub fn renderizar_con_tema(
        &self,
        bloque: &Bloque,
        tema_base: &Theme,
    ) -> Result<String, ErrorDiagrama> {
        // Un bloque que no pasó por `leer_bloques` (JSON de la API) también admite `X.blank`.
        let normalizado;
        let bloque = if bloque.values().any(contiene_sufijo_blanco) {
            normalizado = match normalizar_blancos(Value::Object(bloque.clone())) {
                Value::Object(m) => m,
                _ => unreachable!("normalizar un objeto da un objeto"),
            };
            &normalizado
        } else {
            bloque
        };
        let raiz = Value::Object(bloque.clone());
        let mut blancos = Vec::new();
        rutas_en_blanco(&raiz, "", &mut blancos);
        if blancos.is_empty() {
            return self.renderizar_interno(bloque, tema_base, None).map(|(svg, _)| svg);
        }
        // Con valores en blanco, cada uno tiene que cambiar el dibujo: si sin su marca sale igual,
        // el diagrama no lo dibuja en blanco (lo calcula y lo muestra con otro formato, o no lo
        // muestra), y se avisa en lugar de dejar la respuesta a la vista.
        let sin_marca = |ruta: &str| -> Bloque {
            let mut v = raiz.clone();
            if let Some(x) = v.pointer_mut(ruta) {
                *x = plano(x).into_owned();
            }
            match v {
                Value::Object(m) => m,
                _ => unreachable!(),
            }
        };
        let (svg, sufijo) = match self.renderizar_interno(bloque, tema_base, None) {
            Ok(r) => r,
            Err(e) => {
                for ruta in &blancos {
                    if self.renderizar_interno(&sin_marca(ruta), tema_base, None).is_ok() {
                        return Err(ErrorDiagrama::new(format!(
                            "{}: este diagrama no admite un valor en blanco ahí ({})",
                            ruta_legible(ruta),
                            sin_marcas(&e.0)
                        )));
                    }
                }
                return Err(ErrorDiagrama::new(sin_marcas(&e.0)));
            }
        };
        // Lo que deja un valor en blanco en el SVG: un renglón para completar o `____`.
        let huellas = |svg: &str| svg.matches(r#"stroke-dasharray="3 2""#).count() + svg.matches("____").count();
        for ruta in &blancos {
            if let Ok((otro, _)) = self.renderizar_interno(&sin_marca(ruta), tema_base, Some(&sufijo)) {
                if otro == svg {
                    return Err(ErrorDiagrama::new(format!(
                        "{}: este diagrama no dibuja ese valor como texto, así que no se puede dejar en blanco",
                        ruta_legible(ruta)
                    )));
                }
                // Cambió el dibujo sin dejar un renglón: el campo leyó otro valor.
                if huellas(&svg) <= huellas(&otro) {
                    return Err(ErrorDiagrama::new(format!(
                        "{}: este diagrama no admite un valor en blanco ahí",
                        ruta_legible(ruta)
                    )));
                }
            }
        }
        Ok(svg)
    }

    /// El render, sin verificar los valores en blanco; devuelve también el sufijo de los `id`.
    fn renderizar_interno(
        &self,
        bloque: &Bloque,
        tema_base: &Theme,
        sufijo_fijo: Option<&str>,
    ) -> Result<(String, String), ErrorDiagrama> {
        NUMEROS_EN_BLANCO.with(|n| n.borrow_mut().clear());
        let tipo = bloque
            .get("type")
            .and_then(Value::as_str)
            .ok_or_else(|| ErrorDiagrama::new("el bloque tiene que tener 'type'"))?;
        let render_fn = self.mapa.get(tipo).ok_or_else(|| {
            ErrorDiagrama::new(format!(
                "tipo de bloque no soportado: {tipo:?} (disponibles: {})",
                self.tipos().join(", ")
            ))
        })?;
        let tema = if let Some(val) = bloque.get("theme") {
            Theme::from_value(val, tema_base)?
        } else if let Some(ink_saver) = bloque.get("inkSaver").and_then(Value::as_bool) {
            let mut t = Theme::para_tinta(ink_saver);
            t.typography = tema_base.typography.clone();
            t
        } else {
            tema_base.clone()
        };
        let canon = serde_json::to_string(bloque).unwrap_or_default();
        let sufijo = if let Some(s) = sufijo_fijo {
            s.to_string()
        } else if tema.es_default_clasico() {
            sha1_hex(canon.as_bytes())[..8].to_string()
        } else {
            let mut entrada = canon;
            entrada.push('|');
            entrada.push_str(&tema.huella());
            sha1_hex(entrada.as_bytes())[..8].to_string()
        };
        let mut lienzo = Lienzo::con_tema(&tema, sufijo.clone());
        let (ancho_raw, alto) = {
            let _tipografia = GuardiaTipografia::activar(&tema.typography);
            render_fn(&mut lienzo, bloque)?
        };
        if !ancho_raw.is_finite() || !alto.is_finite() || ancho_raw < 0.0 || alto < 0.0 {
            return Err(ErrorDiagrama::new(format!(
                "{tipo}: dimensiones de SVG inválidas ({ancho_raw}×{alto})"
            )));
        }
        let ancho = ancho_raw.max(lienzo.ancho_min);
        if !ancho.is_finite() || ancho < 0.0 {
            return Err(ErrorDiagrama::new(format!(
                "{tipo}: dimensiones de SVG inválidas ({ancho}×{alto})"
            )));
        }
        let m = 8.0_f64;
        let w_f = (ancho + 2.0 * m).ceil();
        let h_f = (alto + 2.0 * m).ceil();
        if !w_f.is_finite()
            || !h_f.is_finite()
            || w_f <= 0.0
            || h_f <= 0.0
            || w_f > MAX_SVG_DIM
            || h_f > MAX_SVG_DIM
        {
            return Err(ErrorDiagrama::new(format!(
                "{tipo}: dimensiones de SVG fuera de rango ({w_f}×{h_f})"
            )));
        }
        let w = w_f as i64;
        let h = h_f as i64;
        let mi = m as i64;
        let aria = bloque
            .get("title")
            .and_then(Value::as_str)
            .filter(|s| !s.is_empty())
            .unwrap_or(tipo);
        let defs = format!(
            r#"<defs><marker id="flecha-main-{sufijo}" viewBox="0 0 10 10" refX="8.5" refY="5" markerWidth="6.5" markerHeight="6.5" orient="auto-start-reverse"><path d="M 0 1.5 L 9 5 L 0 8.5 z" fill="{}"/></marker><marker id="flecha-sub-{sufijo}" viewBox="0 0 10 10" refX="8.5" refY="5" markerWidth="6" markerHeight="6" orient="auto-start-reverse"><path d="M 0 1.5 L 9 5 L 0 8.5 z" fill="{}"/></marker></defs>"#,
            tema.colors.accent_primary, tema.colors.stroke_sub
        );
        let mut svg = String::with_capacity(512 + lienzo.partes.len() * 80);
        let _ = write!(
            &mut svg,
            r#"<svg xmlns="http://www.w3.org/2000/svg" class="estructura" width="{w}" height="{h}" viewBox="{neg_m} {neg_m} {w} {h}" role="img" aria-label="{}">{defs}<rect x="{neg_m}" y="{neg_m}" width="{w}" height="{h}" fill="{}"/>"#,
            esc(aria),
            tema.colors.background,
            neg_m = -mi
        );
        for p in &lienzo.partes {
            svg.push_str(p);
        }
        svg.push_str("</svg>");
        Ok((svg, sufijo))
    }

    pub fn renderizar_bloque_markdown(&self, cuerpo: &str, tinta: bool) -> Result<String, ErrorDiagrama> {
        self.renderizar_bloque_markdown_con_tema(cuerpo, &Theme::para_tinta(tinta))
    }

    pub fn renderizar_bloque_markdown_con_tema(
        &self,
        cuerpo: &str,
        tema_base: &Theme,
    ) -> Result<String, ErrorDiagrama> {
        let bloques = leer_bloques(cuerpo)?;
        let mut svgs = Vec::with_capacity(bloques.len());
        for b in &bloques {
            svgs.push(self.renderizar_con_tema(b, tema_base)?);
        }
        apilar_svgs(&svgs)
    }
}

fn extraer_attr_int(svg: &str, nombre: &str) -> Result<i64, ErrorDiagrama> {
    let patron = format!(r#"{nombre}=""#);
    let idx = svg
        .find(&patron)
        .ok_or_else(|| ErrorDiagrama::new(format!("el SVG no tiene {nombre}")))?;
    let resto = &svg[idx + patron.len()..];
    let fin = resto
        .find('"')
        .ok_or_else(|| ErrorDiagrama::new(format!("atributo {nombre} mal formado")))?;
    resto[..fin]
        .parse::<i64>()
        .map_err(|_| ErrorDiagrama::new(format!("atributo {nombre} no es entero")))
}

/// Apila múltiples SVGs verticalmente con separación de 8px (idéntico a `alucarD`).
pub fn apilar_svgs(svgs: &[String]) -> Result<String, ErrorDiagrama> {
    if svgs.is_empty() {
        return Err(ErrorDiagrama::new("no hay bloques para renderizar"));
    }
    if svgs.len() == 1 {
        let w = extraer_attr_int(&svgs[0], "width")?;
        let h = extraer_attr_int(&svgs[0], "height")?;
        if w <= 0 || h <= 0 || w > MAX_SVG_DIM_I64 || h > MAX_SVG_DIM_I64 {
            return Err(ErrorDiagrama::new(format!(
                "dimensiones de SVG inválidas ({w}×{h})"
            )));
        }
        return Ok(svgs[0].clone());
    }
    let mut ancho: i64 = 0;
    let mut y: i64 = 0;
    let mut partes = Vec::with_capacity(svgs.len());
    for svg in svgs {
        let w = extraer_attr_int(svg, "width")?;
        let h = extraer_attr_int(svg, "height")?;
        if w <= 0 || h <= 0 || w > MAX_SVG_DIM_I64 || h > MAX_SVG_DIM_I64 {
            return Err(ErrorDiagrama::new(format!(
                "dimensiones de SVG inválidas al apilar ({w}×{h})"
            )));
        }
        partes.push(svg.replacen("<svg ", &format!(r#"<svg x="0" y="{y}" "#), 1));
        ancho = ancho.max(w);
        y = y
            .checked_add(h)
            .and_then(|v| v.checked_add(8))
            .ok_or_else(|| ErrorDiagrama::new("desbordamiento de altura al apilar SVGs"))?;
    }
    let alto = y.checked_sub(8).unwrap_or(0).max(0);
    if alto > MAX_SVG_DIM_I64 {
        return Err(ErrorDiagrama::new("altura total apilada fuera de rango"));
    }
    Ok(format!(
        r#"<svg xmlns="http://www.w3.org/2000/svg" width="{ancho}" height="{alto}" viewBox="0 0 {ancho} {alto}">{}</svg>"#,
        partes.concat()
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nombre_svg_coincide_con_sha1_del_cuerpo_recortado() {
        let n1 = nombre_svg("type: array\nvalues: [1, 2]\n");
        let n2 = nombre_svg("  \ntype: array\nvalues: [1, 2]\n  ");
        assert_eq!(n1, n2);
        assert!(n1.starts_with("estructura-"));
        assert!(n1.ends_with(".svg"));
        assert_eq!(n1.len(), "estructura-".len() + 12 + ".svg".len());
    }

    #[test]
    fn leer_bloques_acepta_objeto_lista_y_blocks() {
        assert_eq!(leer_bloques("type: array\nvalues: [1]").unwrap().len(), 1);
        assert_eq!(
            leer_bloques("- type: array\n- type: stack").unwrap().len(),
            2
        );
        assert_eq!(
            leer_bloques("blocks:\n  - type: array\n  - type: tree")
                .unwrap()
                .len(),
            2
        );
    }

    #[test]
    fn escape_xml_basico() {
        assert_eq!(esc("a < b & c > \"d\""), "a &lt; b &amp; c &gt; &quot;d&quot;");
    }

    #[test]
    fn escape_xml_filtra_c0_y_nonchars() {
        let entrada = "a\x00\x01\x08\x0b\x0c\x0e\x1fb\u{fffe}c\u{ffff}d\t\n\re";
        assert_eq!(esc(entrada), "abcd\t\n\re");
    }

    #[test]
    fn leer_bloques_acepta_bom_utf8() {
        let bloques = leer_bloques("\u{feff}type: array\nvalues: [1, 2]\n").unwrap();
        assert_eq!(bloques.len(), 1);
        assert_eq!(bloques[0]["type"], "array");
    }

    #[test]
    fn renderizar_rechaza_dimensiones_invalidas_o_desbordantes() {
        let mut reg = Registro::new();
        reg.registrar("nan_w", |_, _| Ok((f64::NAN, 10.0)));
        reg.registrar("inf_h", |_, _| Ok((10.0, f64::INFINITY)));
        reg.registrar("neg_h", |_, _| Ok((100.0, -50.0)));
        reg.registrar("huge_h", |_, _| Ok((100.0, 1e30)));
        for t in ["nan_w", "inf_h", "neg_h", "huge_h"] {
            let mut b = Bloque::new();
            b.insert("type".into(), Value::String(t.into()));
            assert!(reg.renderizar(&b, true).is_err(), "{t} debía fallar");
        }
    }

    #[test]
    fn apilar_svgs_rechaza_overflow_i64_sin_panic() {
        let s1 = format!(r#"<svg width="100" height="{}"></svg>"#, i64::MAX);
        let s2 = r#"<svg width="100" height="20"></svg>"#.to_string();
        assert!(apilar_svgs(&[s1.clone(), s2]).is_err());
        assert!(apilar_svgs(&[s1]).is_err());
    }
}
