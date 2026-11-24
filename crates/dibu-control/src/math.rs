//! Cuentas de control de procesos: polinomios, raíces (Durand-Kerner), simulación RK4,
//! respuesta en frecuencia (Bode/Nyquist/márgenes) y lugar de las raíces.

use dibu_core::{Bloque, ErrorDiagrama};
use num_complex::Complex64;
use serde_json::Value;
use std::collections::VecDeque;
use std::f64::consts::PI;

pub type C64 = Complex64;

pub const MAX_GRADO_SISTEMA: usize = 24;
const MAX_COEFICIENTES_RAW: usize = 128;

pub fn err(msg: impl Into<String>) -> ErrorDiagrama {
    ErrorDiagrama::new(msg)
}

pub fn recortar(p: &[C64]) -> Vec<C64> {
    if p.is_empty() {
        return vec![C64::new(0.0, 0.0)];
    }
    let mut i = 0;
    while i + 1 < p.len() && p[i].norm() < 1e-14 {
        i += 1;
    }
    p[i..].to_vec()
}

pub fn mul(a: &[C64], b: &[C64]) -> Vec<C64> {
    if a.is_empty() || b.is_empty() {
        return vec![C64::new(0.0, 0.0)];
    }
    let mut salida = vec![C64::new(0.0, 0.0); a.len() + b.len() - 1];
    for (i, &x) in a.iter().enumerate() {
        for (j, &y) in b.iter().enumerate() {
            salida[i + j] += x * y;
        }
    }
    salida
}

pub fn suma(a: &[C64], b: &[C64]) -> Vec<C64> {
    let n = a.len().max(b.len());
    let mut out = vec![C64::new(0.0, 0.0); n];
    let oa = n - a.len();
    let ob = n - b.len();
    for (i, &v) in a.iter().enumerate() {
        out[oa + i] += v;
    }
    for (i, &v) in b.iter().enumerate() {
        out[ob + i] += v;
    }
    recortar(&out)
}

pub fn evaluar(p: &[C64], s: C64) -> C64 {
    let mut valor = C64::new(0.0, 0.0);
    for &coef in p {
        valor = valor * s + coef;
    }
    valor
}

pub fn de_raices(rs: &[C64], k: C64) -> Vec<C64> {
    let mut p = vec![k];
    for &r in rs {
        p = mul(&p, &[C64::new(1.0, 0.0), -r]);
    }
    p
}

pub fn raices(p_in: &[C64]) -> Vec<C64> {
    let mut p = recortar(p_in);
    if p.iter().any(|c| !c.re.is_finite() || !c.im.is_finite()) {
        return Vec::new();
    }
    let mut n = p.len().saturating_sub(1);
    if n < 1 {
        return Vec::new();
    }
    let mut ceros = 0usize;
    while n > 0 && p[n].norm() < 1e-14 {
        p.pop();
        n -= 1;
        ceros += 1;
    }
    let mut salida = vec![C64::new(0.0, 0.0); ceros];
    if n == 0 {
        return salida;
    }
    let p0 = p[0];
    let monico_orig: Vec<C64> = p.iter().map(|&c| c / p0).collect();
    if monico_orig.iter().any(|c| !c.re.is_finite() || !c.im.is_finite()) {
        return salida;
    }
    let max_coef = monico_orig[1..]
        .iter()
        .map(|c| c.norm())
        .fold(0.0_f64, f64::max);
    let scale = if max_coef > 1e50 {
        let mut s = 1.0_f64;
        for (k, c) in monico_orig.iter().enumerate().skip(1) {
            let ck = c.norm();
            if ck > 0.0 {
                s = s.max(ck.powf(1.0 / k as f64));
            }
        }
        if s.is_finite() && s > 1.0 { s } else { 1.0 }
    } else {
        1.0
    };
    let mut monico = Vec::with_capacity(monico_orig.len());
    let mut factor = 1.0_f64;
    for (k, &c) in monico_orig.iter().enumerate() {
        if k > 0 {
            factor *= scale;
        }
        if factor.is_finite() && factor > 0.0 {
            monico.push(c / factor);
        } else {
            monico.push(C64::new(0.0, 0.0));
        }
    }
    let radio = 1.0
        + monico[1..]
            .iter()
            .map(|c| c.norm())
            .fold(0.0_f64, f64::max);
    if !radio.is_finite() {
        return salida;
    }
    let mut z: Vec<C64> = (0..n)
        .map(|k| {
            let ang = 2.0 * PI * (k as f64 + 0.25) / (n as f64);
            C64::from_polar(radio, ang)
        })
        .collect();

    for _ in 0..500 {
        let mut maximo = 0.0_f64;
        for i in 0..n {
            let mut den = C64::new(1.0, 0.0);
            for j in 0..n {
                if i != j {
                    den *= z[i] - z[j];
                }
            }
            if den.norm() == 0.0 || !den.re.is_finite() || !den.im.is_finite() {
                den = C64::new(1e-12, 0.0);
            }
            let delta = evaluar(&monico, z[i]) / den;
            if !delta.re.is_finite() || !delta.im.is_finite() {
                break;
            }
            z[i] -= delta;
            maximo = maximo.max(delta.norm());
        }
        if !maximo.is_finite() || maximo < 1e-13 * radio {
            break;
        }
    }

    let derivada: Vec<C64> = monico[..n]
        .iter()
        .enumerate()
        .map(|(i, &c)| c * ((n - i) as f64))
        .collect();
    for zi in &mut z {
        for _ in 0..3 {
            let d = evaluar(&derivada, *zi);
            if d.norm() > 1e-14 && d.re.is_finite() && d.im.is_finite() {
                let paso = evaluar(&monico, *zi) / d;
                if paso.re.is_finite() && paso.im.is_finite() {
                    *zi -= paso;
                }
            }
        }
        *zi *= scale;
    }

    for r in z {
        if !r.re.is_finite() || !r.im.is_finite() {
            continue;
        }
        if r.im.abs() < 1e-7 * r.norm().max(1.0) {
            salida.push(C64::new(r.re, 0.0));
        } else {
            salida.push(r);
        }
    }
    salida.sort_by(|a, b| a.re.total_cmp(&b.re).then_with(|| a.im.total_cmp(&b.im)));
    salida
}

