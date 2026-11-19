//! Diagramas de control en ejes cartesianos: `time_response`, `bode`, `nyquist`, `pole_zero`, `root_locus`.

use crate::math::{
    self, abierto_de, asintota_db, asintotas, caracteristicas, cruce_imaginario, fase_continua,
    lazo, lugar_raices, magnitud_db, margenes, numero, raices, rango_frecuencias, simular_abierto,
    simular_lazo, sistema, suma, tangente_inflexion, Lazo, Perturbacion, Sistema, C64,
};
use dibu_core::{
    ancho_texto, bool_opt, encabezado, str_opt, Bloque, ErrorDiagrama, Lienzo, Punto, MONO,
};
use dibu_plot::{fmt_g, leyenda, rotulo, Ejes, EjesConfig};
use serde_json::Value;
use std::f64::consts::PI;

enum ItemSistema {
    Abierto(Sistema),
    Cerrado(Lazo),
}

fn sistemas_de(b: &Bloque, clave: &str) -> Result<Vec<(String, ItemSistema)>, ErrorDiagrama> {
    let mut salida = Vec::new();
    if let Some(cl) = b.get("closedLoop") {
        let lz = lazo(cl, "closedLoop")?;
        salida.push((lz.etiqueta.clone(), ItemSistema::Cerrado(lz)));
    }
    if let Some(sys) = b.get("system") {
        let s = sistema(sys, "system")?;
        salida.push((s.etiqueta.clone(), ItemSistema::Abierto(s)));
    }
    if let Some(v) = b.get(clave) {
        if !v.is_null() {
            let Some(arr) = v.as_array() else {
                return Err(ErrorDiagrama::new(format!(
                    "'{clave}' tiene que ser una lista de sistemas"
                )));
            };
            for (i, dato) in arr.iter().enumerate() {
                if let Some(obj) = dato.as_object() {
                    if let Some(cl) = obj.get("closedLoop") {
                        let lz = lazo(cl, &format!("{clave}[{i}].closedLoop"))?;
                        let etiq = obj
                            .get("label")
                            .and_then(Value::as_str)
                            .filter(|s| !s.is_empty())
                            .map(str::to_string)
                            .unwrap_or_else(|| lz.etiqueta.clone());
                        salida.push((etiq, ItemSistema::Cerrado(lz)));
                        continue;
                    }
                }
                let s = sistema(dato, &format!("{clave}[{i}]"))?;
                salida.push((s.etiqueta.clone(), ItemSistema::Abierto(s)));
            }
        }
    }
    Ok(salida)
}

fn tiempo_final(sistemas: &[ItemSistema]) -> f64 {
    let mut max_t = 0.0_f64;
    for s in sistemas {
        let (base, retardo) = match s {
            ItemSistema::Cerrado(lz) => (lz.cerrado(), lz.planta.delay),
            ItemSistema::Abierto(sys) => (sys.clone(), sys.delay),
        };
        let min_neg = base
            .polos()
            .into_iter()
            .filter(|p| p.re < -1e-9)
            .map(|p| -p.re)
            .fold(f64::INFINITY, f64::min);
        let t = (if min_neg.is_finite() { 6.0 / min_neg } else { 10.0 }) + 2.0 * retardo;
        max_t = max_t.max(t);
    }
    if max_t <= 0.0 {
        10.0
    } else {
        max_t.min(1e4)
    }
}

fn f64_opt(b: &Bloque, k: &str) -> Result<Option<f64>, ErrorDiagrama> {
    match b.get(k) {
        None | Some(Value::Null) => Ok(None),
        Some(v) => Ok(Some(numero(v, k)?)),
    }
}

const ANOTACIONES: &[&str] = &[
    "final",
    "overshoot",
    "peak",
    "rise",
    "settling",
    "tau",
    "delay",
    "tangent",
];

