//! Arquitectura de computadoras: `pipeline`, `ieee754` y `cache`.

use crate::comun::*;
use serde_json::Value;

// ---------------------------------------------------------------------------
// pipeline
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct Instr {
    pub texto: String,
    pub destino: Option<String>,
    pub fuentes: Vec<String>,
    pub carga: bool,
}

fn es_registro(t: &str) -> bool {
    let t = t.trim();
    t.starts_with('$')
        || (t.len() >= 2
            && (t.starts_with('x') || t.starts_with('r') || t.starts_with('R'))
            && t[1..].chars().all(|c| c.is_ascii_digit()))
        || matches!(t, "zero" | "ra" | "sp" | "gp" | "fp")
        || (t.len() >= 2
            && matches!(&t[..1], "a" | "t" | "s")
            && t[1..].chars().all(|c| c.is_ascii_digit()))
}

fn es_cero(r: &str) -> bool {
    matches!(r, "$zero" | "$0" | "x0" | "zero" | "r0" | "R0")
}

fn registros_en(op: &str) -> Vec<String> {
    // "0($t1)" → $t1 ; "$t2" → $t2
    let mut out = Vec::new();
    let t = op.trim();
    if let (Some(a), Some(z)) = (t.find('('), t.rfind(')')) {
        let r = t[a + 1..z].trim();
        if es_registro(r) {
            out.push(r.to_string());
        }
    } else if es_registro(t) {
        out.push(t.to_string());
    }
    out
}

pub fn analizar_instr(texto: &str) -> Instr {
    let texto = texto.trim();
    let (op, resto) = match texto.find(char::is_whitespace) {
        Some(i) => (&texto[..i], texto[i..].trim()),
        None => (texto, ""),
    };
    let op = op.to_lowercase();
    let ops: Vec<&str> = if resto.is_empty() {
        Vec::new()
    } else {
        resto.split(',').map(str::trim).collect()
    };
    let regs = |desde: usize| -> Vec<String> {
        ops.iter()
            .skip(desde)
            .flat_map(|o| registros_en(o))
            .collect()
    };
    let carga = matches!(
        op.as_str(),
        "lw" | "lb"
            | "lh"
            | "lbu"
            | "lhu"
            | "ld"
            | "lwu"
            | "flw"
            | "fld"
            | "ldr"
            | "ldrb"
            | "l.s"
            | "l.d"
    );
    let almacen = matches!(
        op.as_str(),
        "sw" | "sb" | "sh" | "sd" | "fsw" | "fsd" | "str" | "strb" | "s.s" | "s.d"
    );
    let salto = op.starts_with('b') && op != "bl"
        || matches!(
            op.as_str(),
            "j" | "jr" | "jal" | "jalr" | "nop" | "ret" | "halt"
        );
    let (destino, fuentes) = if carga {
        (ops.first().and_then(|o| registros_en(o).pop()), regs(1))
    } else if almacen || salto {
        (None, regs(0))
    } else {
        (ops.first().and_then(|o| registros_en(o).pop()), regs(1))
    };
    Instr {
        texto: texto.to_string(),
        destino: destino.filter(|d| !es_cero(d)),
        fuentes: fuentes.into_iter().filter(|f| !es_cero(f)).collect(),
        carga,
    }
}

/// Ciclos (1-based) de cada instrucción: IF, entrada a ID, EX. MEM = EX+1, WB = EX+2.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Tiempos {
    pub f: usize,
    pub d: usize,
    pub e: usize,
}

/// Dependencias RAW: (productor, consumidor, registro).
pub fn dependencias(instrs: &[Instr]) -> Vec<(usize, usize, String)> {
    let mut out = Vec::new();
    for (i, ins) in instrs.iter().enumerate() {
        for r in &ins.fuentes {
            if let Some(p) = (0..i)
                .rev()
                .find(|&p| instrs[p].destino.as_deref() == Some(r))
            {
                if !out
                    .iter()
                    .any(|(pp, ii, rr): &(usize, usize, String)| *pp == p && *ii == i && rr == r)
                {
                    out.push((p, i, r.clone()));
                }
            }
        }
    }
    out
}

pub fn planificar(instrs: &[Instr], forwarding: bool) -> Vec<Tiempos> {
    let deps = dependencias(instrs);
    let mut t: Vec<Tiempos> = Vec::with_capacity(instrs.len());
    for i in 0..instrs.len() {
        let (f, d_min, e_prev) = match t.last() {
            None => (1, 2, 0),
            Some(p) => {
                let f = (p.f + 1).max(p.d);
                (f, (f + 1).max(p.e), p.e)
            }
        };
        let d = d_min;
        let mut e = (d + 1).max(e_prev + 1);
        for (p, _, _) in deps.iter().filter(|(_, c, _)| *c == i) {
            let ep = t[*p].e;
            let minimo = if !forwarding {
                ep + 3
            } else if instrs[*p].carga {
                ep + 2
            } else {
                ep + 1
            };
            e = e.max(minimo);
        }
        t.push(Tiempos { f, d, e });
    }
    t
}

