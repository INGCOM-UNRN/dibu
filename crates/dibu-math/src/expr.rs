//! Expresiones matemáticas de una variable para señales: `2·sin(2π·t) + u(t−1)`.
//!
//! Operadores `+ − * / ^` (y `·`, `−`, `×`), multiplicación implícita (`2t`, `3(t+1)`,
//! `2 sin(t)`), constantes `pi`/`π` y `e`, y funciones: sin cos tan exp log/ln log10 sqrt
//! abs sgn u/step rect tri sinc ramp/r floor ceil round min max pow mod delta.

use dibu_comp::comun::{err, ErrorDiagrama};

#[derive(Debug, Clone)]
pub enum Ex {
    Num(f64),
    Var,
    Neg(Box<Ex>),
    Bin(char, Box<Ex>, Box<Ex>),
    Fun(String, Vec<Ex>),
}

#[derive(Debug, Clone, PartialEq)]
enum Tok {
    Num(f64),
    Id(String),
    Op(char),
    Abre,
    Cierra,
    Coma,
}

fn tokenizar(s: &str) -> Result<Vec<Tok>, ErrorDiagrama> {
    let cs: Vec<char> = s.chars().collect();
    let mut out = Vec::new();
    let mut i = 0;
    while i < cs.len() {
        let ch = cs[i];
        if ch.is_whitespace() {
            i += 1;
        } else if ch.is_ascii_digit()
            || (ch == '.' && cs.get(i + 1).is_some_and(|c| c.is_ascii_digit()))
        {
            let ini = i;
            while i < cs.len() && (cs[i].is_ascii_digit() || cs[i] == '.') {
                i += 1;
            }
            if i < cs.len()
                && (cs[i] == 'e' || cs[i] == 'E')
                && cs
                    .get(i + 1)
                    .is_some_and(|c| c.is_ascii_digit() || *c == '-' || *c == '+')
            {
                i += 2;
                while i < cs.len() && cs[i].is_ascii_digit() {
                    i += 1;
                }
            }
            let t: String = cs[ini..i].iter().collect();
            out.push(Tok::Num(
                t.parse()
                    .map_err(|_| err(format!("número inválido {t:?}")))?,
            ));
        } else if ch.is_alphabetic() || ch == '_' {
            let ini = i;
            while i < cs.len() && (cs[i].is_alphanumeric() || cs[i] == '_') {
                i += 1;
            }
            out.push(Tok::Id(cs[ini..i].iter().collect()));
        } else {
            out.push(match ch {
                '+' => Tok::Op('+'),
                '-' | '−' => Tok::Op('-'),
                '*' | '·' | '×' => Tok::Op('*'),
                '/' => Tok::Op('/'),
                '^' => Tok::Op('^'),
                '(' | '[' => Tok::Abre,
                ')' | ']' => Tok::Cierra,
                ',' | ';' => Tok::Coma,
                _ => return Err(err(format!("carácter inesperado {ch:?} en {s:?}"))),
            });
            i += 1;
        }
    }
    Ok(out)
}

struct P<'a> {
    t: Vec<Tok>,
    i: usize,
    var: &'a str,
}

