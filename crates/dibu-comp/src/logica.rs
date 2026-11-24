//! Álgebra de Boole: parser de expresiones, evaluación, minimización (Quine–McCluskey con
//! cobertura exacta) y escritura con barra superior, prima o `¬`.
//!
//! Sintaxis aceptada (sin distinguir mayúsculas en las palabras clave):
//! - NOT: `!A`, `~A`, `¬A`, `A'`, `NOT A`
//! - AND: `A & B`, `A · B`, `A * B`, `A ∧ B`, `A AND B`, o yuxtaposición `A B`, `A(B+C)`
//! - OR: `A + B`, `A | B`, `A ∨ B`, `A OR B`
//! - XOR: `A ^ B`, `A ⊕ B`, `A XOR B`; también `NAND`, `NOR`, `XNOR`
//! - constantes `0` y `1`; variables `[A-Za-z_][A-Za-z0-9_]*`

use crate::comun::{err, ErrorDiagrama, Lienzo, MONO};

pub const MAX_VARS: usize = 8;

#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    Var(usize),
    Const(bool),
    Not(Box<Expr>),
    And(Vec<Expr>),
    Or(Vec<Expr>),
    Xor(Vec<Expr>),
}

#[derive(Debug, Clone, PartialEq)]
enum Tok {
    Id(String),
    Const(bool),
    Not,
    Prima,
    And,
    Nand,
    Or,
    Nor,
    Xor,
    Xnor,
    Abre,
    Cierra,
}

fn tokenizar(
    s: &str,
    vars: &[String],
    fijas: bool,
    separar: bool,
) -> Result<Vec<Tok>, ErrorDiagrama> {
    let cs: Vec<char> = s.chars().collect();
    let mut out = Vec::new();
    let mut i = 0;
    while i < cs.len() {
        let ch = cs[i];
        if ch.is_whitespace() {
            i += 1;
            continue;
        }
        if ch.is_ascii_alphabetic() || ch == '_' {
            let ini = i;
            while i < cs.len() && (cs[i].is_ascii_alphanumeric() || cs[i] == '_') {
                i += 1;
            }
            let id: String = cs[ini..i].iter().collect();
            // `ABC'` → A, B, C' (variables de una letra), salvo que `id` sea una variable declarada.
            let divisible = separar
                && id.len() > 1
                && id.chars().all(|ch| ch.is_ascii_uppercase())
                && !vars.contains(&id)
                && !matches!(
                    id.as_str(),
                    "AND" | "OR" | "NOT" | "XOR" | "NAND" | "NOR" | "XNOR"
                )
                && (!fijas || id.chars().all(|ch| vars.contains(&ch.to_string())));
            if divisible {
                out.extend(id.chars().map(|ch| Tok::Id(ch.to_string())));
                continue;
            }
            out.push(match id.to_ascii_uppercase().as_str() {
                "AND" => Tok::And,
                "OR" => Tok::Or,
                "NOT" => Tok::Not,
                "XOR" => Tok::Xor,
                "NAND" => Tok::Nand,
                "NOR" => Tok::Nor,
                "XNOR" => Tok::Xnor,
                _ => Tok::Id(id),
            });
            continue;
        }
        out.push(match ch {
            '0' => Tok::Const(false),
            '1' => Tok::Const(true),
            '!' | '~' | '¬' => Tok::Not,
            '\'' | '’' => Tok::Prima,
            '&' | '·' | '*' | '.' | '∧' | '⋅' => Tok::And,
            '+' | '|' | '∨' => Tok::Or,
            '^' | '⊕' => Tok::Xor,
            '(' | '[' => Tok::Abre,
            ')' | ']' => Tok::Cierra,
            _ => {
                return Err(err(format!(
                    "carácter inesperado {ch:?} en la expresión {s:?}"
                )))
            }
        });
        i += 1;
    }
    Ok(out)
}

pub struct Parser<'a> {
    toks: Vec<Tok>,
    pos: usize,
    pub vars: &'a mut Vec<String>,
    fijas: bool,
    texto: String,
}

