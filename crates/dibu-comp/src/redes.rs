//! Redes: `subnetting` (subredes de igual tamaño o VLSM sobre IPv4).

use crate::comun::*;
use serde_json::Value;

pub fn ip_a_texto(ip: u32) -> String {
    format!(
        "{}.{}.{}.{}",
        ip >> 24,
        (ip >> 16) & 255,
        (ip >> 8) & 255,
        ip & 255
    )
}

pub fn parsear_ip(s: &str) -> Result<u32, ErrorDiagrama> {
    let partes: Vec<&str> = s.trim().split('.').collect();
    if partes.len() != 4 {
        return Err(err(format!("{s:?} no es una dirección IPv4")));
    }
    let mut ip = 0u32;
    for p in partes {
        let o: u8 = p
            .parse()
            .map_err(|_| err(format!("{s:?} no es una dirección IPv4")))?;
        ip = (ip << 8) | o as u32;
    }
    Ok(ip)
}

pub fn mascara(prefijo: u32) -> u32 {
    if prefijo == 0 {
        0
    } else {
        u32::MAX << (32 - prefijo)
    }
}

#[derive(Debug, Clone)]
pub struct Subred {
    pub nombre: String,
    pub pedidos: Option<u64>,
    pub red: u32,
    pub prefijo: u32,
}

impl Subred {
    pub fn tamano(&self) -> u64 {
        1u64 << (32 - self.prefijo)
    }
    pub fn broadcast(&self) -> u32 {
        (self.red as u64 + self.tamano() - 1) as u32
    }
    /// Hosts utilizables (las /31 y /32 son casos especiales: 2 y 1).
    pub fn utiles(&self) -> u64 {
        match self.prefijo {
            32 => 1,
            31 => 2,
            _ => self.tamano() - 2,
        }
    }
}

/// Prefijo mínimo para `hosts` hosts utilizables.
pub fn prefijo_para(hosts: u64) -> u32 {
    let mut p = 30;
    while p > 0 && (1u64 << (32 - p)) - 2 < hosts {
        p -= 1;
    }
    p
}

/// Asigna subredes VLSM en orden de tamaño decreciente (o en el orden dado).
pub fn vlsm(
    red: u32,
    prefijo: u32,
    pedidos: &[(String, u64)],
    ordenar: bool,
) -> Result<Vec<Subred>, ErrorDiagrama> {
    let mut orden: Vec<usize> = (0..pedidos.len()).collect();
    if ordenar {
        orden.sort_by_key(|&i| std::cmp::Reverse(pedidos[i].1));
    }
    let fin = red as u64 + (1u64 << (32 - prefijo));
    let mut actual = red as u64;
    let mut out = Vec::new();
    for i in orden {
        let (nombre, hosts) = &pedidos[i];
        let p = prefijo_para(*hosts);
        if p < prefijo {
            return Err(err(format!(
                "{nombre}: {hosts} hosts no entran en una /{prefijo}"
            )));
        }
        let tam = 1u64 << (32 - p);
        // Alinear al tamaño del bloque.
        actual = actual.div_ceil(tam) * tam;
        if actual + tam > fin {
            return Err(err(format!(
                "no queda espacio para {nombre} ({hosts} hosts, /{p}) dentro de {}/{prefijo}",
                ip_a_texto(red)
            )));
        }
        out.push(Subred {
            nombre: nombre.clone(),
            pedidos: Some(*hosts),
            red: actual as u32,
            prefijo: p,
        });
        actual += tam;
    }
    Ok(out)
}