fn leer_instrs(b: &Bloque) -> Result<Vec<Instr>, ErrorDiagrama> {
    let mut out = Vec::new();
    for (k, v) in lista(b, "instructions", false)?.iter().enumerate() {
        match v {
            Value::String(s) => out.push(analizar_instr(s)),
            Value::Object(m) => {
                let texto = m
                    .get("text")
                    .map(valor_str)
                    .ok_or_else(|| err(format!("instructions[{k}] necesita 'text'")))?;
                let mut ins = analizar_instr(&texto);
                if let Some(d) = m.get("dest") {
                    ins.destino = Some(valor_str(d)).filter(|s| !s.is_empty());
                }
                if let Some(Value::Array(src)) = m.get("src") {
                    ins.fuentes = src.iter().map(valor_str).collect();
                }
                if let Some(l) = m.get("load").and_then(Value::as_bool) {
                    ins.carga = l;
                }
                out.push(ins);
            }
            _ => return Err(err("cada instrucción es un texto o {text, dest, src}")),
        }
    }
    if out.is_empty() {
        return Err(err("'instructions' está vacío"));
    }
    if out.len() > 40 {
        return Err(err("como máximo 40 instrucciones"));
    }
    Ok(out)
}

pub fn render_pipeline(c: &mut Lienzo, b: &Bloque) -> Result<Punto, ErrorDiagrama> {
    let instrs = leer_instrs(b)?;
    let fwd = bool_opt(b, "forwarding", true);
    let blanco = bool_opt(b, "blank", false);
    let flechas = bool_opt(b, "arrows", true) && !blanco;
    let t = planificar(&instrs, fwd);
    let ciclos = t.iter().map(|x| x.e + 2).max().unwrap_or(5);
    let n_ciclos = if blanco {
        ciclos.max(instrs.len() + 4)
    } else {
        ciclos
    };
    let extra = format!(
        "{} · {}",
        if fwd {
            "con forwarding"
        } else {
            "sin forwarding"
        },
        "IF ID EX MEM WB"
    );
    let y0 = encabezado(c, b, &extra);
    let w_ins = instrs
        .iter()
        .map(|i| ancho_texto(&i.texto, 10.5))
        .fold(90.0_f64, f64::max)
        + 18.0;
    let cw = 38.0;
    let fh = 26.0;
    let top = y0 + 8.0;
    // Cabecera de ciclos.
    c.r(0.0, top, w_ins, fh, "fillHeader", 1.2, 0.0);
    c.t(
        8.0,
        top + fh / 2.0 + 4.0,
        "Instrucción",
        10.5,
        700,
        "textMain",
        "start",
        MONO,
    );
    for k in 0..n_ciclos {
        let x = w_ins + k as f64 * cw;
        c.r(x, top, cw, fh, "fillHeader", 1.2, 0.0);
        c.t(
            x + cw / 2.0,
            top + fh / 2.0 + 4.0,
            &(k + 1).to_string(),
            10.5,
            700,
            "textMain",
            "middle",
            MONO,
        );
    }
    let x_ciclo = |k: usize| w_ins + (k - 1) as f64 * cw;
    let mut stalls = 0usize;
    for (i, ins) in instrs.iter().enumerate() {
        let y = top + fh * (i + 1) as f64;
        c.r(0.0, y, w_ins, fh, "fillCell", 1.0, 0.0);
        c.t(
            8.0,
            y + fh / 2.0 + 4.0,
            &ins.texto,
            10.5,
            600,
            "textMain",
            "start",
            MONO,
        );
        for k in 1..=n_ciclos {
            c.r(x_ciclo(k), y, cw, fh, "fillCell", 0.6, 0.0);
        }
        if blanco {
            continue;
        }
        let ti = t[i];
        let mut celdas: Vec<(usize, &str, bool)> = vec![(ti.f, "IF", false)];
        for k in ti.f + 1..ti.d {
            celdas.push((k, "stall", true));
        }
        celdas.push((ti.d, "ID", false));
        for k in ti.d + 1..ti.e {
            celdas.push((k, "stall", true));
            stalls += 1;
        }
        celdas.extend([
            (ti.e, "EX", false),
            (ti.e + 1, "MEM", false),
            (ti.e + 2, "WB", false),
        ]);
        for (k, etiqueta, burbuja) in celdas {
            let x = x_ciclo(k);
            if burbuja {
                c.r(
                    x + 2.0,
                    y + 3.0,
                    cw - 4.0,
                    fh - 6.0,
                    "fillHighlight",
                    0.0,
                    3.0,
                );
                c.t(
                    x + cw / 2.0,
                    y + fh / 2.0 + 3.5,
                    etiqueta,
                    8.5,
                    500,
                    "textMuted",
                    "middle",
                    MONO,
                );
            } else {
                c.r(
                    x + 2.0,
                    y + 3.0,
                    cw - 4.0,
                    fh - 6.0,
                    "fillPointerBox",
                    1.0,
                    3.0,
                );
                c.t(
                    x + cw / 2.0,
                    y + fh / 2.0 + 4.0,
                    etiqueta,
                    10.0,
                    700,
                    "textMain",
                    "middle",
                    MONO,
                );
            }
        }
    }
    let ancho = w_ins + n_ciclos as f64 * cw;
    let mut alto = top + fh * (instrs.len() + 1) as f64;
    let deps = dependencias(&instrs);
    if flechas {
        for (p, q, _) in &deps {
            let (tp, tq) = (t[*p], t[*q]);
            let (yp, yq) = (top + fh * (*p + 1) as f64, top + fh * (*q + 1) as f64);
            // Con forwarding: fin de EX (o de MEM si es carga) → inicio de EX del consumidor.
            // Sin forwarding: WB (primera mitad) → ID del consumidor (segunda mitad).
            let (x1, x2, ydst) = if fwd {
                let k = if instrs[*p].carga { tp.e + 1 } else { tp.e };
                (x_ciclo(k) + cw - 4.0, x_ciclo(tq.e) + 6.0, yq + 4.0)
            } else {
                (
                    x_ciclo(tp.e + 2) + cw / 2.0,
                    x_ciclo(tq.e - 1) + cw / 2.0,
                    yq + 4.0,
                )
            };
            let y1 = yp + fh - 4.0;
            if x2 < x1 - 1.0 {
                continue; // no hace falta (dependencia lejana)
            }
            c.add(format!(
                r#"<path d="M {x1:.1} {y1:.1} C {x1:.1} {:.1} {x2:.1} {:.1} {x2:.1} {ydst:.1}" fill="none" stroke="{}" stroke-width="1.5" marker-end="url(#flecha-main-{})"/>"#,
                (y1 + ydst) / 2.0,
                (y1 + ydst) / 2.0,
                c.pal.color("accentPrimary"),
                c.sufijo
            ));
        }
    }
    if bool_opt(b, "summary", true) {
        alto += 18.0;
        let linea = if blanco {
            "Ciclos totales: ____ · Burbujas: ____ · CPI: ____".to_string()
        } else {
            let total = ciclos;
            format!(
                "Ciclos totales: {total} · Burbujas: {stalls} · CPI: {}",
                fmt((total - 4) as f64 / instrs.len() as f64, 2)
            )
        };
        c.t(0.0, alto, &linea, 10.5, 600, "textMain", "start", MONO);
        if bool_opt(b, "hazards", true) && !deps.is_empty() {
            for (p, q, r) in &deps {
                alto += 15.0;
                let txt = if blanco {
                    format!("I{} → I{}: ____", p + 1, q + 1)
                } else {
                    let tipo = if instrs[*p].carga && fwd {
                        " (load-use)"
                    } else {
                        ""
                    };
                    format!("RAW I{} → I{} por {r}{tipo}", p + 1, q + 1)
                };
                c.t(0.0, alto, &txt, 10.0, 400, "textSub", "start", MONO);
            }
        }
        alto += 6.0;
    }
    Ok((ancho, alto))
}

