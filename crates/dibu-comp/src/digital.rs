//! Sistemas digitales: `truth_table`, `karnaugh` y `logic_circuit`.

use crate::comun::*;
use crate::logica::{self, dibujar_escrito, escribir, Expr, Implicante, Notacion};
use serde_json::Value;

/// Función lógica de un bloque: desde `expr` o desde `minterms` (+ `dontcares`) con `vars`.
pub struct Funcion {
    pub vars: Vec<String>,
    pub nombre: String,
    pub expr: Option<Expr>,
    pub unos: Vec<u32>,
    pub indif: Vec<u32>,
}

fn indices(b: &Bloque, clave: &str, n: usize) -> Result<Vec<u32>, ErrorDiagrama> {
    let mut out = Vec::new();
    for v in lista(b, clave, true)? {
        let x = num(v)
            .filter(|x| x.fract() == 0.0 && *x >= 0.0)
            .ok_or_else(|| err(format!("'{clave}' tiene que contener enteros ≥ 0")))?;
        if x as u64 >= 1u64 << n {
            return Err(err(format!(
                "'{clave}': {x} no entra en {n} variables (máximo {})",
                (1u64 << n) - 1
            )));
        }
        if !out.contains(&(x as u32)) {
            out.push(x as u32);
        }
    }
    out.sort();
    Ok(out)
}

/// Lee `vars` (lista o texto "A B C") si está.
fn vars_declaradas(b: &Bloque) -> Result<Option<Vec<String>>, ErrorDiagrama> {
    match b.get("vars") {
        None | Some(Value::Null) => Ok(None),
        Some(Value::String(s)) => Ok(Some(
            s.split(|ch: char| ch == ',' || ch.is_whitespace())
                .filter(|x| !x.is_empty())
                .map(String::from)
                .collect(),
        )),
        Some(Value::Array(a)) => Ok(Some(a.iter().map(valor_str).collect())),
        _ => Err(err("'vars' tiene que ser una lista de nombres")),
    }
}

pub fn leer_funcion(
    b: &Bloque,
    expr_txt: Option<&str>,
    nombre: &str,
    max_vars: usize,
) -> Result<Funcion, ErrorDiagrama> {
    let declaradas = vars_declaradas(b)?;
    if let Some(txt) = expr_txt {
        let mut vars = declaradas.clone().unwrap_or_default();
        let fijas = declaradas.is_some();
        let e = logica::parsear_con(
            txt,
            &mut vars,
            fijas,
            !bool_opt(b, "multiLetterVars", false),
        )?;
        let e = if fijas {
            e
        } else {
            // Orden alfabético por defecto.
            let mut orden = vars.clone();
            orden.sort();
            let e2 = logica::renumerar(&e, &vars, &orden);
            vars = orden;
            e2
        };
        if vars.is_empty() {
            return Err(err("la expresión no tiene variables"));
        }
        if vars.len() > max_vars {
            return Err(err(format!(
                "como máximo {max_vars} variables (hay {})",
                vars.len()
            )));
        }
        let unos = logica::minterms(&e, vars.len());
        let indif = indices(b, "dontcares", vars.len())?;
        return Ok(Funcion {
            vars,
            nombre: nombre.to_string(),
            expr: Some(e),
            unos,
            indif,
        });
    }
    let vars = declaradas.ok_or_else(|| err("falta 'expr' o bien 'minterms' junto con 'vars'"))?;
    if vars.is_empty() || vars.len() > max_vars {
        return Err(err(format!(
            "'vars' tiene que tener entre 1 y {max_vars} nombres"
        )));
    }
    if !b.contains_key("minterms") && !b.contains_key("maxterms") {
        return Err(err("falta 'expr', 'minterms' o 'maxterms'"));
    }
    let n = vars.len();
    let indif = indices(b, "dontcares", n)?;
    let unos = if b.contains_key("maxterms") {
        let ceros = indices(b, "maxterms", n)?;
        (0..(1u32 << n))
            .filter(|m| !ceros.contains(m) && !indif.contains(m))
            .collect()
    } else {
        indices(b, "minterms", n)?
    };
    Ok(Funcion {
        vars,
        nombre: nombre.to_string(),
        expr: None,
        unos,
        indif,
    })
}

fn notacion(b: &Bloque) -> Result<Notacion, ErrorDiagrama> {
    Notacion::desde(&str_b(b, "notation", "overline").to_lowercase())
}

