//! Renderizadores de gráficos y curvas de química general (M1b2):
//! `titration`, `speciation`, `kinetics`, `phase_diagram`, `heating_curve`, `spectrum`.

use crate::chem;
use crate::common::{
    ancho_formula, booleano, cadena, caja_vacia, coma, encabezado_q, entero, envolver_error,
    leyenda_q, lista, num, opt_lista, opt_num, opt_texto, rotulo_q, tq,
};
use dibu_core::{valor_str, Bloque, ErrorDiagrama, Lienzo, Punto, Registro, SANS};
use dibu_plot::{fmt_g, Ejes, EjesConfig};
use serde_json::{json, Value};
use std::fmt::Write as _;

const MAX_VECINOS_NMR: i64 = 20;
const MAX_PKA: usize = 20;

pub fn registrar(reg: &mut Registro) {
    reg.registrar("titration", render_titration);
    reg.registrar("speciation", render_speciation);
    reg.registrar("kinetics", render_kinetics);
    reg.registrar("phase_diagram", render_phase_diagram);
    reg.registrar("heating_curve", render_heating_curve);
    reg.registrar("spectrum", render_spectrum);
}

fn extraer_rango(val: &Value, donde: &str) -> Result<(f64, f64), ErrorDiagrama> {
    let arr = lista(val, donde)?;
    if arr.len() != 2 {
        return Err(ErrorDiagrama::new(format!(
            "{donde}: tiene que tener exactamente 2 valores"
        )));
    }
    let a = num(&arr[0], &format!("{donde}[0]"))?;
    let b = num(&arr[1], &format!("{donde}[1]"))?;
    Ok((a, b))
}

fn leer_anotaciones(
    b: &Bloque,
    donde: &str,
    defecto: &[&str],
) -> Result<Vec<String>, ErrorDiagrama> {
    match opt_lista(b, "annotations", donde)? {
        None => Ok(defecto.iter().map(|s| (*s).to_string()).collect()),
        Some(arr) => arr
            .iter()
            .enumerate()
            .map(|(i, v)| cadena(v, &format!("{donde}[{i}]")))
            .collect(),
    }
}

fn rango_ejes_valido(lo: f64, hi: f64) -> bool {
    let span = hi - lo;
    lo.is_finite()
        && hi.is_finite()
        && lo != hi
        && span.is_finite()
        && (1000.0 / span).is_finite()
        && (lo * 1e13).is_finite()
        && (hi * 1e13).is_finite()
}

fn curva_segura(
    c: &mut Lienzo,
    ejes: &Ejes,
    puntos: &[Punto],
    estilo: usize,
    ancho: f64,
    color: &str,
) {
    let filtrados: Vec<Punto> = puntos
        .iter()
        .map(|&(x, y)| {
            if x.is_finite() && y.is_finite() && ejes.px(x).is_finite() && ejes.py(y).is_finite() {
                (x, y)
            } else {
                (f64::NAN, f64::NAN)
            }
        })
        .collect();
    ejes.curva(c, &filtrados, estilo, ancho, color);
}

// --- 1. Titulación ---------------------------------------------------------------------------------------

fn render_titration(c: &mut Lienzo, b: &Bloque) -> Result<Punto, ErrorDiagrama> {
    let titulante = match b.get("titrant") {
        None | Some(Value::Null) => None,
        Some(Value::Object(m)) => Some(m),
        Some(_) => return Err(ErrorDiagrama::new("titration: 'titrant' es {conc, name}")),
    };
    let def_conc = json!(0.1);
    let conc_tit = titulante
        .and_then(|m| m.get("conc"))
        .filter(|v| !v.is_null())
        .unwrap_or(&def_conc);
    let t = envolver_error(
        "titration",
        chem::titulacion(b.get("analyte"), conc_tit, b.get("vMax"), 400),
    )?;
    if !rango_ejes_valido(0.0, t.vmax) || t.vmax <= 0.0 {
        return Err(ErrorDiagrama::new(
            "titration: el volumen máximo vMax tiene que ser positivo y finito",
        ));
    }

    let nombre = titulante
        .and_then(|m| m.get("name"))
        .filter(|v| !v.is_null())
        .map(valor_str)
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| {
            if t.analito.es_acido() {
                "NaOH".to_string()
            } else {
                "HCl".to_string()
            }
        });

    let top = encabezado_q(c, b);
    let (x0, y0, w, h) = (58.0, top + 16.0, 460.0, 260.0);
    let ejes = Ejes::dibujar(
        c,
        EjesConfig {
            x0,
            y0,
            w,
            h,
            xr: (0.0, t.vmax),
            yr: (0.0, 14.0),
            xlabel: "",
            ylabel: "pH",
            log: false,
            grilla: true,
            origen: false,
            ypaso: Some(2.0),
        },
    );
    tq(
        c,
        x0 + w / 2.0,
        y0 + h + 30.0,
        &format!("V {nombre} (mL)"),
        10.0,
        600,
        "textMain",
        "middle",
        SANS,
        false,
    );

    if let Some(ind_val) = b.get("indicator").filter(|v| !v.is_null()) {
        let ind = ind_val
            .as_object()
            .ok_or_else(|| ErrorDiagrama::new("titration.indicator: tiene que ser un diccionario"))?;
        if let Some(rango_val) = ind.get("range").filter(|v| !v.is_null()) {
            let (raw_a, raw_z) = extraer_rango(rango_val, "titration.indicator.range")?;
            let (a, z) = if raw_a <= raw_z {
                (raw_a, raw_z)
            } else {
                (raw_z, raw_a)
            };
            if z >= 0.0 && a <= 14.0 {
                let a_vis = a.clamp(0.0, 14.0);
                let z_vis = z.clamp(0.0, 14.0);
                let y_top = ejes.py(z_vis).clamp(y0, y0 + h);
                let y_bot = ejes.py(a_vis).clamp(y0, y0 + h);
                if !(y_top.is_finite()
                    && y_bot.is_finite()
                    && (y_bot - y_top).is_finite()
                    && (y_top - 3.0).is_finite())
                {
                    return Err(ErrorDiagrama::new(
                        "titration.indicator.range: rango fuera de límites finitos",
                    ));
                }
                let height = (y_bot - y_top).max(0.0);
                c.add(format!(
                    r#"<rect x="{x0:.1}" y="{y_top:.1}" width="{w:.1}" height="{height:.1}" fill="{}" opacity="0.7"/>"#,
                    c.pal.fill_highlight
                ));
                let ind_nombre = ind
                    .get("name")
                    .filter(|v| !v.is_null())
                    .map(valor_str)
                    .filter(|s| !s.is_empty())
                    .unwrap_or_else(|| "indicador".to_string());
                let fmt_ph = |v: f64| {
                    if v.abs() < 1e4 {
                        coma(v, 1)
                    } else {
                        fmt_g(Some(v), 3)
                    }
                };
                c.t(
                    x0 + w - 4.0,
                    y_top - 3.0,
                    &format!("{ind_nombre} ({}–{})", fmt_ph(a), fmt_ph(z)),
                    9.0,
                    400,
                    "textSub",
                    "end",
                    SANS,
                );
            }
        }
    }

    let curva: Vec<Punto> = t
        .volumenes
        .iter()
        .copied()
        .zip(t.phs.iter().copied())
        .collect();
    curva_segura(c, &ejes, &curva, 0, 2.0, "strokeMain");

    if let Some(datos) = opt_lista(b, "data", "titration.data")? {
        for (i, item) in datos.iter().enumerate() {
            let (v, ph) = extraer_rango(item, &format!("titration.data[{i}]"))?;
            let cx = ejes.px(v);
            let cy = ejes.py(ph);
            if cx.is_finite()
                && cy.is_finite()
                && cx >= x0 - 2.0
                && cx <= x0 + w + 2.0
                && cy >= y0 - 2.0
                && cy <= y0 + h + 2.0
            {
                c.add(format!(
                    r#"<circle cx="{cx:.1}" cy="{cy:.1}" r="2.6" fill="none" stroke="{}" stroke-width="1.2"/>"#,
                    c.pal.stroke_sub
                ));
            }
        }
    }

    let pedidas = leer_anotaciones(b, "titration.annotations", &["equivalence", "half"])?;
    let blanco = booleano(b, "blankAnnotations", false);
    let varias = t.equivalencias.len() > 1;

    if pedidas.iter().any(|s| s == "equivalence") {
        for (idx, &(v, ph)) in t.equivalencias.iter().enumerate() {
            let k = idx + 1;
            let (px_v, py_ph, py_0) = (ejes.px(v), ejes.py(ph), ejes.py(0.0));
            let x_txt = px_v + 8.0;
            let arriba = t.analito.es_acido();
            let y_txt = py_ph + if arriba { 14.0 } else { -8.0 };
            if !(px_v.is_finite()
                && py_ph.is_finite()
                && py_0.is_finite()
                && x_txt.is_finite()
                && y_txt.is_finite()
                && (y_txt + 13.0).is_finite())
            {
                continue;
            }
            ejes.guia(c, v, 0.0, v, ph, "4 3", "strokeSub");
            ejes.guia(c, 0.0, ph, v, ph, "4 3", "strokeSub");
            c.punto(px_v, py_ph, "accentPrimary", 3.4);
            let sufijo = if varias {
                format!("_{k}")
            } else {
                String::new()
            };
            tq(
                c,
                x_txt,
                y_txt,
                &format!("V_eq{sufijo} = {} mL", fmt_g(Some(v), 3)),
                9.5,
                700,
                "textMain",
                "start",
                SANS,
                true,
            );
            rotulo_q(
                c,
                x_txt,
                y_txt + 13.0,
                "pH",
                Some(&coma(ph, 2)),
                blanco,
                "start",
                9.5,
            );
        }
    }

    if pedidas.iter().any(|s| s == "half") {
        for (idx, &(v, ph, pka)) in t.semi.iter().enumerate() {
            let k = idx + 1;
            let (px_v, py_ph, py_0) = (ejes.px(v), ejes.py(ph), ejes.py(0.0));
            if !(px_v.is_finite() && py_ph.is_finite() && py_0.is_finite()) {
                continue;
            }
            ejes.guia(c, v, 0.0, v, ph, "2 3", "strokeSub");
            c.add(format!(
                r#"<circle cx="{px_v:.1}" cy="{py_ph:.1}" r="3.2" fill="{}" stroke="{}" stroke-width="1.4"/>"#,
                c.pal.fill_cell,
                c.pal.stroke_main
            ));
            let sufijo = if t.semi.len() > 1 {
                format!("_{k}")
            } else {
                String::new()
            };
            let texto = if blanco {
                format!("pK_a{sufijo} =")
            } else {
                format!("pH = pK_a{sufijo} = {}", coma(pka, 2))
            };
            let ancho = ancho_formula(&texto, 9.5) + if blanco { 44.0 } else { 0.0 };
            let y_txt = if t.analito.es_acido() {
                py_ph - 8.0
            } else {
                py_ph + 16.0
            };
            let (x_txt, ancla) = if px_v - 6.0 - ancho >= x0 + 4.0 {
                (px_v - 6.0 - if blanco { 44.0 } else { 0.0 }, "end")
            } else {
                (px_v + 8.0, "start")
            };
            if !(x_txt.is_finite() && y_txt.is_finite()) {
                continue;
            }
            tq(c, x_txt, y_txt, &texto, 9.5, 700, "textMain", ancla, SANS, true);
            if blanco {
                let inicio = if ancla == "end" {
                    x_txt + 3.0
                } else {
                    x_txt + ancho_formula(&texto, 9.5) + 3.0
                };
                c.vacio(inicio, inicio + 38.0, y_txt + 2.0);
            }
        }
    }

    if pedidas.iter().any(|s| s == "initial") {
        let valor = if blanco {
            None
        } else {
            Some(coma(t.ph_inicial(), 2))
        };
        rotulo_q(
            c,
            x0 + 8.0,
            y0 + 16.0,
            "pH inicial",
            valor.as_deref(),
            blanco,
            "start",
            9.5,
        );
    }

    Ok((x0 + w + 10.0, y0 + h + 40.0))
}

// --- 2. Especiación --------------------------------------------------------------------------------------

fn especies_por_defecto(n: usize) -> Vec<String> {
    let mut salida = Vec::with_capacity(n + 1);
    for j in 0..=n {
        let protones = n - j;
        let h = match protones {
            0 => String::new(),
            1 => "H".to_string(),
            p => format!("H{p}"),
        };
        let carga = if j == 0 {
            String::new()
        } else if j == 1 {
            "^-".to_string()
        } else {
            format!("^{j}-")
        };
        salida.push(format!("{h}A{carga}"));
    }
    salida
}

