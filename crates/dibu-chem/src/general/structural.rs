//! Renderizadores estructurales de química general (M1b1):
//! `molecule`, `reaction`, `vsepr`, `orbitals`, `energy_profile`, `galvanic_cell`, `unit_cell`.

use crate::chem;
use crate::common::{
    ancho_formula, booleano, caja_vacia, cajas, carga_sup, coma, encabezado_q, entero,
    envolver_error, es_bool_true, flecha_doble, geometria, incrustar, num, opt_lista, opt_num,
    opt_texto, por_pares, rotulo_q, tq, GEOMETRIAS,
};
use dibu_core::{fmt_num_clean, valor_str, Bloque, ErrorDiagrama, Lienzo, Punto, Registro, SANS};
use dibu_plot::{fmt_g, GUIONES};
use serde_json::Value;
use std::collections::{BTreeMap, HashMap, HashSet};
use std::f64::consts::PI;
use std::fmt::Write as _;

const MAX_PUNTOS_LEWIS: i64 = 24;

pub fn registrar(reg: &mut Registro) {
    reg.registrar("molecule", render_molecule);
    reg.registrar("reaction", render_reaction);
    reg.registrar("vsepr", render_vsepr);
    reg.registrar("orbitals", render_orbitals);
    reg.registrar("energy_profile", render_energy_profile);
    reg.registrar("galvanic_cell", render_galvanic_cell);
    reg.registrar("unit_cell", render_unit_cell);
}

fn es_truthy_val(v: &Value) -> bool {
    match v {
        Value::Null => false,
        Value::Bool(b) => *b,
        Value::Number(n) => n.as_f64().is_some_and(|x| x != 0.0),
        Value::String(s) => !s.is_empty(),
        Value::Array(a) => !a.is_empty(),
        Value::Object(m) => !m.is_empty(),
    }
}

fn par_coords(val: &Value, donde: &str) -> Result<Punto, ErrorDiagrama> {
    let arr = val
        .as_array()
        .ok_or_else(|| ErrorDiagrama::new(format!("{donde}: tiene que ser [x, y]")))?;
    if arr.len() != 2 {
        return Err(ErrorDiagrama::new(format!(
            "{donde}: tiene que tener exactamente 2 coordenadas [x, y]"
        )));
    }
    let x = num(&arr[0], donde)?;
    let y = num(&arr[1], donde)?;
    Ok((x, y))
}

// --- 1. Molécula -----------------------------------------------------------------------------------------

struct Atomo {
    id: String,
    x: f64,
    y: f64,
    el: String,
    carga: Option<Value>,
    pares: usize,
    radicales: usize,
    blanco: bool,
    mostrar: bool,
    nota: String,
    vecinos: Vec<usize>,
}

impl Atomo {
    fn new(id: String, x: f64, y: f64) -> Self {
        Self {
            id,
            x,
            y,
            el: "C".to_string(),
            carga: None,
            pares: 0,
            radicales: 0,
            blanco: false,
            mostrar: false,
            nota: String::new(),
            vecinos: Vec::new(),
        }
    }
}

struct Enlace {
    a: usize,
    b: usize,
    orden: f64,
    estereo: Option<String>,
    dashed: bool,
    blank: bool,
}

struct Anillo {
    ids: Vec<usize>,
    cx: f64,
    cy: f64,
    n: usize,
    aromatico: bool,
}

fn carga_texto(carga: &Option<Value>) -> String {
    let Some(v) = carga else {
        return String::new();
    };
    match v {
        Value::Null | Value::Bool(_) => String::new(),
        Value::Number(num_val) => {
            let Some(f) = num_val.as_f64() else {
                return String::new();
            };
            if !f.is_finite() || f == 0.0 {
                return String::new();
            }
            let n = f as i64;
            if n == 0 {
                return String::new();
            }
            let signo = if n > 0 { "+" } else { "−" };
            if n.unsigned_abs() == 1 {
                signo.to_string()
            } else {
                format!("{}{signo}", n.unsigned_abs())
            }
        }
        Value::String(s) => {
            if s.is_empty() {
                String::new()
            } else {
                s.replace('-', "−")
            }
        }
        otro => {
            let s = valor_str(otro);
            if s.is_empty() || s == "0" {
                String::new()
            } else {
                s.replace('-', "−")
            }
        }
    }
}

struct Molecula {
    donde: String,
    atomos: Vec<Atomo>,
    indices: HashMap<String, usize>,
    alias: HashMap<String, String>,
    enlaces: Vec<Enlace>,
    anillos: Vec<Anillo>,
}

impl Molecula {
    fn from_bloque(b: &Bloque, donde: &str) -> Result<Self, ErrorDiagrama> {
        let mut m = Self {
            donde: donde.to_string(),
            atomos: Vec::new(),
            indices: HashMap::new(),
            alias: HashMap::new(),
            enlaces: Vec::new(),
            anillos: Vec::new(),
        };
        if let Some(rings) = opt_lista(b, "rings", &format!("{donde}.rings"))? {
            for (i, anillo) in rings.iter().enumerate() {
                m.agregar_anillo(anillo, i)?;
            }
        }
        if let Some(atoms) = opt_lista(b, "atoms", &format!("{donde}.atoms"))? {
            for (i, dato) in atoms.iter().enumerate() {
                m.agregar_atomo(dato, i)?;
            }
        }
        if let Some(bonds) = opt_lista(b, "bonds", &format!("{donde}.bonds"))? {
            for (i, dato) in bonds.iter().enumerate() {
                m.agregar_enlace(dato, &i.to_string())?;
            }
        }
        if m.atomos.is_empty() {
            return Err(ErrorDiagrama::new(format!(
                "{donde}: faltan atoms (o rings)"
            )));
        }
        for e in &m.enlaces {
            m.atomos[e.a].vecinos.push(e.b);
            m.atomos[e.b].vecinos.push(e.a);
        }
        Ok(m)
    }

    fn atomo_idx(&self, id: &str) -> Result<usize, ErrorDiagrama> {
        let clave = self.alias.get(id).map_or(id, String::as_str);
        self.indices
            .get(clave)
            .copied()
            .ok_or_else(|| ErrorDiagrama::new(format!("{}: no hay un átomo '{id}'", self.donde)))
    }

    fn nuevo(&mut self, id: &str, x: f64, y: f64) -> usize {
        for (idx, a) in self.atomos.iter().enumerate() {
            if (a.x - x).hypot(a.y - y) < 0.05 {
                self.alias.insert(id.to_string(), a.id.clone());
                return idx;
            }
        }
        let idx = self.atomos.len();
        self.atomos.push(Atomo::new(id.to_string(), x, y));
        self.indices.insert(id.to_string(), idx);
        idx
    }

    fn agregar_anillo(&mut self, dato: &Value, i: usize) -> Result<(), ErrorDiagrama> {
        let map = dato.as_object().ok_or_else(|| {
            ErrorDiagrama::new(format!("{}.rings[{i}]: {{size, center, ids}}", self.donde))
        })?;
        let n_i64 = match map.get("size") {
            None | Some(Value::Null) => 6,
            Some(v) => entero(v, &format!("{}.rings[{i}].size", self.donde))?,
        };
        if !(3..=8).contains(&n_i64) {
            return Err(ErrorDiagrama::new(format!(
                "{}.rings[{i}].size: de 3 a 8 átomos",
                self.donde
            )));
        }
        let n = n_i64 as usize;
        let (cx, cy) = match map.get("center") {
            None | Some(Value::Null) => (0.0, 0.0),
            Some(v) => par_coords(v, &format!("{}.rings[{i}].center", self.donde))?,
        };
        let inicio = opt_num(map, "start", &format!("{}.rings[{i}].start", self.donde))?
            .unwrap_or(90.0);
        let ids: Vec<String> = match map.get("ids") {
            None | Some(Value::Null) => (0..n).map(|k| format!("r{i}_{k}")).collect(),
            Some(Value::Array(arr)) if arr.is_empty() => {
                (0..n).map(|k| format!("r{i}_{k}")).collect()
            }
            Some(Value::Array(arr)) => arr.iter().map(valor_str).collect(),
            Some(_) => {
                return Err(ErrorDiagrama::new(format!(
                    "{}.rings[{i}].ids: tienen que ser {n}",
                    self.donde
                )));
            }
        };
        if ids.len() != n {
            return Err(ErrorDiagrama::new(format!(
                "{}.rings[{i}].ids: tienen que ser {n}",
                self.donde
            )));
        }
        let radio = 1.0 / (2.0 * (PI / n as f64).sin());
        let elementos = map.get("el").and_then(Value::as_array);
        let mut reales_idx = Vec::with_capacity(n);
        let mut reales_id = Vec::with_capacity(n);
        for (k, id) in ids.iter().enumerate() {
            let ang = (inicio + 360.0 * k as f64 / n as f64).to_radians();
            let vx = cx + radio * ang.cos();
            let vy = cy + radio * ang.sin();
            if !vx.is_finite() || !vy.is_finite() {
                return Err(ErrorDiagrama::new(format!(
                    "{}.rings[{i}]: coordenadas fuera de rango",
                    self.donde
                )));
            }
            let idx = self.nuevo(id, vx, vy);
            if let Some(el_arr) = elementos {
                if let Some(el_val) = el_arr.get(k) {
                    if es_truthy_val(el_val) {
                        self.atomos[idx].el = valor_str(el_val);
                    }
                }
            }
            reales_idx.push(idx);
            reales_id.push(self.atomos[idx].id.clone());
        }
        let kekule = booleano(map, "kekule", false);
        for k in 0..n {
            let mut bond_map = serde_json::Map::new();
            bond_map.insert("from".to_string(), Value::String(reales_id[k].clone()));
            bond_map.insert(
                "to".to_string(),
                Value::String(reales_id[(k + 1) % n].clone()),
            );
            bond_map.insert(
                "order".to_string(),
                Value::from(if kekule && k % 2 == 0 { 2 } else { 1 }),
            );
            self.agregar_enlace(&Value::Object(bond_map), &format!("rings[{i}]"))?;
        }
        let aromatico = booleano(map, "aromatic", false);
        self.anillos.push(Anillo {
            ids: reales_idx,
            cx,
            cy,
            n,
            aromatico,
        });
        Ok(())
    }

    fn agregar_atomo(&mut self, dato: &Value, i: usize) -> Result<(), ErrorDiagrama> {
        let owned_map;
        let map = match dato {
            Value::String(s) => {
                let mut m = serde_json::Map::new();
                m.insert("el".to_string(), Value::String(s.clone()));
                owned_map = m;
                &owned_map
            }
            Value::Object(m) => m,
            _ => {
                return Err(ErrorDiagrama::new(format!(
                    "{}.atoms[{i}]: {{id, el, at: [x, y]}} o {{id, el, from, angle}}",
                    self.donde
                )));
            }
        };
        let id = map
            .get("id")
            .filter(|v| !v.is_null())
            .map(valor_str)
            .unwrap_or_else(|| format!("a{i}"));
        let clave = self.alias.get(&id).map_or(id.as_str(), String::as_str);
        let existente = self.indices.contains_key(clave)
            && !map.contains_key("at")
            && !map.contains_key("from");

        let idx = if existente {
            self.atomo_idx(&id)?
        } else if let Some(at_val) = map.get("at") {
            let (x, y) = par_coords(at_val, &format!("{}.atoms[{i}].at", self.donde))?;
            self.nuevo(&id, x, y)
        } else if let Some(from_val) = map.get("from") {
            let from_id = valor_str(from_val);
            let base_idx = self.atomo_idx(&from_id)?;
            let base_x = self.atomos[base_idx].x;
            let base_y = self.atomos[base_idx].y;
            let base_id = self.atomos[base_idx].id.clone();
            let ang = opt_num(map, "angle", &format!("{}.atoms[{i}].angle", self.donde))?
                .unwrap_or(0.0)
                .to_radians();
            let largo = opt_num(map, "length", &format!("{}.atoms[{i}].length", self.donde))?
                .unwrap_or(1.0);
            let new_x = base_x + largo * ang.cos();
            let new_y = base_y + largo * ang.sin();
            if !new_x.is_finite() || !new_y.is_finite() {
                return Err(ErrorDiagrama::new(format!(
                    "{}.atoms[{i}]: coordenadas fuera de rango",
                    self.donde
                )));
            }
            let new_idx = self.nuevo(&id, new_x, new_y);
            let new_id = self.atomos[new_idx].id.clone();

            let crear_enlace = match map.get("bond") {
                None => true,
                Some(v) => es_truthy_val(v),
            };
            if crear_enlace {
                let mut bond_map = serde_json::Map::new();
                bond_map.insert("from".to_string(), Value::String(base_id));
                bond_map.insert("to".to_string(), Value::String(new_id));
                match map.get("bond") {
                    None => {
                        bond_map.insert("order".to_string(), Value::from(1));
                    }
                    Some(Value::Object(bm)) => {
                        for (k, v) in bm {
                            bond_map.insert(k.clone(), v.clone());
                        }
                    }
                    Some(v) => {
                        bond_map.insert("order".to_string(), v.clone());
                    }
                }
                self.agregar_enlace(&Value::Object(bond_map), &format!("atoms[{i}]"))?;
            }
            new_idx
        } else if self.atomos.is_empty() {
            self.nuevo(&id, 0.0, 0.0)
        } else {
            return Err(ErrorDiagrama::new(format!(
                "{}.atoms[{i}]: falta at: [x, y] o from: <id> (con angle)",
                self.donde
            )));
        };

        if let Some(el_val) = map.get("el").or_else(|| map.get("label")) {
            if !el_val.is_null() {
                self.atomos[idx].el = valor_str(el_val);
            }
        }
        if let Some(c_val) = map.get("charge") {
            self.atomos[idx].carga = Some(c_val.clone());
        }
        if let Some(lp_val) = map.get("lonePairs").filter(|v| !v.is_null()) {
            let lp = entero(lp_val, &format!("{}.atoms[{i}].lonePairs", self.donde))?;
            if !(0..=MAX_PUNTOS_LEWIS).contains(&lp) {
                return Err(ErrorDiagrama::new(format!(
                    "{}.atoms[{i}].lonePairs: tiene que estar entre 0 y {MAX_PUNTOS_LEWIS}",
                    self.donde
                )));
            }
            self.atomos[idx].pares = lp as usize;
        }
        if let Some(rad_val) = map.get("radical").filter(|v| !v.is_null()) {
            let rad = entero(rad_val, &format!("{}.atoms[{i}].radical", self.donde))?;
            if !(0..=MAX_PUNTOS_LEWIS).contains(&rad) {
                return Err(ErrorDiagrama::new(format!(
                    "{}.atoms[{i}].radical: tiene que estar entre 0 y {MAX_PUNTOS_LEWIS}",
                    self.donde
                )));
            }
            self.atomos[idx].radicales = rad as usize;
        }
        self.atomos[idx].blanco = booleano(map, "blank", self.atomos[idx].blanco);
        self.atomos[idx].mostrar = booleano(map, "show", self.atomos[idx].mostrar);
        if let Some(note_val) = map.get("note") {
            if es_truthy_val(note_val) {
                self.atomos[idx].nota = valor_str(note_val);
            } else {
                self.atomos[idx].nota.clear();
            }
        }
        Ok(())
    }