fn bits(m: u32, n: usize) -> String {
    (0..n)
        .map(|i| {
            if (m >> (n - 1 - i)) & 1 == 1 {
                '1'
            } else {
                '0'
            }
        })
        .collect()
}

fn lista_indices(v: &[u32]) -> String {
    v.iter()
        .map(|m| m.to_string())
        .collect::<Vec<_>>()
        .join(", ")
}

// ---------------------------------------------------------------------------
// truth_table
// ---------------------------------------------------------------------------

pub fn render_truth_table(c: &mut Lienzo, b: &Bloque) -> Result<Punto, ErrorDiagrama> {
    let nota = notacion(b)?;
    // Salidas: `outputs: [{name, expr}]`, `expr` (una), o `minterms`.
    let mut funciones: Vec<Funcion> = Vec::new();
    if let Some(Value::Array(outs)) = b.get("outputs") {
        if outs.len() > 8 {
            return Err(err("'outputs' admite como máximo 8 funciones"));
        }
        // Todas comparten variables: se juntan primero.
        let mut vars = vars_declaradas(b)?.unwrap_or_default();
        let fijas = b.contains_key("vars");
        let mut exprs = Vec::new();
        for (k, o) in outs.iter().enumerate() {
            let (nombre, txt) = match o {
                Value::String(s) => (format!("F{}", k + 1), s.clone()),
                Value::Object(m) => (
                    m.get("name")
                        .map(valor_str)
                        .unwrap_or_else(|| format!("F{}", k + 1)),
                    m.get("expr")
                        .map(valor_str)
                        .ok_or_else(|| err(format!("outputs[{k}] necesita 'expr'")))?,
                ),
                _ => return Err(err("cada salida es un texto o {name, expr}")),
            };
            exprs.push((
                nombre,
                logica::parsear_con(
                    &txt,
                    &mut vars,
                    fijas,
                    !bool_opt(b, "multiLetterVars", false),
                )?,
            ));
        }
        let mut orden = vars.clone();
        if !fijas {
            orden.sort();
        }
        if orden.len() > 6 {
            return Err(err("como máximo 6 variables en una tabla de verdad"));
        }
        for (nombre, e) in exprs {
            let e = logica::renumerar(&e, &vars, &orden);
            let unos = logica::minterms(&e, orden.len());
            funciones.push(Funcion {
                vars: orden.clone(),
                nombre,
                expr: Some(e),
                unos,
                indif: Vec::new(),
            });
        }
    } else {
        let txt = b.get("expr").map(valor_str);
        funciones.push(leer_funcion(b, txt.as_deref(), &str_b(b, "name", "F"), 6)?);
    }
    if funciones.is_empty() {
        return Err(err("'outputs' está vacío"));
    }
    let vars = funciones[0].vars.clone();
    let n = vars.len();
    let pasos = bool_opt(b, "steps", false);
    let blanco = bool_opt(b, "blank", false);
    let con_indice = bool_opt(b, "index", true);

    // Columnas: [m] vars [intermedias] salidas.
    let mut intermedias: Vec<Expr> = Vec::new();
    if pasos {
        for f in &funciones {
            if let Some(e) = &f.expr {
                let mut subs = Vec::new();
                logica::subexpresiones(e, &mut subs);
                subs.pop(); // la propia salida
                for s in subs {
                    if !intermedias.contains(&s) && intermedias.len() < 10 {
                        intermedias.push(s);
                    }
                }
            }
        }
    }
    let y0 = encabezado(c, b, "");
    let escritos_int: Vec<_> = intermedias
        .iter()
        .map(|e| escribir(e, &vars, nota))
        .collect();
    let mut cab: Vec<(String, Option<logica::Escrito>)> = Vec::new();
    if con_indice {
        cab.push(("m".into(), None));
    }
    for v in &vars {
        cab.push((v.clone(), None));
    }
    for e in &escritos_int {
        cab.push((e.texto.clone(), Some(e.clone())));
    }
    for f in &funciones {
        cab.push((f.nombre.clone(), None));
    }
    let primera_salida = cab.len() - funciones.len();
    let filas: Vec<Vec<Celda>> = (0..(1u32 << n))
        .map(|m| {
            let mut fila = Vec::new();
            if con_indice {
                fila.push(celda(m.to_string()).atenuada(true));
            }
            for ch in bits(m, n).chars() {
                fila.push(celda(ch.to_string()));
            }
            for e in &intermedias {
                fila.push(
                    celda(if logica::evaluar(e, m, n) { "1" } else { "0" }).en_blanco(blanco),
                );
            }
            for f in &funciones {
                let t = if f.indif.contains(&m) {
                    "X"
                } else if f.unos.contains(&m) {
                    "1"
                } else {
                    "0"
                };
                fila.push(
                    celda(t)
                        .en_blanco(blanco)
                        .fuerte(true)
                        .resaltada(!blanco && t == "1"),
                );
            }
            fila
        })
        .collect();
    let enc: Vec<String> = cab.iter().map(|(t, _)| t.clone()).collect();
    let o = OpcTabla {
        fila_h: if n >= 5 { 19.0 } else { 22.0 },
        separadores: &[if con_indice { n } else { n - 1 }, primera_salida - 1],
        ..Default::default()
    };
    let mut anchos = anchos_tabla(&enc, &filas, &o);
    // Las cabeceras con barra superior necesitan aire arriba: se dibujan aparte.
    let enc_vacio: Vec<String> = cab
        .iter()
        .map(|(t, e)| {
            if e.is_some() {
                String::new()
            } else {
                t.clone()
            }
        })
        .collect();
    for (j, (_, e)) in cab.iter().enumerate() {
        if let Some(e) = e {
            anchos[j] =
                anchos[j].max(e.texto.chars().count() as f64 * logica::ancho_car(10.5) + 16.0);
        }
    }
    let (w, h) = tabla_con_anchos(c, 0.0, y0, &enc_vacio, &filas, &anchos, &o);
    let mut x = 0.0;
    for (j, (_, e)) in cab.iter().enumerate() {
        if let Some(e) = e {
            dibujar_escrito(
                c,
                x + anchos[j] / 2.0,
                y0 + o.fila_h / 2.0 + 6.0,
                e,
                10.5,
                700,
                "textMain",
                "middle",
            );
        }
        x += anchos[j];
    }
    let mut alto = y0 + h;
    // Formas canónicas.
    if bool_opt(b, "canonical", false) {
        alto += 8.0;
        for f in &funciones {
            let ceros: Vec<u32> = (0..(1u32 << n))
                .filter(|m| !f.unos.contains(m) && !f.indif.contains(m))
                .collect();
            let vs = vars.join(",");
            let mut l1 = format!("{}({vs}) = Σm({})", f.nombre, lista_indices(&f.unos));
            if !f.indif.is_empty() {
                l1.push_str(&format!(" + Σd({})", lista_indices(&f.indif)));
            }
            let l2 = format!("{}({vs}) = ΠM({})", f.nombre, lista_indices(&ceros));
            for l in [l1, l2] {
                alto += 16.0;
                if blanco {
                    let pre = l.split('=').next().unwrap_or("").to_string() + "= ";
                    c.t(0.0, alto, &pre, 10.5, 600, "textMain", "start", MONO);
                    let x1 = ancho_texto(&pre, 10.5);
                    c.vacio(x1, (x1 + 220.0).max(w * 0.7), alto + 2.0);
                } else {
                    c.t(0.0, alto, &l, 10.5, 600, "textMain", "start", MONO);
                    c.ancho_min = c.ancho_min.max(ancho_texto(&l, 10.5));
                }
            }
        }
        alto += 4.0;
    }
    Ok((w, alto))
}