pub fn render_subnetting(c: &mut Lienzo, b: &Bloque) -> Result<Punto, ErrorDiagrama> {
    let red_txt = str_b(b, "network", "");
    let (ip_txt, pref_txt) = red_txt
        .split_once('/')
        .ok_or_else(|| err("'network' tiene que ser A.B.C.D/prefijo (p. ej. 192.168.10.0/24)"))?;
    let prefijo: u32 = pref_txt
        .trim()
        .parse()
        .ok()
        .filter(|p| *p <= 30)
        .ok_or_else(|| err("el prefijo de 'network' va de 0 a 30"))?;
    let ip = parsear_ip(ip_txt)?;
    let red = ip & mascara(prefijo);
    let blanco = bool_opt(b, "blank", false);
    let binario = bool_opt(b, "binary", false);

    let subredes: Vec<Subred> = if let Some(Value::Array(lst)) = b.get("subnets") {
        if lst.len() > 64 {
            return Err(err("como máximo 64 subredes"));
        }
        let mut pedidos = Vec::new();
        for (k, v) in lst.iter().enumerate() {
            let (nombre, hosts) = match v {
                Value::Object(m) => (
                    m.get("name")
                        .map(valor_str)
                        .unwrap_or_else(|| format!("Red {}", k + 1)),
                    m.get("hosts")
                        .and_then(num)
                        .ok_or_else(|| err(format!("subnets[{k}] necesita 'hosts'")))?,
                ),
                _ => (
                    format!("Red {}", k + 1),
                    num(v)
                        .ok_or_else(|| err("cada subred es {name, hosts} o un número de hosts"))?,
                ),
            };
            if hosts < 1.0 || hosts > 1e9 || hosts.fract() != 0.0 {
                return Err(err(format!(
                    "subnets[{k}]: 'hosts' es un entero entre 1 y 10⁹"
                )));
            }
            pedidos.push((nombre, hosts as u64));
        }
        vlsm(red, prefijo, &pedidos, bool_opt(b, "sort", true))?
    } else {
        let n = entero_b(b, "count", 0, 1, 256)? as u32;
        if n == 0 {
            return Err(err(
                "falta 'subnets' (lista de {name, hosts}) o 'count' (subredes iguales)",
            ));
        }
        let bits = 32 - (n - 1).leading_zeros().min(32);
        let bits = if n == 1 { 0 } else { bits };
        let p = prefijo + bits;
        if p > 30 {
            return Err(err(format!("{n} subredes no entran en una /{prefijo}")));
        }
        (0..n)
            .map(|k| Subred {
                nombre: format!("Subred {k}"),
                pedidos: None,
                red: red + (k << (32 - p)),
                prefijo: p,
            })
            .collect()
    };

    let extra = format!(
        "{}/{prefijo} · máscara {}",
        ip_a_texto(red),
        ip_a_texto(mascara(prefijo))
    );
    let y0 = encabezado(c, b, &extra);
    let con_pedidos = subredes.iter().any(|s| s.pedidos.is_some());
    let mut enc: Vec<String> = vec!["Subred".into()];
    if con_pedidos {
        enc.push("Pedidos".into());
    }
    enc.extend(
        [
            "Útiles",
            "Red",
            "Prefijo",
            "Máscara",
            "Primer host",
            "Último host",
            "Broadcast",
        ]
        .iter()
        .map(|s| s.to_string()),
    );
    let filas: Vec<Vec<Celda>> = subredes
        .iter()
        .map(|s| {
            let mut f = vec![celda(&s.nombre).fuerte(true)];
            if con_pedidos {
                f.push(celda(s.pedidos.map(|h| h.to_string()).unwrap_or_default()));
            }
            let (primero, ultimo) = if s.prefijo >= 31 {
                (s.red, s.broadcast())
            } else {
                (s.red + 1, s.broadcast() - 1)
            };
            for t in [
                s.utiles().to_string(),
                ip_a_texto(s.red),
                format!("/{}", s.prefijo),
                ip_a_texto(mascara(s.prefijo)),
                ip_a_texto(primero),
                ip_a_texto(ultimo),
                ip_a_texto(s.broadcast()),
            ] {
                f.push(celda(t).en_blanco(blanco));
            }
            f
        })
        .collect();
    let (w, h) = tabla(c, 0.0, y0 + 4.0, &enc, &filas, &OpcTabla::default());
    let mut y = y0 + 4.0 + h;
    let total = 1u64 << (32 - prefijo);
    let usado: u64 = subredes.iter().map(|s| s.tamano()).sum();
    if bool_opt(b, "summary", true) {
        y += 18.0;
        let txt = if blanco {
            "Direcciones usadas: ____ de ____".to_string()
        } else {
            format!(
                "Direcciones asignadas: {usado} de {total} ({} % del bloque)",
                fmt(100.0 * usado as f64 / total as f64, 1)
            )
        };
        c.t(0.0, y, &txt, 10.5, 600, "textMain", "start", MONO);
    }
    let mut ancho = w;
    // Barra del espacio de direcciones.
    if bool_opt(b, "map", true) && !blanco {
        y += 14.0;
        let bw = w.max(420.0);
        c.r(0.0, y, bw, 20.0, "fillCell", 1.2, 0.0);
        for (k, s) in subredes.iter().enumerate() {
            let x1 = (s.red - red) as f64 / total as f64 * bw;
            let x2 = x1 + s.tamano() as f64 / total as f64 * bw;
            c.r(
                x1,
                y,
                (x2 - x1).max(1.0),
                20.0,
                if k % 2 == 0 {
                    "fillPointerBox"
                } else {
                    "fillHighlight"
                },
                1.0,
                0.0,
            );
            if ancho_texto(&s.nombre, 9.0) < x2 - x1 - 4.0 {
                c.t(
                    (x1 + x2) / 2.0,
                    y + 14.0,
                    &s.nombre,
                    9.0,
                    600,
                    "textMain",
                    "middle",
                    SANS,
                );
            }
        }
        c.t(
            0.0,
            y + 33.0,
            &ip_a_texto(red),
            9.0,
            400,
            "textMuted",
            "start",
            MONO,
        );
        c.t(
            bw,
            y + 33.0,
            &ip_a_texto((red as u64 + total - 1) as u32),
            9.0,
            400,
            "textMuted",
            "end",
            MONO,
        );
        y += 36.0;
        ancho = ancho.max(bw);
    }
    if binario && !blanco {
        y += 8.0;
        for s in &subredes {
            y += 15.0;
            let bits = format!("{:032b}", s.red);
            let partes: Vec<String> = (0..4).map(|k| bits[k * 8..k * 8 + 8].to_string()).collect();
            let txt = format!(
                "{:<12} {}  (red: {} bits · host: {} bits)",
                s.nombre,
                partes.join("."),
                s.prefijo,
                32 - s.prefijo
            );
            c.t(0.0, y, &txt, 9.5, 400, "textSub", "start", MONO);
            ancho = ancho.max(ancho_texto(&txt, 9.5));
        }
    }
    Ok((ancho, y + 8.0))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn vlsm_clasico() {
        let red = parsear_ip("192.168.1.0").unwrap();
        let s = vlsm(
            red,
            24,
            &[
                ("A".into(), 50),
                ("B".into(), 100),
                ("C".into(), 20),
                ("D".into(), 2),
            ],
            true,
        )
        .unwrap();
        let v: Vec<(String, String, u32)> = s
            .iter()
            .map(|x| (x.nombre.clone(), ip_a_texto(x.red), x.prefijo))
            .collect();
        assert_eq!(
            v,
            vec![
                ("B".into(), "192.168.1.0".into(), 25),
                ("A".into(), "192.168.1.128".into(), 26),
                ("C".into(), "192.168.1.192".into(), 27),
                ("D".into(), "192.168.1.224".into(), 30),
            ]
        );
        assert!(vlsm(red, 24, &[("X".into(), 300)], true).is_err());
        assert_eq!(prefijo_para(2), 30);
        assert_eq!(prefijo_para(254), 24);
        assert_eq!(prefijo_para(255), 23);
    }
}