// ---------------------------------------------------------------------------
// ieee754
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy)]
pub struct Formato {
    pub nombre: &'static str,
    pub exp: u32,
    pub man: u32,
}

impl Formato {
    pub fn sesgo(&self) -> i64 {
        (1i64 << (self.exp - 1)) - 1
    }
    pub fn total(&self) -> u32 {
        1 + self.exp + self.man
    }
}

pub const HALF: Formato = Formato {
    nombre: "media precisión (16 bits)",
    exp: 5,
    man: 10,
};
pub const SINGLE: Formato = Formato {
    nombre: "precisión simple (32 bits)",
    exp: 8,
    man: 23,
};
pub const DOUBLE: Formato = Formato {
    nombre: "precisión doble (64 bits)",
    exp: 11,
    man: 52,
};

/// f64 → bits en el formato dado (redondeo al par más cercano).
pub fn codificar(v: f64, f: Formato) -> u64 {
    match f.total() {
        64 => v.to_bits(),
        32 => (v as f32).to_bits() as u64,
        _ => f32_a_f16((v as f32).to_bits()) as u64,
    }
}

fn f32_a_f16(x: u32) -> u16 {
    let signo = ((x >> 16) & 0x8000) as u16;
    let exp = ((x >> 23) & 0xff) as i32;
    let man = x & 0x7f_ffff;
    if exp == 0xff {
        return signo | 0x7c00 | if man != 0 { 0x200 } else { 0 };
    }
    let e = exp - 127 + 15;
    if e >= 0x1f {
        return signo | 0x7c00;
    }
    if e <= 0 {
        if e < -10 {
            return signo;
        }
        let m = man | 0x80_0000;
        let corr = (14 - e) as u32;
        let mut h = m >> corr;
        let resto = m & ((1 << corr) - 1);
        let mitad = 1 << (corr - 1);
        if resto > mitad || (resto == mitad && h & 1 == 1) {
            h += 1;
        }
        return signo | h as u16;
    }
    let mut h = ((e as u32) << 10) | (man >> 13);
    let resto = man & 0x1fff;
    if resto > 0x1000 || (resto == 0x1000 && h & 1 == 1) {
        h += 1;
    }
    signo | h as u16
}