// ---------------------------------------------------------------------------
// karnaugh
// ---------------------------------------------------------------------------

const GRAY2: [u32; 2] = [0, 1];
const GRAY4: [u32; 4] = [0, 1, 3, 2];

fn gray(bits: usize) -> &'static [u32] {
    if bits <= 1 {
        &GRAY2
    } else {
        &GRAY4
    }
}

/// Posiciones (en el orden Gray) cubiertas por un implicante en un eje de `nb` bits.
fn posiciones_eje(imp: &Implicante, desplazamiento: usize, nb: usize) -> Vec<usize> {
    let g = gray(nb);
    let mask_eje = ((1u32 << nb) - 1) << desplazamiento;
    let libre = (imp.mascara & mask_eje) >> desplazamiento;
    let valor = (imp.valor & mask_eje) >> desplazamiento;
    (0..g.len())
        .filter(|&p| (g[p] & !libre) == (valor & !libre))
        .collect()
}

/// Agrupa posiciones contiguas en un toro de largo `len`: [(ini, largo)].
fn intervalos(pos: &[usize], len: usize) -> Vec<(usize, usize)> {
    if pos.len() == len {
        return vec![(0, len)];
    }
    let mut out: Vec<(usize, usize)> = Vec::new();
    for &p in pos {
        if let Some(u) = out.last_mut() {
            if u.0 + u.1 == p {
                u.1 += 1;
                continue;
            }
        }
        out.push((p, 1));
    }
    // ¿El primero continúa al último a través del borde? Se dibujan como dos piezas abiertas.
    out
}

