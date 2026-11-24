//! Estructuras de datos: `heap`, `hash_table`, `avl` y `bplus_tree`.

use crate::comun::*;
use serde_json::Value;

// ---------------------------------------------------------------------------
// Árbol binario genérico (disposición por recorrido en orden)
// ---------------------------------------------------------------------------

pub struct NodoDib {
    pub texto: String,
    pub izq: Option<usize>,
    pub der: Option<usize>,
    pub nota: String,
    pub resaltar: bool,
}

const SEP_X: f64 = 40.0;
const SEP_Y: f64 = 54.0;
const RADIO: f64 = 16.0;

/// Dibuja el árbol con la raíz `raiz` desde (x0, y0). Devuelve (ancho, alto).
pub fn dibujar_arbol(
    c: &mut Lienzo,
    nodos: &[NodoDib],
    raiz: Option<usize>,
    x0: f64,
    y0: f64,
    blanco: bool,
) -> Punto {
    let Some(raiz) = raiz else {
        c.t(
            x0,
            y0 + 16.0,
            "(vacío)",
            11.0,
            500,
            "textMuted",
            "start",
            MONO,
        );
        return (60.0, 24.0);
    };
    let mut pos = vec![(0.0, 0.0); nodos.len()];
    let mut orden = 0usize;
    let mut prof_max = 0usize;
    fn recorrer(
        n: &[NodoDib],
        i: usize,
        prof: usize,
        orden: &mut usize,
        pos: &mut [(f64, f64)],
        pm: &mut usize,
    ) {
        if let Some(l) = n[i].izq {
            recorrer(n, l, prof + 1, orden, pos, pm);
        }
        pos[i] = (*orden as f64, prof as f64);
        *orden += 1;
        *pm = (*pm).max(prof);
        if let Some(r) = n[i].der {
            recorrer(n, r, prof + 1, orden, pos, pm);
        }
    }
    recorrer(nodos, raiz, 0, &mut orden, &mut pos, &mut prof_max);
    let ancho_texto_max = nodos
        .iter()
        .map(|n| ancho_texto(&n.texto, 11.0))
        .fold(0.0, f64::max);
    let sep = SEP_X.max(ancho_texto_max + 14.0);
    let xy = |i: usize| {
        (
            x0 + RADIO + 4.0 + pos[i].0 * sep,
            y0 + RADIO + 4.0 + pos[i].1 * SEP_Y,
        )
    };
    // Aristas primero.
    let mut pila = vec![raiz];
    while let Some(i) = pila.pop() {
        for h in [nodos[i].izq, nodos[i].der].into_iter().flatten() {
            let (a, b) = (xy(i), xy(h));
            c.linea(
                a.0,
                a.1 + RADIO * 0.8,
                b.0,
                b.1 - RADIO * 0.9,
                "strokeMain",
                1.3,
                None,
                "",
            );
            pila.push(h);
        }
    }
    let mut pila = vec![raiz];
    while let Some(i) = pila.pop() {
        let (x, y) = xy(i);
        let r = RADIO.max(ancho_texto(&nodos[i].texto, 11.0) / 2.0 + 5.0);
        circulo(
            c,
            x,
            y,
            r,
            if nodos[i].resaltar {
                "fillHighlight"
            } else {
                "fillPointerBox"
            },
            "strokeMain",
            1.5,
        );
        if !blanco {
            c.t(
                x,
                y + 4.0,
                &nodos[i].texto,
                11.0,
                700,
                "textMain",
                "middle",
                MONO,
            );
        }
        if !nodos[i].nota.is_empty() {
            c.t(
                x + r + 2.0,
                y - r + 2.0,
                &nodos[i].nota,
                9.0,
                600,
                "textSub",
                "start",
                MONO,
            );
        }
        pila.extend([nodos[i].izq, nodos[i].der].into_iter().flatten());
    }
    (
        2.0 * RADIO + 8.0 + (orden.max(1) - 1) as f64 * sep + 16.0,
        2.0 * RADIO + 8.0 + prof_max as f64 * SEP_Y,
    )
}

/// Fila de celdas con índices (para el arreglo del heap).
pub fn arreglo(
    c: &mut Lienzo,
    x: f64,
    y: f64,
    vals: &[String],
    blanco: bool,
    desde: usize,
) -> Punto {
    let w = vals
        .iter()
        .map(|v| ancho_texto(v, 11.0) + 14.0)
        .fold(30.0, f64::max);
    for (k, v) in vals.iter().enumerate() {
        let xx = x + k as f64 * w;
        c.r(xx, y, w, 26.0, "fillCell", 1.2, 0.0);
        if !blanco {
            c.t(
                xx + w / 2.0,
                y + 17.0,
                v,
                11.0,
                600,
                "textMain",
                "middle",
                MONO,
            );
        }
        c.t(
            xx + w / 2.0,
            y + 39.0,
            &(k + desde).to_string(),
            9.0,
            400,
            "textMuted",
            "middle",
            MONO,
        );
    }
    (vals.len() as f64 * w, 44.0)
}

