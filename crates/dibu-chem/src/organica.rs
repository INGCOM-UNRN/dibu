//! Renderizadores de química orgánica (M1c): `newman`, `chair`, `fischer`.

use crate::chem::{self, SustituyenteSilla};
use crate::common::{
    ancho_formula, booleano, caja_vacia, encabezado_q, entero, envolver_error, num, opt_lista,
    opt_texto, rotulo_q, tq,
};
use dibu_core::{valor_str, Bloque, ErrorDiagrama, Lienzo, Punto, Registro, SANS};
use serde_json::Value;
use std::fmt::Write as _;

pub fn registrar(reg: &mut Registro) {
    reg.registrar("newman", render_newman);
    reg.registrar("chair", render_chair);
    reg.registrar("fischer", render_fischer);
}

fn mal_formado(tipo: &str, err: ErrorDiagrama) -> ErrorDiagrama {
    ErrorDiagrama::new(format!("bloque {tipo} mal formado: {err}"))
}

fn info(c: &mut Lienzo, x: f64, mut y: f64, filas: &[(&str, &str, bool)]) -> f64 {
    for &(nombre, valor, blanco) in filas {
        let etiqueta = format!("{nombre}:");
        c.t(x, y, &etiqueta, 10.5, 700, "textMain", "start", SANS);
        let x_valor = x + ancho_formula(&etiqueta, 10.5) * 1.08 + 8.0;
        if blanco {
            c.vacio(x_valor, x_valor + 120.0, y + 2.0);
        } else {
            tq(
                c, x_valor, y, valor, 10.5, 400, "textMain", "start", SANS, false,
            );
        }
        y += 20.0;
    }
    y
}

// --- Newman ---------------------------------------------------------------------------------------------

fn render_newman(c: &mut Lienzo, b: &Bloque) -> Result<Punto, ErrorDiagrama> {
    let adelante_arr = opt_lista(b, "front", "front").map_err(|e| mal_formado("newman", e))?;
    let atras_arr = opt_lista(b, "back", "back").map_err(|e| mal_formado("newman", e))?;
    let adelante: Vec<String> = adelante_arr
        .map(|a| a.iter().map(valor_str).collect())
        .unwrap_or_default();
    let atras: Vec<String> = atras_arr
        .map(|a| a.iter().map(valor_str).collect())
        .unwrap_or_default();
    if adelante.len() != 3 || atras.len() != 3 {
        return Err(ErrorDiagrama::new(
            "newman: front y back son los tres sustituyentes de cada carbono",
        ));
    }

    let diedro = match b.get("dihedral") {
        None => 180.0,
        Some(v) => num(v, "dihedral").map_err(|e| mal_formado("newman", e))?,
    };
    let inicio = match b.get("start") {
        None => 90.0,
        Some(v) => num(v, "start").map_err(|e| mal_formado("newman", e))?,
    };

    let top = encabezado_q(c, b);
    let cx = 120.0_f64;
    let cy = top + 110.0;
    let r = 30.0_f64;
    let largo = 70.0_f64;
    let trazo = c.pal.stroke_main;

    let rem = diedro.rem_euclid(120.0);
    let correccion = if rem < 1e-9 || (120.0 - rem) < 1e-9 {
        16.0
    } else {
        0.0
    };
    let inicio_ang = if inicio.abs() < 1e12 {
        inicio
    } else {
        inicio.rem_euclid(360.0)
    };
    let diedro_ang = if diedro.abs() < 1e12 {
        diedro
    } else {
        diedro.rem_euclid(360.0)
    };

    for (k, etiqueta) in atras.iter().enumerate() {
        let ang = (inicio_ang + diedro_ang + correccion + 120.0 * k as f64).to_radians();
        let x1 = cx + r * ang.cos();
        let y1 = cy - r * ang.sin();
        let x2 = cx + largo * ang.cos();
        let y2 = cy - largo * ang.sin();
        c.add(format!(
            r#"<line x1="{x1:.1}" y1="{y1:.1}" x2="{x2:.1}" y2="{y2:.1}" stroke="{trazo}" stroke-width="1.6"/>"#
        ));
        let afuera = largo + if correccion != 0.0 { 24.0 } else { 14.0 };
        tq(
            c,
            cx + afuera * ang.cos(),
            cy - afuera * ang.sin() + 4.0,
            etiqueta,
            11.0,
            400,
            "textMain",
            "middle",
            SANS,
            false,
        );
    }

    c.add(format!(
        r#"<circle cx="{cx:.1}" cy="{cy:.1}" r="{r:.1}" fill="{}" stroke="{trazo}" stroke-width="1.8"/>"#,
        c.pal.fill_cell
    ));

    for (k, etiqueta) in adelante.iter().enumerate() {
        let ang = (inicio_ang + 120.0 * k as f64).to_radians();
        let x2 = cx + largo * ang.cos();
        let y2 = cy - largo * ang.sin();
        c.add(format!(
            r#"<line x1="{cx:.1}" y1="{cy:.1}" x2="{x2:.1}" y2="{y2:.1}" stroke="{trazo}" stroke-width="1.8"/>"#
        ));
        tq(
            c,
            cx + (largo + 14.0) * ang.cos(),
            cy - (largo + 14.0) * ang.sin() + 4.0,
            etiqueta,
            11.0,
            600,
            "textMain",
            "middle",
            SANS,
            false,
        );
    }

    c.punto(cx, cy, "strokeMain", 2.2);

    let nombre = chem::conformacion_newman(diedro);
    let blanco = booleano(b, "blank", false);
    let diedro_norm = ((diedro + 180.0).rem_euclid(360.0) - 180.0).abs();
    let diedro_txt = format!("{}°", chem::fmt_g_py(diedro_norm));

    let mut y = top + 70.0;
    if let Some(eje) = opt_texto(b, "axis") {
        tq(
            c,
            270.0,
            y - 26.0,
            &format!("Mirando a lo largo de {eje}"),
            10.0,
            400,
            "textSub",
            "start",
            SANS,
            false,
        );
    }
    let filas = [
        ("Diedro", diedro_txt.as_str(), blanco || booleano(b, "blankDihedral", false)),
        ("Conformación", nombre.as_str(), blanco || booleano(b, "blankName", false)),
    ];
    y = info(c, 270.0, y, &filas);
    Ok((500.0, (top + 210.0).max(y)))
}