#[derive(Debug, Clone)]
pub struct Sistema {
    pub num: Vec<C64>,
    pub den: Vec<C64>,
    pub delay: f64,
    pub etiqueta: String,
    pub pid: Option<(f64, f64, f64)>,
}

impl Sistema {
    pub fn new(num: Vec<C64>, den: Vec<C64>) -> Self {
        Self {
            num,
            den,
            delay: 0.0,
            etiqueta: String::new(),
            pid: None,
        }
    }

    pub fn eval(&self, s: C64) -> C64 {
        evaluar(&self.num, s) / evaluar(&self.den, s) * (-s * self.delay).exp()
    }

    pub fn ceros(&self) -> Vec<C64> {
        raices(&self.num)
    }

    pub fn polos(&self) -> Vec<C64> {
        raices(&self.den)
    }

    pub fn ganancia_estatica(&self) -> Option<f64> {
        let d = evaluar(&self.den, C64::new(0.0, 0.0));
        if d.norm() < 1e-12 {
            None
        } else {
            Some((evaluar(&self.num, C64::new(0.0, 0.0)) / d).re)
        }
    }

    pub fn serie(&self, otro: &Sistema) -> Sistema {
        Sistema {
            num: mul(&self.num, &otro.num),
            den: mul(&self.den, &otro.den),
            delay: self.delay + otro.delay,
            etiqueta: String::new(),
            pid: None,
        }
    }
}

pub fn numero(val: &Value, donde: &str) -> Result<f64, ErrorDiagrama> {
    if val.is_boolean() {
        return Err(err(format!("{donde}: {val} tiene que ser un número")));
    }
    let Some(x) = val.as_f64() else {
        return Err(err(format!("{donde}: {val} tiene que ser un número")));
    };
    if !x.is_finite() {
        return Err(err(format!("{donde}: tiene que ser un número finito")));
    }
    Ok(x)
}

fn coeficientes(val: Option<&Value>, donde: &str) -> Result<Vec<C64>, ErrorDiagrama> {
    let Some(v) = val else {
        return Err(err(format!(
            "{donde}: tiene que ser una lista de coeficientes, como [1, 3, 2]"
        )));
    };
    let lista: Vec<f64> = if !v.is_boolean() && v.is_number() {
        vec![numero(v, donde)?]
    } else if let Some(arr) = v.as_array() {
        if arr.is_empty() {
            return Err(err(format!(
                "{donde}: tiene que ser una lista de coeficientes, como [1, 3, 2]"
            )));
        }
        if arr.len() > MAX_COEFICIENTES_RAW {
            return Err(err(format!(
                "{donde}: el polinomio tiene demasiados coeficientes (máximo {MAX_COEFICIENTES_RAW})"
            )));
        }
        let mut out = Vec::with_capacity(arr.len());
        for item in arr {
            out.push(numero(item, donde)?);
        }
        out
    } else {
        return Err(err(format!(
            "{donde}: tiene que ser una lista de coeficientes, como [1, 3, 2]"
        )));
    };
    let p = recortar(&lista.into_iter().map(|x| C64::new(x, 0.0)).collect::<Vec<_>>());
    if p.iter().all(|c| c.norm() < 1e-14) {
        return Err(err(format!("{donde}: es el polinomio nulo")));
    }
    if p.len().saturating_sub(1) > MAX_GRADO_SISTEMA {
        return Err(err(format!(
            "{donde}: el grado del polinomio ({}) supera el máximo permitido ({MAX_GRADO_SISTEMA})",
            p.len() - 1
        )));
    }
    Ok(p)
}

fn raices_de(val: Option<&Value>, donde: &str) -> Result<Vec<C64>, ErrorDiagrama> {
    let Some(v) = val else {
        return Ok(Vec::new());
    };
    if v.is_null() {
        return Ok(Vec::new());
    }
    let Some(arr) = v.as_array() else {
        return Err(err(format!(
            "{donde}: tiene que ser una lista, como [-1, [-2, 3]]"
        )));
    };
    if arr.len() > MAX_GRADO_SISTEMA {
        return Err(err(format!(
            "{donde}: la cantidad de raíces supera el máximo permitido ({MAX_GRADO_SISTEMA})"
        )));
    }
    let mut salida = Vec::new();
    for r in arr {
        if let Some(par) = r.as_array() {
            if par.len() != 2 {
                return Err(err(format!(
                    "{donde}: un par complejo es [real, imaginaria]: {r}"
                )));
            }
            let re = numero(&par[0], donde)?;
            let im = numero(&par[1], donde)?;
            if im != 0.0 {
                salida.push(C64::new(re, im.abs()));
                salida.push(C64::new(re, -im.abs()));
            } else {
                salida.push(C64::new(re, 0.0));
            }
        } else {
            salida.push(C64::new(numero(r, donde)?, 0.0));
        }
        if salida.len() > MAX_GRADO_SISTEMA {
            return Err(err(format!(
                "{donde}: la cantidad de raíces supera el máximo permitido ({MAX_GRADO_SISTEMA})"
            )));
        }
    }
    Ok(salida)
}