fn render_speciation(c: &mut Lienzo, b: &Bloque) -> Result<Punto, ErrorDiagrama> {
    let pka_vals = chem::lista(b.get("pKa"));
    if pka_vals.is_empty() {
        return Err(ErrorDiagrama::new(
            "speciation: falta pKa (uno o una lista)",
        ));
    }
    if pka_vals.len() > MAX_PKA {
        return Err(ErrorDiagrama::new(format!(
            "speciation: como máximo se admiten {MAX_PKA} valores de pKa"
        )));
    }
    let mut pkas = Vec::with_capacity(pka_vals.len());
    for (i, v) in pka_vals.into_iter().enumerate() {
        let pka = num(v, &format!("speciation.pKa[{i}]"))?;
        if !(-10.0..=24.0).contains(&pka) {
            return Err(ErrorDiagrama::new(format!(
                "speciation.pKa[{i}]: pKa ({pka}) fuera de rango [-10, 24]"
            )));
        }
        pkas.push(pka);
    }
    pkas.sort_by(|a, b| a.total_cmp(b));

    let especies: Vec<String> = match opt_lista(b, "species", "speciation.species")? {
        Some(arr) if !arr.is_empty() => arr.iter().map(valor_str).collect(),
        _ => especies_por_defecto(pkas.len()),
    };
    if especies.len() != pkas.len() + 1 {
        return Err(ErrorDiagrama::new(format!(
            "speciation: con {} pKa hay {} especies (species tiene {})",
            pkas.len(),
            pkas.len() + 1,
            especies.len()
        )));
    }

    let top = encabezado_q(c, b);
    let (x0, y0, w, h) = (58.0, top + 20.0, 460.0, 230.0);
    let ejes = Ejes::dibujar(
        c,
        EjesConfig {
            x0,
            y0,
            w,
            h,
            xr: (0.0, 14.0),
            yr: (0.0, 1.0),
            xlabel: "pH",
            ylabel: "fracción α",
            log: false,
            grilla: true,
            origen: false,
            ypaso: Some(0.2),
        },
    );

    let kas: Vec<f64> = pkas.iter().map(|&p| 10_f64.powf(-p)).collect();
    let mut curvas: Vec<Vec<Punto>> = vec![Vec::with_capacity(561); especies.len()];
    for i in 0..=560 {
        let ph = 14.0 * i as f64 / 560.0;
        for (j, alfa) in chem::fracciones(10_f64.powf(-ph), &kas)
            .into_iter()
            .enumerate()
        {
            curvas[j].push((ph, alfa));
        }
    }

    let blanco_especies = booleano(b, "blankSpecies", false);
    for (j, curva) in curvas.iter().enumerate() {
        curva_segura(c, &ejes, curva, j, 1.8, "strokeMain");
        // Desempate estricto con `>`: primer máximo, igual que `max()` de Python.
        let mut ph_max = curva[0].0;
        let mut a_max = curva[0].1;
        for &(ph, a) in &curva[1..] {
            if a > a_max {
                ph_max = ph;
                a_max = a;
            }
        }
        let mut x = ejes.px(ph_max);
        let y = ejes.py(a_max) - 6.0;
        if !(x.is_finite() && y.is_finite()) {
            continue;
        }
        if j == 0 {
            x = x.max(x0 + 26.0);
        }
        if j == curvas.len() - 1 {
            x = x.min(x0 + w - 26.0);
        }
        if blanco_especies {
            caja_vacia(c, x, y - 4.0, 40.0, 14.0);
        } else {
            tq(
                c,
                x,
                y,
                &especies[j],
                10.5,
                700,
                "textMain",
                "middle",
                SANS,
                false,
            );
        }
    }

    let blanco = booleano(b, "blankPKa", false);
    for (idx, &pka) in pkas.iter().enumerate() {
        let k = idx + 1;
        let x_pka = ejes.px(pka);
        let y_mid = ejes.py(0.5);
        let y_bot = ejes.py(0.0);
        if !(x_pka.is_finite()
            && y_mid.is_finite()
            && y_bot.is_finite()
            && (x_pka + 3.0).is_finite())
        {
            return Err(ErrorDiagrama::new(
                "speciation: pKa fuera de rango finito del gráfico",
            ));
        }
        ejes.guia(c, pka, 0.0, pka, 0.5, "2 3", "strokeSub");
        c.punto(x_pka, y_mid, "accentPrimary", 2.6);
        let sufijo = if pkas.len() > 1 {
            format!("_{k}")
        } else {
            String::new()
        };
        let y = y0 + h - 8.0 - if k % 2 == 0 { 12.0 } else { 0.0 };
        rotulo_q(
            c,
            x_pka + 3.0,
            y,
            &format!("pK_a{sufijo}"),
            Some(&coma(pka, 2)),
            blanco,
            "start",
            9.0,
        );
    }

    Ok((x0 + w + 10.0, y0 + h + 40.0))
}

// --- 3. Cinética -----------------------------------------------------------------------------------------

fn chem_pos_o_def(b: &Bloque, clave: &str, donde: &str, defecto: f64) -> Result<f64, ErrorDiagrama> {
    let def_val = json!(defecto);
    let val = b.get(clave).filter(|v| !v.is_null()).unwrap_or(&def_val);
    envolver_error("kinetics", chem::num(val, donde, true, None))
}

fn leer_t_end(b: &Bloque, defecto: f64) -> Result<f64, ErrorDiagrama> {
    let t_fin = match opt_num(b, "tEnd", "kinetics.tEnd")? {
        Some(v) => {
            if v <= 0.0 {
                return Err(ErrorDiagrama::new("kinetics.tEnd: tiene que ser positivo"));
            }
            v
        }
        None => defecto,
    };
    if !t_fin.is_finite() || t_fin <= 0.0 {
        return Err(ErrorDiagrama::new(
            "kinetics: tEnd resultante tiene que ser positivo y finito",
        ));
    }
    Ok(t_fin)
}

fn render_kinetics(c: &mut Lienzo, b: &Bloque) -> Result<Punto, ErrorDiagrama> {
    let top = encabezado_q(c, b);
    let (x0, y0, w, h) = (64.0, top + 16.0, 450.0, 240.0);
    let modelo = opt_texto(b, "model").unwrap_or_else(|| "simple".to_string());
    let grafico = opt_texto(b, "plot").unwrap_or_else(|| "concentration".to_string());
    let blanco = booleano(b, "blankAnnotations", false);
    let pedidas = leer_anotaciones(b, "kinetics.annotations", &[])?;

    if grafico == "arrhenius" {
        let ea = chem_pos_o_def(b, "Ea", "Ea (kJ/mol)", 50.0)? * 1000.0;
        let a = chem_pos_o_def(b, "A", "A", 1e10)?;
        if !ea.is_finite() || !a.ln().is_finite() {
            return Err(ErrorDiagrama::new(
                "kinetics: parámetros Ea o A fuera de rango finito",
            ));
        }
        let (t1, t2) = match b.get("T").filter(|v| !v.is_null()) {
            Some(v) => extraer_rango(v, "kinetics.T")?,
            None => (280.0, 380.0),
        };
        if !(t1 > 0.0 && t2 > 0.0 && t1 != t2) {
            return Err(ErrorDiagrama::new(
                "kinetics: T requiere dos temperaturas positivas distintas [T1, T2]",
            ));
        }
        let mut xs = Vec::with_capacity(101);
        let mut ys = Vec::with_capacity(101);
        for i in 0..=100 {
            let t_val = t1 + (t2 - t1) * (i as f64 / 100.0);
            let x = 1000.0 / t_val;
            let y = a.ln() - ea / (chem::R * t_val);
            if !x.is_finite() || !y.is_finite() {
                return Err(ErrorDiagrama::new(
                    "kinetics: valores de Arrhenius fuera de rango finito",
                ));
            }
            xs.push(x);
            ys.push(y);
        }
        let min_x = xs.iter().copied().fold(f64::INFINITY, f64::min);
        let max_x = xs.iter().copied().fold(f64::NEG_INFINITY, f64::max);
        let min_y = ys.iter().copied().fold(f64::INFINITY, f64::min);
        let max_y = ys.iter().copied().fold(f64::NEG_INFINITY, f64::max);
        let (y_lo, y_hi) = (min_y - 0.5, max_y + 0.5);
        if !(min_x < max_x
            && y_lo < y_hi
            && rango_ejes_valido(min_x, max_x)
            && rango_ejes_valido(y_lo, y_hi))
        {
            return Err(ErrorDiagrama::new(
                "kinetics: rango degenerado en gráfico de Arrhenius",
            ));
        }
        let ejes = Ejes::dibujar(
            c,
            EjesConfig {
                x0,
                y0,
                w,
                h,
                xr: (min_x, max_x),
                yr: (y_lo, y_hi),
                xlabel: "1000/T (K⁻¹)",
                ylabel: "ln k",
                log: false,
                grilla: true,
                origen: false,
                ypaso: None,
            },
        );
        let pts: Vec<Punto> = xs.into_iter().zip(ys).collect();
        for &(x, y) in &pts {
            if !(ejes.px(x).is_finite() && ejes.py(y).is_finite()) {
                return Err(ErrorDiagrama::new(
                    "kinetics: coordenadas de Arrhenius fuera de rango finito",
                ));
            }
        }
        curva_segura(c, &ejes, &pts, 0, 2.0, "strokeMain");
        let pendiente = -ea / chem::R;
        if !pendiente.is_finite() {
            return Err(ErrorDiagrama::new(
                "kinetics: pendiente de Arrhenius no finita",
            ));
        }
        let y = y0 + 18.0;
        let val_txt = format!("{pendiente:.0} K").replace('-', "−");
        rotulo_q(
            c,
            x0 + w - 200.0,
            y,
            "pendiente = −Ea/R",
            Some(&val_txt),
            blanco,
            "start",
            10.0,
        );
        c.t(
            x0 + w - 200.0,
            y + 15.0,
            "(el eje x es 1000/T: la recta baja Ea/R por cada 1000)",
            8.5,
            400,
            "textSub",
            "start",
            SANS,
        );
        return Ok((x0 + w + 10.0, y0 + h + 40.0));
    }

    if modelo == "consecutive" {
        let k1 = chem_pos_o_def(b, "k1", "k1", 1.0)?;
        let k2 = chem_pos_o_def(b, "k2", "k2", 0.5)?;
        let a0 = chem_pos_o_def(b, "A0", "A0", 1.0)?;
        let t_fin = leer_t_end(b, 5.0 / k1.min(k2))?;
        let y_top = a0 * 1.08;
        if !rango_ejes_valido(0.0, t_fin) || !rango_ejes_valido(0.0, y_top) || y_top <= 0.0 {
            return Err(ErrorDiagrama::new("kinetics: parámetros fuera de rango finito"));
        }
        let ts: Vec<f64> = (0..=300).map(|i| t_fin * (i as f64 / 300.0)).collect();
        let valores: Vec<(f64, f64, f64)> =
            ts.iter().map(|&t| chem::consecutiva(k1, k2, a0, t)).collect();
        let xlabel = opt_texto(b, "xLabel").unwrap_or_else(|| "t (s)".to_string());
        let ylabel = opt_texto(b, "yLabel").unwrap_or_else(|| "concentración (M)".to_string());
        let ejes = Ejes::dibujar(
            c,
            EjesConfig {
                x0,
                y0,
                w,
                h,
                xr: (0.0, t_fin),
                yr: (0.0, y_top),
                xlabel: &xlabel,
                ylabel: &ylabel,
                log: false,
                grilla: true,
                origen: false,
                ypaso: None,
            },
        );
        let nombres: Vec<String> = match opt_lista(b, "species", "kinetics.species")? {
            Some(arr) if !arr.is_empty() => arr.iter().map(valor_str).collect(),
            _ => vec!["A".to_string(), "B".to_string(), "C".to_string()],
        };
        for j in 0..3 {
            let pts: Vec<Punto> = ts
                .iter()
                .copied()
                .zip(valores.iter().map(|v| match j {
                    0 => v.0,
                    1 => v.1,
                    _ => v.2,
                }))
                .collect();
            curva_segura(c, &ejes, &pts, j, 1.8, "strokeMain");
        }
        let etiquetas: Vec<String> = nombres.iter().map(|n| format!("[{n}]")).collect();
        leyenda_q(c, x0 + w - 70.0, y0 + 14.0, &etiquetas);
        let tmax = chem::t_maximo_intermediario(k1, k2);
        if tmax.is_finite() && tmax <= t_fin {
            let bmax = chem::consecutiva(k1, k2, a0, tmax).1;
            let (px_t, py_b, py_0) = (ejes.px(tmax), ejes.py(bmax), ejes.py(0.0));
            if bmax.is_finite()
                && px_t.is_finite()
                && py_b.is_finite()
                && py_0.is_finite()
                && (px_t + 6.0).is_finite()
                && (py_b - 8.0).is_finite()
            {
                ejes.guia(c, tmax, 0.0, tmax, bmax, "4 3", "strokeSub");
                c.punto(px_t, py_b, "accentPrimary", 3.0);
                rotulo_q(
                    c,
                    px_t + 6.0,
                    py_b - 8.0,
                    "t máx",
                    Some(&fmt_g(Some(tmax), 3)),
                    blanco,
                    "start",
                    9.5,
                );
            }
        }
        return Ok((x0 + w + 10.0, y0 + h + 40.0));
    }

    if modelo == "reversible" {
        let kf = chem_pos_o_def(b, "kf", "kf", 1.0)?;
        let kb = chem_pos_o_def(b, "kb", "kb", 0.5)?;
        let a0 = chem_pos_o_def(b, "A0", "A0", 1.0)?;
        let k_eq = kf / kb;
        if !(kf + kb).is_finite() || !k_eq.is_finite() {
            return Err(ErrorDiagrama::new(
                "kinetics: constantes kf/kb fuera de rango finito",
            ));
        }
        let t_fin = leer_t_end(b, 5.0 / (kf + kb))?;
        let y_top = a0 * 1.08;
        if !rango_ejes_valido(0.0, t_fin) || !rango_ejes_valido(0.0, y_top) || y_top <= 0.0 {
            return Err(ErrorDiagrama::new("kinetics: parámetros fuera de rango finito"));
        }
        let ts: Vec<f64> = (0..=300).map(|i| t_fin * (i as f64 / 300.0)).collect();
        let valores: Vec<(f64, f64)> = ts.iter().map(|&t| chem::reversible(kf, kb, a0, t)).collect();
        let xlabel = opt_texto(b, "xLabel").unwrap_or_else(|| "t (s)".to_string());
        let ylabel = opt_texto(b, "yLabel").unwrap_or_else(|| "concentración (M)".to_string());
        let ejes = Ejes::dibujar(
            c,
            EjesConfig {
                x0,
                y0,
                w,
                h,
                xr: (0.0, t_fin),
                yr: (0.0, y_top),
                xlabel: &xlabel,
                ylabel: &ylabel,
                log: false,
                grilla: true,
                origen: false,
                ypaso: None,
            },
        );
        let nombres: Vec<String> = match opt_lista(b, "species", "kinetics.species")? {
            Some(arr) if !arr.is_empty() => arr.iter().map(valor_str).collect(),
            _ => vec!["A".to_string(), "B".to_string()],
        };
        for j in 0..2 {
            let pts: Vec<Punto> = ts
                .iter()
                .copied()
                .zip(valores.iter().map(|v| if j == 0 { v.0 } else { v.1 }))
                .collect();
            curva_segura(c, &ejes, &pts, j, 1.8, "strokeMain");
        }
        let etiquetas: Vec<String> = nombres.iter().map(|n| format!("[{n}]")).collect();
        leyenda_q(c, x0 + w - 70.0, y0 + 14.0, &etiquetas);
        let k_eq = kf / kb;
        rotulo_q(
            c,
            x0 + w - 150.0,
            y0 + h - 14.0,
            "K = kf/kb",
            Some(&fmt_g(Some(k_eq), 3)),
            blanco,
            "start",
            10.0,
        );
        return Ok((x0 + w + 10.0, y0 + h + 40.0));
    }

    if modelo != "simple" {
        return Err(ErrorDiagrama::new(format!(
            "kinetics: model '{modelo}' (vale simple, consecutive o reversible)"
        )));
    }

    let raiz_curva;
    let curvas_refs: Vec<&Bloque> = match opt_lista(b, "curves", "kinetics.curves")? {
        Some(arr) if !arr.is_empty() => {
            let mut v = Vec::with_capacity(arr.len());
            for (i, item) in arr.iter().enumerate() {
                v.push(
                    item.as_object()
                        .ok_or_else(|| ErrorDiagrama::new(format!("kinetics.curves[{i}]: un diccionario")))?,
                );
            }
            v
        }
        _ => {
            let mut m = Bloque::new();
            if let Some(v) = b.get("order") {
                m.insert("order".to_string(), v.clone());
            }
            if let Some(v) = b.get("k") {
                m.insert("k".to_string(), v.clone());
            }
            if let Some(v) = b.get("A0") {
                m.insert("A0".to_string(), v.clone());
            }
            raiz_curva = m;
            vec![&raiz_curva]
        }
    };

    let mut datos: Vec<(i32, f64, f64, String)> = Vec::with_capacity(curvas_refs.len());
    for (i, cv) in curvas_refs.iter().enumerate() {
        let orden_i64 = match cv.get("order").filter(|v| !v.is_null()) {
            Some(v) => entero(v, &format!("kinetics.curves[{i}].order"))?,
            None => 1,
        };
        if !(0..=2).contains(&orden_i64) {
            return Err(ErrorDiagrama::new(format!(
                "kinetics: orden {orden_i64} (vale 0, 1 o 2)"
            )));
        }
        let orden = orden_i64 as i32;
        let k = chem_pos_o_def(cv, "k", &format!("curves[{i}].k"), 0.1)?;
        let a0 = chem_pos_o_def(cv, "A0", &format!("curves[{i}].A0"), 1.0)?;
        let label = opt_texto(cv, "label").unwrap_or_else(|| format!("orden {orden}"));
        datos.push((orden, k, a0, label));
    }

    let mut def_t_fin = 0.0_f64;
    for &(orden, k, a0, _) in &datos {
        let cand = if orden != 0 {
            4.0 * envolver_error("kinetics", chem::vida_media(orden, k, a0))?
        } else {
            a0 / k * 1.1
        };
        if cand > def_t_fin {
            def_t_fin = cand;
        }
    }
    let t_fin = leer_t_end(b, def_t_fin)?;
    if !rango_ejes_valido(0.0, t_fin) {
        return Err(ErrorDiagrama::new(
            "kinetics: tEnd resultante fuera de rango finito",
        ));
    }
    let mut ts: Vec<f64> = (0..=300).map(|i| t_fin * (i as f64 / 300.0)).collect();

    if grafico == "linear" {
        if datos.len() != 1 {
            return Err(ErrorDiagrama::new(
                "kinetics: la linealización es de una sola curva",
            ));
        }
        let (orden, k, a0, _) = datos[0];
        let nombre_y = match orden {
            0 => "[A] (M)",
            1 => "ln [A]",
            _ => "1/[A] (M⁻¹)",
        };
        if orden == 0 {
            ts.retain(|&t| t <= a0 / k);
        }
        let mut ys = Vec::with_capacity(ts.len());
        for &t in &ts {
            let v = envolver_error("kinetics", chem::concentracion(orden, k, a0, t))?;
            let y = match orden {
                0 => v,
                1 => {
                    if v > 0.0 {
                        v.ln()
                    } else {
                        f64::NAN
                    }
                }
                _ => {
                    if v > 0.0 {
                        1.0 / v
                    } else {
                        f64::NAN
                    }
                }
            };
            ys.push(y);
        }
        let validos: Vec<f64> = ys.iter().copied().filter(|y| y.is_finite()).collect();
        if validos.is_empty() {
            return Err(ErrorDiagrama::new(
                "kinetics: la linealización no tiene puntos finitos",
            ));
        }
        let lo = validos.iter().copied().fold(f64::INFINITY, f64::min);
        let hi = validos.iter().copied().fold(f64::NEG_INFINITY, f64::max);
        let span = hi - lo;
        if !span.is_finite() {
            return Err(ErrorDiagrama::new(
                "kinetics: rango degenerado en linealización",
            ));
        }
        let margen = if span.abs() == 0.0 { 1.0 } else { span * 0.1 };
        let (y_lo, y_hi) = (lo - margen, hi + margen);
        if !(y_lo < y_hi && rango_ejes_valido(y_lo, y_hi)) {
            return Err(ErrorDiagrama::new(
                "kinetics: rango degenerado en linealización",
            ));
        }
        let xlabel = opt_texto(b, "xLabel").unwrap_or_else(|| "t (s)".to_string());
        let ejes = Ejes::dibujar(
            c,
            EjesConfig {
                x0,
                y0,
                w,
                h,
                xr: (0.0, t_fin),
                yr: (y_lo, y_hi),
                xlabel: &xlabel,
                ylabel: nombre_y,
                log: false,
                grilla: true,
                origen: false,
                ypaso: None,
            },
        );
        let pts: Vec<Punto> = ts.into_iter().zip(ys).collect();
        curva_segura(c, &ejes, &pts, 0, 2.0, "strokeMain");
        let pendiente = if orden == 2 { k } else { -k };
        rotulo_q(
            c,
            x0 + w - 170.0,
            y0 + 18.0,
            "pendiente",
            Some(&fmt_g(Some(pendiente), 3)),
            blanco,
            "start",
            10.0,
        );
        c.t(
            x0 + w - 170.0,
            y0 + 33.0,
            if orden == 2 { "= +k" } else { "= −k" },
            9.0,
            400,
            "textSub",
            "start",
            SANS,
        );
        return Ok((x0 + w + 10.0, y0 + h + 40.0));
    }

    let a_max = datos.iter().map(|d| d.2).fold(0.0_f64, f64::max);
    let y_top = a_max * 1.08;
    if !rango_ejes_valido(0.0, y_top) || y_top <= 0.0 {
        return Err(ErrorDiagrama::new("kinetics: A0 fuera de rango finito"));
    }
    let xlabel = opt_texto(b, "xLabel").unwrap_or_else(|| "t (s)".to_string());
    let ylabel = opt_texto(b, "yLabel").unwrap_or_else(|| "[A] (M)".to_string());
    let ejes = Ejes::dibujar(
        c,
        EjesConfig {
            x0,
            y0,
            w,
            h,
            xr: (0.0, t_fin),
            yr: (0.0, y_top),
            xlabel: &xlabel,
            ylabel: &ylabel,
            log: false,
            grilla: true,
            origen: false,
            ypaso: None,
        },
    );
    for (j, &(orden, k, a0, _)) in datos.iter().enumerate() {
        let mut pts = Vec::with_capacity(ts.len());
        for &t in &ts {
            pts.push((t, envolver_error("kinetics", chem::concentracion(orden, k, a0, t))?));
        }
        curva_segura(c, &ejes, &pts, j, 1.8, "strokeMain");
    }
    if booleano(b, "showProduct", false) && datos.len() == 1 {
        let (orden, k, a0, _) = datos[0];
        let mut pts_p = Vec::with_capacity(ts.len());
        for &t in &ts {
            let ca = envolver_error("kinetics", chem::concentracion(orden, k, a0, t))?;
            pts_p.push((t, a0 - ca));
        }
        curva_segura(c, &ejes, &pts_p, 1, 1.4, "strokeSub");
        leyenda_q(
            c,
            x0 + w - 70.0,
            y0 + 14.0,
            &["[A]".to_string(), "[P]".to_string()],
        );
    } else if datos.len() > 1 {
        let etiqs: Vec<String> = datos.iter().map(|d| d.3.clone()).collect();
        leyenda_q(c, x0 + w - 100.0, y0 + 14.0, &etiqs);
    }
    if pedidas.iter().any(|s| s == "halfLife") && datos.len() == 1 {
        let (orden, k, a0, _) = datos[0];
        let t_m = envolver_error("kinetics", chem::vida_media(orden, k, a0))?;
        let marcas = if orden == 1 { 3 } else { 1 };
        for i in 1..=marcas {
            let t = t_m * i as f64;
            if !t.is_finite() || t > t_fin {
                break;
            }
            let v = envolver_error("kinetics", chem::concentracion(orden, k, a0, t))?;
            let (px_t, py_v, px_0, py_0) = (ejes.px(t), ejes.py(v), ejes.px(0.0), ejes.py(0.0));
            if !(px_t.is_finite() && py_v.is_finite() && px_0.is_finite() && py_0.is_finite()) {
                break;
            }
            ejes.guia(c, 0.0, v, t, v, "2 3", "strokeSub");
            ejes.guia(c, t, 0.0, t, v, "2 3", "strokeSub");
            let lbl = if i > 1 {
                format!("{i}·t½")
            } else {
                "t½".to_string()
            };
            c.t(
                px_t,
                y0 + h - 4.0,
                &lbl,
                9.0,
                700,
                "textMain",
                "middle",
                SANS,
            );
        }
        rotulo_q(
            c,
            x0 + w - 150.0,
            y0 + 60.0,
            "t½",
            Some(&fmt_g(Some(t_m), 3)),
            blanco,
            "start",
            10.0,
        );
    }

    Ok((x0 + w + 10.0, y0 + h + 40.0))
}

