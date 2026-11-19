//! Primitivas de gráficos cartesianos x-y (lineales y logarítmicos), marcas, grillas y leyendas.
//! Compartido entre `dibu-control` y `dibu-chem`.

use dibu_core::{ancho_texto, esc, Lienzo, Punto, MONO, SANS};

pub const GUIONES: &[&str] = &["", "7 3", "2 3", "9 3 2 3", "4 2"];

/// Formatea un número corto con coma decimal (paridad con `_g` de `alucarD`).
pub fn fmt_g(x: Option<f64>, cifras: usize) -> String {
    let Some(x) = x else {
        return "—".to_string();
    };
    if !x.is_finite() {
        return "—".to_string();
    }
    if x == 0.0 {
        return "0".to_string();
    }
    let ax = x.abs();
    if (1e3..1e6).contains(&ax) && (x - x.round()).abs() < 1e-6 * ax + 1e-9 {
        return format!("{}", x.round() as i64);
    }
    let texto = format_general(x, cifras.max(1));
    if texto.contains('e') || texto.contains('E') {
        format_sci(x, 2).replace('.', ",")
    } else {
        texto.replace('.', ",")
    }
}

fn format_sci(x: f64, decimales: usize) -> String {
    let s = format!("{x:.decimales$e}");
    let Some((mantisa, exp_str)) = s.split_once('e') else {
        return s;
    };
    let exp: i32 = exp_str.parse().unwrap_or(0);
    if exp < 0 {
        format!("{mantisa}e-{:02}", -exp)
    } else {
        format!("{mantisa}e+{exp:02}")
    }
}

fn format_general(x: f64, sig: usize) -> String {
    let ax = x.abs();
    let exp = ax.log10().floor() as i32;
    if exp < -4 || exp >= sig as i32 {
        return format_sci(x, sig.saturating_sub(1));
    }
    let dec = (sig as i32 - 1 - exp).max(0) as usize;
    let mut s = format!("{x:.dec$}");
    if s.contains('.') {
        while s.ends_with('0') {
            s.pop();
        }
        if s.ends_with('.') {
            s.pop();
        }
    }
    if s == "-0" {
        "0".to_string()
    } else {
        s
    }
}

pub fn paso_lindo(rango: f64, marcas: usize) -> f64 {
    if rango <= 0.0 || !rango.is_finite() {
        return 1.0;
    }
    let bruto = rango / marcas.max(1) as f64;
    let base = 10_f64.powf(bruto.log10().floor());
    for m in [1.0, 2.0, 2.5, 5.0, 10.0] {
        if bruto <= m * base {
            return m * base;
        }
    }
    10.0 * base
}

pub fn marcas(lo: f64, hi: f64, n_marcas: usize, paso: Option<f64>) -> Vec<f64> {
    let p = paso.unwrap_or_else(|| paso_lindo(hi - lo, n_marcas));
    if p <= 0.0 || !p.is_finite() {
        return vec![lo, hi];
    }
    let inicio = (lo / p - 1e-9).ceil() * p;
    let mut salida = Vec::new();
    let mut v = inicio;
    let mut guard = 0;
    while v <= hi + p * 1e-6 && guard < 1000 {
        let r = (v * 1e12).round() / 1e12;
        salida.push(if r == 0.0 { 0.0 } else { r });
        v += p;
        guard += 1;
    }
    salida
}

#[derive(Debug, Clone, Copy)]
pub struct EjesConfig<'a> {
    pub x0: f64,
    pub y0: f64,
    pub w: f64,
    pub h: f64,
    pub xr: Punto,
    pub yr: Punto,
    pub xlabel: &'a str,
    pub ylabel: &'a str,
    pub log: bool,
    pub grilla: bool,
    pub origen: bool,
    pub ypaso: Option<f64>,
}

pub struct Ejes {
    pub x0: f64,
    pub y0: f64,
    pub w: f64,
    pub h: f64,
    pub xr: Punto,
    pub yr: Punto,
    pub log: bool,
}

