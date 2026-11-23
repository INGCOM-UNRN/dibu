//! Cálculos químicos para los diagramas de `dibu-chem`: configuraciones electrónicas, orbitales
//! moleculares de diatómicas, titulaciones y especiación ácido-base, cinética, diagramas de fases,
//! curvas de calentamiento, electroquímica, celdas unidad, RMN, orgánica e inorgánica.

use dibu_core::{Bloque, ErrorDiagrama};
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};
use std::f64::consts::PI;
use std::fmt;

pub const R: f64 = 8.314462618; // J/(mol·K)
pub const F: f64 = 96485.33212; // C/mol
pub const NA: f64 = 6.02214076e23;
pub const KW: f64 = 1e-14; // a 25 °C

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ErrorQuimica(pub String);

impl fmt::Display for ErrorQuimica {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl std::error::Error for ErrorQuimica {}

impl From<ErrorQuimica> for ErrorDiagrama {
    fn from(e: ErrorQuimica) -> Self {
        ErrorDiagrama::new(e.0)
    }
}

/// Formatea un `f64` al estilo `{x:g}` de Python (6 cifras significativas, sin ceros finales
/// innecesarios y con punto decimal).
pub fn fmt_g_py(x: f64) -> String {
    if !x.is_finite() {
        return x.to_string();
    }
    if x == 0.0 {
        return "0".to_string();
    }
    let s = format!("{x:.5e}");
    let Some((mantisa_raw, exp_raw)) = s.split_once('e') else {
        return s;
    };
    let exp: i32 = exp_raw.parse().unwrap_or(0);
    if (-4..6).contains(&exp) {
        let dec = (5 - exp).max(0) as usize;
        let mut out = format!("{x:.dec$}");
        recortar_ceros_decimales(&mut out);
        if out == "-0" {
            "0".to_string()
        } else {
            out
        }
    } else {
        let mut mantisa = mantisa_raw.to_string();
        recortar_ceros_decimales(&mut mantisa);
        if exp < 0 {
            format!("{mantisa}e-{:02}", -exp)
        } else {
            format!("{mantisa}e+{exp:02}")
        }
    }
}

fn recortar_ceros_decimales(s: &mut String) {
    if s.contains('.') {
        while s.ends_with('0') {
            s.pop();
        }
        if s.ends_with('.') {
            s.pop();
        }
    }
}

/// Convierte un `Value` JSON/YAML a `f64` finito con validación opcional de positividad y mínimo.
pub fn num(
    valor: &Value,
    donde: &str,
    positivo: bool,
    minimo: Option<f64>,
) -> Result<f64, ErrorQuimica> {
    let x = match valor {
        Value::Number(n) => n
            .as_f64()
            .ok_or_else(|| ErrorQuimica(format!("{donde}: {valor} no es un número")))?,
        Value::String(s) => s
            .trim()
            .parse::<f64>()
            .map_err(|_| ErrorQuimica(format!("{donde}: '{s}' no es un número")))?,
        _ => {
            return Err(ErrorQuimica(format!("{donde}: {valor} no es un número")));
        }
    };
    if !x.is_finite() {
        return Err(ErrorQuimica(format!(
            "{donde}: tiene que ser un número finito"
        )));
    }
    if positivo && x <= 0.0 {
        return Err(ErrorQuimica(format!("{donde}: tiene que ser positivo")));
    }
    if let Some(m) = minimo {
        if x < m {
            return Err(ErrorQuimica(format!(
                "{donde}: tiene que ser al menos {}",
                fmt_g_py(m)
            )));
        }
    }
    Ok(x)
}

/// Extrae un entero exacto de un `Value` JSON/YAML (rechaza booleanos, NaN, infinitos y decimales no enteros).
pub fn entero_val(valor: &Value, donde: &str) -> Result<i32, ErrorQuimica> {
    match valor {
        Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                i32::try_from(i)
                    .map_err(|_| ErrorQuimica(format!("{donde}: {valor} fuera de rango entero")))
            } else if let Some(u) = n.as_u64() {
                i32::try_from(u)
                    .map_err(|_| ErrorQuimica(format!("{donde}: {valor} fuera de rango entero")))
            } else if let Some(f) = n.as_f64() {
                if f.is_finite()
                    && (f - f.round()).abs() < 1e-12
                    && f >= i32::MIN as f64
                    && f <= i32::MAX as f64
                {
                    Ok(f.round() as i32)
                } else {
                    Err(ErrorQuimica(format!("{donde}: {valor} no es un entero")))
                }
            } else {
                Err(ErrorQuimica(format!("{donde}: {valor} no es un entero")))
            }
        }
        Value::String(s) => s
            .trim()
            .parse::<i32>()
            .map_err(|_| ErrorQuimica(format!("{donde}: '{s}' no es un entero"))),
        _ => Err(ErrorQuimica(format!("{donde}: {valor} no es un entero"))),
    }
}

/// Normaliza un `Option<&Value>` a una lista de referencias: `None`/`Null` → vacío, array → elementos, escalar → `[v]`.
pub fn lista(valor: Option<&Value>) -> Vec<&Value> {
    match valor {
        None | Some(Value::Null) => Vec::new(),
        Some(Value::Array(arr)) => arr.iter().collect(),
        Some(v) => vec![v],
    }
}

fn val_a_texto_raw(v: &Value) -> String {
    match v {
        Value::Null => String::new(),
        Value::String(s) => s.clone(),
        Value::Number(n) => n.to_string(),
        Value::Bool(b) => if *b { "true" } else { "false" }.to_string(),
        _ => v.to_string(),
    }
}

fn es_truthy(v: &Value) -> bool {
    match v {
        Value::Null => false,
        Value::Bool(b) => *b,
        Value::Number(n) => n.as_f64().is_some_and(|x| x != 0.0),
        Value::String(s) => !s.is_empty(),
        Value::Array(a) => !a.is_empty(),
        Value::Object(m) => !m.is_empty(),
    }
}

// --- Elementos y configuraciones electrónicas -------------------------------------------------------------

pub const SIMBOLOS: [&str; 118] = [
    "H", "He", "Li", "Be", "B", "C", "N", "O", "F", "Ne", "Na", "Mg", "Al", "Si", "P", "S", "Cl",
    "Ar", "K", "Ca", "Sc", "Ti", "V", "Cr", "Mn", "Fe", "Co", "Ni", "Cu", "Zn", "Ga", "Ge", "As",
    "Se", "Br", "Kr", "Rb", "Sr", "Y", "Zr", "Nb", "Mo", "Tc", "Ru", "Rh", "Pd", "Ag", "Cd", "In",
    "Sn", "Sb", "Te", "I", "Xe", "Cs", "Ba", "La", "Ce", "Pr", "Nd", "Pm", "Sm", "Eu", "Gd", "Tb",
    "Dy", "Ho", "Er", "Tm", "Yb", "Lu", "Hf", "Ta", "W", "Re", "Os", "Ir", "Pt", "Au", "Hg", "Tl",
    "Pb", "Bi", "Po", "At", "Rn", "Fr", "Ra", "Ac", "Th", "Pa", "U", "Np", "Pu", "Am", "Cm", "Bk",
    "Cf", "Es", "Fm", "Md", "No", "Lr", "Rf", "Db", "Sg", "Bh", "Hs", "Mt", "Ds", "Rg", "Cn", "Nh",
    "Fl", "Mc", "Lv", "Ts", "Og",
];

pub const GASES_NOBLES: [(&str, u8); 6] = [
    ("He", 2),
    ("Ne", 10),
    ("Ar", 18),
    ("Kr", 36),
    ("Xe", 54),
    ("Rn", 86),
];

/// Electronegatividad de Pauling (H..Ar), para ordenar los orbitales atómicos de una diatómica heteronuclear.
pub const ELECTRONEGATIVIDAD: [(&str, f64); 18] = [
    ("H", 2.20),
    ("He", 0.0),
    ("Li", 0.98),
    ("Be", 1.57),
    ("B", 2.04),
    ("C", 2.55),
    ("N", 3.04),
    ("O", 3.44),
    ("F", 3.98),
    ("Ne", 0.0),
    ("Na", 0.93),
    ("Mg", 1.31),
    ("Al", 1.61),
    ("Si", 1.90),
    ("P", 2.19),
    ("S", 2.58),
    ("Cl", 3.16),
    ("Ar", 0.0),
];

pub fn electronegatividad(simbolo: &str) -> f64 {
    ELECTRONEGATIVIDAD
        .iter()
        .find(|&&(s, _)| s == simbolo)
        .map_or(0.0, |&(_, en)| en)
}

pub const LETRAS: &str = "spdf";

/// Orden de llenado de Madelung `(n + l, n)` para los 22 subniveles hasta 7f.
pub const SUBNIVELES: [(u8, u8); 22] = [
    (1, 0),
    (2, 0),
    (2, 1),
    (3, 0),
    (3, 1),
    (4, 0),
    (3, 2),
    (4, 1),
    (5, 0),
    (4, 2),
    (5, 1),
    (6, 0),
    (4, 3),
    (5, 2),
    (6, 1),
    (7, 0),
    (5, 3),
    (6, 2),
    (7, 1),
    (6, 3),
    (7, 2),
    (7, 3),
];

pub fn excepciones(simbolo: &str) -> &'static [((u8, u8), u8)] {
    match simbolo {
        "Cr" => &[((4, 0), 1), ((3, 2), 5)],
        "Cu" => &[((4, 0), 1), ((3, 2), 10)],
        "Nb" => &[((5, 0), 1), ((4, 2), 4)],
        "Mo" => &[((5, 0), 1), ((4, 2), 5)],
        "Ru" => &[((5, 0), 1), ((4, 2), 7)],
        "Rh" => &[((5, 0), 1), ((4, 2), 8)],
        "Pd" => &[((5, 0), 0), ((4, 2), 10)],
        "Ag" => &[((5, 0), 1), ((4, 2), 10)],
        "Pt" => &[((6, 0), 1), ((5, 2), 9)],
        "Au" => &[((6, 0), 1), ((5, 2), 10)],
        _ => &[],
    }
}

pub fn nombre_subnivel(nl: (u8, u8)) -> String {
    let letra = LETRAS.chars().nth(nl.1 as usize).unwrap_or('?');
    format!("{}{}", nl.0, letra)
}

pub fn capacidad(nl: (u8, u8)) -> u8 {
    2 * (2 * nl.1 + 1)
}

pub fn numero_atomico_str(elemento: &str) -> Result<u8, ErrorQuimica> {
    let texto = elemento.trim();
    let z: i64 = if !texto.is_empty() && texto.chars().all(|c| c.is_ascii_digit()) {
        texto
            .parse::<i64>()
            .map_err(|_| ErrorQuimica(format!("número atómico fuera de rango: {texto}")))?
    } else if let Some(pos) = SIMBOLOS.iter().position(|&s| s == texto) {
        (pos + 1) as i64
    } else {
        return Err(ErrorQuimica(format!("elemento desconocido: '{elemento}'")));
    };
    if !(1..=SIMBOLOS.len() as i64).contains(&z) {
        return Err(ErrorQuimica(format!("número atómico fuera de rango: {z}")));
    }
    Ok(z as u8)
}

pub fn numero_atomico(elemento: &Value) -> Result<u8, ErrorQuimica> {
    let z: i64 = match elemento {
        Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                i
            } else if let Some(u) = n.as_u64() {
                i64::try_from(u)
                    .map_err(|_| ErrorQuimica(format!("número atómico fuera de rango: {elemento}")))?
            } else if let Some(f) = n.as_f64() {
                if f.is_finite() && (f - f.round()).abs() < 1e-12 && (0.0..=1e9).contains(&f) {
                    f.round() as i64
                } else {
                    return Err(ErrorQuimica(format!("elemento desconocido: {elemento}")));
                }
            } else {
                return Err(ErrorQuimica(format!("elemento desconocido: {elemento}")));
            }
        }
        Value::String(s) => return numero_atomico_str(s),
        _ => {
            return Err(ErrorQuimica(format!("elemento desconocido: {elemento}")));
        }
    };
    if !(1..=SIMBOLOS.len() as i64).contains(&z) {
        return Err(ErrorQuimica(format!("número atómico fuera de rango: {z}")));
    }
    Ok(z as u8)
}

pub fn superindice(texto: &str) -> String {
    texto
        .chars()
        .map(|ch| match ch {
            '0' => '⁰',
            '1' => '¹',
            '2' => '²',
            '3' => '³',
            '4' => '⁴',
            '5' => '⁵',
            '6' => '⁶',
            '7' => '⁷',
            '8' => '⁸',
            '9' => '⁹',
            '+' => '⁺',
            '-' => '⁻',
            otro => otro,
        })
        .collect()
}