pub fn grado_sistema(s: &Sistema) -> usize {
    s.num
        .len()
        .saturating_sub(1)
        .max(s.den.len().saturating_sub(1))
}

pub fn sistema(dato: &Value, donde: &str) -> Result<Sistema, ErrorDiagrama> {
    if !dato.is_boolean() && dato.is_number() {
        let k = numero(dato, donde)?;
        return Ok(Sistema::new(vec![C64::new(k, 0.0)], vec![C64::new(1.0, 0.0)]));
    }
    let Some(obj) = dato.as_object() else {
        return Err(err(format!(
            "{donde}: tiene que ser un diccionario (num/den, gain/tau, gain/zeta/wn, k/poles/zeros o kp/ki/kd)"
        )));
    };
    let delay = match obj.get("delay") {
        Some(v) => numero(v, &format!("{donde}.delay"))?,
        None => 0.0,
    };
    if delay < 0.0 {
        return Err(err(format!("{donde}.delay: el tiempo muerto no puede ser negativo")));
    }
    let etiqueta = obj
        .get("label")
        .and_then(Value::as_str)
        .unwrap_or("")
        .to_string();

    let mut s = if obj.contains_key("num") || obj.contains_key("den") {
        let num = match obj.get("num") {
            Some(v) => coeficientes(Some(v), &format!("{donde}.num"))?,
            None => vec![C64::new(1.0, 0.0)],
        };
        let den = coeficientes(obj.get("den"), &format!("{donde}.den"))?;
        Sistema::new(num, den)
    } else if obj.contains_key("tau") {
        let tau = numero(&obj["tau"], &format!("{donde}.tau"))?;
        if tau <= 0.0 {
            return Err(err(format!("{donde}.tau: tiene que ser positiva")));
        }
        let gain = match obj.get("gain") {
            Some(v) => numero(v, &format!("{donde}.gain"))?,
            None => 1.0,
        };
        Sistema::new(
            vec![C64::new(gain, 0.0)],
            vec![C64::new(tau, 0.0), C64::new(1.0, 0.0)],
        )
    } else if obj.contains_key("zeta") || obj.contains_key("wn") {
        let zeta = match obj.get("zeta") {
            Some(v) => numero(v, &format!("{donde}.zeta"))?,
            None => 1.0,
        };
        let wn = match obj.get("wn") {
            Some(v) => numero(v, &format!("{donde}.wn"))?,
            None => 1.0,
        };
        if wn <= 0.0 || zeta < 0.0 {
            return Err(err(format!(
                "{donde}: wn tiene que ser positiva y zeta no negativa"
            )));
        }
        let k = match obj.get("gain") {
            Some(v) => numero(v, &format!("{donde}.gain"))?,
            None => 1.0,
        };
        Sistema::new(
            vec![C64::new(k * wn * wn, 0.0)],
            vec![
                C64::new(1.0, 0.0),
                C64::new(2.0 * zeta * wn, 0.0),
                C64::new(wn * wn, 0.0),
            ],
        )
    } else if obj.contains_key("poles") || obj.contains_key("zeros") {
        let k = match obj.get("k") {
            Some(v) => numero(v, &format!("{donde}.k"))?,
            None => 1.0,
        };
        let zeros = raices_de(obj.get("zeros"), &format!("{donde}.zeros"))?;
        let poles = raices_de(obj.get("poles"), &format!("{donde}.poles"))?;
        let mut s = Sistema::new(
            de_raices(&zeros, C64::new(k, 0.0)),
            de_raices(&poles, C64::new(1.0, 0.0)),
        );
        for c in &mut s.num {
            if c.re.abs() < 1e15 {
                *c = C64::new((c.re * 1e12).round() / 1e12, 0.0);
            }
        }
        for c in &mut s.den {
            if c.re.abs() < 1e15 {
                *c = C64::new((c.re * 1e12).round() / 1e12, 0.0);
            }
        }
        s
    } else if obj.contains_key("kp") || obj.contains_key("ki") || obj.contains_key("kd") {
        let kp = match obj.get("kp") {
            Some(v) => numero(v, &format!("{donde}.kp"))?,
            None => 0.0,
        };
        let ki = match obj.get("ki") {
            Some(v) => numero(v, &format!("{donde}.ki"))?,
            None => 0.0,
        };
        let kd = match obj.get("kd") {
            Some(v) => numero(v, &format!("{donde}.kd"))?,
            None => 0.0,
        };
        let (num, den) = if ki != 0.0 {
            (
                recortar(&[C64::new(kd, 0.0), C64::new(kp, 0.0), C64::new(ki, 0.0)]),
                vec![C64::new(1.0, 0.0), C64::new(0.0, 0.0)],
            )
        } else {
            (
                recortar(&[C64::new(kd, 0.0), C64::new(kp, 0.0)]),
                vec![C64::new(1.0, 0.0)],
            )
        };
        let mut s = Sistema::new(num, den);
        s.pid = Some((kp, ki, kd));
        s
    } else {
        return Err(err(format!(
            "{donde}: falta num/den, gain/tau, gain/zeta/wn, k/poles/zeros o kp/ki/kd"
        )));
    };
    if s.num
        .iter()
        .chain(s.den.iter())
        .any(|c| !c.re.is_finite() || !c.im.is_finite())
    {
        return Err(err(format!(
            "{donde}: los coeficientes del sistema exceden el rango finito"
        )));
    }
    s.delay = delay;
    s.etiqueta = etiqueta;
    Ok(s)
}