impl<'a> Parser<'a> {
    fn ver(&self) -> Option<&Tok> {
        self.toks.get(self.pos)
    }
    fn tomar(&mut self) -> Option<Tok> {
        let t = self.toks.get(self.pos).cloned();
        self.pos += 1;
        t
    }
    fn o(&mut self) -> Result<Expr, ErrorDiagrama> {
        let mut e = self.xor()?;
        loop {
            match self.ver() {
                Some(Tok::Or) => {
                    self.pos += 1;
                    let d = self.xor()?;
                    e = unir_or(e, d);
                }
                Some(Tok::Nor) => {
                    self.pos += 1;
                    let d = self.xor()?;
                    e = Expr::Not(Box::new(Expr::Or(vec![e, d])));
                }
                _ => return Ok(e),
            }
        }
    }
    fn xor(&mut self) -> Result<Expr, ErrorDiagrama> {
        let mut e = self.y()?;
        loop {
            match self.ver() {
                Some(Tok::Xor) => {
                    self.pos += 1;
                    let d = self.y()?;
                    e = match e {
                        Expr::Xor(mut v) => {
                            v.push(d);
                            Expr::Xor(v)
                        }
                        otro => Expr::Xor(vec![otro, d]),
                    };
                }
                Some(Tok::Xnor) => {
                    self.pos += 1;
                    let d = self.y()?;
                    e = Expr::Not(Box::new(Expr::Xor(vec![e, d])));
                }
                _ => return Ok(e),
            }
        }
    }
    fn y(&mut self) -> Result<Expr, ErrorDiagrama> {
        let mut e = self.unario()?;
        loop {
            match self.ver() {
                Some(Tok::And) => {
                    self.pos += 1;
                    let d = self.unario()?;
                    e = unir_and(e, d);
                }
                Some(Tok::Nand) => {
                    self.pos += 1;
                    let d = self.unario()?;
                    e = Expr::Not(Box::new(Expr::And(vec![e, d])));
                }
                // Yuxtaposición: `A B`, `A(B+C)`, `A !B`.
                Some(Tok::Id(_)) | Some(Tok::Const(_)) | Some(Tok::Abre) | Some(Tok::Not) => {
                    let d = self.unario()?;
                    e = unir_and(e, d);
                }
                _ => return Ok(e),
            }
        }
    }
    fn unario(&mut self) -> Result<Expr, ErrorDiagrama> {
        let mut e = match self.tomar() {
            Some(Tok::Not) => Expr::Not(Box::new(self.unario()?)),
            Some(Tok::Abre) => {
                let e = self.o()?;
                if self.tomar() != Some(Tok::Cierra) {
                    return Err(err(format!("falta ')' en la expresión {:?}", self.texto)));
                }
                e
            }
            Some(Tok::Const(b)) => Expr::Const(b),
            Some(Tok::Id(n)) => {
                let i = match self.vars.iter().position(|v| *v == n) {
                    Some(i) => i,
                    None if self.fijas => {
                        return Err(err(format!("la variable {n:?} no está en 'vars' ({})", self.vars.join(", "))))
                    }
                    None => {
                        if self.vars.len() >= MAX_VARS {
                            return Err(err(format!("como máximo {MAX_VARS} variables")));
                        }
                        self.vars.push(n);
                        self.vars.len() - 1
                    }
                };
                Expr::Var(i)
            }
            otro => {
                return Err(err(format!(
                    "expresión incompleta o mal formada {:?} (se esperaba una variable, constante o '(' y llegó {:?})",
                    self.texto, otro
                )))
            }
        };
        while self.ver() == Some(&Tok::Prima) {
            self.pos += 1;
            e = Expr::Not(Box::new(e));
        }
        Ok(e)
    }
}

fn unir_and(a: Expr, b: Expr) -> Expr {
    match a {
        Expr::And(mut v) => {
            v.push(b);
            Expr::And(v)
        }
        otro => Expr::And(vec![otro, b]),
    }
}
fn unir_or(a: Expr, b: Expr) -> Expr {
    match a {
        Expr::Or(mut v) => {
            v.push(b);
            Expr::Or(v)
        }
        otro => Expr::Or(vec![otro, b]),
    }
}

/// Parsea `texto`. Si `fijas`, sólo admite las variables ya listadas en `vars`.
/// Los identificadores de varias mayúsculas (`AB'C`) se separan en variables de una letra.
pub fn parsear(texto: &str, vars: &mut Vec<String>, fijas: bool) -> Result<Expr, ErrorDiagrama> {
    parsear_con(texto, vars, fijas, true)
}