// --- 4. Diagramas de fases -------------------------------------------------------------------------------

fn texto_tp(t: f64, p: f64) -> String {
    format!("{} K; {} atm", fmt_g(Some(t), 4), fmt_g(Some(p), 3))
}

fn emitir_polilinea(c: &mut Lienzo, puntos: &[Punto], ancho: f64) -> Result<(), ErrorDiagrama> {
    if puntos.is_empty() {
        return Ok(());
    }
    let mut camino = String::new();
    for (i, &(x, y)) in puntos.iter().enumerate() {
        if !(x.is_finite() && y.is_finite()) {
            return Err(ErrorDiagrama::new(
                "phase_diagram: coordenadas no finitas en curva de equilibrio",
            ));
        }
        if i > 0 {
            camino.push(' ');
        }
        let _ = write!(&mut camino, "{x:.1},{y:.1}");
    }
    c.add(format!(
        r#"<polyline points="{camino}" fill="none" stroke="{}" stroke-width="{}"/>"#,
        c.pal.stroke_main,
        dibu_core::fmt_num_clean(ancho)
    ));
    Ok(())
}

fn recortar_y_emitir_polilinea(
    c: &mut Lienzo,
    puntos: &[Punto],
    ancho: f64,
    x0: f64,
    y0: f64,
    w: f64,
    h: f64,
) -> Result<(), ErrorDiagrama> {
    if puntos.is_empty() {
        return Ok(());
    }
    let (x1, y1) = (x0 + w, y0 + h);
    let recortar = |a: Punto, b: Punto| -> Option<(Punto, Punto)> {
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
    };

    let mut tramos: Vec<Vec<Punto>> = Vec::new();
    let mut previo: Option<Punto> = None;
    for &(x, y) in puntos {
        if !(x.is_finite() && y.is_finite()) {
            return Err(ErrorDiagrama::new(
                "phase_diagram: coordenadas no finitas en curva de equilibrio",
            ));
        }
        let actual = (x, y);
        if let Some(prev) = previo {
            if let Some((ini, fin)) = recortar(prev, actual) {
                let unir = tramos
                    .last()
                    .and_then(|t| t.last())
                    .is_some_and(|&(lx, ly)| (lx - ini.0).hypot(ly - ini.1) < 0.01);
                if unir {
                    if let Some(ultimo) = tramos.last_mut() {
                        ultimo.push(fin);
                    }
                } else {
                    tramos.push(vec![ini, fin]);
                }
            }
        }
        previo = Some(actual);
    }
    for tramo in tramos {
        emitir_polilinea(c, &tramo, ancho)?;
    }
    Ok(())
}