pub fn render_karnaugh(c: &mut Lienzo, b: &Bloque) -> Result<Punto, ErrorDiagrama> {
    let nota = notacion(b)?;
    let txt = b.get("expr").map(valor_str);
    let f = leer_funcion(b, txt.as_deref(), &str_b(b, "name", "F"), 4)?;
    let n = f.vars.len();
    if n < 2 {
        return Err(err("un mapa de Karnaugh necesita entre 2 y 4 variables"));
    }
    let forma = opcion(b, "form", &["sop", "pos"], "sop")?;
    let pos = forma == "pos";
    let blanco = bool_opt(b, "blank", false);
    let mostrar_grupos =
        bool_opt(b, "groups", true) && !blanco && !bool_opt(b, "blankGroups", false);
    let mostrar_resultado = bool_opt(b, "result", true);

    // Filas: primeras variables; columnas: el resto.
    let nf = n / 2;
    let nc = n - nf;
    let filas_g = gray(nf);
    let cols_g = gray(nc);
    let celda_l = 40.0;
    let y0 = encabezado(c, b, "");
    let etiqueta_filas: String = f.vars[..nf].join("");
    let etiqueta_cols: String = f.vars[nf..].join("");
    let margen_izq = 30.0 + ancho_texto(&etiqueta_filas, 11.0).max(18.0);
    let gx = margen_izq;
    let gy = y0 + 44.0;

    // Diagonal con los nombres de las variables.
    c.linea(gx - 26.0, gy - 26.0, gx, gy, "strokeMain", 1.0, None, "");
    c.t(
        gx - 2.0,
        gy - 24.0,
        &etiqueta_cols,
        11.0,
        700,
        "textSub",
        "end",
        MONO,
    );
    c.t(
        gx - 22.0,
        gy - 4.0,
        &etiqueta_filas,
        11.0,
        700,
        "textSub",
        "end",
        MONO,
    );
    for (j, &g) in cols_g.iter().enumerate() {
        c.t(
            gx + j as f64 * celda_l + celda_l / 2.0,
            gy - 7.0,
            &bits(g, nc),
            10.5,
            600,
            "textMuted",
            "middle",
            MONO,
        );
    }
    for (i, &g) in filas_g.iter().enumerate() {
        c.t(
            gx - 6.0,
            gy + i as f64 * celda_l + celda_l / 2.0 + 4.0,
            &bits(g, nf),
            10.5,
            600,
            "textMuted",
            "end",
            MONO,
        );
    }
    for (i, &gf) in filas_g.iter().enumerate() {
        for (j, &gc) in cols_g.iter().enumerate() {
            let m = (gf << nc) | gc;
            let x = gx + j as f64 * celda_l;
            let y = gy + i as f64 * celda_l;
            c.r(x, y, celda_l, celda_l, "fillCell", 1.2, 0.0);
            c.t(
                x + celda_l - 3.0,
                y + 10.0,
                &m.to_string(),
                8.0,
                400,
                "textMuted",
                "end",
                MONO,
            );
            let v = if f.indif.contains(&m) {
                "X"
            } else if f.unos.contains(&m) {
                "1"
            } else {
                "0"
            };
            if !bool_opt(b, "blankCells", false) {
                c.t(
                    x + celda_l / 2.0,
                    y + celda_l / 2.0 + 6.0,
                    v,
                    15.0,
                    600,
                    if v == "0" { "textMuted" } else { "textMain" },
                    "middle",
                    MONO,
                );
            }
        }
    }
    let ancho_mapa = cols_g.len() as f64 * celda_l;
    let alto_mapa = filas_g.len() as f64 * celda_l;
    c.add(format!(
        r#"<rect x="{gx:.1}" y="{gy:.1}" width="{ancho_mapa:.1}" height="{alto_mapa:.1}" fill="none" stroke="{}" stroke-width="2"/>"#,
        c.pal.color("strokeMain")
    ));

    let (cob, resultado) = logica::minimizar(&f.unos, &f.indif, n, pos);
    if mostrar_grupos {
        let estilos = ["", "6 3", "2 2", "8 3 2 3", "4 2"];
        let colores = [
            "accentPrimary",
            "strokeSub",
            "accentPrimary",
            "strokeSub",
            "strokeMain",
        ];
        for (k, imp) in cob.iter().enumerate() {
            let filas_p = posiciones_eje(imp, nc, nf);
            let cols_p = posiciones_eje(imp, 0, nc);
            let inset = 4.0 + (k % 4) as f64 * 2.5;
            for &(fi, fl) in &intervalos(&filas_p, filas_g.len()) {
                for &(ci, cl) in &intervalos(&cols_p, cols_g.len()) {
                    let x = gx + ci as f64 * celda_l + inset;
                    let y = gy + fi as f64 * celda_l + inset;
                    let w = cl as f64 * celda_l - 2.0 * inset;
                    let h = fl as f64 * celda_l - 2.0 * inset;
                    let dash = estilos[k % estilos.len()];
                    let dash = if dash.is_empty() {
                        String::new()
                    } else {
                        format!(r#" stroke-dasharray="{dash}""#)
                    };
                    c.add(format!(
                        r#"<rect x="{x:.1}" y="{y:.1}" width="{w:.1}" height="{h:.1}" rx="12" fill="none" stroke="{}" stroke-width="2"{dash}/>"#,
                        c.pal.color(colores[k % colores.len()])
                    ));
                }
            }
        }
    }
    let mut alto = gy + alto_mapa + 10.0;
    let mut ancho = gx + ancho_mapa;
    if mostrar_resultado {
        let vs = f.vars.join(",");
        let prefijo = format!("{}({vs}) = ", f.nombre);
        alto += 20.0;
        c.t(0.0, alto, &prefijo, 11.5, 700, "textMain", "start", MONO);
        let x1 = ancho_texto(&prefijo, 11.5);
        if blanco {
            c.vacio(x1, x1 + 200.0, alto + 2.0);
            ancho = ancho.max(x1 + 200.0);
        } else {
            let e = escribir(&resultado, &f.vars, nota);
            let w = dibujar_escrito(c, x1, alto, &e, 11.5, 700, "textMain", "start");
            ancho = ancho.max(x1 + w);
        }
        if bool_opt(b, "implicants", false) && !blanco {
            for (k, imp) in cob.iter().enumerate() {
                alto += 15.0;
                let cubre: Vec<u32> = (0..(1u32 << n)).filter(|&m| imp.cubre(m)).collect();
                let e = escribir(&imp.expr(n, pos), &f.vars, nota);
                let pre = format!("{}. m({}) → ", k + 1, lista_indices(&cubre));
                c.t(0.0, alto, &pre, 10.0, 400, "textSub", "start", MONO);
                let x1 = ancho_texto(&pre, 10.0);
                let w = dibujar_escrito(c, x1, alto, &e, 10.0, 600, "textSub", "start");
                ancho = ancho.max(x1 + w);
            }
        }
        alto += 6.0;
    }
    Ok((ancho, alto))
}

// ---------------------------------------------------------------------------
// logic_circuit
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq)]
enum Compuerta {
    And,
    Or,
    Xor,
    Nand,
    Nor,
    Xnor,
    Not,
}