pub fn render_time_response(c: &mut Lienzo, b: &Bloque) -> Result<Punto, ErrorDiagrama> {
    let tipo = str_opt(b, "input").unwrap_or("step");
    let amplitud = f64_opt(b, "amplitude")?.unwrap_or(1.0);
    let sistemas = sistemas_de(b, "systems")?;
    let datos = b.get("data").filter(|v| !v.is_null());
    if sistemas.is_empty() && datos.is_none() {
        return Err(ErrorDiagrama::new(
            "time_response: falta system, systems, closedLoop o data (puntos medidos)",
        ));
    }
    let items_ref: Vec<&ItemSistema> = sistemas.iter().map(|(_, s)| s).collect();
    let _ = &items_ref;
    let t_fin_calc = if !sistemas.is_empty() {
        let solo: Vec<ItemSistema> = sistemas
            .iter()
            .map(|(_, s)| match s {
                ItemSistema::Abierto(x) => ItemSistema::Abierto(x.clone()),
                ItemSistema::Cerrado(x) => ItemSistema::Cerrado(x.clone()),
            })
            .collect();
        tiempo_final(&solo)
    } else {
        10.0
    };
    let mut t_fin = f64_opt(b, "tEnd")?.filter(|&t| t > 0.0).unwrap_or(t_fin_calc);

    let pert_obj = b.get("disturbance").and_then(Value::as_object);
    let perturbacion = if let Some(p) = pert_obj {
        Some(Perturbacion {
            at: p.get("at").and_then(Value::as_f64).unwrap_or(0.0),
            amplitude: p.get("amplitude").and_then(Value::as_f64).unwrap_or(0.0),
            where_at: p.get("where").and_then(Value::as_str).unwrap_or("output"),
        })
    } else {
        None
    };

    let mut curvas: Vec<(String, Vec<f64>, Vec<f64>)> = Vec::new();
    let mut referencia: Option<(Vec<f64>, Vec<f64>)> = None;
    let mut finales: Vec<Option<f64>> = Vec::new();

    for (etiq, item) in &sistemas {
        match item {
            ItemSistema::Cerrado(lz) => {
                let (ts, rs, ys, _) = simular_lazo(lz, t_fin, tipo, amplitud, perturbacion, 4000)?;
                if referencia.is_none() {
                    referencia = Some((ts.clone(), rs));
                }
                let cerrado = lz.cerrado();
                let estable = lz.planta.delay == 0.0 && cerrado.polos().iter().all(|p| p.re < -1e-9);
                let k = cerrado.ganancia_estatica();
                finales.push(if tipo == "step" && estable && perturbacion.is_none() {
                    k.map(|kv| amplitud * kv)
                } else {
                    None
                });
                curvas.push((etiq.clone(), ts, ys));
            }
            ItemSistema::Abierto(s) => {
                let (ts, us, ys) = simular_abierto(s, t_fin, tipo, amplitud, 3000)?;
                if referencia.is_none() {
                    referencia = Some((ts.clone(), us));
                }
                let estable = s.polos().iter().all(|p| p.re < -1e-9);
                let k = s.ganancia_estatica();
                finales.push(if tipo == "step" && estable {
                    k.map(|kv| amplitud * kv)
                } else {
                    None
                });
                curvas.push((etiq.clone(), ts, ys));
            }
        }
    }

    if let Some(d_val) = datos {
        let Some(arr) = d_val.as_array() else {
            return Err(ErrorDiagrama::new("time_response: 'data' es una lista de [t, y]"));
        };
        let mut pts = Vec::with_capacity(arr.len());
        for p in arr {
            let Some(par) = p.as_array() else {
                return Err(ErrorDiagrama::new("time_response: 'data' es una lista de [t, y]"));
            };
            if par.len() < 2 {
                return Err(ErrorDiagrama::new("time_response: 'data' es una lista de [t, y]"));
            }
            pts.push((numero(&par[0], "data")?, numero(&par[1], "data")?));
        }
        pts.sort_by(|a, b| a.0.total_cmp(&b.0));
        if let Some(last) = pts.last() {
            t_fin = if sistemas.is_empty() {
                last.0.max(1e-6)
            } else {
                t_fin.max(last.0)
            };
        }
        let etiq = str_opt(b, "dataLabel").unwrap_or("medición").to_string();
        curvas.push((
            etiq,
            pts.iter().map(|p| p.0).collect(),
            pts.iter().map(|p| p.1).collect(),
        ));
        finales.push(None);
    }

    let top = encabezado(c, b, "");
    let (x0, y0, w, h) = (58.0, top + 16.0, 460.0, 250.0);
    let show_input = bool_opt(b, "showInput", true);
    let mut y_min = 0.0_f64;
    let mut y_max = f64::NEG_INFINITY;
    for (_, _, ys) in &curvas {
        for &y in ys {
            if y.is_finite() {
                y_min = y_min.min(y);
                y_max = y_max.max(y);
            }
        }
    }
    if show_input && tipo != "impulse" {
        if let Some((_, rs)) = &referencia {
            for &y in rs {
                if y.is_finite() {
                    y_min = y_min.min(y);
                    y_max = y_max.max(y);
                }
            }
        }
    }
    if !y_max.is_finite() {
        y_max = 1.0;
    }
    if let Some(ym) = f64_opt(b, "yMax")? {
        y_max = ym;
    }
    if let Some(ym) = f64_opt(b, "yMin")? {
        y_min = ym;
    }
    let margen = {
        let m = (y_max - y_min) * 0.12;
        if m == 0.0 { 1.0 } else { m }
    };
    let xlabel = str_opt(b, "xLabel").unwrap_or("t (s)");
    let ylabel = str_opt(b, "yLabel").unwrap_or("y(t)");
    let ejes = Ejes::dibujar(
        c,
        EjesConfig {
            x0,
            y0,
            w,
            h,
            xr: (0.0, t_fin),
            yr: (y_min, y_max + margen),
            xlabel,
            ylabel,
            log: false,
            grilla: true,
            origen: true,
            ypaso: None,
        },
    );
    if show_input && tipo != "impulse" {
        if let Some((ts, rs)) = &referencia {
            let pts: Vec<Punto> = ts.iter().copied().zip(rs.iter().copied()).collect();
            ejes.curva(c, &pts, 2, 1.2, "strokeSub");
        }
    }
    for (i, (_, ts, ys)) in curvas.iter().enumerate() {
        let pts: Vec<Punto> = ts.iter().copied().zip(ys.iter().copied()).collect();
        ejes.curva(c, &pts, i, 1.8, "strokeMain");
    }

    let pedidas: Vec<String> = match b.get("annotations") {
        None | Some(Value::Null) => Vec::new(),
        Some(Value::String(s)) if s == "all" => ANOTACIONES.iter().map(|s| s.to_string()).collect(),
        Some(Value::Array(arr)) => arr
            .iter()
            .filter_map(|v| v.as_str().map(str::to_string))
            .collect(),
        Some(otro) => {
            return Err(ErrorDiagrama::new(format!(
                "time_response: anotaciones inválidas {otro}"
            )))
        }
    };
    let desconocidas: Vec<&str> = pedidas
        .iter()
        .map(String::as_str)
        .filter(|a| !ANOTACIONES.contains(a))
        .collect();
    if !desconocidas.is_empty() {
        return Err(ErrorDiagrama::new(format!(
            "time_response: anotaciones desconocidas {:?} (valen {})",
            desconocidas,
            ANOTACIONES.join(", ")
        )));
    }
    let tiene = |name: &str| pedidas.iter().any(|p| p == name);
    let blanco = bool_opt(b, "blankAnnotations", false);
    let mut rotulos: Vec<(String, String)> = Vec::new();

    if !pedidas.is_empty() && !curvas.is_empty() {
        let (_, ts_full, ys_full) = &curvas[0];
        let corte = if let Some(p) = pert_obj {
            if let Some(at) = p.get("at").and_then(Value::as_f64) {
                ts_full.iter().position(|&t| t >= at).unwrap_or(ts_full.len())
            } else {
                ts_full.len()
            }
        } else {
            ts_full.len()
        };
        let ts = &ts_full[..corte];
        let ys = &ys_full[..corte];
        let banda = f64_opt(b, "settlingBand")?.unwrap_or(0.02);
        let car = caracteristicas(ts, ys, finales[0], banda, 0.0);
        if let Some(fin) = car.final_val {
            if tiene("final") {
                ejes.guia(c, 0.0, fin, t_fin, fin, "4 3", "strokeSub");
                rotulos.push(("y∞".into(), fmt_g(Some(fin), 3)));
            }
            if (tiene("overshoot") || tiene("peak")) && car.peak.is_some() {
                let tp = car.peak.unwrap();
                let pico = if fin >= 0.0 {
                    ys.iter().copied().fold(f64::NEG_INFINITY, f64::max)
                } else {
                    ys.iter().copied().fold(f64::INFINITY, f64::min)
                };
                ejes.guia(c, tp, y_min, tp, pico, "4 3", "strokeSub");
                c.punto(ejes.px(tp), ejes.py(pico), "accentPrimary", 3.0);
                if tiene("overshoot") {
                    rotulos.push(("Mp".into(), format!("{} %", fmt_g(car.overshoot, 3))));
                }
                if tiene("peak") {
                    rotulos.push(("tp".into(), fmt_g(Some(tp), 3)));
                    c.t(ejes.px(tp), y0 + h - 4.0, "tp", 9.0, 700, "textMain", "middle", MONO);
                }
            }
            if tiene("rise") && car.rise.is_some() {
                let signo = if fin > 0.0 { 1.0 } else { -1.0 };
                for frac in [0.1, 0.9] {
                    let nivel = frac * fin;
                    if let Some((&cruce, _)) = ts.iter().zip(ys.iter()).find(|(_, &y)| (y - nivel) * signo >= 0.0) {
                        ejes.guia(c, 0.0, nivel, cruce, nivel, "1 3", "strokeSub");
                        ejes.guia(c, cruce, y_min, cruce, nivel, "1 3", "strokeSub");
                    }
                }
                rotulos.push(("tr".into(), fmt_g(car.rise, 3)));
            }
            if tiene("settling") && car.settling.is_some() {
                let ts_v = car.settling.unwrap();
                for signo in [1.0, -1.0] {
                    let lv = fin * (1.0 + signo * banda);
                    ejes.guia(c, 0.0, lv, t_fin, lv, "1 2", "strokeMuted");
                }
                ejes.guia(c, ts_v, y_min, ts_v, fin, "4 3", "strokeSub");
                c.t(ejes.px(ts_v), y0 + h - 4.0, "ts", 9.0, 700, "textMain", "middle", MONO);
                rotulos.push((format!("ts ({} %)", fmt_g(Some(banda * 100.0), 3)), fmt_g(Some(ts_v), 3)));
            }
            if tiene("delay") && car.delay.is_some() {
                let d_v = car.delay.unwrap();
                ejes.guia(c, d_v, y_min, d_v, fin * 0.3, "4 3", "strokeSub");
                c.t(ejes.px(d_v), y0 + h - 4.0, "θ", 10.0, 700, "textMain", "middle", MONO);
                rotulos.push(("θ".into(), fmt_g(Some(d_v), 3)));
            }
            if tiene("tau") && car.tau.is_some() && car.delay.is_some() {
                let t63 = car.delay.unwrap() + car.tau.unwrap();
                ejes.guia(c, 0.0, 0.632 * fin, t63, 0.632 * fin, "4 3", "strokeSub");
                ejes.guia(c, t63, y_min, t63, 0.632 * fin, "4 3", "strokeSub");
                c.punto(ejes.px(t63), ejes.py(0.632 * fin), "accentPrimary", 3.0);
                c.t(x0 + 4.0, ejes.py(0.632 * fin) - 4.0, "63,2 %", 9.0, 400, "textSub", "start", MONO);
                rotulos.push(("τ".into(), fmt_g(car.tau, 3)));
            }
            if tiene("tangent") {
                if let Some(tg) = tangente_inflexion(ts, ys, fin, 0.0) {
                    let fin_t = tg.l + tg.big_t;
                    ejes.curva(c, &[(tg.l, 0.0), (fin_t, fin)], 1, 1.2, "strokeSub");
                    ejes.guia(c, fin_t, y_min, fin_t, fin, "4 3", "strokeSub");
                    c.t(ejes.px(tg.l), y0 + h - 4.0, "L", 10.0, 700, "textMain", "middle", MONO);
                    c.t(ejes.px(fin_t), y0 + h - 4.0, "L+T", 9.0, 700, "textMain", "middle", MONO);
                    rotulos.push(("L".into(), fmt_g(Some(tg.l), 3)));
                    rotulos.push(("T".into(), fmt_g(Some(tg.big_t), 3)));
                }
            }
        }
    }

    let mut ancho = x0 + w;
    let xr = x0 + w + 14.0;
    let mut yy = y0 + 10.0;
    let etiquetas: Vec<String> = curvas
        .iter()
        .map(|(e, _, _)| e.clone())
        .filter(|e| !e.is_empty())
        .collect();
    if !etiquetas.is_empty() {
        yy += leyenda(c, xr, yy, &etiquetas) + 8.0;
        let max_e = etiquetas
            .iter()
            .map(|e| ancho_texto(e, 10.0))
            .fold(0.0_f64, f64::max);
        ancho = ancho.max(xr + 32.0 + max_e);
    }
    for (nombre, valor) in &rotulos {
        ancho = ancho.max(xr + rotulo(c, xr, yy, nombre, valor, blanco, "start", 10.0));
        yy += 17.0;
    }
    Ok((ancho + 6.0, (y0 + h + 40.0).max(yy)))
}

