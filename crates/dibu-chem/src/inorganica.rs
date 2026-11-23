//! Renderizadores de química inorgánica (M1c):
//! `crystal_field`, `complex`, `latimer`, `frost`, `born_haber`.

use crate::chem;
use crate::common::{
    ancho_formula, booleano, caja_vacia, carga_sup, coma, encabezado_q, entero, envolver_error,
    flecha_doble, flechita, geometria, opt_lista, opt_texto, romano, romano_signo, rotulo_q, tq,
};
use dibu_core::{valor_str, Bloque, ErrorDiagrama, Lienzo, Punto, Registro, SANS};
use dibu_plot::{fmt_g, Ejes, EjesConfig};
use serde_json::Value;
use std::collections::BTreeSet;

pub fn registrar(reg: &mut Registro) {
    reg.registrar("crystal_field", render_crystal_field);
    reg.registrar("complex", render_complex);
    reg.registrar("latimer", render_latimer);
    reg.registrar("frost", render_frost);
    reg.registrar("born_haber", render_born_haber);
}

fn mal_formado(tipo: &str, err: ErrorDiagrama) -> ErrorDiagrama {
    ErrorDiagrama::new(format!("bloque {tipo} mal formado: {err}"))
}

fn filas(c: &mut Lienzo, x: f64, mut y: f64, datos: &[(&str, String, bool)], ancho: f64) -> f64 {
    for (nombre, valor, blanco) in datos {
        c.t(
            x,
            y,
            &format!("{nombre}:"),
            10.5,
            700,
            "textMain",
            "start",
            SANS,
        );
        let x_valor = x + ancho;
        if *blanco {
            c.vacio(x_valor, x_valor + 110.0, y + 2.0);
        } else {
            tq(
                c, x_valor, y, valor, 10.5, 400, "textMain", "start", SANS, false,
            );
        }
        y += 20.0;
    }
    y
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

// --- Campo cristalino ------------------------------------------------------------------------------------

fn delta_simbolo(geometria: &str) -> &'static str {
    match geometria {
        "octahedral" => "Δ_o",
        "tetrahedral" => "Δ_t",
        _ => "Δ",
    }
}

fn electrones_y_espin(b: &Bloque, donde: &str) -> Result<(i32, bool, String), ErrorDiagrama> {
    let n = if let Some(val) = b.get("electrons") {
        let n_i64 = entero(val, "electrons").map_err(|e| mal_formado(donde, e))?;
        i32::try_from(n_i64)
            .map_err(|_| ErrorDiagrama::new(format!("{donde}: de 0 a 10 electrones d")))?
    } else if let (Some(metal_val), Some(ox_val)) = (b.get("metal"), b.get("oxidation")) {
        let metal = valor_str(metal_val);
        let ox_i64 = entero(ox_val, "oxidation").map_err(|e| mal_formado(donde, e))?;
        let ox = i32::try_from(ox_i64).map_err(|_| {
            ErrorDiagrama::new(format!(
                "{donde}: {metal} con estado de oxidación {ox_i64:+} no queda con 0 a 10 electrones d"
            ))
        })?;
        i32::from(envolver_error(donde, chem::electrones_d(&metal, ox))?)
    } else {
        return Err(ErrorDiagrama::new(format!(
            "{donde}: falta electrons (los d) o metal y oxidation"
        )));
    };

    let geom = match b.get("geometry") {
        None => "octahedral".to_string(),
        Some(v) => valor_str(v),
    };
    let spin_opt: Option<String> = match b.get("spin") {
        None | Some(Value::Null) => None,
        Some(v) => Some(valor_str(v)),
    };
    let mut origen = String::new();
    let espin = if geom == "tetrahedral" && spin_opt.is_none() {
        origen = "Δt chico: siempre alto espín".to_string();
        "high".to_string()
    } else if spin_opt.is_none() && b.contains_key("ligand") {
        let lig = b.get("ligand").map(valor_str).unwrap_or_default();
        let fuerte = envolver_error(donde, chem::es_campo_fuerte(&lig))?;
        origen = format!("{lig}: campo {}", if fuerte { "fuerte" } else { "débil" });
        if fuerte { "low" } else { "high" }.to_string()
    } else if let Some(s) = spin_opt {
        s
    } else if geom == "square_planar" {
        "low".to_string()
    } else {
        "high".to_string()
    };

    if espin != "high" && espin != "low" {
        return Err(ErrorDiagrama::new(format!(
            "{donde}: spin vale high o low"
        )));
    }
    Ok((n, espin == "high", origen))
}

