//! Sistema de themes para `dibu`: paletas semánticas, tipografías y presets built-in.

use crate::{ErrorDiagrama, MONO, SANS};
use serde::Deserialize;
use serde_json::Value;
use std::borrow::Cow;

pub const PRESETS: &[&str] = &["ink", "color", "high-contrast", "dark"];

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Paleta {
    pub background: Cow<'static, str>,
    pub stroke_main: Cow<'static, str>,
    pub stroke_sub: Cow<'static, str>,
    pub stroke_muted: Cow<'static, str>,
    pub fill_header: Cow<'static, str>,
    pub fill_cell: Cow<'static, str>,
    pub fill_pointer_box: Cow<'static, str>,
    pub fill_badge: Cow<'static, str>,
    pub accent_primary: Cow<'static, str>,
    pub text_main: Cow<'static, str>,
    pub text_sub: Cow<'static, str>,
    pub text_muted: Cow<'static, str>,
    pub fill_highlight: Cow<'static, str>,
}

pub const PALETA_TINTA: Paleta = Paleta {
    background: Cow::Borrowed("#ffffff"),
    stroke_main: Cow::Borrowed("#000000"),
    stroke_sub: Cow::Borrowed("#334155"),
    stroke_muted: Cow::Borrowed("#64748b"),
    fill_header: Cow::Borrowed("#f8fafc"),
    fill_cell: Cow::Borrowed("#ffffff"),
    fill_pointer_box: Cow::Borrowed("#f1f5f9"),
    fill_badge: Cow::Borrowed("#ffffff"),
    accent_primary: Cow::Borrowed("#000000"),
    text_main: Cow::Borrowed("#000000"),
    text_sub: Cow::Borrowed("#334155"),
    text_muted: Cow::Borrowed("#475569"),
    fill_highlight: Cow::Borrowed("#e2e8f0"),
};

pub const PALETA_COLOR: Paleta = Paleta {
    background: Cow::Borrowed("#ffffff"),
    stroke_main: Cow::Borrowed("#0f172a"),
    stroke_sub: Cow::Borrowed("#1e40af"),
    stroke_muted: Cow::Borrowed("#64748b"),
    fill_header: Cow::Borrowed("#eff6ff"),
    fill_cell: Cow::Borrowed("#ffffff"),
    fill_pointer_box: Cow::Borrowed("#e0f2fe"),
    fill_badge: Cow::Borrowed("#dbeafe"),
    accent_primary: Cow::Borrowed("#1d4ed8"),
    text_main: Cow::Borrowed("#0f172a"),
    text_sub: Cow::Borrowed("#1e3a8a"),
    text_muted: Cow::Borrowed("#475569"),
    fill_highlight: Cow::Borrowed("#fef3c7"),
};

pub const PALETA_HIGH_CONTRAST: Paleta = Paleta {
    background: Cow::Borrowed("#ffffff"),
    stroke_main: Cow::Borrowed("#000000"),
    stroke_sub: Cow::Borrowed("#000000"),
    stroke_muted: Cow::Borrowed("#000000"),
    fill_header: Cow::Borrowed("#ffffff"),
    fill_cell: Cow::Borrowed("#ffffff"),
    fill_pointer_box: Cow::Borrowed("#ffffff"),
    fill_badge: Cow::Borrowed("#ffffff"),
    accent_primary: Cow::Borrowed("#000000"),
    text_main: Cow::Borrowed("#000000"),
    text_sub: Cow::Borrowed("#000000"),
    text_muted: Cow::Borrowed("#000000"),
    fill_highlight: Cow::Borrowed("#e2e8f0"),
};

pub const PALETA_DARK: Paleta = Paleta {
    background: Cow::Borrowed("#0f172a"),
    stroke_main: Cow::Borrowed("#e2e8f0"),
    stroke_sub: Cow::Borrowed("#93c5fd"),
    stroke_muted: Cow::Borrowed("#94a3b8"),
    fill_header: Cow::Borrowed("#1e293b"),
    fill_cell: Cow::Borrowed("#1e293b"),
    fill_pointer_box: Cow::Borrowed("#172554"),
    fill_badge: Cow::Borrowed("#1e3a8a"),
    accent_primary: Cow::Borrowed("#60a5fa"),
    text_main: Cow::Borrowed("#f8fafc"),
    text_sub: Cow::Borrowed("#bfdbfe"),
    text_muted: Cow::Borrowed("#cbd5e1"),
    fill_highlight: Cow::Borrowed("#422006"),
};