    fn agregar_enlace(&mut self, dato: &Value, donde_bond: &str) -> Result<(), ErrorDiagrama> {
        let owned_map;
        let map = match dato {
            Value::Array(arr) => {
                if arr.len() < 2 {
                    return Err(ErrorDiagrama::new(format!(
                        "{}.bonds[{donde_bond}]: [a, b] o [a, b, orden]",
                        self.donde
                    )));
                }
                let mut m = serde_json::Map::new();
                m.insert("from".to_string(), arr[0].clone());
                m.insert("to".to_string(), arr[1].clone());
                if let Some(ord) = arr.get(2) {
                    m.insert("order".to_string(), ord.clone());
                }
                owned_map = m;
                &owned_map
            }
            Value::Object(m) => m,
            _ => {
                return Err(ErrorDiagrama::new(format!(
                    "{}.bonds[{donde_bond}]: [a, b, orden] o {{from, to, order, stereo}}",
                    self.donde
                )));
            }
        };
        let a_id = map
            .get("from")
            .or_else(|| map.get("a"))
            .filter(|v| !v.is_null())
            .map(valor_str)
            .unwrap_or_else(|| "None".to_string());
        let b_id = map
            .get("to")
            .or_else(|| map.get("b"))
            .filter(|v| !v.is_null())
            .map(valor_str)
            .unwrap_or_else(|| "None".to_string());
        let a_idx = self.atomo_idx(&a_id)?;
        let b_idx = self.atomo_idx(&b_id)?;

        let orden = match map.get("order") {
            None | Some(Value::Null) => 1.0,
            Some(Value::Bool(true)) => 1.0,
            Some(v) => match num(v, &format!("{}.bonds[{donde_bond}].order", self.donde)) {
                Ok(x) => x,
                Err(_) => {
                    return Err(ErrorDiagrama::new(format!(
                        "{}.bonds[{donde_bond}].order: 1, 2, 3, 1.5 (resonancia) o 0.5 (parcial)",
                        self.donde
                    )));
                }
            },
        };
        if !matches!(orden, 0.5 | 1.0 | 1.5 | 2.0 | 3.0) {
            return Err(ErrorDiagrama::new(format!(
                "{}.bonds[{donde_bond}].order: 1, 2, 3, 1.5 (resonancia) o 0.5 (parcial)",
                self.donde
            )));
        }

        let estereo = match map.get("stereo") {
            None | Some(Value::Null) => None,
            Some(Value::String(s)) if s == "wedge" || s == "hash" => Some(s.clone()),
            Some(_) => {
                return Err(ErrorDiagrama::new(format!(
                    "{}.bonds[{donde_bond}].stereo: wedge (cuña) o hash (rayas)",
                    self.donde
                )));
            }
        };

        let nuevo = Enlace {
            a: a_idx,
            b: b_idx,
            orden,
            estereo,
            dashed: booleano(map, "dashed", false),
            blank: booleano(map, "blank", false),
        };
        for e in &mut self.enlaces {
            if (e.a == a_idx && e.b == b_idx) || (e.a == b_idx && e.b == a_idx) {
                *e = nuevo;
                return Ok(());
            }
        }
        self.enlaces.push(nuevo);
        Ok(())
    }
}

fn atomo_visible(a: &Atomo, esqueletal: bool) -> bool {
    !esqueletal
        || a.el != "C"
        || a.mostrar
        || a.blanco
        || !carga_texto(&a.carga).is_empty()
        || a.pares > 0
        || a.radicales > 0
        || a.vecinos.is_empty()
}

fn semiejes(a: &Atomo, esqueletal: bool, size: f64) -> Punto {
    if !atomo_visible(a, esqueletal) {
        return (0.0, 0.0);
    }
    if a.blanco {
        return (9.0, 9.0);
    }
    (ancho_formula(&a.el, size) / 2.0 + 1.5, size * 0.62)
}

fn salida(hw: f64, hh: f64, ux: f64, uy: f64) -> f64 {
    if hw == 0.0 || (ux.abs() <= 1e-9 && uy.abs() <= 1e-9) {
        return 0.0;
    }
    let tx = if ux.abs() > 1e-9 {
        hw / ux.abs()
    } else {
        f64::INFINITY
    };
    let ty = if uy.abs() > 1e-9 {
        hh / uy.abs()
    } else {
        f64::INFINITY
    };
    let m = tx.min(ty);
    if m.is_finite() {
        m + 2.0
    } else {
        0.0
    }
}

fn par_ordenado(a: usize, b: usize) -> (usize, usize) {
    if a <= b {
        (a, b)
    } else {
        (b, a)
    }
}

fn dibujar_molecula(c: &mut Lienzo, b: &Bloque, donde: &str) -> Result<Punto, ErrorDiagrama> {
    let m = Molecula::from_bloque(b, donde)?;
    let largo = opt_num(b, "bondLength", &format!("{donde}.bondLength"))?.unwrap_or(34.0);
    if !(largo > 0.0 && largo.is_finite()) {
        return Err(ErrorDiagrama::new(format!(
            "{donde}.bondLength: tiene que ser positivo"
        )));
    }
    let esqueletal = booleano(b, "skeletal", false);
    let size = opt_num(b, "fontSize", &format!("{donde}.fontSize"))?.unwrap_or(12.0);
    if !(size > 0.0 && size.is_finite()) {
        return Err(ErrorDiagrama::new(format!(
            "{donde}.fontSize: tiene que ser positivo"
        )));
    }
    let pos: Vec<Punto> = m.atomos.iter().map(|a| (a.x * largo, -a.y * largo)).collect();
    if pos.iter().any(|&(x, y)| !x.is_finite() || !y.is_finite()) {
        return Err(ErrorDiagrama::new(format!(
            "{donde}: coordenadas escaladas fuera de rango"
        )));
    }
    let mut sub = Lienzo::new(c.pal, c.sufijo.clone());
    let trazo = c.pal.stroke_main;

    let mut centros_anillo: HashMap<(usize, usize), Punto> = HashMap::new();
    for an in &m.anillos {
        let (acx, acy) = (an.cx * largo, -an.cy * largo);
        if !acx.is_finite() || !acy.is_finite() {
            return Err(ErrorDiagrama::new(format!(
                "{donde}: coordenadas de anillo fuera de rango"
            )));
        }
        for i in 0..an.ids.len() {
            let key = par_ordenado(an.ids[i], an.ids[(i + 1) % an.ids.len()]);
            centros_anillo.insert(key, (acx, acy));
        }
    }
    let mut puntos_extra: Vec<Punto> = Vec::new();

    for e in &m.enlaces {
        let a = &m.atomos[e.a];
        let z = &m.atomos[e.b];
        let (x1, y1) = pos[e.a];
        let (x2, y2) = pos[e.b];
        let dist = {
            let d = (x2 - x1).hypot(y2 - y1);
            if !d.is_finite() {
                return Err(ErrorDiagrama::new(format!(
                    "{donde}: distancia entre átomos fuera de rango"
                )));
            }
            if d == 0.0 {
                1.0
            } else {
                d
            }
        };
        let (ux, uy) = ((x2 - x1) / dist, (y2 - y1) / dist);
        let (hw1, hh1) = semiejes(a, esqueletal, size);
        let (hw2, hh2) = semiejes(z, esqueletal, size);
        if !hw1.is_finite() || !hh1.is_finite() || !hw2.is_finite() || !hh2.is_finite() {
            return Err(ErrorDiagrama::new(format!(
                "{donde}: tamaño de etiqueta fuera de rango"
            )));
        }
        let s1 = salida(hw1, hh1, ux, uy);
        let s2 = salida(hw2, hh2, ux, uy);
        let (ax, ay) = (x1 + ux * s1, y1 + uy * s1);
        let (bx, by) = (x2 - ux * s2, y2 - uy * s2);
        if !ax.is_finite()
            || !ay.is_finite()
            || !bx.is_finite()
            || !by.is_finite()
            || !(bx - ax).is_finite()
            || !(by - ay).is_finite()
        {
            return Err(ErrorDiagrama::new(format!(
                "{donde}: extremos de enlace fuera de rango"
            )));
        }
        let (px, py) = (-uy, ux);
        let guiones = if e.dashed || e.orden == 0.5 {
            r#" stroke-dasharray="4 3""#
        } else {
            ""
        };
        if e.blank {
            sub.add(format!(
                r#"<line x1="{ax:.1}" y1="{ay:.1}" x2="{bx:.1}" y2="{by:.1}" stroke="{}" stroke-width="1" stroke-dasharray="2 3"/>"#,
                c.pal.stroke_muted
            ));
            continue;
        }
        if e.estereo.as_deref() == Some("wedge") {
            sub.add(format!(
                r#"<polygon points="{ax:.1},{ay:.1} {:.1},{:.1} {:.1},{:.1}" fill="{trazo}" stroke="{trazo}" stroke-width="0.6"/>"#,
                bx + px * 3.6,
                by + py * 3.6,
                bx - px * 3.6,
                by - py * 3.6
            ));
            continue;
        }
        if e.estereo.as_deref() == Some("hash") {
            for k in 1..=7 {
                let t = k as f64 / 7.0;
                let (hx, hy) = (ax + (bx - ax) * t, ay + (by - ay) * t);
                let ancho = 0.6 + 3.2 * t;
                sub.add(format!(
                    r#"<line x1="{:.1}" y1="{:.1}" x2="{:.1}" y2="{:.1}" stroke="{trazo}" stroke-width="1.1"/>"#,
                    hx + px * ancho,
                    hy + py * ancho,
                    hx - px * ancho,
                    hy - py * ancho
                ));
            }
            continue;
        }

        let emitir_linea = |lienzo: &mut Lienzo, lx1: f64, ly1: f64, lx2: f64, ly2: f64, extra: &str| {
            lienzo.add(format!(
                r#"<line x1="{lx1:.1}" y1="{ly1:.1}" x2="{lx2:.1}" y2="{ly2:.1}" stroke="{trazo}" stroke-width="1.5" stroke-linecap="round"{extra}/>"#
            ));
        };

        let orden = e.orden;
        let centro = centros_anillo.get(&par_ordenado(e.a, e.b)).copied();
        if (orden == 2.0 || orden == 1.5) && centro.is_some() {
            let centro_pt = centro.unwrap_or((0.0, 0.0));
            emitir_linea(&mut sub, ax, ay, bx, by, "");
            let (mx, my) = (ax + (bx - ax) * 0.5, ay + (by - ay) * 0.5);
            let hacia = if (centro_pt.0 - mx) * px + (centro_pt.1 - my) * py > 0.0 {
                1.0
            } else {
                -1.0
            };
            let o = 5.0 * hacia;
            let recorte = 0.16;
            emitir_linea(
                &mut sub,
                ax + (bx - ax) * recorte + px * o,
                ay + (by - ay) * recorte + py * o,
                bx - (bx - ax) * recorte + px * o,
                by - (by - ay) * recorte + py * o,
                if orden == 1.5 {
                    r#" stroke-dasharray="3 2.5""#
                } else {
                    ""
                },
            );
        } else if orden == 2.0 || orden == 1.5 {
            emitir_linea(
                &mut sub,
                ax + px * 2.6,
                ay + py * 2.6,
                bx + px * 2.6,
                by + py * 2.6,
                "",
            );
            emitir_linea(
                &mut sub,
                ax - px * 2.6,
                ay - py * 2.6,
                bx - px * 2.6,
                by - py * 2.6,
                if orden == 1.5 {
                    r#" stroke-dasharray="3 2.5""#
                } else {
                    ""
                },
            );
        } else if orden == 3.0 {
            for o in [-4.0_f64, 0.0_f64, 4.0_f64] {
                emitir_linea(
                    &mut sub,
                    ax + px * o,
                    ay + py * o,
                    bx + px * o,
                    by + py * o,
                    "",
                );
            }
        } else {
            emitir_linea(&mut sub, ax, ay, bx, by, guiones);
        }
    }

    for an in &m.anillos {
        if an.aromatico {
            let interior = largo / (2.0 * (PI / an.n as f64).tan());
            let radio_aro = interior * 0.6;
            if !interior.is_finite() || !radio_aro.is_finite() {
                return Err(ErrorDiagrama::new(format!(
                    "{donde}: radio de anillo aromático fuera de rango"
                )));
            }
            sub.add(format!(
                r#"<circle cx="{:.1}" cy="{:.1}" r="{:.1}" fill="none" stroke="{trazo}" stroke-width="1.4"/>"#,
                an.cx * largo,
                -an.cy * largo,
                radio_aro
            ));
        }
    }

    for (idx, a) in m.atomos.iter().enumerate() {
        let (x, y) = pos[idx];
        if !atomo_visible(a, esqueletal) {
            continue;
        }
        let (hw, hh) = semiejes(a, esqueletal, size);
        if !hw.is_finite() || !hh.is_finite() {
            return Err(ErrorDiagrama::new(format!(
                "{donde}: tamaño de etiqueta fuera de rango"
            )));
        }
        if a.blanco {
            caja_vacia(&mut sub, x, y, 18.0, 18.0);
        } else {
            tq(
                &mut sub,
                x,
                y + size * 0.36,
                &a.el,
                size,
                500,
                "textMain",
                "middle",
                SANS,
                false,
            );
        }
        let carga = carga_texto(&a.carga);
        if !carga.is_empty() {
            let (cx_q, cy_q) = (x + hw + 1.5, y - hh + 1.0);
            let (px_q, py_q) = (x + hw + 10.0, y - hh - 6.0);
            if !cx_q.is_finite() || !cy_q.is_finite() || !px_q.is_finite() || !py_q.is_finite() {
                return Err(ErrorDiagrama::new(format!(
                    "{donde}: posición de carga fuera de rango"
                )));
            }
            sub.t(
                cx_q,
                cy_q,
                &carga,
                size * 0.8,
                700,
                "textMain",
                "start",
                SANS,
            );
            puntos_extra.push((px_q, py_q));
        }
        if !a.nota.is_empty() {
            let (ny, py_n) = (y - hh - 5.0, y - hh - 14.0);
            if !ny.is_finite() || !py_n.is_finite() {
                return Err(ErrorDiagrama::new(format!(
                    "{donde}: posición de nota fuera de rango"
                )));
            }
            sub.t(
                x,
                ny,
                &a.nota,
                size * 0.7,
                400,
                "textSub",
                "middle",
                SANS,
            );
            puntos_extra.push((x, py_n));
        }
        let mut enlaces_ang: Vec<f64> = a
            .vecinos
            .iter()
            .map(|&v| (pos[v].1 - y).atan2(pos[v].0 - x))
            .collect();
        if !carga.is_empty() {
            enlaces_ang.push((-hh).atan2(hw));
        }
        let mut elegidas: Vec<f64> = Vec::new();
        for _ in 0..(a.pares + a.radicales) {
            let mut mejor = 0.0_f64;
            let mut puntaje = -1.0_f64;
            for k in 0..24 {
                let ang = (k as f64 * 15.0).to_radians();
                let mut lejos = PI;
                let mut hay_ocupadas = false;
                for &o in enlaces_ang.iter().chain(elegidas.iter()) {
                    hay_ocupadas = true;
                    let diff = (ang - o).sin().atan2((ang - o).cos()).abs();
                    if diff < lejos {
                        lejos = diff;
                    }
                }
                if !hay_ocupadas {
                    lejos = PI;
                }
                let valor = lejos + if k % 6 == 0 { 0.05 } else { 0.0 };
                if valor > puntaje + 1e-9 {
                    mejor = ang;
                    puntaje = valor;
                }
            }
            elegidas.push(mejor);
        }
        for (i, ang) in elegidas.into_iter().enumerate() {
            let (ux, uy) = (ang.cos(), ang.sin());
            let r = salida(hw.max(5.0), hh.max(5.0), ux, uy) + 4.0;
            let (cx_, cy_) = (x + ux * r, y + uy * r);
            puntos_extra.push((cx_ + ux * 4.0, cy_ + uy * 4.0));
            if i < a.pares {
                for s in [-2.8_f64, 2.8_f64] {
                    sub.punto(cx_ - uy * s, cy_ + ux * s, "strokeMain", 1.4);
                }
            } else {
                sub.punto(cx_, cy_, "strokeMain", 1.6);
            }
        }
    }

    if let Some(arrows) = opt_lista(b, "arrows", &format!("{donde}.arrows"))? {
        let lugar = |ref_val: &Value, i: usize| -> Result<Punto, ErrorDiagrama> {
            if let Some(arr) = ref_val.as_array() {
                if arr.len() < 2 {
                    return Err(ErrorDiagrama::new(format!(
                        "{donde}.arrows[{i}]: {{from: átomo o [a, b], to: átomo o [a, b]}}"
                    )));
                }
                let p1 = pos[m.atomo_idx(&valor_str(&arr[0]))?];
                let p2 = pos[m.atomo_idx(&valor_str(&arr[1]))?];
                let (mx, my) = (p1.0 + (p2.0 - p1.0) * 0.5, p1.1 + (p2.1 - p1.1) * 0.5);
                if !mx.is_finite() || !my.is_finite() {
                    return Err(ErrorDiagrama::new(format!(
                        "{donde}.arrows[{i}]: punto medio fuera de rango"
                    )));
                }
                Ok((mx, my))
            } else {
                Ok(pos[m.atomo_idx(&valor_str(ref_val))?])
            }
        };
        for (i, flecha_val) in arrows.iter().enumerate() {
            let Some(f_map) = flecha_val.as_object() else {
                return Err(ErrorDiagrama::new(format!(
                    "{donde}.arrows[{i}]: {{from: átomo o [a, b], to: átomo o [a, b]}}"
                )));
            };
            let (Some(from_val), Some(to_val)) = (f_map.get("from"), f_map.get("to")) else {
                return Err(ErrorDiagrama::new(format!(
                    "{donde}.arrows[{i}]: {{from: átomo o [a, b], to: átomo o [a, b]}}"
                )));
            };
            let (mut sx, mut sy) = lugar(from_val, i)?;
            let (mut ex, mut ey) = lugar(to_val, i)?;
            let dist = {
                let d = (ex - sx).hypot(ey - sy);
                if !d.is_finite() {
                    return Err(ErrorDiagrama::new(format!(
                        "{donde}.arrows[{i}]: longitud de flecha fuera de rango"
                    )));
                }
                if d == 0.0 {
                    1.0
                } else {
                    d
                }
            };
            let (nx, ny) = (-(ey - sy) / dist, (ex - sx) / dist);
            let curva =
                opt_num(f_map, "bend", &format!("{donde}.arrows[{i}].bend"))?.unwrap_or(1.0);
            sx += nx * 9.0 * curva;
            sy += ny * 9.0 * curva;
            ex += nx * 9.0 * curva;
            ey += ny * 9.0 * curva;
            let (kx, ky) = (
                sx + (ex - sx) * 0.5 + nx * dist * 0.45 * curva,
                sy + (ey - sy) * 0.5 + ny * dist * 0.45 * curva,
            );
            if !sx.is_finite()
                || !sy.is_finite()
                || !ex.is_finite()
                || !ey.is_finite()
                || !kx.is_finite()
                || !ky.is_finite()
            {
                return Err(ErrorDiagrama::new(format!(
                    "{donde}.arrows[{i}]: coordenadas de flecha fuera de rango"
                )));
            }
            puntos_extra.push((kx, ky));
            let acento = c.pal.accent_primary;
            if booleano(f_map, "half", false) {
                let (mut tx, mut ty) = (ex - kx, ey - ky);
                let nt = {
                    let h = tx.hypot(ty);
                    if !h.is_finite() {
                        return Err(ErrorDiagrama::new(format!(
                            "{donde}.arrows[{i}]: vector tangente de flecha fuera de rango"
                        )));
                    }
                    if h == 0.0 {
                        1.0
                    } else {
                        h
                    }
                };
                tx /= nt;
                ty /= nt;
                let ang = 28.0_f64.to_radians();
                let hx = ex - 8.0 * (tx * ang.cos() - ty * ang.sin());
                let hy = ey - 8.0 * (tx * ang.sin() + ty * ang.cos());
                if !hx.is_finite() || !hy.is_finite() {
                    return Err(ErrorDiagrama::new(format!(
                        "{donde}.arrows[{i}]: punta de flecha fuera de rango"
                    )));
                }
                sub.add(format!(
                    r#"<path d="M {sx:.1} {sy:.1} Q {kx:.1} {ky:.1} {ex:.1} {ey:.1}" fill="none" stroke="{acento}" stroke-width="1.3"/>"#
                ));
                sub.add(format!(
                    r#"<line x1="{ex:.1}" y1="{ey:.1}" x2="{hx:.1}" y2="{hy:.1}" stroke="{acento}" stroke-width="1.3"/>"#
                ));
            } else {
                sub.add(format!(
                    r#"<path d="M {sx:.1} {sy:.1} Q {kx:.1} {ky:.1} {ex:.1} {ey:.1}" fill="none" stroke="{acento}" stroke-width="1.3" marker-end="url(#flecha-main-{})"/>"#,
                    c.sufijo
                ));
            }
        }
    }

    let mut min_x = f64::INFINITY;
    let mut max_x = f64::NEG_INFINITY;
    let mut min_y = f64::INFINITY;
    let mut max_y = f64::NEG_INFINITY;
    for &(px, py) in pos.iter().chain(puntos_extra.iter()) {
        min_x = min_x.min(px);
        max_x = max_x.max(px);
        min_y = min_y.min(py);
        max_y = max_y.max(py);
    }
    let margen = 14.0;
    let (x0, y0) = (min_x - margen, min_y - margen);
    let (w, h) = (max_x - x0 + margen, max_y - y0 + margen);
    if !x0.is_finite() || !y0.is_finite() || !w.is_finite() || !h.is_finite() {
        return Err(ErrorDiagrama::new(format!(
            "{donde}: dimensiones del diagrama fuera de rango"
        )));
    }
    incrustar(c, &sub, -x0, -y0);
    Ok((w, h))
}