// --- Silla del ciclohexano ------------------------------------------------------------------------------

const RADIO_SILLA: f64 = 1.0;
const ALTURA_SILLA: f64 = 0.36;
const ESCALA_SILLA: f64 = 58.0;

fn elevacion_silla() -> f64 {
    20.0_f64.to_radians()
}

fn proyectar_silla(x: f64, y: f64, z: f64) -> Punto {
    let elev = elevacion_silla();
    (
        x * ESCALA_SILLA,
        (-z * elev.cos() - y * elev.sin()) * ESCALA_SILLA,
    )
}

fn carbono_silla(i: usize, invertida: bool) -> (f64, f64, f64) {
    let theta = (60.0 * (i as f64 - 1.0)).to_radians();
    let signo = if (i % 2 == 1) != invertida { 1.0 } else { -1.0 };
    (
        RADIO_SILLA * theta.cos(),
        RADIO_SILLA * theta.sin(),
        signo * ALTURA_SILLA,
    )
}

fn dibujar_silla(
    c: &mut Lienzo,
    x0: f64,
    y0: f64,
    sustituyentes: &[SustituyenteSilla],
    invertida: bool,
) {
    let trazo = c.pal.stroke_main;
    let tres_d: Vec<(f64, f64, f64)> = (1..=6).map(|i| carbono_silla(i, invertida)).collect();
    let puntos: Vec<Punto> = tres_d
        .iter()
        .map(|&(x, y, z)| {
            let (px, py) = proyectar_silla(x, y, z);
            (x0 + px, y0 + py)
        })
        .collect();

    let mut pts_str = String::new();
    for (idx, &(x, y)) in puntos.iter().enumerate() {
        if idx > 0 {
            pts_str.push(' ');
        }
        let _ = write!(&mut pts_str, "{x:.1},{y:.1}");
    }
    c.add(format!(
        r#"<polygon points="{pts_str}" fill="none" stroke="{trazo}" stroke-width="1.7" stroke-linejoin="round"/>"#
    ));

    for (idx, (&(x, y), &(_, _, z))) in puntos.iter().zip(tres_d.iter()).enumerate() {
        let i = idx + 1;
        c.t(
            x + 9.0,
            y + if z < 0.0 { 12.0 } else { -5.0 },
            &i.to_string(),
            8.0,
            400,
            "textMuted",
            "middle",
            SANS,
        );
    }

    for s in sustituyentes {
        let idx = usize::from(s.position).saturating_sub(1);
        let (Some(&(cx3, cy3, cz3)), Some(&(x, y))) = (tres_d.get(idx), puntos.get(idx)) else {
            continue;
        };
        let signo = if cz3 > 0.0 { 1.0 } else { -1.0 };
        let vector = if s.axial {
            (0.0, 0.0, signo * 0.62)
        } else {
            let norma = cx3.hypot(cy3);
            let n = if norma == 0.0 { 1.0 } else { norma };
            (cx3 / n * 0.62, cy3 / n * 0.62, -signo * 0.2)
        };
        let fin = proyectar_silla(cx3 + vector.0, cy3 + vector.1, cz3 + vector.2);
        let dx = fin.0 - (x - x0);
        let dy = fin.1 - (y - y0);
        c.add(format!(
            r#"<line x1="{x:.1}" y1="{y:.1}" x2="{:.1}" y2="{:.1}" stroke="{trazo}" stroke-width="1.4"/>"#,
            x + dx,
            y + dy
        ));
        let norma = {
            let h = dx.hypot(dy);
            if h == 0.0 { 1.0 } else { h }
        };
        let ex = x + dx + dx / norma * 12.0;
        let ey = y + dy + dy / norma * 10.0;
        if s.blank {
            caja_vacia(c, ex, ey, 26.0, 14.0);
        } else {
            tq(
                c,
                ex,
                ey + 4.0,
                &s.group,
                10.0,
                400,
                "textMain",
                "middle",
                SANS,
                false,
            );
        }
    }
}