fn render_phase_diagram(c: &mut Lienzo, b: &Bloque) -> Result<Punto, ErrorDiagrama> {
    if opt_texto(b, "kind").as_deref() == Some("eutectic") {
        return render_eutectico(c, b);
    }
    let f = envolver_error("phase_diagram", chem::fases(b))?;
    let top = encabezado_q(c, b);
    let (x0, y0, w, h) = (60.0, top + 16.0, 440.0, 270.0);
    let esquematico = opt_texto(b, "scale").as_deref() != Some("log");
    let (tt, pt) = f.triple;
    let (tc, pc) = f.critico;
    let (t_min, t_max) = (tt * 0.78, tc * 1.12);
    let p_min = f.p_sublimacion(t_min);
    let p_max = pc * 6.0;
    let lmin = p_min.log10();
    let lmax = p_max.log10();

    if !(t_min < t_max
        && p_min > 0.0
        && p_min.is_finite()
        && p_max.is_finite()
        && lmin < lmax
        && rango_ejes_valido(t_min, t_max)
        && rango_ejes_valido(lmin, lmax))
    {
        return Err(ErrorDiagrama::new(
            "phase_diagram: dominio de temperatura o presión inválido o no finito",
        ));
    }

    let ejes_log = if esquematico {
        let trazo = c.pal.stroke_main.clone();
        c.add(format!(
            r#"<line x1="{x0:.1}" y1="{:.1}" x2="{x0:.1}" y2="{:.1}" stroke="{trazo}" stroke-width="1.3" marker-end="url(#flecha-sub-{})"/><line x1="{x0:.1}" y1="{:.1}" x2="{:.1}" y2="{:.1}" stroke="{trazo}" stroke-width="1.3" marker-end="url(#flecha-sub-{})"/>"#,
            y0 + h,
            y0 - 6.0,
            c.sufijo,
            y0 + h,
            x0 + w + 8.0,
            y0 + h,
            c.sufijo
        ));
        c.t(
            x0 + w + 4.0,
            y0 + h + 16.0,
            "T",
            11.0,
            700,
            "textMain",
            "end",
            SANS,
        );
        c.t(x0 - 10.0, y0 + 4.0, "P", 11.0, 700, "textMain", "end", SANS);
        None
    } else {
        Some(Ejes::dibujar(
            c,
            EjesConfig {
                x0,
                y0,
                w,
                h,
                xr: (t_min, t_max),
                yr: (lmin, lmax),
                xlabel: "T (K)",
                ylabel: "log₁₀ P (atm)",
                log: false,
                grilla: true,
                origen: false,
                ypaso: None,
            },
        ))
    };

    let px = |t: f64| -> f64 {
        match &ejes_log {
            Some(e) => e.px(t),
            None => x0 + (t - t_min) / (t_max - t_min) * w,
        }
    };
    let py = |p: f64| -> f64 {
        match &ejes_log {
            Some(e) => e.py(p.log10()),
            None => y0 + h - (p.log10() - lmin) / (lmax - lmin) * h,
        }
    };

    let sub_pts: Vec<Punto> = (0..=60)
        .map(|i| {
            let t = t_min + (tt - t_min) * (i as f64 / 60.0);
            (px(t), py(f.p_sublimacion(t)))
        })
        .collect();
    recortar_y_emitir_polilinea(c, &sub_pts, 2.0, x0, y0, w, h)?;

    let vap_pts: Vec<Punto> = (0..=80)
        .map(|i| {
            let t = tt + (tc - tt) * (i as f64 / 80.0);
            (px(t), py(f.p_vapor(t)))
        })
        .collect();
    recortar_y_emitir_polilinea(c, &vap_pts, 2.0, x0, y0, w, h)?;

    let inclinacion = if f.pendiente_fusion < 0.0 {
        -0.18
    } else {
        0.18
    };
    let (x_tp, y_tp) = (px(tt), py(pt));
    let fusion_pts: Vec<Punto> = if esquematico {
        vec![
            (x_tp + inclinacion * (y_tp - (y0 + 2.0)), y0 + 2.0),
            (x_tp, y_tp),
        ]
    } else {
        (0..=60)
            .map(|i| {
                let p = pt * (p_max / pt).powf(i as f64 / 60.0);
                (px(f.t_fusion(p)), py(p))
            })
            .collect()
    };
    recortar_y_emitir_polilinea(c, &fusion_pts, 2.0, x0, y0, w, h)?;

    let x_fusion = |p: f64| -> f64 {
        if esquematico {
            x_tp + inclinacion * (y_tp - py(p))
        } else {
            px(f.t_fusion(p))
        }
    };

    let blancos = booleano(b, "blankPoints", false);
    for (nombre, (t, p), dx) in [
        ("Punto triple", f.triple, 8.0),
        ("Punto crítico", f.critico, 8.0),
    ] {
        c.punto(px(t), py(p), "strokeMain", 3.6);
        let (x_txt, y_txt) = (px(t) + dx, py(p) + 16.0);
        c.t(x_txt, y_txt, nombre, 9.5, 700, "textMain", "start", SANS);
        if blancos {
            c.vacio(x_txt, x_txt + 110.0, y_txt + 14.0);
        } else {
            c.t(
                x_txt,
                y_txt + 12.0,
                &texto_tp(t, p),
                9.0,
                400,
                "textSub",
                "start",
                SANS,
            );
        }
    }

    if booleano(b, "atm", true) && p_min < 1.0 && 1.0 < p_max {
        let y1 = py(1.0);
        c.linea(x0, y1, x0 + w, y1, "strokeSub", 0.9, None, "5 3");
        c.t(x0 + 4.0, y1 - 4.0, "1 atm", 9.0, 400, "textSub", "start", SANS);
        let mut cortes: Vec<(&str, f64, f64)> = Vec::new();
        if pt < 1.0 {
            let tf = f
                .normales
                .get("normalMelting")
                .copied()
                .unwrap_or_else(|| f.t_fusion(1.0));
            cortes.push(("Tf", tf, x_fusion(1.0)));
        }
        let (tipo_p, tb_calc) = f.t_de_p(1.0);
        let clave_norm = if tipo_p == "ebullición" {
            "normalBoiling"
        } else {
            "normalSublimation"
        };
        let tb = f.normales.get(clave_norm).copied().unwrap_or(tb_calc);
        let nombre_corte = if tipo_p == "ebullición" {
            "Teb"
        } else {
            "Tsub"
        };
        cortes.push((nombre_corte, tb, px(tb)));
        for (nombre, t, x) in cortes {
            if x.is_finite() && t.is_finite() && (x0..=x0 + w).contains(&x) {
                c.punto(x, y1, "accentPrimary", 2.8);
                c.linea(x, y1, x, y0 + h, "strokeMuted", 0.7, None, "2 3");
                rotulo_q(
                    c,
                    x + 3.0,
                    y0 + h - 6.0,
                    nombre,
                    Some(&format!("{} K", fmt_g(Some(t), 4))),
                    blancos,
                    "start",
                    9.0,
                );
            }
        }
    }

    let p_de_y = |y: f64| -> f64 { 10_f64.powf(lmin + (y0 + h - y) / h * (lmax - lmin)) };
    let y_solido = y0 + h * 0.3;
    let x_fusion_solido = x_fusion(p_de_y(y_solido)).clamp(x0, x0 + w);
    let x_solido = (x0 + 30.0).max((x0 + x_fusion_solido) / 2.0);
    let x_liq_izq = x_fusion(pc).clamp(x0, x0 + w);
    let mut regiones = vec![
        ("Sólido", x_solido, y_solido),
        ("Líquido", (x_liq_izq + px(tc)) / 2.0 + 14.0, y0 + h * 0.2),
        ("Gas", px(tt + (tc - tt) * 0.7), y0 + h * 0.85),
    ];
    if booleano(b, "supercritical", true) {
        regiones.push(("Fluido supercrítico", px(tc) + 6.0, py(pc) - 12.0));
    }
    let blank_reg = booleano(b, "blankRegions", false);
    for (nombre, x, y) in regiones {
        if !(x.is_finite() && y.is_finite()) {
            continue;
        }
        if blank_reg {
            caja_vacia(c, x, y - 4.0, 64.0, 16.0);
        } else {
            let (size, ancla) = if nombre != "Fluido supercrítico" {
                (11.0, "middle")
            } else {
                (9.0, "start")
            };
            c.t(x, y, nombre, size, 700, "textSub", ancla, SANS);
        }
    }

    if let Some(pts) = opt_lista(b, "points", "phase_diagram.points")? {
        for (i, punto_val) in pts.iter().enumerate() {
            let err_punto = || {
                ErrorDiagrama::new(format!("phase_diagram.points[{i}]: {{T, P, label}}"))
            };
            let punto = punto_val.as_object().ok_or_else(err_punto)?;
            let t_val = punto.get("T").ok_or_else(err_punto)?;
            let p_val = punto.get("P").ok_or_else(err_punto)?;
            let t = num(t_val, "T").map_err(|_| err_punto())?;
            let p = num(p_val, "P").map_err(|_| err_punto())?;
            if p <= 0.0 {
                return Err(ErrorDiagrama::new(format!(
                    "phase_diagram.points[{i}]: P tiene que ser positivo"
                )));
            }
            let (x, y) = (px(t), py(p));
            if !x.is_finite() || !y.is_finite() || x.abs() > 1e5 || y.abs() > 1e5 {
                return Err(ErrorDiagrama::new(format!(
                    "phase_diagram.points[{i}]: punto ({t}, {p}) fuera de rango visible"
                )));
            }
            if x >= x0 - 2.0 && x <= x0 + w + 2.0 && y >= y0 - 2.0 && y <= y0 + h + 2.0 {
                c.add(format!(
                    r#"<circle cx="{x:.1}" cy="{y:.1}" r="4" fill="{}" stroke="{}" stroke-width="1.6"/>"#,
                    c.pal.fill_cell, c.pal.stroke_main
                ));
                let label = punto
                    .get("label")
                    .filter(|v| !v.is_null())
                    .map(valor_str)
                    .unwrap_or_else(|| (i + 1).to_string());
                c.t(
                    x + 7.0,
                    y - 6.0,
                    &label,
                    10.0,
                    700,
                    "textMain",
                    "start",
                    SANS,
                );
            }
        }
    }

    if !f.nombre.is_empty() {
        tq(
            c,
            x0 + w - 4.0,
            y0 + h - 8.0,
            &f.nombre,
            11.0,
            700,
            "textMain",
            "end",
            SANS,
            false,
        );
    }

    Ok((x0 + w + 130.0, y0 + h + 40.0))
}

fn render_eutectico(c: &mut Lienzo, b: &Bloque) -> Result<Punto, ErrorDiagrama> {
    let err_falta = || {
        ErrorDiagrama::new(
            "phase_diagram: el eutéctico necesita meltingA, meltingB y eutectic: [x %, T]",
        )
    };
    let ta = num(b.get("meltingA").ok_or_else(err_falta)?, "meltingA")
        .map_err(|_| err_falta())?;
    let tb = num(b.get("meltingB").ok_or_else(err_falta)?, "meltingB")
        .map_err(|_| err_falta())?;
    let eut_val = b.get("eutectic").ok_or_else(err_falta)?;
    let (xe, te) = extraer_rango(eut_val, "eutectic").map_err(|_| err_falta())?;

    if !(0.0 < xe && xe < 100.0 && te < ta.min(tb)) {
        return Err(ErrorDiagrama::new(
            "phase_diagram: el eutéctico está entre 0 y 100 % y por debajo de los dos puntos de fusión",
        ));
    }

    let (a, z) = match opt_lista(b, "components", "phase_diagram.components")? {
        Some(arr) if !arr.is_empty() => {
            if arr.len() != 2 {
                return Err(ErrorDiagrama::new(
                    "phase_diagram.components: tiene que tener exactamente 2 componentes [A, B]",
                ));
            }
            (valor_str(&arr[0]), valor_str(&arr[1]))
        }
        _ => ("A".to_string(), "B".to_string()),
    };

    let t_max_ab = ta.max(tb);
    let t_lo = te - (t_max_ab - te) * 0.35;
    let t_hi = t_max_ab + (t_max_ab - te) * 0.12;
    if !(t_lo < t_hi && rango_ejes_valido(t_lo, t_hi)) {
        return Err(ErrorDiagrama::new(
            "phase_diagram: rango de temperatura eutéctica no finito",
        ));
    }

    let top = encabezado_q(c, b);
    let (x0, y0, w, h) = (64.0, top + 16.0, 420.0, 260.0);
    let y_label = opt_texto(b, "yLabel").unwrap_or_else(|| "T (°C)".to_string());
    let ejes = Ejes::dibujar(
        c,
        EjesConfig {
            x0,
            y0,
            w,
            h,
            xr: (0.0, 100.0),
            yr: (t_lo, t_hi),
            xlabel: &format!("% {z} (en masa)"),
            ylabel: &y_label,
            log: false,
            grilla: true,
            origen: false,
            ypaso: None,
        },
    );

    let liquidus = |x: f64| -> f64 {
        if x <= xe {
            let s = x / xe;
            ta + (te - ta) * (s * 0.65 + s * s * 0.35)
        } else {
            let s = (100.0 - x) / (100.0 - xe);
            tb + (te - tb) * (s * 0.65 + s * s * 0.35)
        }
    };

    let liq_pts: Vec<Punto> = (0..=200)
        .map(|i| {
            let x = i as f64 / 2.0;
            (x, liquidus(x))
        })
        .collect();
    curva_segura(c, &ejes, &liq_pts, 0, 2.0, "strokeMain");
    curva_segura(c, &ejes, &[(0.0, te), (100.0, te)], 0, 2.0, "strokeMain");
    curva_segura(c, &ejes, &[(0.0, t_lo), (0.0, ta)], 0, 2.0, "strokeMain");
    curva_segura(c, &ejes, &[(100.0, t_lo), (100.0, tb)], 0, 2.0, "strokeMain");

    c.punto(ejes.px(xe), ejes.py(te), "strokeMain", 3.6);
    let blancos = booleano(b, "blankPoints", false);
    rotulo_q(
        c,
        ejes.px(xe) + 6.0,
        ejes.py(te) + 16.0,
        "E",
        Some(&format!(
            "{} %, {} °C",
            fmt_g(Some(xe), 3),
            fmt_g(Some(te), 3)
        )),
        blancos,
        "start",
        9.5,
    );

    let regiones = [
        (
            "L".to_string(),
            50.0,
            (t_hi + liquidus(50.0).max(te)) / 2.0 + (t_hi - te) * 0.12,
        ),
        (
            format!("L + {a}(s)"),
            xe * 0.38,
            (te + liquidus(xe * 0.5)) / 2.0,
        ),
        (
            format!("L + {z}(s)"),
            xe + (100.0 - xe) * 0.62,
            (te + liquidus(xe + (100.0 - xe) * 0.5)) / 2.0,
        ),
        (format!("{a}(s) + {z}(s)"), 50.0, (te + t_lo) / 2.0),
    ];
    let blank_reg = booleano(b, "blankRegions", false);
    for (nombre, x, t) in regiones {
        let (rx, ry) = (ejes.px(x), ejes.py(t));
        if !(rx.is_finite() && ry.is_finite()) {
            continue;
        }
        if blank_reg {
            caja_vacia(c, rx, ry - 4.0, 64.0, 16.0);
        } else {
            tq(
                c,
                rx,
                ry,
                &nombre,
                10.5,
                700,
                "textSub",
                "middle",
                SANS,
                false,
            );
        }
    }

    if let Some(x) = opt_num(b, "cooling", "phase_diagram.cooling")? {
        if !(0.0..=100.0).contains(&x) {
            return Err(ErrorDiagrama::new(
                "phase_diagram.cooling: la composición tiene que estar entre 0 y 100 %",
            ));
        }
        let tl = liquidus(x);
        let px_x = ejes.px(x);
        let py_tl = ejes.py(tl);
        let py_te = ejes.py(te);
        let py_thi = ejes.py(t_hi);
        let py_tlo = ejes.py(t_lo);
        if !(tl.is_finite()
            && px_x.is_finite()
            && py_tl.is_finite()
            && py_te.is_finite()
            && py_thi.is_finite()
            && py_tlo.is_finite()
            && (px_x + 5.0).is_finite()
            && (py_tl - 6.0).is_finite())
        {
            return Err(ErrorDiagrama::new(
                "phase_diagram: cooling produce una temperatura o coordenada no finita",
            ));
        }
        ejes.guia(c, x, t_hi, x, t_lo, "5 3", "strokeSub");
        c.punto(px_x, py_tl, "accentPrimary", 3.0);
        c.punto(px_x, py_te, "accentPrimary", 3.0);
        rotulo_q(
            c,
            px_x + 5.0,
            py_tl - 6.0,
            "T_liq",
            Some(&format!("{} °C", fmt_g(Some(tl), 3))),
            booleano(b, "blankCooling", false),
            "start",
            9.0,
        );
    }

    Ok((x0 + w + 10.0, y0 + h + 40.0))
}

// --- 5. Curva de calentamiento ---------------------------------------------------------------------------