fn render_crystal_field(c: &mut Lienzo, b: &Bloque) -> Result<Punto, ErrorDiagrama> {
    let geom = match b.get("geometry") {
        None => "octahedral".to_string(),
        Some(v) => valor_str(v),
    };
    let (n, alto, origen) = electrones_y_espin(b, "crystal_field")?;
    let cc = envolver_error("crystal_field", chem::campo_cristalino(&geom, n, alto))?;
    let niveles = chem::niveles_campo(&geom).ok_or_else(|| {
        ErrorDiagrama::new(format!(
            "crystal_field: geometry '{geom}' (vale octahedral, tetrahedral o square_planar)"
        ))
    })?;

    let top = encabezado_q(c, b);
    let lado = 22.0_f64;
    let x_libre = 30.0_f64;
    let x_campo = 210.0_f64;
    let y_base = top + 40.0;
    let alto_px = 150.0_f64;

    let mut lo = f64::INFINITY;
    let mut hi = f64::NEG_INFINITY;
    for nc in niveles {
        lo = lo.min(nc.energia);
        hi = hi.max(nc.energia);
    }
    let span = hi - lo;
    let py = |e: f64| -> f64 {
        if span.abs() < 1e-12 {
            y_base + alto_px / 2.0
        } else {
            y_base + alto_px - (e - lo) / span * alto_px
        }
    };

    let mut alturas: Vec<f64> = Vec::with_capacity(niveles.len());
    for nc in niveles {
        let mut y = py(nc.energia);
        if let Some(&prev) = alturas.last() {
            if prev - y < 32.0 {
                y = prev - 32.0;
            }
        }
        alturas.push(y);
    }

    let y_cero = py(0.0);
    let trazo = c.pal.stroke_main;
    for k in 0..5 {
        c.add(format!(
            r#"<rect x="{:.1}" y="{:.1}" width="{lado:.1}" height="{lado:.1}" fill="{}" stroke="{trazo}" stroke-width="1.2"/>"#,
            x_libre + k as f64 * lado,
            y_cero - lado / 2.0,
            c.pal.fill_cell
        ));
    }
    c.t(
        x_libre + 2.5 * lado,
        y_cero + lado / 2.0 + 14.0,
        "ion libre",
        9.5,
        400,
        "textSub",
        "middle",
        SANS,
    );

    let vacio = booleano(b, "blankElectrons", false);
    for (idx, (nc, llenado)) in niveles.iter().zip(cc.llenado.iter()).enumerate() {
        let y = alturas.get(idx).copied().unwrap_or(y_cero);
        let ancho = nc.orbitales.len() as f64 * lado;
        let x0 = x_campo + (3.0 * lado - ancho) / 2.0;
        for (j, &e) in llenado.iter().enumerate() {
            let bx = x0 + j as f64 * lado;
            c.add(format!(
                r#"<rect x="{bx:.1}" y="{:.1}" width="{lado:.1}" height="{lado:.1}" fill="{}" stroke="{trazo}" stroke-width="1.2"/>"#,
                y - lado / 2.0,
                c.pal.fill_cell
            ));
            if vacio {
                continue;
            }
            if e >= 1 {
                let xf = bx + if e == 2 { lado / 2.0 - 4.0 } else { lado / 2.0 };
                flechita(c, xf, y, true, 14.0);
            }
            if e == 2 {
                flechita(c, bx + lado / 2.0 + 4.0, y, false, 14.0);
            }
        }
        c.linea(
            x_libre + 5.0 * lado + 4.0,
            y_cero,
            x0 - 4.0,
            y,
            "strokeMuted",
            0.8,
            None,
            "3 3",
        );
        let etiqueta = if !nc.nombre.is_empty() {
            nc.nombre.to_string()
        } else {
            nc.orbitales.join(", ")
        };
        let etiq_fmt = etiqueta
            .replace("t2g", "t_2g")
            .replace("t2", "t_2")
            .replace("eg", "e_g");
        tq(
            c,
            x_campo + 3.0 * lado + 10.0,
            y + 4.0,
            &etiq_fmt,
            10.5,
            600,
            "textMain",
            "start",
            SANS,
            false,
        );
    }

    let delta = delta_simbolo(&geom);
    if geom != "square_planar" {
        if let (Some(&y_min), Some(&y_max)) = (alturas.first(), alturas.last()) {
            let x_d = x_campo - 22.0;
            flecha_doble(c, x_d, y_min - 2.0, y_max + 2.0, "strokeSub");
            tq(
                c,
                x_d - 6.0,
                (y_min + y_max) / 2.0 + 4.0,
                delta,
                11.0,
                700,
                "textMain",
                "end",
                SANS,
                false,
            );
        }
    }

    if let Some(comp) = opt_texto(b, "complex") {
        tq(
            c,
            0.0,
            top + 14.0,
            &comp,
            12.0,
            700,
            "textMain",
            "start",
            SANS,
            false,
        );
    }

    let reparto = niveles
        .iter()
        .zip(cc.llenado.iter())
        .map(|(nc, llenado)| {
            let base = if !nc.nombre.is_empty() {
                nc.nombre
            } else {
                nc.orbitales.first().copied().unwrap_or("")
            };
            let base_fmt = base
                .replace("t2g", "t_2g")
                .replace("t2", "t_2")
                .replace("eg", "e_g");
            let suma: u8 = llenado.iter().sum();
            format!("{base_fmt}^{suma}")
        })
        .collect::<Vec<_>>()
        .join(" ");

    let blanco = booleano(b, "blank", false);
    let mut datos = vec![
        (
            "Configuración",
            if geom != "square_planar" {
                format!("d^{n}: {reparto}")
            } else {
                format!("d^{n}")
            },
            blanco || booleano(b, "blankConfig", false),
        ),
        (
            "Espín",
            format!(
                "{}{}",
                if alto { "alto" } else { "bajo" },
                if origen.is_empty() {
                    String::new()
                } else {
                    format!(" ({origen})")
                }
            ),
            blanco || booleano(b, "blankSpin", false),
        ),
        (
            "Desapareados",
            cc.desapareados().to_string(),
            blanco || booleano(b, "blankUnpaired", false),
        ),
        (
            "μ (solo espín)",
            format!("{} μB", coma(cc.momento(), 2)),
            blanco || booleano(b, "blankMoment", false),
        ),
    ];

    if geom != "square_planar" {
        let eecc = if cc.eecc().abs() < 1e-9 {
            0.0
        } else {
            cc.eecc()
        };
        datos.push((
            "EECC",
            format!("{} {delta}", coma(eecc, 1).replace('-', "−")),
            blanco || booleano(b, "blankCFSE", false),
        ));
    }

    let y = filas(c, 0.0, y_base + alto_px + 40.0, &datos, 120.0);
    Ok((430.0, y))
}

// --- Complejos ------------------------------------------------------------------------------------------

fn formula_complejo(metal: &str, ligandos: &[String], carga: i64) -> String {
    let mut orden: Vec<&str> = Vec::new();
    for lig in ligandos {
        if !orden.contains(&lig.as_str()) {
            orden.push(lig.as_str());
        }
    }
    let mut partes = Vec::with_capacity(orden.len());
    for lig in orden {
        let limpio = lig.split('^').next().unwrap_or("");
        let n = ligandos.iter().filter(|x| x.as_str() == lig).count();
        let compuesto = limpio.chars().filter(|ch| ch.is_uppercase()).count() > 1
            || limpio.chars().any(|ch| ch.is_ascii_digit())
            || limpio.chars().count() > 2;
        let texto = if compuesto && n > 1 {
            format!("({limpio})")
        } else {
            limpio.to_string()
        };
        if n > 1 {
            partes.push(format!("{texto}{n}"));
        } else {
            partes.push(texto);
        }
    }
    format!("[{metal}{}]{}", partes.concat(), carga_sup(carga))
}