/// Bits → valor f64 (exacto).
pub fn decodificar(bits: u64, f: Formato) -> f64 {
    let s = (bits >> (f.exp + f.man)) & 1;
    let e = ((bits >> f.man) & ((1 << f.exp) - 1)) as i64;
    let m = bits & ((1u64 << f.man) - 1);
    let signo = if s == 1 { -1.0 } else { 1.0 };
    let max_e = (1i64 << f.exp) - 1;
    if e == max_e {
        return if m == 0 {
            signo * f64::INFINITY
        } else {
            f64::NAN
        };
    }
    let frac = m as f64 / (1u64 << f.man) as f64;
    if e == 0 {
        signo * frac * 2f64.powi((1 - f.sesgo()) as i32)
    } else {
        signo * (1.0 + frac) * 2f64.powi((e - f.sesgo()) as i32)
    }
}

/// Representación binaria de |v| (parte entera y hasta `max_frac` dígitos fraccionarios).
fn binario(v: f64, max_frac: usize) -> (String, String, bool) {
    let a = v.abs();
    let ent = a.trunc();
    let entera = if ent == 0.0 {
        "0".to_string()
    } else if ent < 2f64.powi(100) {
        format!("{:b}", ent as u128)
    } else {
        "…".to_string()
    };
    let mut fr = a - ent;
    let mut frac = String::new();
    while fr > 0.0 && frac.len() < max_frac {
        fr *= 2.0;
        if fr >= 1.0 {
            frac.push('1');
            fr -= 1.0;
        } else {
            frac.push('0');
        }
    }
    (entera, frac, fr > 0.0)
}

fn sub2(n: i64) -> String {
    const SUP: [char; 10] = ['⁰', '¹', '²', '³', '⁴', '⁵', '⁶', '⁷', '⁸', '⁹'];
    let mut s = String::new();
    if n < 0 {
        s.push('⁻');
    }
    for ch in n.abs().to_string().chars() {
        s.push(SUP[ch.to_digit(10).unwrap() as usize]);
    }
    s
}