impl Compuerta {
    fn negada(self) -> bool {
        matches!(self, Self::Nand | Self::Nor | Self::Xnor | Self::Not)
    }
}

enum Nodo {
    Entrada(usize),
    Constante(bool),
    Puerta(Compuerta, Vec<usize>),
}

/// Convierte la expresión en un grafo de compuertas (árbol; las variables son hojas).
fn construir(e: &Expr, nodos: &mut Vec<Nodo>, sin_negadas: bool) -> usize {
    let nodo = match e {
        Expr::Var(i) => Nodo::Entrada(*i),
        Expr::Const(b) => Nodo::Constante(*b),
        Expr::Not(x) => match (&**x, sin_negadas) {
            (Expr::And(v), false) => Nodo::Puerta(
                Compuerta::Nand,
                v.iter().map(|y| construir(y, nodos, sin_negadas)).collect(),
            ),
            (Expr::Or(v), false) => Nodo::Puerta(
                Compuerta::Nor,
                v.iter().map(|y| construir(y, nodos, sin_negadas)).collect(),
            ),
            (Expr::Xor(v), false) => Nodo::Puerta(
                Compuerta::Xnor,
                v.iter().map(|y| construir(y, nodos, sin_negadas)).collect(),
            ),
            _ => Nodo::Puerta(Compuerta::Not, vec![construir(x, nodos, sin_negadas)]),
        },
        Expr::And(v) => Nodo::Puerta(
            Compuerta::And,
            v.iter().map(|y| construir(y, nodos, sin_negadas)).collect(),
        ),
        Expr::Or(v) => Nodo::Puerta(
            Compuerta::Or,
            v.iter().map(|y| construir(y, nodos, sin_negadas)).collect(),
        ),
        Expr::Xor(v) => Nodo::Puerta(
            Compuerta::Xor,
            v.iter().map(|y| construir(y, nodos, sin_negadas)).collect(),
        ),
    };
    nodos.push(nodo);
    nodos.len() - 1
}