fn render_molecule(c: &mut Lienzo, b: &Bloque) -> Result<Punto, ErrorDiagrama> {
    let top = encabezado_q(c, b);
    let mut sub = Lienzo::new(c.pal, c.sufijo.clone());
    let (w, h) = dibujar_molecula(&mut sub, b, "molecule")?;
    let mut alto = top + h;
    let cap = opt_texto(b, "caption");
    let alto_final = if cap.is_some() { alto + 22.0 } else { alto };
    if !w.is_finite() || !alto_final.is_finite() {
        return Err(ErrorDiagrama::new(
            "molecule: dimensiones del diagrama fuera de rango",
        ));
    }
    incrustar(c, &sub, 0.0, top);
    if let Some(cap) = cap {
        alto += 4.0;
        tq(
            c,
            w / 2.0,
            alto + 12.0,
            &cap,
            11.0,
            400,
            "textMain",
            "middle",
            SANS,
            false,
        );
        alto += 18.0;
    }
    Ok((w, alto))
}

// --- 2. Reacción -----------------------------------------------------------------------------------------

fn flecha_reaccion(s: &str) -> Option<&'static str> {
    match s {
        "->" | "→" => Some("forward"),
        "<=>" | "eq" | "⇌" => Some("equilibrium"),
        "<->" | "↔" => Some("resonance"),
        "=>" => Some("retro"),
        _ => None,
    }
}

fn pieza_reaccion(
    c: &Lienzo,
    dato: &Value,
    i: usize,
) -> Result<(Lienzo, f64, f64), ErrorDiagrama> {
    let mut sub = Lienzo::new(c.pal, c.sufijo.clone());
    let texto_opt = match dato {
        Value::Number(n) => Some(n.to_string()),
        Value::String(s) => Some(s.clone()),
        _ => None,
    };
    if let Some(texto) = texto_opt {
        if texto.trim() == "+" {
            sub.t(9.0, 5.0, "+", 16.0, 600, "textMain", "middle", SANS);
            return Ok((sub, 18.0, 0.0));
        }
        let w = tq(
            &mut sub,
            0.0,
            5.0,
            &texto,
            13.0,
            400,
            "textMain",
            "start",
            SANS,
            false,
        );
        return Ok((sub, w, 0.0));
    }
    let Some(map) = dato.as_object() else {
        return Err(ErrorDiagrama::new(format!(
            "reaction.steps[{i}]: una fórmula, '+', {{arrow: ...}} o una molécula"
        )));
    };
    if let Some(arrow_val) = map.get("arrow") {
        let arrow_str = valor_str(arrow_val);
        let Some(tipo) = flecha_reaccion(&arrow_str) else {
            return Err(ErrorDiagrama::new(format!(
                "reaction.steps[{i}].arrow: '{arrow_str}' (vale ->, <=>, <->, =>)"
            )));
        };
        let arriba = opt_texto(map, "above");
        let abajo = opt_texto(map, "below");
        let largo = 64.0_f64
            .max(ancho_formula(arriba.as_deref().unwrap_or(""), 10.0) + 16.0)
            .max(ancho_formula(abajo.as_deref().unwrap_or(""), 10.0) + 16.0);
        let (trazo, acento) = (c.pal.stroke_main, c.pal.accent_primary);
        match tipo {
            "forward" => {
                sub.add(format!(
                    r#"<line x1="4" y1="0" x2="{:.1}" y2="0" stroke="{acento}" stroke-width="1.6" marker-end="url(#flecha-main-{})"/>"#,
                    largo - 4.0,
                    c.sufijo
                ));
            }
            "resonance" => {
                sub.add(format!(
                    r#"<line x1="6" y1="0" x2="{:.1}" y2="0" stroke="{acento}" stroke-width="1.6" marker-start="url(#flecha-main-{})" marker-end="url(#flecha-main-{})"/>"#,
                    largo - 6.0,
                    c.sufijo,
                    c.sufijo
                ));
            }
            "retro" => {
                for o in [-2.5_f64, 2.5_f64] {
                    sub.add(format!(
                        r#"<line x1="4" y1="{o}" x2="{:.1}" y2="{o}" stroke="{trazo}" stroke-width="1.3"/>"#,
                        largo - 10.0
                    ));
                }
                sub.add(format!(
                    r#"<path d="M {:.1} -7 L {:.1} 0 L {:.1} 7" fill="none" stroke="{trazo}" stroke-width="1.3"/>"#,
                    largo - 14.0,
                    largo - 4.0,
                    largo - 14.0
                ));
            }
            _ => {
                sub.add(format!(
                    r#"<path d="M 4 -3 L {:.1} -3 L {:.1} -8" fill="none" stroke="{acento}" stroke-width="1.5"/>"#,
                    largo - 4.0,
                    largo - 11.0
                ));
                sub.add(format!(
                    r#"<path d="M {:.1} 3 L 4 3 L 11 8" fill="none" stroke="{acento}" stroke-width="1.5"/>"#,
                    largo - 4.0
                ));
            }
        }
        if booleano(map, "blank", false) {
            caja_vacia(&mut sub, largo / 2.0, -16.0, 46.0, 14.0);
        } else if let Some(ref arr) = arriba {
            tq(
                &mut sub,
                largo / 2.0,
                -9.0,
                arr,
                10.0,
                400,
                "textSub",
                "middle",
                SANS,
                false,
            );
        }
        if let Some(ref ab) = abajo {
            tq(
                &mut sub,
                largo / 2.0,
                18.0,
                ab,
                10.0,
                400,
                "textSub",
                "middle",
                SANS,
                false,
            );
        }
        return Ok((sub, largo, 0.0));
    }
    if booleano(map, "blank", false)
        && !map.get("atoms").is_some_and(es_truthy_val)
        && !map.get("rings").is_some_and(es_truthy_val)
    {
        let ancho = opt_num(map, "width", &format!("reaction.steps[{i}].width"))?.unwrap_or(70.0);
        if !(ancho > 0.0 && ancho.is_finite()) {
            return Err(ErrorDiagrama::new(format!(
                "reaction: steps[{i}].width tiene que ser positivo"
            )));
        }
        caja_vacia(&mut sub, ancho / 2.0, 0.0, ancho, 26.0);
        return Ok((sub, ancho, 0.0));
    }
    if let Some(f_val) = map.get("formula") {
        let f_str = valor_str(f_val);
        let w = tq(
            &mut sub,
            0.0,
            5.0,
            &f_str,
            13.0,
            400,
            "textMain",
            "start",
            SANS,
            false,
        );
        return Ok((sub, w, 0.0));
    }
    let mut interior = Lienzo::new(c.pal, c.sufijo.clone());
    let (w, h) = dibujar_molecula(&mut interior, map, &format!("reaction.steps[{i}]"))?;
    incrustar(&mut sub, &interior, 0.0, -h / 2.0);
    Ok((sub, w, h))
}

fn render_reaction(c: &mut Lienzo, b: &Bloque) -> Result<Punto, ErrorDiagrama> {
    let Some(pasos) = b.get("steps").and_then(Value::as_array) else {
        return Err(ErrorDiagrama::new(
            "reaction: 'steps' es una lista de fórmulas, '+', flechas y moléculas",
        ));
    };
    if pasos.is_empty() {
        return Err(ErrorDiagrama::new(
            "reaction: 'steps' es una lista de fórmulas, '+', flechas y moléculas",
        ));
    }
    let top = encabezado_q(c, b);
    let maximo = opt_num(b, "maxWidth", "reaction.maxWidth")?.unwrap_or(640.0);
    if !(maximo > 0.0 && maximo.is_finite()) {
        return Err(ErrorDiagrama::new(
            "reaction: maxWidth tiene que ser positivo",
        ));
    }
    let mut piezas = Vec::with_capacity(pasos.len());
    for (i, p) in pasos.iter().enumerate() {
        piezas.push(pieza_reaccion(c, p, i)?);
    }
    let mut filas: Vec<Vec<(Lienzo, f64, f64)>> = vec![Vec::new()];
    let mut ancho_fila = 0.0_f64;
    for pieza in piezas {
        if filas.last().is_some_and(|f| !f.is_empty()) && ancho_fila + pieza.1 > maximo {
            filas.push(Vec::new());
            ancho_fila = 0.0;
        }
        ancho_fila += pieza.1 + 10.0;
        if let Some(ultima) = filas.last_mut() {
            ultima.push(pieza);
        }
    }
    let mut y = top;
    let mut ancho = 0.0_f64;
    for fila in filas {
        let alto = fila.iter().map(|p| p.2).fold(40.0_f64, f64::max);
        let centro = y + alto / 2.0;
        let mut x = 0.0_f64;
        for (sub, w, _h) in fila {
            if !x.is_finite() || !centro.is_finite() || !(x + w + 10.0).is_finite() {
                return Err(ErrorDiagrama::new(
                    "reaction: dimensiones del diagrama fuera de rango",
                ));
            }
            incrustar(c, &sub, x, centro);
            x += w + 10.0;
        }
        ancho = ancho.max(x);
        y += alto + 14.0;
        if !ancho.is_finite() || !y.is_finite() {
            return Err(ErrorDiagrama::new(
                "reaction: dimensiones del diagrama fuera de rango",
            ));
        }
    }
    Ok((ancho, y - 14.0))
}

// --- 3. TRePEV -------------------------------------------------------------------------------------------

fn axe_sub(texto: &str) -> String {
    let mut out = String::with_capacity(texto.len() + 4);
    for ch in texto.chars() {
        if ch.is_ascii_digit() {
            out.push('_');
        }
        out.push(ch);
    }
    out
}

fn lobulo(c: &mut Lienzo, x: f64, y: f64, ang: f64, largo: f64) {
    let (ux, uy) = (ang.cos(), -ang.sin());
    let (cx, cy) = (x + ux * (11.0 + largo * 0.5), y + uy * (11.0 + largo * 0.5));
    let grados = -ang.to_degrees();
    c.add(format!(
        r#"<ellipse cx="{cx:.1}" cy="{cy:.1}" rx="{:.1}" ry="{:.1}" fill="none" stroke="{}" stroke-width="1" transform="rotate({grados:.1} {cx:.1} {cy:.1})"/>"#,
        largo * 0.5,
        largo * 0.27,
        c.pal.stroke_sub
    ));
    for s in [-2.6_f64, 2.6_f64] {
        c.punto(
            cx + ux * 3.0 - uy * s,
            cy + uy * 3.0 + ux * s,
            "strokeMain",
            1.4,
        );
    }
}