pub fn render_ieee754(c: &mut Lienzo, b: &Bloque) -> Result<Punto, ErrorDiagrama> {
    let f = match opcion(
        b,
        "format",
        &["half", "single", "double", "16", "32", "64"],
        "single",
    )?
    .as_str()
    {
        "half" | "16" => HALF,
        "double" | "64" => DOUBLE,
        _ => SINGLE,
    };
    let blanco = bool_opt(b, "blank", false);
    let (bits, valor_pedido) = if let Some(h) = b.get("bits").or_else(|| b.get("hex")) {
        let t = valor_str(h).replace(['_', ' '], "");
        let bits = if let Some(hx) = t.strip_prefix("0x").or_else(|| t.strip_prefix("0X")) {
            u64::from_str_radix(hx, 16)
                .map_err(|_| err(format!("'bits': {t:?} no es hexadecimal válido")))?
        } else if t.chars().all(|ch| ch == '0' || ch == '1') && t.len() == f.total() as usize {
            u64::from_str_radix(&t, 2).map_err(|_| err("'bits' no es binario válido"))?
        } else {
            return Err(err(format!(
                "'bits' tiene que ser 0x… o {} dígitos binarios",
                f.total()
            )));
        };
        if f.total() < 64 && bits >> f.total() != 0 {
            return Err(err(format!("'bits' no entra en {} bits", f.total())));
        }
        (bits, None)
    } else {
        let v = match b.get("value") {
            Some(Value::String(s)) => match s.trim().to_lowercase().as_str() {
                "inf" | "+inf" | "infinity" => f64::INFINITY,
                "-inf" | "-infinity" => f64::NEG_INFINITY,
                "nan" => f64::NAN,
                _ => s
                    .trim()
                    .replace(',', ".")
                    .parse::<f64>()
                    .map_err(|_| err(format!("'value': {s:?} no es un número")))?,
            },
            Some(v) => num(v).ok_or_else(|| err("'value' tiene que ser un número"))?,
            None => return Err(err("falta 'value' (número) o 'bits' (0x…)")),
        };
        (codificar(v, f), Some(v))
    };
    let s = (bits >> (f.exp + f.man)) & 1;
    let e = (bits >> f.man) & ((1u64 << f.exp) - 1);
    let m = bits & ((1u64 << f.man) - 1);
    let bin = |x: u64, n: u32| format!("{x:0width$b}", width = n as usize);
    let (s_b, e_b, m_b) = (bin(s, 1), bin(e, f.exp), bin(m, f.man));
    let almacenado = decodificar(bits, f);

    // Ejercicio: si se dio el valor, se ocultan los bits; si se dieron los bits, se muestran.
    let blanco_bits = blanco && valor_pedido.is_some();
    let y0 = encabezado(c, b, f.nombre);
    let celda_w = match f.total() {
        16 => 22.0,
        32 => 16.0,
        _ => 10.0,
    };
    let (w, h) = fila_bits(
        c,
        0.0,
        y0 + 4.0,
        &[
            CampoBits {
                nombre: "S",
                bits: &s_b,
                relleno: "fillHighlight",
                blanco: blanco_bits,
            },
            CampoBits {
                nombre: "Exponente",
                bits: &e_b,
                relleno: "fillPointerBox",
                blanco: blanco_bits,
            },
            CampoBits {
                nombre: "Mantisa (fracción)",
                bits: &m_b,
                relleno: "fillCell",
                blanco: blanco_bits,
            },
        ],
        celda_w,
    );
    let mut y = y0 + 4.0 + h + 6.0;
    let mut lineas: Vec<(String, String)> = Vec::new();
    let sesgo = f.sesgo();
    let max_e = (1u64 << f.exp) - 1;
    let fmt_v = |x: f64| -> String {
        if x.is_nan() {
            "NaN".into()
        } else if x.is_infinite() {
            if x > 0.0 {
                "+∞".into()
            } else {
                "−∞".into()
            }
        } else if x != 0.0 && (x.abs() < 1e-4 || x.abs() >= 1e9) {
            format!("{x:e}").replace('.', ",").replace('-', "−")
        } else {
            let t = format!("{x:.12}");
            let t = t.trim_end_matches('0').trim_end_matches('.');
            t.replace('.', ",").replace('-', "−")
        }
    };
    if let Some(v) = valor_pedido {
        lineas.push(("Valor".into(), fmt_v(v)));
        if v.is_finite() && v != 0.0 {
            let (ent, frac, sigue) = binario(v, (f.man + 8) as usize);
            let todo = format!("{ent}{frac}");
            let lead = todo.find('1').unwrap_or(0);
            let exp_real = ent.len() as i64 - 1 - lead as i64;
            let digitos: String = todo[lead + 1..].chars().take(f.man as usize).collect();
            let normal = format!(
                "1,{}{} × 2{}",
                if digitos.trim_end_matches('0').is_empty() {
                    "0"
                } else {
                    digitos.trim_end_matches('0')
                },
                if sigue || todo.len() - lead - 1 > f.man as usize {
                    "…"
                } else {
                    ""
                },
                sub2(exp_real)
            );
            let binstr = format!(
                "{ent},{}{}₂",
                if frac.is_empty() { "0" } else { &frac },
                if sigue { "…" } else { "" }
            );
            lineas.push(("Binario".into(), format!("{binstr} = {normal}")));
        }
    }
    lineas.push((
        "Signo".into(),
        format!("{s} → {}", if s == 1 { "negativo" } else { "positivo" }),
    ));
    if e == max_e {
        lineas.push((
            "Exponente".into(),
            format!(
                "{e_b}₂ (todos unos) → {}",
                if m == 0 { "infinito" } else { "NaN" }
            ),
        ));
    } else if e == 0 {
        lineas.push((
            "Exponente".into(),
            format!(
                "{e_b}₂ = 0 → {} (exponente 1 − {sesgo} = {})",
                if m == 0 { "cero" } else { "subnormal" },
                1 - sesgo
            ),
        ));
    } else {
        lineas.push((
            "Exponente".into(),
            format!(
                "{e_b}₂ = {e} → {e} − {sesgo} = {}",
                (e as i64 - sesgo).to_string().replace('-', "−")
            ),
        ));
    }
    if e != max_e {
        let implicito = if e == 0 { "0" } else { "1" };
        let man_corta = m_b.trim_end_matches('0');
        lineas.push((
            "Mantisa".into(),
            format!(
                "{implicito},{}₂",
                if man_corta.is_empty() { "0" } else { man_corta }
            ),
        ));
    }
    lineas.push((
        "Hexadecimal".into(),
        format!("0x{:0w$X}", bits, w = (f.total() / 4) as usize),
    ));
    lineas.push(("Valor almacenado".into(), fmt_v(almacenado)));
    if let Some(v) = valor_pedido {
        if v.is_finite() && almacenado.is_finite() && v != almacenado {
            lineas.push((
                "Error de redondeo".into(),
                format!("{:.3e}", almacenado - v)
                    .replace('.', ",")
                    .replace('-', "−"),
            ));
        }
    }
    let ancho_et = lineas
        .iter()
        .map(|(k, _)| ancho_texto(k, 10.5))
        .fold(0.0_f64, f64::max)
        + 12.0;
    let mut ancho = w;
    for (k, (et, val)) in lineas.iter().enumerate() {
        y += 17.0;
        c.t(0.0, y, et, 10.5, 700, "textSub", "start", SANS);
        // En modo ejercicio sólo se muestra el valor pedido (o los bits si se partió de ellos).
        let mostrar = !blanco
            || (k == 0 && valor_pedido.is_some())
            || (et == "Hexadecimal" && valor_pedido.is_none());
        if mostrar {
            c.t(ancho_et, y, val, 10.5, 500, "textMain", "start", MONO);
            ancho = ancho.max(ancho_et + ancho_texto(val, 10.5));
        } else {
            c.vacio(ancho_et, ancho_et + 220.0, y + 2.0);
            ancho = ancho.max(ancho_et + 220.0);
        }
    }
    Ok((ancho, y + 8.0))
}

// ---------------------------------------------------------------------------
// cache
// ---------------------------------------------------------------------------