fn render_heating_curve(c: &mut Lienzo, b: &Bloque) -> Result<Punto, ErrorDiagrama> {
    let tramos = envolver_error("heating_curve", chem::calentamiento(b))?;
    let (Some(primero), Some(ultimo)) = (tramos.first(), tramos.last()) else {
        return Err(ErrorDiagrama::new("heating_curve: sin tramos para dibujar"));
    };
    let q_total = ultimo.q1;
    let (t_lo, t_hi) = (primero.t0, ultimo.t1);
    let t_span = t_hi - t_lo;
    if !(q_total > 0.0 && rango_ejes_valido(0.0, q_total * 1.04)) {
        return Err(ErrorDiagrama::new(
            "heating_curve: el calor total Q tiene que ser positivo y finito",
        ));
    }
    if !(t_span > 0.0 && t_span.is_finite()) {
        return Err(ErrorDiagrama::new(
            "heating_curve: rango de temperatura inválido o no finito",
        ));
    }
    let margen = t_span * 0.1;
    let (y_lo, y_hi) = (t_lo - margen, t_hi + margen);
    if !(y_lo < y_hi && rango_ejes_valido(y_lo, y_hi)) {
        return Err(ErrorDiagrama::new(
            "heating_curve: rango de temperatura fuera de rango finito",
        ));
    }

    let top = encabezado_q(c, b);
    let (x0, y0, w, h) = (60.0, top + 16.0, 470.0, 250.0);
    let ejes = Ejes::dibujar(
        c,
        EjesConfig {
            x0,
            y0,
            w,
            h,
            xr: (0.0, q_total * 1.04),
            yr: (y_lo, y_hi),
            xlabel: "Q (kJ)",
            ylabel: "T (°C)",
            log: false,
            grilla: true,
            origen: false,
            ypaso: None,
        },
    );

    let mut puntos = Vec::with_capacity(tramos.len() + 1);
    puntos.push((primero.q0, primero.t0));
    for t in &tramos {
        puntos.push((t.q1, t.t1));
    }
    for &(q, t) in &puntos {
        if !(ejes.px(q).is_finite() && ejes.py(t).is_finite()) {
            return Err(ErrorDiagrama::new(
                "heating_curve: coordenadas de tramos fuera de rango finito",
            ));
        }
    }
    curva_segura(c, &ejes, &puntos, 0, 2.2, "strokeMain");

    let blanco = booleano(b, "blankHeat", false);
    let blank_phases = booleano(b, "blankPhases", false);
    let show_heat = booleano(b, "showHeat", true);

    for t in &tramos {
        let xm = ejes.px(t.q0 + (t.q1 - t.q0) / 2.0);
        let ym = ejes.py(t.t0 + (t.t1 - t.t0) / 2.0);
        if !(xm.is_finite() && ym.is_finite()) {
            continue;
        }
        let cambio = t.t0 == t.t1;
        let dy = if cambio { -22.0 } else { 16.0 };
        let mut ancla = if cambio { "middle" } else { "start" };
        let mut x_txt = if cambio { xm } else { xm + 10.0 };
        if !cambio && x_txt + 70.0 > x0 + w {
            ancla = "end";
            x_txt = xm - 8.0;
        }
        if cambio {
            x_txt = x_txt.clamp(x0 + 34.0, x0 + w - 34.0);
        }
        if blank_phases {
            let centro = match ancla {
                "start" => x_txt + 29.0,
                "end" => x_txt - 29.0,
                _ => x_txt,
            };
            caja_vacia(c, centro.max(x0 + 31.0), ym + dy - 4.0, 58.0, 13.0);
        } else {
            tq(
                c,
                x_txt,
                ym + dy,
                t.fase,
                9.5,
                700,
                "textSub",
                ancla,
                SANS,
                true,
            );
        }
        if show_heat {
            let y_q = ym + dy + 12.0;
            if blanco {
                let inicio = match ancla {
                    "start" => x_txt,
                    "end" => x_txt - 44.0,
                    _ => x_txt - 22.0,
                };
                let ix = inicio.max(x0 + 4.0);
                c.vacio(ix, ix + 44.0, y_q + 2.0);
            } else {
                tq(
                    c,
                    x_txt,
                    y_q,
                    &format!("{} kJ", fmt_g(Some(t.q()), 3)),
                    9.0,
                    400,
                    "textSub",
                    ancla,
                    SANS,
                    true,
                );
            }
        }
    }

    let sub_clave = opt_texto(b, "substance")
        .unwrap_or_else(|| "water".to_string())
        .to_lowercase();
    let base = chem::sustancia_calentamiento(&sub_clave);
    for (nombre, clave, base_val) in [
        ("Tf", "melting", base.map(|d| d.melting)),
        ("Teb", "boiling", base.map(|d| d.boiling)),
    ] {
        let val_opt = opt_num(b, clave, &format!("heating_curve.{clave}"))?.or(base_val);
        if let Some(tv) = val_opt {
            if t_lo <= tv && tv <= t_hi {
                let py_tv = ejes.py(tv);
                if py_tv.is_finite()
                    && ejes.px(0.0).is_finite()
                    && ejes.px(q_total * 1.04).is_finite()
                {
                    ejes.guia(c, 0.0, tv, q_total * 1.04, tv, "2 4", "strokeSub");
                    let arriba = clave == "melting";
                    c.t(
                        if arriba { x0 + w - 4.0 } else { x0 + 4.0 },
                        py_tv + if arriba { -3.0 } else { 11.0 },
                        &format!("{nombre} = {} °C", fmt_g(Some(tv), 3)),
                        8.5,
                        400,
                        "textSub",
                        if arriba { "end" } else { "start" },
                        SANS,
                    );
                }
            }
        }
    }

    if booleano(b, "showTotal", true) {
        rotulo_q(
            c,
            x0 + w - 150.0,
            y0 + h - 12.0,
            "Q total",
            Some(&format!("{} kJ", fmt_g(Some(q_total), 4))),
            blanco,
            "start",
            10.0,
        );
    }

    Ok((x0 + w + 10.0, y0 + h + 40.0))
}

// --- 6. Espectros ----------------------------------------------------------------------------------------

fn leer_picos<'a>(b: &'a Bloque, clave: &str) -> Result<Vec<&'a Bloque>, ErrorDiagrama> {
    let val = b
        .get(clave)
        .filter(|v| !v.is_null() && !matches!(v, Value::Array(a) if a.is_empty()))
        .or_else(|| {
            b.get("signals")
                .filter(|v| !v.is_null() && !matches!(v, Value::Array(a) if a.is_empty()))
        });
    let Some(Value::Array(arr)) = val else {
        return Err(ErrorDiagrama::new(format!(
            "spectrum: falta {clave} (una lista)"
        )));
    };
    if arr.is_empty() {
        return Err(ErrorDiagrama::new(format!(
            "spectrum: falta {clave} (una lista)"
        )));
    }
    let mut salida = Vec::with_capacity(arr.len());
    for (i, item) in arr.iter().enumerate() {
        let obj = item.as_object().ok_or_else(|| {
            ErrorDiagrama::new(format!("spectrum.{clave}[{i}]: un diccionario"))
        })?;
        salida.push(obj);
    }
    Ok(salida)
}

struct Rotulador {
    y0: f64,
    h: f64,
    blanco: bool,
    puestos: Vec<(f64, f64, f64)>,
}

impl Rotulador {
    fn new(y0: f64, h: f64, blanco: bool) -> Self {
        Self {
            y0,
            h,
            blanco,
            puestos: Vec::new(),
        }
    }

    fn rotular(&mut self, c: &mut Lienzo, x: f64, y_raw: f64, texto: &str, ancla: &str) {
        if texto.is_empty() || !x.is_finite() || !y_raw.is_finite() {
            return;
        }
        let ancho = if self.blanco {
            40.0
        } else {
            ancho_formula(texto, 9.0)
        };
        let mut y = y_raw.clamp(self.y0 + 10.0, self.y0 + self.h - 4.0);
        let mut iter = 0;
        while iter < 50
            && self.puestos.iter().any(|&(a, z, yy)| {
                a < x + ancho / 2.0 + 3.0
                    && x - ancho / 2.0 - 3.0 < z
                    && (yy - y).abs() < 11.0
            })
        {
            y -= 12.0;
            iter += 1;
        }
        self.puestos.push((x - ancho / 2.0, x + ancho / 2.0, y));
        if self.blanco {
            caja_vacia(c, x, y - 4.0, 40.0, 13.0);
        } else {
            tq(c, x, y, texto, 9.0, 400, "textSub", ancla, SANS, true);
        }
    }
}