fn valores_num(b: &Bloque, clave: &str) -> Result<Vec<f64>, ErrorDiagrama> {
    lista_num(b, clave, true)
}

// ---------------------------------------------------------------------------
// heap
// ---------------------------------------------------------------------------

pub fn hundir(h: &mut [f64], mut i: usize, n: usize, menor: bool) {
    let mejor = |a: f64, b: f64| if menor { a < b } else { a > b };
    loop {
        let (l, r) = (2 * i + 1, 2 * i + 2);
        let mut m = i;
        if l < n && mejor(h[l], h[m]) {
            m = l;
        }
        if r < n && mejor(h[r], h[m]) {
            m = r;
        }
        if m == i {
            return;
        }
        h.swap(i, m);
        i = m;
    }
}

pub fn flotar(h: &mut [f64], mut i: usize, menor: bool) {
    while i > 0 {
        let p = (i - 1) / 2;
        let cambia = if menor { h[i] < h[p] } else { h[i] > h[p] };
        if !cambia {
            return;
        }
        h.swap(i, p);
        i = p;
    }
}

pub fn heapify(h: &mut [f64], menor: bool) {
    let n = h.len();
    for i in (0..n / 2).rev() {
        hundir(h, i, n, menor);
    }
}

pub fn render_heap(c: &mut Lienzo, b: &Bloque) -> Result<Punto, ErrorDiagrama> {
    let menor = opcion(b, "kind", &["min", "max"], "min")? == "min";
    let construir = opcion(b, "build", &["none", "insert", "heapify"], "insert")?;
    let mut vals = valores_num(b, "values")?;
    if vals.len() > 63 {
        return Err(err("como máximo 63 valores"));
    }
    let blanco = bool_opt(b, "blank", false);
    let pasos = bool_opt(b, "steps", false);
    let mut estados: Vec<(String, Vec<f64>)> = Vec::new();
    match construir.as_str() {
        "heapify" => {
            if pasos {
                estados.push(("arreglo inicial".into(), vals.clone()));
            }
            heapify(&mut vals, menor);
            estados.push(("heapify (Floyd)".into(), vals.clone()));
        }
        "insert" => {
            let mut h: Vec<f64> = Vec::new();
            for v in &vals {
                h.push(*v);
                let n = h.len();
                flotar(&mut h, n - 1, menor);
                if pasos {
                    estados.push((format!("insertar {}", fmt(*v, 3)), h.clone()));
                }
            }
            vals = h;
            if !pasos {
                estados.push((String::new(), vals.clone()));
            }
        }
        _ => estados.push((String::new(), vals.clone())),
    }
    for op in lista(b, "ops", true)? {
        let t = valor_str(op).to_lowercase();
        let mut partes = t.split_whitespace();
        match (partes.next(), partes.next()) {
            (Some("insert") | Some("insertar") | Some("push"), Some(x)) => {
                let x: f64 = x
                    .replace(',', ".")
                    .parse()
                    .map_err(|_| err(format!("operación {t:?}: valor inválido")))?;
                vals.push(x);
                let n = vals.len();
                flotar(&mut vals, n - 1, menor);
                estados.push((format!("insertar {}", fmt(x, 3)), vals.clone()));
            }
            (Some(o), None)
                if o.starts_with("extract") || o.starts_with("extraer") || o == "pop" =>
            {
                if vals.is_empty() {
                    return Err(err("extraer de un heap vacío"));
                }
                let raiz = vals.swap_remove(0);
                let n = vals.len();
                hundir(&mut vals, 0, n, menor);
                estados.push((format!("extraer → {}", fmt(raiz, 3)), vals.clone()));
            }
            _ => {
                return Err(err(format!(
                    "operación {t:?} desconocida (insert X, extract)"
                )))
            }
        }
    }
    if !pasos && estados.len() > 1 {
        let ultimo = estados.pop().unwrap();
        estados = vec![(String::new(), ultimo.1)];
    }
    if estados.len() > 16 {
        return Err(err("demasiados pasos para dibujar (máximo 16)"));
    }
    let extra = if menor { "min-heap" } else { "max-heap" };
    let y0 = encabezado(c, b, extra);
    let mut y = y0 + 6.0;
    let mut ancho: f64 = 0.0;
    for (k, (titulo, h)) in estados.iter().enumerate() {
        let oculto = blanco && k == estados.len() - 1;
        if !titulo.is_empty() {
            c.t(0.0, y + 12.0, titulo, 11.0, 700, "textSub", "start", SANS);
            y += 18.0;
        }
        let nodos: Vec<NodoDib> = h
            .iter()
            .enumerate()
            .map(|(i, v)| NodoDib {
                texto: fmt(*v, 3),
                izq: (2 * i + 1 < h.len()).then_some(2 * i + 1),
                der: (2 * i + 2 < h.len()).then_some(2 * i + 2),
                nota: String::new(),
                resaltar: i == 0,
            })
            .collect();
        let (wa, ha) = dibujar_arbol(c, &nodos, (!h.is_empty()).then_some(0), 0.0, y, oculto);
        let textos: Vec<String> = h.iter().map(|v| fmt(*v, 3)).collect();
        let (wr, hr) = arreglo(c, wa + 30.0, y + ha / 2.0 - 13.0, &textos, oculto, 0);
        ancho = ancho.max(wa + 30.0 + wr);
        y += ha.max(hr) + 18.0;
    }
    Ok((ancho, y - 10.0))
}