fn render_complex(c: &mut Lienzo, b: &Bloque) -> Result<Punto, ErrorDiagrama> {
    let geom = match b.get("geometry") {
        None => "octahedral".to_string(),
        Some(v) => valor_str(v),
    };
    if !matches!(
        geom.as_str(),
        "octahedral" | "square_planar" | "tetrahedral"
    ) {
        return Err(ErrorDiagrama::new(
            "complex: geometry vale octahedral, square_planar o tetrahedral",
        ));
    }
    let geom_info = geometria(&geom).ok_or_else(|| {
        ErrorDiagrama::new("complex: geometry vale octahedral, square_planar o tetrahedral")
    })?;
    let sitios = geom_info.ligandos;

    let metal = match b.get("metal") {
        None => "M".to_string(),
        Some(v) => valor_str(v),
    };
    let lig_arr = opt_lista(b, "ligands", "ligands").map_err(|e| mal_formado("complex", e))?;
    let ligandos: Vec<String> = lig_arr
        .map(|a| a.iter().map(valor_str).collect())
        .unwrap_or_default();
    if ligandos.len() != sitios.len() {
        return Err(ErrorDiagrama::new(format!(
            "complex: la geometría {geom} tiene {} sitios (hay {} ligandos); el orden es arriba, abajo, izquierda, derecha, adelante, atrás",
            sitios.len(),
            ligandos.len()
        )));
    }

    let carga = match b.get("charge") {
        None => 0,
        Some(v) => entero(v, "charge").map_err(|e| mal_formado("complex", e))?,
    };

    let top = encabezado_q(c, b);
    let cx = 120.0_f64;
    let cy = top + 96.0;
    let largo = 60.0_f64;
    let trazo = c.pal.stroke_main;

    for (&(ang_g, tipo), etiqueta) in sitios.iter().zip(ligandos.iter()) {
        let ang = ang_g.to_radians();
        let f = if tipo != "p" { 0.8 } else { 1.0 };
        let ux = ang.cos();
        let uy = -ang.sin();
        let ax = cx + ux * 13.0;
        let ay = cy + uy * 13.0;
        let bx = cx + ux * (largo * f - 12.0);
        let by = cy + uy * (largo * f - 12.0);
        let px = -uy;
        let py = ux;

        match tipo {
            "w" => {
                c.add(format!(
                    r#"<polygon points="{ax:.1},{ay:.1} {:.1},{:.1} {:.1},{:.1}" fill="{trazo}"/>"#,
                    bx + px * 4.0,
                    by + py * 4.0,
                    bx - px * 4.0,
                    by - py * 4.0
                ));
            }
            "h" => {
                for k in 1..8 {
                    let t = k as f64 / 7.0;
                    let hx = ax + (bx - ax) * t;
                    let hy = ay + (by - ay) * t;
                    let w = 0.6 + 3.4 * t;
                    c.add(format!(
                        r#"<line x1="{:.1}" y1="{:.1}" x2="{:.1}" y2="{:.1}" stroke="{trazo}" stroke-width="1.1"/>"#,
                        hx + px * w,
                        hy + py * w,
                        hx - px * w,
                        hy - py * w
                    ));
                }
            }
            _ => {
                c.add(format!(
                    r#"<line x1="{ax:.1}" y1="{ay:.1}" x2="{bx:.1}" y2="{by:.1}" stroke="{trazo}" stroke-width="1.8"/>"#
                ));
            }
        }
        let etiq = etiqueta.split('^').next().unwrap_or("");
        tq(
            c,
            cx + ux * largo * f,
            cy + uy * largo * f + 4.0,
            etiq,
            11.0,
            500,
            "textMain",
            "middle",
            SANS,
            false,
        );
    }

    tq(
        c,
        cx,
        cy + 5.0,
        &metal,
        13.0,
        700,
        "textMain",
        "middle",
        SANS,
        false,
    );

    let blancos: BTreeSet<String> = match b.get("blank") {
        None | Some(Value::Null) | Some(Value::Bool(false)) => BTreeSet::new(),
        Some(Value::Bool(true)) => ["formula", "oxidation", "coordination", "d", "isomer"]
            .into_iter()
            .map(str::to_string)
            .collect(),
        Some(Value::Array(arr)) => arr.iter().map(valor_str).collect(),
        Some(Value::String(s)) => s.chars().map(|ch| ch.to_string()).collect(),
        Some(other) => {
            return Err(mal_formado(
                "complex",
                ErrorDiagrama::new(format!("blank: {other} no es iterable")),
            ));
        }
    };

    let lig_refs: Vec<&str> = ligandos.iter().map(String::as_str).collect();
    let oxidacion: i64 = match b.get("oxidation") {
        Some(v) => entero(v, "oxidation").map_err(|e| mal_formado("complex", e))?,
        None => {
            let carga_i32 = i32::try_from(carga)
                .map_err(|_| ErrorDiagrama::new("complex: charge fuera de rango"))?;
            i64::from(envolver_error(
                "complex",
                chem::oxidacion_metal(carga_i32, &lig_refs),
            )?)
        }
    };

    let mut datos = vec![
        (
            "Fórmula",
            formula_complejo(&metal, &ligandos, carga),
            blancos.contains("formula"),
        ),
        (
            "Estado de oxidación",
            format!("{metal}({})", romano(oxidacion)),
            blancos.contains("oxidation"),
        ),
        (
            "N.º de coordinación",
            ligandos.len().to_string(),
            blancos.contains("coordination"),
        ),
    ];

    if let Ok(ox_i32) = i32::try_from(oxidacion) {
        if let Ok(de) = chem::electrones_d(&metal, ox_i32) {
            datos.push(("Electrones d", format!("d^{de}"), blancos.contains("d")));
        }
    }

    if let Some(isom) = chem::isomero(&geom, &lig_refs) {
        datos.push(("Isómero", isom.to_string(), blancos.contains("isomer")));
    }

    let y = filas(c, 250.0, top + 46.0, &datos, 150.0);
    Ok((560.0, (top + 190.0).max(y)))
}

// --- Latimer y Frost ------------------------------------------------------------------------------------

fn render_latimer(c: &mut Lienzo, b: &Bloque) -> Result<Punto, ErrorDiagrama> {
    let lt = envolver_error("latimer", chem::latimer(b))?;
    let ocultos: BTreeSet<usize> = match opt_lista(b, "blankPotentials", "blankPotentials")
        .map_err(|e| mal_formado("latimer", e))?
    {
        None => BTreeSet::new(),
        Some(arr) => {
            let mut set = BTreeSet::new();
            for (idx, v) in arr.iter().enumerate() {
                let n = entero(v, &format!("blankPotentials[{idx}]"))
                    .map_err(|e| mal_formado("latimer", e))?;
                if let Ok(u) = usize::try_from(n) {
                    set.insert(u);
                }
            }
            set
        }
    };

    let top = encabezado_q(c, b) + 28.0;
    let mut x = 0.0_f64;
    let y = top + 20.0;
    let mut posiciones: Vec<(f64, f64)> = Vec::with_capacity(lt.especies.len());

    for (i, especie) in lt.especies.iter().enumerate() {
        let ancho = ancho_formula(especie, 12.0);
        tq(
            c, x, y, especie, 12.0, 600, "textMain", "start", SANS, false,
        );
        c.t(
            x + ancho / 2.0,
            y - 18.0,
            &romano_signo(lt.oxidacion[i]),
            9.0,
            400,
            "textMuted",
            "middle",
            SANS,
        );
        posiciones.push((x, x + ancho));
        x += ancho;
        if let Some(&pot) = lt.potenciales.get(i) {
            let largo = 66.0_f64;
            c.linea(
                x + 6.0,
                y - 4.0,
                x + largo - 6.0,
                y - 4.0,
                "accentPrimary",
                1.5,
                Some("main"),
                "",
            );
            if ocultos.contains(&i) {
                caja_vacia(c, x + largo / 2.0, y - 18.0, 40.0, 13.0);
            } else {
                c.t(
                    x + largo / 2.0,
                    y - 12.0,
                    &coma(pot, 2),
                    9.5,
                    700,
                    "textMain",
                    "middle",
                    SANS,
                );
            }
            x += largo;
        }
    }

    let ancho_total = x;
    let y_arco = y + 12.0;
    let blanco_calc = booleano(b, "blankComputed", false);
    let pedidos_opt = opt_lista(b, "compute", "compute").map_err(|e| mal_formado("latimer", e))?;
    let pedidos = pedidos_opt.map(Vec::as_slice).unwrap_or(&[]);
    let mut rotulos: Vec<(f64, f64, f64)> = Vec::with_capacity(pedidos.len());
    let n_esp = lt.especies.len() as i64;

    for (k, par_val) in pedidos.iter().enumerate() {
        let par = par_val.as_array().ok_or_else(|| {
            mal_formado(
                "latimer",
                ErrorDiagrama::new(format!("compute[{k}]: tiene que ser una lista")),
            )
        })?;
        if par.len() < 2 {
            return Err(mal_formado(
                "latimer",
                ErrorDiagrama::new(format!("compute[{k}]: faltan dos índices")),
            ));
        }
        let ii = entero(&par[0], &format!("compute[{k}][0]"))
            .map_err(|e| mal_formado("latimer", e))?;
        let jj = entero(&par[1], &format!("compute[{k}][1]"))
            .map_err(|e| mal_formado("latimer", e))?;
        if ii < 0 || ii >= n_esp || jj < 0 || jj >= n_esp || ii == jj {
            return Err(ErrorDiagrama::new(format!(
                "latimer: compute[{k}] son dos índices distintos de species"
            )));
        }
        let a = ii.min(jj) as usize;
        let b_idx = ii.max(jj) as usize;
        let pot_calc = lt.potencial(a, b_idx);
        if !pot_calc.is_finite() {
            return Err(ErrorDiagrama::new(format!(
                "latimer: compute[{k}] produce un potencial fuera de rango finito"
            )));
        }
        let (Some(&(xa0, xa1)), Some(&(xb0, xb1))) = (posiciones.get(a), posiciones.get(b_idx))
        else {
            continue;
        };
        let xa = (xa0 + xa1) / 2.0;
        let xb = (xb0 + xb1) / 2.0;
        let hondo = 22.0 + 30.0 * k as f64;
        c.add(format!(
            r#"<path d="M {xa:.1} {y_arco:.1} Q {:.1} {:.1} {xb:.1} {y_arco:.1}" fill="none" stroke="{}" stroke-width="1.1" marker-end="url(#flecha-sub-{})"/>"#,
            (xa + xb) / 2.0,
            y_arco + 2.0 * hondo,
            c.pal.stroke_sub,
            c.sufijo
        ));
        rotulos.push(((xa + xb) / 2.0, y_arco + hondo + 13.0, pot_calc));
    }

    for &(x_r, y_r, e) in &rotulos {
        rotulo_q(
            c,
            x_r,
            y_r,
            "E°",
            Some(&format!("{} V", coma(e, 2))),
            blanco_calc,
            "middle",
            9.5,
        );
    }

    let mut y_fin = if !pedidos.is_empty() {
        y_arco + 22.0 + 30.0 * pedidos.len() as f64 + 10.0
    } else {
        y + 20.0
    };

    if booleano(b, "showDisproportionation", false) {
        let des = lt.desproporcionan();
        let texto = if des.is_empty() {
            "ninguna".to_string()
        } else {
            des.iter()
                .filter_map(|&k| lt.especies.get(k).map(String::as_str))
                .collect::<Vec<_>>()
                .join(", ")
        };
        rotulo_q(
            c,
            0.0,
            y_fin + 14.0,
            "Desproporcionan",
            Some(&texto),
            booleano(b, "blankDisproportionation", false),
            "start",
            10.0,
        );
        y_fin += 26.0;
    }

    Ok((ancho_total + 10.0, y_fin))
}