fn tamano(v: Option<&Value>, clave: &str) -> Result<u64, ErrorDiagrama> {
    let v = v.ok_or_else(|| err(format!("falta '{clave}'")))?;
    let n = match v {
        Value::String(s) => {
            let t = s.trim().to_uppercase().replace(' ', "");
            let (num_t, mult) = if let Some(x) = t
                .strip_suffix("KIB")
                .or_else(|| t.strip_suffix("KB"))
                .or_else(|| t.strip_suffix('K'))
            {
                (x, 1024)
            } else if let Some(x) = t
                .strip_suffix("MIB")
                .or_else(|| t.strip_suffix("MB"))
                .or_else(|| t.strip_suffix('M'))
            {
                (x, 1024 * 1024)
            } else if let Some(x) = t.strip_suffix('B') {
                (x, 1)
            } else {
                (t.as_str(), 1)
            };
            num_t.parse::<u64>().map_err(|_| {
                err(format!(
                    "'{clave}': {s:?} no es un tamaño (p. ej. 256, \"1KB\")"
                ))
            })? * mult
        }
        _ => num(v)
            .filter(|x| x.fract() == 0.0 && *x > 0.0)
            .ok_or_else(|| err(format!("'{clave}' tiene que ser un entero positivo")))?
            as u64,
    };
    if n == 0 || !n.is_power_of_two() {
        return Err(err(format!(
            "'{clave}' tiene que ser potencia de 2 (es {n})"
        )));
    }
    Ok(n)
}