fn render_vsepr(c: &mut Lienzo, b: &Bloque) -> Result<Punto, ErrorDiagrama> {
    let mut ligandos: Vec<String> = opt_lista(b, "ligands", "vsepr.ligands")?
        .map(|arr| arr.iter().map(valor_str).collect())
        .unwrap_or_default();
    let clave = match b.get("geometry") {
        None | Some(Value::Null) => {
            let lp_i64 = match b.get("lonePairs") {
                None | Some(Value::Null) => 0,
                Some(v) => entero(v, "vsepr.lonePairs")?,
            };
            usize::try_from(lp_i64)
                .ok()
                .and_then(|lp| por_pares(ligandos.len(), lp))
                .ok_or_else(|| {
                    ErrorDiagrama::new(format!(
                        "vsepr: {} ligandos y {} pares libres no forman una geometría conocida (o indicá geometry)",
                        ligandos.len(),
                        lp_i64
                    ))
                })?
                .to_string()
        }
        Some(v) => valor_str(v),
    };
    let geom = geometria(&clave).ok_or_else(|| {
        let disponibles: Vec<&str> = GEOMETRIAS.iter().map(|g| g.clave).collect();
        ErrorDiagrama::new(format!(
            "vsepr: geometry '{clave}' (valen {})",
            disponibles.join(", ")
        ))
    })?;
    if ligandos.is_empty() {
        ligandos = vec!["X".to_string(); geom.ligandos.len()];
    }
    if ligandos.len() != geom.ligandos.len() {
        return Err(ErrorDiagrama::new(format!(
            "vsepr: la geometría {} lleva {} ligandos (hay {})",
            geom.clave,
            geom.ligandos.len(),
            ligandos.len()
        )));
    }
    let top = encabezado_q(c, b);
    let largo = 56.0_f64;
    let (cx, cy) = (130.0_f64, top + 84.0);
    let trazo = c.pal.stroke_main;
    let central = b
        .get("central")
        .filter(|v| !v.is_null())
        .map(valor_str)
        .unwrap_or_else(|| "A".to_string());

    for &(ang_g, tipo) in geom.ligandos {
        let ang = ang_g.to_radians();
        let f = if tipo != "p" { 0.82 } else { 1.0 };
        let (ux, uy) = (ang.cos(), -ang.sin());
        let (ax, ay) = (cx + ux * 11.0, cy + uy * 11.0);
        let (bx, by) = (
            cx + ux * largo * f - ux * 9.0,
            cy + uy * largo * f - uy * 9.0,
        );
        let (px, py) = (-uy, ux);
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
                for k in 1..=7 {
                    let t = k as f64 / 7.0;
                    let (hx, hy) = (ax + (bx - ax) * t, ay + (by - ay) * t);
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
    }
    for (&(ang_g, tipo), etiqueta) in geom.ligandos.iter().zip(ligandos.iter()) {
        let ang = ang_g.to_radians();
        let f = if tipo != "p" { 0.82 } else { 1.0 };
        tq(
            c,
            cx + ang.cos() * largo * f,
            cy - ang.sin() * largo * f + 4.0,
            etiqueta,
            12.0,
            500,
            "textMain",
            "middle",
            SANS,
            false,
        );
    }
    for &ang_g in geom.pares_libres {
        lobulo(c, cx, cy, ang_g.to_radians(), 26.0);
    }
    tq(
        c,
        cx,
        cy + 4.5,
        &central,
        13.0,
        700,
        "textMain",
        "middle",
        SANS,
        false,
    );

    let todo_blanco = es_bool_true(b, "blank");
    let blancos: HashSet<String> = match b.get("blank") {
        Some(Value::Array(arr)) => arr.iter().map(valor_str).collect(),
        _ => HashSet::new(),
    };
    let (i, j) = geom.par_angulo;
    let diferencia = (geom.ligandos[j].0 - geom.ligandos[i].0 + 180.0).rem_euclid(360.0) - 180.0;
    if booleano(b, "showAngle", true) && diferencia.abs() < 179.0 {
        let a1 = geom.ligandos[i].0.to_radians();
        let a2 = (geom.ligandos[i].0 + diferencia).to_radians();
        let r = 26.0_f64;
        let (x1, y1) = (cx + a1.cos() * r, cy - a1.sin() * r);
        let (x2, y2) = (cx + a2.cos() * r, cy - a2.sin() * r);
        let sweep = if diferencia > 0.0 { 0 } else { 1 };
        c.add(format!(
            r#"<path d="M {x1:.1} {y1:.1} A 26 26 0 0 {sweep} {x2:.1} {y2:.1}" fill="none" stroke="{}" stroke-width="1" stroke-dasharray="2 2"/>"#,
            c.pal.stroke_sub
        ));
        let medio = a1 + (diferencia / 2.0).to_radians();
        let (tx, ty) = (cx + medio.cos() * (r + 15.0), cy - medio.sin() * (r + 15.0));
        if todo_blanco || blancos.contains("angle") {
            caja_vacia(c, tx, ty, 34.0, 13.0);
        } else {
            let ang_corto = geom.angulo.split(" y ").next().unwrap_or(geom.angulo);
            c.t(tx, ty + 3.5, ang_corto, 8.5, 400, "textSub", "middle", SANS);
        }
    }

    let filas = [
        ("Notación", geom.axe, "axe"),
        ("Geometría", geom.nombre, "name"),
        ("Ángulo", geom.angulo, "angle"),
        ("Hibridación", geom.hibridacion, "hybridization"),
    ];
    let x = 270.0_f64;
    let mut y = top + 50.0;
    let show_hyb = booleano(b, "showHybridization", true);
    for (etiqueta, valor, clave_b) in filas {
        if clave_b == "hybridization" && !show_hyb {
            continue;
        }
        c.t(
            x,
            y,
            &format!("{etiqueta}:"),
            10.5,
            700,
            "textMain",
            "start",
            SANS,
        );
        if todo_blanco || blancos.contains(clave_b) {
            c.vacio(x + 88.0, x + 200.0, y + 2.0);
        } else if clave_b == "axe" {
            tq(
                c,
                x + 88.0,
                y,
                &axe_sub(valor),
                10.5,
                400,
                "textMain",
                "start",
                SANS,
                false,
            );
        } else {
            c.t(x + 88.0, y, valor, 10.5, 400, "textMain", "start", SANS);
        }
        y += 22.0;
    }
    Ok((480.0, (top + 168.0).max(y)))
}

// --- 4. Orbitales ----------------------------------------------------------------------------------------

fn render_orbitals(c: &mut Lienzo, b: &Bloque) -> Result<Punto, ErrorDiagrama> {
    let tipo = b
        .get("kind")
        .filter(|v| !v.is_null())
        .map(valor_str)
        .unwrap_or_else(|| "atomic".to_string());
    match tipo.as_str() {
        "atomic" => orbitales_atomicos(c, b),
        "molecular" => orbitales_moleculares(c, b),
        _ => Err(ErrorDiagrama::new(format!(
            "orbitals: kind '{tipo}' (vale atomic o molecular)"
        ))),
    }
}

fn orbitales_atomicos(c: &mut Lienzo, b: &Bloque) -> Result<Punto, ErrorDiagrama> {
    let elem_val = b
        .get("element")
        .filter(|v| !v.is_null())
        .ok_or_else(|| ErrorDiagrama::new("orbitals: falta element (símbolo o número atómico)"))?;
    let carga_i64 = match b.get("charge") {
        None | Some(Value::Null) => 0,
        Some(v) => entero(v, "orbitals.charge")?,
    };
    let carga = i32::try_from(carga_i64)
        .map_err(|_| ErrorDiagrama::new("orbitals.charge: fuera de rango entero"))?;
    let conf = envolver_error("orbitals", chem::configuracion(elem_val, carga))?;
    let blanco = booleano(b, "blank", false);
    let use_core = booleano(b, "core", false);
    let nucleo = if use_core { conf.nucleo() } else { None };
    let subniveles: Vec<((u8, u8), u8)> = conf
        .ocupacion
        .iter()
        .copied()
        .filter(|(nl, _)| nucleo.as_ref().is_none_or(|(_, llenos)| !llenos.contains(nl)))
        .collect();
    let top = encabezado_q(c, b);
    let mut x = 0.0_f64;
    let mut y = top + 22.0;
    let mut ancho = 0.0_f64;
    if let Some((gas, _)) = &nucleo {
        c.r(0.0, y, 44.0, 22.0, "fillHeader", 1.2, 0.0);
        c.t(
            22.0,
            y + 15.0,
            &format!("[{gas}]"),
            11.0,
            700,
            "textMain",
            "middle",
            SANS,
        );
        x = 58.0;
    }
    let maximo = opt_num(b, "maxWidth", "orbitals.maxWidth")?.unwrap_or(600.0);
    for (nl, e) in subniveles {
        let n_cajas = usize::from(2 * nl.1 + 1);
        let w = n_cajas as f64 * 22.0;
        if x + w > maximo && x > 0.0 {
            x = 0.0;
            y += 52.0;
        }
        cajas(c, x, y, usize::from(e), n_cajas, 22.0, blanco);
        c.t(
            x + w / 2.0,
            y + 36.0,
            &chem::nombre_subnivel(nl),
            10.5,
            600,
            "textMain",
            "middle",
            SANS,
        );
        x += w + 14.0;
        ancho = ancho.max(x);
    }
    y += 58.0;
    let especie = format!("{}{}", conf.simbolo, carga_sup(i64::from(carga)));
    if booleano(b, "showConfig", true) {
        c.t(
            0.0,
            y,
            "Configuración:",
            10.5,
            700,
            "textMain",
            "start",
            SANS,
        );
        tq(
            c, 96.0, y, &especie, 10.5, 700, "textMain", "start", SANS, false,
        );
        if blanco || booleano(b, "blankConfig", false) {
            c.vacio(130.0, 430.0, y + 2.0);
        } else {
            c.t(
                130.0,
                y,
                &conf.texto(use_core),
                10.5,
                400,
                "textMain",
                "start",
                SANS,
            );
        }
        y += 20.0;
    }
    if booleano(b, "showUnpaired", false) {
        let ocultar = blanco || booleano(b, "blankUnpaired", false);
        let valor = if ocultar {
            None
        } else {
            Some(conf.desapareados().to_string())
        };
        rotulo_q(
            c,
            0.0,
            y,
            "Electrones desapareados",
            valor.as_deref(),
            ocultar,
            "start",
            10.5,
        );
        y += 20.0;
    }
    Ok((ancho.max(440.0), y))
}

fn nombre_om(c: &mut Lienzo, x: f64, y: f64, nombre: &str) {
    let mut chars = nombre.chars();
    let letra = chars.next().unwrap_or('?');
    let resto: String = chars.collect();
    let estrella = resto.starts_with('*');
    let origen = resto.trim_start_matches('*');
    let texto = format!("{letra}{}_{{{origen}}}", if estrella { "^*" } else { "" });
    tq(c, x, y, &texto, 10.0, 400, "textSub", "middle", SANS, false);
}

struct PuntoOA<'a> {
    es_izq: bool,
    orb: &'a str,
    x0: f64,
    x1: f64,
    y: f64,
}

fn orbitales_moleculares(c: &mut Lienzo, b: &Bloque) -> Result<Punto, ErrorDiagrama> {
    let atomos_val = b
        .get("atoms")
        .filter(|v| es_truthy_val(v))
        .or_else(|| b.get("molecule").filter(|v| es_truthy_val(v)))
        .ok_or_else(|| {
            ErrorDiagrama::new("orbitals: kind molecular necesita atoms: [O, O] (o molecule: O)")
        })?;
    let carga_i64 = match b.get("charge") {
        None | Some(Value::Null) => 0,
        Some(v) => entero(v, "orbitals.charge")?,
    };
    let carga = i32::try_from(carga_i64)
        .map_err(|_| ErrorDiagrama::new("orbitals.charge: fuera de rango entero"))?;
    let om = envolver_error("orbitals", chem::orbitales_moleculares(atomos_val, carga))?;
    let top = encabezado_q(c, b);
    let (ref izq, ref der) = om.atomos;
    let en_izq = chem::electronegatividad(izq);
    let en_der = chem::electronegatividad(der);
    let (desp_izq, desp_der) = if en_izq != en_der {
        if en_izq > en_der {
            (16.0, 0.0)
        } else {
            (0.0, 16.0)
        }
    } else {
        (0.0, 0.0)
    };
    let paso = 44.0_f64;
    let alto = paso * (om.niveles.len() + 1) as f64;
    let base = top + 30.0 + alto;
    let y_om = |nombre: &str| -> f64 {
        om.niveles
            .iter()
            .position(|n| n.nombre == nombre)
            .map_or(base, |i| base - paso * (i + 1) as f64)
    };
    let (xa, xm, xb) = (40.0_f64, 230.0_f64, 420.0_f64);
    let vacio = booleano(b, "blankElectrons", false);
    let lado = 20.0_f64;
    let orbitales_at: Vec<(&str, usize, f64)> = if om.capa == 1 {
        vec![("1s", 1, (y_om("σ1s") + y_om("σ*1s")) / 2.0)]
    } else {
        let y2s = (y_om("σ2s") + y_om("σ*2s")) / 2.0;
        let y2p = (y_om("π2p").min(y_om("σ2p")) + y_om("π*2p")) / 2.0;
        vec![("2s", 1, y2s), ("2p", 3, y2p)]
    };

    let mut puntos_at: Vec<PuntoOA<'_>> = Vec::new();
    for (lado_x, atomo, desp, es_izq) in [
        (xa, izq.as_str(), desp_izq, true),
        (xb, der.as_str(), desp_der, false),
    ] {
        let conf = envolver_error(
            "orbitals",
            chem::configuracion(&Value::String(atomo.to_string()), 0),
        )?;
        for &(orb, n_cajas, y_base) in &orbitales_at {
            let y = y_base + desp;
            let n_num = orb
                .chars()
                .next()
                .and_then(|ch| ch.to_digit(10))
                .unwrap_or(1) as u8;
            let l_num = orb
                .chars()
                .nth(1)
                .and_then(|ch| chem::LETRAS.find(ch))
                .unwrap_or(0) as u8;
            let electrones = usize::from(conf.ocupacion_de((n_num, l_num)));
            let ancho = n_cajas as f64 * lado;
            let x0 = if es_izq {
                lado_x
            } else {
                lado_x - ancho + lado * 3.0
            };
            cajas(c, x0, y - lado / 2.0, electrones, n_cajas, lado, false);
            tq(
                c,
                x0 + ancho / 2.0,
                y + lado / 2.0 + 13.0,
                orb,
                9.5,
                400,
                "textSub",
                "middle",
                SANS,
                false,
            );
            puntos_at.push(PuntoOA {
                es_izq,
                orb,
                x0,
                x1: x0 + ancho,
                y,
            });
        }
        tq(
            c,
            lado_x + lado * 1.5,
            top + 16.0,
            atomo,
            12.0,
            700,
            "textMain",
            "middle",
            SANS,
            false,
        );
    }

    let mut nombre_molecula = if izq == der {
        format!("{izq}2")
    } else {
        format!("{izq}{der}")
    };
    nombre_molecula.push_str(&carga_sup(i64::from(om.carga)));
    tq(
        c,
        xm + lado,
        top + 16.0,
        &nombre_molecula,
        12.0,
        700,
        "textMain",
        "middle",
        SANS,
        false,
    );

    for nivel in &om.niveles {
        let y = y_om(nivel.nombre);
        let ancho = nivel.degenerado as f64 * lado;
        let x0 = xm + lado - ancho / 2.0;
        cajas(
            c,
            x0,
            y - lado / 2.0,
            nivel.electrones,
            nivel.degenerado,
            lado,
            vacio,
        );
        nombre_om(c, x0 + ancho / 2.0, y + lado / 2.0 + 11.0, nivel.nombre);
        for p_oa in &puntos_at {
            if p_oa.orb != nivel.origen {
                continue;
            }
            if p_oa.es_izq {
                c.linea(
                    p_oa.x1 + 2.0,
                    p_oa.y,
                    x0 - 2.0,
                    y,
                    "strokeMuted",
                    0.8,
                    None,
                    "3 3",
                );
            } else {
                c.linea(
                    p_oa.x0 - 2.0,
                    p_oa.y,
                    x0 + ancho + 2.0,
                    y,
                    "strokeMuted",
                    0.8,
                    None,
                    "3 3",
                );
            }
        }
    }

    c.add(format!(
        r#"<line x1="{:.1}" y1="{base:.1}" x2="{:.1}" y2="{:.1}" stroke="{}" stroke-width="1.2" marker-end="url(#flecha-sub-{})"/>"#,
        xa - 18.0,
        xa - 18.0,
        top + 30.0,
        c.pal.stroke_sub,
        c.sufijo
    ));
    c.add(format!(
        r#"<text x="{:.1}" y="{:.1}" text-anchor="middle" font-family="{SANS}" font-size="10" fill="{}" transform="rotate(-90 {:.1} {:.1})">Energía</text>"#,
        xa - 24.0,
        (base + top) / 2.0,
        c.pal.text_sub,
        xa - 24.0,
        (base + top) / 2.0
    ));

    let mut y = base + 26.0;
    let blanco = es_bool_true(b, "blank");
    let orden = om.orden_enlace();
    let texto_orden = if (orden - orden.round()).abs() < 1e-12 {
        format!("{}", orden.round() as i64)
    } else {
        coma(orden, 1)
    };
    let ocultar_orden = blanco || booleano(b, "blankBondOrder", false);
    rotulo_q(
        c,
        0.0,
        y,
        "Orden de enlace",
        Some(&texto_orden),
        ocultar_orden,
        "start",
        10.5,
    );
    y += 20.0;
    if booleano(b, "blankMagnetism", false) || blanco {
        c.t(0.0, y, "Magnetismo:", 10.5, 700, "textMain", "start", SANS);
        c.vacio(84.0, 220.0, y + 2.0);
    } else {
        c.t(
            0.0,
            y,
            &format!(
                "Magnetismo: {} ({} e⁻ desapareados)",
                om.magnetismo(),
                om.desapareados()
            ),
            10.5,
            700,
            "textMain",
            "start",
            SANS,
        );
    }
    y += 8.0;
    Ok((xb + lado * 3.0 + 10.0, y))
}

// --- 5. Perfil de energía --------------------------------------------------------------------------------

struct EstadoEnergia {
    label: String,
    energy: f64,
    ts: bool,
}

fn parse_estados(val: Option<&Value>, donde: &str) -> Result<Vec<EstadoEnergia>, ErrorDiagrama> {
    let Some(arr) = val.and_then(Value::as_array) else {
        return Err(ErrorDiagrama::new(format!(
            "energy_profile: {donde} es una lista de al menos dos estados {{label, energy}}"
        )));
    };
    if arr.len() < 2 {
        return Err(ErrorDiagrama::new(format!(
            "energy_profile: {donde} es una lista de al menos dos estados {{label, energy}}"
        )));
    }
    let mut salida = Vec::with_capacity(arr.len());
    for (i, item) in arr.iter().enumerate() {
        let Some(map) = item.as_object() else {
            return Err(ErrorDiagrama::new(format!(
                "energy_profile: {donde}[{i}] es {{label, energy, ts}}"
            )));
        };
        let Some(e_val) = map.get("energy").filter(|v| !v.is_null()) else {
            return Err(ErrorDiagrama::new(format!(
                "energy_profile: {donde}[{i}] es {{label, energy, ts}}"
            )));
        };
        let energy = num(e_val, &format!("energy_profile.{donde}[{i}].energy"))?;
        let label = map
            .get("label")
            .filter(|v| !v.is_null())
            .map(valor_str)
            .unwrap_or_default();
        let ts = booleano(map, "ts", false);
        salida.push(EstadoEnergia { label, energy, ts });
    }
    Ok(salida)
}