fn render_frost(c: &mut Lienzo, b: &Bloque) -> Result<Punto, ErrorDiagrama> {
    let lt = envolver_error("frost", chem::latimer(b))?;
    let puntos: Vec<(f64, f64)> = lt.oxidacion.iter().copied().zip(lt.frost()).collect();
    if puntos
        .iter()
        .any(|&(ox, ne)| !(ox.is_finite() && ne.is_finite()))
    {
        return Err(ErrorDiagrama::new("frost: valores fuera de rango finito"));
    }

    let mut min_x = f64::INFINITY;
    let mut max_x = f64::NEG_INFINITY;
    let mut min_y = f64::INFINITY;
    let mut max_y = f64::NEG_INFINITY;
    for &(ox, ne) in &puntos {
        min_x = min_x.min(ox);
        max_x = max_x.max(ox);
        min_y = min_y.min(ne);
        max_y = max_y.max(ne);
    }
    let dy = max_y - min_y;
    if !dy.is_finite() {
        return Err(ErrorDiagrama::new(
            "frost: rango vertical fuera de rango finito",
        ));
    }
    let margen = if (dy * 0.12) == 0.0 { 1.0 } else { dy * 0.12 };
    let xr = (min_x - 0.5, max_x + 0.5);
    let yr = (min_y - margen, max_y + margen);
    if !rango_ejes_valido(xr.0, xr.1) || !rango_ejes_valido(yr.0, yr.1) {
        return Err(ErrorDiagrama::new(
            "frost: rango de ejes fuera de límites finitos",
        ));
    }

    let top = encabezado_q(c, b);
    let (x0, y0, w, h) = (60.0_f64, top + 18.0, 440.0_f64, 250.0_f64);
    let ejes = Ejes::dibujar(
        c,
        EjesConfig {
            x0,
            y0,
            w,
            h,
            xr,
            yr,
            xlabel: "estado de oxidación",
            ylabel: "N·E° (V)",
            log: false,
            grilla: true,
            origen: true,
            ypaso: None,
        },
    );

    for &(ox, ne) in &puntos {
        if !(ejes.px(ox).is_finite() && ejes.py(ne).is_finite()) {
            return Err(ErrorDiagrama::new(
                "frost: proyección fuera de rango finito",
            ));
        }
    }

    let mut orden = puntos.clone();
    orden.sort_by(|a, b_pt| a.0.total_cmp(&b_pt.0).then_with(|| a.1.total_cmp(&b_pt.1)));
    ejes.curva(c, &orden, 0, 1.8, "strokeMain");

    let des: BTreeSet<usize> = if booleano(b, "showDisproportionation", false) {
        lt.desproporcionan().into_iter().collect()
    } else {
        BTreeSet::new()
    };
    let blanco = booleano(b, "blankLabels", false);
    for (k, &(ox, ne)) in puntos.iter().enumerate() {
        let x = ejes.px(ox);
        let y = ejes.py(ne);
        let relleno = if des.contains(&k) {
            c.pal.fill_cell
        } else {
            c.pal.stroke_main
        };
        c.add(format!(
            r#"<circle cx="{x:.1}" cy="{y:.1}" r="4" fill="{relleno}" stroke="{}" stroke-width="1.5"/>"#,
            c.pal.stroke_main
        ));
        if blanco {
            caja_vacia(c, x, y - 15.0, 44.0, 13.0);
        } else if let Some(esp) = lt.especies.get(k) {
            tq(
                c,
                x,
                y - 10.0,
                esp,
                10.0,
                600,
                "textMain",
                "middle",
                SANS,
                true,
            );
        }
    }

    if !des.is_empty() {
        for &k in &des {
            if let (Some(&(oa, na)), Some(&(ob, nb))) = (
                k.checked_sub(1).and_then(|idx| puntos.get(idx)),
                puntos.get(k + 1),
            ) {
                ejes.guia(c, oa, na, ob, nb, "3 3", "strokeSub");
            }
        }
        c.t(
            x0 + 8.0,
            y0 + 16.0,
            "○ desproporciona (queda por encima de la recta entre sus vecinas)",
            9.0,
            400,
            "textSub",
            "start",
            SANS,
        );
    }

    Ok((x0 + w + 10.0, y0 + h + 40.0))
}

// --- Born-Haber -----------------------------------------------------------------------------------------

fn nivel_bh(c: &mut Lienzo, x1: f64, x2: f64, ye: f64, texto: &str, al_final: bool) {
    c.linea(x1, ye, x2, ye, "strokeMain", 1.6, None, "");
    if al_final {
        tq(
            c,
            x2 + 4.0,
            ye + 4.0,
            texto,
            10.0,
            400,
            "textMain",
            "start",
            SANS,
            false,
        );
    } else {
        tq(
            c,
            x1 + 2.0,
            ye - 5.0,
            texto,
            9.5,
            400,
            "textMain",
            "start",
            SANS,
            true,
        );
    }
}