pub fn render_bode(c: &mut Lienzo, b: &Bloque) -> Result<Punto, ErrorDiagrama> {
    let sistemas: Vec<(String, Sistema)> = sistemas_de(b, "systems")?
        .into_iter()
        .map(|(e, item)| {
            let s = match item {
                ItemSistema::Cerrado(lz) => lz.abierto(),
                ItemSistema::Abierto(s) => s,
            };
            (e, s)
        })
        .collect();
    if sistemas.is_empty() {
        return Err(ErrorDiagrama::new("bode: falta system, systems o closedLoop"));
    }
    let rangos: Vec<(f64, f64)> = sistemas.iter().map(|(_, s)| rango_frecuencias(s)).collect();
    let w_min = f64_opt(b, "wMin")?
        .filter(|&x| x > 0.0)
        .unwrap_or_else(|| rangos.iter().map(|r| r.0).fold(f64::INFINITY, f64::min));
    let w_max = f64_opt(b, "wMax")?
        .filter(|&x| x > 0.0)
        .unwrap_or_else(|| rangos.iter().map(|r| r.1).fold(0.0_f64, f64::max));
    if !(w_min > 0.0 && w_min < w_max) {
        return Err(ErrorDiagrama::new("bode: wMin y wMax son positivas y wMin < wMax"));
    }
    let ws: Vec<f64> = (0..=400)
        .map(|k| w_min * (w_max / w_min).powf(k as f64 / 400.0))
        .collect();
    let mostrar = str_opt(b, "show").unwrap_or("both");
    if !matches!(mostrar, "both" | "magnitude" | "phase") {
        return Err(ErrorDiagrama::new("bode: show es both, magnitude o phase"));
    }
    let top = encabezado(c, b, "");
    let (x0, w, h) = (64.0, 460.0, 170.0);
    let mut y = top + 14.0;
    let blanco = bool_opt(b, "blankMargins", false);
    let mg = if bool_opt(b, "margins", false) {
        Some(margenes(&sistemas[0].1, w_min, w_max))
    } else {
        None
    };

    let mut ejes_mag: Option<Ejes> = None;
    let mut ejes_fase: Option<Ejes> = None;

    if matches!(mostrar, "both" | "magnitude") {
        let mags: Vec<Vec<f64>> = sistemas
            .iter()
            .map(|(_, s)| ws.iter().map(|&x| magnitud_db(s, x)).collect())
            .collect();
        let lo_raw = mags.iter().flatten().copied().fold(f64::INFINITY, f64::min);
        let hi_raw = mags.iter().flatten().copied().fold(f64::NEG_INFINITY, f64::max);
        let hi = (hi_raw.max(0.0) / 20.0).ceil() * 20.0
            + if (hi_raw % 20.0).abs() < 1e-12 { 20.0 } else { 0.0 };
        let lo = ((lo_raw.min(-20.0) / 20.0).floor() * 20.0).max(hi - 200.0);
        let em = Ejes::dibujar(
            c,
            EjesConfig {
                x0,
                y0: y,
                w,
                h,
                xr: (w_min, w_max),
                yr: (lo, hi),
                xlabel: if mostrar == "both" { "" } else { "ω (rad/s)" },
                ylabel: "|G(jω)| (dB)",
                log: true,
                grilla: true,
                origen: false,
                ypaso: Some(if hi - lo <= 120.0 { 20.0 } else { 40.0 }),
            },
        );
        em.guia(c, w_min, 0.0, w_max, 0.0, "", "strokeSub");
        for (i, (_, s)) in sistemas.iter().enumerate() {
            let pts: Vec<Punto> = ws.iter().copied().zip(mags[i].iter().copied()).collect();
            em.curva(c, &pts, i, 1.8, "strokeMain");
            if bool_opt(b, "asymptotes", false) {
                let apts: Vec<Punto> = ws.iter().map(|&x| (x, asintota_db(s, x))).collect();
                em.curva(c, &apts, 1, 1.1, "strokeSub");
            }
        }
        ejes_mag = Some(em);
        y += h + if mostrar == "both" { 26.0 } else { 44.0 };
    }

    if matches!(mostrar, "both" | "phase") {
        let fases: Vec<Vec<f64>> = sistemas
            .iter()
            .map(|(_, s)| ws.iter().map(|&x| fase_continua(s, x)).collect())
            .collect();
        let lo_raw = fases.iter().flatten().copied().fold(f64::INFINITY, f64::min);
        let hi_raw = fases.iter().flatten().copied().fold(f64::NEG_INFINITY, f64::max);
        let hi = (hi_raw.max(0.0) / 45.0).ceil() * 45.0;
        let lo = ((lo_raw.min(-180.0) / 45.0).floor() * 45.0).max(-720.0);
        let ef = Ejes::dibujar(
            c,
            EjesConfig {
                x0,
                y0: y,
                w,
                h,
                xr: (w_min, w_max),
                yr: (lo, hi),
                xlabel: "ω (rad/s)",
                ylabel: "∠G(jω) (°)",
                log: true,
                grilla: true,
                origen: false,
                ypaso: Some(if hi - lo <= 270.0 { 45.0 } else { 90.0 }),
            },
        );
        ef.guia(c, w_min, -180.0, w_max, -180.0, "6 3", "strokeSub");
        for (i, f) in fases.iter().enumerate() {
            let pts: Vec<Punto> = ws.iter().copied().zip(f.iter().copied()).collect();
            ef.curva(c, &pts, i, 1.8, "strokeMain");
        }
        ejes_fase = Some(ef);
        y += h + 44.0;
    }

    let mut ancho = x0 + w;
    let xr = x0 + w + 14.0;
    let mut yy = top + 24.0;
    let etiquetas: Vec<String> = sistemas
        .iter()
        .map(|(e, _)| e.clone())
        .filter(|e| !e.is_empty())
        .collect();
    if !etiquetas.is_empty() {
        yy += leyenda(c, xr, yy, &etiquetas) + 8.0;
        let max_e = etiquetas.iter().map(|e| ancho_texto(e, 10.0)).fold(0.0_f64, f64::max);
        ancho = ancho.max(xr + 32.0 + max_e);
    }
    if let Some(m) = mg {
        if let (Some(w180), Some(gm), Some(em)) = (m.w180, m.gm, &ejes_mag) {
            em.guia(c, w180, 0.0, w180, -gm, "4 3", "strokeSub");
            c.punto(em.px(w180), em.py(-gm), "accentPrimary", 3.0);
            c.t(em.px(w180) + 4.0, em.py(-gm / 2.0), "MG", 9.0, 700, "textMain", "start", MONO);
        }
        if let (Some(wc), Some(pm), Some(ef)) = (m.wc, m.pm, &ejes_fase) {
            ef.guia(c, wc, -180.0, wc, pm - 180.0, "4 3", "strokeSub");
            c.punto(ef.px(wc), ef.py(pm - 180.0), "accentPrimary", 3.0);
            c.t(ef.px(wc) + 4.0, ef.py(pm / 2.0 - 180.0), "MF", 9.0, 700, "textMain", "start", MONO);
        }
        let filas = [
            ("MG", m.gm.map_or_else(|| "∞".into(), |v| format!("{} dB", fmt_g(Some(v), 3)))),
            ("ω180", fmt_g(m.w180, 3)),
            ("MF", m.pm.map_or_else(|| "—".into(), |v| format!("{}°", fmt_g(Some(v), 3)))),
            ("ωc", fmt_g(m.wc, 3)),
        ];
        for (nombre, valor) in filas {
            ancho = ancho.max(xr + rotulo(c, xr, yy, nombre, &valor, blanco, "start", 10.0));
            yy += 17.0;
        }
    }
    Ok((ancho + 6.0, y - 10.0))
}