impl P<'_> {
    fn ver(&self) -> Option<&Tok> {
        self.t.get(self.i)
    }
    fn suma(&mut self) -> Result<Ex, ErrorDiagrama> {
        let mut e = self.producto()?;
        while let Some(Tok::Op(o @ ('+' | '-'))) = self.ver().cloned() {
            self.i += 1;
            e = Ex::Bin(o, Box::new(e), Box::new(self.producto()?));
        }
        Ok(e)
    }
    fn producto(&mut self) -> Result<Ex, ErrorDiagrama> {
        let mut e = self.unario()?;
        loop {
            match self.ver().cloned() {
                Some(Tok::Op(o @ ('*' | '/'))) => {
                    self.i += 1;
                    e = Ex::Bin(o, Box::new(e), Box::new(self.unario()?));
                }
                // Multiplicación implícita.
                Some(Tok::Num(_)) | Some(Tok::Id(_)) | Some(Tok::Abre) => {
                    e = Ex::Bin('*', Box::new(e), Box::new(self.potencia()?));
                }
                _ => return Ok(e),
            }
        }
    }
    fn unario(&mut self) -> Result<Ex, ErrorDiagrama> {
        match self.ver() {
            Some(Tok::Op('-')) => {
                self.i += 1;
                Ok(Ex::Neg(Box::new(self.unario()?)))
            }
            Some(Tok::Op('+')) => {
                self.i += 1;
                self.unario()
            }
            _ => self.potencia(),
        }
    }
    fn potencia(&mut self) -> Result<Ex, ErrorDiagrama> {
        let base = self.atomo()?;
        if self.ver() == Some(&Tok::Op('^')) {
            self.i += 1;
            let exp = self.unario()?;
            return Ok(Ex::Bin('^', Box::new(base), Box::new(exp)));
        }
        Ok(base)
    }
    fn atomo(&mut self) -> Result<Ex, ErrorDiagrama> {
        let tok = self.t.get(self.i).cloned();
        self.i += 1;
        match tok {
            Some(Tok::Num(x)) => Ok(Ex::Num(x)),
            Some(Tok::Abre) => {
                let e = self.suma()?;
                if self.ver() != Some(&Tok::Cierra) {
                    return Err(err("falta ')'"));
                }
                self.i += 1;
                Ok(e)
            }
            Some(Tok::Id(id)) => {
                if id == self.var {
                    return Ok(Ex::Var);
                }
                match id.as_str() {
                    "pi" | "π" | "PI" => return Ok(Ex::Num(std::f64::consts::PI)),
                    "e" => return Ok(Ex::Num(std::f64::consts::E)),
                    _ => {}
                }
                if self.ver() != Some(&Tok::Abre) {
                    return Err(err(format!(
                        "identificador desconocido {id:?} (la variable es {:?})",
                        self.var
                    )));
                }
                self.i += 1;
                let mut args = vec![self.suma()?];
                while self.ver() == Some(&Tok::Coma) {
                    self.i += 1;
                    args.push(self.suma()?);
                }
                if self.ver() != Some(&Tok::Cierra) {
                    return Err(err(format!("falta ')' en {id}(…)")));
                }
                self.i += 1;
                let nombre = id.to_lowercase();
                let aridad = match nombre.as_str() {
                    "min" | "max" | "pow" | "mod" => 2,
                    "sin" | "cos" | "tan" | "exp" | "log" | "ln" | "log10" | "sqrt" | "abs"
                    | "sgn" | "sign" | "u" | "step" | "rect" | "tri" | "sinc" | "ramp" | "r"
                    | "floor" | "ceil" | "round" | "delta" | "atan" | "asin" | "acos" | "sinh"
                    | "cosh" | "tanh" => 1,
                    _ => return Err(err(format!("función desconocida {id:?}"))),
                };
                if args.len() != aridad {
                    return Err(err(format!("{id} recibe {aridad} argumento(s)")));
                }
                Ok(Ex::Fun(nombre, args))
            }
            otro => Err(err(format!("expresión mal formada (llegó {otro:?})"))),
        }
    }
}

pub fn parsear(s: &str, var: &str) -> Result<Ex, ErrorDiagrama> {
    if s.chars().count() > 1000 {
        return Err(err("expresión demasiado larga"));
    }
    let t = tokenizar(s)?;
    if t.is_empty() {
        return Err(err("expresión vacía"));
    }
    let mut p = P { t, i: 0, var };
    let e = p.suma()?;
    if p.i < p.t.len() {
        return Err(err(format!("sobra texto en {s:?}")));
    }
    Ok(e)
}