fn direccion(v: &Value) -> Result<u64, ErrorDiagrama> {
    match v {
        Value::String(s) => {
            let t = s.trim().replace('_', "");
            if let Some(h) = t.strip_prefix("0x").or_else(|| t.strip_prefix("0X")) {
                u64::from_str_radix(h, 16).map_err(|_| err(format!("dirección inválida {s:?}")))
            } else if let Some(bn) = t.strip_prefix("0b") {
                u64::from_str_radix(bn, 2).map_err(|_| err(format!("dirección inválida {s:?}")))
            } else {
                t.parse::<u64>()
                    .map_err(|_| err(format!("dirección inválida {s:?}")))
            }
        }
        _ => num(v)
            .filter(|x| x.fract() == 0.0 && *x >= 0.0)
            .map(|x| x as u64)
            .ok_or_else(|| err("las direcciones son enteros ≥ 0")),
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Resultado {
    Acierto,
    Fallo(&'static str),
}

#[derive(Debug, Clone)]
pub struct Via {
    pub tag: u64,
    pub uso: usize,
    pub carga: usize,
}

pub struct SimCache {
    pub conjuntos: Vec<Vec<Via>>,
    pub vias: usize,
}

impl SimCache {
    pub fn new(conjuntos: usize, vias: usize) -> Self {
        Self {
            conjuntos: vec![Vec::new(); conjuntos],
            vias,
        }
    }
    /// Devuelve (acierto, tag desalojado).
    pub fn acceder(
        &mut self,
        conjunto: usize,
        tag: u64,
        t: usize,
        lru: bool,
    ) -> (bool, Option<u64>) {
        let c = &mut self.conjuntos[conjunto];
        if let Some(v) = c.iter_mut().find(|v| v.tag == tag) {
            v.uso = t;
            return (true, None);
        }
        let nueva = Via {
            tag,
            uso: t,
            carga: t,
        };
        if c.len() < self.vias {
            c.push(nueva);
            return (false, None);
        }
        let victima = (0..c.len())
            .min_by_key(|&k| if lru { c[k].uso } else { c[k].carga })
            .unwrap();
        let viejo = c[victima].tag;
        c[victima] = nueva;
        (false, Some(viejo))
    }
}

pub fn render_cache(c: &mut Lienzo, b: &Bloque) -> Result<Punto, ErrorDiagrama> {
    let bits_dir = entero_b(b, "addressBits", 16, 4, 64)? as u32;
    let total = tamano(b.get("cacheSize"), "cacheSize")?;
    let bloque = tamano(b.get("blockSize"), "blockSize")?;
    let n_bloques = total / bloque;
    if n_bloques == 0 {
        return Err(err("'blockSize' no puede ser mayor que 'cacheSize'"));
    }
    let vias = match b.get("associativity") {
        None | Some(Value::Null) => 1,
        Some(Value::String(s))
            if matches!(
                s.to_lowercase().as_str(),
                "full" | "fully" | "completa" | "total"
            ) =>
        {
            n_bloques
        }
        Some(v) => num(v)
            .filter(|x| x.fract() == 0.0 && *x >= 1.0)
            .ok_or_else(|| err("'associativity' es 1, 2, 4, … o \"full\""))?
            as u64,
    };
    if !vias.is_power_of_two() || vias > n_bloques {
        return Err(err(format!(
            "'associativity' tiene que ser potencia de 2 y ≤ {n_bloques} bloques"
        )));
    }
    if n_bloques > 1024 {
        return Err(err("como máximo 1024 bloques en la caché"));
    }
    let conjuntos = n_bloques / vias;
    let b_off = bloque.trailing_zeros();
    let b_idx = conjuntos.trailing_zeros();
    if b_off + b_idx > bits_dir {
        return Err(err(format!(
            "índice + offset ({} bits) no entra en {bits_dir} bits de dirección",
            b_off + b_idx
        )));
    }
    let b_tag = bits_dir - b_off - b_idx;
    let lru = opcion(b, "policy", &["lru", "fifo"], "lru")? == "lru";
    let accesos: Vec<u64> = lista(b, "accesses", false)?
        .iter()
        .map(direccion)
        .collect::<Result<_, _>>()?;
    if accesos.len() > 64 {
        return Err(err("como máximo 64 accesos"));
    }
    let max_dir = if bits_dir >= 64 {
        u64::MAX
    } else {
        (1u64 << bits_dir) - 1
    };
    if let Some(a) = accesos.iter().find(|&&a| a > max_dir) {
        return Err(err(format!(
            "la dirección 0x{a:X} no entra en {bits_dir} bits"
        )));
    }
    let blanco = bool_opt(b, "blank", false);
    let clasificar = bool_opt(b, "missTypes", true);

    let hexw = bits_dir.div_ceil(4) as usize;
    let tipo_org = if vias == 1 {
        "correspondencia directa".to_string()
    } else if vias == n_bloques {
        "totalmente asociativa".to_string()
    } else {
        format!("asociativa de {vias} vías")
    };
    let extra = format!(
        "{} B, bloques de {bloque} B, {tipo_org}, {}",
        total,
        if lru { "LRU" } else { "FIFO" }
    );
    let y0 = encabezado(c, b, &extra);

    // Formato de la dirección.
    let campos = [
        ("Tag", b_tag, "fillPointerBox"),
        ("Índice", b_idx, "fillHighlight"),
        ("Offset", b_off, "fillCell"),
    ];
    let ancho_barra = 360.0_f64.max(bits_dir as f64 * 8.0);
    let mut x = 0.0;
    let yb = y0 + 6.0;
    for (nombre, n, relleno) in campos {
        if n == 0 {
            continue;
        }
        let w = ancho_barra * n as f64 / bits_dir as f64;
        c.r(x, yb, w, 26.0, relleno, 1.6, 0.0);
        let txt = if ancho_texto(nombre, 10.5) + 8.0 < w {
            nombre.to_string()
        } else {
            nombre.chars().next().unwrap().to_string()
        };
        c.t(
            x + w / 2.0,
            yb + 17.0,
            &txt,
            10.5,
            700,
            "textMain",
            "middle",
            SANS,
        );
        c.t(
            x + w / 2.0,
            yb + 40.0,
            &format!("{n} bits"),
            9.5,
            500,
            "textMuted",
            "middle",
            MONO,
        );
        x += w;
    }
    c.t(
        ancho_barra + 10.0,
        yb + 17.0,
        &format!(
            "{bits_dir} bits · {conjuntos} conjunto{}",
            if conjuntos == 1 { "" } else { "s" }
        ),
        10.0,
        500,
        "textSub",
        "start",
        MONO,
    );
    let mut y = yb + 54.0;

    // Simulación.
    let mut sim = SimCache::new(conjuntos as usize, vias as usize);
    let mut total_asoc = SimCache::new(1, n_bloques as usize);
    let mut vistos: Vec<u64> = Vec::new();
    let mut filas = Vec::new();
    let mut aciertos = 0;
    for (t, &a) in accesos.iter().enumerate() {
        let off = a & (bloque - 1);
        let idx = (a >> b_off) & (conjuntos - 1);
        let tag = if b_off + b_idx >= 64 {
            0
        } else {
            a >> (b_off + b_idx)
        };
        let n_bloque = a >> b_off;
        let (hit, desalojo) = sim.acceder(idx as usize, tag, t, lru);
        let (hit_full, _) = total_asoc.acceder(0, n_bloque, t, true);
        let res = if hit {
            aciertos += 1;
            Resultado::Acierto
        } else if !vistos.contains(&n_bloque) {
            Resultado::Fallo("forzoso")
        } else if !hit_full {
            Resultado::Fallo("capacidad")
        } else {
            Resultado::Fallo("conflicto")
        };
        if !vistos.contains(&n_bloque) {
            vistos.push(n_bloque);
        }
        let mut fila = vec![
            celda((t + 1).to_string()).atenuada(true),
            celda(format!("0x{a:0hexw$X}")),
            celda(format!("0x{tag:X}")).en_blanco(blanco),
            celda(idx.to_string()).en_blanco(blanco),
            celda(off.to_string()).en_blanco(blanco),
            celda(if hit { "acierto" } else { "fallo" })
                .en_blanco(blanco)
                .fuerte(true)
                .resaltada(!blanco && !hit),
        ];
        if clasificar {
            fila.push(
                celda(match &res {
                    Resultado::Fallo(k) => k.to_string(),
                    _ => String::new(),
                })
                .en_blanco(blanco && !hit),
            );
        }
        fila.push(
            celda(desalojo.map(|d| format!("0x{d:X}")).unwrap_or_default())
                .en_blanco(blanco && !hit),
        );
        filas.push(fila);
    }
    let mut enc: Vec<String> = ["#", "Dirección", "Tag", "Índice", "Offset", "Resultado"]
        .iter()
        .map(|s| s.to_string())
        .collect();
    if clasificar {
        enc.push("Tipo de fallo".into());
    }
    enc.push("Desaloja".into());
    let (w1, h1) = tabla(c, 0.0, y, &enc, &filas, &OpcTabla::default());
    y += h1 + 18.0;
    let resumen = if blanco {
        format!(
            "Aciertos: ____ / {} · Tasa de aciertos: ____",
            accesos.len()
        )
    } else {
        format!(
            "Aciertos: {aciertos} / {} · Tasa de aciertos: {} %",
            accesos.len(),
            fmt(100.0 * aciertos as f64 / accesos.len().max(1) as f64, 1)
        )
    };
    c.t(0.0, y, &resumen, 10.5, 700, "textMain", "start", MONO);
    let mut ancho = w1.max(ancho_barra + 160.0);
    // Estado final.
    if bool_opt(b, "finalState", true) && conjuntos * vias <= 64 {
        y += 14.0;
        c.t(
            0.0,
            y + 12.0,
            "Contenido final",
            11.0,
            700,
            "textSub",
            "start",
            SANS,
        );
        y += 18.0;
        let mut filas = Vec::new();
        for (k, conj) in sim.conjuntos.iter().enumerate() {
            for v in 0..vias as usize {
                let via = conj.get(v);
                let rango = via.map(|vv| {
                    let base = ((vv.tag << b_idx) | k as u64) << b_off;
                    format!("0x{base:0hexw$X}–0x{:0hexw$X}", base + bloque - 1)
                });
                filas.push(vec![
                    celda(k.to_string()),
                    celda(v.to_string()).atenuada(true),
                    celda(if via.is_some() { "1" } else { "0" }).en_blanco(blanco),
                    celda(
                        via.map(|vv| format!("0x{:X}", vv.tag))
                            .unwrap_or_else(|| "—".into()),
                    )
                    .en_blanco(blanco),
                    celda(rango.unwrap_or_else(|| "—".into())).en_blanco(blanco),
                ]);
            }
        }
        let enc: Vec<String> = ["Conjunto", "Vía", "V", "Tag", "Bloque en memoria"]
            .iter()
            .map(|s| s.to_string())
            .collect();
        let (w2, h2) = tabla(
            c,
            0.0,
            y,
            &enc,
            &filas,
            &OpcTabla {
                fila_h: 21.0,
                ..Default::default()
            },
        );
        ancho = ancho.max(w2);
        y += h2;
    }
    Ok((ancho, y + 6.0))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pipeline_load_use_y_sin_forwarding() {
        let ins: Vec<Instr> = ["lw $t0, 0($t1)", "add $t2, $t0, $t3", "sub $t4, $t2, $t5"]
            .iter()
            .map(|s| analizar_instr(s))
            .collect();
        assert_eq!(ins[0].destino.as_deref(), Some("$t0"));
        assert_eq!(ins[0].fuentes, vec!["$t1"]);
        assert_eq!(ins[1].fuentes, vec!["$t0", "$t3"]);
        // Con forwarding: 1 burbuja (load-use); EX: 3, 5, 6.
        let t = planificar(&ins, true);
        assert_eq!(t.iter().map(|x| x.e).collect::<Vec<_>>(), vec![3, 5, 6]);
        // Sin forwarding: el consumidor espera al WB (EX ≥ EX_prod + 3).
        let t = planificar(&ins, false);
        assert_eq!(t.iter().map(|x| x.e).collect::<Vec<_>>(), vec![3, 6, 9]);
        // El tercero hace IF en el ciclo 3 y queda retenido hasta que el segundo deja ID.
        assert_eq!((t[2].f, t[2].d), (3, 6));
    }

    #[test]
    fn ieee754_codifica_casos_conocidos() {
        assert_eq!(codificar(-6.75, SINGLE), 0xC0D8_0000);
        assert_eq!(codificar(0.1, SINGLE), 0x3DCC_CCCD);
        assert_eq!(codificar(1.0, HALF), 0x3C00);
        assert_eq!(codificar(-2.0, HALF), 0xC000);
        assert_eq!(codificar(65504.0, HALF), 0x7BFF);
        assert_eq!(codificar(1e6, HALF), 0x7C00);
        assert_eq!(decodificar(0xC0D8_0000, SINGLE), -6.75);
        assert_eq!(decodificar(0x0001, HALF), 2f64.powi(-24));
    }

    #[test]
    fn cache_lru_y_clasificacion() {
        let mut s = SimCache::new(1, 2);
        assert_eq!(s.acceder(0, 1, 0, true), (false, None));
        assert_eq!(s.acceder(0, 2, 1, true), (false, None));
        assert_eq!(s.acceder(0, 1, 2, true), (true, None));
        assert_eq!(s.acceder(0, 3, 3, true), (false, Some(2)));
    }
}