// ---------------------------------------------------------------------------
// hash_table
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct Insercion {
    pub colision: bool,
    pub clave: String,
    pub hash: usize,
    pub sondeos: Vec<usize>,
    pub final_: Option<usize>,
}

fn hash_de(clave: &Value, m: usize, modo_str: &str) -> Result<(String, u64), ErrorDiagrama> {
    match clave {
        Value::Number(n) => {
            let x = n
                .as_i64()
                .ok_or_else(|| err("las claves numéricas tienen que ser enteras"))?;
            Ok((x.to_string(), x.rem_euclid(m as i64) as u64))
        }
        Value::String(s) => {
            let h = if modo_str == "poly" {
                s.chars().fold(0u64, |a, ch| {
                    (a.wrapping_mul(31).wrapping_add(ch as u64)) % (m as u64)
                })
            } else {
                s.chars().map(|ch| ch as u64).sum::<u64>() % m as u64
            };
            Ok((s.clone(), h))
        }
        _ => Err(err("las claves son enteros o textos")),
    }
}

fn primo_menor(m: usize) -> usize {
    (2..m)
        .rev()
        .find(|&p| (2..p).take_while(|d| d * d <= p).all(|d| p % d != 0))
        .unwrap_or(1)
}

pub fn render_hash_table(c: &mut Lienzo, b: &Bloque) -> Result<Punto, ErrorDiagrama> {
    let m = entero_b(b, "size", 11, 1, 101)? as usize;
    let metodo = opcion(
        b,
        "collision",
        &["chaining", "linear", "quadratic", "double"],
        "linear",
    )?;
    let modo_str = opcion(b, "stringHash", &["sum", "poly"], "sum")?;
    let r2 = entero_b(b, "R", primo_menor(m) as i64, 1, 1000)? as usize;
    let blanco = bool_opt(b, "blank", false);
    let claves = lista(b, "keys", false)?;
    if claves.len() > 60 {
        return Err(err("como máximo 60 claves"));
    }
    let mut celdas: Vec<Vec<String>> = vec![Vec::new(); m];
    let mut log: Vec<Insercion> = Vec::new();
    for k in claves {
        let (texto, h) = hash_de(k, m, &modo_str)?;
        let h = h as usize;
        if metodo == "chaining" {
            let colision = !celdas[h].is_empty();
            celdas[h].push(texto.clone());
            log.push(Insercion {
                colision,
                clave: texto,
                hash: h,
                sondeos: vec![h],
                final_: Some(h),
            });
            continue;
        }
        let h2 = r2 - (h % r2.max(1));
        let num_clave = k.as_i64().unwrap_or(h as i64);
        let h2 = if metodo == "double" {
            (r2 as i64 - num_clave.rem_euclid(r2 as i64)) as usize
        } else {
            h2
        };
        let mut sondeos = Vec::new();
        let mut final_ = None;
        for i in 0..m {
            let pos = match metodo.as_str() {
                "linear" => (h + i) % m,
                "quadratic" => (h + i * i) % m,
                _ => (h + i * h2.max(1)) % m,
            };
            sondeos.push(pos);
            if celdas[pos].is_empty() {
                celdas[pos].push(texto.clone());
                final_ = Some(pos);
                break;
            }
        }
        log.push(Insercion {
            colision: sondeos.len() > 1,
            clave: texto,
            hash: h,
            sondeos,
            final_,
        });
    }
    let nombre_metodo = match metodo.as_str() {
        "chaining" => "encadenamiento".to_string(),
        "linear" => "sondeo lineal".to_string(),
        "quadratic" => "sondeo cuadrático".to_string(),
        _ => format!("doble hashing, h₂(k) = {r2} − (k mod {r2})"),
    };
    let ocupadas: usize = celdas.iter().map(|v| v.len()).sum();
    let y0 = encabezado(c, b, &format!("m = {m}, h(k) = k mod {m}, {nombre_metodo}"));
    let y = y0 + 6.0;
    // Tabla de cubetas.
    let fh = 24.0;
    let wi = 34.0;
    let wc = celdas
        .iter()
        .flat_map(|v| v.iter())
        .map(|t| ancho_texto(t, 11.0) + 16.0)
        .fold(54.0, f64::max);
    let mut ancho_cadenas: f64 = 0.0;
    for (i, cont) in celdas.iter().enumerate() {
        let yy = y + i as f64 * fh;
        c.r(0.0, yy, wi, fh, "fillHeader", 1.0, 0.0);
        c.t(
            wi / 2.0,
            yy + 16.0,
            &i.to_string(),
            10.0,
            600,
            "textMuted",
            "middle",
            MONO,
        );
        c.r(wi, yy, wc, fh, "fillCell", 1.2, 0.0);
        if metodo == "chaining" {
            if cont.is_empty() {
                c.t(
                    wi + wc / 2.0,
                    yy + 16.0,
                    "∅",
                    10.0,
                    400,
                    "textMuted",
                    "middle",
                    MONO,
                );
                continue;
            }
            c.punto(wi + wc / 2.0, yy + fh / 2.0, "strokeMain", 2.6);
            let mut x = wi + wc / 2.0;
            for (j, t) in cont.iter().enumerate() {
                let nx = wi + wc + 24.0 + j as f64 * (wc + 24.0);
                c.linea(
                    x,
                    yy + fh / 2.0,
                    nx,
                    yy + fh / 2.0,
                    "strokeMain",
                    1.3,
                    Some("main"),
                    "",
                );
                c.r(nx, yy + 2.0, wc, fh - 4.0, "fillPointerBox", 1.2, 3.0);
                if !blanco {
                    c.t(
                        nx + wc / 2.0,
                        yy + 16.0,
                        t,
                        11.0,
                        600,
                        "textMain",
                        "middle",
                        MONO,
                    );
                }
                x = nx + wc;
                ancho_cadenas = ancho_cadenas.max(nx + wc);
            }
        } else if let Some(t) = cont.first() {
            if !blanco {
                c.t(
                    wi + wc / 2.0,
                    yy + 16.0,
                    t,
                    11.0,
                    600,
                    "textMain",
                    "middle",
                    MONO,
                );
            }
        }
    }
    let alto_tabla = m as f64 * fh;
    let x_log = (wi + wc).max(ancho_cadenas) + 34.0;
    // Registro de inserciones.
    let enc: Vec<String> = ["Clave", "h(k)", "Sondeos", "Posición"]
        .iter()
        .map(|s| s.to_string())
        .collect();
    let filas: Vec<Vec<Celda>> = log
        .iter()
        .map(|ins| {
            let sondeos = ins
                .sondeos
                .iter()
                .map(|s| s.to_string())
                .collect::<Vec<_>>()
                .join(" → ");
            vec![
                celda(&ins.clave).fuerte(true),
                celda(ins.hash.to_string()).en_blanco(blanco),
                celda(if metodo == "chaining" {
                    "—".into()
                } else {
                    sondeos
                })
                .en_blanco(blanco && metodo != "chaining"),
                celda(
                    ins.final_
                        .map(|p| p.to_string())
                        .unwrap_or_else(|| "sin lugar".into()),
                )
                .en_blanco(blanco)
                .resaltada(!blanco && (ins.sondeos.len() > 1 || ins.final_.is_none())),
            ]
        })
        .collect();
    let (wl, hl) = tabla(
        c,
        x_log,
        y,
        &enc,
        &filas,
        &OpcTabla {
            fila_h: 22.0,
            ..Default::default()
        },
    );
    let mut yy = y + alto_tabla.max(hl) + 18.0;
    let alfa = ocupadas as f64 / m as f64;
    let colisiones = log.iter().filter(|i| i.colision).count();
    let txt = if blanco {
        "Factor de carga α = ____".to_string()
    } else {
        format!(
            "Factor de carga α = {ocupadas}/{m} = {} · inserciones con colisión: {colisiones}",
            fmt(alfa, 3)
        )
    };
    c.t(0.0, yy, &txt, 10.5, 700, "textMain", "start", MONO);
    yy += 6.0;
    Ok(((x_log + wl).max(ancho_texto(&txt, 10.5)), yy))
}