pub fn parsear_con(
    texto: &str,
    vars: &mut Vec<String>,
    fijas: bool,
    separar: bool,
) -> Result<Expr, ErrorDiagrama> {
    if texto.chars().count() > 2000 {
        return Err(err("expresión demasiado larga"));
    }
    let toks = tokenizar(texto, vars, fijas, separar)?;
    if toks.is_empty() {
        return Err(err("expresión vacía"));
    }
    let mut p = Parser {
        toks,
        pos: 0,
        vars,
        fijas,
        texto: texto.to_string(),
    };
    let e = p.o()?;
    if p.pos < p.toks.len() {
        return Err(err(format!("sobra texto en la expresión {texto:?}")));
    }
    Ok(e)
}

/// Renumera las variables de `e` según el orden alfabético (u otro orden) de `nuevas`.
pub fn renumerar(e: &Expr, viejas: &[String], nuevas: &[String]) -> Expr {
    match e {
        Expr::Var(i) => Expr::Var(nuevas.iter().position(|n| *n == viejas[*i]).unwrap_or(*i)),
        Expr::Const(b) => Expr::Const(*b),
        Expr::Not(x) => Expr::Not(Box::new(renumerar(x, viejas, nuevas))),
        Expr::And(v) => Expr::And(v.iter().map(|x| renumerar(x, viejas, nuevas)).collect()),
        Expr::Or(v) => Expr::Or(v.iter().map(|x| renumerar(x, viejas, nuevas)).collect()),
        Expr::Xor(v) => Expr::Xor(v.iter().map(|x| renumerar(x, viejas, nuevas)).collect()),
    }
}

/// Evalúa con la asignación `fila`: el bit de la variable i es `(fila >> (n-1-i)) & 1`.
pub fn evaluar(e: &Expr, fila: u32, n: usize) -> bool {
    match e {
        Expr::Var(i) => (fila >> (n - 1 - i)) & 1 == 1,
        Expr::Const(b) => *b,
        Expr::Not(x) => !evaluar(x, fila, n),
        Expr::And(v) => v.iter().all(|x| evaluar(x, fila, n)),
        Expr::Or(v) => v.iter().any(|x| evaluar(x, fila, n)),
        Expr::Xor(v) => v.iter().fold(false, |a, x| a ^ evaluar(x, fila, n)),
    }
}

pub fn minterms(e: &Expr, n: usize) -> Vec<u32> {
    (0..(1u32 << n)).filter(|&m| evaluar(e, m, n)).collect()
}

/// Subexpresiones compuestas en post-orden (sin repetir), útiles como columnas intermedias.
pub fn subexpresiones(e: &Expr, out: &mut Vec<Expr>) {
    match e {
        Expr::Var(_) | Expr::Const(_) => return,
        Expr::Not(x) => subexpresiones(x, out),
        Expr::And(v) | Expr::Or(v) | Expr::Xor(v) => v.iter().for_each(|x| subexpresiones(x, out)),
    }
    if !out.contains(e) {
        out.push(e.clone());
    }
}

// ---------------------------------------------------------------------------
// Escritura
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Notacion {
    Barra,
    Prima,
    Neg,
}

impl Notacion {
    pub fn desde(s: &str) -> Result<Self, ErrorDiagrama> {
        match s {
            "overline" | "bar" | "barra" => Ok(Self::Barra),
            "prime" | "prima" => Ok(Self::Prima),
            "neg" | "not" | "¬" => Ok(Self::Neg),
            _ => Err(err(format!(
                "'notation' no admite {s:?} (opciones: overline, prime, neg)"
            ))),
        }
    }
}

/// Texto plano + tramos con barra superior (inicio, fin en caracteres, nivel de anidamiento).
#[derive(Debug, Clone, Default)]
pub struct Escrito {
    pub texto: String,
    pub barras: Vec<(usize, usize, usize)>,
}

impl Escrito {
    fn push(&mut self, s: &str) {
        self.texto.push_str(s);
    }
    fn len(&self) -> usize {
        self.texto.chars().count()
    }
    pub fn niveles(&self) -> usize {
        self.barras.iter().map(|b| b.2 + 1).max().unwrap_or(0)
    }
}