impl Ejes {
    pub fn dibujar(c: &mut Lienzo, cfg: EjesConfig<'_>) -> Self {
        let ejes = Self {
            x0: cfg.x0,
            y0: cfg.y0,
            w: cfg.w,
            h: cfg.h,
            xr: cfg.xr,
            yr: cfg.yr,
            log: cfg.log,
        };
        c.add(format!(
            r#"<rect x="{:.1}" y="{:.1}" width="{:.1}" height="{:.1}" fill="none" stroke="{}" stroke-width="1.2"/>"#,
            cfg.x0, cfg.y0, cfg.w, cfg.h, c.pal.stroke_main
        ));
        if cfg.log {
            let dec0 = (cfg.xr.0.log10() + 1e-9).floor() as i32;
            let dec1 = (cfg.xr.1.log10() - 1e-9).ceil() as i32;
            for e in dec0..=dec1 {
                for m in 1..10 {
                    let v = (m as f64) * 10_f64.powi(e);
                    if !(cfg.xr.0 * 0.999..=cfg.xr.1 * 1.001).contains(&v) {
                        continue;
                    }
                    let x = ejes.px(v);
                    if cfg.grilla {
                        c.linea(
                            x,
                            cfg.y0,
                            x,
                            cfg.y0 + cfg.h,
                            "strokeMuted",
                            if m == 1 { 0.6 } else { 0.3 },
                            None,
                            if m == 1 { "" } else { "1 3" },
                        );
                    }
                    if m == 1 {
                        let lbl = if e.abs() > 3 {
                            format!("10^{e}")
                        } else {
                            fmt_g(Some(v), 3)
                        };
                        c.t(x, cfg.y0 + cfg.h + 14.0, &lbl, 9.0, 400, "textSub", "middle", MONO);
                    }
                }
            }
        } else {
            let (lo, hi) = if cfg.xr.0 <= cfg.xr.1 {
                (cfg.xr.0, cfg.xr.1)
            } else {
                (cfg.xr.1, cfg.xr.0)
            };
            for v in marcas(lo, hi, 5, None) {
                let x = ejes.px(v);
                if cfg.grilla {
                    c.linea(x, cfg.y0, x, cfg.y0 + cfg.h, "strokeMuted", 0.4, None, "2 4");
                }
                c.t(
                    x,
                    cfg.y0 + cfg.h + 14.0,
                    &fmt_g(Some(v), 3),
                    9.0,
                    400,
                    "textSub",
                    "middle",
                    MONO,
                );
            }
        }
        for v in marcas(cfg.yr.0, cfg.yr.1, 5, cfg.ypaso) {
            let y = ejes.py(v);
            if cfg.grilla {
                c.linea(cfg.x0, y, cfg.x0 + cfg.w, y, "strokeMuted", 0.4, None, "2 4");
            }
            c.t(
                cfg.x0 - 6.0,
                y + 3.0,
                &fmt_g(Some(v), 3),
                9.0,
                400,
                "textSub",
                "end",
                MONO,
            );
        }
        if cfg.origen {
            if cfg.yr.0 < 0.0 && 0.0 < cfg.yr.1 {
                let y = ejes.py(0.0);
                c.linea(cfg.x0, y, cfg.x0 + cfg.w, y, "strokeSub", 1.0, None, "");
            }
            if !cfg.log && cfg.xr.0 < 0.0 && 0.0 < cfg.xr.1 {
                let x = ejes.px(0.0);
                c.linea(x, cfg.y0, x, cfg.y0 + cfg.h, "strokeSub", 1.0, None, "");
            }
        }
        if !cfg.xlabel.is_empty() {
            c.t(
                cfg.x0 + cfg.w / 2.0,
                cfg.y0 + cfg.h + 30.0,
                cfg.xlabel,
                10.0,
                600,
                "textMain",
                "middle",
                SANS,
            );
        }
        if !cfg.ylabel.is_empty() {
            let lx = cfg.x0 - 38.0;
            let ly = cfg.y0 + cfg.h / 2.0;
            c.add(format!(
                r#"<text x="{lx:.1}" y="{ly:.1}" text-anchor="middle" font-family="{SANS}" font-size="10" font-weight="600" fill="{}" transform="rotate(-90 {lx:.1} {ly:.1})">{}</text>"#,
                c.pal.text_main,
                esc(cfg.ylabel)
            ));
        }
        ejes
    }

    pub fn px(&self, v: f64) -> f64 {
        if self.log {
            let a = self.xr.0.log10();
            let b = self.xr.1.log10();
            self.x0 + (v.max(1e-300).log10() - a) / (b - a) * self.w
        } else {
            self.x0 + (v - self.xr.0) / (self.xr.1 - self.xr.0) * self.w
        }
    }

    pub fn py(&self, v: f64) -> f64 {
        self.y0 + self.h - (v - self.yr.0) / (self.yr.1 - self.yr.0) * self.h
    }

    pub fn adentro(&self, x: f64, y: f64) -> bool {
        x >= self.x0 - 0.5 && x <= self.x0 + self.w + 0.5 && y >= self.y0 - 0.5 && y <= self.y0 + self.h + 0.5
    }

