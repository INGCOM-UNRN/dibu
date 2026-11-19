use clap::{Parser, Subcommand};
use dibu_core::{leer_bloques, nombre_svg, Bloque, Registro};
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::io::{self, BufRead, Write};
use std::path::{Path, PathBuf};
use std::process::ExitCode;

#[derive(Parser)]
#[command(name = "dibu", version, about = "Generador rápido de diagramas SVG educativos")]
struct Cli {
    #[command(subcommand)]
    cmd: Cmd,
}

#[derive(Subcommand)]
enum Cmd {
    /// Lista los tipos de diagramas compilados en este binario.
    ListTypes,
    /// Renderiza un archivo YAML/JSON con uno o varios bloques a SVG.
    Render {
        archivo: PathBuf,
        #[arg(short, long, default_value = "output")]
        output_dir: PathBuf,
        #[arg(long)]
        color: bool,
    },
    /// Modo lote por stdin/stdout (JSONL): 1 spawn por examen/documento.
    Batch,
    /// Sincroniza `<dir>/diagramas/estructura-<hash>.svg` a partir de archivos Markdown.
    SyncMd {
        #[arg(required = true)]
        rutas: Vec<PathBuf>,
        #[arg(long)]
        verificar: bool,
        #[arg(long)]
        color: bool,
    },
}

#[derive(Deserialize)]
struct BatchReq {
    id: String,
    block: Bloque,
    #[serde(default = "default_true")]
    tinta: bool,
}

fn default_true() -> bool {
    true
}

#[derive(Serialize)]
struct BatchResp {
    id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    svg: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<String>,
}

fn construir_registro() -> Registro {
    let mut reg = Registro::new();
    #[cfg(feature = "cs")]
    dibu_cs::registrar(&mut reg);
    #[cfg(feature = "control")]
    dibu_control::registrar(&mut reg);
    #[cfg(feature = "chem")]
    dibu_chem::registrar(&mut reg);
    reg
}

fn extraer_cuerpos_markdown(texto: &str) -> Vec<String> {
    let re = Regex::new(
        r"(?ms)^(?P<valla>`{3,}|~{3,})[ \t]*(?:estructura|statics)[ \t]*\r?\n(?P<cuerpo>.*?)^[ \t]*(`{3,}|~{3,})[ \t]*$",
    )
    .unwrap();
    // Verificamos que la valla de cierre coincida con la de apertura.
    let mut cuerpos = Vec::new();
    let lineas: Vec<&str> = texto.lines().collect();
    let mut i = 0;
    let re_abre = Regex::new(r"^(?P<valla>`{3,}|~{3,})[ \t]*(?:estructura|statics)[ \t]*$").unwrap();
    let _ = re;
    while i < lineas.len() {
        if let Some(cap) = re_abre.captures(lineas[i]) {
            let valla = &cap["valla"];
            let mut cuerpo = Vec::new();
            i += 1;
            while i < lineas.len() && lineas[i].trim_end() != valla {
                cuerpo.push(lineas[i]);
                i += 1;
            }
            if i < lineas.len() {
                cuerpos.push(cuerpo.join("\n"));
            }
        }
        i += 1;
    }
    cuerpos
}

fn sincronizar_dir(reg: &Registro, dir: &Path, tinta: bool, escribir: bool) -> (usize, usize) {
    let destino = dir.join("diagramas");
    let mut esperados: BTreeMap<String, String> = BTreeMap::new();
    let mut errores = 0usize;
    let mut cambios = 0usize;

    let mut mds: Vec<PathBuf> = fs::read_dir(dir)
        .into_iter()
        .flatten()
        .flatten()
        .map(|e| e.path())
        .filter(|p| p.extension().and_then(|e| e.to_str()).is_some_and(|e| e.eq_ignore_ascii_case("md")))
        .collect();
    mds.sort();

    for md in mds {
        let Ok(contenido) = fs::read_to_string(&md) else {
            eprintln!("Error: no se pudo leer {}", md.display());
            errores += 1;
            continue;
        };
        for cuerpo in extraer_cuerpos_markdown(&contenido) {
            let nombre = nombre_svg(&cuerpo);
            if esperados.contains_key(&nombre) {
                continue;
            }
            match reg.renderizar_bloque_markdown(&cuerpo, tinta) {
                Ok(svg) => {
                    esperados.insert(nombre, svg);
                }
                Err(e) => {
                    eprintln!("Error: {}: {e}", md.display());
                    errores += 1;
                }
            }
        }
    }

    for (nombre, svg) in &esperados {
        let ruta = destino.join(nombre);
        let igual = fs::read_to_string(&ruta).ok().as_deref() == Some(svg.as_str());
        if !igual {
            cambios += 1;
            if escribir {
                let _ = fs::create_dir_all(&destino);
                let _ = fs::write(&ruta, svg);
                println!("✓ Escrito: {}", ruta.display());
            } else {
                println!("Desactualizado: {}", ruta.display());
            }
        }
    }

    if destino.is_dir() {
        let mut existentes: Vec<PathBuf> = fs::read_dir(&destino)
            .into_iter()
            .flatten()
            .flatten()
            .map(|e| e.path())
            .filter(|p| {
                p.file_name()
                    .and_then(|n| n.to_str())
                    .is_some_and(|n| n.starts_with("estructura-") && n.ends_with(".svg"))
            })
            .collect();
        existentes.sort();
        for ruta in existentes {
            let nombre = ruta.file_name().unwrap().to_string_lossy();
            if !esperados.contains_key(nombre.as_ref()) {
                cambios += 1;
                if escribir {
                    let _ = fs::remove_file(&ruta);
                    println!("✓ Borrado: {}", ruta.display());
                } else {
                    println!("Sobra: {}", ruta.display());
                }
            }
        }
        if escribir && fs::read_dir(&destino).map(|mut it| it.next().is_none()).unwrap_or(false) {
            let _ = fs::remove_dir(&destino);
        }
    }

    (cambios, errores)
}