fn leer_sustituyentes(b: &Bloque) -> Result<Vec<SustituyenteSilla>, ErrorDiagrama> {
    let arr = opt_lista(b, "substituents", "substituents").map_err(|e| mal_formado("chair", e))?;
    let mut salida = Vec::new();
    for (i, item) in arr.map(Vec::as_slice).unwrap_or(&[]).iter().enumerate() {
        let Some(obj) = item.as_object() else {
            return Err(ErrorDiagrama::new(format!(
                "chair: substituents[{i}] es {{position: 1-6, group, axial}}"
            )));
        };
        let (Some(pos_val), Some(group_val)) = (obj.get("position"), obj.get("group")) else {
            return Err(ErrorDiagrama::new(format!(
                "chair: substituents[{i}] es {{position: 1-6, group, axial}}"
            )));
        };
        let pos = entero(pos_val, &format!("substituents[{i}].position"))
            .map_err(|e| mal_formado("chair", e))?;
        if !(1..=6).contains(&pos) {
            return Err(ErrorDiagrama::new(format!(
                "chair: substituents[{i}].position va de 1 a 6"
            )));
        }
        salida.push(SustituyenteSilla {
            position: pos as u8,
            group: valor_str(group_val),
            axial: booleano(obj, "axial", false),
            blank: booleano(obj, "blank", false),
        });
    }
    Ok(salida)
}

fn render_chair(c: &mut Lienzo, b: &Bloque) -> Result<Punto, ErrorDiagrama> {
    let sust = leer_sustituyentes(b)?;
    let top = encabezado_q(c, b) + 70.0;
    dibujar_silla(c, 110.0, top, &sust, false);
    let mut ancho = 230.0_f64;
    let invertidos: Vec<SustituyenteSilla> = sust
        .iter()
        .map(|s| SustituyenteSilla {
            position: s.position,
            group: s.group.clone(),
            axial: !s.axial,
            blank: s.blank,
        })
        .collect();

    let flip = booleano(b, "flip", false);
    if flip {
        let acento = c.pal.accent_primary;
        c.add(format!(
            r#"<path d="M 236 {:.1} L 276 {:.1} L 269 {:.1}" fill="none" stroke="{acento}" stroke-width="1.5"/><path d="M 276 {:.1} L 236 {:.1} L 243 {:.1}" fill="none" stroke="{acento}" stroke-width="1.5"/>"#,
            top - 3.0,
            top - 3.0,
            top - 8.0,
            top + 3.0,
            top + 3.0,
            top + 8.0
        ));
        dibujar_silla(c, 400.0, top, &invertidos, true);
        ancho = 510.0;
    }

    let blanco = booleano(b, "blank", false);
    let show_relation = booleano(b, "showRelation", true);
    let show_stability = booleano(b, "showStability", true);
    let mut filas: Vec<(&str, String, bool)> = Vec::new();

    if show_relation {
        if let Some(rel) = chem::relacion_cis_trans(&sust) {
            let nombres: Vec<&SustituyenteSilla> =
                sust.iter().filter(|s| s.group != "H").collect();
            if let (Some(n0), Some(n1)) = (nombres.first(), nombres.get(1)) {
                filas.push((
                    "Relación",
                    format!("{rel} ({},{})", n0.position, n1.position),
                    blanco || booleano(b, "blankRelation", false),
                ));
            }
        }
    }

    if flip
        && show_stability
        && sust.iter().all(|s| chem::valor_a(&s.group).is_some())
    {
        let a1 = envolver_error("chair", chem::costo_axial(&sust))?;
        let a2 = envolver_error("chair", chem::costo_axial(&invertidos))?;
        let texto = if (a1 - a2).abs() < 1e-9 {
            "las dos sillas tienen la misma energía".to_string()
        } else {
            let lado = if a1 < a2 { "izquierda" } else { "derecha" };
            let diff = ((a1 - a2).abs() * 10.0).round() / 10.0;
            format!("la {lado}, por {} kJ/mol", chem::corto(Some(diff)))
        };
        filas.push((
            "Más estable",
            texto,
            blanco || booleano(b, "blankStability", false),
        ));
    }

    let filas_ref: Vec<(&str, &str, bool)> = filas
        .iter()
        .map(|(n, v, bl)| (*n, v.as_str(), *bl))
        .collect();
    let y = info(c, 0.0, top + 80.0, &filas_ref);
    Ok((ancho, y))
}

// --- Fischer --------------------------------------------------------------------------------------------

const LUGARES: [&str; 4] = ["top", "right", "bottom", "left"];

