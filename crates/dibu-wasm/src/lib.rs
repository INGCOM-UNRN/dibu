//! El motor de dibu para WebAssembly, con todas las familias de diagramas registradas.
//!
//! Tiene dos partes:
//!
//! - Una API pura ([`render`], [`render_md`], [`parse`], [`stack`], [`leer_tema`]) que recibe
//!   texto y devuelve JSON. La usan los puentes de cada entorno, como el plugin de Typst.
//! - Con la feature `abi-c` (por omisión), el puente C-ABI sin wasm-bindgen que usan dibu-web
//!   y dibu-myst: JS reserva memoria con `dibu_alloc`, copia el texto UTF-8, llama a una
//!   operación (que devuelve la longitud de la respuesta) y lee la respuesta JSON desde
//!   `dibu_out_ptr()`. La memoria de entrada se libera dentro de cada operación.
//!
//! Se compila con `cargo build --profile wasm -p dibu-wasm --target wasm32-unknown-unknown`;
//! el módulo resultante no tiene imports.

#![warn(clippy::missing_safety_doc)]

pub use dibu_core::Theme;
use dibu_core::{apilar_svgs, leer_bloques, nombre_svg, Registro};
use serde_json::{json, Value};
use std::sync::OnceLock;

/// El registro con todas las familias de diagramas.
pub fn registro() -> &'static Registro {
    static REG: OnceLock<Registro> = OnceLock::new();
    REG.get_or_init(|| {
        let mut reg = Registro::new();
        dibu_cs::registrar(&mut reg);
        dibu_control::registrar(&mut reg);
        dibu_chem::registrar(&mut reg);
        dibu_comp::registrar(&mut reg);
        dibu_math::registrar(&mut reg);
        reg
    })
}

/// Los tipos registrados: `["array", "bode", ...]`.
pub fn tipos() -> Value {
    json!(registro().tipos())
}

/// Los nombres de los themes predefinidos.
pub fn themes() -> Value {
    json!(dibu_core::PRESETS)
}

/// Un theme a partir de su descripción JSON: `null` (tinta), un nombre (`"dark"`) o
/// `{extends, colors, typography}`.
pub fn leer_tema(v: Option<&Value>) -> Result<Theme, String> {
    match v {
        None | Some(Value::Null) => Ok(Theme::para_tinta(true)),
        Some(t) => Theme::from_value(t, &Theme::para_tinta(true)).map_err(|e| e.to_string()),
    }
}

/// Lee un sobre JSON `{ "text": "…", "theme": … }`.
pub fn leer_sobre(crudo: &str) -> Result<(String, Theme), String> {
    let v: Value = serde_json::from_str(crudo).map_err(|e| format!("sobre JSON inválido: {e}"))?;
    let texto = v.get("text").and_then(Value::as_str).unwrap_or("").to_string();
    Ok((texto, leer_tema(v.get("theme"))?))
}

/// Renderiza un texto YAML/JSON con uno o varios bloques, cada uno por separado.
/// Respuesta: `{"blocks":[{"type","id","title","svg"}|{"type","id","title","error"}]}` o `{"error"}`.
pub fn render(texto: &str, tema: &Theme) -> Value {
    let bloques = match leer_bloques(texto) {
        Ok(b) => b,
        Err(e) => return json!({ "error": e.to_string() }),
    };
    let reg = registro();
    let salida: Vec<Value> = bloques
        .iter()
        .map(|b| {
            let tipo = b.get("type").and_then(Value::as_str).unwrap_or("");
            let id = b.get("id").and_then(Value::as_str).unwrap_or("");
            let titulo = b.get("title").and_then(Value::as_str).unwrap_or("");
            match reg.renderizar_con_tema(b, tema) {
                Ok(svg) => json!({ "type": tipo, "id": id, "title": titulo, "svg": svg }),
                Err(e) => json!({ "type": tipo, "id": id, "title": titulo, "error": e.to_string() }),
            }
        })
        .collect();
    json!({ "blocks": salida })
}

/// Renderiza el cuerpo de un fence ```estructura``` tal como lo hace `dibu sync-md`
/// (bloques apilados) y devuelve además el nombre de archivo `estructura-<hash>.svg`.
/// Respuesta: `{"name","types","titles","svg"}` o `{"name","types","titles","error"}`.
pub fn render_md(cuerpo: &str, tema: &Theme) -> Value {
    let nombre = nombre_svg(cuerpo);
    let bloques = leer_bloques(cuerpo).unwrap_or_default();
    let campo = |k: &str| -> Vec<String> {
        bloques
            .iter()
            .map(|b| b.get(k).and_then(Value::as_str).unwrap_or("").to_string())
            .collect()
    };
    let (tipos, titulos) = (campo("type"), campo("title"));
    match registro().renderizar_bloque_markdown_con_tema(cuerpo, tema) {
        Ok(svg) => json!({ "name": nombre, "types": tipos, "titles": titulos, "svg": svg }),
        Err(e) => json!({ "name": nombre, "types": tipos, "titles": titulos, "error": e.to_string() }),
    }
}