fn render_energy_profile(c: &mut Lienzo, b: &Bloque) -> Result<Punto, ErrorDiagrama> {
    let estados = parse_estados(b.get("states"), "states")?;
    let catal = match b.get("catalyzed") {
        None | Some(Value::Null) => None,
        Some(Value::Array(arr)) if arr.is_empty() => None,
        Some(v) => Some(parse_estados(Some(v), "catalyzed")?),
    };
    let top = encabezado_q(c, b);
    let (x0, y0, w, h) = (50.0_f64, top + 16.0, 480.0_f64, 250.0_f64);
    let mut lo = f64::INFINITY;
    let mut hi = f64::NEG_INFINITY;
    for e in estados
        .iter()
        .chain(catal.as_deref().unwrap_or(&[]).iter())
    {
        lo = lo.min(e.energy);
        hi = hi.max(e.energy);
    }
    let rango = hi - lo;
    if !rango.is_finite() {
        return Err(ErrorDiagrama::new(
            "energy_profile: rango de energías fuera de rango",
        ));
    }
    let margen = {
        let m = rango * 0.15;
        if m == 0.0 {
            10.0
        } else {
            m
        }
    };
    let span = rango + 2.0 * margen;
    if !margen.is_finite() || !span.is_finite() || span == 0.0 {
        return Err(ErrorDiagrama::new(
            "energy_profile: escala vertical fuera de rango",
        ));
    }
    let py = |v: f64| -> f64 { y0 + h - (v - lo + margen) / span * h };

    let trazo = c.pal.stroke_main;
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
    let unidad = b
        .get("unit")
        .filter(|v| !v.is_null())
        .map(valor_str)
        .unwrap_or_else(|| "kJ/mol".to_string());
    let y_label = opt_texto(b, "yLabel").unwrap_or_else(|| format!("Energía ({unidad})"));
    let x_label = opt_texto(b, "xLabel").unwrap_or_else(|| "Coordenada de reacción".to_string());
    c.add(format!(
        r#"<text x="{:.1}" y="{:.1}" text-anchor="middle" font-family="{SANS}" font-size="10" font-weight="600" fill="{}" transform="rotate(-90 {:.1} {:.1})">{}</text>"#,
        x0 - 14.0,
        y0 + h / 2.0,
        c.pal.text_main,
        x0 - 14.0,
        y0 + h / 2.0,
        dibu_core::esc(&y_label)
    ));
    c.t(
        x0 + w / 2.0,
        y0 + h + 18.0,
        &x_label,
        10.0,
        600,
        "textMain",
        "middle",
        SANS,
    );

    let dibujar = |lienzo: &mut Lienzo, lista: &[EstadoEnergia], estilo: usize| -> Vec<f64> {
        let n = lista.len() - 1;
        let xs: Vec<f64> = (0..=n)
            .map(|i| x0 + 30.0 + (w - 60.0) * i as f64 / n as f64)
            .collect();
        let mut puntos_str = String::new();
        let _ = write!(&mut puntos_str, "{:.1},{:.1}", xs[0] - 22.0, py(lista[0].energy));
        for i in 0..n {
            for k in 0..=24 {
                let t = k as f64 / 24.0;
                let s = (1.0 - (PI * t).cos()) / 2.0;
                let px = xs[i] + (xs[i + 1] - xs[i]) * t;
                let p_y = py(lista[i].energy + (lista[i + 1].energy - lista[i].energy) * s);
                let _ = write!(&mut puntos_str, " {px:.1},{p_y:.1}");
            }
        }
        let _ = write!(
            &mut puntos_str,
            " {:.1},{:.1}",
            xs[n] + 22.0,
            py(lista[n].energy)
        );
        let guiones = GUIONES[estilo % GUIONES.len()];
        let dash = if guiones.is_empty() {
            String::new()
        } else {
            format!(r#" stroke-dasharray="{guiones}""#)
        };
        lienzo.add(format!(
            r#"<polyline points="{puntos_str}" fill="none" stroke="{trazo}" stroke-width="1.9"{dash}/>"#
        ));
        xs
    };

    let xs = dibujar(c, &estados, 0);
    let blanco = booleano(b, "blankAnnotations", false);
    let show_energies = booleano(b, "showEnergies", false);
    for (&x, e) in xs.iter().zip(estados.iter()) {
        let y = py(e.energy);
        if !e.label.is_empty() {
            if e.ts {
                tq(
                    c,
                    x,
                    y - 10.0,
                    &e.label,
                    10.0,
                    400,
                    "textMain",
                    "middle",
                    SANS,
                    false,
                );
            } else {
                tq(
                    c,
                    x,
                    y + 18.0,
                    &e.label,
                    10.5,
                    400,
                    "textMain",
                    "middle",
                    SANS,
                    false,
                );
            }
        }
        if show_energies {
            c.t(
                x,
                if e.ts { y - 24.0 } else { y - 6.0 },
                &fmt_g(Some(e.energy), 3),
                9.0,
                400,
                "textSub",
                "middle",
                SANS,
            );
        }
    }
    if let Some(ref cat) = catal {
        dibujar(c, cat, 1);
        c.t(
            x0 + w - 4.0,
            y0 + 8.0,
            "— sin catalizador",
            9.5,
            400,
            "textMain",
            "end",
            SANS,
        );
        c.t(
            x0 + w - 4.0,
            y0 + 22.0,
            "- - con catalizador",
            9.5,
            400,
            "textMain",
            "end",
            SANS,
        );
    }

    let pedidas: Vec<String> = match opt_lista(b, "annotations", "energy_profile.annotations")? {
        Some(arr) => arr.iter().map(valor_str).collect(),
        None => vec!["Ea".to_string(), "dH".to_string()],
    };
    let mut etapas: Vec<(usize, usize)> = Vec::new();
    let mut minimo_idx = 0usize;
    for (i, e) in estados.iter().enumerate() {
        if e.ts {
            etapas.push((i, minimo_idx));
        } else {
            minimo_idx = i;
        }
    }
    if pedidas.iter().any(|p| p == "Ea") {
        let varias = etapas.len() > 1;
        for (k0, &(i, base_idx)) in etapas.iter().enumerate() {
            let k = k0 + 1;
            let x = xs[i];
            let yb = py(estados[base_idx].energy);
            let yt = py(estados[i].energy);
            let xb = xs[base_idx];
            c.linea(xb, yb, x + 8.0, yb, "strokeSub", 0.9, None, "4 3");
            flecha_doble(c, x + 4.0, yb - 1.0, yt + 1.0, "strokeSub");
            let nombre = if varias {
                format!("Ea_{k}")
            } else {
                "Ea".to_string()
            };
            let valor = format!(
                "{} {unidad}",
                fmt_g(Some(estados[i].energy - estados[base_idx].energy), 3)
            );
            if blanco {
                let eq_txt = format!("{nombre} =");
                tq(
                    c,
                    x + 10.0,
                    (yb + yt) / 2.0 + 4.0,
                    &eq_txt,
                    10.0,
                    700,
                    "textMain",
                    "start",
                    SANS,
                    true,
                );
                c.vacio(
                    x + 10.0 + ancho_formula(&eq_txt, 10.0) + 4.0,
                    x + 10.0 + ancho_formula(&nombre, 10.0) + 60.0,
                    (yb + yt) / 2.0 + 6.0,
                );
            } else {
                tq(
                    c,
                    x + 10.0,
                    (yb + yt) / 2.0 + 4.0,
                    &format!("{nombre} = {valor}"),
                    10.0,
                    700,
                    "textMain",
                    "start",
                    SANS,
                    true,
                );
            }
        }
    }
    if pedidas.iter().any(|p| p == "dH") {
        let ei = estados[0].energy;
        let ef = estados[estados.len() - 1].energy;
        let x = xs[xs.len() - 1] + 36.0;
        c.linea(xs[0], py(ei), x + 6.0, py(ei), "strokeSub", 0.9, None, "4 3");
        flecha_doble(c, x, py(ei), py(ef), "strokeSub");
        let signo = if ef < ei {
            "exotérmica"
        } else {
            "endotérmica"
        };
        let texto = format!("{} {unidad}", fmt_g(Some(ef - ei), 3));
        let y_texto = (py(ei) + py(ef)) / 2.0 + 4.0;
        if blanco {
            c.t(x + 6.0, y_texto, "ΔH =", 10.0, 700, "textMain", "start", SANS);
            c.vacio(x + 38.0, x + 100.0, y_texto + 2.0);
        } else {
            c.t(
                x + 6.0,
                y_texto,
                &format!("ΔH = {texto}"),
                10.0,
                700,
                "textMain",
                "start",
                SANS,
            );
            c.t(
                x + 6.0,
                y_texto + 13.0,
                &format!("({signo})"),
                9.0,
                400,
                "textSub",
                "start",
                SANS,
            );
        }
    }
    if booleano(b, "showRateDetermining", false) && etapas.len() > 1 {
        let mut best_k = 1usize;
        let mut best_i = etapas[0].0;
        let mut best_ea = estados[etapas[0].0].energy - estados[etapas[0].1].energy;
        for (idx, &(i, base_idx)) in etapas.iter().enumerate().skip(1) {
            let ea = estados[i].energy - estados[base_idx].energy;
            if ea > best_ea {
                best_ea = ea;
                best_k = idx + 1;
                best_i = i;
            }
        }
        c.t(
            xs[best_i],
            y0 + h - 6.0,
            &format!("etapa {best_k}: determinante"),
            9.0,
            400,
            "textSub",
            "middle",
            SANS,
        );
    }
    Ok((x0 + w + 130.0, y0 + h + 26.0))
}

// --- 6. Pila galvánica ----------------------------------------------------------------------------------

fn render_galvanic_cell(c: &mut Lienzo, b: &Bloque) -> Result<Punto, ErrorDiagrama> {
    let semis = if let Some(hc_val) = b.get("halfCells").filter(|v| es_truthy_val(v)) {
        let Some(arr) = hc_val.as_array() else {
            return Err(ErrorDiagrama::new(
                "galvanic_cell: halfCells (dos semiceldas {couple, E0, n})",
            ));
        };
        if arr.len() != 2 || arr[0].is_null() || arr[1].is_null() {
            return Err(ErrorDiagrama::new(
                "galvanic_cell: halfCells (dos semiceldas {couple, E0, n})",
            ));
        }
        vec![
            envolver_error("galvanic_cell", chem::semicelda(&arr[0], "halfCells[0]"))?,
            envolver_error("galvanic_cell", chem::semicelda(&arr[1], "halfCells[1]"))?,
        ]
    } else if let (Some(an), Some(ca)) = (
        b.get("anode").filter(|v| !v.is_null()),
        b.get("cathode").filter(|v| !v.is_null()),
    ) {
        vec![
            envolver_error("galvanic_cell", chem::semicelda(an, "halfCells[0]"))?,
            envolver_error("galvanic_cell", chem::semicelda(ca, "halfCells[1]"))?,
        ]
    } else {
        return Err(ErrorDiagrama::new(
            "galvanic_cell: halfCells (dos semiceldas {couple, E0, n})",
        ));
    };

    let p = envolver_error("galvanic_cell", chem::pila(semis))?;
    let show_delta_g = booleano(b, "showDeltaG", false);
    if !p.e0.is_finite()
        || !p.e.is_none_or(f64::is_finite)
        || (show_delta_g && !p.delta_g().is_finite())
    {
        return Err(ErrorDiagrama::new(
            "galvanic_cell: valores fuera de rango numérico",
        ));
    }
    let top = encabezado_q(c, b);
    let trazo = c.pal.stroke_main;
    let (y_cable, y_boca, y_fondo, y_liq) = (top + 34.0, top + 110.0, top + 240.0, top + 140.0);
    let vasos = [(50.0_f64, 200.0_f64, &p.anodo), (350.0_f64, 500.0_f64, &p.catodo)];
    for &(xa, xb, _) in &vasos {
        c.add(format!(
            r#"<rect x="{xa:.1}" y="{y_liq:.1}" width="{:.1}" height="{:.1}" fill="{}" opacity="0.6"/><path d="M {xa:.1} {y_boca:.1} L {xa:.1} {y_fondo:.1} L {xb:.1} {y_fondo:.1} L {xb:.1} {y_boca:.1}" fill="none" stroke="{trazo}" stroke-width="1.8"/>"#,
            xb - xa,
            y_fondo - y_liq,
            c.pal.fill_highlight
        ));
    }
    let x_elecs = [95.0_f64, 455.0_f64];
    for (&(xa, xb, s), &x_elec) in vasos.iter().zip(x_elecs.iter()) {
        c.r(
            x_elec - 9.0,
            y_cable + 24.0,
            18.0,
            y_fondo - 30.0 - (y_cable + 24.0),
            "fillPointerBox",
            1.4,
            0.0,
        );
        tq(
            c,
            x_elec,
            y_cable + 20.0,
            &s.electrodo,
            11.0,
            700,
            "textMain",
            "middle",
            SANS,
            false,
        );
        tq(
            c,
            (xa + xb) / 2.0 + 22.0,
            y_fondo - 14.0,
            &s.solucion,
            9.5,
            400,
            "textMain",
            "middle",
            SANS,
            false,
        );
    }
    let (xv, rv) = (275.0_f64, 17.0_f64);
    let (c0, c1) = (x_elecs[0], x_elecs[1]);
    c.add(format!(
        r#"<path d="M {c0:.1} {:.1} L {c0:.1} {y_cable:.1} L {:.1} {y_cable:.1} M {:.1} {y_cable:.1} L {c1:.1} {y_cable:.1} L {c1:.1} {:.1}" fill="none" stroke="{trazo}" stroke-width="1.6"/><circle cx="{xv:.1}" cy="{y_cable:.1}" r="{rv:.1}" fill="{}" stroke="{trazo}" stroke-width="1.6"/>"#,
        y_cable + 8.0,
        xv - rv,
        xv + rv,
        y_cable + 8.0,
        c.pal.fill_cell
    ));
    c.t(xv, y_cable + 5.0, "V", 13.0, 700, "textMain", "middle", SANS);
    let blanco_e = booleano(b, "blankPotential", false);
    rotulo_q(
        c,
        xv,
        y_cable - 24.0,
        "E°",
        Some(&format!("{} V", coma(p.e0, 2))),
        blanco_e,
        "middle",
        10.0,
    );
    if booleano(b, "blankFlow", false) {
        caja_vacia(c, (c0 + xv) / 2.0, y_cable - 12.0, 54.0, 13.0);
    } else {
        c.linea(
            c0 + 30.0,
            y_cable - 10.0,
            xv - rv - 14.0,
            y_cable - 10.0,
            "accentPrimary",
            1.4,
            Some("main"),
            "",
        );
        c.t(
            (c0 + xv) / 2.0,
            y_cable - 15.0,
            "e⁻",
            10.0,
            700,
            "textMain",
            "middle",
            SANS,
        );
        c.linea(
            xv + rv + 14.0,
            y_cable - 10.0,
            c1 - 30.0,
            y_cable - 10.0,
            "accentPrimary",
            1.4,
            Some("main"),
            "",
        );
    }

    let (sal, cation, anion) = match b.get("saltBridge") {
        Some(Value::Object(sb)) => (
            sb.get("name")
                .filter(|v| !v.is_null())
                .map(valor_str)
                .unwrap_or_else(|| "KNO3".to_string()),
            sb.get("cation")
                .filter(|v| !v.is_null())
                .map(valor_str)
                .unwrap_or_else(|| "K^+".to_string()),
            sb.get("anion")
                .filter(|v| !v.is_null())
                .map(valor_str)
                .unwrap_or_else(|| "NO3^-".to_string()),
        ),
        Some(v) if !v.is_null() => (valor_str(v), "K^+".to_string(), "NO3^-".to_string()),
        _ => ("KNO3".to_string(), "K^+".to_string(), "NO3^-".to_string()),
    };
    let (xi, xd, yb) = (170.0_f64, 380.0_f64, top + 76.0);
    let camino = format!(
        "M {xi:.1} {:.1} L {xi:.1} {yb:.1} L {xd:.1} {yb:.1} L {xd:.1} {:.1}",
        y_liq + 34.0,
        y_liq + 34.0
    );
    c.add(format!(
        r#"<path d="{camino}" fill="none" stroke="{trazo}" stroke-width="15" stroke-linejoin="round"/><path d="{camino}" fill="none" stroke="{}" stroke-width="11.4" stroke-linejoin="round"/>"#,
        c.pal.fill_cell
    ));
    tq(
        c,
        (xi + xd) / 2.0,
        yb + 22.0,
        &format!("puente salino ({sal})"),
        9.5,
        400,
        "textSub",
        "middle",
        SANS,
        false,
    );
    if booleano(b, "showIons", true) {
        c.linea(
            (xi + xd) / 2.0 - 12.0,
            yb,
            xi + 24.0,
            yb,
            "strokeSub",
            1.1,
            Some("sub"),
            "",
        );
        tq(
            c,
            (xi + xd) / 2.0 - 40.0,
            yb - 11.0,
            &anion,
            9.0,
            400,
            "textMain",
            "middle",
            SANS,
            false,
        );
        c.linea(
            (xi + xd) / 2.0 + 12.0,
            yb,
            xd - 24.0,
            yb,
            "strokeSub",
            1.1,
            Some("sub"),
            "",
        );
        tq(
            c,
            (xi + xd) / 2.0 + 40.0,
            yb - 11.0,
            &cation,
            9.0,
            400,
            "textMain",
            "middle",
            SANS,
            false,
        );
    }

    let mut y = y_fondo + 20.0;
    let blanco_p = booleano(b, "blankPolarity", false);
    let blanco_r = booleano(b, "blankReactions", false);
    let lados = [
        (vasos[0], "Ánodo (−)", "oxidación", p.anodo.oxidacion()),
        (
            vasos[1],
            "Cátodo (+)",
            "reducción",
            p.catodo.reduccion.clone(),
        ),
    ];
    for ((xa, xb, s), nombre, proceso, reaccion) in lados {
        let xm = (xa + xb) / 2.0;
        if blanco_p {
            caja_vacia(c, xm, y - 4.0, 90.0, 15.0);
            caja_vacia(c, xm, y + 13.0, 70.0, 13.0);
        } else {
            c.t(xm, y, nombre, 11.0, 700, "textMain", "middle", SANS);
            c.t(xm, y + 15.0, proceso, 10.0, 400, "textSub", "middle", SANS);
        }
        if blanco_r {
            c.vacio(xm - 80.0, xm + 80.0, y + 34.0);
        } else {
            tq(
                c, xm, y + 32.0, &reaccion, 10.0, 400, "textMain", "middle", SANS, false,
            );
        }
        tq(
            c,
            xm,
            y + 48.0,
            &format!("E° = {} V", coma(s.e0, 2)),
            9.0,
            400,
            "textSub",
            "middle",
            SANS,
            false,
        );
    }
    y += 72.0;
    if booleano(b, "showNotation", true) {
        c.t(0.0, y, "Notación:", 10.0, 700, "textMain", "start", SANS);
        if booleano(b, "blankNotation", false) {
            c.vacio(66.0, 470.0, y + 2.0);
        } else {
            tq(
                c,
                66.0,
                y,
                &p.notacion(),
                10.0,
                400,
                "textMain",
                "start",
                SANS,
                false,
            );
        }
        y += 18.0;
    }
    if let Some(e_nernst) = p.e {
        rotulo_q(
            c,
            0.0,
            y,
            "E (Nernst, 25 °C)",
            Some(&format!("{} V", coma(e_nernst, 3))),
            blanco_e,
            "start",
            10.0,
        );
        y += 18.0;
    }
    if show_delta_g {
        rotulo_q(
            c,
            0.0,
            y,
            "ΔG°",
            Some(&format!("{} kJ/mol", fmt_g(Some(p.delta_g()), 4))),
            blanco_e,
            "start",
            10.0,
        );
        y += 18.0;
    }
    Ok((560.0, y))
}

// --- 7. Celda unidad -------------------------------------------------------------------------------------

fn nombre_red(red: &str) -> &'static str {
    match red {
        "sc" => "cúbica simple",
        "bcc" => "cúbica centrada en el cuerpo",
        "fcc" => "cúbica centrada en las caras",
        "nacl" => "tipo NaCl",
        "cscl" => "tipo CsCl",
        _ => "",
    }
}