fn prec(e: &Expr) -> u8 {
    match e {
        Expr::Or(_) => 1,
        Expr::Xor(_) => 2,
        Expr::And(_) => 3,
        _ => 4,
    }
}

pub fn escribir(e: &Expr, vars: &[String], n: Notacion) -> Escrito {
    let mut out = Escrito::default();
    escribir_en(e, vars, n, &mut out);
    out
}

fn escribir_en(e: &Expr, vars: &[String], n: Notacion, out: &mut Escrito) -> usize {
    // Devuelve la profundidad de barras usada dentro de `e`.
    match e {
        Expr::Var(i) => {
            out.push(&vars[*i]);
            0
        }
        Expr::Const(b) => {
            out.push(if *b { "1" } else { "0" });
            0
        }
        Expr::Not(x) => match n {
            Notacion::Barra => {
                let ini = out.len();
                let interior = escribir_en(x, vars, n, out);
                let fin = out.len();
                out.barras.push((ini, fin, interior));
                interior + 1
            }
            Notacion::Prima => {
                let compuesto = !matches!(**x, Expr::Var(_) | Expr::Const(_) | Expr::Not(_));
                if compuesto {
                    out.push("(");
                }
                let d = escribir_en(x, vars, n, out);
                if compuesto {
                    out.push(")");
                }
                out.push("'");
                d
            }
            Notacion::Neg => {
                out.push("¬");
                let compuesto = !matches!(**x, Expr::Var(_) | Expr::Const(_) | Expr::Not(_));
                if compuesto {
                    out.push("(");
                }
                let d = escribir_en(x, vars, n, out);
                if compuesto {
                    out.push(")");
                }
                d
            }
        },
        Expr::And(v) | Expr::Or(v) | Expr::Xor(v) => {
            let sep = match e {
                Expr::And(_) => "·",
                Expr::Or(_) => " + ",
                _ => " ⊕ ",
            };
            let p = prec(e);
            let mut prof = 0;
            for (k, x) in v.iter().enumerate() {
                if k > 0 {
                    out.push(sep);
                }
                let paren = prec(x) < p
                    || (prec(x) == p
                        && !matches!(x, Expr::Not(_) | Expr::Var(_) | Expr::Const(_))
                        && p != 4);
                if paren {
                    out.push("(");
                }
                prof = prof.max(escribir_en(x, vars, n, out));
                if paren {
                    out.push(")");
                }
            }
            prof
        }
    }
}

/// Ancho de carácter de la fuente monoespaciada (DejaVu Sans Mono ≈ 0,602 em).
pub fn ancho_car(size: f64) -> f64 {
    dibu_core::avance_mono(1.0, size)
}

/// Dibuja un `Escrito` con sus barras superiores. Devuelve el ancho.
#[allow(clippy::too_many_arguments)]
pub fn dibujar_escrito(
    c: &mut Lienzo,
    x: f64,
    y: f64,
    e: &Escrito,
    size: f64,
    peso: u16,
    color: &str,
    ancla: &str,
) -> f64 {
    let ancho = e.texto.chars().count() as f64 * ancho_car(size);
    let x0 = match ancla {
        "middle" => x - ancho / 2.0,
        "end" => x - ancho,
        _ => x,
    };
    c.t(x0, y, &e.texto, size, peso, color, "start", MONO);
    let cw = ancho_car(size);
    for &(a, b, nivel) in &e.barras {
        let escala = dibu_core::tipografia_activa().0;
        let yb = y - size * escala * 0.82 - nivel as f64 * 2.6 * escala;
        c.linea(
            x0 + a as f64 * cw + 0.8,
            yb,
            x0 + b as f64 * cw - 0.8,
            yb,
            color,
            1.0,
            None,
            "",
        );
    }
    ancho
}

// ---------------------------------------------------------------------------
// Minimización
// ---------------------------------------------------------------------------

/// Implicante: `valor` en las posiciones fijas, `mascara` con 1 en las posiciones libres.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Implicante {
    pub valor: u32,
    pub mascara: u32,
}