/// Apila verticalmente una lista JSON de SVGs (`["<svg…>", …]`). Respuesta `{"svg"}` o `{"error"}`.
pub fn stack(lista_json: &str) -> Value {
    let svgs: Vec<String> = match serde_json::from_str(lista_json) {
        Ok(v) => v,
        Err(e) => return json!({ "error": format!("JSON inválido: {e}") }),
    };
    match apilar_svgs(&svgs) {
        Ok(svg) => json!({ "svg": svg }),
        Err(e) => json!({ "error": e.to_string() }),
    }
}

/// Parsea YAML/JSON y devuelve la raíz tal cual, `{"root": …}`, con el orden de claves
/// original (el editor visual lo necesita), o `{"error": …}`.
pub fn parse(texto: &str) -> String {
    let texto = texto.strip_prefix('\u{feff}').unwrap_or(texto);
    match serde_yaml::from_str::<serde_yaml::Value>(texto) {
        Ok(v) => {
            let mut s = String::from("{\"root\":");
            yaml_a_json(&v, &mut s);
            s.push('}');
            s
        }
        Err(e) => json!({ "error": format!("el bloque no es YAML/JSON válido: {e}") }).to_string(),
    }
}

/// Escribe un `serde_yaml::Value` como JSON conservando el orden de las claves
/// (serde_json::Map de dibu las ordena alfabéticamente; el editor visual necesita el original).
fn yaml_a_json(v: &serde_yaml::Value, out: &mut String) {
    use serde_yaml::Value as Y;
    match v {
        Y::Null => out.push_str("null"),
        Y::Bool(b) => out.push_str(if *b { "true" } else { "false" }),
        Y::Number(n) => {
            if let Some(i) = n.as_i64() {
                out.push_str(&i.to_string());
            } else if let Some(u) = n.as_u64() {
                out.push_str(&u.to_string());
            } else {
                match n.as_f64() {
                    Some(f) if f.is_finite() => out.push_str(&serde_json::to_string(&f).unwrap_or_else(|_| "null".into())),
                    _ => out.push_str("null"),
                }
            }
        }
        Y::String(s) => out.push_str(&serde_json::to_string(s).unwrap_or_default()),
        Y::Sequence(seq) => {
            out.push('[');
            for (i, x) in seq.iter().enumerate() {
                if i > 0 {
                    out.push(',');
                }
                yaml_a_json(x, out);
            }
            out.push(']');
        }
        Y::Mapping(m) => {
            out.push('{');
            for (i, (k, x)) in m.iter().enumerate() {
                if i > 0 {
                    out.push(',');
                }
                let clave = match k {
                    Y::String(s) => s.clone(),
                    otro => serde_yaml::to_string(otro).unwrap_or_default().trim().to_string(),
                };
                out.push_str(&serde_json::to_string(&clave).unwrap_or_default());
                out.push(':');
                yaml_a_json(x, out);
            }
            out.push('}');
        }
        Y::Tagged(t) => yaml_a_json(&t.value, out),
    }
}

#[cfg(feature = "abi-c")]
mod abi {
    //! El puente C-ABI: ver la documentación del crate.
    use super::*;
    use std::cell::RefCell;

    thread_local! {
        static SALIDA: RefCell<Vec<u8>> = const { RefCell::new(Vec::new()) };
    }

    fn guardar(bytes: Vec<u8>) -> usize {
        SALIDA.with(|s| {
            let mut s = s.borrow_mut();
            *s = bytes;
            s.len()
        })
    }

    fn responder(v: Value) -> usize {
        guardar(serde_json::to_vec(&v).unwrap_or_else(|_| b"{\"error\":\"serializacion\"}".to_vec()))
    }

    /// Toma posesión del buffer reservado con `dibu_alloc` y lo convierte en String.
    unsafe fn tomar_texto(ptr: *mut u8, len: usize) -> Result<String, String> {
        if ptr.is_null() || len == 0 {
            return Ok(String::new());
        }
        let caja: Box<[u8]> = Box::from_raw(std::ptr::slice_from_raw_parts_mut(ptr, len));
        String::from_utf8(caja.into_vec()).map_err(|e| format!("UTF-8 inválido: {e}"))
    }