pub fn superindice_num(n: u8) -> String {
    superindice(&n.to_string())
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Configuracion {
    pub simbolo: String,
    pub z: u8,
    pub carga: i32,
    /// Ocupación en orden de llenado de Madelung (`SUBNIVELES`), conservando subniveles `s` vacíos no finales.
    pub ocupacion: Vec<((u8, u8), u8)>,
}

impl Configuracion {
    pub fn electrones(&self) -> u16 {
        self.ocupacion.iter().map(|&(_, e)| u16::from(e)).sum()
    }

    pub fn ocupacion_de(&self, nl: (u8, u8)) -> u8 {
        self.ocupacion
            .iter()
            .find(|&&(k, _)| k == nl)
            .map_or(0, |&(_, e)| e)
    }

    pub fn nucleo(&self) -> Option<(&'static str, Vec<(u8, u8)>)> {
        let e_tot = self.electrones();
        let mut previo: Option<(&'static str, u8)> = None;
        for &(simbolo, z) in &GASES_NOBLES {
            if z < self.z && u16::from(z) <= e_tot {
                previo = Some((simbolo, z));
            }
        }
        let (simbolo, z_gas) = previo?;
        let mut llenos = Vec::new();
        let mut cuenta: u8 = 0;
        for &nl in &SUBNIVELES {
            if cuenta >= z_gas {
                break;
            }
            llenos.push(nl);
            cuenta = cuenta.saturating_add(capacidad(nl));
        }
        Some((simbolo, llenos))
    }

    pub fn texto(&self, nucleo: bool) -> String {
        let gas = if nucleo { self.nucleo() } else { None };
        let prefijo = match &gas {
            Some((sym, _)) => format!("[{sym}] "),
            None => String::new(),
        };
        let partes: Vec<String> = self
            .ocupacion
            .iter()
            .filter(|&&(nl, e)| {
                e > 0 && gas.as_ref().is_none_or(|(_, llenos)| !llenos.contains(&nl))
            })
            .map(|&(nl, e)| format!("{}{}", nombre_subnivel(nl), superindice_num(e)))
            .collect();
        format!("{prefijo}{}", partes.join(" "))
    }

    pub fn desapareados(&self) -> usize {
        let mut total = 0usize;
        for &(nl, e) in &self.ocupacion {
            let cajas = usize::from(2 * nl.1 + 1);
            let e = usize::from(e);
            total += if e <= cajas {
                e
            } else {
                (2 * cajas).saturating_sub(e)
            };
        }
        total
    }
}

pub fn configuracion(elemento: &Value, carga: i32) -> Result<Configuracion, ErrorQuimica> {
    let z = numero_atomico(elemento)?;
    let simbolo = SIMBOLOS[(z - 1) as usize].to_string();
    let electrones = i64::from(z) - i64::from(carga);
    if electrones < 0 {
        return Err(ErrorQuimica(format!(
            "{simbolo} con carga {carga:+} no tiene electrones"
        )));
    }
    let mut ocupacion: BTreeMap<(u8, u8), u8> = BTreeMap::new();
    let mut quedan = z;
    for &nl in &SUBNIVELES {
        if quedan == 0 {
            break;
        }
        let pone = capacidad(nl).min(quedan);
        ocupacion.insert(nl, pone);
        quedan -= pone;
    }
    for &(nl, e) in excepciones(&simbolo) {
        ocupacion.insert(nl, e);
    }
    if carga > 0 {
        for _ in 0..carga {
            let nl_opt = ocupacion
                .iter()
                .filter(|&(_, &e)| e > 0)
                .map(|(&k, _)| k)
                .max_by_key(|&(n, l)| (n, l));
            if let Some(nl) = nl_opt {
                if let Some(entry) = ocupacion.get_mut(&nl) {
                    *entry -= 1;
                }
            } else {
                break;
            }
        }
    } else if carga < 0 {
        let mut extra = u64::from(carga.unsigned_abs());
        for &nl in &SUBNIVELES {
            if extra == 0 {
                break;
            }
            let actual = ocupacion.get(&nl).copied().unwrap_or(0);
            let libre = u64::from(capacidad(nl).saturating_sub(actual));
            if libre > 0 {
                let pone = libre.min(extra) as u8;
                ocupacion.insert(nl, actual + pone);
                extra -= u64::from(pone);
            }
        }
    }
    let mut ordenada: Vec<((u8, u8), u8)> = SUBNIVELES
        .iter()
        .copied()
        .filter_map(|nl| {
            ocupacion
                .get(&nl)
                .copied()
                .filter(|&e| e > 0 || nl.1 == 0)
                .map(|e| (nl, e))
        })
        .collect();
    while ordenada.last().is_some_and(|&(_, e)| e == 0) {
        ordenada.pop();
    }
    Ok(Configuracion {
        simbolo,
        z,
        carga,
        ocupacion: ordenada,
    })
}

/// Regla de Hund: primero uno en cada caja (↑) y después se aparean (↑↓). 1 = ↑, 2 = ↑↓.
pub fn llenar_cajas(electrones: usize, cajas: usize) -> Vec<u8> {
    if cajas == 0 {
        return Vec::new();
    }
    let base = u8::try_from(electrones / cajas).unwrap_or(u8::MAX);
    let resto = electrones % cajas;
    (0..cajas)
        .map(|i| {
            if i < resto {
                base.saturating_add(1)
            } else {
                base
            }
        })
        .collect()
}

// --- Orbitales moleculares de diatómicas ---------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NivelOM {
    pub nombre: &'static str,
    pub enlazante: bool,
    pub degenerado: usize,
    pub electrones: usize,
    pub origen: &'static str,
}

impl NivelOM {
    fn new(nombre: &'static str, enlazante: bool, degenerado: usize, origen: &'static str) -> Self {
        Self {
            nombre,
            enlazante,
            degenerado,
            electrones: 0,
            origen,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiagramaOM {
    pub atomos: (String, String),
    pub carga: i32,
    pub niveles: Vec<NivelOM>,
    pub electrones: i32,
    pub mezcla_sp: bool,
    pub capa: u8,
}

impl DiagramaOM {
    pub fn orden_enlace(&self) -> f64 {
        let enl: usize = self
            .niveles
            .iter()
            .filter(|n| n.enlazante)
            .map(|n| n.electrones)
            .sum();
        let anti: usize = self
            .niveles
            .iter()
            .filter(|n| !n.enlazante)
            .map(|n| n.electrones)
            .sum();
        (enl as f64 - anti as f64) / 2.0
    }

    pub fn desapareados(&self) -> usize {
        self.niveles
            .iter()
            .map(|n| {
                llenar_cajas(n.electrones, n.degenerado)
                    .into_iter()
                    .filter(|&e| e == 1)
                    .count()
            })
            .sum()
    }

    pub fn magnetismo(&self) -> &'static str {
        if self.desapareados() > 0 {
            "paramagnética"
        } else {
            "diamagnética"
        }
    }
}

fn electrones_de_valencia(simbolo: &str) -> Result<i32, ErrorQuimica> {
    let z = numero_atomico_str(simbolo)?;
    if z <= 2 {
        Ok(i32::from(z))
    } else if z <= 10 {
        Ok(i32::from(z - 2))
    } else {
        Err(ErrorQuimica(format!(
            "orbitales moleculares: sólo diatómicas de los dos primeros períodos ({simbolo} no)"
        )))
    }
}

pub fn orbitales_moleculares(atomos: &Value, carga: i32) -> Result<DiagramaOM, ErrorQuimica> {
    let mut items: Vec<String> = lista(Some(atomos))
        .into_iter()
        .map(|v| val_a_texto_raw(v).trim().to_string())
        .collect();
    if items.len() == 1 {
        items.push(items[0].clone());
    }
    if items.len() != 2 {
        return Err(ErrorQuimica(
            "orbitales moleculares: 'atoms' son los dos átomos de la diatómica".to_string(),
        ));
    }
    let a = &items[0];
    let b = &items[1];
    let za = numero_atomico_str(a)?;
    let zb = numero_atomico_str(b)?;
    let capa = if za <= 2 && zb <= 2 { 1 } else { 2 };
    let (electrones, mezcla, mut niveles) = if capa == 1 {
        (
            i64::from(za) + i64::from(zb) - i64::from(carga),
            false,
            vec![
                NivelOM::new("σ1s", true, 1, "1s"),
                NivelOM::new("σ*1s", false, 1, "1s"),
            ],
        )
    } else {
        if za <= 2 || zb <= 2 {
            return Err(ErrorQuimica(
                "orbitales moleculares: las dos capas tienen que ser la misma (H2 o N2, no NH)"
                    .to_string(),
            ));
        }
        let electrones = i64::from(electrones_de_valencia(a)?)
            + i64::from(electrones_de_valencia(b)?)
            - i64::from(carga);
        let mezcla = electrones <= 10;
        let pi = NivelOM::new("π2p", true, 2, "2p");
        let sigma = NivelOM::new("σ2p", true, 1, "2p");
        let mut lv = vec![
            NivelOM::new("σ2s", true, 1, "2s"),
            NivelOM::new("σ*2s", false, 1, "2s"),
        ];
        if mezcla {
            lv.push(pi);
            lv.push(sigma);
        } else {
            lv.push(sigma);
            lv.push(pi);
        }
        lv.push(NivelOM::new("π*2p", false, 2, "2p"));
        lv.push(NivelOM::new("σ*2p", false, 1, "2p"));
        (electrones, mezcla, lv)
    };
    let capacidad_total: i64 = niveles.iter().map(|n| 2 * n.degenerado as i64).sum();
    if !(0..=capacidad_total).contains(&electrones) {
        return Err(ErrorQuimica(format!(
            "orbitales moleculares: {electrones} electrones no entran en los niveles"
        )));
    }
    let electrones_i32 = electrones as i32;
    let mut quedan = electrones as usize;
    for n in &mut niveles {
        let cap = 2 * n.degenerado;
        n.electrones = quedan.min(cap);
        quedan -= n.electrones;
    }
    Ok(DiagramaOM {
        atomos: (
            SIMBOLOS[(za - 1) as usize].to_string(),
            SIMBOLOS[(zb - 1) as usize].to_string(),
        ),
        carga,
        niveles,
        electrones: electrones_i32,
        mezcla_sp: mezcla,
        capa,
    })
}

// --- Ácido-base: titulaciones y fracciones de especies ---------------------------------------------------

pub fn fracciones(h: f64, kas: &[f64]) -> Vec<f64> {
    let n = kas.len();
    let mut terminos = Vec::with_capacity(n + 1);
    let mut producto = 1.0_f64;
    for j in 0..=n {
        if j > 0 {
            producto *= kas[j - 1];
        }
        terminos.push(h.powi((n - j) as i32) * producto);
    }
    let total: f64 = terminos.iter().sum();
    if total == 0.0 || !total.is_finite() {
        return vec![0.0; n + 1];
    }
    terminos.into_iter().map(|t| t / total).collect()
}

#[derive(Debug, Clone, PartialEq)]
pub struct Analito {
    pub tipo: String,
    pub conc: f64,
    pub volumen: f64,
    pub pkas: Vec<f64>,
    pub carga: i32,
}

impl Analito {
    pub fn es_acido(&self) -> bool {
        self.tipo == "strong_acid" || self.tipo == "weak_acid"
    }

    pub fn equivalentes(&self) -> usize {
        if self.tipo.starts_with("weak") {
            self.pkas.len()
        } else {
            1
        }
    }

    pub fn z0(&self) -> i32 {
        if self.tipo == "weak_acid" {
            self.carga
        } else {
            self.carga
                .saturating_add(i32::try_from(self.pkas.len()).unwrap_or(i32::MAX))
        }
    }

    fn z0_i64(&self) -> i64 {
        if self.tipo == "weak_acid" {
            i64::from(self.carga)
        } else {
            i64::from(self.carga) + self.pkas.len() as i64
        }
    }
}

pub fn analito(dato: Option<&Value>) -> Result<Analito, ErrorQuimica> {
    let Some(Value::Object(map)) = dato else {
        return Err(ErrorQuimica(
            "titration: 'analyte' es {kind, conc, volume, pKa}".to_string(),
        ));
    };
    let tipo = match map.get("kind") {
        None | Some(Value::Null) => "weak_acid".to_string(),
        Some(v) => val_a_texto_raw(v),
    };
    if !matches!(
        tipo.as_str(),
        "strong_acid" | "strong_base" | "weak_acid" | "weak_base"
    ) {
        return Err(ErrorQuimica(format!(
            "analyte.kind: '{tipo}' (vale strong_acid, strong_base, weak_acid o weak_base)"
        )));
    }
    let def_conc = serde_json::json!(0.1);
    let def_vol = serde_json::json!(25.0);
    let conc = num(
        map.get("conc").filter(|v| !v.is_null()).unwrap_or(&def_conc),
        "analyte.conc",
        true,
        None,
    )?;
    let volumen = num(
        map.get("volume")
            .filter(|v| !v.is_null())
            .unwrap_or(&def_vol),
        "analyte.volume",
        true,
        None,
    )?;
    let mut pkas: Vec<f64> = Vec::new();
    if tipo.starts_with("weak") {
        if tipo == "weak_base" && map.contains_key("pKb") {
            for p in lista(map.get("pKb")) {
                pkas.push(14.0 - num(p, "analyte.pKb", false, None)?);
            }
        } else {
            for p in lista(map.get("pKa")) {
                pkas.push(num(p, "analyte.pKa", false, None)?);
            }
        }
        if pkas.is_empty() {
            return Err(ErrorQuimica(format!(
                "analyte: un {tipo} necesita pKa (o pKb)"
            )));
        }
        pkas.sort_by(|a, b| a.total_cmp(b));
    }
    let carga = match map.get("charge") {
        None | Some(Value::Null) => 0,
        Some(v) => entero_val(v, "analyte.charge")?,
    };
    Ok(Analito {
        tipo,
        conc,
        volumen,
        pkas,
        carga,
    })
}

#[derive(Debug, Clone, PartialEq)]
pub struct Titulacion {
    pub analito: Analito,
    pub titulante: f64,
    pub vmax: f64,
    pub volumenes: Vec<f64>,
    pub phs: Vec<f64>,
    pub equivalencias: Vec<(f64, f64)>,
    pub semi: Vec<(f64, f64, f64)>,
}

impl Titulacion {
    pub fn ph_inicial(&self) -> f64 {
        self.phs.first().copied().unwrap_or(7.0)
    }
}

pub fn ph_de(a: &Analito, ct: f64, vt: f64) -> f64 {
    let v = a.volumen + vt;
    let c_sist = a.conc * a.volumen / v;
    let c_tit = ct * vt / v;
    let kas: Vec<f64> = a.pkas.iter().map(|&p| 10_f64.powf(-p)).collect();
    let mut fijo = if a.es_acido() { c_tit } else { -c_tit };
    if a.tipo == "strong_acid" {
        fijo -= c_sist;
    } else if a.tipo == "strong_base" {
        fijo += c_sist;
    } else {
        let agregada = if a.tipo == "weak_base" {
            i64::from(a.carga)
        } else {
            a.z0_i64()
        };
        fijo -= (agregada as f64) * c_sist;
    }

    let z0 = a.z0_i64();
    let balance = |log_h: f64| -> f64 {
        let h = 10_f64.powf(log_h);
        let mut carga = h - KW / h + fijo;
        if !kas.is_empty() {
            for (j, alfa) in fracciones(h, &kas).into_iter().enumerate() {
                carga += ((z0 - j as i64) as f64) * c_sist * alfa;
            }
        }
        carga
    };

    let mut lo = -15.5_f64;
    let mut hi = 1.5_f64;
    let mut flo = balance(lo);
    for _ in 0..200 {
        let medio = (lo + hi) / 2.0;
        let fm = balance(medio);
        if (fm > 0.0) == (flo > 0.0) {
            lo = medio;
            flo = fm;
        } else {
            hi = medio;
        }
        if hi - lo < 1e-10 {
            break;
        }
    }
    -(lo + hi) / 2.0
}

pub fn titulacion(
    dato_analito: Option<&Value>,
    conc_titulante: &Value,
    vmax: Option<&Value>,
    puntos: usize,
) -> Result<Titulacion, ErrorQuimica> {
    let a = analito(dato_analito)?;
    let ct = num(conc_titulante, "titrant.conc", true, None)?;
    let veq = a.conc * a.volumen / ct;
    let vmax_val = match vmax {
        Some(v) if !v.is_null() => num(v, "vMax", true, None)?,
        _ => veq * (a.equivalentes() as f64 + 0.6),
    };
    let n_pts = puntos.max(1);
    let volumenes: Vec<f64> = (0..=n_pts)
        .map(|i| vmax_val * i as f64 / n_pts as f64)
        .collect();
    let phs: Vec<f64> = volumenes.iter().map(|&v| ph_de(&a, ct, v)).collect();
    let mut equivalencias = Vec::new();
    for k in 1..=a.equivalentes() {
        let v = k as f64 * veq;
        if v > vmax_val {
            break;
        }
        let antes = ph_de(&a, ct, v * 0.97);
        let despues = ph_de(&a, ct, v * 1.03);
        if (despues - antes).abs() >= 1.0 {
            equivalencias.push((v, ph_de(&a, ct, v)));
        }
    }
    let mut semi = Vec::new();
    if a.tipo.starts_with("weak") {
        let pkas: Vec<f64> = if a.es_acido() {
            a.pkas.clone()
        } else {
            a.pkas.iter().copied().rev().collect()
        };
        for (k, pka) in pkas.into_iter().enumerate() {
            let v = (k as f64 + 0.5) * veq;
            if v <= vmax_val {
                semi.push((v, ph_de(&a, ct, v), pka));
            }
        }
    }
    Ok(Titulacion {
        analito: a,
        titulante: ct,
        vmax: vmax_val,
        volumenes,
        phs,
        equivalencias,
        semi,
    })
}

// --- Cinética ------------------------------------------------------------------------------------------

pub fn concentracion(orden: i32, k: f64, a0: f64, t: f64) -> Result<f64, ErrorQuimica> {
    match orden {
        0 => Ok((a0 - k * t).max(0.0)),
        1 => Ok(a0 * (-k * t).exp()),
        2 => Ok(a0 / (1.0 + a0 * k * t)),
        _ => Err(ErrorQuimica(format!(
            "kinetics: orden {orden} (vale 0, 1 o 2)"
        ))),
    }
}

pub fn vida_media(orden: i32, k: f64, a0: f64) -> Result<f64, ErrorQuimica> {
    match orden {
        0 => Ok(a0 / (2.0 * k)),
        1 => Ok(2.0_f64.ln() / k),
        2 => Ok(1.0 / (k * a0)),
        _ => Err(ErrorQuimica(format!(
            "kinetics: orden {orden} (vale 0, 1 o 2)"
        ))),
    }
}

/// `A → B → C`.
pub fn consecutiva(k1: f64, k2: f64, a0: f64, t: f64) -> (f64, f64, f64) {
    let a = a0 * (-k1 * t).exp();
    let b = if (k1 - k2).abs() < 1e-12 {
        a0 * k1 * t * (-k1 * t).exp()
    } else {
        a0 * k1 / (k2 - k1) * ((-k1 * t).exp() - (-k2 * t).exp())
    };
    (a, b, a0 - a - b)
}

pub fn t_maximo_intermediario(k1: f64, k2: f64) -> f64 {
    if (k1 - k2).abs() < 1e-12 {
        1.0 / k1
    } else {
        (k2 / k1).ln() / (k2 - k1)
    }
}

/// `A ⇌ B`, empezando con A puro.
pub fn reversible(kf: f64, kb: f64, a0: f64, t: f64) -> (f64, f64) {
    let s = kf + kb;
    let a = a0 * (kb + kf * (-s * t).exp()) / s;
    (a, a0 - a)
}

// --- Diagramas de fases ----------------------------------------------------------------------------------

#[derive(Debug, Clone, Copy)]
pub struct SustanciaFases {
    pub nombre: &'static str,
    pub triple: (f64, f64),
    pub critico: (f64, f64),
    pub pendiente_fusion: f64,
    pub normal_melting: Option<f64>,
    pub normal_boiling: Option<f64>,
    pub normal_sublimation: Option<f64>,
}

pub fn sustancia_fases(nombre: &str) -> Option<SustanciaFases> {
    match nombre {
        "water" => Some(SustanciaFases {
            nombre: "H2O",
            triple: (273.16, 0.00604),
            critico: (647.1, 217.7),
            pendiente_fusion: -134.0,
            normal_melting: Some(273.15),
            normal_boiling: Some(373.15),
            normal_sublimation: None,
        }),
        "co2" => Some(SustanciaFases {
            nombre: "CO2",
            triple: (216.55, 5.11),
            critico: (304.13, 72.8),
            pendiente_fusion: 40.0,
            normal_melting: None,
            normal_boiling: None,
            normal_sublimation: Some(194.65),
        }),
        _ => None,
    }
}

fn interpolar_cc(anclas: &[(f64, f64)], t: f64) -> f64 {
    if anclas.len() < 2 {
        return anclas.first().map_or(1.0, |a| a.1);
    }
    let mut ord = anclas.to_vec();
    ord.sort_by(|a, b| a.0.total_cmp(&b.0));
    let ultimo = ord[ord.len() - 1];
    for w in ord.windows(2) {
        let (t0, p0) = w[0];
        let (t1, p1) = w[1];
        if t <= t1 || (t1, p1) == ultimo {
            let b = (p1 / p0).ln() / (1.0 / t0 - 1.0 / t1);
            return (p0.ln() - b * (1.0 / t - 1.0 / t0)).exp();
        }
    }
    ultimo.1
}

fn t_de_p_cc(anclas: &[(f64, f64)], p: f64) -> f64 {
    if anclas.len() < 2 {
        return anclas.first().map_or(0.0, |a| a.0);
    }
    let mut ord = anclas.to_vec();
    ord.sort_by(|a, b| a.0.total_cmp(&b.0));
    let ultimo = ord[ord.len() - 1];
    for w in ord.windows(2) {
        let (t0, p0) = w[0];
        let (t1, p1) = w[1];
        if p <= p1 || (t1, p1) == ultimo {
            let b = (p1 / p0).ln() / (1.0 / t0 - 1.0 / t1);
            return 1.0 / (1.0 / t0 - (p / p0).ln() / b);
        }
    }
    ultimo.0
}

#[derive(Debug, Clone, PartialEq)]
pub struct Fases {
    pub nombre: String,
    pub triple: (f64, f64),
    pub critico: (f64, f64),
    pub pendiente_fusion: f64,
    pub anclas_vap: Vec<(f64, f64)>,
    pub anclas_sub: Vec<(f64, f64)>,
    pub normales: BTreeMap<String, f64>,
}

impl Fases {
    pub fn p_vapor(&self, t: f64) -> f64 {
        interpolar_cc(&self.anclas_vap, t)
    }

    pub fn p_sublimacion(&self, t: f64) -> f64 {
        interpolar_cc(&self.anclas_sub, t)
    }

    pub fn t_fusion(&self, p: f64) -> f64 {
        self.triple.0 + (p - self.triple.1) / self.pendiente_fusion
    }

    pub fn t_de_p(&self, p: f64) -> (&'static str, f64) {
        if p < self.triple.1 {
            ("sublimación", t_de_p_cc(&self.anclas_sub, p))
        } else {
            ("ebullición", t_de_p_cc(&self.anclas_vap, p))
        }
    }
}

fn par_nums(v: &Value) -> Option<(f64, f64)> {
    let arr = v.as_array()?;
    if arr.len() != 2 {
        return None;
    }
    let a = num(&arr[0], "par[0]", false, None).ok()?;
    let b = num(&arr[1], "par[1]", false, None).ok()?;
    Some((a, b))
}

pub fn fases(dato: &Bloque) -> Result<Fases, ErrorQuimica> {
    let sub_opt = dato.get("substance").filter(|v| !v.is_null());
    let base = match sub_opt {
        Some(v) if es_truthy(v) => {
            let clave = val_a_texto_raw(v).to_lowercase();
            let s = sustancia_fases(&clave).ok_or_else(|| {
                ErrorQuimica(format!(
                    "phase_diagram: sustancia '{}' (valen water, co2; si no, triple y critical)",
                    val_a_texto_raw(v)
                ))
            })?;
            Some(s)
        }
        _ => None,
    };

    let triple = match dato.get("triple").filter(|v| !v.is_null()) {
        Some(v) => par_nums(v),
        None => base.map(|b| b.triple),
    }
    .ok_or_else(|| {
        ErrorQuimica(
            "phase_diagram: faltan triple: [T, P] y critical: [T, P] (o substance)".to_string(),
        )
    })?;

    let critico = match dato.get("critical").filter(|v| !v.is_null()) {
        Some(v) => par_nums(v),
        None => base.map(|b| b.critico),
    }
    .ok_or_else(|| {
        ErrorQuimica(
            "phase_diagram: faltan triple: [T, P] y critical: [T, P] (o substance)".to_string(),
        )
    })?;

    let (tt, pt) = triple;
    let (tc, pc) = critico;
    if !(0.0 < tt && tt < tc && 0.0 < pt && pt < pc) {
        return Err(ErrorQuimica(
            "phase_diagram: el punto crítico tiene que estar arriba y a la derecha del triple"
                .to_string(),
        ));
    }

    let pendiente = match dato.get("meltingSlope").filter(|v| !v.is_null()) {
        Some(v) => num(v, "meltingSlope", false, None)?,
        None => base.map_or(50.0, |b| b.pendiente_fusion),
    };
    if pendiente == 0.0 {
        return Err(ErrorQuimica(
            "phase_diagram: meltingSlope no puede ser 0".to_string(),
        ));
    }

    let mut normales: BTreeMap<String, f64> = BTreeMap::new();
    if let Some(b) = base {
        if let Some(v) = b.normal_melting {
            normales.insert("normalMelting".to_string(), v);
        }
        if let Some(v) = b.normal_boiling {
            normales.insert("normalBoiling".to_string(), v);
        }
        if let Some(v) = b.normal_sublimation {
            normales.insert("normalSublimation".to_string(), v);
        }
    }
    for k in ["normalMelting", "normalBoiling", "normalSublimation"] {
        if let Some(v) = dato.get(k).filter(|v| !v.is_null()) {
            normales.insert(k.to_string(), num(v, k, false, None)?);
        }
    }

    let mut vap = vec![(tt, pt), (tc, pc)];
    if let Some(&tb_norm) = normales.get("normalBoiling") {
        if pt < 1.0 && 1.0 < pc {
            vap.push((tb_norm, 1.0));
        }
    }
    let b_vap = (pc / pt).ln() / (1.0 / tt - 1.0 / tc);
    let sub = if let Some(&tsub) = normales.get("normalSublimation").filter(|_| pt > 1.0) {
        vec![(tsub, 1.0), (tt, pt)]
    } else {
        let ratio = match dato.get("sublimationRatio").filter(|v| !v.is_null()) {
            Some(v) => num(v, "sublimationRatio", false, None)?,
            None => 1.3,
        };
        let b_sub = b_vap * ratio;
        let t_bajo = tt * 0.8;
        vec![
            (t_bajo, pt * (-b_sub * (1.0 / t_bajo - 1.0 / tt)).exp()),
            (tt, pt),
        ]
    };

    let nombre = match dato.get("name").filter(|v| !v.is_null()) {
        Some(v) => val_a_texto_raw(v),
        None => base.map_or_else(String::new, |b| b.nombre.to_string()),
    };

    Ok(Fases {
        nombre,
        triple: (tt, pt),
        critico: (tc, pc),
        pendiente_fusion: pendiente,
        anclas_vap: vap,
        anclas_sub: sub,
        normales,
    })
}

// --- Curva de calentamiento -------------------------------------------------------------------------------

#[derive(Debug, Clone, Copy)]
pub struct DatosCalentamiento {
    pub nombre: &'static str,
    pub cp_solid: f64,
    pub cp_liquid: f64,
    pub cp_gas: f64,
    pub melting: f64,
    pub boiling: f64,
    pub h_fusion: f64,
    pub h_vaporization: f64,
}

pub fn sustancia_calentamiento(nombre: &str) -> Option<DatosCalentamiento> {
    if nombre.eq_ignore_ascii_case("water") {
        Some(DatosCalentamiento {
            nombre: "H2O",
            cp_solid: 2.09,
            cp_liquid: 4.18,
            cp_gas: 2.01,
            melting: 0.0,
            boiling: 100.0,
            h_fusion: 334.0,
            h_vaporization: 2257.0,
        })
    } else {
        None
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Tramo {
    pub fase: &'static str,
    pub q0: f64,
    pub q1: f64,
    pub t0: f64,
    pub t1: f64,
}

impl Tramo {
    pub fn q(&self) -> f64 {
        self.q1 - self.q0
    }
}

fn leer_campo_termico(
    dato: &Bloque,
    clave: &str,
    base_val: Option<f64>,
) -> Result<f64, ErrorQuimica> {
    match dato.get(clave).filter(|v| !v.is_null()) {
        Some(v) => num(v, clave, false, None).map_err(|_| {
            ErrorQuimica(
                "heating_curve: faltan datos (melting, boiling, cpSolid, cpLiquid, cpGas, hFusion, hVaporization) o substance".to_string(),
            )
        }),
        None => base_val.ok_or_else(|| {
            ErrorQuimica(
                "heating_curve: faltan datos (melting, boiling, cpSolid, cpLiquid, cpGas, hFusion, hVaporization) o substance".to_string(),
            )
        }),
    }
}

pub fn calentamiento(dato: &Bloque) -> Result<Vec<Tramo>, ErrorQuimica> {
    let sub_nombre = match dato.get("substance").filter(|v| !v.is_null()) {
        Some(v) => val_a_texto_raw(v).to_lowercase(),
        None => "water".to_string(),
    };
    let base = sustancia_calentamiento(&sub_nombre);

    let def_masa = serde_json::json!(100.0);
    let def_from = serde_json::json!(-20.0);
    let def_to = serde_json::json!(120.0);

    let masa = num(
        dato.get("mass").filter(|v| !v.is_null()).unwrap_or(&def_masa),
        "mass (g)",
        true,
        None,
    )?;
    let t0 = num(
        dato.get("from").filter(|v| !v.is_null()).unwrap_or(&def_from),
        "from (°C)",
        false,
        None,
    )?;
    let t1 = num(
        dato.get("to").filter(|v| !v.is_null()).unwrap_or(&def_to),
        "to (°C)",
        false,
        None,
    )?;

    let tm = leer_campo_termico(dato, "melting", base.map(|b| b.melting))?;
    let tb = leer_campo_termico(dato, "boiling", base.map(|b| b.boiling))?;
    let cps = leer_campo_termico(dato, "cpSolid", base.map(|b| b.cp_solid))?;
    let cpl = leer_campo_termico(dato, "cpLiquid", base.map(|b| b.cp_liquid))?;
    let cpg = leer_campo_termico(dato, "cpGas", base.map(|b| b.cp_gas))?;
    let hf = leer_campo_termico(dato, "hFusion", base.map(|b| b.h_fusion))?;
    let hv = leer_campo_termico(dato, "hVaporization", base.map(|b| b.h_vaporization))?;

    if !(t0 < t1 && tm < tb) {
        return Err(ErrorQuimica(
            "heating_curve: tiene que ser from < to y melting < boiling".to_string(),
        ));
    }

    let mut tramos = Vec::new();
    let mut q = 0.0_f64;
    let mut t = t0;

    let calentar = |tramos: &mut Vec<Tramo>,
                    q: &mut f64,
                    t: &mut f64,
                    fase: &'static str,
                    hasta: f64,
                    cp: f64| {
        if hasta > *t {
            let dq = masa * cp * (hasta - *t) / 1000.0;
            tramos.push(Tramo {
                fase,
                q0: *q,
                q1: *q + dq,
                t0: *t,
                t1: hasta,
            });
            *q += dq;
            *t = hasta;
        }
    };

    let cambio = |tramos: &mut Vec<Tramo>, q: &mut f64, t_actual: f64, fase: &'static str, h: f64| {
        let dq = masa * h / 1000.0;
        tramos.push(Tramo {
            fase,
            q0: *q,
            q1: *q + dq,
            t0: t_actual,
            t1: t_actual,
        });
        *q += dq;
    };

    if t0 <= tm {
        calentar(&mut tramos, &mut q, &mut t, "sólido", tm.min(t1), cps);
        if t1 > tm {
            cambio(&mut tramos, &mut q, t, "fusión", hf);
        }
    }
    if t0 <= tb && t1 > tm {
        calentar(&mut tramos, &mut q, &mut t, "líquido", tb.min(t1), cpl);
        if t1 > tb {
            cambio(&mut tramos, &mut q, t, "vaporización", hv);
        }
    }
    if t1 > tb {
        calentar(&mut tramos, &mut q, &mut t, "gas", t1, cpg);
    }
    Ok(tramos)
}

// --- Electroquímica -------------------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq)]
pub struct Semicelda {
    pub pareja: String,
    pub e0: f64,
    pub n: u32,
    pub electrodo: String,
    pub solucion: String,
    pub conc: Option<f64>,
    pub reduccion: String,
}

impl Semicelda {
    pub fn oxidada(&self) -> &str {
        self.pareja.split('/').next().unwrap_or("").trim()
    }

    pub fn reducida(&self) -> &str {
        self.pareja.split('/').next_back().unwrap_or("").trim()
    }

    pub fn oxidacion(&self) -> String {
        match self.reduccion.split_once('→') {
            Some((izq, der)) => format!("{} → {}", der.trim(), izq.trim()),
            None => format!(" → {}", self.reduccion.trim()),
        }
    }
}

pub fn corto(x: Option<f64>) -> String {
    match x {
        Some(v) => fmt_g_py(v).replace('.', ","),
        None => String::new(),
    }
}

pub fn semicelda(dato: &Value, donde: &str) -> Result<Semicelda, ErrorQuimica> {
    let Value::Object(map) = dato else {
        return Err(ErrorQuimica(format!(
            "{donde}: cada semicelda es {{couple: 'Zn^2+/Zn', E0: -0.76, n: 2}}"
        )));
    };
    let (Some(couple_val), Some(e0_val)) = (map.get("couple"), map.get("E0")) else {
        return Err(ErrorQuimica(format!(
            "{donde}: cada semicelda es {{couple: 'Zn^2+/Zn', E0: -0.76, n: 2}}"
        )));
    };
    let pareja = val_a_texto_raw(couple_val);
    let Some((ox, red)) = pareja.split_once('/') else {
        return Err(ErrorQuimica(format!(
            "{donde}.couple: 'oxidada/reducida', por ejemplo 'Cu^2+/Cu'"
        )));
    };
    let n_i32 = match map.get("n") {
        None | Some(Value::Null) => 2,
        Some(v) => entero_val(v, &format!("{donde}.n"))?,
    };
    if n_i32 < 1 {
        return Err(ErrorQuimica(format!(
            "{donde}.n: los electrones son al menos 1"
        )));
    }
    let n = n_i32 as u32;
    let ox_trim = ox.trim();
    let red_trim = red.trim();
    let reduccion = match map.get("reaction") {
        Some(v) if es_truthy(v) => val_a_texto_raw(v),
        _ => {
            let n_txt = if n > 1 {
                n.to_string()
            } else {
                String::new()
            };
            format!("{ox_trim} + {n_txt}e^- → {red_trim}")
        }
    };
    let conc = match map.get("conc") {
        Some(v) => Some(num(v, &format!("{donde}.conc"), true, None)?),
        None => None,
    };
    let electrodo = match map.get("electrode") {
        Some(v) if es_truthy(v) => val_a_texto_raw(v),
        _ => red_trim.to_string(),
    };
    let solucion = match map.get("solution") {
        Some(v) if es_truthy(v) => val_a_texto_raw(v),
        _ => {
            if conc.is_some() {
                format!("{ox_trim} {} M", corto(conc))
            } else {
                format!("{ox_trim} 1 M")
            }
        }
    };
    let e0 = num(e0_val, &format!("{donde}.E0"), false, None)?;
    Ok(Semicelda {
        pareja,
        e0,
        n,
        electrodo,
        solucion,
        conc,
        reduccion,
    })
}

#[derive(Debug, Clone, PartialEq)]
pub struct Pila {
    pub anodo: Semicelda,
    pub catodo: Semicelda,
    pub n: u32,
    pub e0: f64,
    pub e: Option<f64>,
}

impl Pila {
    pub fn notacion(&self) -> String {
        let lado = |s: &Semicelda| -> String {
            let c = corto(s.conc);
            format!(
                "{} ({} M)",
                s.oxidada(),
                if c.is_empty() { "1" } else { c.as_str() }
            )
        };
        format!(
            "{}(s) | {} || {} | {}(s)",
            self.anodo.electrodo,
            lado(&self.anodo),
            lado(&self.catodo),
            self.catodo.electrodo
        )
    }

    pub fn delta_g(&self) -> f64 {
        -(f64::from(self.n)) * F * self.e0 / 1000.0
    }
}

fn gcd_u64(mut a: u64, mut b: u64) -> u64 {
    while b != 0 {
        let r = a % b;
        a = b;
        b = r;
    }
    a.max(1)
}

pub fn pila(mut semiceldas: Vec<Semicelda>) -> Result<Pila, ErrorQuimica> {
    if semiceldas.len() != 2 {
        return Err(ErrorQuimica("galvanic_cell: dos semiceldas".to_string()));
    }
    semiceldas.sort_by(|a, b| a.e0.total_cmp(&b.e0));
    let mut it = semiceldas.into_iter();
    let anodo = it
        .next()
        .ok_or_else(|| ErrorQuimica("galvanic_cell: dos semiceldas".to_string()))?;
    let catodo = it
        .next()
        .ok_or_else(|| ErrorQuimica("galvanic_cell: dos semiceldas".to_string()))?;
    let g = gcd_u64(u64::from(anodo.n), u64::from(catodo.n));
    let n_u64 = u64::from(anodo.n)
        .checked_mul(u64::from(catodo.n))
        .map(|p| p / g)
        .ok_or_else(|| ErrorQuimica("galvanic_cell: n demasiado grande".to_string()))?;
    let n = u32::try_from(n_u64)
        .map_err(|_| ErrorQuimica("galvanic_cell: n demasiado grande".to_string()))?;
    let e0 = catodo.e0 - anodo.e0;
    let e = match (anodo.conc, catodo.conc) {
        (Some(ca), Some(cc)) => {
            let q = ca.powf(f64::from(n) / f64::from(anodo.n))
                / cc.powf(f64::from(n) / f64::from(catodo.n));
            Some(e0 - 0.05916 / f64::from(n) * q.log10())
        }
        _ => None,
    };
    Ok(Pila {
        anodo,
        catodo,
        n,
        e0,
        e,
    })
}

// --- Celdas unidad --------------------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq)]
pub struct Celda {
    pub red: &'static str,
    pub sitios: Vec<(f64, f64, f64, &'static str)>,
    pub z: BTreeMap<&'static str, f64>,
    pub coordinacion: &'static str,
    pub empaquetamiento: Option<f64>,
    pub relacion: &'static str,
}

impl Celda {
    pub fn total(&self) -> f64 {
        self.z.values().sum()
    }
}

pub fn fraccion(x: f64, y: f64, z: f64) -> f64 {
    let bordes = [x, y, z]
        .into_iter()
        .filter(|&v| v == 0.0 || v == 1.0)
        .count() as i32;
    1.0 / 2.0_f64.powi(bordes)
}

pub fn celda_unidad(red: &str) -> Result<Celda, ErrorQuimica> {
    let mut vertices = Vec::with_capacity(8);
    for &x in &[0.0, 1.0] {
        for &y in &[0.0, 1.0] {
            for &z in &[0.0, 1.0] {
                vertices.push((x, y, z));
            }
        }
    }
    let caras = [
        (0.5, 0.5, 0.0),
        (0.5, 0.5, 1.0),
        (0.5, 0.0, 0.5),
        (0.5, 1.0, 0.5),
        (0.0, 0.5, 0.5),
        (1.0, 0.5, 0.5),
    ];
    let mut aristas = Vec::with_capacity(12);
    for &a in &[0.0, 0.5, 1.0] {
        for &b in &[0.0, 0.5, 1.0] {
            for &c in &[0.0, 0.5, 1.0] {
                let n_medios = [a, b, c].into_iter().filter(|&v| v == 0.5).count();
                if n_medios == 1 {
                    aristas.push((a, b, c));
                }
            }
        }
    }
    let centro = [(0.5, 0.5, 0.5)];

    let (red_st, sitios, coord, emp, rel) = match red {
        "sc" => (
            "sc",
            vertices.iter().map(|&(x, y, z)| (x, y, z, "A")).collect(),
            "6",
            Some(PI / 6.0),
            "a = 2r",
        ),
        "bcc" => (
            "bcc",
            vertices
                .iter()
                .chain(centro.iter())
                .map(|&(x, y, z)| (x, y, z, "A"))
                .collect(),
            "8",
            Some(PI * 3.0_f64.sqrt() / 8.0),
            "a = 4r/√3",
        ),
        "fcc" => (
            "fcc",
            vertices
                .iter()
                .chain(caras.iter())
                .map(|&(x, y, z)| (x, y, z, "A"))
                .collect(),
            "12",
            Some(PI / (3.0 * 2.0_f64.sqrt())),
            "a = 2√2 r",
        ),
        "nacl" => (
            "nacl",
            vertices
                .iter()
                .chain(caras.iter())
                .map(|&(x, y, z)| (x, y, z, "A"))
                .chain(aristas.iter().chain(centro.iter()).map(|&(x, y, z)| (x, y, z, "B")))
                .collect(),
            "6:6",
            None,
            "a = 2(r₊ + r₋)",
        ),
        "cscl" => (
            "cscl",
            vertices
                .iter()
                .map(|&(x, y, z)| (x, y, z, "A"))
                .chain(centro.iter().map(|&(x, y, z)| (x, y, z, "B")))
                .collect(),
            "8:8",
            None,
            "a√3 = 2(r₊ + r₋)",
        ),
        _ => {
            return Err(ErrorQuimica(format!(
                "unit_cell: red '{red}' (valen sc, bcc, fcc, nacl, cscl)"
            )));
        }
    };

    let mut z: BTreeMap<&'static str, f64> = BTreeMap::new();
    for &(x, y, zz, especie) in &sitios {
        *z.entry(especie).or_insert(0.0) += fraccion(x, y, zz);
    }
    Ok(Celda {
        red: red_st,
        sitios,
        z,
        coordinacion: coord,
        empaquetamiento: emp,
        relacion: rel,
    })
}

/// g/cm³: `Z·M / (N_A·a³)`. Con dos especies, `masa_molar` es la de la unidad fórmula.
pub fn densidad(celda: &Celda, a_pm: f64, masa_molar: f64) -> f64 {
    let unidades = if celda.z.len() == 1 {
        celda.z.get("A").copied().unwrap_or(0.0)
    } else {
        celda.z.values().copied().reduce(f64::min).unwrap_or(0.0)
    };
    unidades * masa_molar / (NA * (a_pm * 1e-10).powi(3))
}

// --- RMN -----------------------------------------------------------------------------------------------

pub const MULTIPLICIDAD: [(&str, usize); 7] = [
    ("s", 0),
    ("d", 1),
    ("t", 2),
    ("q", 3),
    ("quint", 4),
    ("sext", 5),
    ("sept", 6),
];

pub fn multiplicidad(clave: &str) -> Option<usize> {
    MULTIPLICIDAD
        .iter()
        .find(|&&(k, _)| k == clave)
        .map(|&(_, v)| v)
}

pub fn pascal(n: usize) -> Vec<u64> {
    let mut fila = vec![1u64];
    for _ in 0..n {
        let mut sig = Vec::with_capacity(fila.len() + 1);
        for i in 0..=fila.len() {
            let izq = if i == 0 { 0 } else { fila[i - 1] };
            let der = if i == fila.len() { 0 } else { fila[i] };
            sig.push(izq.saturating_add(der));
        }
        fila = sig;
    }
    fila
}

pub fn multiplete(desplazamiento: f64, vecinos: usize, j_hz: f64, mhz: f64) -> Vec<(f64, f64)> {
    let intensidades = pascal(vecinos);
    let total: f64 = intensidades.iter().map(|&v| v as f64).sum();
    let paso = j_hz / mhz;
    let centro = (intensidades.len() - 1) as f64 / 2.0;
    intensidades
        .into_iter()
        .enumerate()
        .map(|(i, v)| (desplazamiento + (i as f64 - centro) * paso, v as f64 / total))
        .collect()
}

// --- Orgánica: prioridades CIP, R/S, Newman y silla ------------------------------------------------------

pub const GRUPOS_CIP: &[(&str, (u8, &[u8]))] = &[
    ("H", (1, &[])),
    ("D", (1, &[0])),
    ("CH3", (6, &[1, 1, 1])),
    ("C2H5", (6, &[6, 1, 1])),
    ("CH2CH3", (6, &[6, 1, 1])),
    ("Et", (6, &[6, 1, 1])),
    ("CH(CH3)2", (6, &[6, 6, 1])),
    ("iPr", (6, &[6, 6, 1])),
    ("C(CH3)3", (6, &[6, 6, 6])),
    ("tBu", (6, &[6, 6, 6])),
    ("C6H5", (6, &[6, 6, 6])),
    ("Ph", (6, &[6, 6, 6])),
    ("CH2OH", (6, &[8, 1, 1])),
    ("CHO", (6, &[8, 8, 1])),
    ("COOH", (6, &[8, 8, 8])),
    ("COCH3", (6, &[8, 8, 6])),
    ("CN", (6, &[7, 7, 7])),
    ("CH2NH2", (6, &[7, 1, 1])),
    ("CONH2", (6, &[8, 8, 7])),
    ("CH2Cl", (6, &[17, 1, 1])),
    ("CH2Br", (6, &[35, 1, 1])),
    ("CH2SH", (6, &[16, 1, 1])),
    ("CCl3", (6, &[17, 17, 17])),
    ("OH", (8, &[1])),
    ("OCH3", (8, &[6])),
    ("NH2", (7, &[1, 1])),
    ("NHCH3", (7, &[6, 1])),
    ("N(CH3)2", (7, &[6, 6])),
    ("NH3^+", (7, &[1, 1, 1])),
    ("NO2", (7, &[8, 8, 8])),
    ("SH", (16, &[1])),
    ("F", (9, &[])),
    ("Cl", (17, &[])),
    ("Br", (35, &[])),
    ("I", (53, &[])),
];

pub fn grupo_cip(nombre: &str) -> Option<(u8, &'static [u8])> {
    GRUPOS_CIP
        .iter()
        .find(|&&(k, _)| k == nombre)
        .map(|&(_, v)| v)
}

fn clave_cip(grupo: &str) -> Result<(u8, Vec<u8>), ErrorQuimica> {
    let Some((z, lista_z)) = grupo_cip(grupo.trim()) else {
        return Err(ErrorQuimica(format!(
            "no sé la prioridad CIP de '{grupo}': indicá priorities (de mayor a menor)"
        )));
    };
    let mut ord = lista_z.to_vec();
    ord.sort_by(|a, b| b.cmp(a));
    Ok((z, ord))
}

pub fn prioridades(grupos: &[&str]) -> Result<Vec<usize>, ErrorQuimica> {
    let mut claves = Vec::with_capacity(grupos.len());
    for &g in grupos {
        claves.push(clave_cip(g)?);
    }
    let unicas: BTreeSet<&(u8, Vec<u8>)> = claves.iter().collect();
    if unicas.len() != claves.len() {
        return Err(ErrorQuimica(format!(
            "dos grupos empatan en la primera esfera ({}): indicá priorities",
            grupos.join(", ")
        )));
    }
    let mut orden: Vec<usize> = (0..grupos.len()).collect();
    orden.sort_by(|&a, &b| claves[b].cmp(&claves[a]));
    let mut rango = vec![0usize; grupos.len()];
    for (puesto, i) in orden.into_iter().enumerate() {
        rango[i] = puesto + 1;
    }
    Ok(rango)
}

fn coord_fischer(lugar: &str) -> Option<(i32, i32)> {
    match lugar {
        "top" => Some((0, 1)),
        "right" => Some((1, 0)),
        "bottom" => Some((0, -1)),
        "left" => Some((-1, 0)),
        _ => None,
    }
}

pub fn configuracion_fischer(por_lugar: &[(&str, usize)]) -> Result<&'static str, ErrorQuimica> {
    if por_lugar.len() != 4 {
        return Err(ErrorQuimica(
            "Fischer: las prioridades son 1, 2, 3 y 4, una por lugar".to_string(),
        ));
    }
    let mut lugar_de: [&str; 4] = [""; 4];
    let mut coords: [(i32, i32); 4] = [(0, 0); 4];
    let mut vistos = [false; 4];
    let mut lugares_vistos: BTreeSet<&str> = BTreeSet::new();
    for &(sitio, prio) in por_lugar {
        let Some(c) = coord_fischer(sitio) else {
            return Err(ErrorQuimica(
                "Fischer: las prioridades son 1, 2, 3 y 4, una por lugar".to_string(),
            ));
        };
        if !(1..=4).contains(&prio) || !lugares_vistos.insert(sitio) || vistos[prio - 1] {
            return Err(ErrorQuimica(
                "Fischer: las prioridades son 1, 2, 3 y 4, una por lugar".to_string(),
            ));
        }
        vistos[prio - 1] = true;
        lugar_de[prio - 1] = sitio;
        coords[prio - 1] = c;
    }
    let (x1, y1) = coords[0];
    let (x2, y2) = coords[1];
    let (x3, y3) = coords[2];
    let giro = (x2 - x1) * (y3 - y1) - (y2 - y1) * (x3 - x1);
    let mut horario = giro < 0;
    if lugar_de[3] == "left" || lugar_de[3] == "right" {
        horario = !horario;
    }
    Ok(if horario { "R" } else { "S" })
}

pub fn conformacion_newman(diedro: f64) -> String {
    let d = ((diedro + 180.0).rem_euclid(360.0) - 180.0).abs();
    if d < 15.0 {
        "eclipsada (sinperiplanar)".to_string()
    } else if (d - 60.0).abs() < 15.0 {
        "alternada gauche".to_string()
    } else if (d - 120.0).abs() < 15.0 {
        "eclipsada (anticlinal)".to_string()
    } else if d > 165.0 {
        "alternada anti".to_string()
    } else {
        format!("diedro de {}°", fmt_g_py(d))
    }
}

pub const VALORES_A: &[(&str, f64)] = &[
    ("CH3", 7.3),
    ("C2H5", 7.5),
    ("CH2CH3", 7.5),
    ("Et", 7.5),
    ("CH(CH3)2", 9.2),
    ("iPr", 9.2),
    ("C(CH3)3", 21.0),
    ("tBu", 21.0),
    ("C6H5", 12.6),
    ("Ph", 12.6),
    ("OH", 3.9),
    ("OCH3", 2.5),
    ("NH2", 5.9),
    ("F", 1.0),
    ("Cl", 2.2),
    ("Br", 2.0),
    ("I", 1.8),
    ("CN", 0.8),
    ("COOH", 5.9),
    ("CHO", 3.0),
    ("H", 0.0),
];

pub fn valor_a(grupo: &str) -> Option<f64> {
    VALORES_A
        .iter()
        .find(|&&(k, _)| k == grupo)
        .map(|&(_, v)| v)
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SustituyenteSilla {
    pub position: u8,
    pub group: String,
    pub axial: bool,
    pub blank: bool,
}

pub fn axial_arriba(carbono: u8) -> bool {
    carbono % 2 == 1
}

pub fn cara(carbono: u8, axial: bool) -> &'static str {
    if axial == axial_arriba(carbono) {
        "arriba"
    } else {
        "abajo"
    }
}

pub fn costo_axial(sustituyentes: &[SustituyenteSilla]) -> Result<f64, ErrorQuimica> {
    let mut total = 0.0_f64;
    for s in sustituyentes {
        if s.axial {
            let val = valor_a(&s.group).ok_or_else(|| {
                ErrorQuimica(format!("chair: no hay valor A para '{}'", s.group))
            })?;
            total += val;
        }
    }
    Ok(total)
}

pub fn relacion_cis_trans(sustituyentes: &[SustituyenteSilla]) -> Option<&'static str> {
    let distintos: Vec<&SustituyenteSilla> =
        sustituyentes.iter().filter(|s| s.group != "H").collect();
    if distintos.len() != 2 {
        return None;
    }
    let a = distintos[0];
    let b = distintos[1];
    Some(
        if cara(a.position, a.axial) == cara(b.position, b.axial) {
            "cis"
        } else {
            "trans"
        },
    )
}

// --- Inorgánica: campo cristalino, Latimer y Frost, Born-Haber y complejos ---------------------------------

pub fn electrones_d(metal: &str, oxidacion: i32) -> Result<u8, ErrorQuimica> {
    let z = i64::from(numero_atomico_str(metal)?);
    for &(inicio, fin, base) in &[(21_i64, 30_i64, 18_i64), (39, 48, 36), (72, 80, 68)] {
        if inicio <= z && z <= fin {
            let n = z - base - i64::from(oxidacion);
            if !(0..=10).contains(&n) {
                return Err(ErrorQuimica(format!(
                    "{metal} con estado de oxidación {oxidacion:+} no queda con 0 a 10 electrones d"
                )));
            }
            return Ok(n as u8);
        }
    }
    if z == 57 {
        return Ok((1_i64 - i64::from(oxidacion) + 2).clamp(0, 255) as u8);
    }
    Err(ErrorQuimica(format!(
        "{metal} no es un metal de transición del bloque d"
    )))
}

pub const SERIE_ESPECTROQUIMICA: [&str; 20] = [
    "I^-", "Br^-", "S^2-", "SCN^-", "Cl^-", "NO3^-", "F^-", "OH^-", "C2O4^2-", "H2O", "NCS^-",
    "CH3CN", "NH3", "en", "bipy", "phen", "NO2^-", "PPh3", "CN^-", "CO",
];

pub const CAMPO_FUERTE_DESDE: usize = 12;

#[derive(Debug, Clone, Copy)]
pub struct NivelCampo {
    pub nombre: &'static str,
    pub orbitales: &'static [&'static str],
    pub energia: f64,
}

pub const NIVELES_OCTAHEDRAL: &[NivelCampo] = &[
    NivelCampo {
        nombre: "t2g",
        orbitales: &["dxy", "dxz", "dyz"],
        energia: -0.4,
    },
    NivelCampo {
        nombre: "eg",
        orbitales: &["dz²", "dx²−y²"],
        energia: 0.6,
    },
];

pub const NIVELES_TETRAHEDRAL: &[NivelCampo] = &[
    NivelCampo {
        nombre: "e",
        orbitales: &["dz²", "dx²−y²"],
        energia: -0.6,
    },
    NivelCampo {
        nombre: "t2",
        orbitales: &["dxy", "dxz", "dyz"],
        energia: 0.4,
    },
];

pub const NIVELES_SQUARE_PLANAR: &[NivelCampo] = &[
    NivelCampo {
        nombre: "",
        orbitales: &["dxz", "dyz"],
        energia: -0.514,
    },
    NivelCampo {
        nombre: "",
        orbitales: &["dz²"],
        energia: -0.428,
    },
    NivelCampo {
        nombre: "",
        orbitales: &["dxy"],
        energia: 0.228,
    },
    NivelCampo {
        nombre: "",
        orbitales: &["dx²−y²"],
        energia: 1.228,
    },
];

pub fn niveles_campo(geometria: &str) -> Option<&'static [NivelCampo]> {
    match geometria {
        "octahedral" => Some(NIVELES_OCTAHEDRAL),
        "tetrahedral" => Some(NIVELES_TETRAHEDRAL),
        "square_planar" => Some(NIVELES_SQUARE_PLANAR),
        _ => None,
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CampoCristalino {
    pub geometria: &'static str,
    pub electrones: u8,
    pub alto_espin: bool,
    pub llenado: Vec<Vec<u8>>,
}

impl CampoCristalino {
    pub fn desapareados(&self) -> usize {
        self.llenado
            .iter()
            .flat_map(|nivel| nivel.iter())
            .filter(|&&e| e == 1)
            .count()
    }

    pub fn momento(&self) -> f64 {
        let n = self.desapareados() as f64;
        (n * (n + 2.0)).sqrt()
    }

    pub fn eecc(&self) -> f64 {
        let Some(niveles) = niveles_campo(self.geometria) else {
            return 0.0;
        };
        self.llenado
            .iter()
            .zip(niveles.iter())
            .map(|(nivel, nc)| {
                let suma: u8 = nivel.iter().sum();
                f64::from(suma) * nc.energia
            })
            .sum()
    }

    pub fn pares(&self) -> usize {
        self.llenado
            .iter()
            .flat_map(|nivel| nivel.iter())
            .filter(|&&e| e == 2)
            .count()
    }
}

pub fn campo_cristalino(
    geometria: &str,
    electrones: i32,
    alto_espin: bool,
) -> Result<CampoCristalino, ErrorQuimica> {
    let (geom_st, niveles) = match geometria {
        "octahedral" => ("octahedral", NIVELES_OCTAHEDRAL),
        "tetrahedral" => ("tetrahedral", NIVELES_TETRAHEDRAL),
        "square_planar" => ("square_planar", NIVELES_SQUARE_PLANAR),
        _ => {
            return Err(ErrorQuimica(format!(
                "crystal_field: geometry '{geometria}' (vale octahedral, tetrahedral o square_planar)"
            )));
        }
    };
    if !(0..=10).contains(&electrones) {
        return Err(ErrorQuimica(
            "crystal_field: de 0 a 10 electrones d".to_string(),
        ));
    }
    let mut llenado: Vec<Vec<u8>> = niveles
        .iter()
        .map(|nc| vec![0u8; nc.orbitales.len()])
        .collect();
    let mut quedan = electrones as u8;
    if alto_espin {
        let mut orden = Vec::new();
        for (i, nc) in niveles.iter().enumerate() {
            for j in 0..nc.orbitales.len() {
                orden.push((i, j));
            }
        }
        for vuelta in [1u8, 2u8] {
            for &(i, j) in &orden {
                if quedan > 0 && llenado[i][j] < vuelta {
                    llenado[i][j] += 1;
                    quedan -= 1;
                }
            }
        }
    } else {
        for (i, _nc) in niveles.iter().enumerate() {
            for vuelta in [1u8, 2u8] {
                for slot in &mut llenado[i] {
                    if quedan > 0 && *slot < vuelta {
                        *slot += 1;
                        quedan -= 1;
                    }
                }
            }
        }
    }
    Ok(CampoCristalino {
        geometria: geom_st,
        electrones: electrones as u8,
        alto_espin,
        llenado,
    })
}

pub fn es_campo_fuerte(ligando: &str) -> Result<bool, ErrorQuimica> {
    let pos = SERIE_ESPECTROQUIMICA
        .iter()
        .position(|&l| l == ligando)
        .ok_or_else(|| {
            ErrorQuimica(format!(
                "el ligando '{ligando}' no está en la serie espectroquímica: indicá spin"
            ))
        })?;
    Ok(pos >= CAMPO_FUERTE_DESDE)
}

#[derive(Debug, Clone, PartialEq)]
pub struct Latimer {
    pub especies: Vec<String>,
    pub oxidacion: Vec<f64>,
    pub potenciales: Vec<f64>,
}

impl Latimer {
    pub fn electrones(&self, i: usize) -> f64 {
        self.oxidacion[i] - self.oxidacion[i + 1]
    }

    pub fn potencial(&self, mut i: usize, mut j: usize) -> f64 {
        if i > j {
            std::mem::swap(&mut i, &mut j);
        }
        if i == j || j >= self.especies.len() {
            return 0.0;
        }
        let n: f64 = (i..j).map(|k| self.electrones(k)).sum();
        if n == 0.0 {
            return 0.0;
        }
        let suma: f64 = (i..j).map(|k| self.electrones(k) * self.potenciales[k]).sum();
        suma / n
    }

    pub fn frost(&self) -> Vec<f64> {
        let ref_idx = self
            .oxidacion
            .iter()
            .position(|&o| o == 0.0)
            .unwrap_or_else(|| self.especies.len().saturating_sub(1));
        (0..self.especies.len())
            .map(|k| {
                if k == ref_idx {
                    0.0
                } else if k < ref_idx {
                    (k..ref_idx)
                        .map(|m| self.electrones(m) * self.potenciales[m])
                        .sum()
                } else {
                    -(ref_idx..k)
                        .map(|m| self.electrones(m) * self.potenciales[m])
                        .sum::<f64>()
                }
            })
            .collect()
    }

    pub fn desproporcionan(&self) -> Vec<usize> {
        if self.especies.len() < 3 {
            return Vec::new();
        }
        (1..self.especies.len() - 1)
            .filter(|&k| self.potenciales[k] > self.potenciales[k - 1])
            .collect()
    }
}

pub fn latimer(dato: &Bloque) -> Result<Latimer, ErrorQuimica> {
    let especies: Vec<String> = lista(dato.get("species"))
        .into_iter()
        .map(val_a_texto_raw)
        .collect();
    let mut oxidacion = Vec::new();
    for (i, v) in lista(dato.get("oxidation")).into_iter().enumerate() {
        oxidacion.push(num(v, &format!("latimer.oxidation[{i}]"), false, None)?);
    }
    let mut potenciales = Vec::new();
    for (i, v) in lista(dato.get("potentials")).into_iter().enumerate() {
        potenciales.push(num(v, &format!("latimer.potentials[{i}]"), false, None)?);
    }
    if especies.len() < 2
        || oxidacion.len() != especies.len()
        || potenciales.len() != especies.len() - 1
    {
        return Err(ErrorQuimica(
            "latimer: species y oxidation (uno por especie) y potentials (uno entre cada par), de la más oxidada a la más reducida".to_string(),
        ));
    }
    if oxidacion.windows(2).any(|w| w[0] <= w[1]) {
        return Err(ErrorQuimica(
            "latimer: los estados de oxidación tienen que bajar de izquierda a derecha".to_string(),
        ));
    }
    Ok(Latimer {
        especies,
        oxidacion,
        potenciales,
    })
}

#[derive(Debug, Clone, PartialEq)]
pub struct BornHaber {
    pub pasos: Vec<(String, f64)>,
    pub formacion: f64,
    pub producto: String,
    pub inicio: String,
}

impl BornHaber {
    pub fn energia_red(&self) -> f64 {
        let suma: f64 = self.pasos.iter().map(|(_, dh)| dh).sum();
        self.formacion - suma
    }
}

pub fn born_haber(dato: &Bloque) -> Result<BornHaber, ErrorQuimica> {
    let mut pasos = Vec::new();
    for (i, p) in lista(dato.get("steps")).into_iter().enumerate() {
        let Value::Object(map) = p else {
            return Err(ErrorQuimica(format!(
                "born_haber: steps[{i}] es {{to: estado, dH: kJ/mol, label}}"
            )));
        };
        let Some(dh_val) = map.get("dH") else {
            return Err(ErrorQuimica(format!(
                "born_haber: steps[{i}] es {{to: estado, dH: kJ/mol, label}}"
            )));
        };
        let to = match map.get("to").filter(|v| !v.is_null()) {
            Some(v) => val_a_texto_raw(v),
            None => String::new(),
        };
        let dh = num(dh_val, &format!("steps[{i}].dH"), false, None)?;
        pasos.push((to, dh));
    }
    let Some(formacion_val) = dato.get("formation") else {
        return Err(ErrorQuimica(
            "born_haber: faltan steps (de los elementos a los iones gaseosos) y formation (ΔHf)"
                .to_string(),
        ));
    };
    if pasos.is_empty() {
        return Err(ErrorQuimica(
            "born_haber: faltan steps (de los elementos a los iones gaseosos) y formation (ΔHf)"
                .to_string(),
        ));
    }
    let formacion = num(formacion_val, "formation", false, None)?;
    let producto = match dato.get("product").filter(|v| !v.is_null()) {
        Some(v) => val_a_texto_raw(v),
        None => "MX(s)".to_string(),
    };
    let inicio = match dato.get("start").filter(|v| !v.is_null()) {
        Some(v) => val_a_texto_raw(v),
        None => "elementos".to_string(),
    };
    Ok(BornHaber {
        pasos,
        formacion,
        producto,
        inicio,
    })
}

pub const CARGA_LIGANDOS: &[(&str, i32)] = &[
    ("NH3", 0),
    ("H2O", 0),
    ("CO", 0),
    ("en", 0),
    ("py", 0),
    ("PPh3", 0),
    ("NO", 0),
    ("Cl", -1),
    ("Cl^-", -1),
    ("Br", -1),
    ("Br^-", -1),
    ("I", -1),
    ("I^-", -1),
    ("F", -1),
    ("F^-", -1),
    ("CN", -1),
    ("CN^-", -1),
    ("OH", -1),
    ("OH^-", -1),
    ("NO2", -1),
    ("NO2^-", -1),
    ("SCN", -1),
    ("SCN^-", -1),
    ("NCS^-", -1),
    ("ox", -2),
    ("C2O4^2-", -2),
    ("O^2-", -2),
];

pub fn carga_ligando(lig: &str) -> Option<i32> {
    CARGA_LIGANDOS
        .iter()
        .find(|&&(k, _)| k == lig)
        .map(|&(_, c)| c)
}

pub const SITIOS_COMPLEJO: &[(&str, usize)] = &[
    ("octahedral", 6),
    ("square_planar", 4),
    ("tetrahedral", 4),
];

pub fn sitios_complejo(geometria: &str) -> Option<usize> {
    SITIOS_COMPLEJO
        .iter()
        .find(|&&(k, _)| k == geometria)
        .map(|&(_, s)| s)
}

pub fn opuestos(geometria: &str) -> &'static [(usize, usize)] {
    match geometria {
        "octahedral" => &[(0, 1), (2, 3), (4, 5)],
        "square_planar" => &[(0, 1), (2, 3)],
        _ => &[],
    }
}

pub fn isomero(geometria: &str, ligandos: &[&str]) -> Option<&'static str> {
    let ops = opuestos(geometria);
    let mut conteo: BTreeMap<&str, Vec<usize>> = BTreeMap::new();
    for (i, &lig) in ligandos.iter().enumerate() {
        conteo.entry(lig).or_default().push(i);
    }
    let mut tamanos: Vec<usize> = conteo.values().map(|v| v.len()).collect();
    tamanos.sort_unstable();
    if (geometria == "octahedral" || geometria == "square_planar")
        && tamanos.contains(&2)
        && conteo.len() == 2
    {
        if let Some(par) = conteo.values().find(|v| v.len() == 2) {
            let mut p = [par[0], par[1]];
            p.sort_unstable();
            return Some(if ops.contains(&(p[0], p[1])) {
                "trans"
            } else {
                "cis"
            });
        }
    }
    if geometria == "octahedral" && tamanos == [3, 3] {
        if let Some(tres) = conteo.values().next() {
            let mut alguno_opuesto = false;
            for &a in tres {
                for &b in tres {
                    if a < b && ops.contains(&(a, b)) {
                        alguno_opuesto = true;
                    }
                }
            }
            return Some(if alguno_opuesto { "mer" } else { "fac" });
        }
    }
    None
}

pub fn oxidacion_metal(carga: i32, ligandos: &[&str]) -> Result<i32, ErrorQuimica> {
    let mut total: i64 = 0;
    for &lig in ligandos {
        let c = carga_ligando(lig).ok_or_else(|| {
            ErrorQuimica(format!(
                "complex: no sé la carga del ligando '{lig}': indicá oxidation"
            ))
        })?;
        total += i64::from(c);
    }
    i32::try_from(i64::from(carga) - total).map_err(|_| {
        ErrorQuimica(format!(
            "complex: estado de oxidación fuera de rango para carga {carga}"
        ))
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn approx(a: f64, b: f64, eps: f64) {
        assert!(
            (a - b).abs() <= eps,
            "esperado {b} (±{eps}), obtenido {a} (dif={})",
            (a - b).abs()
        );
    }

    #[test]
    fn tablas_completas() {
        assert_eq!(SIMBOLOS.len(), 118);
        assert_eq!(SIMBOLOS[0], "H");
        assert_eq!(SIMBOLOS[117], "Og");
        assert_eq!(GRUPOS_CIP.len(), 35);
        assert_eq!(VALORES_A.len(), 21);
        assert_eq!(SERIE_ESPECTROQUIMICA.len(), 20);
        assert_eq!(CARGA_LIGANDOS.len(), 27);
        assert_eq!(ELECTRONEGATIVIDAD.len(), 18);
        assert_eq!(SUBNIVELES.len(), 22);
        let cap_total: u16 = SUBNIVELES.iter().map(|&nl| u16::from(capacidad(nl))).sum();
        assert_eq!(cap_total, 156);
    }

    #[test]
    fn test_configuraciones() {
        let casos = [
            ("O", 0, "[He] 2s² 2p⁴", 2usize),
            ("Fe", 0, "[Ar] 4s² 3d⁶", 4),
            ("Fe", 2, "[Ar] 3d⁶", 4),
            ("Cr", 0, "[Ar] 4s¹ 3d⁵", 6),
            ("Cu", 0, "[Ar] 4s¹ 3d¹⁰", 1),
            ("Cu", 2, "[Ar] 3d⁹", 1),
            ("Cl", -1, "[Ne] 3s² 3p⁶", 0),
            ("Na", 0, "[Ne] 3s¹", 1),
        ];
        for (el, carga, nucleo, des) in casos {
            let conf = configuracion(&json!(el), carga).unwrap();
            assert_eq!(conf.texto(true), nucleo);
            assert_eq!(conf.desapareados(), des);
        }
        // Fe2+ conserva 4s⁰ en ocupacion antes de 3d⁶ (para dibujar cajas vacías)
        let fe2 = configuracion(&json!("Fe"), 2).unwrap();
        assert_eq!(fe2.ocupacion_de((4, 0)), 0);
        assert!(fe2.ocupacion.iter().any(|&(nl, e)| nl == (4, 0) && e == 0));
        // Ca2+ elimina 4s⁰ porque queda al final
        let ca2 = configuracion(&json!("Ca"), 2).unwrap();
        assert!(!ca2.ocupacion.iter().any(|&(nl, _)| nl == (4, 0)));
    }

    #[test]
    fn test_configuracion_completa_y_por_numero_atomico() {
        assert_eq!(
            configuracion(&json!(8), 0).unwrap().texto(false),
            "1s² 2s² 2p⁴"
        );
        let err = configuracion(&json!("Xx"), 0).unwrap_err();
        assert!(err.0.contains("desconocido"), "err={}", err.0);
        let err_rango = configuracion(&json!(119), 0).unwrap_err();
        assert!(err_rango.0.contains("fuera de rango"));
        let err_bool = configuracion(&json!(true), 0).unwrap_err();
        assert!(err_bool.0.contains("desconocido"));
        let err_sin_e = configuracion(&json!("H"), 2).unwrap_err();
        assert!(err_sin_e.0.contains("no tiene electrones"));
    }

    #[test]
    fn test_orbitales_moleculares() {
        let casos: Vec<(Value, i32, f64, &str)> = vec![
            (json!("H"), 0, 1.0, "diamagnética"),
            (json!("He"), 0, 0.0, "diamagnética"),
            (json!("N"), 0, 3.0, "diamagnética"),
            (json!("O"), 0, 2.0, "paramagnética"),
            (json!("O"), -1, 1.5, "paramagnética"),
            (json!("O"), 2, 3.0, "diamagnética"),
            (json!("F"), 0, 1.0, "diamagnética"),
            (json!("C"), 0, 2.0, "diamagnética"),
            (json!("B"), 0, 1.0, "paramagnética"),
            (json!(["C", "O"]), 0, 3.0, "diamagnética"),
            (json!(["N", "O"]), 0, 2.5, "paramagnética"),
        ];
        for (atomos, carga, orden, mag) in casos {
            let om = orbitales_moleculares(&atomos, carga).unwrap();
            approx(om.orden_enlace(), orden, 1e-12);
            assert_eq!(om.magnetismo(), mag);
        }
        // O2^2+ tiene 10 e- de valencia -> mezcla_sp = true
        let o2_2plus = orbitales_moleculares(&json!("O"), 2).unwrap();
        assert!(o2_2plus.mezcla_sp);
    }

    #[test]
    fn test_mezcla_sp_cambia_el_orden_de_los_niveles() {
        let om_n = orbitales_moleculares(&json!("N"), 0).unwrap();
        let nombres_n: Vec<&str> = om_n.niveles.iter().map(|n| n.nombre).collect();
        assert!(
            nombres_n.iter().position(|&x| x == "π2p") < nombres_n.iter().position(|&x| x == "σ2p")
        );
        let om_o = orbitales_moleculares(&json!("O"), 0).unwrap();
        let nombres_o: Vec<&str> = om_o.niveles.iter().map(|n| n.nombre).collect();
        assert!(
            nombres_o.iter().position(|&x| x == "σ2p") < nombres_o.iter().position(|&x| x == "π2p")
        );
    }

    #[test]
    fn test_orbitales_moleculares_errores() {
        let e1 = orbitales_moleculares(&json!(["Na", "Na"]), 0).unwrap_err();
        assert!(e1.0.contains("dos primeros períodos"));
        let e2 = orbitales_moleculares(&json!(["N", "H"]), 0).unwrap_err();
        assert!(e2.0.contains("misma"));
        let e3 = orbitales_moleculares(&json!(["H", "H", "H"]), 0).unwrap_err();
        assert!(e3.0.contains("'atoms'"));
        let e4 = orbitales_moleculares(&json!("He"), -2).unwrap_err();
        assert!(e4.0.contains("no entran"));
    }

    #[test]
    fn test_titulaciones() {
        let round2 = |x: f64| (x * 100.0).round() / 100.0;
        type CasoTit = (Value, f64, Vec<(f64, f64)>, Vec<(f64, f64)>);
        let casos: Vec<CasoTit> = vec![
            (
                json!({"kind": "weak_acid", "pKa": 4.76, "conc": 0.1, "volume": 25}),
                2.88,
                vec![(25.0, 8.73)],
                vec![(12.5, 4.76)],
            ),
            (
                json!({"kind": "strong_acid", "conc": 0.1, "volume": 25}),
                1.0,
                vec![(25.0, 7.0)],
                vec![],
            ),
            (
                json!({"kind": "weak_base", "pKb": 4.75, "conc": 0.1, "volume": 25}),
                11.12,
                vec![(25.0, 5.28)],
                vec![(12.5, 9.25)],
            ),
            (
                json!({"kind": "weak_acid", "pKa": [2.15, 7.2, 12.35], "conc": 0.1, "volume": 25}),
                1.63,
                vec![(25.0, 4.70), (50.0, 9.66)],
                vec![(12.5, 2.29), (37.5, 7.2), (62.5, 11.86)],
            ),
            (
                json!({"kind": "weak_base", "pKa": [6.35, 10.33], "charge": -2, "conc": 0.1, "volume": 25}),
                11.65,
                vec![(25.0, 8.34), (50.0, 3.91)],
                vec![(12.5, 10.32), (37.5, 6.35)],
            ),
        ];
        let ct = json!(0.1);
        for (a, inicial, eq_esp, semi_esp) in casos {
            let t = titulacion(Some(&a), &ct, None, 400).unwrap();
            approx(t.ph_inicial(), inicial, 0.01);
            let eq_got: Vec<(f64, f64)> = t
                .equivalencias
                .iter()
                .map(|&(v, ph)| (round2(v), round2(ph)))
                .collect();
            assert_eq!(eq_got, eq_esp);
            assert_eq!(t.semi.len(), semi_esp.len());
            for ((v, ph, _), (ve, phe)) in t.semi.iter().zip(semi_esp.iter()) {
                approx(*v, *ve, 1e-6);
                approx(*ph, *phe, 0.02);
            }
        }
    }

    #[test]
    fn test_titulacion_errores() {
        let e1 = analito(Some(&json!({"kind": "weak_acid", "conc": 0.1}))).unwrap_err();
        assert!(e1.0.contains("pKa"));
        let e2 = analito(Some(&json!({"kind": "sal"}))).unwrap_err();
        assert!(e2.0.contains("analyte.kind"));
        let e3 = analito(None).unwrap_err();
        assert!(e3.0.contains("'analyte'"));
    }

    #[test]
    fn test_fracciones_suman_uno_y_se_cruzan_en_el_pka() {
        let kas = [
            10_f64.powf(-2.15),
            10_f64.powf(-7.2),
            10_f64.powf(-12.35),
        ];
        let alfas = fracciones(10_f64.powf(-7.2), &kas);
        let suma: f64 = alfas.iter().sum();
        approx(suma, 1.0, 1e-12);
        approx(alfas[1], alfas[2], 1e-3 * alfas[1].abs());
    }

    #[test]
    fn test_vida_media_y_concentracion() {
        for (orden, k, a0, t_medio) in [
            (0, 0.05, 1.0, 10.0),
            (1, 0.1, 1.0, 2.0_f64.ln() / 0.1),
            (2, 0.2, 1.0, 5.0),
        ] {
            approx(vida_media(orden, k, a0).unwrap(), t_medio, 1e-12);
            approx(
                concentracion(orden, k, a0, t_medio).unwrap(),
                a0 / 2.0,
                1e-12,
            );
        }
        assert!(concentracion(3, 0.1, 1.0, 1.0)
            .unwrap_err()
            .0
            .contains("orden 3"));
        assert!(vida_media(3, 0.1, 1.0).is_err());
    }

    #[test]
    fn test_consecutivas_y_reversibles() {
        let tmax = t_maximo_intermediario(0.5, 0.2);
        approx(tmax, (0.2_f64 / 0.5).ln() / (0.2 - 0.5), 1e-12);
        let (a, b, c) = consecutiva(0.5, 0.2, 1.0, tmax);
        approx(a + b + c, 1.0, 1e-12);
        // k1 == k2 no divide por cero
        let (a_eq, b_eq, c_eq) = consecutiva(0.5, 0.5, 1.0, t_maximo_intermediario(0.5, 0.5));
        approx(a_eq + b_eq + c_eq, 1.0, 1e-12);
        let (ar, br) = reversible(0.3, 0.1, 1.0, 1e3);
        approx(br / ar, 3.0, 1e-9);
    }

    #[test]
    fn test_diagrama_de_fases_pasa_por_los_puntos_normales() {
        let b_agua: Bloque = serde_json::from_value(json!({"substance": "water"})).unwrap();
        let agua = fases(&b_agua).unwrap();
        let (tipo_a, ta) = agua.t_de_p(1.0);
        assert_eq!(tipo_a, "ebullición");
        approx(ta, 373.15, 1e-6);
        approx(agua.p_vapor(647.1), 217.7, 217.7 * 1e-6);

        let b_co2: Bloque = serde_json::from_value(json!({"substance": "co2"})).unwrap();
        let co2 = fases(&b_co2).unwrap();
        let (tipo_c, tc) = co2.t_de_p(1.0);
        assert_eq!(tipo_c, "sublimación");
        approx(tc, 194.65, 1e-6);

        let b_bad: Bloque = serde_json::from_value(json!({"substance": "mercurio"})).unwrap();
        assert!(fases(&b_bad).unwrap_err().0.contains("sustancia"));

        let b_crit: Bloque =
            serde_json::from_value(json!({"triple": [300, 1], "critical": [200, 50]})).unwrap();
        assert!(fases(&b_crit).unwrap_err().0.contains("crítico"));
    }

    #[test]
    fn test_curva_de_calentamiento_del_agua() {
        let b1: Bloque = serde_json::from_value(
            json!({"substance": "water", "mass": 100, "from": -20, "to": 120}),
        )
        .unwrap();
        let tramos = calentamiento(&b1).unwrap();
        let nombres: Vec<&str> = tramos.iter().map(|t| t.fase).collect();
        assert_eq!(
            nombres,
            vec!["sólido", "fusión", "líquido", "vaporización", "gas"]
        );
        let qs: Vec<f64> = tramos
            .iter()
            .map(|t| (t.q() * 100.0).round() / 100.0)
            .collect();
        assert_eq!(qs, vec![4.18, 33.4, 41.8, 225.7, 4.02]);

        let b2: Bloque =
            serde_json::from_value(json!({"substance": "water", "from": 20, "to": 80})).unwrap();
        let solo_liq = calentamiento(&b2).unwrap();
        let nombres2: Vec<&str> = solo_liq.iter().map(|t| t.fase).collect();
        assert_eq!(nombres2, vec!["líquido"]);

        let b_err: Bloque = serde_json::from_value(json!({"from": 50, "to": 10})).unwrap();
        assert!(calentamiento(&b_err).unwrap_err().0.contains("from < to"));
    }

    #[test]
    fn test_pila_de_daniell_con_nernst() {
        let s1 = semicelda(&json!({"couple": "Cu^2+/Cu", "E0": 0.34, "conc": 1}), "a").unwrap();
        let s2 =
            semicelda(&json!({"couple": "Zn^2+/Zn", "E0": -0.76, "conc": 0.01}), "b").unwrap();
        let p = pila(vec![s1, s2]).unwrap();
        assert_eq!(p.anodo.electrodo, "Zn");
        approx(p.e0, 1.10, 1e-12);
        approx(p.e.unwrap(), 1.10 + 0.05916, 1e-4);
        approx(p.delta_g(), -2.0 * F * 1.10 / 1000.0, 1e-9);
        assert_eq!(p.anodo.oxidacion(), "Zn → Zn^2+ + 2e^-");
        assert_eq!(
            p.notacion(),
            "Zn(s) | Zn^2+ (0,01 M) || Cu^2+ (1 M) | Cu(s)"
        );
    }

    #[test]
    fn test_pila_con_distinto_numero_de_electrones() {
        let s1 =
            semicelda(&json!({"couple": "Ag^+/Ag", "E0": 0.80, "n": 1, "conc": 0.1}), "a").unwrap();
        let s2 =
            semicelda(&json!({"couple": "Cu^2+/Cu", "E0": 0.34, "n": 2, "conc": 1}), "b").unwrap();
        let p = pila(vec![s1, s2]).unwrap();
        assert_eq!(p.n, 2);
        approx(p.e0, 0.46, 1e-12);
        approx(
            p.e.unwrap(),
            0.46 - 0.05916 / 2.0 * (1.0 / 0.1_f64.powi(2)).log10(),
            1e-9,
        );
        let err = semicelda(&json!({"couple": "Zn", "E0": 1}), "halfCells[0]").unwrap_err();
        assert!(err.0.contains("oxidada/reducida"));
    }

    #[test]
    fn test_celdas_unidad_y_densidades() {
        let c_sc = celda_unidad("sc").unwrap();
        assert_eq!(c_sc.z.get("A"), Some(&1.0));
        assert_eq!(c_sc.coordinacion, "6");
        approx(100.0 * c_sc.empaquetamiento.unwrap(), 52.4, 0.1);

        let c_bcc = celda_unidad("bcc").unwrap();
        assert_eq!(c_bcc.z.get("A"), Some(&2.0));
        assert_eq!(c_bcc.coordinacion, "8");
        approx(100.0 * c_bcc.empaquetamiento.unwrap(), 68.0, 0.1);

        let c_fcc = celda_unidad("fcc").unwrap();
        assert_eq!(c_fcc.z.get("A"), Some(&4.0));
        assert_eq!(c_fcc.coordinacion, "12");
        approx(100.0 * c_fcc.empaquetamiento.unwrap(), 74.0, 0.1);

        let c_nacl = celda_unidad("nacl").unwrap();
        assert_eq!(c_nacl.z.get("A"), Some(&4.0));
        assert_eq!(c_nacl.z.get("B"), Some(&4.0));
        assert_eq!(c_nacl.coordinacion, "6:6");
        assert!(c_nacl.empaquetamiento.is_none());

        let c_cscl = celda_unidad("cscl").unwrap();
        assert_eq!(c_cscl.z.get("A"), Some(&1.0));
        assert_eq!(c_cscl.z.get("B"), Some(&1.0));
        assert_eq!(c_cscl.coordinacion, "8:8");
        assert!(c_cscl.empaquetamiento.is_none());

        assert!(celda_unidad("hcp").unwrap_err().0.contains("red"));

        approx(densidad(&c_fcc, 361.5, 63.55), 8.93, 0.01);
        approx(densidad(&c_nacl, 564.0, 58.44), 2.16, 0.01);
    }

    #[test]
    fn test_multiplete_n_mas_uno() {
        let lineas = multiplete(1.2, 2, 7.0, 400.0);
        let ints: Vec<f64> = lineas
            .iter()
            .map(|&(_, i)| (i * 1000.0).round() / 1000.0)
            .collect();
        assert_eq!(ints, vec![0.25, 0.5, 0.25]);
        approx(lineas[1].0, 1.2, 1e-12);
        approx(lineas[2].0 - lineas[1].0, 7.0 / 400.0, 1e-12);
    }

    #[test]
    fn test_r_s_en_fischer_y_cip() {
        let casos = [
            (["CHO", "OH", "CH2OH", "H"], "R"),
            (["CHO", "H", "CH2OH", "OH"], "S"),
            (["COOH", "H", "CH3", "NH2"], "S"),
            (["COOH", "NH2", "CH3", "H"], "R"),
            (["H", "Cl", "CH3", "Br"], "R"),
        ];
        for (grupos, esperado) in casos {
            let rango = prioridades(&grupos).unwrap();
            let mapa = [
                ("top", rango[0]),
                ("right", rango[1]),
                ("bottom", rango[2]),
                ("left", rango[3]),
            ];
            assert_eq!(configuracion_fischer(&mapa).unwrap(), esperado);
        }
        // D > H en CIP
        assert_eq!(prioridades(&["D", "H"]).unwrap(), vec![1, 2]);

        let e_empate = prioridades(&["tBu", "Ph", "H", "OH"]).unwrap_err();
        assert!(e_empate.0.contains("empatan"));
        let e_desc = prioridades(&["CH2CH2CH2OH", "H", "OH", "Cl"]).unwrap_err();
        assert!(e_desc.0.contains("priorities"));
    }

    #[test]
    fn test_conformaciones_de_newman() {
        for (diedro, nombre) in [
            (180.0, "anti"),
            (60.0, "gauche"),
            (-60.0, "gauche"),
            (0.0, "sinperiplanar"),
            (120.0, "anticlinal"),
        ] {
            let got = conformacion_newman(diedro);
            assert!(got.contains(nombre), "diedro={diedro} -> {got}");
        }
        assert_eq!(conformacion_newman(30.0), "diedro de 30°");
    }

    #[test]
    fn test_silla_estabilidad_y_cis_trans() {
        let metilo = vec![SustituyenteSilla {
            position: 1,
            group: "CH3".to_string(),
            axial: true,
            blank: false,
        }];
        approx(costo_axial(&metilo).unwrap(), 7.3, 1e-12);

        let cis = vec![
            SustituyenteSilla {
                position: 1,
                group: "CH3".to_string(),
                axial: true,
                blank: false,
            },
            SustituyenteSilla {
                position: 2,
                group: "CH3".to_string(),
                axial: false,
                blank: false,
            },
        ];
        let trans = vec![
            SustituyenteSilla {
                position: 1,
                group: "CH3".to_string(),
                axial: false,
                blank: false,
            },
            SustituyenteSilla {
                position: 2,
                group: "CH3".to_string(),
                axial: false,
                blank: false,
            },
        ];
        assert_eq!(relacion_cis_trans(&cis), Some("cis"));
        assert_eq!(relacion_cis_trans(&trans), Some("trans"));

        let cis_14 = vec![
            SustituyenteSilla {
                position: 1,
                group: "tBu".to_string(),
                axial: false,
                blank: false,
            },
            SustituyenteSilla {
                position: 4,
                group: "CH3".to_string(),
                axial: true,
                blank: false,
            },
        ];
        assert_eq!(relacion_cis_trans(&cis_14), Some("cis"));
    }

    #[test]
    fn test_campo_cristalino_y_electrones_d() {
        let casos = [
            ("octahedral", 6, false, 0usize, Some(-2.4)),
            ("octahedral", 6, true, 4, Some(-0.4)),
            ("octahedral", 5, true, 5, Some(0.0)),
            ("octahedral", 3, true, 3, Some(-1.2)),
            ("octahedral", 9, true, 1, Some(-0.6)),
            ("tetrahedral", 7, true, 3, Some(-1.2)),
            ("square_planar", 8, false, 0, None),
        ];
        for (geom, de, alto, des, eecc) in casos {
            let cc = campo_cristalino(geom, de, alto).unwrap();
            assert_eq!(cc.desapareados(), des);
            approx(cc.momento(), ((des * (des + 2)) as f64).sqrt(), 1e-12);
            if let Some(esp) = eecc {
                approx(cc.eecc(), esp, 1e-9);
            }
        }
        for (metal, ox, de) in [
            ("Fe", 3, 5u8),
            ("Fe", 2, 6),
            ("Co", 3, 6),
            ("Cu", 2, 9),
            ("Pt", 2, 8),
            ("Ti", 4, 0),
            ("Zn", 2, 10),
            ("La", 3, 0),
        ] {
            assert_eq!(electrones_d(metal, ox).unwrap(), de);
        }
        assert!(electrones_d("Na", 1).unwrap_err().0.contains("transición"));
        assert!(es_campo_fuerte("CN^-").unwrap());
        assert!(es_campo_fuerte("NH3").unwrap());
        assert!(!es_campo_fuerte("H2O").unwrap());
        assert!(es_campo_fuerte("agua")
            .unwrap_err()
            .0
            .contains("serie espectroquímica"));
    }

    #[test]
    fn test_latimer_frost_y_born_haber() {
        let b_mn: Bloque = serde_json::from_value(json!({
            "species": ["MnO4^-", "MnO4^2-", "MnO2", "Mn^3+", "Mn^2+", "Mn"],
            "oxidation": [7, 6, 4, 3, 2, 0],
            "potentials": [0.56, 2.26, 0.95, 1.51, -1.18]
        }))
        .unwrap();
        let lt = latimer(&b_mn).unwrap();
        approx(lt.potencial(0, 2), 1.693, 1e-3);
        approx(lt.potencial(0, 4), 1.508, 1e-3);
        let frost_r: Vec<f64> = lt
            .frost()
            .into_iter()
            .map(|x| (x * 100.0).round() / 100.0)
            .collect();
        assert_eq!(frost_r, vec![5.18, 4.62, 0.1, -0.85, -2.36, 0.0]);
        let desp: Vec<&str> = lt
            .desproporcionan()
            .into_iter()
            .map(|k| lt.especies[k].as_str())
            .collect();
        assert_eq!(desp, vec!["MnO4^2-", "Mn^3+"]);

        let b_cu: Bloque = serde_json::from_value(json!({
            "species": ["Cu^2+", "Cu^+", "Cu"],
            "oxidation": [2, 1, 0],
            "potentials": [0.16, 0.52]
        }))
        .unwrap();
        let cobre = latimer(&b_cu).unwrap();
        assert_eq!(cobre.desproporcionan(), vec![1]);
        approx(cobre.potencial(0, 2), 0.34, 1e-12);

        // Latimer sin estado 0 toma la última especie como referencia 0.0
        let b_no0: Bloque = serde_json::from_value(json!({
            "species": ["A", "B"],
            "oxidation": [3, 1],
            "potentials": [0.5]
        }))
        .unwrap();
        let l_no0 = latimer(&b_no0).unwrap();
        assert_eq!(l_no0.frost(), vec![1.0, 0.0]);

        let b_lat_err: Bloque = serde_json::from_value(json!({
            "species": ["A", "B"],
            "oxidation": [0, 2],
            "potentials": [1]
        }))
        .unwrap();
        assert!(latimer(&b_lat_err).unwrap_err().0.contains("bajar"));

        let b_bh: Bloque = serde_json::from_value(json!({
            "steps": [{"dH": 107}, {"dH": 496}, {"dH": 122}, {"dH": -349}],
            "formation": -411
        }))
        .unwrap();
        let nacl = born_haber(&b_bh).unwrap();
        approx(nacl.energia_red(), -787.0, 1e-9);

        let b_bh_err: Bloque = serde_json::from_value(json!({"steps": []})).unwrap();
        assert!(born_haber(&b_bh_err).unwrap_err().0.contains("faltan steps"));
    }

    #[test]
    fn test_isomeros_y_oxidacion_complejo() {
        let casos: &[(&str, &[&str], Option<&str>)] = &[
            (
                "octahedral",
                &["NH3", "NH3", "Cl", "Cl", "NH3", "NH3"],
                Some("trans"),
            ),
            (
                "octahedral",
                &["Cl", "NH3", "Cl", "NH3", "NH3", "NH3"],
                Some("cis"),
            ),
            (
                "octahedral",
                &["Cl", "NH3", "Cl", "NH3", "Cl", "NH3"],
                Some("fac"),
            ),
            (
                "octahedral",
                &["Cl", "Cl", "Cl", "NH3", "NH3", "NH3"],
                Some("mer"),
            ),
            ("square_planar", &["Cl", "NH3", "Cl", "NH3"], Some("cis")),
            ("square_planar", &["Cl", "Cl", "NH3", "NH3"], Some("trans")),
            ("tetrahedral", &["Cl", "Cl", "Br", "Br"], None),
            (
                "octahedral",
                &["Cl", "Cl", "Br", "NH3", "NH3", "NH3"],
                None,
            ),
        ];
        for &(geom, ligs, esp) in casos {
            assert_eq!(isomero(geom, ligs), esp);
        }
        assert_eq!(
            oxidacion_metal(1, &["NH3", "NH3", "NH3", "NH3", "Cl", "Cl"]).unwrap(),
            3
        );
        assert!(oxidacion_metal(0, &["desconocido"])
            .unwrap_err()
            .0
            .contains("no sé la carga"));
    }

    #[test]
    fn test_fmt_g_py() {
        assert_eq!(fmt_g_py(0.0), "0");
        assert_eq!(fmt_g_py(-0.0), "0");
        assert_eq!(fmt_g_py(0.01), "0.01");
        assert_eq!(fmt_g_py(1.0), "1");
        assert_eq!(fmt_g_py(12.5), "12.5");
        assert_eq!(fmt_g_py(1e-5), "1e-05");
        assert_eq!(fmt_g_py(1.2e-5), "1.2e-05");
        assert_eq!(fmt_g_py(1e6), "1e+06");
        assert_eq!(fmt_g_py(999999.9), "1e+06");
        assert_eq!(corto(Some(0.01)), "0,01");
        assert_eq!(corto(None), "");
    }

    #[test]
    fn test_regresion_overflows_truncamientos_y_subniveles_og() {
        // Hallazgo 1 + 2: configuracion con cargas extremas (sin panic ni truncamiento u16)
        for carga in [i32::MIN, -65536, -65537] {
            for el in ["H", "Fe", "Og"] {
                let conf = configuracion(&json!(el), carga).unwrap();
                assert_eq!(conf.electrones(), 156);
                assert_eq!(conf.desapareados(), 0);
                assert!(conf.texto(false).ends_with("7f¹⁴"));
            }
        }
        for el in ["H", "Fe", "Og"] {
            let err = configuracion(&json!(el), i32::MAX).unwrap_err();
            assert!(err.0.contains("no tiene electrones"));
        }

        // Hallazgo 1: orbitales_moleculares con cargas extremas (sin panic)
        for carga in [i32::MIN, -65536, -65537, i32::MAX] {
            for atomos in [json!(["H", "H"]), json!(["N", "N"]), json!(["F", "Ne"])] {
                let err = orbitales_moleculares(&atomos, carga).unwrap_err();
                assert!(err.0.contains("no entran en los niveles"));
            }
        }

        // Hallazgo 1: electrones_d con oxidacion extrema (sin panic)
        for metal in ["Sc", "Fe", "Zn", "Au"] {
            assert!(electrones_d(metal, i32::MIN).is_err());
            assert!(electrones_d(metal, i32::MAX).is_err());
        }
        assert_eq!(electrones_d("La", i32::MIN).unwrap(), 255);
        assert_eq!(electrones_d("La", i32::MAX).unwrap(), 0);

        // Hallazgo 1: oxidacion_metal con carga extrema (sin panic)
        let err_ox = oxidacion_metal(i32::MAX, &["Cl", "Cl", "ox"]).unwrap_err();
        assert!(err_ox.0.contains("fuera de rango"));
        assert_eq!(oxidacion_metal(i32::MAX, &["NH3"]).unwrap(), i32::MAX);
        assert_eq!(oxidacion_metal(i32::MIN, &["Cl"]).unwrap(), i32::MIN + 1);

        // Hallazgo 1: llenar_cajas con electrones > 255 * cajas (sin panic)
        assert_eq!(llenar_cajas(1000, 3), vec![255, 255, 255]);
        assert_eq!(llenar_cajas(usize::MAX, 3), vec![255, 255, 255]);
        assert_eq!(llenar_cajas(1000, 0), Vec::<u8>::new());

        // Hallazgo 3: SUBNIVELES con 22 subniveles (aniones de Og usando 6f, 7d, 7f)
        let og1 = configuracion(&json!("Og"), -1).unwrap();
        assert_eq!(og1.electrones(), 119);
        assert_eq!(og1.desapareados(), 1);
        assert_eq!(og1.texto(true), "[Rn] 7s² 5f¹⁴ 6d¹⁰ 7p⁶ 6f¹");

        let og2 = configuracion(&json!("Og"), -2).unwrap();
        assert_eq!(og2.electrones(), 120);
        assert_eq!(og2.desapareados(), 2);
        assert_eq!(og2.texto(true), "[Rn] 7s² 5f¹⁴ 6d¹⁰ 7p⁶ 6f²");

        let og5 = configuracion(&json!("Og"), -5).unwrap();
        assert_eq!(og5.electrones(), 123);
        assert_eq!(og5.desapareados(), 5);
        assert_eq!(og5.texto(true), "[Rn] 7s² 5f¹⁴ 6d¹⁰ 7p⁶ 6f⁵");

        let og38 = configuracion(&json!("Og"), -38).unwrap();
        assert_eq!(og38.electrones(), 156);
        assert_eq!(og38.desapareados(), 0);
        assert_eq!(og38.texto(true), "[Rn] 7s² 5f¹⁴ 6d¹⁰ 7p⁶ 6f¹⁴ 7d¹⁰ 7f¹⁴");

        // Extra: titulación con charge extremo no paniquea
        let tit_max = titulacion(
            Some(&json!({"kind": "weak_base", "pKa": [4.75], "charge": i32::MAX})),
            &json!(0.1),
            None,
            10,
        )
        .unwrap();
        assert!(tit_max.ph_inicial().is_finite());
        let tit_min = titulacion(
            Some(&json!({"kind": "weak_acid", "pKa": [4.75], "charge": i32::MIN})),
            &json!(0.1),
            None,
            10,
        )
        .unwrap();
        assert!(tit_min.ph_inicial().is_finite());
    }

    #[test]
    fn test_regresion_formato_errores_comillas_simples() {
        let e_red = celda_unidad("hcp").unwrap_err().0;
        assert!(e_red.contains("red 'hcp'"), "{e_red}");
        assert!(!e_red.contains("\"hcp\""), "{e_red}");

        let e_num = num(&json!("abc"), "x", false, None).unwrap_err().0;
        assert!(e_num.contains("'abc' no es un número"), "{e_num}");

        let e_ent = entero_val(&json!("xyz"), "n").unwrap_err().0;
        assert!(e_ent.contains("'xyz' no es un entero"), "{e_ent}");

        let e_el = configuracion(&json!("Xx"), 0).unwrap_err().0;
        assert!(e_el.contains("elemento desconocido: 'Xx'"), "{e_el}");

        let e_an = analito(Some(&json!({"kind": "sal"}))).unwrap_err().0;
        assert!(e_an.contains("analyte.kind: 'sal'"), "{e_an}");

        let mut blq_fases = Bloque::new();
        blq_fases.insert("substance".to_string(), json!("mercurio"));
        let e_fas = fases(&blq_fases).unwrap_err().0;
        assert!(e_fas.contains("sustancia 'mercurio'"), "{e_fas}");

        let e_cip = prioridades(&["Desconocido", "H", "OH", "Cl"]).unwrap_err().0;
        assert!(e_cip.contains("prioridad CIP de 'Desconocido'"), "{e_cip}");

        let e_silla = costo_axial(&[SustituyenteSilla {
            position: 1,
            group: "NO2".to_string(),
            axial: true,
            blank: false,
        }])
        .unwrap_err()
        .0;
        assert!(e_silla.contains("valor A para 'NO2'"), "{e_silla}");

        let e_cf = campo_cristalino("hexagonal", 5, true).unwrap_err().0;
        assert!(e_cf.contains("geometry 'hexagonal'"), "{e_cf}");

        let e_lig = es_campo_fuerte("agua").unwrap_err().0;
        assert!(e_lig.contains("el ligando 'agua'"), "{e_lig}");

        let e_ox = oxidacion_metal(0, &["agua"]).unwrap_err().0;
        assert!(e_ox.contains("del ligando 'agua'"), "{e_ox}");
    }
}