fn render_born_haber(c: &mut Lienzo, b: &Bloque) -> Result<Punto, ErrorDiagrama> {
    let bh = envolver_error("born_haber", chem::born_haber(b))?;

    let mut etiquetas = Vec::with_capacity(bh.pasos.len());
    let mut ocultos = BTreeSet::new();
    if let Some(Value::Array(steps_arr)) = b.get("steps") {
        for (i, p) in steps_arr.iter().enumerate() {
            if let Some(map) = p.as_object() {
                let lbl = map
                    .get("label")
                    .filter(|v| !v.is_null())
                    .map(valor_str)
                    .unwrap_or_default();
                etiquetas.push(lbl);
                if booleano(map, "blank", false) {
                    ocultos.insert(i);
                }
            } else {
                etiquetas.push(String::new());
            }
        }
    }
    while etiquetas.len() < bh.pasos.len() {
        etiquetas.push(String::new());
    }

    let mut niveles = Vec::with_capacity(bh.pasos.len() + 1);
    niveles.push(0.0_f64);
    for (_, dh) in &bh.pasos {
        let ult = niveles.last().copied().unwrap_or(0.0);
        let sig = ult + dh;
        if !sig.is_finite() {
            return Err(ErrorDiagrama::new(
                "born_haber: las energías acumuladas superan el rango finito",
            ));
        }
        niveles.push(sig);
    }

    let energia_red = bh.energia_red();
    if !energia_red.is_finite() || !bh.formacion.is_finite() {
        return Err(ErrorDiagrama::new(
            "born_haber: energías fuera de rango finito",
        ));
    }

    let mut lo = bh.formacion;
    let mut hi = bh.formacion;
    for &e in &niveles {
        lo = lo.min(e);
        hi = hi.max(e);
    }
    let span = hi - lo;
    if !span.is_finite() {
        return Err(ErrorDiagrama::new(
            "born_haber: rango de energías fuera de límites finitos",
        ));
    }

    let top = encabezado_q(c, b);
    let y0 = top + 20.0;
    let alto = 340.0_f64;
    let py = |e: f64| -> f64 {
        if span.abs() < 1e-12 {
            y0 + alto / 2.0
        } else {
            y0 + alto - (e - lo) / span * alto
        }
    };

    let x_inicio = 40.0_f64;
    let paso_x = 112.0_f64;
    let trazo = c.pal.stroke_main;
    let acento = c.pal.accent_primary;

    nivel_bh(
        c,
        x_inicio - 20.0,
        x_inicio + 60.0,
        py(0.0),
        &bh.inicio,
        false,
    );

    for (k, ((estado, dh), etiqueta)) in bh.pasos.iter().zip(etiquetas.iter()).enumerate() {
        let x = x_inicio + 60.0 + paso_x * k as f64;
        let e0 = niveles.get(k).copied().unwrap_or(0.0);
        let e1 = niveles.get(k + 1).copied().unwrap_or(0.0);
        let y_e0 = py(e0);
        let y_e1 = py(e1);
        c.add(format!(
            r#"<line x1="{x:.1}" y1="{y_e0:.1}" x2="{x:.1}" y2="{y_e1:.1}" stroke="{acento}" stroke-width="1.5" marker-end="url(#flecha-main-{})"/>"#,
            c.sufijo
        ));
        nivel_bh(c, x - 14.0, x + paso_x, y_e1, estado, false);
        let texto = if etiqueta.is_empty() {
            String::new()
        } else {
            format!("{etiqueta} ")
        };
        let y_texto = (y_e0 + y_e1) / 2.0 + 4.0;
        if ocultos.contains(&k) {
            let prefijo = format!("{texto}=");
            tq(
                c,
                x + 5.0,
                y_texto,
                &prefijo,
                9.0,
                700,
                "textMain",
                "start",
                SANS,
                true,
            );
            c.vacio(
                x + 6.0 + ancho_formula(&prefijo, 9.0),
                x + 50.0 + ancho_formula(&texto, 9.0),
                y_texto + 2.0,
            );
        } else {
            let signo = if *dh > 0.0 { "+" } else { "" };
            let lbl = format!("{texto}{signo}{}", fmt_g(Some(*dh), 4));
            tq(
                c,
                x + 5.0,
                y_texto,
                &lbl,
                9.0,
                700,
                "textMain",
                "start",
                SANS,
                true,
            );
        }
    }

    let x_red = x_inicio + 60.0 + paso_x * bh.pasos.len() as f64;
    let e_iones = niveles.last().copied().unwrap_or(0.0);
    let y_iones = py(e_iones);
    let y_form = py(bh.formacion);
    let y_cero = py(0.0);

    c.add(format!(
        r#"<line x1="{x_red:.1}" y1="{y_iones:.1}" x2="{x_red:.1}" y2="{y_form:.1}" stroke="{acento}" stroke-width="2" marker-end="url(#flecha-main-{})"/>"#,
        c.sufijo
    ));
    nivel_bh(
        c,
        x_inicio - 20.0,
        x_red + 20.0,
        y_form,
        &bh.producto,
        true,
    );
    let u_str = format!("{} kJ/mol", fmt_g(Some(energia_red), 4));
    rotulo_q(
        c,
        x_red + 6.0,
        (y_iones + y_form) / 2.0,
        "U_red",
        Some(&u_str),
        booleano(b, "blankLattice", false),
        "start",
        10.0,
    );

    c.add(format!(
        r#"<line x1="{x_inicio:.1}" y1="{y_cero:.1}" x2="{x_inicio:.1}" y2="{y_form:.1}" stroke="{trazo}" stroke-width="1.3" stroke-dasharray="4 3" marker-end="url(#flecha-main-{})"/>"#,
        c.sufijo
    ));
    tq(
        c,
        x_inicio + 4.0,
        (y_cero + y_form) / 2.0,
        &format!("ΔH_f = {}", fmt_g(Some(bh.formacion), 4)),
        9.0,
        700,
        "textMain",
        "start",
        SANS,
        true,
    );
    c.t(
        0.0,
        y0 + alto + 24.0,
        "Energías en kJ/mol.",
        9.0,
        400,
        "textSub",
        "start",
        SANS,
    );

    Ok((x_red + 150.0, y0 + alto + 30.0))
}

#[cfg(test)]
mod tests {
    use super::*;
    use dibu_core::leer_bloques;
    use serde_json::json;