    /// Aplica `f` al texto de entrada, o responde el error de lectura.
    unsafe fn con_texto(ptr: *mut u8, len: usize, f: impl FnOnce(&str) -> Value) -> usize {
        match tomar_texto(ptr, len) {
            Ok(t) => responder(f(&t)),
            Err(e) => responder(json!({ "error": e })),
        }
    }

    #[no_mangle]
    pub extern "C" fn dibu_alloc(len: usize) -> *mut u8 {
        if len == 0 {
            return std::ptr::NonNull::<u8>::dangling().as_ptr();
        }
        Box::into_raw(vec![0u8; len].into_boxed_slice()) as *mut u8
    }

    #[no_mangle]
    pub extern "C" fn dibu_out_ptr() -> *const u8 {
        SALIDA.with(|s| s.borrow().as_ptr())
    }

    #[no_mangle]
    pub extern "C" fn dibu_types() -> usize {
        responder(tipos())
    }

    #[no_mangle]
    pub extern "C" fn dibu_themes() -> usize {
        responder(themes())
    }

    /// # Safety
    /// `ptr` debe venir de `dibu_alloc(len)`.
    #[no_mangle]
    pub unsafe extern "C" fn dibu_render(ptr: *mut u8, len: usize, tinta: u32) -> usize {
        con_texto(ptr, len, |t| render(t, &Theme::para_tinta(tinta != 0)))
    }

    /// Como `dibu_render`, con un sobre JSON `{ "text", "theme" }`.
    ///
    /// # Safety
    /// `ptr` debe venir de `dibu_alloc(len)`.
    #[no_mangle]
    pub unsafe extern "C" fn dibu_render_theme(ptr: *mut u8, len: usize) -> usize {
        con_texto(ptr, len, |s| match leer_sobre(s) {
            Ok((t, tema)) => render(&t, &tema),
            Err(e) => json!({ "error": e }),
        })
    }

    /// # Safety
    /// `ptr` debe venir de `dibu_alloc(len)`.
    #[no_mangle]
    pub unsafe extern "C" fn dibu_render_md(ptr: *mut u8, len: usize, tinta: u32) -> usize {
        con_texto(ptr, len, |t| render_md(t, &Theme::para_tinta(tinta != 0)))
    }

    /// Como `dibu_render_md`, con un sobre JSON `{ "text", "theme" }`.
    ///
    /// # Safety
    /// `ptr` debe venir de `dibu_alloc(len)`.
    #[no_mangle]
    pub unsafe extern "C" fn dibu_render_md_theme(ptr: *mut u8, len: usize) -> usize {
        con_texto(ptr, len, |s| match leer_sobre(s) {
            Ok((t, tema)) => render_md(&t, &tema),
            Err(e) => json!({ "error": e }),
        })
    }

    /// # Safety
    /// `ptr` debe venir de `dibu_alloc(len)`.
    #[no_mangle]
    pub unsafe extern "C" fn dibu_stack(ptr: *mut u8, len: usize) -> usize {
        con_texto(ptr, len, stack)
    }

    /// # Safety
    /// `ptr` debe venir de `dibu_alloc(len)`.
    #[no_mangle]
    pub unsafe extern "C" fn dibu_parse(ptr: *mut u8, len: usize) -> usize {
        match tomar_texto(ptr, len) {
            Ok(t) => guardar(parse(&t).into_bytes()),
            Err(e) => responder(json!({ "error": e })),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn render_y_errores() {
        let tema = Theme::para_tinta(true);
        let v = render("type: stack\nvalues: [1, 2]\n", &tema);
        assert!(v["blocks"][0]["svg"].as_str().unwrap().starts_with("<svg"));
        let v = render("type: no_existe\n", &tema);
        assert!(v["blocks"][0]["error"].is_string());
        let v = render_md("type: stack\nvalues: [1]\n", &tema);
        assert!(v["name"].as_str().unwrap().starts_with("estructura-"));
    }

    #[test]
    fn sobre_y_parse() {
        let (t, _) = leer_sobre(r#"{"text":"type: stack","theme":"dark"}"#).unwrap();
        assert_eq!(t, "type: stack");
        assert!(leer_sobre(r#"{"text":"","theme":"inexistente"}"#).is_err());
        assert_eq!(parse("b: 1\na: 2"), r#"{"root":{"b":1,"a":2}}"#);
        assert!(tipos().as_array().unwrap().len() >= 73);
    }
}