pub fn render_nyquist(c: &mut Lienzo, b: &Bloque) -> Result<Punto, ErrorDiagrama> {
    let sistemas: Vec<(String, Sistema)> = sistemas_de(b, "systems")?
        .into_iter()
        .map(|(e, item)| {
            let s = match item {
                ItemSistema::Cerrado(lz) => lz.abierto(),
                ItemSistema::Abierto(s) => s,
            };
            (e, s)
        })
        .collect();
    if sistemas.is_empty() {
        return Err(ErrorDiagrama::new("nyquist: falta system, systems o closedLoop"));
    }
    let rangos: Vec<(f64, f64)> = sistemas.iter().map(|(_, s)| rango_frecuencias(s)).collect();
    let w_min = f64_opt(b, "wMin")?
        .filter(|&x| x > 0.0)
        .unwrap_or_else(|| rangos.iter().map(|r| r.0).fold(f64::INFINITY, f64::min) / 10.0);
    let w_max = f64_opt(b, "wMax")?
        .filter(|&x| x > 0.0)
        .unwrap_or_else(|| rangos.iter().map(|r| r.1).fold(0.0_f64, f64::max) * 10.0);
    let ws: Vec<f64> = (0..=1200)
        .map(|k| w_min * (w_max / w_min).powf(k as f64 / 1200.0))
        .collect();
    let curvas: Vec<Vec<C64>> = sistemas
        .iter()
        .map(|(_, s)| ws.iter().map(|&w| s.eval(C64::new(0.0, w))).collect())
        .collect();
    let mut modulos: Vec<f64> = curvas
        .iter()
        .flatten()
        .map(|z| z.norm())
        .filter(|a| a.is_finite())
        .collect();
    modulos.sort_by(f64::total_cmp);
    let p90 = if modulos.is_empty() {
        1.5
    } else {
        modulos[(modulos.len() * 9 / 10).min(modulos.len() - 1)]
    };
    let limite = f64_opt(b, "limit")?
        .filter(|&x| x > 0.0)
        .unwrap_or_else(|| (p90 * 1.1).max(1.5).min(50.0));
    let mut re_lo = -1.3_f64;
    let mut re_hi = 0.5_f64;
    let mut im_lim = 1.0_f64;
    for curva in &curvas {
        for z in curva {
            if z.norm() <= limite {
                re_lo = re_lo.min(z.re);
                re_hi = re_hi.max(z.re);
                im_lim = im_lim.max(z.im.abs());
            }
        }
    }
    let alto_r = 2.0 * im_lim * 1.1;
    let ancho_r = (re_hi - re_lo) * 1.15;
    let escala = (460.0 / ancho_r).min(340.0 / alto_r);
    let (w, h) = (ancho_r * escala, alto_r * escala);
    let top = encabezado(c, b, "");
    let (x0, y0) = (58.0, top + 16.0);
    let margen_re = (re_hi - re_lo) * 0.075;
    let ejes = Ejes::dibujar(
        c,
        EjesConfig {
            x0,
            y0,
            w,
            h,
            xr: (re_lo - margen_re, re_hi + margen_re),
            yr: (-alto_r / 2.0, alto_r / 2.0),
            xlabel: "Re",
            ylabel: "Im",
            log: false,
            grilla: true,
            origen: true,
            ypaso: None,
        },
    );
    if bool_opt(b, "unitCircle", false) {
        let circ: Vec<Punto> = (0..=120)
            .map(|a| {
                let t = a as f64 / 60.0 * PI;
                (t.cos(), t.sin())
            })
            .collect();
        ejes.curva(c, &circ, 2, 0.9, "strokeMuted");
    }
    let show_neg = bool_opt(b, "showNegative", true);
    for (i, curva) in curvas.iter().enumerate() {
        let positivo: Vec<Punto> = curva.iter().map(|z| (z.re, z.im)).collect();
        ejes.curva(c, &positivo, i, 1.8, "strokeMain");
        if show_neg {
            let neg: Vec<Punto> = positivo.iter().map(|&(x, y)| (x, -y)).collect();
            ejes.curva(c, &neg, 2, 1.0, "strokeSub");
        }
        for frac in [0.35, 0.6] {
            let k = (positivo.len() as f64 * frac) as usize;
            let k2 = (k + 4).min(positivo.len().saturating_sub(1));
            let (xa, ya) = positivo[k];
            let (xb, yb) = positivo[k2];
            let a = (ejes.px(xa), ejes.py(ya));
            let bp = (ejes.px(xb), ejes.py(yb));
            if ejes.adentro(a.0, a.1) && ejes.adentro(bp.0, bp.1) && (a.0 - bp.0).hypot(a.1 - bp.1) > 0.5 {
                c.linea(a.0, a.1, bp.0, bp.1, "strokeMain", 1.2, Some("main"), "");
            }
        }
    }
    let (cx, cy) = (ejes.px(-1.0), ejes.py(0.0));
    c.linea(cx - 6.0, cy, cx + 6.0, cy, "strokeMain", 2.0, None, "");
    c.linea(cx, cy - 6.0, cx, cy + 6.0, "strokeMain", 2.0, None, "");
    c.t(cx - 4.0, cy - 9.0, "−1", 10.0, 700, "textMain", "end", MONO);
    let mut ancho = x0 + w;
    let etiquetas: Vec<String> = sistemas
        .iter()
        .map(|(e, _)| e.clone())
        .filter(|e| !e.is_empty())
        .collect();
    if !etiquetas.is_empty() {
        leyenda(c, x0 + w + 14.0, y0 + 10.0, &etiquetas);
        let max_e = etiquetas.iter().map(|e| ancho_texto(e, 10.0)).fold(0.0_f64, f64::max);
        ancho = ancho.max(x0 + w + 46.0 + max_e);
    }
    Ok((ancho + 6.0, y0 + h + 40.0))
}