impl Paleta {
    pub fn para(tinta: bool) -> Self {
        if tinta {
            PALETA_TINTA.clone()
        } else {
            PALETA_COLOR.clone()
        }
    }

    pub fn color(&self, clave: &str) -> &str {
        match clave {
            "background" => &self.background,
            "strokeMain" => &self.stroke_main,
            "strokeSub" => &self.stroke_sub,
            "strokeMuted" => &self.stroke_muted,
            "fillHeader" => &self.fill_header,
            "fillCell" => &self.fill_cell,
            "fillPointerBox" => &self.fill_pointer_box,
            "fillBadge" => &self.fill_badge,
            "accentPrimary" => &self.accent_primary,
            "textMain" => &self.text_main,
            "textSub" => &self.text_sub,
            "textMuted" => &self.text_muted,
            "fillHighlight" => &self.fill_highlight,
            _ => &self.stroke_main,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Tipografia {
    pub sans: Cow<'static, str>,
    pub mono: Cow<'static, str>,
    pub scale: f64,
    pub char_width_factor: f64,
}

impl Default for Tipografia {
    fn default() -> Self {
        Self {
            sans: Cow::Borrowed(SANS),
            mono: Cow::Borrowed(MONO),
            scale: 1.0,
            char_width_factor: 0.62,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Theme {
    pub name: Cow<'static, str>,
    pub colors: Paleta,
    pub typography: Tipografia,
}

#[derive(Debug, Deserialize, Default)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct ColorsPatch {
    background: Option<String>,
    stroke_main: Option<String>,
    stroke_sub: Option<String>,
    stroke_muted: Option<String>,
    fill_header: Option<String>,
    fill_cell: Option<String>,
    fill_pointer_box: Option<String>,
    fill_badge: Option<String>,
    accent_primary: Option<String>,
    text_main: Option<String>,
    text_sub: Option<String>,
    text_muted: Option<String>,
    fill_highlight: Option<String>,
}

#[derive(Debug, Deserialize, Default)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct TypographyPatch {
    sans: Option<String>,
    mono: Option<String>,
    scale: Option<f64>,
    char_width_factor: Option<f64>,
}

#[derive(Debug, Deserialize, Default)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct ThemePatch {
    extends: Option<String>,
    colors: Option<ColorsPatch>,
    typography: Option<TypographyPatch>,
}

pub fn validar_color(valor: &str) -> Result<String, ErrorDiagrama> {
    let s = valor.trim();
    if s.eq_ignore_ascii_case("transparent") || s.eq_ignore_ascii_case("none") {
        return Ok(s.to_ascii_lowercase());
    }
    if let Some(hex) = s.strip_prefix('#') {
        if matches!(hex.len(), 3 | 4 | 6 | 8) && hex.bytes().all(|b| b.is_ascii_hexdigit()) {
            return Ok(s.to_ascii_lowercase());
        }
    }
    Err(ErrorDiagrama::new(format!(
        "theme: color inválido {valor:?} (se acepta #RGB, #RRGGBB, #RRGGBBAA, transparent o none)"
    )))
}

pub fn validar_fuente(valor: &str) -> Result<String, ErrorDiagrama> {
    let s = valor.trim();
    if s.is_empty() || s.len() > 256 {
        return Err(ErrorDiagrama::new(
            "theme: la familia tipográfica debe tener entre 1 y 256 bytes",
        ));
    }
    if s.chars().any(|c| c.is_control() || matches!(c, '"' | '<' | '>' | '&')) {
        return Err(ErrorDiagrama::new(format!(
            "theme: familia tipográfica con caracteres inválidos: {valor:?}"
        )));
    }
    Ok(s.to_string())
}

impl Theme {
    pub fn preset(nombre: &str) -> Result<Self, ErrorDiagrama> {
        match nombre.trim().to_ascii_lowercase().as_str() {
            "ink" | "tinta" => Ok(Self {
                name: Cow::Borrowed("ink"),
                colors: PALETA_TINTA.clone(),
                typography: Tipografia::default(),
            }),
            "color" => Ok(Self {
                name: Cow::Borrowed("color"),
                colors: PALETA_COLOR.clone(),
                typography: Tipografia::default(),
            }),
            "high-contrast" | "alto-contraste" | "high_contrast" => Ok(Self {
                name: Cow::Borrowed("high-contrast"),
                colors: PALETA_HIGH_CONTRAST.clone(),
                typography: Tipografia::default(),
            }),
            "dark" | "oscuro" => Ok(Self {
                name: Cow::Borrowed("dark"),
                colors: PALETA_DARK.clone(),
                typography: Tipografia::default(),
            }),
            otro => Err(ErrorDiagrama::new(format!(
                "theme desconocido: {otro:?} (presets disponibles: {})",
                PRESETS.join(", ")
            ))),
        }
    }

    pub fn para_tinta(tinta: bool) -> Self {
        if tinta {
            Self {
                name: Cow::Borrowed("ink"),
                colors: PALETA_TINTA.clone(),
                typography: Tipografia::default(),
            }
        } else {
            Self {
                name: Cow::Borrowed("color"),
                colors: PALETA_COLOR.clone(),
                typography: Tipografia::default(),
            }
        }
    }

    pub fn es_default_clasico(&self) -> bool {
        self.typography == Tipografia::default()
            && (self.colors == PALETA_TINTA || self.colors == PALETA_COLOR)
    }

    pub fn from_value(val: &Value, base: &Theme) -> Result<Self, ErrorDiagrama> {
        match val {
            Value::String(s) => Self::preset(s),
            Value::Object(_) => {
                let patch: ThemePatch = serde_json::from_value(val.clone())
                    .map_err(|e| ErrorDiagrama::new(format!("theme inválido: {e}")))?;
                Self::aplicar_patch(patch, base)
            }
            _ => Err(ErrorDiagrama::new(
                "theme debe ser el nombre de un preset o un objeto {extends?, colors?, typography?}",
            )),
        }
    }

    pub fn from_yaml_or_json_str(texto: &str) -> Result<Self, ErrorDiagrama> {
        let texto = texto.strip_prefix('\u{feff}').unwrap_or(texto);
        let patch: ThemePatch = serde_yaml::from_str(texto)
            .map_err(|e| ErrorDiagrama::new(format!("archivo de theme inválido: {e}")))?;
        let base = Self::para_tinta(true);
        Self::aplicar_patch(patch, &base)
    }

    fn aplicar_patch(patch: ThemePatch, base: &Theme) -> Result<Self, ErrorDiagrama> {
        let mut tema = match patch.extends.as_deref() {
            Some(nombre) => Self::preset(nombre)?,
            None => base.clone(),
        };
        tema.name = Cow::Borrowed("custom");

        if let Some(c) = patch.colors {
            if let Some(v) = c.background {
                tema.colors.background = Cow::Owned(validar_color(&v)?);
            }
            if let Some(v) = c.stroke_main {
                tema.colors.stroke_main = Cow::Owned(validar_color(&v)?);
            }
            if let Some(v) = c.stroke_sub {
                tema.colors.stroke_sub = Cow::Owned(validar_color(&v)?);
            }
            if let Some(v) = c.stroke_muted {
                tema.colors.stroke_muted = Cow::Owned(validar_color(&v)?);
            }
            if let Some(v) = c.fill_header {
                tema.colors.fill_header = Cow::Owned(validar_color(&v)?);
            }
            if let Some(v) = c.fill_cell {
                tema.colors.fill_cell = Cow::Owned(validar_color(&v)?);
            }
            if let Some(v) = c.fill_pointer_box {
                tema.colors.fill_pointer_box = Cow::Owned(validar_color(&v)?);
            }
            if let Some(v) = c.fill_badge {
                tema.colors.fill_badge = Cow::Owned(validar_color(&v)?);
            }
            if let Some(v) = c.accent_primary {
                tema.colors.accent_primary = Cow::Owned(validar_color(&v)?);
            }
            if let Some(v) = c.text_main {
                tema.colors.text_main = Cow::Owned(validar_color(&v)?);
            }
            if let Some(v) = c.text_sub {
                tema.colors.text_sub = Cow::Owned(validar_color(&v)?);
            }
            if let Some(v) = c.text_muted {
                tema.colors.text_muted = Cow::Owned(validar_color(&v)?);
            }
            if let Some(v) = c.fill_highlight {
                tema.colors.fill_highlight = Cow::Owned(validar_color(&v)?);
            }
        }

        if let Some(t) = patch.typography {
            if let Some(v) = t.sans {
                tema.typography.sans = Cow::Owned(validar_fuente(&v)?);
            }
            if let Some(v) = t.mono {
                tema.typography.mono = Cow::Owned(validar_fuente(&v)?);
            }
            if let Some(s) = t.scale {
                if !s.is_finite() || !(0.6..=1.8).contains(&s) {
                    return Err(ErrorDiagrama::new(
                        "theme: typography.scale debe estar entre 0.6 y 1.8",
                    ));
                }
                tema.typography.scale = s;
            }
            if let Some(f) = t.char_width_factor {
                if !f.is_finite() || !(0.4..=1.0).contains(&f) {
                    return Err(ErrorDiagrama::new(
                        "theme: typography.charWidthFactor debe estar entre 0.4 y 1.0",
                    ));
                }
                tema.typography.char_width_factor = f;
            }
        }

        Ok(tema)
    }

    pub fn huella(&self) -> String {
        format!(
            "{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{:.4}|{:.4}",
            self.colors.background,
            self.colors.stroke_main,
            self.colors.stroke_sub,
            self.colors.stroke_muted,
            self.colors.fill_header,
            self.colors.fill_cell,
            self.colors.fill_pointer_box,
            self.colors.fill_badge,
            self.colors.accent_primary,
            self.colors.text_main,
            self.colors.text_sub,
            self.colors.text_muted,
            self.colors.fill_highlight,
            self.typography.sans,
            self.typography.mono,
            self.typography.scale,
            self.typography.char_width_factor,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn presets_conocidos_y_alias() {
        for p in PRESETS {
            assert!(Theme::preset(p).is_ok(), "falló preset {p}");
        }
        assert_eq!(Theme::preset("tinta").unwrap(), Theme::preset("ink").unwrap());
        assert_eq!(
            Theme::preset("alto-contraste").unwrap(),
            Theme::preset("high-contrast").unwrap()
        );
        assert_eq!(
            Theme::preset("oscuro").unwrap(),
            Theme::preset("dark").unwrap()
        );
        assert!(Theme::preset("desconocido").is_err());
    }

    #[test]
    fn herencia_y_override_yaml() {
        let yaml = r##"
extends: dark
colors:
  background: "#111827"
  accentPrimary: "#38bdf8"
typography:
  sans: "Liberation Sans, Arial, sans-serif"
  mono: "JetBrains Mono, monospace"
  scale: 1.15
  charWidthFactor: 0.65
"##;
        let t = Theme::from_yaml_or_json_str(yaml).unwrap();
        assert_eq!(t.colors.background, "#111827");
        assert_eq!(t.colors.accent_primary, "#38bdf8");
        assert_eq!(t.colors.text_main, PALETA_DARK.text_main);
        assert_eq!(t.typography.sans, "Liberation Sans, Arial, sans-serif");
        assert_eq!(t.typography.mono, "JetBrains Mono, monospace");
        assert!((t.typography.scale - 1.15).abs() < 1e-9);
        assert!((t.typography.char_width_factor - 0.65).abs() < 1e-9);
    }

    #[test]
    fn rechaza_colores_y_fuentes_invalidos() {
        let base = Theme::para_tinta(true);
        let malo_color = json!({"colors": {"background": "\" onload=\"alert(1)"}});
        assert!(Theme::from_value(&malo_color, &base).is_err());

        let malo_fuente = json!({"typography": {"sans": "Arial\" onload=\"x"}});
        assert!(Theme::from_value(&malo_fuente, &base).is_err());

        let mala_escala = json!({"typography": {"scale": 5.0}});
        assert!(Theme::from_value(&mala_escala, &base).is_err());

        let campo_extra = json!({"unknownKey": true});
        assert!(Theme::from_value(&campo_extra, &base).is_err());
    }
}