// ---------------------------------------------------------------------------
// avl
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct NodoAvl {
    pub clave: f64,
    pub izq: Option<usize>,
    pub der: Option<usize>,
    pub altura: i32,
}

#[derive(Default)]
pub struct Avl {
    pub n: Vec<NodoAvl>,
    pub raiz: Option<usize>,
    pub log: Vec<String>,
}

impl Avl {
    fn alt(&self, i: Option<usize>) -> i32 {
        i.map(|k| self.n[k].altura).unwrap_or(0)
    }
    pub fn balance(&self, i: usize) -> i32 {
        self.alt(self.n[i].izq) - self.alt(self.n[i].der)
    }
    fn actualizar(&mut self, i: usize) {
        self.n[i].altura = 1 + self.alt(self.n[i].izq).max(self.alt(self.n[i].der));
    }
    fn rot_der(&mut self, y: usize) -> usize {
        let x = self.n[y].izq.unwrap();
        self.n[y].izq = self.n[x].der;
        self.n[x].der = Some(y);
        self.actualizar(y);
        self.actualizar(x);
        x
    }
    fn rot_izq(&mut self, x: usize) -> usize {
        let y = self.n[x].der.unwrap();
        self.n[x].der = self.n[y].izq;
        self.n[y].izq = Some(x);
        self.actualizar(x);
        self.actualizar(y);
        y
    }
    fn rebalancear(&mut self, i: usize) -> usize {
        self.actualizar(i);
        let bal = self.balance(i);
        let k = fmt(self.n[i].clave, 3);
        if bal > 1 {
            let l = self.n[i].izq.unwrap();
            if self.balance(l) < 0 {
                self.log
                    .push(format!("rotación doble izquierda-derecha en {k}"));
                self.n[i].izq = Some(self.rot_izq(l));
            } else {
                self.log.push(format!("rotación simple a derecha en {k}"));
            }
            return self.rot_der(i);
        }
        if bal < -1 {
            let r = self.n[i].der.unwrap();
            if self.balance(r) > 0 {
                self.log
                    .push(format!("rotación doble derecha-izquierda en {k}"));
                self.n[i].der = Some(self.rot_der(r));
            } else {
                self.log.push(format!("rotación simple a izquierda en {k}"));
            }
            return self.rot_izq(i);
        }
        i
    }
    fn ins(&mut self, i: Option<usize>, k: f64) -> usize {
        let Some(i) = i else {
            self.n.push(NodoAvl {
                clave: k,
                izq: None,
                der: None,
                altura: 1,
            });
            return self.n.len() - 1;
        };
        if k < self.n[i].clave {
            let l = self.ins(self.n[i].izq, k);
            self.n[i].izq = Some(l);
        } else if k > self.n[i].clave {
            let r = self.ins(self.n[i].der, k);
            self.n[i].der = Some(r);
        } else {
            self.log.push(format!("{} ya está", fmt(k, 3)));
            return i;
        }
        self.rebalancear(i)
    }
    pub fn insertar(&mut self, k: f64) {
        let r = self.ins(self.raiz, k);
        self.raiz = Some(r);
    }
    fn min(&self, mut i: usize) -> usize {
        while let Some(l) = self.n[i].izq {
            i = l;
        }
        i
    }
    fn del(&mut self, i: Option<usize>, k: f64) -> Option<usize> {
        let i = i?;
        if k < self.n[i].clave {
            self.n[i].izq = self.del(self.n[i].izq, k);
        } else if k > self.n[i].clave {
            self.n[i].der = self.del(self.n[i].der, k);
        } else {
            match (self.n[i].izq, self.n[i].der) {
                (None, r) => return r,
                (l, None) => return l,
                (_, Some(r)) => {
                    let s = self.min(r);
                    self.n[i].clave = self.n[s].clave;
                    let sk = self.n[s].clave;
                    self.n[i].der = self.del(Some(r), sk);
                }
            }
        }
        Some(self.rebalancear(i))
    }
    pub fn eliminar(&mut self, k: f64) {
        self.raiz = self.del(self.raiz, k);
    }
    fn dibujable(&self, balances: bool, resaltar: Option<f64>) -> Vec<NodoDib> {
        self.n
            .iter()
            .enumerate()
            .map(|(i, nd)| NodoDib {
                texto: fmt(nd.clave, 3),
                izq: nd.izq,
                der: nd.der,
                nota: if balances {
                    format!("{:+}", self.balance(i)).replace("+0", "0")
                } else {
                    String::new()
                },
                resaltar: resaltar == Some(nd.clave),
            })
            .collect()
    }
}