impl Implicante {
    pub fn cubre(&self, m: u32) -> bool {
        (m & !self.mascara) == (self.valor & !self.mascara)
    }
    pub fn literales(&self, n: usize) -> usize {
        n - self.mascara.count_ones() as usize
    }
    /// Producto (SOP) o suma (POS, `pos = true`) que representa.
    pub fn expr(&self, n: usize, pos: bool) -> Expr {
        let mut lits = Vec::new();
        for i in 0..n {
            let bit = 1u32 << (n - 1 - i);
            if self.mascara & bit != 0 {
                continue;
            }
            let uno = self.valor & bit != 0;
            // En POS, un 0 en la celda se escribe como la variable sin negar.
            let negada = if pos { uno } else { !uno };
            lits.push(if negada {
                Expr::Not(Box::new(Expr::Var(i)))
            } else {
                Expr::Var(i)
            });
        }
        match lits.len() {
            0 => Expr::Const(!pos),
            1 => lits.pop().unwrap(),
            _ => {
                if pos {
                    Expr::Or(lits)
                } else {
                    Expr::And(lits)
                }
            }
        }
    }
}

pub fn implicantes_primos(unos: &[u32], indiferentes: &[u32]) -> Vec<Implicante> {
    let mut actuales: Vec<Implicante> = unos
        .iter()
        .chain(indiferentes)
        .map(|&m| Implicante {
            valor: m,
            mascara: 0,
        })
        .collect();
    actuales.sort();
    actuales.dedup();
    let mut primos = Vec::new();
    while !actuales.is_empty() {
        let mut usados = vec![false; actuales.len()];
        let mut siguientes = Vec::new();
        for i in 0..actuales.len() {
            for j in i + 1..actuales.len() {
                let (a, b) = (actuales[i], actuales[j]);
                if a.mascara != b.mascara {
                    continue;
                }
                let dif = (a.valor ^ b.valor) & !a.mascara;
                if dif.count_ones() == 1 {
                    usados[i] = true;
                    usados[j] = true;
                    siguientes.push(Implicante {
                        valor: a.valor & !dif,
                        mascara: a.mascara | dif,
                    });
                }
            }
        }
        for (i, imp) in actuales.iter().enumerate() {
            if !usados[i] {
                primos.push(*imp);
            }
        }
        siguientes.sort();
        siguientes.dedup();
        actuales = siguientes;
    }
    primos.sort_by_key(|p| (std::cmp::Reverse(p.mascara.count_ones()), p.valor));
    primos
}

/// Cobertura mínima: esenciales + búsqueda exacta (pocos implicantes) o voraz.
pub fn cobertura_minima(unos: &[u32], primos: &[Implicante], n: usize) -> Vec<Implicante> {
    let mut elegidos: Vec<Implicante> = Vec::new();
    let mut pendientes: Vec<u32> = unos.to_vec();
    // Esenciales.
    for &m in unos {
        let cubren: Vec<&Implicante> = primos.iter().filter(|p| p.cubre(m)).collect();
        if cubren.len() == 1 && !elegidos.contains(cubren[0]) {
            elegidos.push(*cubren[0]);
        }
    }
    pendientes.retain(|&m| !elegidos.iter().any(|p| p.cubre(m)));
    if pendientes.is_empty() {
        return elegidos;
    }
    let candidatos: Vec<Implicante> = primos
        .iter()
        .filter(|p| !elegidos.contains(p) && pendientes.iter().any(|&m| p.cubre(m)))
        .copied()
        .collect();
    if candidatos.len() <= 16 {
        let mut mejor: Option<(usize, usize, u32)> = None;
        for sub in 1u32..(1u32 << candidatos.len()) {
            let elegidos_sub: Vec<&Implicante> = candidatos
                .iter()
                .enumerate()
                .filter(|(i, _)| sub >> i & 1 == 1)
                .map(|(_, p)| p)
                .collect();
            if pendientes
                .iter()
                .all(|&m| elegidos_sub.iter().any(|p| p.cubre(m)))
            {
                let costo = (
                    elegidos_sub.len(),
                    elegidos_sub.iter().map(|p| p.literales(n)).sum::<usize>(),
                );
                if mejor.is_none_or(|(a, b, _)| costo < (a, b)) {
                    mejor = Some((costo.0, costo.1, sub));
                }
            }
        }
        if let Some((_, _, sub)) = mejor {
            for (i, p) in candidatos.iter().enumerate() {
                if sub >> i & 1 == 1 {
                    elegidos.push(*p);
                }
            }
        }
    } else {
        while !pendientes.is_empty() {
            let p = *candidatos
                .iter()
                .max_by_key(|p| {
                    (
                        pendientes.iter().filter(|&&m| p.cubre(m)).count(),
                        p.mascara.count_ones(),
                    )
                })
                .expect("hay candidatos");
            elegidos.push(p);
            pendientes.retain(|&m| !p.cubre(m));
        }
    }
    elegidos
}