fn dibujar_x(c: &mut Lienzo, x: f64, y: f64, r: f64) {
    c.linea(x - r, y - r, x + r, y + r, "strokeMain", 2.0, None, "");
    c.linea(x - r, y + r, x + r, y - r, "strokeMain", 2.0, None, "");
}

fn dibujar_o(c: &mut Lienzo, x: f64, y: f64, r: f64) {
    c.add(format!(
        r##"<circle cx="{x:.1}" cy="{y:.1}" r="{}" fill="#ffffff" stroke="{}" stroke-width="2"/>"##,
        dibu_core::fmt_num_clean(r),
        c.pal.stroke_main
    ));
}

fn plano(c: &mut Lienzo, b: &Bloque, puntos: &[C64], minimo: f64) -> Result<Ejes, ErrorDiagrama> {
    let mut escala = minimo;
    for p in puntos {
        escala = escala.max(p.re.abs()).max(p.im.abs());
    }
    let mut re_lo = (-escala * 0.3_f64).min(puntos.iter().map(|p| p.re).fold(f64::INFINITY, f64::min)) - escala * 0.3;
    let mut re_hi = (escala * 0.3_f64).max(puntos.iter().map(|p| p.re).fold(f64::NEG_INFINITY, f64::max)) + escala * 0.3;
    let mut im = (escala * 0.5_f64).max(puntos.iter().map(|p| p.im.abs()).fold(0.0_f64, f64::max)) * 1.3;
    if let Some(arr) = b.get("xRange").and_then(Value::as_array) {
        if arr.len() == 2 {
            re_lo = numero(&arr[0], "xRange")?;
            re_hi = numero(&arr[1], "xRange")?;
        }
    }
    if let Some(arr) = b.get("yRange").and_then(Value::as_array) {
        if !arr.is_empty() {
            im = arr
                .iter()
                .map(|v| numero(v, "yRange").map(f64::abs))
                .collect::<Result<Vec<_>, _>>()?
                .into_iter()
                .fold(0.0_f64, f64::max);
        }
    }
    let px_por = (440.0 / (re_hi - re_lo)).min(340.0 / (2.0 * im));
    let top = encabezado(c, b, "");
    Ok(Ejes::dibujar(
        c,
        EjesConfig {
            x0: 58.0,
            y0: top + 16.0,
            w: (re_hi - re_lo) * px_por,
            h: 2.0 * im * px_por,
            xr: (re_lo, re_hi),
            yr: (-im, im),
            xlabel: "σ (Re s)",
            ylabel: "jω (Im s)",
            log: false,
            grilla: true,
            origen: true,
            ypaso: None,
        },
    ))
}