#[derive(Debug, Clone)]
pub struct Lazo {
    pub planta: Sistema,
    pub controlador: Sistema,
    pub sensor: Sistema,
    pub signo: f64,
    pub etiqueta: String,
}

impl Lazo {
    pub fn abierto(&self) -> Sistema {
        self.controlador.serie(&self.planta).serie(&self.sensor)
    }

    pub fn cerrado(&self) -> Sistema {
        let cg = self.controlador.serie(&self.planta);
        let h = &self.sensor;
        let num = mul(&cg.num, &h.den);
        let prod_nh: Vec<C64> = mul(&cg.num, &h.num).into_iter().map(|c| c * (-self.signo)).collect();
        let den = suma(&mul(&cg.den, &h.den), &prod_nh);
        Sistema::new(recortar(&num), recortar(&den))
    }
}

pub fn lazo(dato: &Value, donde: &str) -> Result<Lazo, ErrorDiagrama> {
    let Some(obj) = dato.as_object() else {
        return Err(err(format!(
            "{donde}: tiene que tener plant (y opcionalmente controller, feedback y sign)"
        )));
    };
    let Some(plant_v) = obj.get("plant") else {
        return Err(err(format!(
            "{donde}: tiene que tener plant (y opcionalmente controller, feedback y sign)"
        )));
    };
    let signo_str = obj.get("sign").and_then(Value::as_str).unwrap_or("-");
    let signo = match signo_str {
        "-" => -1.0,
        "+" => 1.0,
        _ => return Err(err(format!("{donde}.sign: es '-' (realimentación negativa) o '+'"))),
    };
    let planta = sistema(plant_v, &format!("{donde}.plant"))?;
    let controlador = match obj.get("controller") {
        Some(v) => sistema(v, &format!("{donde}.controller"))?,
        None => Sistema::new(vec![C64::new(1.0, 0.0)], vec![C64::new(1.0, 0.0)]),
    };
    let sensor = match obj.get("feedback") {
        Some(v) => sistema(v, &format!("{donde}.feedback"))?,
        None => Sistema::new(vec![C64::new(1.0, 0.0)], vec![C64::new(1.0, 0.0)]),
    };
    let etiqueta = obj
        .get("label")
        .and_then(Value::as_str)
        .unwrap_or("")
        .to_string();
    Ok(Lazo {
        planta,
        controlador,
        sensor,
        signo,
        etiqueta,
    })
}

pub fn abierto_de(b: &Bloque, donde: &str) -> Result<Sistema, ErrorDiagrama> {
    let s = if let Some(cl) = b.get("closedLoop") {
        lazo(cl, "closedLoop")?.abierto()
    } else {
        let Some(sys) = b.get("system") else {
            return Err(err(format!("falta {donde} (o closedLoop)")));
        };
        sistema(sys, donde)?
    };
    if grado_sistema(&s) > MAX_GRADO_SISTEMA {
        return Err(err(format!(
            "{donde}: el grado del sistema ({}) supera el máximo permitido ({MAX_GRADO_SISTEMA})",
            grado_sistema(&s)
        )));
    }
    Ok(s)
}

struct Estado {
    d: f64,
    c: Vec<f64>,
    a: Vec<f64>,
    x: Vec<f64>,
    dt: f64,
    atraso: VecDeque<f64>,
}

impl Estado {
    fn new(s: &Sistema, dt: f64) -> Result<Self, ErrorDiagrama> {
        let num_c = recortar(&s.num);
        let den_c = recortar(&s.den);
        if num_c.len() > den_c.len() {
            return Err(err(
                "el sistema es impropio (más ceros que polos): no se puede simular",
            ));
        }
        let a0 = den_c[0];
        let den: Vec<f64> = den_c.iter().map(|&c| (c / a0).re).collect();
        let num_raw: Vec<f64> = num_c.iter().map(|&c| (c / a0).re).collect();
        let n = den.len() - 1;
        let mut num = vec![0.0; n + 1 - num_raw.len()];
        num.extend(num_raw);
        let d = num[0];
        let mut c = Vec::with_capacity(n);
        for i in (0..n).rev() {
            c.push(num[i + 1] - num[0] * den[i + 1]);
        }
        let mut a = Vec::with_capacity(n);
        for i in (1..=n).rev() {
            a.push(-den[i]);
        }
        if !s.delay.is_finite() || s.delay < 0.0 {
            return Err(err("delay tiene que ser finito y no negativo"));
        }
        if !dt.is_finite() || dt <= 0.0 {
            return Err(err("dt de simulación inválido"));
        }
        let n_atraso_f = (s.delay / dt).round();
        if !n_atraso_f.is_finite() || n_atraso_f > 100_000.0 {
            return Err(err("delay demasiado grande para simular"));
        }
        let n_atraso = n_atraso_f.max(0.0) as usize;
        let mut atraso = VecDeque::with_capacity(n_atraso);
        for _ in 0..n_atraso {
            atraso.push_back(0.0);
        }
        Ok(Self {
            d,
            c,
            a,
            x: vec![0.0; n],
            dt,
            atraso,
        })
    }