pub fn render_avl(c: &mut Lienzo, b: &Bloque) -> Result<Punto, ErrorDiagrama> {
    let claves = valores_num(b, "keys")?;
    if claves.len() > 40 {
        return Err(err("como máximo 40 claves"));
    }
    let borrar = valores_num(b, "delete")?;
    let pasos = opcion(b, "steps", &["none", "rotations", "all"], "none")?;
    let balances = bool_opt(b, "balance", true);
    let blanco = bool_opt(b, "blank", false);
    let mut t = Avl::default();
    let mut estados: Vec<(String, Vec<NodoDib>, Option<usize>)> = Vec::new();
    let ops: Vec<(bool, f64)> = claves
        .iter()
        .map(|k| (true, *k))
        .chain(borrar.iter().map(|k| (false, *k)))
        .collect();
    for (insertar, k) in ops {
        let antes = t.log.len();
        if insertar {
            t.insertar(k);
        } else {
            t.eliminar(k);
        }
        let rot = t.log[antes..].join("; ");
        let mostrar = pasos == "all" || (pasos == "rotations" && !rot.is_empty());
        if mostrar {
            let accion = if insertar { "insertar" } else { "eliminar" };
            let titulo = if rot.is_empty() {
                format!("{accion} {}", fmt(k, 3))
            } else {
                format!("{accion} {} → {rot}", fmt(k, 3))
            };
            estados.push((titulo, t.dibujable(balances, Some(k)), t.raiz));
        }
    }
    if estados.len() > 20 {
        return Err(err("demasiados pasos para dibujar (máximo 20)"));
    }
    let y0 = encabezado(c, b, "");
    let mut y = y0 + 4.0;
    let mut ancho: f64 = 0.0;
    if !estados.is_empty() {
        // Pasos en una grilla de dos columnas.
        let mut x = 0.0;
        let mut alto_fila: f64 = 0.0;
        for (k, (titulo, nodos, raiz)) in estados.iter().enumerate() {
            let mut sub_y = y;
            c.t(x, sub_y + 12.0, titulo, 10.5, 700, "textSub", "start", SANS);
            sub_y += 18.0;
            let (w, h) = dibujar_arbol(c, nodos, *raiz, x, sub_y, false);
            let w = w.max(ancho_texto(titulo, 10.5));
            ancho = ancho.max(x + w);
            alto_fila = alto_fila.max(h + 26.0);
            if k % 2 == 0 && k + 1 < estados.len() {
                x += w + 40.0;
            } else {
                x = 0.0;
                y += alto_fila + 10.0;
                alto_fila = 0.0;
            }
        }
        if alto_fila > 0.0 {
            y += alto_fila + 10.0;
        }
        c.t(
            0.0,
            y + 10.0,
            "Árbol final",
            11.0,
            700,
            "textMain",
            "start",
            SANS,
        );
        y += 18.0;
    }
    let (w, h) = dibujar_arbol(
        c,
        &t.dibujable(balances && !blanco, None),
        t.raiz,
        0.0,
        y,
        blanco,
    );
    ancho = ancho.max(w);
    y += h + 10.0;
    if balances && !blanco {
        c.t(
            0.0,
            y + 8.0,
            "número junto a cada nodo: factor de balance (altura izq. − altura der.)",
            9.5,
            400,
            "textMuted",
            "start",
            MONO,
        );
        ancho = ancho.max(ancho_texto(
            "número junto a cada nodo: factor de balance (altura izq. − altura der.)",
            9.5,
        ));
        y += 14.0;
    }
    Ok((ancho, y))
}