/// Minimiza a SOP (`pos = false`, sobre los unos) o POS (sobre los ceros).
pub fn minimizar(
    unos: &[u32],
    indiferentes: &[u32],
    n: usize,
    pos: bool,
) -> (Vec<Implicante>, Expr) {
    let total = 1u32 << n;
    let objetivo: Vec<u32> = if pos {
        (0..total)
            .filter(|m| !unos.contains(m) && !indiferentes.contains(m))
            .collect()
    } else {
        unos.to_vec()
    };
    if objetivo.is_empty() {
        return (Vec::new(), Expr::Const(pos));
    }
    let primos = implicantes_primos(&objetivo, indiferentes);
    let mut cob = cobertura_minima(&objetivo, &primos, n);
    cob.sort_by_key(|p| (std::cmp::Reverse(p.mascara.count_ones()), p.valor));
    let terminos: Vec<Expr> = cob.iter().map(|p| p.expr(n, pos)).collect();
    let e = if terminos.len() == 1 {
        terminos.into_iter().next().unwrap()
    } else if pos {
        Expr::And(terminos)
    } else {
        Expr::Or(terminos)
    };
    (cob, e)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn p(s: &str) -> (Expr, Vec<String>) {
        let mut v = Vec::new();
        let e = parsear(s, &mut v, false).unwrap();
        (e, v)
    }

    #[test]
    fn parsea_y_evalua_operadores() {
        let (e, v) = p("A·B + !C");
        assert_eq!(v, ["A", "B", "C"]);
        assert_eq!(minterms(&e, 3), vec![0, 2, 4, 6, 7]);
        let (e, _) = p("A'B + A B'");
        assert_eq!(minterms(&e, 2), vec![1, 2]);
        let (e, _) = p("A XOR B");
        assert_eq!(minterms(&e, 2), vec![1, 2]);
        let (e, _) = p("A NAND B");
        assert_eq!(minterms(&e, 2), vec![0, 1, 2]);
        let (e, v) = p("A'B'C + ABC'");
        assert_eq!(v, ["A", "B", "C"]);
        assert_eq!(minterms(&e, 3), vec![1, 6]);
        let mut v = Vec::new();
        parsear_con("SET + RESET", &mut v, false, false).unwrap();
        assert_eq!(v, ["SET", "RESET"]);
        let (e, _) = p("not (A or B)");
        assert_eq!(minterms(&e, 2), vec![0]);
        assert!(parsear("A +", &mut Vec::new(), false).is_err());
        assert!(parsear("(A", &mut Vec::new(), false).is_err());
    }

    #[test]
    fn minimiza_casos_clasicos() {
        // f = Σm(0,1,2,5,6,7,8,9,10,14) → B'C' + CD' + A'BD (B'D' es redundante)
        let unos = [0, 1, 2, 5, 6, 7, 8, 9, 10, 14];
        let (cob, _) = minimizar(&unos, &[], 4, false);
        assert_eq!(cob.len(), 3);
        for m in 0..16 {
            assert_eq!(cob.iter().any(|p| p.cubre(m)), unos.contains(&m));
        }
        // Con indiferentes.
        let (cob, e) = minimizar(&[1, 3, 7, 11, 15], &[0, 2, 5], 4, false);
        assert_eq!(cob.len(), 2, "{e:?}");
        // Constantes.
        assert_eq!(minimizar(&[], &[], 2, false).1, Expr::Const(false));
        assert_eq!(minimizar(&[0, 1, 2, 3], &[], 2, false).1, Expr::Const(true));
    }

    #[test]
    fn escribe_con_barras_y_primas() {
        let (e, v) = p("!(A + B)·C");
        let b = escribir(&e, &v, Notacion::Barra);
        assert_eq!(b.texto, "A + B·C");
        assert_eq!(b.barras, vec![(0, 5, 0)]);
        let pr = escribir(&e, &v, Notacion::Prima);
        assert_eq!(pr.texto, "(A + B)'·C");
    }
}