fn render_fischer(c: &mut Lienzo, b: &Bloque) -> Result<Punto, ErrorDiagrama> {
    let centros = match b.get("centers") {
        Some(Value::Array(arr)) if !arr.is_empty() => arr,
        _ => {
            return Err(ErrorDiagrama::new(
                "fischer: centers es una lista de {left, right} (de arriba hacia abajo)",
            ));
        }
    };

    let arriba = b.get("top").map(valor_str).unwrap_or_default();
    let abajo = b.get("bottom").map(valor_str).unwrap_or_default();
    let top = encabezado_q(c, b) + 20.0;
    let cx = 120.0_f64;
    let paso = 52.0_f64;
    let brazo = 46.0_f64;
    let trazo = c.pal.stroke_main;

    let mut y = top + 14.0;
    tq(
        c, cx, y, &arriba, 11.5, 600, "textMain", "middle", SANS, false,
    );
    y += 8.0;

    let blanco = booleano(b, "blank", false);
    let blank_config = booleano(b, "blankConfig", false);
    let show_config = booleano(b, "showConfig", true);

    for (i, centro_val) in centros.iter().enumerate() {
        let Some(centro) = centro_val.as_object() else {
            return Err(ErrorDiagrama::new(format!(
                "fischer: centers[{i}] es {{left, right}}"
            )));
        };
        let (Some(left_val), Some(right_val)) = (centro.get("left"), centro.get("right")) else {
            return Err(ErrorDiagrama::new(format!(
                "fischer: centers[{i}] es {{left, right}}"
            )));
        };

        let yc = y + paso * (i as f64 + 0.5) + 6.0;
        c.add(format!(
            r#"<line x1="{cx:.1}" y1="{:.1}" x2="{cx:.1}" y2="{:.1}" stroke="{trazo}" stroke-width="1.6"/><line x1="{:.1}" y1="{yc:.1}" x2="{:.1}" y2="{yc:.1}" stroke="{trazo}" stroke-width="1.6"/>"#,
            yc - paso / 2.0,
            yc + paso / 2.0,
            cx - brazo,
            cx + brazo
        ));

        let es_blank_bool = matches!(centro.get("blank"), Some(Value::Bool(true)));
        let blank_str = centro.get("blank").and_then(Value::as_str);
        let left_str = valor_str(left_val);
        let right_str = valor_str(right_val);

        for (lado, signo, texto) in [("left", -1.0_f64, &left_str), ("right", 1.0_f64, &right_str)] {
            if es_blank_bool || blank_str == Some(lado) {
                caja_vacia(c, cx + signo * (brazo + 18.0), yc, 30.0, 15.0);
            } else {
                tq(
                    c,
                    cx + signo * (brazo + 4.0),
                    yc + 4.0,
                    texto,
                    11.0,
                    400,
                    "textMain",
                    if signo > 0.0 { "start" } else { "end" },
                    SANS,
                    false,
                );
            }
        }

        let centro_blank = booleano(centro, "blank", false);
        let config: Option<&'static str> = if centro_blank {
            None
        } else if let Some(prio_val) = centro.get("priorities") {
            let arr = prio_val.as_array().ok_or_else(|| {
                mal_formado(
                    "fischer",
                    ErrorDiagrama::new(format!("centers[{i}].priorities tiene que ser una lista")),
                )
            })?;
            let lugares: Vec<String> = arr.iter().map(valor_str).collect();
            let mut ordenados = lugares.clone();
            ordenados.sort();
            if ordenados.as_slice() != ["bottom", "left", "right", "top"] {
                return Err(ErrorDiagrama::new(format!(
                    "fischer: centers[{i}].priorities son top, right, bottom y left, en orden"
                )));
            }
            let pares: Vec<(&str, usize)> = lugares
                .iter()
                .enumerate()
                .map(|(k, l)| (l.as_str(), k + 1))
                .collect();
            Some(envolver_error(
                "fischer",
                chem::configuracion_fischer(&pares),
            )?)
        } else if centros.len() == 1 {
            let grupos = [
                arriba.as_str(),
                right_str.as_str(),
                abajo.as_str(),
                left_str.as_str(),
            ];
            let rango = envolver_error("fischer", chem::prioridades(&grupos))?;
            if rango.len() != 4 {
                return Err(ErrorDiagrama::new(
                    "fischer: prioridades inesperadas para el centro",
                ));
            }
            let pares = [
                (LUGARES[0], rango[0]),
                (LUGARES[1], rango[1]),
                (LUGARES[2], rango[2]),
                (LUGARES[3], rango[3]),
            ];
            Some(envolver_error(
                "fischer",
                chem::configuracion_fischer(&pares),
            )?)
        } else {
            None
        };

        if let Some(cfg) = config {
            if show_config {
                if blanco || blank_config {
                    caja_vacia(c, cx + brazo + 90.0, yc, 22.0, 15.0);
                } else {
                    c.t(
                        cx + brazo + 90.0,
                        yc + 4.0,
                        &format!("({cfg})"),
                        11.0,
                        700,
                        "textSub",
                        "middle",
                        SANS,
                    );
                }
            }
        }
    }

    let mut y_fin = y + paso * centros.len() as f64 + 20.0;
    tq(
        c, cx, y_fin, &abajo, 11.5, 600, "textMain", "middle", SANS, false,
    );
    y_fin += 26.0;

    let mut serie: Option<&'static str> = None;
    if let Some(ultimo) = centros.last().and_then(Value::as_object) {
        let u_right = ultimo.get("right").map(valor_str).unwrap_or_default();
        let u_left = ultimo.get("left").map(valor_str).unwrap_or_default();
        for grupo in ["OH", "NH2", "NH3^+"] {
            if u_right == grupo {
                serie = Some("D");
            } else if u_left == grupo {
                serie = Some("L");
            }
        }
    }

    if let Some(s) = serie {
        if booleano(b, "showSeries", true) {
            rotulo_q(
                c,
                0.0,
                y_fin,
                "Serie",
                Some(s),
                blanco || booleano(b, "blankSeries", false),
                "start",
                10.5,
            );
            y_fin += 20.0;
        }
    }

    Ok((320.0_f64.max(cx + brazo + 120.0), y_fin))
}