// ---------------------------------------------------------------------------
// bplus_tree
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct NodoBp {
    pub claves: Vec<f64>,
    pub hijos: Vec<usize>,
    pub hoja: bool,
}

pub struct BPlus {
    pub n: Vec<NodoBp>,
    pub raiz: usize,
    pub max_claves: usize,
    pub log: Vec<String>,
}

impl BPlus {
    pub fn new(max_claves: usize) -> Self {
        Self {
            n: vec![NodoBp {
                claves: Vec::new(),
                hijos: Vec::new(),
                hoja: true,
            }],
            raiz: 0,
            max_claves,
            log: Vec::new(),
        }
    }
    /// Inserta; devuelve (clave separadora, nuevo nodo derecho) si el nodo `i` se partió.
    fn ins(&mut self, i: usize, k: f64) -> Option<(f64, usize)> {
        if self.n[i].hoja {
            let pos = self.n[i].claves.partition_point(|&x| x < k);
            if self.n[i].claves.get(pos) == Some(&k) {
                return None;
            }
            self.n[i].claves.insert(pos, k);
        } else {
            let pos = self.n[i].claves.partition_point(|&x| x <= k);
            let hijo = self.n[i].hijos[pos];
            if let Some((sep, nuevo)) = self.ins(hijo, k) {
                self.n[i].claves.insert(pos, sep);
                self.n[i].hijos.insert(pos + 1, nuevo);
            }
        }
        if self.n[i].claves.len() <= self.max_claves {
            return None;
        }
        let total = self.n[i].claves.len();
        if self.n[i].hoja {
            let medio = total.div_ceil(2);
            let derecha = self.n[i].claves.split_off(medio);
            let sep = derecha[0];
            self.n.push(NodoBp {
                claves: derecha,
                hijos: Vec::new(),
                hoja: true,
            });
            self.log.push(format!(
                "se parte una hoja; sube una copia de {}",
                fmt(sep, 3)
            ));
            Some((sep, self.n.len() - 1))
        } else {
            let medio = total / 2;
            let sep = self.n[i].claves[medio];
            let derecha_claves = self.n[i].claves.split_off(medio + 1);
            self.n[i].claves.pop();
            let derecha_hijos = self.n[i].hijos.split_off(medio + 1);
            self.n.push(NodoBp {
                claves: derecha_claves,
                hijos: derecha_hijos,
                hoja: false,
            });
            self.log
                .push(format!("se parte un nodo interno; sube {}", fmt(sep, 3)));
            Some((sep, self.n.len() - 1))
        }
    }
    pub fn insertar(&mut self, k: f64) {
        if let Some((sep, nuevo)) = self.ins(self.raiz, k) {
            self.n.push(NodoBp {
                claves: vec![sep],
                hijos: vec![self.raiz, nuevo],
                hoja: false,
            });
            self.raiz = self.n.len() - 1;
            self.log.push("crece la altura (nueva raíz)".into());
        }
    }
    pub fn niveles(&self) -> Vec<Vec<usize>> {
        let mut out = vec![vec![self.raiz]];
        loop {
            let sig: Vec<usize> = out
                .last()
                .unwrap()
                .iter()
                .flat_map(|&i| self.n[i].hijos.clone())
                .collect();
            if sig.is_empty() {
                return out;
            }
            out.push(sig);
        }
    }
}