fn main() -> ExitCode {
    let cli = Cli::parse();
    let reg = construir_registro();

    match cli.cmd {
        Cmd::ListTypes => {
            for t in reg.tipos() {
                println!("{t}");
            }
            ExitCode::SUCCESS
        }
        Cmd::Render {
            archivo,
            output_dir,
            color,
        } => {
            let texto = match fs::read_to_string(&archivo) {
                Ok(t) => t,
                Err(e) => {
                    eprintln!("Error leyendo {}: {e}", archivo.display());
                    return ExitCode::from(2);
                }
            };
            let bloques = match leer_bloques(&texto) {
                Ok(b) => b,
                Err(e) => {
                    eprintln!("{}: {e}", archivo.display());
                    return ExitCode::from(2);
                }
            };
            if let Err(e) = fs::create_dir_all(&output_dir) {
                eprintln!("Error creando {}: {e}", output_dir.display());
                return ExitCode::from(2);
            }
            let stem = archivo.file_stem().and_then(|s| s.to_str()).unwrap_or("bloque");
            let mut escritos = 0usize;
            for (idx, bloque) in bloques.iter().enumerate() {
                let n = idx + 1;
                let tipo = bloque.get("type").and_then(|v| v.as_str()).unwrap_or("");
                if !reg.contiene(tipo) {
                    eprintln!("Aviso: bloque {n} ({tipo:?}) no soportado; se saltea.");
                    continue;
                }
                let id = bloque
                    .get("id")
                    .and_then(|v| v.as_str())
                    .filter(|s| !s.is_empty())
                    .map(str::to_string)
                    .unwrap_or_else(|| format!("{stem}_{n}"));
                let destino = output_dir.join(format!("{id}.svg"));
                match reg.renderizar(bloque, !color) {
                    Ok(svg) => {
                        if let Err(e) = fs::write(&destino, svg) {
                            eprintln!("Error escribiendo {}: {e}", destino.display());
                            continue;
                        }
                        println!("✓ {}", destino.display());
                        escritos += 1;
                    }
                    Err(e) => eprintln!("Error: bloque {n}: {e}"),
                }
            }
            if escritos > 0 {
                ExitCode::SUCCESS
            } else {
                ExitCode::FAILURE
            }
        }
        Cmd::Batch => {
            let stdin = io::stdin();
            let stdout = io::stdout();
            let mut out = io::BufWriter::new(stdout.lock());
            for linea in stdin.lock().lines() {
                let Ok(linea) = linea else { break };
                let trimmed = linea.trim();
                if trimmed.is_empty() {
                    continue;
                }
                let resp = match serde_json::from_str::<BatchReq>(trimmed) {
                    Ok(req) => match reg.renderizar(&req.block, req.tinta) {
                        Ok(svg) => BatchResp {
                            id: req.id,
                            svg: Some(svg),
                            error: None,
                        },
                        Err(e) => BatchResp {
                            id: req.id,
                            svg: None,
                            error: Some(e.to_string()),
                        },
                    },
                    Err(e) => BatchResp {
                        id: String::new(),
                        svg: None,
                        error: Some(format!("JSON inválido: {e}")),
                    },
                };
                if writeln!(out, "{}", serde_json::to_string(&resp).unwrap()).is_err() {
                    break;
                }
            }
            let _ = out.flush();
            ExitCode::SUCCESS
        }
        Cmd::SyncMd {
            rutas,
            verificar,
            color,
        } => {
            let mut dirs: BTreeSet<PathBuf> = BTreeSet::new();
            for r in rutas {
                if r.is_dir() {
                    if let Ok(c) = r.canonicalize() {
                        dirs.insert(c);
                    }
                } else if r.extension().and_then(|e| e.to_str()).is_some_and(|e| e.eq_ignore_ascii_case("md")) {
                    if let Some(p) = r.parent().and_then(|p| p.canonicalize().ok()) {
                        dirs.insert(p);
                    }
                }
            }
            let mut cambios = 0usize;
            let mut errores = 0usize;
            for d in dirs {
                let (c, e) = sincronizar_dir(&reg, &d, !color, !verificar);
                cambios += c;
                errores += e;
            }
            if cambios > 0 && !verificar {
                eprintln!("Se actualizaron los SVG de los diagramas: agregalos al commit (git add).");
            }
            if cambios > 0 || errores > 0 {
                ExitCode::FAILURE
            } else {
                ExitCode::SUCCESS
            }
        }
    }
}