#[cfg(test)]
mod tests {
    use super::*;
    use dibu_core::leer_bloques;
    use serde_json::json;

    fn reg_organica() -> Registro {
        let mut r = Registro::new();
        registrar(&mut r);
        r
    }

    fn bloque_de(val: Value) -> Bloque {
        val.as_object().cloned().unwrap()
    }

    fn texto_plano(svg: &str) -> String {
        let mut out = String::new();
        let mut dentro_tag = false;
        let mut en_text = false;
        let bytes = svg.as_bytes();
        let mut i = 0;
        while i < bytes.len() {
            if svg[i..].starts_with("<text ") || svg[i..].starts_with("<text>") {
                en_text = true;
                dentro_tag = true;
            } else if svg[i..].starts_with("</text>") {
                en_text = false;
                i += "</text>".len();
                continue;
            } else if bytes[i] == b'<' {
                dentro_tag = true;
            } else if bytes[i] == b'>' {
                dentro_tag = false;
                i += 1;
                continue;
            } else if !dentro_tag && en_text {
                let ch = svg[i..].chars().next().unwrap();
                if !ch.is_whitespace() && ch != '\u{00A0}' {
                    out.push(ch);
                }
                i += ch.len_utf8();
                continue;
            }
            i += 1;
        }
        out
    }