const PUERTA_W: f64 = 46.0;
const PASO_ENTRADA: f64 = 16.0;
const SEP_COL: f64 = 40.0;

fn alto_puerta(g: Compuerta, entradas: usize) -> f64 {
    if g == Compuerta::Not {
        22.0
    } else {
        (entradas as f64 * PASO_ENTRADA + 6.0).max(32.0)
    }
}

/// Dibuja el cuerpo de la compuerta con su borde izquierdo en `x` y centro vertical `y`.
/// Devuelve la x de la salida.
fn dibujar_puerta(c: &mut Lienzo, g: Compuerta, x: f64, y: f64, h: f64) -> f64 {
    let w = PUERTA_W;
    let (top, bot) = (y - h / 2.0, y + h / 2.0);
    let d = match g {
        Compuerta::And | Compuerta::Nand => {
            let r = h / 2.0;
            format!("M {x:.1} {top:.1} L {:.1} {top:.1} A {r:.1} {r:.1} 0 0 1 {:.1} {bot:.1} L {x:.1} {bot:.1} Z", x + w - r, x + w - r)
        }
        Compuerta::Or | Compuerta::Nor | Compuerta::Xor | Compuerta::Xnor => format!(
            "M {x:.1} {top:.1} Q {:.1} {top:.1} {:.1} {y:.1} Q {:.1} {bot:.1} {x:.1} {bot:.1} Q {:.1} {y:.1} {x:.1} {top:.1} Z",
            x + w * 0.62,
            x + w,
            x + w * 0.62,
            x + w * 0.28
        ),
        Compuerta::Not => format!("M {x:.1} {top:.1} L {:.1} {y:.1} L {x:.1} {bot:.1} Z", x + w - 12.0),
    };
    camino(c, &d, "fillCell", "strokeMain", 1.6);
    if matches!(g, Compuerta::Xor | Compuerta::Xnor) {
        camino(
            c,
            &format!(
                "M {:.1} {top:.1} Q {:.1} {y:.1} {:.1} {bot:.1}",
                x - 6.0,
                x + w * 0.28 - 6.0,
                x - 6.0
            ),
            "none",
            "strokeMain",
            1.6,
        );
    }
    let mut xs = if g == Compuerta::Not {
        x + w - 12.0
    } else {
        x + w
    };
    if g.negada() {
        circulo(c, xs + 4.0, y, 4.0, "fillCell", "strokeMain", 1.5);
        xs += 8.0;
    }
    xs
}

/// x donde termina un cable de entrada en el borde (curvo en OR/XOR).
fn borde_entrada(g: Compuerta, x: f64, dy: f64, h: f64) -> f64 {
    match g {
        Compuerta::Or | Compuerta::Nor => {
            let t = (dy / (h / 2.0)).clamp(-1.0, 1.0);
            x + w_curva(t)
        }
        Compuerta::Xor | Compuerta::Xnor => x - 6.0 + w_curva((dy / (h / 2.0)).clamp(-1.0, 1.0)),
        _ => x,
    }
}