fn amortiguamiento(c: &mut Lienzo, ejes: &Ejes, b: &Bloque) -> Result<(), ErrorDiagrama> {
    let lejos = ejes.xr.0.abs().max(ejes.yr.1) * 2.0;
    if let Some(arr) = b.get("dampingLines").and_then(Value::as_array) {
        for v in arr {
            let zeta = numero(v, "dampingLines")?;
            if !(0.0..=1.0).contains(&zeta) {
                return Err(ErrorDiagrama::new("dampingLines: cada ζ va de 0 a 1"));
            }
            let ang = zeta.acos();
            for signo in [1.0, -1.0] {
                ejes.curva(
                    c,
                    &[(0.0, 0.0), (-lejos * ang.cos(), signo * lejos * ang.sin())],
                    1,
                    0.9,
                    "strokeMuted",
                );
            }
            let t = (ejes.yr.1 * 0.9 / ang.sin().max(1e-9)).min(ejes.xr.0.abs() * 0.9 / ang.cos().max(1e-9));
            let x = ejes.px(-t * ang.cos());
            let y = ejes.py(t * ang.sin());
            c.t(x - 6.0, y + 4.0, &format!("ζ = {}", fmt_g(Some(zeta), 3)), 9.0, 400, "textSub", "end", MONO);
        }
    }
    if let Some(arr) = b.get("wnCircles").and_then(Value::as_array) {
        for v in arr {
            let wn = numero(v, "wnCircles")?;
            let pts: Vec<Punto> = (30..=90)
                .map(|a| {
                    let t = a as f64 / 60.0 * PI;
                    (wn * t.cos(), wn * t.sin())
                })
                .collect();
            ejes.curva(c, &pts, 2, 0.9, "strokeMuted");
            c.t(
                ejes.px(-wn),
                ejes.py(0.0) + 12.0,
                &format!("ωn = {}", fmt_g(Some(wn), 3)),
                9.0,
                400,
                "textSub",
                "middle",
                MONO,
            );
        }
    }
    Ok(())
}