fn desglose(sitios: &[(f64, f64, f64, &'static str)], especie: &str) -> String {
    // Clave entera proporcional a la fracción: 1/8 -> 1, 1/4 -> 2, 1/2 -> 4, 1 -> 8.
    let mut cuentas: BTreeMap<u8, usize> = BTreeMap::new();
    for &(x, y, z, e) in sitios {
        if e == especie {
            let fr = chem::fraccion(x, y, z);
            let clave = (fr * 8.0).round() as u8;
            *cuentas.entry(clave).or_insert(0) += 1;
        }
    }
    let partes: Vec<String> = cuentas
        .into_iter()
        .map(|(clave, n)| match clave {
            1 => format!("{n} × 1/8"),
            2 => format!("{n} × 1/4"),
            4 => format!("{n} × 1/2"),
            8 => format!("{n}"),
            other => format!("{n} × {other}/8"),
        })
        .collect();
    partes.join(" + ")
}

fn render_unit_cell(c: &mut Lienzo, b: &Bloque) -> Result<Punto, ErrorDiagrama> {
    let red = b
        .get("lattice")
        .filter(|v| !v.is_null())
        .map(valor_str)
        .unwrap_or_else(|| "fcc".to_string());
    let celda = envolver_error("unit_cell", chem::celda_unidad(&red))?;
    let top = encabezado_q(c, b);
    let a = 150.0_f64;
    let k = 0.42_f64;
    let (ox, oy) = (30.0_f64, top + 30.0 + a + a * k * 0.5);
    let (cos30, sin30) = (30.0_f64.to_radians().cos(), 30.0_f64.to_radians().sin());
    let proy = |x: f64, y: f64, z: f64| -> Punto {
        (ox + a * x + a * k * z * cos30, oy - a * y - a * k * z * sin30)
    };

    let mut vertices = Vec::with_capacity(8);
    for x in [0u8, 1] {
        for y in [0u8, 1] {
            for z in [0u8, 1] {
                vertices.push((x, y, z));
            }
        }
    }
    let oculto = (0u8, 0u8, 1u8);
    for &p1 in &vertices {
        for &p2 in &vertices {
            let manhattan = p1.0.abs_diff(p2.0) + p1.1.abs_diff(p2.1) + p1.2.abs_diff(p2.2);
            if p1 < p2 && manhattan == 1 {
                let (x1, y1) = proy(f64::from(p1.0), f64::from(p1.1), f64::from(p1.2));
                let (x2, y2) = proy(f64::from(p2.0), f64::from(p2.1), f64::from(p2.2));
                let atras = p1 == oculto || p2 == oculto;
                c.linea(
                    x1,
                    y1,
                    x2,
                    y2,
                    if atras { "strokeMuted" } else { "strokeMain" },
                    if atras { 1.0 } else { 1.4 },
                    None,
                    if atras { "4 3" } else { "" },
                );
            }
        }
    }

    let dos = celda.z.len() > 1;
    let r_a = a * if dos { 0.12 } else { 0.1 };
    let r_b = a * 0.075;
    let mut sitios = celda.sitios.clone();
    sitios.sort_by(|s1, s2| (-s1.2).total_cmp(&(-s2.2)));
    for &(sx, sy, sz, especie) in &sitios {
        let (px, py) = proy(sx, sy, sz);
        let relleno = if especie == "A" {
            c.pal.fill_highlight
        } else {
            c.pal.stroke_sub
        };
        let r = if especie == "A" { r_a } else { r_b };
        let opacidad = if sz == 1.0 && (sx, sy) != (1.0, 1.0) {
            0.55
        } else {
            1.0
        };
        c.add(format!(
            r#"<circle cx="{px:.1}" cy="{py:.1}" r="{r:.1}" fill="{relleno}" stroke="{}" stroke-width="1.1" opacity="{}"/>"#,
            c.pal.stroke_main,
            fmt_num_clean(opacidad)
        ));
    }

    let elem = b
        .get("element")
        .filter(|v| !v.is_null())
        .map(valor_str)
        .unwrap_or_else(|| "M".to_string());
    let nombres: Vec<String> = match opt_lista(b, "species", "unit_cell.species")? {
        Some(arr) if !arr.is_empty() => arr.iter().map(valor_str).collect(),
        _ => match celda.red {
            "nacl" => vec!["Cl^-".to_string(), "Na^+".to_string()],
            "cscl" => vec!["Cl^-".to_string(), "Cs^+".to_string()],
            _ => vec![elem],
        },
    };
    let x = 270.0_f64;
    let mut y = top + 34.0;
    let blancos: HashSet<String> = if es_bool_true(b, "blank") {
        ["z", "coordination", "packing", "density"]
            .into_iter()
            .map(str::to_string)
            .collect()
    } else {
        match b.get("blank") {
            Some(Value::Array(arr)) => arr.iter().map(valor_str).collect(),
            _ => HashSet::new(),
        }
    };

    c.t(
        x,
        y,
        &format!("Red {}", nombre_red(celda.red)),
        11.0,
        700,
        "textMain",
        "start",
        SANS,
    );
    y += 22.0;
    for (i, (&especie, &z_val)) in celda.z.iter().enumerate() {
        let nombre = nombres.get(i).map(String::as_str).unwrap_or(especie);
        tq(
            c,
            x,
            y,
            &format!("{nombre}:"),
            10.5,
            700,
            "textMain",
            "start",
            SANS,
            false,
        );
        if blancos.contains("z") {
            c.vacio(x + 44.0, x + 230.0, y + 2.0);
        } else {
            c.t(
                x + 44.0,
                y,
                &format!(
                    "{} = {}",
                    desglose(&celda.sitios, especie),
                    fmt_g(Some(z_val), 3)
                ),
                10.5,
                400,
                "textMain",
                "start",
                SANS,
            );
        }
        y += 18.0;
    }
    if dos {
        let n0 = nombres.first().map(String::as_str).unwrap_or("A");
        let n1 = nombres.get(1).map(String::as_str).unwrap_or("B");
        c.add(format!(
            r#"<circle cx="{:.1}" cy="{:.1}" r="5" fill="{}" stroke="{}"/>"#,
            x + 4.0,
            y - 3.0,
            c.pal.fill_highlight,
            c.pal.stroke_main
        ));
        tq(
            c,
            x + 14.0,
            y + 1.0,
            n0,
            9.5,
            400,
            "textMain",
            "start",
            SANS,
            false,
        );
        c.add(format!(
            r#"<circle cx="{:.1}" cy="{:.1}" r="4" fill="{}" stroke="{}"/>"#,
            x + 64.0,
            y - 3.0,
            c.pal.stroke_sub,
            c.pal.stroke_main
        ));
        tq(
            c,
            x + 74.0,
            y + 1.0,
            n1,
            9.5,
            400,
            "textMain",
            "start",
            SANS,
            false,
        );
        y += 20.0;
    }
    y += 4.0;
    rotulo_q(
        c,
        x,
        y,
        "Número de coordinación",
        Some(celda.coordinacion),
        blancos.contains("coordination"),
        "start",
        10.5,
    );
    y += 18.0;
    if let Some(emp) = celda.empaquetamiento {
        rotulo_q(
            c,
            x,
            y,
            "Empaquetamiento",
            Some(&format!("{} %", fmt_g(Some(100.0 * emp), 3))),
            blancos.contains("packing"),
            "start",
            10.5,
        );
        y += 18.0;
    }
    c.t(x, y, celda.relacion, 10.5, 400, "textSub", "start", SANS);
    y += 18.0;
    if let (Some(a_val), Some(m_val)) = (b.get("a"), b.get("molarMass")) {
        let a_pm = envolver_error("unit_cell", chem::num(a_val, "a (pm)", true, None))?;
        let molar = envolver_error(
            "unit_cell",
            chem::num(m_val, "molarMass (g/mol)", true, None),
        )?;
        let rho = chem::densidad(&celda, a_pm, molar);
        if !rho.is_finite() {
            return Err(ErrorDiagrama::new(
                "unit_cell: densidad fuera de rango numérico",
            ));
        }
        c.t(
            x,
            y,
            &format!(
                "a = {} pm; M = {} g/mol",
                fmt_g(Some(a_pm), 4),
                fmt_g(Some(molar), 4)
            ),
            10.0,
            400,
            "textSub",
            "start",
            SANS,
        );
        y += 18.0;
        rotulo_q(
            c,
            x,
            y,
            "ρ",
            Some(&format!("{} g/cm³", coma(rho, 2))),
            blancos.contains("density"),
            "start",
            10.5,
        );
        y += 18.0;
    }
    Ok((540.0, (oy + 16.0).max(y)))
}

// --- Tests unitarios y de errores -----------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use dibu_core::leer_bloques;

    fn registro_m1b1() -> Registro {
        let mut reg = Registro::new();
        registrar(&mut reg);
        reg
    }

    fn render_yaml(yaml: &str, tinta: bool) -> Result<String, ErrorDiagrama> {
        let reg = registro_m1b1();
        let bloques = leer_bloques(yaml)?;
        reg.renderizar(&bloques[0], tinta)
    }

    /// Extrae el texto visible dentro de `<text>...</text>` eliminando etiquetas `<tspan...>` y
    /// reemplazando NBSP por espacio normal para verificaciones semánticas equivalentes a `_texto` en Python.
    fn extraer_texto_svg(svg: &str) -> String {
        let mut salida = String::new();
        let mut resto = svg;
        while let Some(ini) = resto.find("<text ") {
            resto = &resto[ini..];
            let Some(gt) = resto.find('>') else { break };
            resto = &resto[gt + 1..];
            let Some(fin) = resto.find("</text>") else { break };
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
        assert!(!contiene_palabra_no_finita(&lower, "nan"), "contiene NaN: {svg}");
        assert!(!contiene_palabra_no_finita(&lower, "inf"), "contiene inf: {svg}");
    }

    #[test]
    fn registra_los_7_tipos_exactos() {
        let reg = registro_m1b1();
        assert_eq!(
            reg.tipos(),
            vec![
                "energy_profile",
                "galvanic_cell",
                "molecule",
                "orbitals",
                "reaction",
                "unit_cell",
                "vsepr",
            ]
        );
    }

    #[test]
    fn fixtures_estructurales_ambas_paletas_validos() {
        let yaml = include_str!("../../../../fixtures/diagramas_quimica.yaml");
        let bloques = leer_bloques(yaml).unwrap();
        let reg = registro_m1b1();
        let estructurales: Vec<&Bloque> = bloques
            .iter()
            .filter(|b| {
                b.get("type")
                    .and_then(Value::as_str)
                    .is_some_and(|t| reg.contiene(t))
            })
            .collect();
        assert_eq!(
            estructurales.len(),
            29,
            "se esperaban 29 bloques de los 7 tipos estructurales en fixtures/diagramas_quimica.yaml"
        );
        for b in estructurales {
            for tinta in [true, false] {
                let svg = reg.renderizar(b, tinta).unwrap();
                assert_svg_valido(&svg);
            }
        }
    }

    #[test]
    fn errores_de_borde_y_cotas_o1() {
        // Rechaza lonePairs/radical gigantes o negativos (evita bucles O(n) sin cota)
        assert!(render_yaml(
            "type: molecule\natoms:\n  - {id: a, el: O, at: [0, 0], lonePairs: 1000000}",
            true
        )
        .is_err());
        assert!(render_yaml(
            "type: molecule\natoms:\n  - {id: a, el: O, at: [0, 0], radical: -1}",
            true
        )
        .is_err());

        // Auto-enlace y auto-flecha no producen NaN/Inf
        let self_mol = render_yaml(
            r#"
type: molecule
atoms:
  - {id: a, el: C, at: [0, 0]}
bonds:
  - [a, a, 1]
arrows:
  - {from: a, to: a, half: true}
"#,
            true,
        )
        .unwrap();
        assert_svg_valido(&self_mol);

        // Errores adicionales de molecule, orbitals y unit_cell
        assert!(render_yaml(
            "type: molecule\nrings:\n  - {size: 6, ids: [a, b]}",
            true
        )
        .unwrap_err()
        .0
        .contains("tienen que ser 6"));
        assert!(render_yaml(
            "type: molecule\natoms:\n  - {id: a, el: C}\n  - {id: b, el: O}",
            true
        )
        .unwrap_err()
        .0
        .contains("falta at"));
        assert!(render_yaml(
            "type: molecule\natoms:\n  - {id: a, el: C}\narrows:\n  - {from: a}",
            true
        )
        .unwrap_err()
        .0
        .contains("arrows[0]"));
        assert!(render_yaml("type: orbitals\nkind: crystal", true)
            .unwrap_err()
            .0
            .contains("kind 'crystal'"));
    }

    // --- 1. molecule ---

    #[test]
    fn molecule_lewis_agua_nitrato_naftaleno_estereo_y_flechas() {
        let agua = render_yaml(
            r#"
type: molecule
title: Estructura de Lewis del agua
atoms:
  - {id: o, el: O, at: [0, 0], lonePairs: 2}
  - {id: h1, el: H, from: o, angle: 218}
  - {id: h2, el: H, from: o, angle: 322}
"#,
            true,
        )
        .unwrap();
        assert_svg_valido(&agua);
        assert!(agua.contains("<circle"));
        let txt = extraer_texto_svg(&agua);
        assert!(txt.contains("Estructura de Lewis del agua") && txt.contains('O') && txt.contains('H'));

        let nitrato = render_yaml(
            r#"
type: molecule
title: Una estructura resonante del nitrato
atoms:
  - {id: n, el: N, at: [0, 0], charge: 1}
  - {id: o1, el: O, from: n, angle: 90, bond: 2, lonePairs: 2}
  - {id: o2, el: O, from: n, angle: 210, lonePairs: 3, charge: -1}
  - {id: o3, el: O, from: n, angle: 330, lonePairs: 3, charge: -1}
"#,
            true,
        )
        .unwrap();
        let txt_n = extraer_texto_svg(&nitrato);
        assert!(txt_n.contains('+') && txt_n.contains('−'));

        // Naftaleno Kekulé: fusión de anillos (10 átomos, 11 enlaces, 5 dobles)
        let bloques_naf = leer_bloques(
            r#"
type: molecule
skeletal: true
rings:
  - {size: 6, center: [0, 0], start: 30, kekule: true}
  - {size: 6, center: [1.732, 0], start: 30}
bonds:
  - [r1_1, r1_2, 2]
  - [r1_5, r1_0, 2]
"#,
        )
        .unwrap();
        let m_naf = Molecula::from_bloque(&bloques_naf[0], "molecule").unwrap();
        assert_eq!(m_naf.atomos.len(), 10);
        assert_eq!(m_naf.enlaces.len(), 11);
        assert_eq!(
            m_naf.enlaces.iter().filter(|e| e.orden == 2.0).count(),
            5
        );

        let estereo = render_yaml(
            r#"
type: molecule
title: Un carbono quiral con cuña y rayas
caption: "Quiral (C*)"
atoms:
  - {id: c, el: C, at: [0, 0]}
  - {id: br, el: Br, from: c, angle: 90}
  - {id: h, el: H, from: c, angle: 210, bond: {stereo: hash}}
  - {id: cl, el: Cl, from: c, angle: 330, bond: {stereo: wedge}}
  - {id: f, el: F, from: c, angle: 270, length: 0.9, bond: {order: 1}}
"#,
            true,
        )
        .unwrap();
        assert!(estereo.contains("<polygon"));
        assert!(extraer_texto_svg(&estereo).contains("Quiral (C*)"));

        let sn2 = render_yaml(
            r#"
type: molecule
title: "S_N\\2 (flechas curvas)"
atoms:
  - {id: oh, el: HO, at: [-2.2, 0], charge: -1, lonePairs: 2}
  - {id: c, el: C, at: [0, 0]}
  - {id: br, el: Br, from: c, angle: 0, lonePairs: 3}
  - {id: h1, el: H, from: c, angle: 90}
  - {id: h2, el: H, from: c, angle: 225}
  - {id: h3, el: H, from: c, angle: 135, length: 0.8}
arrows:
  - {from: oh, to: c, bend: 1}
  - {from: [c, br], to: br, bend: -1, half: true}
"#,
            false,
        )
        .unwrap();
        assert!(sn2.contains("Q "));
        assert!(sn2.contains("#1d4ed8"));
        assert!(texto_plano(&sn2).contains("SN2"));
    }

    #[test]
    fn molecule_blank_skeletal_carga_literal_y_errores() {
        let blank_mol = render_yaml(
            r#"
type: molecule
inkSaver: true
atoms:
  - {id: a, el: C, at: [0, 0], blank: true, radical: 1, note: "α-C"}
  - {id: b, el: O, from: a, angle: 0, bond: {order: 0.5}}
  - {id: c, el: N, from: b, angle: 0, bond: {order: 1.5}}
  - {id: d, el: H, from: c, angle: 0, bond: {blank: true}}
"#,
            false,
        )
        .unwrap();
        assert!(blank_mol.contains(r#"stroke-dasharray="3 2""#));
        assert!(blank_mol.contains(r#"stroke-dasharray="2 3""#));
        assert!(blank_mol.contains(r#"stroke-dasharray="4 3""#));
        assert!(!blank_mol.contains("#1d4ed8"));
        assert!(extraer_texto_svg(&blank_mol).contains("α-C"));

        let ring_upd = render_yaml(
            r#"
type: molecule
rings:
  - {size: 5, ids: [c1, c2, c3, c4, c5], el: [N, C, C, C, C]}
atoms:
  - {id: c1, el: NH, charge: "+1"}
caption: "H2O + NH4^+"
"#,
            true,
        )
        .unwrap();
        let txt = texto_plano(&ring_upd);
        assert!(txt.contains("NH+1") || txt.contains("+1"));
        assert!(txt.contains("H2O+NH4+"));

        let skel = render_yaml(
            r#"
type: molecule
skeletal: true
atoms:
  - {id: c1, el: C, at: [0, 0]}
  - {id: o1, el: O, at: [2, 0]}
  - {id: c2, el: C, from: o1, angle: 0, bond: false, show: true}
  - {id: c3, el: C, from: o1, angle: 180, note: "oculto"}
"#,
            true,
        )
        .unwrap();
        assert!(!extraer_texto_svg(&skel).contains("oculto"));

        // Errores esperados
        assert!(render_yaml("type: molecule", true)
            .unwrap_err()
            .0
            .contains("faltan atoms"));
        assert!(render_yaml("type: molecule\nrings:\n  - {size: 9}", true)
            .unwrap_err()
            .0
            .contains("de 3 a 8 átomos"));
        assert!(render_yaml(
            "type: molecule\natoms:\n  - {id: a, el: C}\n  - {id: b, from: z}",
            true
        )
        .unwrap_err()
        .0
        .contains("no hay un átomo"));
        assert!(render_yaml(
            "type: molecule\natoms:\n  - {el: C}\n  - {from: a0, angle: 0}\nbonds:\n  - [a0, a1, 4]",
            true
        )
        .unwrap_err()
        .0
        .contains("bonds[0].order"));
        assert!(render_yaml(
            "type: molecule\natoms:\n  - {el: C}\n  - {from: a0, angle: 0, bond: {stereo: bad}}",
            true
        )
        .unwrap_err()
        .0
        .contains("stereo"));
    }

    // --- 2. reaction ---

    #[test]
    fn reaction_variantes_wrap_blank_y_errores() {
        let est = render_yaml(
            r#"
type: reaction
title: Esterificación de Fischer
steps:
  - "CH3COOH"
  - "+"
  - "CH3CH2OH"
  - {arrow: "<=>", above: "H2SO4", below: "Δ"}
  - "CH3COOCH2CH3"
  - "+"
  - "H2O"
"#,
            true,
        )
        .unwrap();
        assert_svg_valido(&est);
        let txt = texto_plano(&est);
        assert!(txt.contains("CH3COOH") && txt.contains("H2SO4") && txt.contains('Δ'));

        let res = render_yaml(
            r#"
type: reaction
steps:
  - atoms:
      - {id: c, el: C, at: [0, 0]}
      - {id: r, el: R, from: c, angle: 180}
      - {id: o1, el: O, from: c, angle: 60, bond: 2}
  - {arrow: "<->"}
  - {formula: "RCO2^-"}
  - {arrow: "=>", above: "FGI"}
  - {blank: true, width: 60}
"#,
            false,
        )
        .unwrap();
        assert!(res.contains(r#"marker-start="url(#flecha-main-"#));
        assert!(res.contains("#1d4ed8"));

        let wrap = render_yaml(
            r#"
type: reaction
maxWidth: 160
steps: ["CH4", "+", "2 O2", {arrow: "->"}, "CO2", "+", "2 H2O"]
"#,
            true,
        )
        .unwrap();
        assert_svg_valido(&wrap);

        // Errores esperados
        assert!(render_yaml("type: reaction\nsteps: []", true)
            .unwrap_err()
            .0
            .contains("'steps' es una lista"));
        assert!(render_yaml("type: reaction\nsteps: [true]", true)
            .unwrap_err()
            .0
            .contains("una fórmula, '+'"));
        assert!(render_yaml("type: reaction\nsteps:\n  - {arrow: \"~>\"}", true)
            .unwrap_err()
            .0
            .contains("steps[0].arrow"));
        assert!(render_yaml(
            "type: reaction\nsteps:\n  - atoms:\n      - {id: a, el: C}\n      - {id: b, from: x}",
            true
        )
        .unwrap_err()
        .0
        .contains("reaction.steps[0]: no hay un átomo"));
    }

    // --- 3. vsepr ---

    #[test]
    fn vsepr_geometrias_blank_y_errores() {
        let nh3 = render_yaml(
            r#"
type: vsepr
title: Amoníaco
central: N
ligands: [H, H, H]
lonePairs: 1
"#,
            true,
        )
        .unwrap();
        assert_svg_valido(&nh3);
        let txt = extraer_texto_svg(&nh3);
        assert!(txt.contains("piramidal trigonal") && txt.contains("107") && txt.contains("sp³"));
        assert!(texto_plano(&nh3).contains("AX3E"));

        let sf4 = render_yaml(
            r#"
type: vsepr
central: S
ligands: [F, F, F, F]
lonePairs: 1
"#,
            false,
        )
        .unwrap();
        assert!(sf4.contains("#1d4ed8"));
        assert!(extraer_texto_svg(&sf4).contains("balancín"));

        let xef4 = render_yaml(
            r#"
type: vsepr
central: Xe
ligands: [F, F, F, F]
lonePairs: 2
blank: [name, angle, hybridization]
"#,
            true,
        )
        .unwrap();
        let txt_xe = extraer_texto_svg(&xef4);
        assert!(!txt_xe.contains("cuadrada plana"));
        assert!(!txt_xe.contains("sp³d²"));
        assert!(xef4.contains(r#"stroke-dasharray="3 2""#));

        let sin_hyb = render_yaml(
            r#"
type: vsepr
geometry: trigonal_planar
central: B
ligands: [F, F, F]
showAngle: false
showHybridization: false
"#,
            true,
        )
        .unwrap();
        assert!(!extraer_texto_svg(&sin_hyb).contains("Hibridación:"));

        let lin = render_yaml(
            r#"
type: vsepr
geometry: linear_tbp
central: I
ligands: [Cl, Cl]
"#,
            true,
        )
        .unwrap();
        assert!(!lin.contains(" A 26 26 "));

        // Errores esperados
        assert!(render_yaml("type: vsepr\nligands: [F, F, F, F, F, F, F]", true)
            .unwrap_err()
            .0
            .contains("geometría conocida"));
        assert!(render_yaml("type: vsepr\ngeometry: pentagonal", true)
            .unwrap_err()
            .0
            .contains("geometry 'pentagonal'"));
        assert!(
            render_yaml("type: vsepr\ngeometry: tetrahedral\nligands: [H, H, H]", true)
                .unwrap_err()
                .0
                .contains("lleva 4 ligandos")
        );
    }

    // --- 4. orbitals ---

    #[test]
    fn orbitals_atomicos_moleculares_y_errores() {
        let cr = render_yaml(
            r#"
type: orbitals
element: Cr
core: true
showUnpaired: true
"#,
            true,
        )
        .unwrap();
        let txt_cr = extraer_texto_svg(&cr);
        assert!(txt_cr.contains("[Ar] 4s¹ 3d⁵") && txt_cr.contains("Electrones desapareados = 6"));

        let h_plus = render_yaml(
            r#"
type: orbitals
element: H
charge: 1
core: true
showUnpaired: true
"#,
            true,
        )
        .unwrap();
        assert!(!extraer_texto_svg(&h_plus).contains("[He]"));
        assert!(extraer_texto_svg(&h_plus).contains("Electrones desapareados = 0"));

        let cu_blank = render_yaml(
            r#"
type: orbitals
element: Cu
charge: 2
core: true
blank: true
showUnpaired: true
"#,
            true,
        )
        .unwrap();
        assert!(!extraer_texto_svg(&cu_blank).contains("3d⁹"));

        let o2_anion = render_yaml(
            r#"
type: orbitals
kind: molecular
molecule: O
charge: -1
blankElectrons: true
blankBondOrder: true
blankMagnetism: true
inkSaver: true
"#,
            false,
        )
        .unwrap();
        assert!(!o2_anion.contains("#1d4ed8"));
        assert!(!extraer_texto_svg(&o2_anion).contains("1,5"));

        let co = render_yaml(
            r#"
type: orbitals
kind: molecular
atoms: [C, O]
"#,
            false,
        )
        .unwrap();
        assert!(texto_plano(&co).contains("Ordendeenlace=3"));
        assert!(extraer_texto_svg(&co).contains("diamagnética"));

        // Errores esperados
        assert!(render_yaml("type: orbitals", true)
            .unwrap_err()
            .0
            .contains("falta element"));
        assert!(render_yaml("type: orbitals\nelement: Xx", true)
            .unwrap_err()
            .0
            .contains("desconocido"));
        assert!(render_yaml("type: orbitals\nelement: H\ncharge: 2", true)
            .unwrap_err()
            .0
            .contains("no tiene electrones"));
        assert!(
            render_yaml("type: orbitals\nkind: molecular\natoms: [Na, Na]", true)
                .unwrap_err()
                .0
                .contains("dos primeros períodos")
        );
    }

    // --- 5. energy_profile ---

    #[test]
    fn energy_profile_etapas_tie_break_borde_y_errores() {
        let p1 = render_yaml(
            r#"
type: energy_profile
title: Reacción exotérmica en una etapa
states:
  - {label: "A + B", energy: 0}
  - {label: "ET‡", energy: 85, ts: true}
  - {label: "C + D", energy: -40}
catalyzed:
  - {energy: 0}
  - {energy: 45, ts: true}
  - {energy: -40}
"#,
            true,
        )
        .unwrap();
        let txt1 = extraer_texto_svg(&p1);
        assert!(txt1.contains("Ea = 85 kJ/mol"));
        assert!(txt1.contains("ΔH = -40 kJ/mol"));
        assert!(txt1.contains("(exotérmica)"));

        // Empate en Ea: gana la primera etapa (igual que max() de Python)
        let empate = render_yaml(
            r#"
type: energy_profile
states:
  - {label: R, energy: 0}
  - {label: ET1, energy: 50, ts: true}
  - {label: Int, energy: 10}
  - {label: ET2, energy: 60, ts: true}
  - {label: P, energy: 0}
showRateDetermining: true
"#,
            true,
        )
        .unwrap();
        assert!(extraer_texto_svg(&empate).contains("etapa 1: determinante"));

        // Energías iguales -> sin división por cero
        let plano = render_yaml(
            r#"
type: energy_profile
states:
  - {label: A, energy: 10}
  - {label: B, energy: 10}
showEnergies: true
"#,
            true,
        )
        .unwrap();
        assert_svg_valido(&plano);
        assert!(extraer_texto_svg(&plano).contains("ΔH = 0 kJ/mol"));

        // Errores esperados
        assert!(
            render_yaml("type: energy_profile\nstates:\n  - {energy: 0}", true)
                .unwrap_err()
                .0
                .contains("al menos dos estados")
        );
        assert!(render_yaml(
            "type: energy_profile\nstates:\n  - {label: A}\n  - {energy: 10}",
            true
        )
        .unwrap_err()
        .0
        .contains("states[0] es {label, energy, ts}"));
        assert!(render_yaml(
            "type: energy_profile\nstates:\n  - {energy: 0}\n  - {energy: 10}\ncatalyzed:\n  - {energy: 0}",
            true
        )
        .unwrap_err()
        .0
        .contains("catalyzed es una lista de al menos dos estados"));
    }

    // --- 6. galvanic_cell ---

    #[test]
    fn galvanic_cell_daniell_blank_concentracion_y_errores() {
        let daniell = render_yaml(
            r#"
type: galvanic_cell
title: Pila de Daniell
halfCells:
  - {couple: "Cu^2+/Cu", E0: 0.34, n: 2, conc: 1}
  - {couple: "Zn^2+/Zn", E0: -0.76, n: 2, conc: 0.01}
showDeltaG: true
"#,
            true,
        )
        .unwrap();
        assert_svg_valido(&daniell);
        assert!(
            daniell.contains(r#"marker-end="url(#flecha-main-"#),
            "galvanic_cell debe emitir marker-end en las flechas de flujo de electrones"
        );
        let txt = texto_plano(&daniell);
        assert!(txt.contains("Ánodo(−)") && txt.contains("E°=1,10V"));
        assert!(txt.contains("1,159V") && txt.contains("-212,3kJ/mol"));

        let blank_cell = render_yaml(
            r#"
type: galvanic_cell
halfCells:
  - {couple: "Ag^+/Ag", E0: 0.80, n: 1}
  - {couple: "Cu^2+/Cu", E0: 0.34, n: 2}
blankPolarity: true
blankPotential: true
blankReactions: true
blankFlow: true
blankNotation: true
"#,
            true,
        )
        .unwrap();
        assert!(!blank_cell.contains(r#"marker-end="url(#flecha-main-"#));
        let txt_b = extraer_texto_svg(&blank_cell);
        assert!(!txt_b.contains("Ánodo") && !txt_b.contains("0,46") && !txt_b.contains('→'));

        // Errores esperados
        assert!(render_yaml(
            "type: galvanic_cell\nhalfCells:\n  - {couple: Zn, E0: 1}\n  - {couple: \"Cu^2+/Cu\", E0: 0.34}",
            true
        )
        .unwrap_err()
        .0
        .contains("oxidada/reducida"));
        assert!(render_yaml(
            "type: galvanic_cell\nhalfCells:\n  - {couple: \"Cu^2+/Cu\", E0: 0.34}",
            true
        )
        .unwrap_err()
        .0
        .contains("dos semiceldas"));
        assert!(render_yaml(
            "type: galvanic_cell\nhalfCells:\n  - {couple: \"Cu^2+/Cu\", E0: 0.34, n: 0}\n  - {couple: \"Zn^2+/Zn\", E0: -0.76}",
            true
        )
        .unwrap_err()
        .0
        .contains("al menos 1"));
        assert!(render_yaml(
            "type: galvanic_cell\nhalfCells:\n  - {couple: \"Cu^2+/Cu\", E0: 0.34, conc: -0.1}\n  - {couple: \"Zn^2+/Zn\", E0: -0.76}",
            true
        )
        .unwrap_err()
        .0
        .contains("tiene que ser positivo"));
    }

    // --- 7. unit_cell ---

    #[test]
    fn unit_cell_redes_densidad_blank_y_errores() {
        let cu = render_yaml(
            r#"
type: unit_cell
title: Cobre (fcc)
lattice: fcc
element: Cu
a: 361.5
molarMass: 63.55
"#,
            true,
        )
        .unwrap();
        assert_svg_valido(&cu);
        let txt = texto_plano(&cu);
        assert!(txt.contains("8×1/8+6×1/2=4") && txt.contains("74%") && txt.contains("8,94"));

        let nacl = render_yaml(
            r#"
type: unit_cell
lattice: nacl
a: 564
molarMass: 58.44
"#,
            false,
        )
        .unwrap();
        assert!(!extraer_texto_svg(&nacl).contains("Empaquetamiento"));
        assert!(texto_plano(&nacl).contains("2,16g/cm³"));

        let bcc_blank = render_yaml(
            r#"
type: unit_cell
lattice: bcc
element: Fe
blank: true
"#,
            true,
        )
        .unwrap();
        assert!(!extraer_texto_svg(&bcc_blank).contains("68"));

        // Errores esperados
        assert!(render_yaml("type: unit_cell\nlattice: hcp", true)
            .unwrap_err()
            .0
            .contains("red"));
        assert!(render_yaml(
            "type: unit_cell\nlattice: fcc\na: 0\nmolarMass: 63.55",
            true
        )
        .unwrap_err()
        .0
        .contains("a (pm): tiene que ser positivo"));
        assert!(render_yaml(
            "type: unit_cell\nlattice: fcc\na: 361.5\nmolarMass: -10",
            true
        )
        .unwrap_err()
        .0
        .contains("molarMass (g/mol): tiene que ser positivo"));
    }

    #[test]
    fn cobertura_variantes_e2e_restantes() {
        // reaction: coeficientes numéricos, flecha Unicode, alias eq y altura multilínea
        let coef = render_yaml(
            "type: reaction\nsteps: [2, \"H2\", \"+\", \"O2\", {arrow: \"→\", above: \"Pt\"}, 2, \"H2O\"]",
            false,
        )
        .unwrap();
        assert!(texto_plano(&coef).contains("2H2+O2Pt2H2O"));
        let eq = render_yaml(
            "type: reaction\ninkSaver: true\nsteps: [\"N2O4\", {arrow: \"eq\", above: \"ΔH° = +57,2 kJ/mol\", below: \"T = 298 K\"}, \"2 NO2\"]",
            false,
        )
        .unwrap();
        assert!(!eq.contains("#1d4ed8"));
        assert!(extraer_texto_svg(&eq).contains("ΔH° = +57,2 kJ/mol"));

        // vsepr: bent sin ligands, blank: true
        let bent = render_yaml("type: vsepr\ngeometry: bent\ncentral: O", false).unwrap();
        assert!(texto_plano(&bent).contains("AX2E2") && extraer_texto_svg(&bent).contains('X'));
        let vsepr_b = render_yaml("type: vsepr\ngeometry: bent_trigonal\nblank: true", true).unwrap();
        assert!(!extraer_texto_svg(&vsepr_b).contains("AX2E"));

        // orbitals: Fe^2+, Cl- por Z=17, mezcla s-p N2 vs O2
        let fe2 = render_yaml(
            "type: orbitals\nelement: Fe\ncharge: 2\ncore: true\nshowUnpaired: true",
            true,
        )
        .unwrap();
        assert!(extraer_texto_svg(&fe2).contains("[Ar] 3d⁶"));
        let cl = render_yaml(
            "type: orbitals\nelement: 17\ncharge: -1\ncore: true\nshowUnpaired: true",
            true,
        )
        .unwrap();
        assert!(texto_plano(&cl).contains("Cl−") && extraer_texto_svg(&cl).contains("[Ne] 3s² 3p⁶"));
        let n2 = render_yaml("type: orbitals\nkind: molecular\natoms: [N, N]", true).unwrap();
        let o2 = render_yaml("type: orbitals\nkind: molecular\natoms: [O, O]", true).unwrap();
        assert!(extraer_texto_svg(&n2).contains("Orden de enlace = 3"));
        assert!(extraer_texto_svg(&o2).contains("paramagnética"));

        // energy_profile: 2 etapas showEnergies, 3 etapas, solo dH, blankAnnotations
        let p2 = render_yaml(
            r#"
type: energy_profile
states:
  - {label: Reactivos, energy: 0}
  - {label: "ET1", energy: 60, ts: true}
  - {label: Intermediario, energy: 25}
  - {label: "ET2", energy: 90, ts: true}
  - {label: Productos, energy: 15}
showRateDetermining: true
showEnergies: true
"#,
            true,
        )
        .unwrap();
        let t_p2 = texto_plano(&p2);
        assert!(
            t_p2.contains("Ea1=60kJ/mol")
                && t_p2.contains("Ea2=65kJ/mol")
                && t_p2.contains("etapa2:determinante")
        );
        let solo_dh = render_yaml(
            "type: energy_profile\nannotations: [dH]\nstates:\n  - {label: R, energy: 50}\n  - {label: ET, energy: 90, ts: true}\n  - {label: P, energy: 10}",
            false,
        )
        .unwrap();
        assert!(!extraer_texto_svg(&solo_dh).contains("Ea ="));
        let p_blank = render_yaml(
            "type: energy_profile\nblankAnnotations: true\nstates:\n  - {energy: 0}\n  - {energy: 60, ts: true}\n  - {energy: 25}\n  - {energy: 90, ts: true}\n  - {energy: 15}",
            true,
        )
        .unwrap();
        let t_pb = texto_plano(&p_blank);
        assert!(t_pb.contains("Ea1=") && t_pb.contains("ΔH=") && !t_pb.contains("60kJ/mol"));

        // galvanic_cell: n distinto, Pt, showIons false, 1 conc, concentración E0=0
        let n_dist = render_yaml(
            r#"
type: galvanic_cell
halfCells:
  - {couple: "Ag^+/Ag", E0: 0.80, n: 1, conc: 0.1}
  - {couple: "Cu^2+/Cu", E0: 0.34, n: 2, conc: 1}
saltBridge: {name: NaCl, cation: "Na^+", anion: "Cl^-"}
"#,
            true,
        )
        .unwrap();
        let t_nd = texto_plano(&n_dist);
        assert!(t_nd.contains("E°=0,46V") && t_nd.contains("0,401V") && t_nd.contains("puentesalino(NaCl)"));
        let pt = render_yaml(
            r#"
type: galvanic_cell
anode: {couple: "H^+/H2", E0: 0.0, n: 2, electrode: Pt, solution: "HCl 1 M"}
cathode: {couple: "Fe^3+/Fe^2+", E0: 0.77, n: 1, electrode: Pt, solution: "Fe^3+/Fe^2+ 1 M"}
"#,
            true,
        )
        .unwrap();
        assert!(texto_plano(&pt).contains("Pt(s)|H+(1M)||Fe3+(1M)|Pt(s)"));
        let conc_cell = render_yaml(
            "type: galvanic_cell\nhalfCells:\n  - {couple: \"Cu^2+/Cu\", E0: 0.34, conc: 0.01}\n  - {couple: \"Cu^2+/Cu\", E0: 0.34, conc: 1}",
            true,
        )
        .unwrap();
        let t_cc = texto_plano(&conc_cell);
        assert!(t_cc.contains("E°=0,00V") && t_cc.contains("0,059V"));
        let una_conc = render_yaml(
            "type: galvanic_cell\nshowIons: false\nshowNotation: false\nhalfCells:\n  - {couple: \"Zn^2+/Zn\", E0: -0.76, conc: 0.1}\n  - {couple: \"Cu^2+/Cu\", E0: 0.34}",
            true,
        )
        .unwrap();
        assert!(!extraer_texto_svg(&una_conc).contains("Nernst") && !extraer_texto_svg(&una_conc).contains("Notación:"));

        // unit_cell: sc, cscl custom, blank selectivo, solo a
        let sc = render_yaml("type: unit_cell\nlattice: sc\nelement: Po", true).unwrap();
        assert!(texto_plano(&sc).contains("8×1/8=1") && extraer_texto_svg(&sc).contains("52,4 %"));
        let cscl = render_yaml("type: unit_cell\nlattice: cscl\nspecies: [\"Br^-\", \"Cs^+\"]", true).unwrap();
        assert!(texto_plano(&cscl).contains("Br−:") && texto_plano(&cscl).contains("Cs+:"));
        let sel = render_yaml("type: unit_cell\nlattice: fcc\nelement: Ag\nblank: [packing, z]", true).unwrap();
        let t_sel = extraer_texto_svg(&sel);
        assert!(t_sel.contains("Número de coordinación = 12") && !t_sel.contains("74 %") && !t_sel.contains("8 × 1/8"));
        let solo_a = render_yaml("type: unit_cell\nlattice: fcc\nelement: Al\na: 405\ninkSaver: true", false).unwrap();
        assert!(!extraer_texto_svg(&solo_a).contains("ρ =") && !solo_a.contains("#1d4ed8"));
    }

    #[test]
    fn regresion_extremos_finitos_y_reaction_width() {
        let reg = registro_m1b1();
        let payloads: &[(&str, &str)] = &[
            (
                "mol_bondLength_1e308",
                r#"{"type":"molecule","bondLength":1e308,"atoms":[{"id":"a","el":"C","at":[1,1]},{"id":"b","el":"O","at":[2,2]}],"bonds":[["a","b"]]}"#,
            ),
            (
                "mol_bondLength_1e308_ring",
                r#"{"type":"molecule","bondLength":1e308,"rings":[{"size":6,"center":[1,1],"aromatic":true,"kekule":true}]}"#,
            ),
            (
                "mol_at_1e308",
                r#"{"type":"molecule","atoms":[{"id":"a","el":"C","at":[1e308,-1e308]},{"id":"b","el":"O","at":[-1e308,1e308]}],"bonds":[["a","b"]]}"#,
            ),
            (
                "mol_length_1e308",
                r#"{"type":"molecule","atoms":[{"id":"a","el":"C","at":[0,0]},{"id":"b","el":"O","from":"a","angle":45,"length":1e308}]}"#,
            ),
            (
                "mol_arrows_bend_1e308",
                r#"{"type":"molecule","atoms":[{"id":"a","el":"C","at":[0,0]},{"id":"b","el":"N","at":[1,0]}],"arrows":[{"from":"a","to":"b","bend":1e308,"half":true}]}"#,
            ),
            (
                "rxn_steps_mol_1e308",
                r#"{"type":"reaction","steps":[{"atoms":[{"id":"a","el":"C","at":[1e308,0]}]}]}"#,
            ),
            (
                "rxn_blank_neg5",
                r#"{"type":"reaction","steps":["A",{"arrow":"->"},{"blank":true,"width":-5}]}"#,
            ),
            (
                "ep_1e308_sym",
                r#"{"type":"energy_profile","states":[{"label":"R","energy":-1e308},{"label":"TS","energy":1e308,"ts":true},{"label":"P","energy":0}],"showEnergies":true}"#,
            ),
            (
                "gc_conc_1e_100",
                r#"{"type":"galvanic_cell","halfCells":[{"couple":"A+/A","E0":0.0,"n":1,"conc":1e-100},{"couple":"B+/B","E0":1.0,"n":100,"conc":1.0}]}"#,
            ),
            (
                "gc_conc_1e300_pow",
                r#"{"type":"galvanic_cell","halfCells":[{"couple":"A+/A","E0":0.0,"n":1,"conc":1e300},{"couple":"B+/B","E0":1.0,"n":100,"conc":1e300}]}"#,
            ),
            (
                "gc_conc_1e300_nan",
                r#"{"type":"galvanic_cell","halfCells":[{"couple":"A+/A","E0":0.0,"n":2,"conc":1e300},{"couple":"B+/B","E0":1.0,"n":3,"conc":1e300}]}"#,
            ),
            (
                "gc_e0_1e308",
                r#"{"type":"galvanic_cell","halfCells":[{"couple":"A+/A","E0":-1e308,"n":1},{"couple":"B+/B","E0":1e308,"n":1}],"showDeltaG":true}"#,
            ),
            (
                "uc_a_1e_100",
                r#"{"type":"unit_cell","lattice":"fcc","a":1e-100,"molarMass":63.5}"#,
            ),
            (
                "uc_m_1e308",
                r#"{"type":"unit_cell","lattice":"fcc","a":1.0,"molarMass":1e308}"#,
            ),
        ];

        for &(nombre, json) in payloads {
            let bloques = leer_bloques(json).expect(nombre);
            for tinta in [true, false] {
                match reg.renderizar(&bloques[0], tinta) {
                    Ok(svg) => {
                        assert_svg_valido(&svg);
                        assert!(
                            !svg.contains(r#"width="-"#),
                            "{nombre} (tinta={tinta}) emitió width negativo: {svg}"
                        );
                    }
                    Err(e) => {
                        assert!(!e.0.is_empty(), "{nombre} devolvió error vacío");
                    }
                }
            }
        }

        // Casos adicionales de borde: reaction blank.width <= 0, maxWidth <= 0, molecule bondLength=1 at=±1e308, energy_profile span overflow
        for &(nombre, json) in &[
            (
                "rxn_blank_zero",
                r#"{"type":"reaction","steps":[{"blank":true,"width":0}]}"#,
            ),
            (
                "rxn_blank_neg_zero",
                r#"{"type":"reaction","steps":[{"blank":true,"width":-0.0}]}"#,
            ),
            (
                "rxn_maxwidth_neg",
                r#"{"type":"reaction","maxWidth":-10,"steps":["A"]}"#,
            ),
            (
                "rxn_maxwidth_zero",
                r#"{"type":"reaction","maxWidth":0,"steps":["A"]}"#,
            ),
            (
                "mol_at_1e308_unit_bond",
                r#"{"type":"molecule","bondLength":1,"atoms":[{"id":"a","el":"C","at":[1e308,-1e308]},{"id":"b","el":"O","at":[-1e308,1e308]}],"bonds":[["a","b"]]}"#,
            ),
            (
                "ep_span_overflow",
                r#"{"type":"energy_profile","states":[{"label":"R","energy":-0.8e308},{"label":"P","energy":0.8e308}]}"#,
            ),
        ] {
            let bloques = leer_bloques(json).expect(nombre);
            let res = reg.renderizar(&bloques[0], true);
            assert!(
                res.is_err(),
                "{nombre} debería rechazarse con ErrorDiagrama, devolvió: {res:?}"
            );
        }
    }
}