fn render_spectrum(c: &mut Lienzo, b: &Bloque) -> Result<Punto, ErrorDiagrama> {
    let tipo = opt_texto(b, "kind").unwrap_or_else(|| "ir".to_string());
    let top = encabezado_q(c, b);
    let (x0, y0, w, h) = (58.0, top + 22.0, 480.0, 220.0);
    let blanco = booleano(b, "blankLabels", false);
    let mut rot = Rotulador::new(y0, h, blanco);

    match tipo.as_str() {
        "ir" => {
            let picos_raw = leer_picos(b, "peaks")?;
            let (r0, r1) = match b.get("range").filter(|v| !v.is_null()) {
                Some(v) => extraer_rango(v, "spectrum.range")?,
                None => (4000.0, 400.0),
            };
            if r0 == r1 {
                return Err(ErrorDiagrama::new(
                    "spectrum: range tiene que tener extremos distintos",
                ));
            }
            if !rango_ejes_valido(r0, r1) {
                return Err(ErrorDiagrama::new(
                    "spectrum: range inválido o fuera de rango finito",
                ));
            }
            struct PicoIr {
                at: f64,
                ancho: f64,
                prof: f64,
                label: String,
            }
            let mut picos = Vec::with_capacity(picos_raw.len());
            for (i, p) in picos_raw.iter().enumerate() {
                let at_val = p.get("at").ok_or_else(|| {
                    ErrorDiagrama::new(format!("spectrum.peaks[{i}]: falta 'at'"))
                })?;
                let at = num(at_val, &format!("spectrum.peaks[{i}].at"))?;
                let def_ancho = if booleano(p, "broad", false) {
                    120.0
                } else {
                    22.0
                };
                let ancho = opt_num(p, "width", &format!("spectrum.peaks[{i}].width"))?
                    .unwrap_or(def_ancho);
                if ancho <= 0.0 {
                    return Err(ErrorDiagrama::new(format!(
                        "spectrum.peaks[{i}].width: tiene que ser positivo"
                    )));
                }
                let prof =
                    opt_num(p, "depth", &format!("spectrum.peaks[{i}].depth"))?.unwrap_or(0.7);
                let label = p
                    .get("label")
                    .filter(|v| !v.is_null())
                    .map(valor_str)
                    .unwrap_or_default();
                picos.push(PicoIr {
                    at,
                    ancho,
                    prof,
                    label,
                });
            }

            let ejes = Ejes::dibujar(
                c,
                EjesConfig {
                    x0,
                    y0,
                    w,
                    h,
                    xr: (r0, r1),
                    yr: (0.0, 100.0),
                    xlabel: "número de onda (cm⁻¹)",
                    ylabel: "transmitancia (%)",
                    log: false,
                    grilla: true,
                    origen: false,
                    ypaso: Some(20.0),
                },
            );
            let transmitancia = |nu: f64| -> f64 {
                let mut total = 0.0;
                for p in &picos {
                    total += p.prof / (1.0 + ((nu - p.at) / (p.ancho / 2.0)).powi(2));
                }
                (100.0 * (1.0 - total.min(0.96)) - 4.0).max(2.0)
            };
            let pts: Vec<Punto> = (0..=900)
                .map(|i| {
                    let nu = r0 + (r1 - r0) * (i as f64 / 900.0);
                    (nu, transmitancia(nu))
                })
                .collect();
            curva_segura(c, &ejes, &pts, 0, 1.5, "strokeMain");
            for p in &picos {
                let px = ejes.px(p.at);
                let py = ejes.py(transmitancia(p.at));
                if px.is_finite() && py.is_finite() && px >= x0 - 2.0 && px <= x0 + w + 2.0 {
                    rot.rotular(c, px, py + 14.0, &p.label, "middle");
                }
            }
            Ok((x0 + w + 10.0, y0 + h + 40.0))
        }
        "nmr" | "c13" => {
            let es_nmr = tipo == "nmr";
            let picos_raw = leer_picos(b, "signals")?;
            let mhz = opt_num(b, "MHz", "spectrum.MHz")?.unwrap_or(400.0);
            if mhz <= 0.0 {
                return Err(ErrorDiagrama::new(
                    "spectrum: MHz tiene que ser positivo",
                ));
            }
            let separacion = opt_num(b, "lineSpacing", "spectrum.lineSpacing")?.unwrap_or(5.0);
            if separacion < 0.0 {
                return Err(ErrorDiagrama::new(
                    "spectrum: lineSpacing no puede ser negativo",
                ));
            }

            struct Signal {
                shift: f64,
                intensity: f64,
                vecinos: usize,
                j_hz: f64,
                integracion: f64,
                label: String,
            }
            let mut signals = Vec::with_capacity(picos_raw.len());
            for (i, p) in picos_raw.iter().enumerate() {
                let shift_val = p.get("shift").ok_or_else(|| {
                    ErrorDiagrama::new(format!("spectrum.signals[{i}]: falta 'shift'"))
                })?;
                let shift = num(shift_val, &format!("spectrum.signals[{i}].shift"))?;
                let label = p
                    .get("label")
                    .filter(|v| !v.is_null())
                    .map(valor_str)
                    .unwrap_or_default();
                if !es_nmr {
                    let intensity =
                        opt_num(p, "intensity", &format!("spectrum.signals[{i}].intensity"))?
                            .unwrap_or(0.8);
                    if intensity < 0.0 {
                        return Err(ErrorDiagrama::new(format!(
                            "spectrum.signals[{i}].intensity: no puede ser negativo"
                        )));
                    }
                    signals.push(Signal {
                        shift,
                        intensity,
                        vecinos: 0,
                        j_hz: 0.0,
                        integracion: 1.0,
                        label,
                    });
                    continue;
                }
                let vecinos = match p.get("neighbors").filter(|v| !v.is_null()) {
                    Some(v) => {
                        let n = entero(v, &format!("spectrum.signals[{i}].neighbors"))?;
                        if !(0..=MAX_VECINOS_NMR).contains(&n) {
                            return Err(ErrorDiagrama::new(format!(
                                "spectrum: neighbors ({n}) tiene que estar entre 0 y {MAX_VECINOS_NMR}"
                            )));
                        }
                        n as usize
                    }
                    None => {
                        let mult = p
                            .get("multiplicity")
                            .filter(|v| !v.is_null())
                            .map(valor_str)
                            .unwrap_or_else(|| "s".to_string());
                        if mult == "m" {
                            4
                        } else if let Some(n) = chem::multiplicidad(&mult) {
                            n
                        } else {
                            return Err(ErrorDiagrama::new(format!(
                                "spectrum: multiplicidad '{mult}' (s, d, t, q, quint, sext, sept, m)"
                            )));
                        }
                    }
                };
                let j_hz = opt_num(p, "J", &format!("spectrum.signals[{i}].J"))?.unwrap_or(7.0);
                let integracion =
                    opt_num(p, "integration", &format!("spectrum.signals[{i}].integration"))?
                        .unwrap_or(1.0);
                if integracion < 0.0 {
                    return Err(ErrorDiagrama::new(format!(
                        "spectrum.signals[{i}].integration: no puede ser negativo"
                    )));
                }
                signals.push(Signal {
                    shift,
                    intensity: 0.0,
                    vecinos,
                    j_hz,
                    integracion,
                    label,
                });
            }

            let maximo: f64 = if es_nmr { 12.0 } else { 220.0 };
            let max_shift = signals
                .iter()
                .map(|s| s.shift)
                .fold(f64::NEG_INFINITY, f64::max);
            let hi = maximo.max(max_shift + 1.0);
            if !rango_ejes_valido(hi, 0.0) || hi <= 0.0 {
                return Err(ErrorDiagrama::new(
                    "spectrum: rango de desplazamiento δ inválido",
                ));
            }

            let mut lineas: Vec<(f64, f64, usize)> = Vec::new();
            for (idx, sig) in signals.iter().enumerate() {
                if !es_nmr {
                    lineas.push((sig.shift, sig.intensity, idx));
                } else {
                    for (delta_l, rel) in
                        chem::multiplete(sig.shift, sig.vecinos, sig.j_hz, mhz)
                    {
                        lineas.push((delta_l, rel * sig.integracion, idx));
                    }
                }
            }

            let tope = lineas
                .iter()
                .map(|l| l.1)
                .fold(f64::NEG_INFINITY, f64::max);
            if es_nmr && (!tope.is_finite() || tope <= 0.0) {
                return Err(ErrorDiagrama::new(
                    "spectrum: la intensidad máxima de RMN tiene que ser positiva",
                ));
            }

            let ejes = Ejes::dibujar(
                c,
                EjesConfig {
                    x0,
                    y0,
                    w,
                    h,
                    xr: (hi, 0.0),
                    yr: (0.0, 1.15),
                    xlabel: "δ (ppm)",
                    ylabel: "",
                    log: false,
                    grilla: false,
                    origen: false,
                    ypaso: None,
                },
            );

            for &(delta, v, idx) in &lineas {
                let altura = if es_nmr {
                    0.95 * v / tope
                } else {
                    v.min(1.0)
                };
                let mut x = ejes.px(delta);
                if es_nmr {
                    let centro = ejes.px(signals[idx].shift);
                    let real = x - centro;
                    let paso_real = (ejes.px(0.0) - ejes.px(signals[idx].j_hz / mhz)).abs();
                    if 0.0 < paso_real && paso_real < separacion {
                        x = centro + real * separacion / paso_real;
                    }
                }
                let y_top = ejes.py(altura).clamp(y0, y0 + h);
                let y_base = ejes.py(0.0).clamp(y0, y0 + h);
                if x.is_finite()
                    && y_top.is_finite()
                    && y_base.is_finite()
                    && x >= x0 - 2.0
                    && x <= x0 + w + 2.0
                {
                    c.linea(
                        x,
                        y_base,
                        x,
                        y_top,
                        "strokeMain",
                        1.3,
                        None,
                        "",
                    );
                }
            }

            let mostrar_integ = booleano(b, "showIntegration", true);
            for (idx, sig) in signals.iter().enumerate() {
                let x_sig = ejes.px(sig.shift);
                if !x_sig.is_finite() || x_sig < x0 - 2.0 || x_sig > x0 + w + 2.0 {
                    continue;
                }
                let propias_max = lineas
                    .iter()
                    .filter(|l| l.2 == idx)
                    .map(|l| l.1)
                    .fold(f64::NEG_INFINITY, f64::max);
                let altura = if es_nmr {
                    0.95 * propias_max / tope
                } else {
                    sig.intensity.min(1.0)
                };
                let mut texto = sig.label.clone();
                if es_nmr && mostrar_integ {
                    if !texto.is_empty() {
                        texto.push(' ');
                    }
                    let _ = write!(&mut texto, "{}H", fmt_g(Some(sig.integracion), 3));
                }
                rot.rotular(
                    c,
                    x_sig,
                    ejes.py(altura).clamp(y0, y0 + h) - 6.0,
                    &texto,
                    "middle",
                );
            }

            if booleano(b, "tms", false) {
                let (x_tms, y_tms0, y_tms1) = (ejes.px(0.0), ejes.py(0.0), ejes.py(0.35));
                if x_tms.is_finite() && y_tms0.is_finite() && y_tms1.is_finite() {
                    c.linea(
                        x_tms,
                        y_tms0,
                        x_tms,
                        y_tms1,
                        "strokeSub",
                        1.2,
                        None,
                        "",
                    );
                    c.t(
                        x_tms,
                        y_tms1 - 4.0,
                        "TMS",
                        8.5,
                        400,
                        "textSub",
                        "middle",
                        SANS,
                    );
                }
            }

            let leyenda = if es_nmr {
                format!("RMN de ¹H, {} MHz", fmt_g(Some(mhz), 3))
            } else {
                "RMN de ¹³C".to_string()
            };
            c.t(
                x0 + 6.0,
                y0 + 14.0,
                &leyenda,
                9.5,
                400,
                "textSub",
                "start",
                SANS,
            );
            Ok((x0 + w + 10.0, y0 + h + 40.0))
        }
        "uv" => {
            let bandas_raw = leer_picos(b, "bands")?;
            let (r0, r1) = match b.get("range").filter(|v| !v.is_null()) {
                Some(v) => extraer_rango(v, "spectrum.range")?,
                None => (200.0, 800.0),
            };
            if r0 == r1 {
                return Err(ErrorDiagrama::new(
                    "spectrum: range tiene que tener extremos distintos",
                ));
            }
            if !rango_ejes_valido(r0, r1) {
                return Err(ErrorDiagrama::new(
                    "spectrum: range inválido o fuera de rango finito",
                ));
            }
            struct BandaUv {
                at: f64,
                absorbance: f64,
                width: f64,
                label: String,
            }
            let mut bandas = Vec::with_capacity(bandas_raw.len());
            for (i, p) in bandas_raw.iter().enumerate() {
                let at_val = p.get("at").ok_or_else(|| {
                    ErrorDiagrama::new(format!("spectrum.bands[{i}]: falta 'at'"))
                })?;
                let at = num(at_val, &format!("spectrum.bands[{i}].at"))?;
                let absorbance =
                    opt_num(p, "absorbance", &format!("spectrum.bands[{i}].absorbance"))?
                        .unwrap_or(1.0);
                let width =
                    opt_num(p, "width", &format!("spectrum.bands[{i}].width"))?.unwrap_or(25.0);
                if width <= 0.0 {
                    return Err(ErrorDiagrama::new(format!(
                        "spectrum.bands[{i}].width: tiene que ser positivo"
                    )));
                }
                let label = match p.get("label") {
                    None => format!("λmáx = {} nm", fmt_g(Some(at), 3)),
                    Some(Value::Null) => String::new(),
                    Some(v) => valor_str(v),
                };
                bandas.push(BandaUv {
                    at,
                    absorbance,
                    width,
                    label,
                });
            }

            let a_max = bandas
                .iter()
                .map(|p| p.absorbance)
                .fold(f64::NEG_INFINITY, f64::max)
                * 1.15;
            if !rango_ejes_valido(0.0, a_max) || a_max <= 0.0 {
                return Err(ErrorDiagrama::new(
                    "spectrum: la absorbancia máxima tiene que ser positiva",
                ));
            }
            let ejes = Ejes::dibujar(
                c,
                EjesConfig {
                    x0,
                    y0,
                    w,
                    h,
                    xr: (r0, r1),
                    yr: (0.0, a_max),
                    xlabel: "λ (nm)",
                    ylabel: "absorbancia",
                    log: false,
                    grilla: true,
                    origen: false,
                    ypaso: None,
                },
            );
            let absorbancia = |lam: f64| -> f64 {
                bandas
                    .iter()
                    .map(|p| p.absorbance * (-((lam - p.at) / p.width).powi(2)).exp())
                    .sum()
            };
            let pts: Vec<Punto> = (0..=600)
                .map(|i| {
                    let lam = r0 + (r1 - r0) * (i as f64 / 600.0);
                    (lam, absorbancia(lam))
                })
                .collect();
            curva_segura(c, &ejes, &pts, 0, 1.8, "strokeMain");
            for p in &bandas {
                let px = ejes.px(p.at);
                let py = ejes.py(absorbancia(p.at));
                if px.is_finite() && py.is_finite() && px >= x0 - 2.0 && px <= x0 + w + 2.0 {
                    rot.rotular(c, px, py - 6.0, &p.label, "middle");
                }
            }
            Ok((x0 + w + 10.0, y0 + h + 40.0))
        }
        "ms" => {
            let picos_raw = leer_picos(b, "peaks")?;
            struct PicoMs {
                mz: f64,
                intensity: f64,
                label: Option<String>,
            }
            let mut picos = Vec::with_capacity(picos_raw.len());
            for (i, p) in picos_raw.iter().enumerate() {
                let mz_val = p.get("mz").ok_or_else(|| {
                    ErrorDiagrama::new(format!("spectrum.peaks[{i}]: falta 'mz'"))
                })?;
                let mz = num(mz_val, &format!("spectrum.peaks[{i}].mz"))?;
                if mz < 0.0 {
                    return Err(ErrorDiagrama::new(format!(
                        "spectrum.peaks[{i}].mz: no puede ser negativo"
                    )));
                }
                let intensity =
                    opt_num(p, "intensity", &format!("spectrum.peaks[{i}].intensity"))?
                        .unwrap_or(100.0);
                if intensity < 0.0 {
                    return Err(ErrorDiagrama::new(format!(
                        "spectrum.peaks[{i}].intensity: no puede ser negativo"
                    )));
                }
                let label = p
                    .get("label")
                    .filter(|v| !v.is_null())
                    .map(valor_str)
                    .filter(|s| !s.is_empty());
                picos.push(PicoMs {
                    mz,
                    intensity,
                    label,
                });
            }

            let max_mz = picos
                .iter()
                .map(|p| p.mz)
                .fold(f64::NEG_INFINITY, f64::max);
            if !rango_ejes_valido(0.0, max_mz * 1.1) || max_mz <= 0.0 {
                return Err(ErrorDiagrama::new(
                    "spectrum: el máximo m/z tiene que ser positivo y finito",
                ));
            }
            let tope = picos
                .iter()
                .map(|p| p.intensity)
                .fold(f64::NEG_INFINITY, f64::max);
            if !tope.is_finite() || tope <= 0.0 {
                return Err(ErrorDiagrama::new(
                    "spectrum: la intensidad máxima tiene que ser positiva",
                ));
            }

            let ejes = Ejes::dibujar(
                c,
                EjesConfig {
                    x0,
                    y0,
                    w,
                    h,
                    xr: (0.0, max_mz * 1.1),
                    yr: (0.0, 110.0),
                    xlabel: "m/z",
                    ylabel: "intensidad relativa (%)",
                    log: false,
                    grilla: true,
                    origen: false,
                    ypaso: Some(20.0),
                },
            );

            for p in &picos {
                let v = 100.0 * (p.intensity / tope);
                let (x, y0_px, y1_px) = (ejes.px(p.mz), ejes.py(0.0), ejes.py(v));
                if !(v.is_finite() && x.is_finite() && y0_px.is_finite() && y1_px.is_finite()) {
                    return Err(ErrorDiagrama::new(
                        "spectrum: coordenadas de MS fuera de rango finito",
                    ));
                }
                c.linea(
                    x,
                    y0_px,
                    x,
                    y1_px,
                    "strokeMain",
                    2.2,
                    None,
                    "",
                );
                let texto = match &p.label {
                    Some(l) => format!("{} {l}", fmt_g(Some(p.mz), 4)),
                    None => fmt_g(Some(p.mz), 4),
                };
                rot.rotular(c, x, y1_px - 5.0, &texto, "middle");
            }
            Ok((x0 + w + 10.0, y0 + h + 40.0))
        }
        otro => Err(ErrorDiagrama::new(format!(
            "spectrum: kind '{otro}' (vale ir, nmr, c13, uv o ms)"
        ))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use dibu_core::leer_bloques;

    fn registro_m1b2() -> Registro {
        let mut reg = Registro::new();
        registrar(&mut reg);
        reg
    }

    fn render_yaml(yaml: &str, tinta: bool) -> Result<String, ErrorDiagrama> {
        let reg = registro_m1b2();
        let bloques = leer_bloques(yaml)?;
        reg.renderizar(&bloques[0], tinta)
    }

    fn extraer_texto_svg(svg: &str) -> String {
        let mut salida = String::new();
        let mut resto = svg;
        while let Some(ini) = resto.find("<text ") {
            resto = &resto[ini..];
            let Some(gt) = resto.find('>') else { break };
            resto = &resto[gt + 1..];
            let Some(fin) = resto.find("</text>") else {
                break;
            };
            let contenido = &resto[..fin];
            let mut en_tag = false;
            for ch in contenido.chars() {
                match ch {
                    '<' => en_tag = true,
                    '>' => en_tag = false,
                    _ if !en_tag => salida.push(if ch == '\u{00A0}' { ' ' } else { ch }),
                    _ => {}
                }
            }
            salida.push(' ');
            resto = &resto[fin + 7..];
        }
        salida
            .replace("&amp;", "&")
            .replace("&lt;", "<")
            .replace("&gt;", ">")
            .replace("&quot;", "\"")
            .replace("&#x27;", "'")
    }

    fn texto_plano(svg: &str) -> String {
        extraer_texto_svg(svg)
            .split_whitespace()
            .collect::<Vec<_>>()
            .join("")
    }

    fn contiene_palabra_no_finita(lower: &str, palabra: &str) -> bool {
        let bytes = lower.as_bytes();
        let mut inicio = 0usize;
        while let Some(rel) = lower[inicio..].find(palabra) {
            let idx = inicio + rel;
            let antes_ok = idx == 0 || !bytes[idx - 1].is_ascii_alphabetic();
            let fin = idx + palabra.len();
            let despues_ok = fin >= bytes.len() || !bytes[fin].is_ascii_alphabetic();
            if antes_ok && despues_ok {
                return true;
            }
            inicio = fin;
        }
        false
    }

    fn assert_svg_valido(svg: &str) {
        assert!(svg.starts_with("<svg "), "no empieza con <svg");
        assert!(svg.ends_with("</svg>"), "no termina con </svg>");
        let lower = svg.to_lowercase();
        assert!(
            !contiene_palabra_no_finita(&lower, "nan"),
            "contiene NaN: {svg}"
        );
        assert!(
            !contiene_palabra_no_finita(&lower, "inf"),
            "contiene inf: {svg}"
        );
        assert!(
            !svg.contains(r#"width="-"#) && !svg.contains(r#"height="-"#),
            "contiene width/height negativo: {svg}"
        );
    }

    #[test]
    fn registra_los_6_tipos_exactos() {
        let reg = registro_m1b2();
        assert_eq!(
            reg.tipos(),
            vec![
                "heating_curve",
                "kinetics",
                "phase_diagram",
                "speciation",
                "spectrum",
                "titration",
            ]
        );
    }

    #[test]
    fn fixtures_cartesianos_ambas_paletas_validos() {
        let yaml = include_str!("../../../../fixtures/diagramas_quimica.yaml");
        let bloques = leer_bloques(yaml).unwrap();
        let reg = registro_m1b2();
        let cartesianos: Vec<&Bloque> = bloques
            .iter()
            .filter(|b| {
                b.get("type")
                    .and_then(Value::as_str)
                    .is_some_and(|t| reg.contiene(t))
            })
            .collect();
        assert_eq!(
            cartesianos.len(),
            23,
            "se esperaban 23 bloques de los 6 tipos cartesianos en fixtures/diagramas_quimica.yaml"
        );
        for b in cartesianos {
            for tinta in [true, false] {
                let svg = reg.renderizar(b, tinta).unwrap();
                assert_svg_valido(&svg);
            }
        }
    }

    #[test]
    fn titration_casos_validos_indicador_invertido_blank_y_errores() {
        // Ácido débil con indicador y pH inicial
        let acetico = render_yaml(
            r#"
type: titration
title: CH3COOH 0,1 M con NaOH 0,1 M
analyte: {kind: weak_acid, pKa: 4.76, conc: 0.1, volume: 25}
titrant: {conc: 0.1, name: NaOH}
indicator: {name: fenolftaleína, range: [8.2, 10]}
annotations: [equivalence, half, initial]
"#,
            true,
        )
        .unwrap();
        assert_svg_valido(&acetico);
        let plano = texto_plano(&acetico);
        assert!(plano.contains("Veq=25mL"));
        assert!(plano.contains("pH=8,73"));
        assert!(plano.contains("pH=pKa=4,76"));
        assert!(plano.contains("pHinicial=2,88"));
        assert!(extraer_texto_svg(&acetico).contains("fenolftaleína (8,2–10,0)"));

        // Triprótico H3PO4
        let h3po4 = render_yaml(
            r#"
type: titration
analyte: {kind: weak_acid, pKa: [2.15, 7.2, 12.35], conc: 0.1, volume: 20}
titrant: {conc: 0.1}
"#,
            false,
        )
        .unwrap();
        assert_svg_valido(&h3po4);
        assert!(h3po4.contains("#1d4ed8"));
        let p_h3 = texto_plano(&h3po4);
        assert!(p_h3.contains("Veq1=20mL") && p_h3.contains("Veq2=40mL") && !p_h3.contains("Veq3"));

        // Base débil con pKb, blankAnnotations y rango de indicador invertido
        let nh3 = render_yaml(
            r#"
type: titration
analyte: {kind: weak_base, pKb: 4.75, conc: 0.1, volume: 25}
titrant: {conc: 0.1, name: HCl}
indicator: {range: [10, 8.2]}
annotations: [equivalence, half, initial]
blankAnnotations: true
"#,
            true,
        )
        .unwrap();
        assert_svg_valido(&nh3);
        let t_nh3 = extraer_texto_svg(&nh3);
        assert!(t_nh3.contains("Veq = 25 mL") || texto_plano(&nh3).contains("Veq=25mL"));
        assert!(!t_nh3.contains("5,28") && !t_nh3.contains("9,25") && !t_nh3.contains("11,12"));

        // annotations: [] -> sin anotaciones; data experimentales; vMax chico
        let vacio = render_yaml(
            r#"
type: titration
analyte: {kind: strong_acid, conc: 0.1, volume: 25}
data: [[0, 1.0], [25, 7.0]]
annotations: []
vMax: 10
"#,
            true,
        )
        .unwrap();
        assert_svg_valido(&vacio);
        assert!(vacio.contains("<circle"));
        assert!(!texto_plano(&vacio).contains("Veq"));

        // Errores
        assert!(render_yaml("type: titration", true)
            .unwrap_err()
            .0
            .contains("'analyte' es"));
        assert!(render_yaml(
            "type: titration\nanalyte: {kind: weak_acid, conc: 0.1}",
            true
        )
        .unwrap_err()
        .0
        .contains("pKa"));
        assert!(render_yaml("type: titration\nanalyte: {kind: sal}", true)
            .unwrap_err()
            .0
            .contains("analyte.kind"));
        assert!(render_yaml(
            "type: titration\nanalyte: {kind: strong_acid}\ntitrant: invalid",
            true
        )
        .unwrap_err()
        .0
        .contains("'titrant' es"));
    }

    #[test]
    fn speciation_casos_validos_desordenado_blank_y_errores() {
        let mono = render_yaml("type: speciation\npKa: 4.76", true).unwrap();
        assert_svg_valido(&mono);
        let tm = texto_plano(&mono);
        assert!(tm.contains("HA") && tm.contains("A−") && tm.contains("pKa=4,76"));

        let tri = render_yaml(
            r#"
type: speciation
title: Ácido fosfórico
pKa: [12.35, 2.15, 7.2]
species: ["H3PO4", "H2PO4^-", "HPO4^2-", "PO4^3-"]
"#,
            false,
        )
        .unwrap();
        assert_svg_valido(&tri);
        let tt = texto_plano(&tri);
        assert!(tt.contains("pKa1=2,15") && tt.contains("pKa2=7,20") && tt.contains("pKa3=12,35"));

        let blank = render_yaml(
            r#"
type: speciation
pKa: [6.35, 10.33]
blankSpecies: true
blankPKa: true
"#,
            true,
        )
        .unwrap();
        assert_svg_valido(&blank);
        assert!(!extraer_texto_svg(&blank).contains("6,35"));
        assert!(!extraer_texto_svg(&blank).contains("H2A"));

        assert!(render_yaml("type: speciation", true)
            .unwrap_err()
            .0
            .contains("falta pKa"));
        assert!(render_yaml("type: speciation\npKa: []", true)
            .unwrap_err()
            .0
            .contains("falta pKa"));
        assert!(render_yaml(
            "type: speciation\npKa: [2, 7]\nspecies: [A, B]",
            true
        )
        .unwrap_err()
        .0
        .contains("3 especies"));
    }

    #[test]
    fn kinetics_modelos_lineal_arrhenius_bordes_y_errores() {
        let ord1 = render_yaml(
            r#"
type: kinetics
order: 1
k: 0.1
A0: 1
annotations: [halfLife]
showProduct: true
"#,
            true,
        )
        .unwrap();
        assert_svg_valido(&ord1);
        let t1 = texto_plano(&ord1);
        assert!(t1.contains("[A]") && t1.contains("[P]") && t1.contains("t½=6,93"));

        let comp = render_yaml(
            r#"
type: kinetics
curves:
  - {order: 0, k: 0.05, A0: 1}
  - {order: 1, k: 0.1, A0: 1}
  - {order: 2, k: 0.2, A0: 1}
tEnd: 30
"#,
            false,
        )
        .unwrap();
        assert_svg_valido(&comp);
        let tc = extraer_texto_svg(&comp);
        assert!(tc.contains("orden 0") && tc.contains("orden 1") && tc.contains("orden 2"));

        let lin0 = render_yaml(
            "type: kinetics\norder: 0\nk: 0.2\nA0: 1\ntEnd: 10\nplot: linear",
            true,
        )
        .unwrap();
        assert_svg_valido(&lin0);
        assert!(texto_plano(&lin0).contains("pendiente=-0,2"));

        let lin2 = render_yaml(
            "type: kinetics\norder: 2\nk: 0.5\nA0: 0.8\nplot: linear",
            true,
        )
        .unwrap();
        assert_svg_valido(&lin2);
        assert!(texto_plano(&lin2).contains("pendiente=0,5"));

        // Degenerado linear: k altísimo -> ts filtrado/underflow pero no NaN/div0 gracias a margen=1.0
        let lin_deg = render_yaml(
            "type: kinetics\norder: 1\nk: 1e100\nA0: 1\nplot: linear",
            true,
        )
        .unwrap();
        assert_svg_valido(&lin_deg);

        let cons = render_yaml(
            "type: kinetics\nmodel: consecutive\nk1: 0.4\nk2: 0.4\nblankAnnotations: true",
            false,
        )
        .unwrap();
        assert_svg_valido(&cons);
        assert!(texto_plano(&cons).contains("tmáx="));

        let rev = render_yaml("type: kinetics\nmodel: reversible\nkf: 0.3\nkb: 0.1", true).unwrap();
        assert_svg_valido(&rev);
        assert!(texto_plano(&rev).contains("K=kf/kb=3"));

        let arr = render_yaml(
            "type: kinetics\nplot: arrhenius\nEa: 75\nA: 1.0e+12\nT: [290, 360]",
            false,
        )
        .unwrap();
        assert_svg_valido(&arr);
        assert!(texto_plano(&arr).contains("−9020K"));

        // Errores y dominios degenerados
        assert!(render_yaml("type: kinetics\norder: 3", true)
            .unwrap_err()
            .0
            .contains("orden 3"));
        assert!(render_yaml("type: kinetics\nmodel: foo", true)
            .unwrap_err()
            .0
            .contains("simple, consecutive o reversible"));
        assert!(render_yaml(
            "type: kinetics\nplot: linear\ncurves:\n  - {order: 1, k: 0.1}\n  - {order: 2, k: 0.2}",
            true
        )
        .unwrap_err()
        .0
        .contains("una sola curva"));
        assert!(render_yaml("type: kinetics\nplot: arrhenius\nT: [300, 300]", true).is_err());
        assert!(render_yaml("type: kinetics\nplot: arrhenius\nT: [0, 300]", true).is_err());
        assert!(render_yaml("type: kinetics\nk: 0", true).is_err());
    }

    #[test]
    fn phase_diagram_single_log_eutectic_y_errores() {
        let agua = render_yaml(
            r#"
type: phase_diagram
substance: water
points:
  - {T: 300, P: 0.01, label: X}
"#,
            true,
        )
        .unwrap();
        assert_svg_valido(&agua);
        let ta = extraer_texto_svg(&agua);
        assert!(ta.contains("Punto triple") && ta.contains("Teb = 373,1 K") && ta.contains('X'));

        let co2 = render_yaml(
            "type: phase_diagram\nsubstance: co2\nscale: log\nblankRegions: true\nblankPoints: true",
            false,
        )
        .unwrap();
        assert_svg_valido(&co2);
        assert!(!extraer_texto_svg(&co2).contains("Sólido"));
        assert!(extraer_texto_svg(&co2).contains("log₁₀ P (atm)"));

        let eut = render_yaml(
            r#"
type: phase_diagram
kind: eutectic
components: [Pb, Sn]
meltingA: 327
meltingB: 232
eutectic: [61.9, 183]
cooling: 30
"#,
            true,
        )
        .unwrap();
        assert_svg_valido(&eut);
        let te = texto_plano(&eut);
        assert!(te.contains("E=61,9%,183°C") && te.contains("Tliq=270°C"));

        // Errores
        assert!(render_yaml("type: phase_diagram\nsubstance: mercurio", true)
            .unwrap_err()
            .0
            .contains("sustancia"));
        assert!(render_yaml(
            "type: phase_diagram\ntriple: [300, 1]\ncritical: [200, 50]",
            true
        )
        .unwrap_err()
        .0
        .contains("crítico"));
        assert!(render_yaml(
            "type: phase_diagram\ntriple: [200, 1]\ncritical: [400, 50]\nmeltingSlope: 0",
            true
        )
        .unwrap_err()
        .0
        .contains("meltingSlope no puede ser 0"));
        assert!(render_yaml(
            "type: phase_diagram\nkind: eutectic\nmeltingA: 300",
            true
        )
        .unwrap_err()
        .0
        .contains("eutéctico"));
        assert!(render_yaml(
            "type: phase_diagram\nkind: eutectic\nmeltingA: 327\nmeltingB: 232\neutectic: [60, 250]",
            true
        )
        .unwrap_err()
        .0
        .contains("por debajo"));
        assert!(render_yaml(
            "type: phase_diagram\nsubstance: water\npoints:\n  - {T: 300}",
            true
        )
        .unwrap_err()
        .0
        .contains("points[0]"));
        assert!(render_yaml(
            "type: phase_diagram\nsubstance: water\npoints:\n  - {T: 300, P: 0}",
            true
        )
        .is_err());
    }

    #[test]
    fn heating_curve_tramos_bordes_y_errores() {
        let cinco = render_yaml(
            "type: heating_curve\nsubstance: water\nmass: 100\nfrom: -20\nto: 120",
            true,
        )
        .unwrap();
        assert_svg_valido(&cinco);
        let t5 = texto_plano(&cinco);
        assert!(t5.contains("sólido") && t5.contains("fusión") && t5.contains("Qtotal=309,1kJ"));

        let borde = render_yaml(
            "type: heating_curve\nsubstance: water\nmass: 100\nfrom: 0\nto: 100",
            false,
        )
        .unwrap();
        assert_svg_valido(&borde);
        let tb = texto_plano(&borde);
        assert!(tb.contains("Qtotal=75,2kJ") && !tb.contains("sólido"));

        let blank = render_yaml(
            "type: heating_curve\nsubstance: water\nfrom: -10\nto: 110\nblankHeat: true\nblankPhases: true",
            true,
        )
        .unwrap();
        assert_svg_valido(&blank);
        assert!(!extraer_texto_svg(&blank).contains("sólido"));

        assert!(render_yaml("type: heating_curve\nfrom: 50\nto: 10", true)
            .unwrap_err()
            .0
            .contains("from < to"));
        assert!(render_yaml(
            "type: heating_curve\nmelting: 120\nboiling: 100",
            true
        )
        .unwrap_err()
        .0
        .contains("melting < boiling"));
        assert!(render_yaml("type: heating_curve\nsubstance: benceno", true)
            .unwrap_err()
            .0
            .contains("faltan datos"));
        assert!(render_yaml("type: heating_curve\nmass: 0", true)
            .unwrap_err()
            .0
            .contains("positivo"));
        assert!(render_yaml(
            "type: heating_curve\nsubstance: water\nfrom: 20\nto: 80\ncpLiquid: 0",
            true
        )
        .is_err());
    }

    #[test]
    fn spectrum_todos_los_modos_limites_y_errores() {
        let ir = render_yaml(
            r#"
type: spectrum
kind: ir
peaks:
  - {at: 3000, depth: 0.75, broad: true, width: 500, label: "O–H"}
  - {at: 1710, depth: 0.85, label: "C=O"}
"#,
            true,
        )
        .unwrap();
        assert_svg_valido(&ir);
        assert!(extraer_texto_svg(&ir).contains("O–H"));

        let nmr = render_yaml(
            r#"
type: spectrum
kind: nmr
tms: true
MHz: 600
signals:
  - {shift: 1.22, integration: 3, neighbors: 2, J: 7, label: "a"}
  - {shift: 13.5, multiplicity: sept, integration: 1, label: "b"}
"#,
            false,
        )
        .unwrap();
        assert_svg_valido(&nmr);
        let tn = extraer_texto_svg(&nmr);
        assert!(tn.contains("600 MHz") && tn.contains("TMS") && tn.contains("a 3H"));

        let c13 = render_yaml(
            "type: spectrum\nkind: c13\nsignals:\n  - {shift: 171, label: \"C=O\"}",
            true,
        )
        .unwrap();
        assert_svg_valido(&c13);
        assert!(extraer_texto_svg(&c13).contains("RMN de ¹³C"));

        let uv = render_yaml(
            "type: spectrum\nkind: uv\nbands:\n  - {at: 254, absorbance: 0.9, width: 18}\nblankLabels: true",
            true,
        )
        .unwrap();
        assert_svg_valido(&uv);
        assert!(!extraer_texto_svg(&uv).contains("λmáx"));

        let ms = render_yaml(
            "type: spectrum\nkind: ms\npeaks:\n  - {mz: 31, intensity: 100, label: \"pico base\"}",
            true,
        )
        .unwrap();
        assert_svg_valido(&ms);
        assert!(extraer_texto_svg(&ms).contains("31 pico base"));

        // Errores y guardrails obligatorios
        assert!(render_yaml(
            "type: spectrum\nkind: raman\npeaks:\n  - {at: 1}",
            true
        )
        .unwrap_err()
        .0
        .contains("kind 'raman'"));
        assert!(render_yaml(
            "type: spectrum\nkind: nmr\nsignals:\n  - {shift: 1, multiplicity: x}",
            true
        )
        .unwrap_err()
        .0
        .contains("multiplicidad"));
        assert!(render_yaml("type: spectrum\nkind: ir\npeaks: []", true)
            .unwrap_err()
            .0
            .contains("falta peaks"));
        assert!(render_yaml("type: spectrum\nkind: ms\npeaks: [31]", true)
            .unwrap_err()
            .0
            .contains("un diccionario"));

        // Guardrails: neighbors negativo/gigante, MHz<=0, width=0, range degenerado, max_mz<=0, tope<=0, NaN
        assert!(render_yaml(
            "type: spectrum\nkind: nmr\nsignals:\n  - {shift: 1, neighbors: -1}",
            true
        )
        .is_err());
        assert!(render_yaml(
            "type: spectrum\nkind: nmr\nsignals:\n  - {shift: 1, neighbors: 100000}",
            true
        )
        .is_err());
        assert!(render_yaml(
            "type: spectrum\nkind: nmr\nMHz: 0\nsignals:\n  - {shift: 1}",
            true
        )
        .is_err());
        assert!(render_yaml(
            "type: spectrum\nkind: nmr\nsignals:\n  - {shift: 1, integration: 0}",
            true
        )
        .is_err());
        assert!(render_yaml(
            "type: spectrum\nkind: ir\npeaks:\n  - {at: 1700, width: 0}",
            true
        )
        .is_err());
        assert!(render_yaml(
            "type: spectrum\nkind: ir\nrange: [500, 500]\npeaks:\n  - {at: 500}",
            true
        )
        .is_err());
        assert!(render_yaml(
            "type: spectrum\nkind: uv\nbands:\n  - {at: 254, absorbance: 0}",
            true
        )
        .is_err());
        assert!(render_yaml(
            "type: spectrum\nkind: ms\npeaks:\n  - {mz: 0, intensity: 100}",
            true
        )
        .is_err());
        assert!(render_yaml(
            "type: spectrum\nkind: ms\npeaks:\n  - {mz: 31, intensity: 0}",
            true
        )
        .is_err());
        assert!(render_yaml(
            "type: spectrum\nkind: ir\npeaks:\n  - {at: .nan}",
            true
        )
        .is_err());

        // Variantes adicionales: strong_base, carbonato diprótico con data, extremos finitos 1e308
        let sb = render_yaml(
            r#"
type: titration
analyte: {kind: strong_base, conc: 0.1, volume: 25}
titrant: {conc: 0.1, name: HCl}
annotations: [equivalence, initial]
"#,
            true,
        )
        .unwrap();
        assert_svg_valido(&sb);
        let t_sb = texto_plano(&sb);
        assert!(t_sb.contains("pHinicial=13,00") && t_sb.contains("Veq=25mL") && t_sb.contains("pH=7,00"));

        let carb = render_yaml(
            r#"
type: titration
analyte: {kind: weak_base, pKa: [6.35, 10.33], charge: -2, conc: 0.1, volume: 25}
titrant: {conc: 0.1, name: HCl}
data: [[0, 11.6], [25, 8.3]]
annotations: [equivalence, half, initial]
"#,
            false,
        )
        .unwrap();
        assert_svg_valido(&carb);
        assert!(texto_plano(&carb).contains("pHinicial=11,65"));

        // Extremos numéricos sin NaN/Inf/panic
        assert!(render_yaml(
            "type: kinetics\nmodel: reversible\nkf: 1e308\nkb: 1e308",
            true
        )
        .is_err());
        assert!(render_yaml(
            "type: phase_diagram\nkind: eutectic\nmeltingA: 327\nmeltingB: 232\neutectic: [60, 180]\ncooling: 1e308",
            true
        )
        .is_err());
        let nmr_neg = render_yaml(
            "type: spectrum\nkind: nmr\nsignals:\n  - {shift: -1e308, label: x}",
            true,
        )
        .unwrap();
        assert_svg_valido(&nmr_neg);
    }

    #[test]
    fn regresion_extremos_finitos_sin_nan_ni_inf() {
        let casos = [
            // Finding 1: spectrum ir y uv con range overflow
            "type: spectrum\nkind: ir\nrange: [1e308, -1e308]\npeaks:\n  - {at: 1700}",
            "type: spectrum\nkind: uv\nrange: [1e308, -1e308]\nbands:\n  - {at: 254}",
            "type: spectrum\nkind: ir\nrange: [1e300, 2e300]\npeaks:\n  - {at: 1700}",
            // Finding 2: spectrum ms extremos y negativos
            "type: spectrum\nkind: ms\npeaks:\n  - {mz: 10, intensity: 100}\n  - {mz: 20, intensity: -1e308}",
            "type: spectrum\nkind: ms\npeaks:\n  - {mz: 10, intensity: 100}\n  - {mz: -1e308, intensity: 50}",
            "type: spectrum\nkind: ms\npeaks:\n  - {mz: 10, intensity: 1e307}",
            "type: spectrum\nkind: ms\npeaks:\n  - {mz: 1e300, intensity: 100}",
            // Finding 3: titration indicator.range extremo
            "type: titration\nanalyte: {kind: strong_acid}\nindicator: {range: [0, 1e308]}",
            "type: titration\nanalyte: {kind: strong_acid}\nindicator: {range: [0, 1e307]}",
            "type: titration\nanalyte: {kind: strong_acid}\nindicator: {range: [-1e307, 1e307]}",
            "type: titration\nanalyte: {kind: strong_acid}\nvMax: 1e307",
            // Finding 4: speciation pKa extremo
            "type: speciation\npKa: 1e307",
            "type: speciation\npKa: -1e307",
            // Finding 5: heating_curve, arrhenius, eutectic cooling extremos
            "type: heating_curve\nsubstance: water\nmass: 1e-305\nfrom: -8e307\nto: 8e307",
            "type: heating_curve\nsubstance: water\nmass: 1e-305\nfrom: 1e300\nto: 2e300",
            "type: kinetics\nplot: arrhenius\nEa: 1e305\nA: 1e10\nT: [0.08, 1000]",
            "type: kinetics\nplot: arrhenius\nEa: 1e304\nA: 1e10\nT: [1, 2]",
            "type: phase_diagram\nkind: eutectic\nmeltingA: 327\nmeltingB: 232\neutectic: [61.9, 183]\ncooling: 1e155",
            "type: phase_diagram\nkind: eutectic\nmeltingA: 327\nmeltingB: 232\neutectic: [61.9, 183]\ncooling: -1e308",
            "type: spectrum\nkind: nmr\nsignals:\n  - {shift: 1e300, label: x}",
        ];
        for (idx, yaml) in casos.iter().enumerate() {
            match render_yaml(yaml, true) {
                Ok(svg) => assert_svg_valido(&svg),
                Err(e) => assert!(
                    !e.0.is_empty(),
                    "caso {idx} devolvió error vacío: {yaml}"
                ),
            }
        }

        // Verificar que titration preserva rangos válidos fuera de [0, 14] como [-1, 15]
        let ind_valido = render_yaml(
            "type: titration\nanalyte: {kind: strong_acid}\nindicator: {name: amplio, range: [-1, 15]}",
            true,
        )
        .unwrap();
        assert_svg_valido(&ind_valido);
        assert!(extraer_texto_svg(&ind_valido).contains("amplio (−1,0–15,0)")
            || extraer_texto_svg(&ind_valido).contains("amplio (-1,0–15,0)"));

        // Verificar que spectrum ms con intensity: 1e307 único devuelve SVG válido (sin overflow 100*intensity)
        let ms_grande = render_yaml(
            "type: spectrum\nkind: ms\npeaks:\n  - {mz: 10, intensity: 1e307}",
            true,
        )
        .unwrap();
        assert_svg_valido(&ms_grande);
    }

    #[test]
    fn regresion_tier5_cotas_y_coordenadas_acotadas() {
        // H1 + H5.4: speciation y titration acotan cantidad de pKa (MAX_PKA = 20)
        let mut muchos_pka = String::from("type: speciation\npKa: [");
        for i in 0..100 {
            if i > 0 {
                muchos_pka.push_str(", ");
            }
            muchos_pka.push_str("4.0");
        }
        muchos_pka.push(']');
        assert!(render_yaml(&muchos_pka, true).is_err());

        let mut tit_pka = String::from("type: titration\nanalyte:\n  kind: weak_acid\n  pKa: [");
        for i in 0..30 {
            if i > 0 {
                tit_pka.push_str(", ");
            }
            tit_pka.push_str("4.0");
        }
        tit_pka.push(']');
        assert!(render_yaml(&tit_pka, true).is_err());

        // H2: speciation con pKa 1e300, spectrum.c13 con intensity < 0 o shift -1e300, phase_diagram points T=1e305
        assert!(render_yaml("type: speciation\npKa: [1e300]", true).is_err());
        assert!(render_yaml(
            "type: spectrum\nkind: c13\nsignals:\n  - {shift: 50, intensity: -1e300}",
            true
        )
        .is_err());
        let svg_c13_shift = render_yaml(
            "type: spectrum\nkind: c13\nsignals:\n  - {shift: -1e300, intensity: 0.5}",
            true,
        )
        .unwrap();
        assert_svg_valido(&svg_c13_shift);
        assert!(svg_c13_shift.len() < 10_000, "no debía emitir coordenadas gigantes");
        assert!(render_yaml(
            "type: phase_diagram\nsubstance: water\npoints:\n  - {T: 1e305, P: 1.0, label: X}",
            true
        )
        .is_err());

        // H2 + H4: titration.data con [1e300, 7] y phase_diagram scale:log meltingSlope:1e-305 no emiten coords gigantes
        let svg_tit = render_yaml(
            "type: titration\nanalyte: {kind: weak_acid, pKa: 4.76}\ndata:\n  - [1e300, 7.0]",
            true,
        )
        .unwrap();
        assert_svg_valido(&svg_tit);
        assert!(!svg_tit.contains("<circle cx=\"1150000"), "{svg_tit}");

        let svg_pd = render_yaml(
            "type: phase_diagram\nsubstance: water\nscale: log\nmeltingSlope: 1e-305",
            true,
        )
        .unwrap();
        assert_svg_valido(&svg_pd);
        for parte in svg_pd.split(r#"<polyline points=""#).skip(1) {
            let pts = parte.split('"').next().unwrap_or("");
            for par in pts.split(' ') {
                let mut nums = par.split(',');
                let x = nums.next().unwrap().parse::<f64>().unwrap();
                let y = nums.next().unwrap().parse::<f64>().unwrap();
                assert!(x.abs() < 1e4 && y.abs() < 1e4, "coordenada fuera de cota: {x},{y}");
            }
        }

        // F3: spectrum.ir at=1e308, titration indicator.range fuera/extremo, phase_diagram.cooling fuera de [0, 100]
        let svg_ir = render_yaml(
            "type: spectrum\nkind: ir\npeaks:\n  - {at: 1e308, label: X}",
            true,
        )
        .unwrap();
        assert_svg_valido(&svg_ir);
        assert!(!extraer_texto_svg(&svg_ir).contains('X'));
        assert!(svg_ir.len() < 25_000, "spectrum.ir demasiado largo: {}", svg_ir.len());

        let svg_ind_out = render_yaml(
            "type: titration\nanalyte: {kind: weak_acid, pKa: 4.76}\nindicator: {name: X, range: [1e305, 1e305]}",
            true,
        )
        .unwrap();
        assert_svg_valido(&svg_ind_out);
        assert!(svg_ind_out.len() < 15_000, "indicator fuera de [0,14] emitió coords gigantes");

        let svg_ind_wide = render_yaml(
            "type: titration\nanalyte: {kind: strong_acid}\nindicator: {name: X, range: [-1e307, 1e307]}",
            true,
        )
        .unwrap();
        assert_svg_valido(&svg_ind_wide);
        assert!(svg_ind_wide.len() < 15_000);

        for bad_cool in [-10_000_000_000.0, -0.1, 100.1, 1e155] {
            let y = format!(
                "type: phase_diagram\nkind: eutectic\nmeltingA: 800\nmeltingB: 600\neutectic: [40, 300]\ncooling: {bad_cool}"
            );
            assert!(render_yaml(&y, true).is_err(), "cooling={bad_cool} debía fallar");
        }
        for good_cool in [0.0, 100.0] {
            let y = format!(
                "type: phase_diagram\nkind: eutectic\nmeltingA: 800\nmeltingB: 600\neutectic: [40, 300]\ncooling: {good_cool}"
            );
            let svg = render_yaml(&y, true).unwrap();
            assert_svg_valido(&svg);
        }
    }
}