pub fn render_pole_zero(c: &mut Lienzo, b: &Bloque) -> Result<Punto, ErrorDiagrama> {
    let (polos, ceros) = if b.contains_key("system") {
        let s = abierto_de(b, "system")?;
        (s.polos(), s.ceros())
    } else if let Some(cl) = b.get("closedLoop") {
        let s = lazo(cl, "closedLoop")?.cerrado();
        (s.polos(), s.ceros())
    } else {
        // Reutilizamos sistema con poles/zeros
        let mut tmp = serde_json::Map::new();
        tmp.insert("poles".into(), b.get("poles").cloned().unwrap_or(Value::Array(vec![])));
        tmp.insert("zeros".into(), b.get("zeros").cloned().unwrap_or(Value::Array(vec![])));
        let s = sistema(&Value::Object(tmp), "pole_zero")?;
        (s.polos(), s.ceros())
    };
    let mut todos = polos.clone();
    todos.extend(ceros.iter().copied());
    let ejes = plano(c, b, &todos, 1.0)?;
    amortiguamiento(c, &ejes, b)?;
    if !bool_opt(b, "blank", false) {
        for z in &ceros {
            dibujar_o(c, ejes.px(z.re), ejes.py(z.im), 5.0);
        }
        for p in &polos {
            dibujar_x(c, ejes.px(p.re), ejes.py(p.im), 5.0);
        }
        if bool_opt(b, "showValues", false) {
            for r in polos.iter().chain(ceros.iter()) {
                let mut texto = fmt_g(Some(r.re), 3);
                if r.im.abs() > 1e-9 {
                    let sig = if r.im >= 0.0 { "+" } else { "−" };
                    texto.push_str(&format!(" {sig} j{}", fmt_g(Some(r.im.abs()), 3)));
                }
                c.t(ejes.px(r.re) + 8.0, ejes.py(r.im) - 7.0, &texto, 9.0, 400, "textSub", "start", MONO);
            }
        }
    }
    Ok((ejes.x0 + ejes.w + 60.0, ejes.y0 + ejes.h + 40.0))
}