pub fn evaluar(e: &Ex, x: f64) -> f64 {
    match e {
        Ex::Num(v) => *v,
        Ex::Var => x,
        Ex::Neg(a) => -evaluar(a, x),
        Ex::Bin(o, a, b) => {
            let (a, b) = (evaluar(a, x), evaluar(b, x));
            match o {
                '+' => a + b,
                '-' => a - b,
                '*' => a * b,
                '/' => a / b,
                _ => a.powf(b),
            }
        }
        Ex::Fun(f, args) => {
            let a = evaluar(&args[0], x);
            match f.as_str() {
                "sin" => a.sin(),
                "cos" => a.cos(),
                "tan" => a.tan(),
                "atan" => a.atan(),
                "asin" => a.asin(),
                "acos" => a.acos(),
                "sinh" => a.sinh(),
                "cosh" => a.cosh(),
                "tanh" => a.tanh(),
                "exp" => a.exp(),
                "log" | "ln" => a.ln(),
                "log10" => a.log10(),
                "sqrt" => a.sqrt(),
                "abs" => a.abs(),
                "sgn" | "sign" => {
                    if a > 0.0 {
                        1.0
                    } else if a < 0.0 {
                        -1.0
                    } else {
                        0.0
                    }
                }
                "u" | "step" => {
                    if a >= 0.0 {
                        1.0
                    } else {
                        0.0
                    }
                }
                "rect" => {
                    if a.abs() < 0.5 {
                        1.0
                    } else if a.abs() == 0.5 {
                        0.5
                    } else {
                        0.0
                    }
                }
                "tri" => (1.0 - a.abs()).max(0.0),
                "sinc" => {
                    if a.abs() < 1e-12 {
                        1.0
                    } else {
                        (std::f64::consts::PI * a).sin() / (std::f64::consts::PI * a)
                    }
                }
                "ramp" | "r" => a.max(0.0),
                "floor" => a.floor(),
                "ceil" => a.ceil(),
                "round" => a.round(),
                "delta" => {
                    if a.abs() < 1e-9 {
                        1.0
                    } else {
                        0.0
                    }
                }
                _ => {
                    let b = evaluar(&args[1], x);
                    match f.as_str() {
                        "min" => a.min(b),
                        "max" => a.max(b),
                        "pow" => a.powf(b),
                        _ => a.rem_euclid(b),
                    }
                }
            }
        }
    }
}

/// Valores de SI: "4.7k", "10mH", "100 uF", "2,2µF", "1M" (mega), "5 Ω".
pub fn valor_si(s: &str) -> Option<f64> {
    let t = s.trim().replace(',', ".");
    let fin_num = t
        .find(|c: char| {
            !(c.is_ascii_digit() || c == '.' || c == '-' || c == '+' || c == 'e' || c == 'E')
        })
        .unwrap_or(t.len());
    // Evitar que la 'e' de una unidad se tome como exponente.
    let (mut n, mut resto) = (&t[..fin_num], t[fin_num..].trim());
    if n.ends_with(['e', 'E']) {
        n = &t[..fin_num - 1];
        resto = t[fin_num - 1..].trim();
    }
    let base: f64 = n.parse().ok()?;
    let mult = match resto.chars().next() {
        Some('p') => 1e-12,
        Some('n') => 1e-9,
        Some('u') | Some('µ') | Some('μ') => 1e-6,
        Some('m') => 1e-3,
        Some('k') | Some('K') => 1e3,
        Some('M') => 1e6,
        Some('G') => 1e9,
        _ => 1.0,
    };
    // "m" sola puede ser metro, pero en circuitos se usa como mili.
    Some(base * mult)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn evalua_expresiones() {
        let e = parsear("2sin(pi t) + u(t-1)", "t").unwrap();
        assert!((evaluar(&e, 0.5) - 2.0).abs() < 1e-12);
        assert!(
            (evaluar(&e, 1.5) - (2.0 * (1.5 * std::f64::consts::PI).sin() + 1.0)).abs() < 1e-12
        );
        let e = parsear("rect(t/2) * tri(t)", "t").unwrap();
        assert_eq!(evaluar(&e, 0.5), 0.5);
        let e = parsear("-2^2", "t").unwrap();
        assert_eq!(evaluar(&e, 0.0), -4.0);
        let e = parsear("0.8^n u(n)", "n").unwrap();
        assert!((evaluar(&e, 2.0) - 0.64).abs() < 1e-12);
        assert!(parsear("foo(t)", "t").is_err());
        assert!(parsear("2 + x", "t").is_err());
    }

    #[test]
    fn valores_si() {
        assert_eq!(valor_si("4.7k"), Some(4700.0));
        assert!((valor_si("100uF").unwrap() - 1e-4).abs() < 1e-18);
        assert!((valor_si("10 mH").unwrap() - 0.01).abs() < 1e-15);
        assert_eq!(valor_si("2,2"), Some(2.2));
        assert_eq!(valor_si("1M"), Some(1e6));
        assert_eq!(valor_si("12 V"), Some(12.0));
        assert_eq!(valor_si("abc"), None);
    }
}