    fn deriv(&self, x: &[f64], u: f64, out: &mut [f64]) {
        let n = x.len();
        if n == 0 {
            return;
        }
        out[..n - 1].copy_from_slice(&x[1..]);
        let mut acc = u;
        for (&ai, &xi) in self.a.iter().zip(x.iter()) {
            acc += ai * xi;
        }
        out[n - 1] = acc;
    }

    fn salida(&self, u: f64) -> f64 {
        let mut acc = self.d * u;
        for (&ci, &xi) in self.c.iter().zip(self.x.iter()) {
            acc += ci * xi;
        }
        acc
    }

    fn paso(&mut self, mut u: f64) -> f64 {
        if !self.atraso.is_empty() {
            self.atraso.push_back(u);
            u = self.atraso.pop_front().unwrap();
        }
        let y = self.salida(u);
        let n = self.x.len();
        if n > 0 {
            let h = self.dt;
            let mut k1 = vec![0.0; n];
            let mut k2 = vec![0.0; n];
            let mut k3 = vec![0.0; n];
            let mut k4 = vec![0.0; n];
            let mut tmp = vec![0.0; n];

            self.deriv(&self.x, u, &mut k1);
            for i in 0..n {
                tmp[i] = self.x[i] + 0.5 * h * k1[i];
            }
            self.deriv(&tmp, u, &mut k2);
            for i in 0..n {
                tmp[i] = self.x[i] + 0.5 * h * k2[i];
            }
            self.deriv(&tmp, u, &mut k3);
            for i in 0..n {
                tmp[i] = self.x[i] + h * k3[i];
            }
            self.deriv(&tmp, u, &mut k4);
            for i in 0..n {
                self.x[i] += h / 6.0 * (k1[i] + 2.0 * k2[i] + 2.0 * k3[i] + k4[i]);
            }
        }
        y
    }
}

pub fn entrada(tipo: &str, amplitud: f64, t: f64, dt: f64) -> Result<f64, ErrorDiagrama> {
    match tipo {
        "step" => Ok(amplitud),
        "ramp" => Ok(amplitud * t),
        "impulse" => Ok(if t < dt / 2.0 { amplitud / dt } else { 0.0 }),
        _ => Err(err(format!("input: {tipo:?} es step, ramp o impulse"))),
    }
}

pub fn simular_abierto(
    s: &Sistema,
    t_fin: f64,
    tipo: &str,
    amplitud: f64,
    pasos: usize,
) -> Result<(Vec<f64>, Vec<f64>, Vec<f64>), ErrorDiagrama> {
    if !t_fin.is_finite() || t_fin <= 0.0 {
        return Err(err("tEnd tiene que ser finito y positivo"));
    }
    let dt = t_fin / pasos as f64;
    let mut estado = Estado::new(s, dt)?;
    let mut ts = Vec::with_capacity(pasos + 1);
    let mut us = Vec::with_capacity(pasos + 1);
    let mut ys = Vec::with_capacity(pasos + 1);
    for i in 0..=pasos {
        let t = i as f64 * dt;
        let u = entrada(tipo, amplitud, t, dt)?;
        let y = estado.paso(u);
        if !t.is_finite() || !u.is_finite() || !y.is_finite() || estado.x.iter().any(|x| !x.is_finite()) {
            return Err(err("la simulación diverge fuera del rango finito"));
        }
        ts.push(t);
        us.push(if tipo != "impulse" { u } else { 0.0 });
        ys.push(y);
    }
    Ok((ts, us, ys))
}

#[derive(Debug, Clone, Copy)]
pub struct Perturbacion<'a> {
    pub at: f64,
    pub amplitude: f64,
    pub where_at: &'a str,
}

pub fn simular_lazo(
    lz: &Lazo,
    t_fin: f64,
    tipo: &str,
    amplitud: f64,
    perturbacion: Option<Perturbacion<'_>>,
    pasos: usize,
) -> Result<(Vec<f64>, Vec<f64>, Vec<f64>, Vec<f64>), ErrorDiagrama> {
    if !t_fin.is_finite() || t_fin <= 0.0 {
        return Err(err("tEnd tiene que ser finito y positivo"));
    }
    let dt = t_fin / pasos as f64;
    let mut planta = Estado::new(&lz.planta, dt)?;
    let mut sensor = Estado::new(&lz.sensor, dt)?;
    let mut controlador = if lz.controlador.pid.is_some() {
        None
    } else {
        Some(Estado::new(&lz.controlador, dt)?)
    };
    let (kp, ki, kd) = lz.controlador.pid.unwrap_or((0.0, 0.0, 0.0));
    let tf = if kd != 0.0 && kp != 0.0 {
        kd / (10.0 * kp)
    } else if kd != 0.0 {
        kd / 10.0
    } else {
        0.0
    };
    let (mut integral, mut derivada, mut e_ant) = (0.0, 0.0, 0.0);
    let mut ts = Vec::with_capacity(pasos + 1);
    let mut rs = Vec::with_capacity(pasos + 1);
    let mut ys = Vec::with_capacity(pasos + 1);
    let mut us = Vec::with_capacity(pasos + 1);
    let mut y = 0.0;
    for i in 0..=pasos {
        let t = i as f64 * dt;
        let r = entrada(tipo, amplitud, t, dt)?;
        let medida = sensor.paso(y);
        let e = r + lz.signo * medida;
        let u = if let Some(ctrl) = controlador.as_mut() {
            ctrl.paso(e)
        } else {
            integral += e * dt;
            if tf != 0.0 {
                if i > 0 {
                    derivada += dt / (tf + dt) * ((e - e_ant) / dt - derivada);
                }
            } else {
                derivada = if i > 0 { (e - e_ant) / dt } else { 0.0 };
            }
            e_ant = e;
            kp * e + ki * integral + kd * derivada
        };
        let (d_in, d_out) = match perturbacion {
            Some(p) if t >= p.at => {
                if p.where_at == "input" {
                    (p.amplitude, 0.0)
                } else {
                    (0.0, p.amplitude)
                }
            }
            _ => (0.0, 0.0),
        };
        y = planta.paso(u + d_in) + d_out;
        if !t.is_finite()
            || !r.is_finite()
            || !u.is_finite()
            || !y.is_finite()
            || planta.x.iter().any(|x| !x.is_finite())
        {
            return Err(err("la simulación diverge fuera del rango finito"));
        }
        ts.push(t);
        rs.push(if tipo != "impulse" { r } else { 0.0 });
        ys.push(y);
        us.push(u);
    }
    Ok((ts, rs, ys, us))
}