    fn reg_inorganica() -> Registro {
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
    fn fixtures_inorganica_en_ambas_paletas() {
        let reg = reg_inorganica();
        let yaml = include_str!("../../../fixtures/diagramas_quimica.yaml");
        let bloques = leer_bloques(yaml).unwrap();
        let inorg_ids = [
            "campo-co-nh3",
            "campo-fe-agua",
            "campo-tetraedrico",
            "campo-cuadrado-plano",
            "campo-blanco",
            "complejo-trans",
            "complejo-fac-blanco",
            "complejo-cisplatino",
            "latimer-manganeso",
            "latimer-cobre-blanco",
            "frost-manganeso",
            "born-haber-nacl",
            "born-haber-blanco",
        ];
        let mut vistos = 0usize;
        for b in &bloques {
            let id = b.get("id").and_then(Value::as_str).unwrap_or("");
            if inorg_ids.contains(&id) {
                vistos += 1;
                for tinta in [true, false] {
                    let svg = reg.renderizar(b, tinta).unwrap();
                    validar_svg(&svg);
                }
            }
        }
        assert_eq!(vistos, inorg_ids.len());
    }

    #[test]
    fn crystal_field_geometrias_espin_y_blanks() {
        let reg = reg_inorganica();

        // Octaédrico bajo espín NH3
        let svg_nh3 = reg
            .renderizar(
                &bloque_de(json!({
                    "type": "crystal_field",
                    "title": "[Co(NH3)6]^3+",
                    "metal": "Co",
                    "oxidation": 3,
                    "ligand": "NH3"
                })),
                true,
            )
            .unwrap();
        validar_svg(&svg_nh3);
        let txt_nh3 = texto_plano(&svg_nh3);
        assert!(txt_nh3.contains("d6:t2g6eg0"));
        assert!(txt_nh3.contains("bajo(NH3:campofuerte)"));
        assert!(txt_nh3.contains("Desapareados:0"));
        assert!(txt_nh3.contains("−2,4Δo"));

        // Octaédrico alto espín H2O (EECC 0,0)
        let txt_h2o = texto_plano(
            &reg.renderizar(
                &bloque_de(json!({
                    "type": "crystal_field",
                    "complex": "[Fe(H2O)6]^3+",
                    "metal": "Fe",
                    "oxidation": 3,
                    "ligand": "H2O"
                })),
                true,
            )
            .unwrap(),
        );
        assert!(txt_h2o.contains("d5:t2g3eg2"));
        assert!(txt_h2o.contains("5,92μB"));
        assert!(txt_h2o.contains("0,0Δo"));

        // Tetraédrico siempre alto espín
        let txt_tet = texto_plano(
            &reg.renderizar(
                &bloque_de(json!({
                    "type": "crystal_field",
                    "geometry": "tetrahedral",
                    "metal": "Co",
                    "oxidation": 2
                })),
                false,
            )
            .unwrap(),
        );
        assert!(txt_tet.contains("d7:e4t23"));
        assert!(txt_tet.contains("−1,2Δt"));

        // Cuadrado plano sin EECC ni flecha Δ
        let txt_sq = texto_plano(
            &reg.renderizar(
                &bloque_de(json!({
                    "type": "crystal_field",
                    "geometry": "square_planar",
                    "metal": "Ni",
                    "oxidation": 2
                })),
                true,
            )
            .unwrap(),
        );
        assert!(txt_sq.contains("dxz,dyz") && txt_sq.contains("dx²−y²"));
        assert!(txt_sq.contains("d8") && !txt_sq.contains("EECC:"));

        // d0 (Ti4+) y La (Z=57) + electrons + spin
        let txt_d0 = texto_plano(
            &reg.renderizar(
                &bloque_de(json!({
                    "type": "crystal_field",
                    "metal": "Ti",
                    "oxidation": 4,
                    "ligand": "Cl^-"
                })),
                true,
            )
            .unwrap(),
        );
        assert!(txt_d0.contains("d0:t2g0eg0") && txt_d0.contains("0,0Δo"));

        let svg_la = reg
            .renderizar(
                &bloque_de(json!({
                    "type": "crystal_field",
                    "metal": "La",
                    "oxidation": 0,
                    "inkSaver": true
                })),
                false,
            )
            .unwrap();
        assert!(texto_plano(&svg_la).contains("d3:t2g3eg0"));
        assert!(!svg_la.contains("#1d4ed8"));

        // blankElectrons + blank + blanks selectivos
        let svg_bl = reg
            .renderizar(
                &bloque_de(json!({
                    "type": "crystal_field",
                    "electrons": 6,
                    "spin": "low",
                    "blankElectrons": true,
                    "blank": true
                })),
                true,
            )
            .unwrap();
        let txt_bl = texto_plano(&svg_bl);
        assert!(!txt_bl.contains("d6") && !txt_bl.contains("bajo") && !txt_bl.contains("−2,4"));

        let txt_sel = texto_plano(
            &reg.renderizar(
                &bloque_de(json!({
                    "type": "crystal_field",
                    "electrons": 4,
                    "spin": "low",
                    "blankConfig": true,
                    "blankSpin": true,
                    "blankUnpaired": true,
                    "blankMoment": true,
                    "blankCFSE": true
                })),
                true,
            )
            .unwrap(),
        );
        assert!(!txt_sel.contains("d4") && !txt_sel.contains("2,83"));
    }

    #[test]
    fn crystal_field_errores() {
        let reg = reg_inorganica();
        let e1 = reg
            .renderizar(
                &bloque_de(json!({"type": "crystal_field", "geometry": "octahedral"})),
                true,
            )
            .unwrap_err()
            .to_string();
        assert!(e1.contains("falta electrons"), "{e1}");

        let e2 = reg
            .renderizar(
                &bloque_de(json!({"type": "crystal_field", "metal": "Na", "oxidation": 1})),
                true,
            )
            .unwrap_err()
            .to_string();
        assert!(e2.contains("bloque d"), "{e2}");

        let e3 = reg
            .renderizar(
                &bloque_de(json!({"type": "crystal_field", "electrons": 5, "ligand": "agua"})),
                true,
            )
            .unwrap_err()
            .to_string();
        assert!(e3.contains("serie espectroquímica"), "{e3}");

        let e4 = reg
            .renderizar(
                &bloque_de(json!({"type": "crystal_field", "electrons": 5, "spin": "medium"})),
                true,
            )
            .unwrap_err()
            .to_string();
        assert!(e4.contains("spin vale high o low"), "{e4}");

        let e5 = reg
            .renderizar(
                &bloque_de(json!({"type": "crystal_field", "electrons": "abc"})),
                true,
            )
            .unwrap_err()
            .to_string();
        assert!(e5.contains("mal formado"), "{e5}");
    }

    #[test]
    fn complex_geometrias_formula_isomeros_y_blanks() {
        let reg = reg_inorganica();

        // trans, fac, mer
        let txt_trans = texto_plano(
            &reg.renderizar(
                &bloque_de(json!({
                    "type": "complex",
                    "metal": "Co",
                    "charge": 1,
                    "ligands": ["NH3", "NH3", "Cl", "Cl", "NH3", "NH3"]
                })),
                true,
            )
            .unwrap(),
        );
        assert!(txt_trans.contains("[Co(NH3)4Cl2]+"));
        assert!(txt_trans.contains("Co(III)"));
        assert!(txt_trans.contains("trans"));

        let txt_fac = texto_plano(
            &reg.renderizar(
                &bloque_de(json!({
                    "type": "complex",
                    "metal": "Co",
                    "ligands": ["Cl", "NH3", "Cl", "NH3", "Cl", "NH3"]
                })),
                true,
            )
            .unwrap(),
        );
        assert!(txt_fac.contains("fac") && txt_fac.contains("[CoCl3(NH3)3]"));

        let txt_mer = texto_plano(
            &reg.renderizar(
                &bloque_de(json!({
                    "type": "complex",
                    "metal": "Co",
                    "ligands": ["Cl", "Cl", "Cl", "NH3", "NH3", "NH3"]
                })),
                true,
            )
            .unwrap(),
        );
        assert!(txt_mer.contains("mer"));

        // square_planar cis
        let txt_cis = texto_plano(
            &reg.renderizar(
                &bloque_de(json!({
                    "type": "complex",
                    "geometry": "square_planar",
                    "metal": "Pt",
                    "ligands": ["Cl", "NH3", "Cl", "NH3"]
                })),
                false,
            )
            .unwrap(),
        );
        assert!(txt_cis.contains("Pt(II)") && txt_cis.contains("d8") && txt_cis.contains("cis"));

        // tetrahedral aniónico sin Isómero
        let txt_tet = texto_plano(
            &reg.renderizar(
                &bloque_de(json!({
                    "type": "complex",
                    "geometry": "tetrahedral",
                    "metal": "Fe",
                    "charge": -1,
                    "ligands": ["Cl^-", "Cl^-", "Cl^-", "Cl^-"]
                })),
                true,
            )
            .unwrap(),
        );
        assert!(txt_tet.contains("[FeCl4]−") && txt_tet.contains("Fe(III)") && !txt_tet.contains("Isómero:"));

        // oxidation 0 con PF3
        let txt_ni0 = texto_plano(
            &reg.renderizar(
                &bloque_de(json!({
                    "type": "complex",
                    "geometry": "tetrahedral",
                    "metal": "Ni",
                    "charge": 0,
                    "oxidation": 0,
                    "ligands": ["PF3", "PF3", "PF3", "PF3"]
                })),
                true,
            )
            .unwrap(),
        );
        assert!(txt_ni0.contains("[Ni(PF3)4]") && txt_ni0.contains("Ni(0)") && txt_ni0.contains("d10"));

        // Al omite Electrones d
        let txt_al = texto_plano(
            &reg.renderizar(
                &bloque_de(json!({
                    "type": "complex",
                    "geometry": "tetrahedral",
                    "metal": "Al",
                    "charge": -1,
                    "ligands": ["Cl^-", "Cl^-", "Cl^-", "Cl^-"]
                })),
                true,
            )
            .unwrap(),
        );
        assert!(txt_al.contains("Al(III)") && !txt_al.contains("Electronesd:"));

        // Fe(-2) negativo
        let txt_fe_neg = texto_plano(
            &reg.renderizar(
                &bloque_de(json!({
                    "type": "complex",
                    "geometry": "tetrahedral",
                    "metal": "Fe",
                    "charge": -2,
                    "ligands": ["CO", "CO", "CO", "CO"]
                })),
                true,
            )
            .unwrap(),
        );
        assert!(txt_fe_neg.contains("[Fe(CO)4]2−") && txt_fe_neg.contains("Fe(-2)"));

        // blank: true y blank: [isomer, oxidation]
        let txt_bl_all = texto_plano(
            &reg.renderizar(
                &bloque_de(json!({
                    "type": "complex",
                    "metal": "Co",
                    "ligands": ["Cl", "NH3", "Cl", "NH3", "Cl", "NH3"],
                    "blank": true
                })),
                true,
            )
            .unwrap(),
        );
        assert!(!txt_bl_all.contains("Co(III)") && !txt_bl_all.contains("fac"));

        let txt_bl_list = texto_plano(
            &reg.renderizar(
                &bloque_de(json!({
                    "type": "complex",
                    "metal": "Co",
                    "ligands": ["Cl", "NH3", "Cl", "NH3", "Cl", "NH3"],
                    "blank": ["isomer", "oxidation"]
                })),
                true,
            )
            .unwrap(),
        );
        assert!(txt_bl_list.contains("[CoCl3(NH3)3]") && txt_bl_list.contains("d6"));
        assert!(!txt_bl_list.contains("Co(III)") && !txt_bl_list.contains("fac"));
    }

    #[test]
    fn complex_errores() {
        let reg = reg_inorganica();
        let e1 = reg
            .renderizar(
                &bloque_de(json!({"type": "complex", "geometry": "linear", "ligands": ["Cl", "Cl"]})),
                true,
            )
            .unwrap_err()
            .to_string();
        assert!(e1.contains("geometry vale octahedral"), "{e1}");

        let e2 = reg
            .renderizar(
                &bloque_de(json!({"type": "complex", "ligands": ["Cl", "Cl", "Cl", "Cl", "Cl"]})),
                true,
            )
            .unwrap_err()
            .to_string();
        assert!(e2.contains("6 sitios"), "{e2}");

        let e3 = reg
            .renderizar(
                &bloque_de(json!({"type": "complex", "metal": "Co", "ligands": ["X", "X", "X", "X", "X", "X"]})),
                true,
            )
            .unwrap_err()
            .to_string();
        assert!(e3.contains("no sé la carga del ligando"), "{e3}");
    }

    #[test]
    fn latimer_arcos_desproporcion_y_blanks() {
        let reg = reg_inorganica();

        let svg_mn = reg
            .renderizar(
                &bloque_de(json!({
                    "type": "latimer",
                    "species": ["MnO4^-", "MnO4^2-", "MnO2", "Mn^3+", "Mn^2+", "Mn"],
                    "oxidation": [7, 6, 4, 3, 2, 0],
                    "potentials": [0.56, 2.26, 0.95, 1.51, -1.18],
                    "compute": [[0, 2], [0, 4]],
                    "showDisproportionation": true
                })),
                true,
            )
            .unwrap();
        validar_svg(&svg_mn);
        assert!(svg_mn.contains("marker-end=\"url(#flecha-main-"));
        let txt_mn = texto_plano(&svg_mn);
        assert!(txt_mn.contains("E°=1,69V"));
        assert!(txt_mn.contains("E°=1,51V"));
        assert!(txt_mn.contains("Desproporcionan=MnO42−,Mn3+"));

        // Fe sin desproporción
        let txt_fe = texto_plano(
            &reg.renderizar(
                &bloque_de(json!({
                    "type": "latimer",
                    "species": ["Fe^3+", "Fe^2+", "Fe"],
                    "oxidation": [3, 2, 0],
                    "potentials": [0.77, -0.44],
                    "showDisproportionation": true
                })),
                false,
            )
            .unwrap(),
        );
        assert!(txt_fe.contains("Desproporcionan=ninguna"));

        // compute invertido [3, 0] + oxidación fraccionaria
        let txt_inv = texto_plano(
            &reg.renderizar(
                &bloque_de(json!({
                    "type": "latimer",
                    "species": ["N2", "NH3OH^+", "N2H5^+", "NH4^+"],
                    "oxidation": [0, -1, -2, -3],
                    "potentials": [-1.87, 1.41, 1.28],
                    "compute": [[3, 0]]
                })),
                true,
            )
            .unwrap(),
        );
        assert!(txt_inv.contains("E°=0,27V"));

        let txt_frac = texto_plano(
            &reg.renderizar(
                &bloque_de(json!({
                    "type": "latimer",
                    "species": ["S4O6^2-", "S2O3^2-"],
                    "oxidation": [2.5, 2],
                    "potentials": [0.08]
                })),
                true,
            )
            .unwrap(),
        );
        assert!(txt_frac.contains("2,5") && txt_frac.contains("+2"));

        // Blanks
        let txt_bl = texto_plano(
            &reg.renderizar(
                &bloque_de(json!({
                    "type": "latimer",
                    "species": ["Cu^2+", "Cu^+", "Cu"],
                    "oxidation": [2, 1, 0],
                    "potentials": [0.16, 0.52],
                    "compute": [[0, 2]],
                    "blankPotentials": [0, 1],
                    "blankComputed": true,
                    "showDisproportionation": true,
                    "blankDisproportionation": true
                })),
                true,
            )
            .unwrap(),
        );
        assert!(!txt_bl.contains("0,16") && !txt_bl.contains("0,52") && !txt_bl.contains("0,34"));
        assert!(txt_bl.contains("E°=") && txt_bl.contains("Desproporcionan="));
        assert!(!txt_bl.contains("Desproporcionan=Cu+"));
    }

    #[test]
    fn latimer_errores() {
        let reg = reg_inorganica();
        let e1 = reg
            .renderizar(
                &bloque_de(json!({
                    "type": "latimer",
                    "species": ["A", "B"],
                    "oxidation": [0, 2],
                    "potentials": [1.0]
                })),
                true,
            )
            .unwrap_err()
            .to_string();
        assert!(e1.contains("tienen que bajar"), "{e1}");

        let e2 = reg
            .renderizar(
                &bloque_de(json!({
                    "type": "latimer",
                    "species": ["A", "B"],
                    "oxidation": [2, 1],
                    "potentials": []
                })),
                true,
            )
            .unwrap_err()
            .to_string();
        assert!(e2.contains("uno por especie"), "{e2}");

        for comp in [json!([[0, 0]]), json!([[0, 5]])] {
            let e3 = reg
                .renderizar(
                    &bloque_de(json!({
                        "type": "latimer",
                        "species": ["A", "B"],
                        "oxidation": [2, 1],
                        "potentials": [0.5],
                        "compute": comp
                    })),
                    true,
                )
                .unwrap_err()
                .to_string();
            assert!(e3.contains("dos índices distintos"), "{e3}");
        }
    }

    #[test]
    fn frost_curva_secantes_y_bordes() {
        let reg = reg_inorganica();

        let svg_mn = reg
            .renderizar(
                &bloque_de(json!({
                    "type": "frost",
                    "species": ["MnO4^-", "MnO4^2-", "MnO2", "Mn^3+", "Mn^2+", "Mn"],
                    "oxidation": [7, 6, 4, 3, 2, 0],
                    "potentials": [0.56, 2.26, 0.95, 1.51, -1.18],
                    "showDisproportionation": true
                })),
                true,
            )
            .unwrap();
        validar_svg(&svg_mn);
        assert!(svg_mn.contains("stroke-dasharray=\"3 3\""));
        assert!(texto_plano(&svg_mn).contains("○desproporciona"));

        // Sin desproporción no dibuja la leyenda ○
        let svg_fe = reg
            .renderizar(
                &bloque_de(json!({
                    "type": "frost",
                    "species": ["Fe^3+", "Fe^2+", "Fe"],
                    "oxidation": [3, 2, 0],
                    "potentials": [0.77, -0.44],
                    "showDisproportionation": true
                })),
                false,
            )
            .unwrap();
        assert!(!texto_plano(&svg_fe).contains("○desproporciona"));

        // Sin oxidación 0
        let svg_no_zero = reg
            .renderizar(
                &bloque_de(json!({
                    "type": "frost",
                    "species": ["FeO4^2-", "Fe^3+", "Fe^2+"],
                    "oxidation": [6, 3, 2],
                    "potentials": [2.20, 0.77]
                })),
                true,
            )
            .unwrap();
        validar_svg(&svg_no_zero);

        // blankLabels
        let txt_bl = texto_plano(
            &reg.renderizar(
                &bloque_de(json!({
                    "type": "frost",
                    "species": ["Cu^2+", "Cu^+", "Cu"],
                    "oxidation": [2, 1, 0],
                    "potentials": [0.16, 0.52],
                    "blankLabels": true,
                    "showDisproportionation": true
                })),
                true,
            )
            .unwrap(),
        );
        assert!(!txt_bl.contains("Cu2+") && !txt_bl.contains("Cu+"));
        assert!(txt_bl.contains("○desproporciona"));

        // Potenciales nulos (dy == 0) sin NaN
        let svg_zeros = reg
            .renderizar(
                &bloque_de(json!({
                    "type": "frost",
                    "species": ["A", "B"],
                    "oxidation": [2, 0],
                    "potentials": [0.0]
                })),
                true,
            )
            .unwrap();
        validar_svg(&svg_zeros);

        // Extremo numérico 1e308 / -1e308: error controlado, nunca NaN ni inf ni panic
        let res_ext = reg.renderizar(
            &bloque_de(json!({
                "type": "frost",
                "species": ["A", "B"],
                "oxidation": [1e308, -1e308],
                "potentials": [1.0]
            })),
            true,
        );
        if let Ok(svg) = res_ext {
            validar_svg(&svg);
        }

        // Error de latimer propagado con prefijo frost:
        let e = reg
            .renderizar(
                &bloque_de(json!({
                    "type": "frost",
                    "species": ["A", "B"],
                    "oxidation": [1, 2],
                    "potentials": [0.5]
                })),
                true,
            )
            .unwrap_err()
            .to_string();
        assert!(e.starts_with("frost:") && e.contains("tienen que bajar"), "{e}");
    }

    #[test]
    fn born_haber_ciclo_blanks_y_bordes() {
        let reg = reg_inorganica();

        let svg_nacl = reg
            .renderizar(
                &bloque_de(json!({
                    "type": "born_haber",
                    "start": "Na(s) + ½ Cl2(g)",
                    "product": "NaCl(s)",
                    "formation": -411,
                    "steps": [
                        {"to": "Na(g) + ½ Cl2(g)", "dH": 107, "label": "ΔH_sub"},
                        {"to": "Na^+(g) + ½ Cl2(g)", "dH": 496, "label": "EI"},
                        {"to": "Na^+(g) + Cl(g)", "dH": 122, "label": "½ D"},
                        {"to": "Na^+(g) + Cl^-(g)", "dH": -349, "label": "AE"}
                    ]
                })),
                true,
            )
            .unwrap();
        validar_svg(&svg_nacl);
        let txt_nacl = texto_plano(&svg_nacl);
        assert!(txt_nacl.contains("Ured=-787kJ/mol") && txt_nacl.contains("ΔHf=-411"));

        // Defaults start/product + formación endotérmica
        let txt_def = texto_plano(
            &reg.renderizar(
                &bloque_de(json!({
                    "type": "born_haber",
                    "formation": 300,
                    "steps": [{"to": "iones(g)", "dH": 1000}]
                })),
                true,
            )
            .unwrap(),
        );
        assert!(txt_def.contains("elementos") && txt_def.contains("MX(s)"));
        assert!(txt_def.contains("Ured=-700kJ/mol") && txt_def.contains("ΔHf=300"));

        // blankLattice y steps[i].blank con y sin label
        let txt_bl = texto_plano(
            &reg.renderizar(
                &bloque_de(json!({
                    "type": "born_haber",
                    "formation": -100,
                    "blankLattice": true,
                    "steps": [
                        {"to": "A", "dH": 50, "blank": true},
                        {"to": "B", "dH": -20, "label": "AE", "blank": true}
                    ]
                })),
                true,
            )
            .unwrap(),
        );
        assert!(!txt_bl.contains("-130") && !txt_bl.contains("+50") && !txt_bl.contains("-20"));
        assert!(txt_bl.contains("AE=") && txt_bl.contains("Ured="));

        // Caso borde lo == hi (dH: 0, formation: 0) sin NaN
        let svg_zero = reg
            .renderizar(
                &bloque_de(json!({
                    "type": "born_haber",
                    "formation": 0,
                    "steps": [{"to": "A", "dH": 0}]
                })),
                true,
            )
            .unwrap();
        validar_svg(&svg_zero);
    }

    #[test]
    fn born_haber_errores() {
        let reg = reg_inorganica();
        let e1 = reg
            .renderizar(
                &bloque_de(json!({"type": "born_haber", "steps": [], "formation": -100})),
                true,
            )
            .unwrap_err()
            .to_string();
        assert!(e1.contains("faltan steps"), "{e1}");

        let e2 = reg
            .renderizar(
                &bloque_de(json!({"type": "born_haber", "steps": [{"to": "A", "dH": 100}]})),
                true,
            )
            .unwrap_err()
            .to_string();
        assert!(e2.contains("faltan steps"), "{e2}");

        let e3 = reg
            .renderizar(
                &bloque_de(json!({
                    "type": "born_haber",
                    "steps": [{"to": "A"}],
                    "formation": -100
                })),
                true,
            )
            .unwrap_err()
            .to_string();
        assert!(e3.contains("steps[0]"), "{e3}");

        let e4 = reg
            .renderizar(
                &bloque_de(json!({
                    "type": "born_haber",
                    "steps": [{"to": "A", "dH": "abc"}],
                    "formation": -100
                })),
                true,
            )
            .unwrap_err()
            .to_string();
        assert!(e4.contains("no es un número"), "{e4}");
    }
}