fn w_curva(t: f64) -> f64 {
    // Curva cuadrática del lado de entrada: x(t) ≈ 0,28·w·2·s(1−s) con s = (1+t)/2.
    let s = (1.0 + t) / 2.0;
    PUERTA_W * 0.28 * 2.0 * s * (1.0 - s)
}

struct Ubicado {
    col: usize,
    y: f64,
    h: f64,
}

pub fn render_logic_circuit(c: &mut Lienzo, b: &Bloque) -> Result<Punto, ErrorDiagrama> {
    let txt = b
        .get("expr")
        .map(valor_str)
        .ok_or_else(|| err("falta 'expr'"))?;
    let f = leer_funcion(
        b,
        Some(&txt),
        &str_b(b, "output", &str_b(b, "name", "F")),
        logica::MAX_VARS,
    )?;
    let e = f.expr.clone().expect("hay expresión");
    let sin_negadas = bool_opt(b, "basicGates", false);
    let mut nodos: Vec<Nodo> = Vec::new();
    let raiz = construir(&e, &mut nodos, sin_negadas);
    if nodos.len() > 200 {
        return Err(err(
            "la expresión es demasiado grande para dibujarla (máximo 200 nodos)",
        ));
    }
    let nota = notacion(b)?;

    // Profundidad (columna desde la salida) y orden de hojas.
    let mut prof = vec![0usize; nodos.len()];
    fn fijar_prof(nodos: &[Nodo], i: usize, p: usize, prof: &mut [usize]) {
        prof[i] = prof[i].max(p);
        if let Nodo::Puerta(_, hijos) = &nodos[i] {
            for &h in hijos {
                fijar_prof(nodos, h, p + 1, prof);
            }
        }
    }
    fijar_prof(&nodos, raiz, 0, &mut prof);
    let max_prof = nodos
        .iter()
        .enumerate()
        .filter(|(_, n)| matches!(n, Nodo::Puerta(..)))
        .map(|(i, _)| prof[i])
        .max()
        .unwrap_or(0);

    // Ubicación vertical: recorrido en profundidad, cada hoja ocupa una franja.
    let mut ubic: Vec<Option<Ubicado>> = (0..nodos.len()).map(|_| None).collect();
    let mut cursor_y = 0.0_f64;
    fn ubicar(
        nodos: &[Nodo],
        i: usize,
        prof: &[usize],
        max_prof: usize,
        cursor: &mut f64,
        ubic: &mut [Option<Ubicado>],
    ) -> f64 {
        match &nodos[i] {
            Nodo::Entrada(_) | Nodo::Constante(_) => {
                let y = *cursor + PASO_ENTRADA / 2.0;
                *cursor += PASO_ENTRADA + 4.0;
                ubic[i] = Some(Ubicado { col: 0, y, h: 0.0 });
                y
            }
            Nodo::Puerta(g, hijos) => {
                let ini = *cursor;
                let ys: Vec<f64> = hijos
                    .iter()
                    .map(|&h| ubicar(nodos, h, prof, max_prof, cursor, ubic))
                    .collect();
                let h = alto_puerta(*g, hijos.len());
                let y = if ys.len() == 1 {
                    ys[0]
                } else {
                    (ys[0] + ys[ys.len() - 1]) / 2.0
                };
                // Si los hijos son más bajos que la compuerta, reservar su alto.
                if *cursor - ini < h + 6.0 {
                    *cursor = ini + h + 6.0;
                }
                let y = y.max(ini + h / 2.0);
                *cursor += 6.0;
                ubic[i] = Some(Ubicado {
                    col: max_prof + 1 - prof[i],
                    y,
                    h,
                });
                y
            }
        }
    }
    let y_out = ubicar(&nodos, raiz, &prof, max_prof, &mut cursor_y, &mut ubic);

    let y0 = encabezado(c, b, "");
    let top = y0 + 26.0;
    let n_vars = f.vars.len();
    let rieles = bool_opt(b, "rails", true);
    let sep_riel = 22.0;
    let x_gates = if rieles {
        14.0 + n_vars as f64 * sep_riel + 24.0
    } else {
        46.0
    };
    let col_x = |col: usize| x_gates + (col.max(1) - 1) as f64 * (PUERTA_W + SEP_COL + 10.0);
    let alto_total = cursor_y.max(30.0);

    // Rieles verticales de entrada.
    let riel_x = |i: usize| 14.0 + i as f64 * sep_riel;
    if rieles {
        for (i, v) in f.vars.iter().enumerate() {
            let x = riel_x(i);
            c.t(x, top - 8.0, v, 11.5, 700, "textMain", "middle", MONO);
            c.linea(
                x,
                top - 2.0,
                x,
                top + alto_total,
                "strokeMain",
                1.4,
                None,
                "",
            );
        }
    }

    // Compuertas y cables.
    let mut salidas_x = vec![0.0f64; nodos.len()];
    // Primero las compuertas de las columnas más a la izquierda.
    let mut orden: Vec<usize> = (0..nodos.len())
        .filter(|&i| matches!(nodos[i], Nodo::Puerta(..)))
        .collect();
    orden.sort_by_key(|&i| ubic[i].as_ref().map(|u| u.col).unwrap_or(0));
    for &i in &orden {
        let Nodo::Puerta(g, hijos) = &nodos[i] else {
            continue;
        };
        let u = ubic[i].as_ref().unwrap();
        let x = col_x(u.col);
        let y = top + u.y;
        salidas_x[i] = dibujar_puerta(c, *g, x, y, u.h);
        let k = hijos.len();
        for (p, &h) in hijos.iter().enumerate() {
            let dy = if k == 1 {
                0.0
            } else {
                -((k - 1) as f64) * PASO_ENTRADA / 2.0 + p as f64 * PASO_ENTRADA
            };
            let yp = y + dy;
            let xe = borde_entrada(*g, x, dy, u.h);
            match &nodos[h] {
                Nodo::Entrada(v) => {
                    if rieles {
                        let xr = riel_x(*v);
                        c.linea(xr, yp, xe, yp, "strokeMain", 1.4, None, "");
                        c.punto(xr, yp, "strokeMain", 2.8);
                    } else {
                        c.t(
                            x - 22.0,
                            yp + 4.0,
                            &f.vars[*v],
                            11.0,
                            700,
                            "textMain",
                            "end",
                            MONO,
                        );
                        c.linea(x - 18.0, yp, xe, yp, "strokeMain", 1.4, None, "");
                    }
                }
                Nodo::Constante(bv) => {
                    c.t(
                        x - 20.0,
                        yp + 4.0,
                        if *bv { "1" } else { "0" },
                        11.0,
                        700,
                        "textMain",
                        "end",
                        MONO,
                    );
                    c.linea(x - 16.0, yp, xe, yp, "strokeMain", 1.4, None, "");
                }
                Nodo::Puerta(..) => {
                    let uh = ubic[h].as_ref().unwrap();
                    let xs = salidas_x[h];
                    let ys = top + uh.y;
                    let xm = x - 12.0 - (p as f64) * 3.0;
                    camino(c, &format!("M {xs:.1} {ys:.1} L {xm:.1} {ys:.1} L {xm:.1} {yp:.1} L {xe:.1} {yp:.1}"), "none", "strokeMain", 1.4);
                }
            }
        }
    }
    // Salida.
    let (xs, ys) = match &nodos[raiz] {
        Nodo::Puerta(..) => (salidas_x[raiz], top + y_out),
        Nodo::Entrada(v) if rieles => (riel_x(*v), top + y_out),
        _ => (x_gates, top + y_out),
    };
    let x_fin = xs + 36.0;
    c.linea(xs, ys, x_fin, ys, "strokeMain", 1.4, None, "");
    c.t(
        x_fin + 6.0,
        ys + 4.0,
        &f.nombre,
        12.0,
        700,
        "textMain",
        "start",
        MONO,
    );
    let mut ancho = x_fin + 10.0 + ancho_texto(&f.nombre, 12.0);
    let mut alto = top + alto_total + 8.0;
    if bool_opt(b, "showExpr", true) {
        let pre = format!("{} = ", f.nombre);
        alto += 16.0;
        c.t(0.0, alto, &pre, 11.0, 700, "textMain", "start", MONO);
        let x1 = ancho_texto(&pre, 11.0);
        let w = dibujar_escrito(
            c,
            x1,
            alto,
            &escribir(&e, &f.vars, nota),
            11.0,
            600,
            "textMain",
            "start",
        );
        ancho = ancho.max(x1 + w);
        alto += 6.0;
    }
    Ok((ancho, alto))
}