#[derive(Debug, Clone, Default)]
pub struct Caracteristicas {
    pub final_val: Option<f64>,
    pub overshoot: Option<f64>,
    pub peak: Option<f64>,
    pub rise: Option<f64>,
    pub settling: Option<f64>,
    pub tau: Option<f64>,
    pub delay: Option<f64>,
}

pub fn caracteristicas(
    ts: &[f64],
    ys: &[f64],
    final_opt: Option<f64>,
    banda: f64,
    inicial: f64,
) -> Caracteristicas {
    let final_v = final_opt.unwrap_or_else(|| *ys.last().unwrap_or(&0.0));
    if !final_v.is_finite() || !inicial.is_finite() {
        return Caracteristicas::default();
    }
    let cambio = final_v - inicial;
    let mut salida = Caracteristicas {
        final_val: Some(final_v),
        ..Default::default()
    };
    if !cambio.is_finite() || cambio.abs() < 1e-12 || ys.is_empty() {
        return salida;
    }
    let signo = if cambio > 0.0 { 1.0 } else { -1.0 };
    let mut i_pico = 0;
    let mut mejor = f64::NEG_INFINITY;
    for (i, &y) in ys.iter().enumerate() {
        let v = signo * y;
        if v > mejor {
            mejor = v;
            i_pico = i;
        }
    }
    let exceso = (ys[i_pico] - final_v) / cambio * 100.0;
    if exceso > 0.05 {
        salida.overshoot = Some(exceso);
        salida.peak = Some(ts[i_pico]);
    }

    let cruce = |fraccion: f64| -> Option<f64> {
        let objetivo = inicial + fraccion * cambio;
        for i in 1..ys.len() {
            if signo * (ys[i] - objetivo) >= 0.0 {
                let (y0, y1) = (ys[i - 1], ys[i]);
                let f = if y1 != y0 {
                    (objetivo - y0) / (y1 - y0)
                } else {
                    0.0
                };
                return Some(ts[i - 1] + f * (ts[i] - ts[i - 1]));
            }
        }
        None
    };

    if let (Some(t10), Some(t90)) = (cruce(0.1), cruce(0.9)) {
        salida.rise = Some(t90 - t10);
    }
    let tolerancia = banda * cambio.abs();
    let ultimo_fuera = ys
        .iter()
        .enumerate()
        .rev()
        .find(|(_, &y)| (y - final_v).abs() > tolerancia)
        .map(|(i, _)| i);
    match ultimo_fuera {
        None => salida.settling = Some(0.0),
        Some(idx) if idx + 1 < ys.len() => salida.settling = Some(ts[idx + 1]),
        _ => {}
    }
    let inicio = ys
        .iter()
        .enumerate()
        .find(|(_, &y)| (y - inicial).abs() > 1e-4 * cambio.abs())
        .map(|(i, _)| ts[i]);
    salida.delay = inicio;
    if let (Some(t632), Some(ini)) = (cruce(0.632), inicio) {
        salida.tau = Some(t632 - ini);
    }
    salida
}

#[derive(Debug, Clone, Copy)]
pub struct TangenteInflexion {
    pub t: f64,
    pub y: f64,
    pub slope: f64,
    pub l: f64,
    pub big_t: f64,
}

pub fn tangente_inflexion(
    ts: &[f64],
    ys: &[f64],
    final_v: f64,
    inicial: f64,
) -> Option<TangenteInflexion> {
    if ys.len() < 3 {
        return None;
    }
    let signo = if final_v >= inicial { 1.0 } else { -1.0 };
    let mut mejor_i = 1usize;
    let mut mejor_val = f64::NEG_INFINITY;
    let mut mejor_m = 0.0;
    for i in 1..(ys.len() - 1) {
        let m = (ys[i + 1] - ys[i - 1]) / (ts[i + 1] - ts[i - 1]);
        if signo * m > mejor_val {
            mejor_val = signo * m;
            mejor_i = i;
            mejor_m = m;
        }
    }
    if mejor_m.abs() < 1e-12 {
        return None;
    }
    let t_l = ts[mejor_i] + (inicial - ys[mejor_i]) / mejor_m;
    let t_f = ts[mejor_i] + (final_v - ys[mejor_i]) / mejor_m;
    Some(TangenteInflexion {
        t: ts[mejor_i],
        y: ys[mejor_i],
        slope: mejor_m,
        l: t_l,
        big_t: t_f - t_l,
    })
}