pub fn render_root_locus(c: &mut Lienzo, b: &Bloque) -> Result<Punto, ErrorDiagrama> {
    let s = abierto_de(b, "system")?;
    let polos = s.polos();
    let ceros = s.ceros();
    if polos.is_empty() {
        return Err(ErrorDiagrama::new("root_locus: el sistema no tiene polos"));
    }
    let escala = polos
        .iter()
        .chain(ceros.iter())
        .map(|r| r.norm())
        .fold(1.0_f64, f64::max);
    let mut k_max = f64_opt(b, "kMax")?.unwrap_or(0.0);
    if k_max <= 0.0 {
        k_max = 1.0;
        while k_max < 1e7 {
            let kn: Vec<C64> = s.num.iter().map(|&x| x * k_max).collect();
            let max_r = raices(&suma(&s.den, &kn))
                .into_iter()
                .map(|r| r.norm())
                .fold(0.0_f64, f64::max);
            if max_r >= 3.0 * escala {
                break;
            }
            k_max *= 2.0;
        }
    }
    let (_, ramas) = lugar_raices(&s, k_max, 400);
    let cruce = if bool_opt(b, "showCrossing", true) {
        cruce_imaginario(&s, k_max)
    } else {
        None
    };
    let mut extras = polos.clone();
    extras.extend(ceros.iter().copied());
    if let Some((_, wc)) = cruce {
        extras.push(C64::new(0.0, wc));
    }
    let ejes = plano(c, b, &extras, escala)?;
    amortiguamiento(c, &ejes, b)?;
    let blanco = bool_opt(b, "blankMarks", false);
    if bool_opt(b, "asymptotes", true) {
        let (centro_opt, angulos) = asintotas(&s);
        if let Some(centro) = centro_opt {
            if angulos.len() > 1 {
                let lejos = 3.0 * escala;
                for a in angulos {
                    let rad = a.to_radians();
                    ejes.curva(
                        c,
                        &[(centro, 0.0), (centro + lejos * rad.cos(), lejos * rad.sin())],
                        1,
                        0.9,
                        "strokeMuted",
                    );
                }
                c.punto(ejes.px(centro), ejes.py(0.0), "strokeSub", 2.5);
                rotulo(
                    c,
                    ejes.px(centro),
                    ejes.py(0.0) + 16.0,
                    "σa",
                    &fmt_g(Some(centro), 3),
                    blanco,
                    "middle",
                    9.0,
                );
            }
        }
    }
    for rama in &ramas {
        let puntos: Vec<Punto> = rama.iter().map(|r| (r.re, r.im)).collect();
        ejes.curva(c, &puntos, 0, 2.0, "strokeMain");
        if !puntos.is_empty() {
            let k = puntos.len() / 3;
            let k2 = (k + 3).min(puntos.len() - 1);
            let a = (ejes.px(puntos[k].0), ejes.py(puntos[k].1));
            let bp = (ejes.px(puntos[k2].0), ejes.py(puntos[k2].1));
            if ejes.adentro(a.0, a.1) && ejes.adentro(bp.0, bp.1) && (a.0 - bp.0).hypot(a.1 - bp.1) > 1.0 {
                c.linea(a.0, a.1, bp.0, bp.1, "strokeMain", 1.4, Some("main"), "");
            }
        }
    }
    for z in &ceros {
        dibujar_o(c, ejes.px(z.re), ejes.py(z.im), 5.0);
    }
    for p in &polos {
        dibujar_x(c, ejes.px(p.re), ejes.py(p.im), 5.0);
    }
    let mut rotulos: Vec<(String, Option<String>)> = Vec::new();
    if let Some(arr) = b.get("marks").and_then(Value::as_array) {
        for kv in arr {
            let k = numero(kv, "marks")?;
            let kn: Vec<C64> = s.num.iter().map(|&x| x * k).collect();
            for r in math::raices(&suma(&s.den, &kn)) {
                let x = ejes.px(r.re);
                let y = ejes.py(r.im);
                if ejes.adentro(x, y) {
                    c.add(format!(
                        r#"<rect x="{:.1}" y="{:.1}" width="7" height="7" fill="{}"/>"#,
                        x - 3.5,
                        y - 3.5,
                        c.pal.stroke_main
                    ));
                }
            }
            rotulos.push((format!("■ K = {}", fmt_g(Some(k), 3)), None));
        }
    }
    if let Some((k_c, w_c)) = cruce {
        for signo in [1.0, -1.0] {
            c.punto(ejes.px(0.0), ejes.py(signo * w_c), "accentPrimary", 3.5);
        }
        rotulos.push(("Kcrít".into(), Some(fmt_g(Some(k_c), 3))));
        rotulos.push(("ωcrít".into(), Some(fmt_g(Some(w_c), 3))));
    }
    let xr = ejes.x0 + ejes.w + 14.0;
    let mut yy = ejes.y0 + 10.0;
    let mut ancho = ejes.x0 + ejes.w;
    for (nombre, valor) in rotulos {
        match valor {
            None => {
                c.t(xr, yy, &nombre, 10.0, 700, "textMain", "start", MONO);
                ancho = ancho.max(xr + ancho_texto(&nombre, 10.0));
            }
            Some(v) => {
                ancho = ancho.max(xr + rotulo(c, xr, yy, &nombre, &v, blanco, "start", 10.0));
            }
        }
        yy += 17.0;
    }
    Ok((ancho + 6.0, ejes.y0 + ejes.h + 40.0))
}