    fn attr_num(svg: &str, nombre: &str) -> f64 {
        let pat = format!(r#"{nombre}=""#);
        let idx = svg.find(&pat).unwrap() + pat.len();
        let fin = svg[idx..].find('"').unwrap();
        svg[idx..idx + fin].parse::<f64>().unwrap()
    }

    fn tiene_token_invalido(svg: &str) -> bool {
        let lower = svg.to_lowercase();
        let bytes = lower.as_bytes();
        for pat in [b"nan".as_slice(), b"inf".as_slice()] {
            let mut pos = 0;
            while let Some(rel) = lower[pos..].find(std::str::from_utf8(pat).unwrap()) {
                let idx = pos + rel;
                let antes_ok = idx == 0 || !bytes[idx - 1].is_ascii_alphabetic();
                let fin = idx + pat.len();
                let despues_ok = fin >= bytes.len() || !bytes[fin].is_ascii_alphabetic();
                if antes_ok && despues_ok {
                    return true;
                }
                pos = idx + 1;
            }
        }
        false
    }

    fn validar_svg(svg: &str) {
        assert!(svg.starts_with("<svg "));
        assert!(svg.ends_with("</svg>"));
        assert!(attr_num(svg, "width") > 80.0);
        assert!(attr_num(svg, "height") > 40.0);
        assert!(!tiene_token_invalido(svg), "SVG contiene NaN/inf: {svg}");
    }

    #[test]
    fn fixtures_organica_en_ambas_paletas() {
        let reg = reg_organica();
        let yaml = include_str!("../../../fixtures/diagramas_quimica.yaml");
        let bloques = leer_bloques(yaml).unwrap();
        let organica_ids = [
            "newman-butano-anti",
            "newman-gauche-blanco",
            "newman-eclipsada",
            "silla-metil",
            "silla-dimetil-cis",
            "silla-terbutil-blanco",
            "fischer-gliceraldehido",
            "fischer-alanina-blanco",
            "fischer-glucosa",
        ];
        let mut vistos = 0usize;
        for b in &bloques {
            let id = b.get("id").and_then(Value::as_str).unwrap_or("");
            if organica_ids.contains(&id) {
                vistos += 1;
                for tinta in [true, false] {
                    let svg = reg.renderizar(b, tinta).unwrap();
                    validar_svg(&svg);
                }
            }
        }
        assert_eq!(vistos, organica_ids.len());
    }

    #[test]
    fn newman_conformaciones_blanks_y_eje() {
        let reg = reg_organica();
        let casos = [
            (180.0, "180°", "alternadaanti"),
            (-60.0, "60°", "alternadagauche"),
            (0.0, "0°", "eclipsada(sinperiplanar)"),
            (120.0, "120°", "eclipsada(anticlinal)"),
            (374.0, "14°", "eclipsada(sinperiplanar)"),
            (45.0, "45°", "diedrode45°"),
        ];
        for (diedro, deg, conf) in casos {
            let b = bloque_de(json!({
                "type": "newman",
                "front": ["CH3", "H", "H"],
                "back": ["CH3", "H", "H"],
                "dihedral": diedro
            }));
            let svg = reg.renderizar(&b, true).unwrap();
            validar_svg(&svg);
            let txt = texto_plano(&svg);
            assert!(txt.contains(deg), "diedro {diedro}: {txt}");
            assert!(txt.contains(conf), "diedro {diedro}: {txt}");
        }

        // axis + blankDihedral + blankName + blank + inkSaver override
        let b_eje = bloque_de(json!({
            "type": "newman",
            "axis": "C2–C3",
            "front": ["NH3^+", "H", "H"],
            "back": ["COO^-", "H", "H"],
            "dihedral": 60,
            "blankName": true,
            "blankDihedral": true,
            "inkSaver": true
        }));
        let svg_eje = reg.renderizar(&b_eje, false).unwrap();
        let txt_eje = texto_plano(&svg_eje);
        assert!(txt_eje.contains("MirandoalolargodeC2–C3"));
        assert!(txt_eje.contains("NH3+"));
        assert!(txt_eje.contains("COO−"));
        assert!(!txt_eje.contains("60°"));
        assert!(!txt_eje.contains("gauche"));
        assert!(!svg_eje.contains("#1d4ed8"));

        let b_blank = bloque_de(json!({
            "type": "newman",
            "front": ["A", "B", "C"],
            "back": ["D", "E", "F"],
            "blank": true
        }));
        let txt_blank = texto_plano(&reg.renderizar(&b_blank, true).unwrap());
        assert!(!txt_blank.contains("180°"));
        assert!(!txt_blank.contains("anti"));

        // Extremo numérico 1e308 sin NaN ni inf (signos opuestos y mismo signo)
        for (diedro, inicio) in [(1e308, -1e308), (1e308, 1e308), (-1e308, -1e308)] {
            let b_ext = bloque_de(json!({
                "type": "newman",
                "front": ["A", "B", "C"],
                "back": ["D", "E", "F"],
                "dihedral": diedro,
                "start": inicio
            }));
            let svg_ext = reg.renderizar(&b_ext, true).unwrap();
            validar_svg(&svg_ext);
        }
    }

    #[test]
    fn newman_errores() {
        let reg = reg_organica();
        let e1 = reg
            .renderizar(
                &bloque_de(json!({"type": "newman", "front": ["H"], "back": ["H", "H", "H"]})),
                true,
            )
            .unwrap_err()
            .to_string();
        assert!(e1.contains("tres sustituyentes"), "{e1}");

        let e2 = reg
            .renderizar(
                &bloque_de(json!({"type": "newman", "back": ["H", "H", "H"]})),
                true,
            )
            .unwrap_err()
            .to_string();
        assert!(e2.contains("tres sustituyentes"), "{e2}");

        let e3 = reg
            .renderizar(
                &bloque_de(json!({
                    "type": "newman",
                    "front": ["H", "H", "H"],
                    "back": ["H", "H", "H"],
                    "dihedral": "abc"
                })),
                true,
            )
            .unwrap_err()
            .to_string();
        assert!(e3.contains("mal formado") && e3.contains("newman"), "{e3}");

        let e4 = reg
            .renderizar(
                &bloque_de(json!({
                    "type": "newman",
                    "front": 123,
                    "back": ["H", "H", "H"]
                })),
                true,
            )
            .unwrap_err()
            .to_string();
        assert!(e4.contains("mal formado"), "{e4}");
    }

    #[test]
    fn chair_estabilidad_relacion_y_blanks() {
        let reg = reg_organica();

        let svg_metil = reg
            .renderizar(
                &bloque_de(json!({
                    "type": "chair",
                    "substituents": [{"position": 1, "group": "CH3", "axial": true}],
                    "flip": true
                })),
                true,
            )
            .unwrap();
        validar_svg(&svg_metil);
        assert!(texto_plano(&svg_metil).contains("laderecha,por7,3kJ/mol"));

        let svg_cis = reg
            .renderizar(
                &bloque_de(json!({
                    "type": "chair",
                    "substituents": [
                        {"position": 1, "group": "CH3", "axial": true},
                        {"position": 2, "group": "CH3", "axial": false}
                    ],
                    "flip": true
                })),
                true,
            )
            .unwrap();
        let txt_cis = texto_plano(&svg_cis);
        assert!(txt_cis.contains("cis(1,2)"));
        assert!(txt_cis.contains("lasdossillastienenlamismaenergía"));

        let svg_trans = reg
            .renderizar(
                &bloque_de(json!({
                    "type": "chair",
                    "substituents": [
                        {"position": 1, "group": "tBu", "axial": false},
                        {"position": 4, "group": "OH", "axial": false}
                    ],
                    "flip": false
                })),
                false,
            )
            .unwrap();
        let txt_trans = texto_plano(&svg_trans);
        assert!(txt_trans.contains("trans(1,4)"));
        assert!(!txt_trans.contains("Másestable"));

        // Grupo sin VALORES_A omite Más estable
        let svg_no2 = reg
            .renderizar(
                &bloque_de(json!({
                    "type": "chair",
                    "substituents": [
                        {"position": 1, "group": "NO2", "axial": true},
                        {"position": 2, "group": "H", "axial": false}
                    ],
                    "flip": true
                })),
                true,
            )
            .unwrap();
        let txt_no2 = texto_plano(&svg_no2);
        assert!(txt_no2.contains("NO2"));
        assert!(!txt_no2.contains("Másestable"));
        assert!(!txt_no2.contains("Relación"));

        // H explícito ignorado en cis/trans pero considerado en VALORES_A
        let svg_h = reg
            .renderizar(
                &bloque_de(json!({
                    "type": "chair",
                    "substituents": [
                        {"position": 1, "group": "CH3", "axial": true},
                        {"position": 1, "group": "H", "axial": false},
                        {"position": 4, "group": "Cl", "axial": false}
                    ],
                    "flip": true
                })),
                true,
            )
            .unwrap();
        let txt_h = texto_plano(&svg_h);
        assert!(txt_h.contains("cis(1,4)"));
        assert!(txt_h.contains("laderecha,por5,1kJ/mol"));

        // blank, blankRelation, blankStability, sust.blank
        let svg_bl = reg
            .renderizar(
                &bloque_de(json!({
                    "type": "chair",
                    "substituents": [
                        {"position": 1, "group": "tBu", "axial": false, "blank": true},
                        {"position": 4, "group": "CH3", "axial": false}
                    ],
                    "flip": true,
                    "blank": true
                })),
                true,
            )
            .unwrap();
        let txt_bl = texto_plano(&svg_bl);
        assert!(txt_bl.contains("CH3"));
        assert!(!txt_bl.contains("tBu"));
        assert!(!txt_bl.contains("trans"));
        assert!(!txt_bl.contains("kJ/mol"));

        let svg_sel = reg
            .renderizar(
                &bloque_de(json!({
                    "type": "chair",
                    "substituents": [
                        {"position": 1, "group": "Et", "axial": true},
                        {"position": 3, "group": "F", "axial": true}
                    ],
                    "flip": true,
                    "blankRelation": true,
                    "blankStability": true
                })),
                true,
            )
            .unwrap();
        let txt_sel = texto_plano(&svg_sel);
        assert!(txt_sel.contains("Relación:"));
        assert!(txt_sel.contains("Másestable:"));
        assert!(!txt_sel.contains("cis"));
        assert!(!txt_sel.contains("kJ/mol"));
    }

    #[test]
    fn chair_errores() {
        let reg = reg_organica();
        for pos in [0, 7] {
            let e = reg
                .renderizar(
                    &bloque_de(json!({
                        "type": "chair",
                        "substituents": [{"position": pos, "group": "CH3"}]
                    })),
                    true,
                )
                .unwrap_err()
                .to_string();
            assert!(e.contains("de 1 a 6"), "{e}");
        }

        let e_grp = reg
            .renderizar(
                &bloque_de(json!({
                    "type": "chair",
                    "substituents": [{"position": 1}]
                })),
                true,
            )
            .unwrap_err()
            .to_string();
        assert!(e_grp.contains("substituents[0]"), "{e_grp}");

        let e_tipo = reg
            .renderizar(
                &bloque_de(json!({
                    "type": "chair",
                    "substituents": [{"position": "uno", "group": "CH3"}]
                })),
                true,
            )
            .unwrap_err()
            .to_string();
        assert!(e_tipo.contains("mal formado") && e_tipo.contains("chair"), "{e_tipo}");
    }

    #[test]
    fn fischer_cip_priorities_serie_y_blanks() {
        let reg = reg_organica();

        // D-gliceraldehído
        let txt_d = texto_plano(
            &reg.renderizar(
                &bloque_de(json!({
                    "type": "fischer",
                    "top": "CHO",
                    "bottom": "CH2OH",
                    "centers": [{"left": "H", "right": "OH"}]
                })),
                true,
            )
            .unwrap(),
        );
        assert!(txt_d.contains("(R)") && txt_d.contains("Serie=D"), "{txt_d}");

        // L-alanina NH3^+
        let txt_l = texto_plano(
            &reg.renderizar(
                &bloque_de(json!({
                    "type": "fischer",
                    "top": "COOH",
                    "bottom": "CH3",
                    "centers": [{"left": "NH3^+", "right": "H"}]
                })),
                false,
            )
            .unwrap(),
        );
        assert!(txt_l.contains("(S)") && txt_l.contains("Serie=L"), "{txt_l}");

        // Menor prioridad arriba sin serie
        let txt_top_h = texto_plano(
            &reg.renderizar(
                &bloque_de(json!({
                    "type": "fischer",
                    "top": "H",
                    "bottom": "CH3",
                    "centers": [{"left": "Br", "right": "Cl"}]
                })),
                true,
            )
            .unwrap(),
        );
        assert!(txt_top_h.contains("(R)") && !txt_top_h.contains("Serie"));

        // Glucosa 4 centros
        let txt_glu = texto_plano(
            &reg.renderizar(
                &bloque_de(json!({
                    "type": "fischer",
                    "top": "CHO",
                    "bottom": "CH2OH",
                    "centers": [
                        {"left": "H", "right": "OH", "priorities": ["right", "top", "bottom", "left"]},
                        {"left": "HO", "right": "H", "priorities": ["left", "top", "bottom", "right"]},
                        {"left": "H", "right": "OH", "priorities": ["right", "top", "bottom", "left"]},
                        {"left": "H", "right": "OH", "priorities": ["right", "top", "bottom", "left"]}
                    ]
                })),
                true,
            )
            .unwrap(),
        );
        assert_eq!(txt_glu.matches("(R)").count(), 3);
        assert_eq!(txt_glu.matches("(S)").count(), 1);
        assert!(txt_glu.contains("Serie=D"));

        // Multicentro sin priorities: omite R/S sin error
        let txt_multi = texto_plano(
            &reg.renderizar(
                &bloque_de(json!({
                    "type": "fischer",
                    "top": "CHO",
                    "bottom": "CH2OH",
                    "centers": [
                        {"left": "H", "right": "OH"},
                        {"left": "HO", "right": "H"}
                    ]
                })),
                true,
            )
            .unwrap(),
        );
        assert!(!txt_multi.contains("(R)") && !txt_multi.contains("(S)"));

        // centro.blank: "left"
        let txt_bl_left = texto_plano(
            &reg.renderizar(
                &bloque_de(json!({
                    "type": "fischer",
                    "top": "COOH",
                    "bottom": "CH3",
                    "centers": [{"left": "NH2", "right": "H", "blank": "left"}],
                    "blankSeries": true
                })),
                true,
            )
            .unwrap(),
        );
        assert!(!txt_bl_left.contains("NH2") && !txt_bl_left.contains("(S)"));
        assert!(txt_bl_left.contains("Serie="));
        assert!(!txt_bl_left.contains("Serie=L"));

        // centro.blank: true mantiene Serie=D
        let txt_bl_true = texto_plano(
            &reg.renderizar(
                &bloque_de(json!({
                    "type": "fischer",
                    "top": "CHO",
                    "bottom": "CH2CH3",
                    "centers": [{"left": "H", "right": "OH", "blank": true}]
                })),
                true,
            )
            .unwrap(),
        );
        assert!(txt_bl_true.contains("Serie=D"));
        assert!(!txt_bl_true.contains("OH") && !txt_bl_true.contains("(R)"));

        // blankConfig, showConfig: false, showSeries: false
        let txt_show = texto_plano(
            &reg.renderizar(
                &bloque_de(json!({
                    "type": "fischer",
                    "top": "CHO",
                    "bottom": "CH2OH",
                    "centers": [{"left": "H", "right": "OH"}],
                    "showConfig": false,
                    "showSeries": false
                })),
                true,
            )
            .unwrap(),
        );
        assert!(!txt_show.contains("(R)") && !txt_show.contains("Serie"));

        let txt_bcfg = texto_plano(
            &reg.renderizar(
                &bloque_de(json!({
                    "type": "fischer",
                    "top": "CHO",
                    "bottom": "CH2OH",
                    "centers": [{"left": "H", "right": "OH"}],
                    "blankConfig": true
                })),
                true,
            )
            .unwrap(),
        );
        assert!(!txt_bcfg.contains("(R)"));
        assert!(txt_bcfg.contains("Serie=D"));
    }

    #[test]
    fn fischer_errores() {
        let reg = reg_organica();
        let e1 = reg
            .renderizar(&bloque_de(json!({"type": "fischer", "centers": []})), true)
            .unwrap_err()
            .to_string();
        assert!(e1.contains("centers"), "{e1}");

        let e2 = reg
            .renderizar(
                &bloque_de(json!({"type": "fischer", "centers": [{"left": "H"}]})),
                true,
            )
            .unwrap_err()
            .to_string();
        assert!(e2.contains("centers[0]"), "{e2}");

        let e3 = reg
            .renderizar(
                &bloque_de(json!({
                    "type": "fischer",
                    "top": "CHO",
                    "bottom": "CH2OH",
                    "centers": [{"left": "H", "right": "OH", "priorities": ["top", "top", "bottom", "left"]}]
                })),
                true,
            )
            .unwrap_err()
            .to_string();
        assert!(e3.contains("priorities son top, right, bottom y left"), "{e3}");

        let e4 = reg
            .renderizar(
                &bloque_de(json!({
                    "type": "fischer",
                    "top": "tBu",
                    "bottom": "Ph",
                    "centers": [{"left": "H", "right": "OH"}]
                })),
                true,
            )
            .unwrap_err()
            .to_string();
        assert!(e4.contains("empatan"), "{e4}");

        let e5 = reg
            .renderizar(
                &bloque_de(json!({
                    "type": "fischer",
                    "top": "CH2CH2CH2OH",
                    "bottom": "CH2OH",
                    "centers": [{"left": "H", "right": "OH"}]
                })),
                true,
            )
            .unwrap_err()
            .to_string();
        assert!(e5.contains("priorities"), "{e5}");
    }
}