fn dibujar_bplus(c: &mut Lienzo, t: &BPlus, y0: f64, blanco: bool) -> Punto {
    let celda_w =
        t.n.iter()
            .flat_map(|n| n.claves.iter())
            .map(|k| ancho_texto(&fmt(*k, 3), 11.0) + 12.0)
            .fold(28.0, f64::max);
    let ancho_nodo = |i: usize| (t.n[i].claves.len().max(1)) as f64 * celda_w;
    let niveles = t.niveles();
    let hojas = niveles.last().unwrap().clone();
    let sep = 18.0;
    // Posición x de cada nodo: hojas en fila; los internos centrados sobre sus hijos.
    let mut x = vec![0.0; t.n.len()];
    let mut cx = 0.0;
    for &h in &hojas {
        x[h] = cx;
        cx += ancho_nodo(h) + sep;
    }
    for nivel in niveles.iter().rev().skip(1) {
        for &i in nivel {
            let hs = &t.n[i].hijos;
            let ini = x[hs[0]];
            let fin = x[*hs.last().unwrap()] + ancho_nodo(*hs.last().unwrap());
            x[i] = (ini + fin) / 2.0 - ancho_nodo(i) / 2.0;
        }
    }
    let min_x = x.iter().cloned().fold(f64::INFINITY, f64::min).min(0.0);
    let alto_n = 26.0;
    let sep_y = 56.0;
    for (nv, nivel) in niveles.iter().enumerate() {
        let y = y0 + nv as f64 * sep_y;
        for &i in nivel {
            let xi = x[i] - min_x;
            for (k, clave) in t.n[i].claves.iter().enumerate() {
                c.r(
                    xi + k as f64 * celda_w,
                    y,
                    celda_w,
                    alto_n,
                    if t.n[i].hoja {
                        "fillCell"
                    } else {
                        "fillPointerBox"
                    },
                    1.3,
                    0.0,
                );
                if !blanco {
                    c.t(
                        xi + k as f64 * celda_w + celda_w / 2.0,
                        y + 17.0,
                        &fmt(*clave, 3),
                        11.0,
                        600,
                        "textMain",
                        "middle",
                        MONO,
                    );
                }
            }
            for (k, &h) in t.n[i].hijos.iter().enumerate() {
                let px = xi + k as f64 * celda_w;
                let hx = x[h] - min_x + ancho_nodo(h) / 2.0;
                c.linea(
                    px,
                    y + alto_n,
                    hx,
                    y + sep_y - 2.0,
                    "strokeMain",
                    1.2,
                    None,
                    "",
                );
                c.punto(px, y + alto_n, "strokeMain", 2.2);
            }
        }
    }
    // Enlaces entre hojas.
    let yh = y0 + (niveles.len() - 1) as f64 * sep_y + alto_n / 2.0;
    for par in hojas.windows(2) {
        let x1 = x[par[0]] - min_x + ancho_nodo(par[0]);
        let x2 = x[par[1]] - min_x;
        c.linea(
            x1 + 1.0,
            yh,
            x2 - 1.0,
            yh,
            "strokeSub",
            1.2,
            Some("sub"),
            "",
        );
    }
    (
        cx - sep - min_x,
        (niveles.len() - 1) as f64 * sep_y + alto_n,
    )
}