    fn recortar(&self, a: Punto, b: Punto) -> Option<(Punto, Punto)> {
        let (x0, y0, x1, y1) = (self.x0, self.y0, self.x0 + self.w, self.y0 + self.h);
        let (dx, dy) = (b.0 - a.0, b.1 - a.1);
        let (mut t0, mut t1) = (0.0_f64, 1.0_f64);
        for (p, q) in [(-dx, a.0 - x0), (dx, x1 - a.0), (-dy, a.1 - y0), (dy, y1 - a.1)] {
            if p == 0.0 {
                if q < 0.0 {
                    return None;
                }
                continue;
            }
            let t = q / p;
            if p < 0.0 {
                t0 = t0.max(t);
            } else {
                t1 = t1.min(t);
            }
            if t0 > t1 {
                return None;
            }
        }
        Some(((a.0 + t0 * dx, a.1 + t0 * dy), (a.0 + t1 * dx, a.1 + t1 * dy)))
    }

    pub fn curva(&self, c: &mut Lienzo, puntos: &[Punto], estilo: usize, ancho: f64, color: &str) {
        let mut tramos: Vec<Vec<Punto>> = Vec::new();
        let mut previo: Option<Punto> = None;
        for &(x, y) in puntos {
            if !(x.is_finite() && y.is_finite()) {
                previo = None;
                continue;
            }
            let actual = (self.px(x), self.py(y));
            if let Some(prev) = previo {
                if actual.0.is_finite() && actual.1.is_finite() {
                    if let Some((inicio, fin)) = self.recortar(prev, actual) {
                        let unir = tramos
                            .last()
                            .and_then(|t| t.last())
                            .is_some_and(|&(lx, ly)| ((lx - inicio.0).hypot(ly - inicio.1)) < 0.01);
                        if unir {
                            tramos.last_mut().unwrap().push(fin);
                        } else {
                            tramos.push(vec![inicio, fin]);
                        }
                    }
                }
            }
            previo = Some(actual);
        }
        let guiones = GUIONES[estilo % GUIONES.len()];
        let dash = if guiones.is_empty() {
            String::new()
        } else {
            format!(r#" stroke-dasharray="{guiones}""#)
        };
        for tramo in tramos {
            let camino: String = tramo
                .iter()
                .map(|(x, y)| format!("{x:.1},{y:.1}"))
                .collect::<Vec<_>>()
                .join(" ");
            c.add(format!(
                r#"<polyline points="{camino}" fill="none" stroke="{}" stroke-width="{}" stroke-linejoin="round"{dash}/>"#,
                c.pal.color(color),
                dibu_core::fmt_num_clean(ancho)
            ));
        }
    }

    pub fn guia(&self, c: &mut Lienzo, x1: f64, y1: f64, x2: f64, y2: f64, guiones: &str, color: &str) {
        c.linea(self.px(x1), self.py(y1), self.px(x2), self.py(y2), color, 1.0, None, guiones);
    }
}

#[allow(clippy::too_many_arguments)]
pub fn rotulo(
    c: &mut Lienzo,
    x: f64,
    y: f64,
    nombre: &str,
    valor: &str,
    blanco: bool,
    ancla: &str,
    size: f64,
) -> f64 {
    if blanco {
        let texto = format!("{nombre} =");
        let tw = ancho_texto(&texto, size);
        let ancho = tw + 44.0;
        let x0 = match ancla {
            "start" => x,
            "middle" => x - ancho / 2.0,
            _ => x - ancho,
        };
        c.t(x0, y, &texto, size, 700, "textMain", "start", MONO);
        c.vacio(x0 + tw + 4.0, x0 + ancho, y + 2.0);
        return ancho;
    }
    let texto = format!("{nombre} = {valor}");
    c.t(x, y, &texto, size, 700, "textMain", ancla, MONO);
    ancho_texto(&texto, size)
}

pub fn leyenda(c: &mut Lienzo, x: f64, y: f64, etiquetas: &[String]) -> f64 {
    for (i, etiqueta) in etiquetas.iter().enumerate() {
        let guiones = GUIONES[i % GUIONES.len()];
        let dash = if guiones.is_empty() {
            String::new()
        } else {
            format!(r#" stroke-dasharray="{guiones}""#)
        };
        let yi = y + (i as f64) * 16.0;
        c.add(format!(
            r#"<line x1="{x:.1}" y1="{yi:.1}" x2="{:.1}" y2="{yi:.1}" stroke="{}" stroke-width="1.8"{dash}/>"#,
            x + 26.0,
            c.pal.stroke_main
        ));
        c.t(x + 32.0, yi + 4.0, etiqueta, 10.0, 400, "textMain", "start", MONO);
    }
    etiquetas.len() as f64 * 16.0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fmt_g_formatea_con_coma_y_enteros_grandes() {
        assert_eq!(fmt_g(None, 3), "—");
        assert_eq!(fmt_g(Some(0.0), 3), "0");
        assert_eq!(fmt_g(Some(4000.0), 3), "4000");
        assert_eq!(fmt_g(Some(1.25), 3), "1,25");
    }
}
