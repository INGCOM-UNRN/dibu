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
    for ch in texto.chars() {
        match ch {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&#x27;"),
            '\x00'..='\x08' | '\x0b' | '\x0c' | '\x0e'..='\x1f' => {}
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
    texto.chars().count() as f64 * size * escala * factor
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
        self.add(format!(
            r#"<text x="{x:.1}" y="{y:.1}" text-anchor="{ancla}" font-family="{fam}" font-size="{}" font-weight="{peso}" fill="{fill}">{}</text>"#,
            fmt_num_clean(sz),
            esc(texto)
        ));
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
    let json_val: Value = serde_json::to_value(yaml_val)
        .map_err(|e| ErrorDiagrama::new(format!("el bloque no se pudo normalizar a JSON: {e}")))?;
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
        let sufijo = if tema.es_default_clasico() {
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
        Ok(svg)
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