pub fn render_bplus_tree(c: &mut Lienzo, b: &Bloque) -> Result<Punto, ErrorDiagrama> {
    let orden = entero_b(b, "order", 4, 3, 12)? as usize;
    let max_claves = entero_b(b, "maxKeys", orden as i64 - 1, 2, 11)? as usize;
    let claves = valores_num(b, "keys")?;
    if claves.len() > 60 {
        return Err(err("como máximo 60 claves"));
    }
    let pasos = bool_opt(b, "steps", false);
    let blanco = bool_opt(b, "blank", false);
    let mut t = BPlus::new(max_claves);
    let y0 = encabezado(
        c,
        b,
        &format!("orden {orden} (hasta {max_claves} claves por nodo)"),
    );
    let mut y = y0 + 6.0;
    let mut ancho: f64 = 0.0;
    let mut mostrados = 0;
    for &k in &claves {
        let antes = t.log.len();
        t.insertar(k);
        if pasos && t.log.len() > antes {
            mostrados += 1;
            if mostrados > 12 {
                return Err(err("demasiados pasos para dibujar (máximo 12 divisiones)"));
            }
            let titulo = format!("insertar {} → {}", fmt(k, 3), t.log[antes..].join("; "));
            c.t(0.0, y + 12.0, &titulo, 10.5, 700, "textSub", "start", SANS);
            let (w, h) = dibujar_bplus(c, &t, y + 20.0, false);
            ancho = ancho.max(w).max(ancho_texto(&titulo, 10.5));
            y += h + 36.0;
        }
    }
    if pasos {
        c.t(
            0.0,
            y + 12.0,
            "Árbol final",
            11.0,
            700,
            "textMain",
            "start",
            SANS,
        );
        y += 20.0;
    }
    let (w, h) = dibujar_bplus(c, &t, y, blanco);
    ancho = ancho.max(w);
    Ok((ancho, y + h + 6.0))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn heap_y_avl() {
        let mut h = vec![5.0, 3.0, 8.0, 1.0, 9.0, 2.0];
        heapify(&mut h, true);
        assert_eq!(h[0], 1.0);
        for i in 1..h.len() {
            assert!(h[(i - 1) / 2] <= h[i]);
        }
        let mut t = Avl::default();
        for k in [10.0, 20.0, 30.0] {
            t.insertar(k);
        }
        assert_eq!(t.n[t.raiz.unwrap()].clave, 20.0);
        assert_eq!(t.log, vec!["rotación simple a izquierda en 10"]);
        let mut t = Avl::default();
        for k in [30.0, 10.0, 20.0] {
            t.insertar(k);
        }
        assert_eq!(t.n[t.raiz.unwrap()].clave, 20.0);
        assert!(t.log[0].starts_with("rotación doble izquierda-derecha"));
        t.eliminar(30.0);
        t.eliminar(20.0);
        assert_eq!(t.n[t.raiz.unwrap()].clave, 10.0);
    }

    #[test]
    fn bplus_divide_y_crece() {
        let mut t = BPlus::new(3);
        for k in [10.0, 20.0, 30.0, 40.0] {
            t.insertar(k);
        }
        let r = &t.n[t.raiz];
        assert_eq!(r.claves, vec![30.0]);
        assert_eq!(t.niveles().len(), 2);
        for k in [50.0, 60.0, 70.0, 80.0, 90.0, 100.0] {
            t.insertar(k);
        }
        // Todas las claves aparecen en las hojas, en orden.
        let hojas: Vec<f64> = t
            .niveles()
            .last()
            .unwrap()
            .iter()
            .flat_map(|&i| t.n[i].claves.clone())
            .collect();
        assert_eq!(hojas, (1..=10).map(|x| x as f64 * 10.0).collect::<Vec<_>>());
        assert_eq!(t.niveles().len(), 3);
    }
}