pub fn fase_continua(s: &Sistema, w: f64) -> f64 {
    let mut total = 0.0;
    let k = s.num[0] / s.den[0];
    if k.im.abs() > 1e-14 || k.re < 0.0 {
        total += k.arg().to_degrees();
    }
    for z in s.ceros() {
        total += if z.norm() > 1e-12 {
            (C64::new(0.0, w) - z).arg().to_degrees()
        } else {
            90.0
        };
    }
    for p in s.polos() {
        total -= if p.norm() > 1e-12 {
            (C64::new(0.0, w) - p).arg().to_degrees()
        } else {
            90.0
        };
    }
    total - (w * s.delay).to_degrees()
}

pub fn magnitud_db(s: &Sistema, w: f64) -> f64 {
    20.0 * s.eval(C64::new(0.0, w)).norm().max(1e-300).log10()
}

pub fn asintota_db(s: &Sistema, w: f64) -> f64 {
    let k = (s.num[0] / s.den[0]).norm();
    let mut total = if k > 0.0 { 20.0 * k.log10() } else { -300.0 };
    for z in s.ceros() {
        let az = z.norm();
        total += 20.0 * (if az < 1e-12 { w } else { w.max(az) }).log10();
    }
    for p in s.polos() {
        let ap = p.norm();
        total -= 20.0 * (if ap < 1e-12 { w } else { w.max(ap) }).log10();
    }
    total
}

pub fn rango_frecuencias(s: &Sistema) -> (f64, f64) {
    let mut quiebres: Vec<f64> = s
        .ceros()
        .into_iter()
        .chain(s.polos())
        .map(|r| r.norm())
        .filter(|&a| a > 1e-12)
        .collect();
    if s.delay > 0.0 {
        quiebres.push(1.0 / s.delay);
    }
    if quiebres.is_empty() {
        return (0.01, 100.0);
    }
    let min_q = quiebres.iter().copied().fold(f64::INFINITY, f64::min);
    let max_q = quiebres.iter().copied().fold(0.0_f64, f64::max);
    (
        10_f64.powf(min_q.log10().floor() - 1.0),
        10_f64.powf(max_q.log10().ceil() + 1.0),
    )
}

fn bisectar<F: Fn(f64) -> f64>(f: F, mut a: f64, mut b: f64) -> f64 {
    let mut fa = f(a);
    for _ in 0..80 {
        let m = (a * b).sqrt();
        let fm = f(m);
        if (fm > 0.0) == (fa > 0.0) {
            a = m;
            fa = fm;
        } else {
            b = m;
        }
    }
    (a * b).sqrt()
}

#[derive(Debug, Clone, Copy, Default)]
pub struct Margenes {
    pub gm: Option<f64>,
    pub w180: Option<f64>,
    pub pm: Option<f64>,
    pub wc: Option<f64>,
}

pub fn margenes(s: &Sistema, w_min: f64, w_max: f64) -> Margenes {
    let ws: Vec<f64> = (0..=800)
        .map(|k| w_min * (w_max / w_min).powf(k as f64 / 800.0))
        .collect();
    let mut out = Margenes::default();
    let mags: Vec<f64> = ws.iter().map(|&w| magnitud_db(s, w)).collect();
    for i in 0..800 {
        if (mags[i] > 0.0) != (mags[i + 1] > 0.0) {
            let wc = bisectar(|w| magnitud_db(s, w), ws[i], ws[i + 1]);
            out.wc = Some(wc);
            out.pm = Some(180.0 + fase_continua(s, wc));
            break;
        }
    }
    let fases: Vec<f64> = ws.iter().map(|&w| fase_continua(s, w) + 180.0).collect();
    for i in 0..800 {
        if (fases[i] > 0.0) != (fases[i + 1] > 0.0) {
            let w180 = bisectar(|w| fase_continua(s, w) + 180.0, ws[i], ws[i + 1]);
            out.w180 = Some(w180);
            out.gm = Some(-magnitud_db(s, w180));
            break;
        }
    }
    out
}

pub fn lugar_raices(s: &Sistema, k_max: f64, muestras: usize) -> (Vec<f64>, Vec<Vec<C64>>) {
    let ks: Vec<f64> = (0..=muestras)
        .map(|i| {
            let t = i as f64 / muestras as f64;
            k_max * t * t
        })
        .collect();
    let mut filas: Vec<Vec<C64>> = Vec::with_capacity(ks.len());
    let mut anterior: Option<Vec<C64>> = None;
    for &k in &ks {
        let kn: Vec<C64> = s.num.iter().map(|&c| c * k).collect();
        let actuales = raices(&suma(&s.den, &kn));
        let orden = if let Some(prev) = &anterior {
            let mut libres = actuales;
            let mut ord = Vec::with_capacity(prev.len());
            for &r in prev {
                if libres.is_empty() {
                    break;
                }
                if let Some((j, _)) = libres
                    .iter()
                    .enumerate()
                    .min_by(|(_, a), (_, b)| (*a - r).norm().total_cmp(&(*b - r).norm()))
                {
                    ord.push(libres.remove(j));
                }
            }
            ord
        } else {
            actuales
        };
        anterior = Some(orden.clone());
        filas.push(orden);
    }
    let n_ramas = filas.first().map_or(0, Vec::len);
    let mut ramas = vec![Vec::with_capacity(ks.len()); n_ramas];
    for fila in filas {
        for (i, r) in fila.into_iter().enumerate() {
            if i < n_ramas {
                ramas[i].push(r);
            }
        }
    }
    (ks, ramas)
}

pub fn asintotas(s: &Sistema) -> (Option<f64>, Vec<f64>) {
    let polos = s.polos();
    let ceros = s.ceros();
    let n = polos.len();
    let m = ceros.len();
    if n <= m {
        return (None, Vec::new());
    }
    let diff = (n - m) as f64;
    let sum_p: f64 = polos.iter().map(|p| p.re).sum();
    let sum_z: f64 = ceros.iter().map(|z| z.re).sum();
    let centro = (sum_p - sum_z) / diff;
    let angulos = (0..(n - m))
        .map(|q| 180.0 * (2 * q + 1) as f64 / diff)
        .collect();
    (Some(centro), angulos)
}

fn refinar_cruce_imaginario(s: &Sistema, mut lo: f64, mut hi: f64) -> Option<(f64, f64)> {
    let mayor_real = |k: f64| -> f64 {
        let kn: Vec<C64> = s.num.iter().map(|&c| c * k).collect();
        raices(&suma(&s.den, &kn))
            .into_iter()
            .map(|r| r.re)
            .fold(f64::NEG_INFINITY, f64::max)
    };
    for _ in 0..60 {
        let mid = 0.5 * (lo + hi);
        if mayor_real(mid) < 0.0 {
            lo = mid;
        } else {
            hi = mid;
        }
    }
    let kn: Vec<C64> = s.num.iter().map(|&c| c * hi).collect();
    let rs = raices(&suma(&s.den, &kn));
    let w = rs
        .iter()
        .filter(|r| r.re.abs() < 1e-3 * r.norm().max(1.0) + 1e-6)
        .map(|r| r.im.abs())
        .fold(0.0_f64, f64::max);
    Some((hi, w))
}

pub fn cruce_imaginario(s: &Sistema, k_max: f64) -> Option<(f64, f64)> {
    let mayor_real = |k: f64| -> f64 {
        let kn: Vec<C64> = s.num.iter().map(|&c| c * k).collect();
        raices(&suma(&s.den, &kn))
            .into_iter()
            .map(|r| r.re)
            .fold(f64::NEG_INFINITY, f64::max)
    };
    let pasos = 2000;
    let mut previo = mayor_real(0.0);
    for i in 1..=pasos {
        let a = k_max * (i - 1) as f64 / pasos as f64;
        let b = k_max * i as f64 / pasos as f64;
        let actual = mayor_real(b);
        if previo < 0.0 && actual >= 0.0 {
            return refinar_cruce_imaginario(s, a, b);
        }
        previo = actual;
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn raices_cuadratica_y_origen() {
        // s^3 + 3 s^2 + 2 s = s(s+1)(s+2)
        let rs = raices(&[
            C64::new(1.0, 0.0),
            C64::new(3.0, 0.0),
            C64::new(2.0, 0.0),
            C64::new(0.0, 0.0),
        ]);
        assert_eq!(rs.len(), 3);
        assert!((rs[0].re + 2.0).abs() < 1e-9);
        assert!((rs[1].re + 1.0).abs() < 1e-9);
        assert!(rs[2].re.abs() < 1e-9);
    }

    #[test]
    fn simulacion_primer_orden_converge_a_ganancia() {
        let s = sistema(&json!({"gain": 2.0, "tau": 1.0}), "system").unwrap();
        let (ts, _us, ys) = simular_abierto(&s, 8.0, "step", 1.0, 2000).unwrap();
        let c = caracteristicas(&ts, &ys, s.ganancia_estatica(), 0.02, 0.0);
        assert!((c.final_val.unwrap() - 2.0).abs() < 1e-6);
        assert!((c.tau.unwrap() - 1.0).abs() < 0.05);
    }

    #[test]
    fn margenes_sistema_tercer_orden() {
        // G(s) = 1 / (s+1)^3 -> cruce -180° en w = sqrt(3) ~ 1.732, |G(j*sqrt(3))| = 1/8 -> GM = +18.06 dB
        let s = sistema(&json!({"num": [1], "den": [1, 3, 3, 1]}), "system").unwrap();
        let m = margenes(&s, 0.1, 100.0);
        assert!((m.w180.unwrap() - 3.0_f64.sqrt()).abs() < 1e-3);
        assert!((m.gm.unwrap() - 18.0618).abs() < 0.05);
    }

    #[test]
    fn time_response_rechaza_delay_extremo_o_invalido_sin_panic() {
        let base = sistema(&json!({"gain": 1.0, "tau": 1.0}), "system").unwrap();
        for malo in [1e308, f64::NAN, f64::INFINITY, f64::NEG_INFINITY, -0.5] {
            let mut s = base.clone();
            s.delay = malo;
            assert!(
                Estado::new(&s, 0.01).is_err(),
                "Estado::new debe rechazar delay={malo:?}"
            );
            assert!(
                simular_abierto(&s, 10.0, "step", 1.0, 3000).is_err(),
                "simular_abierto debe rechazar delay={malo:?}"
            );
        }

        // También desde bloque completo de time_response (delay gigante y negativo)
        for val in [
            json!({"type": "time_response", "system": {"gain": 1, "tau": 1, "delay": 1e308}}),
            json!({"type": "time_response", "system": {"gain": 1, "tau": 1, "delay": -1.0}}),
        ] {
            let mut lienzo = dibu_core::Lienzo::new(dibu_core::Paleta::para(true), "test".to_string());
            let res = crate::plots::render_time_response(&mut lienzo, val.as_object().unwrap());
            assert!(res.is_err(), "render_time_response debe devolver Err para {val}");
        }
    }
}
