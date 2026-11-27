#![warn(clippy::missing_safety_doc)]

use clap::{Parser, Subcommand};
use dibu_core::{leer_bloques, nombre_svg, Bloque, ErrorDiagrama, Registro, Theme, PRESETS};
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
    /// Lista los presets de theme disponibles.
    ListThemes,
    /// Renderiza un archivo YAML/JSON con uno o varios bloques a SVG.
    Render {
        archivo: PathBuf,
        #[arg(short, long, default_value = "output")]
        output_dir: PathBuf,
        #[arg(long)]
        color: bool,
        /// Preset de theme (ink, color, high-contrast, dark) o ruta a archivo YAML/JSON.
        #[arg(long)]
        theme: Option<String>,
    },
    /// Modo lote por stdin/stdout (JSONL): 1 spawn por examen/documento.
    Batch {
        /// Theme base opcional para la sesión (puede sobreescribirse por ítem en JSONL).
        #[arg(long)]
        theme: Option<String>,
    },
    /// Sincroniza `<dir>/diagramas/estructura-<hash>.svg` a partir de archivos Markdown.
    SyncMd {
        #[arg(required = true)]
        rutas: Vec<PathBuf>,
        #[arg(long)]
        verificar: bool,
        #[arg(long)]
        color: bool,
        /// Preset de theme (ink, color, high-contrast, dark) o ruta a archivo YAML/JSON.
        #[arg(long)]
        theme: Option<String>,
    },
}

#[derive(Deserialize)]
struct BatchReq {
    id: String,
    block: Bloque,
    #[serde(default)]
    tinta: Option<bool>,
    #[serde(default)]
    theme: Option<serde_json::Value>,
}

fn cargar_tema(spec: Option<&str>, color: bool) -> Result<Theme, ErrorDiagrama> {
    let env_theme = std::env::var("DIBU_THEME").ok();
    let efectivo = spec
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .or_else(|| env_theme.as_deref().map(str::trim).filter(|s| !s.is_empty()));
    match efectivo {
        Some(s) => {
            let ruta = Path::new(s);
            if ruta.is_file() {
                let contenido = fs::read_to_string(ruta)
                    .map_err(|e| ErrorDiagrama::new(format!("no se pudo leer theme {s}: {e}")))?;
                Theme::from_yaml_or_json_str(&contenido)
            } else {
                Theme::preset(s)
            }
        }
        None => Ok(Theme::para_tinta(!color)),
    }
}

fn id_archivo_seguro(id: &str) -> bool {
    if id.is_empty() || id.contains('/') || id.contains('\\') || id.contains('\0') {
        return false;
    }
    let mut comps = Path::new(id).components();
    matches!(
        (comps.next(), comps.next()),
        (Some(std::path::Component::Normal(_)), None)
    )
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
    #[cfg(feature = "comp")]
    dibu_comp::registrar(&mut reg);
    #[cfg(feature = "math")]
    dibu_math::registrar(&mut reg);
    reg
}

fn extraer_cuerpos_markdown(texto: &str) -> Vec<String> {
    // Verificamos que la valla de cierre coincida con la de apertura.
    let texto = texto.strip_prefix('\u{feff}').unwrap_or(texto);
    let mut cuerpos = Vec::new();
    let lineas: Vec<&str> = texto.lines().collect();
    let mut i = 0;
    // ```dibu y los nombres heredados de statics y alucarD (```estructura, ```statics).
    let re_abre =
        Regex::new(r"^(?P<valla>`{3,}|~{3,})[ \t]*(?:dibu|estructura|statics)[ \t]*$").unwrap();
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

fn sincronizar_dir(reg: &Registro, dir: &Path, tema: &Theme, escribir: bool) -> (usize, usize) {
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
            match reg.renderizar_bloque_markdown_con_tema(&cuerpo, tema) {
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
                if let Err(e) = fs::create_dir_all(&destino).and_then(|_| fs::write(&ruta, svg)) {
                    eprintln!("Error: no se pudo escribir {}: {e}", ruta.display());
                    errores += 1;
                } else {
                    println!("✓ Escrito: {}", ruta.display());
                }
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
                    if let Err(e) = fs::remove_file(&ruta) {
                        eprintln!("Error: no se pudo borrar {}: {e}", ruta.display());
                        errores += 1;
                    } else {
                        println!("✓ Borrado: {}", ruta.display());
                    }
                } else {
                    println!("Sobra: {}", ruta.display());
                }
            }
        }
        if escribir && fs::read_dir(&destino).map(|mut it| it.next().is_none()).unwrap_or(false) {
            if let Err(e) = fs::remove_dir(&destino) {
                eprintln!("Error: no se pudo borrar directorio {}: {e}", destino.display());
                errores += 1;
            }
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
        Cmd::ListThemes => {
            for t in PRESETS {
                println!("{t}");
            }
            ExitCode::SUCCESS
        }
        Cmd::Render {
            archivo,
            output_dir,
            color,
            theme,
        } => {
            let tema_base = match cargar_tema(theme.as_deref(), color) {
                Ok(t) => t,
                Err(e) => {
                    eprintln!("Error: {e}");
                    return ExitCode::from(2);
                }
            };
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
            let mut error_seguridad = false;
            for (idx, bloque) in bloques.iter().enumerate() {
                let n = idx + 1;
                let tipo = bloque.get("type").and_then(|v| v.as_str()).unwrap_or("");
                if !reg.contiene(tipo) {
                    eprintln!("Aviso: bloque {n} ({tipo:?}) no soportado; se saltea.");
                    continue;
                }
                let id = match bloque
                    .get("id")
                    .and_then(|v| v.as_str())
                    .filter(|s| !s.is_empty())
                {
                    Some(raw_id) => {
                        if !id_archivo_seguro(raw_id) {
                            eprintln!(
                                "Error: bloque {n}: id inválido {raw_id:?} (no puede contener rutas ni '..')"
                            );
                            error_seguridad = true;
                            continue;
                        }
                        raw_id.to_string()
                    }
                    None => format!("{stem}_{n}"),
                };
                let destino = output_dir.join(format!("{id}.svg"));
                match reg.renderizar_con_tema(bloque, &tema_base) {
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
            if escritos > 0 && !error_seguridad {
                ExitCode::SUCCESS
            } else {
                ExitCode::FAILURE
            }
        }
        Cmd::Batch { theme } => {
            let tema_sesion = match cargar_tema(theme.as_deref(), false) {
                Ok(t) => t,
                Err(e) => {
                    eprintln!("Error: {e}");
                    return ExitCode::from(2);
                }
            };
            let stdin = io::stdin();
            let mut reader = io::BufReader::new(stdin.lock());
            let stdout = io::stdout();
            let mut out = io::BufWriter::new(stdout.lock());
            let mut buf = Vec::new();
            loop {
                buf.clear();
                match reader.read_until(b'\n', &mut buf) {
                    Ok(0) => break,
                    Ok(_) => {}
                    Err(e) => {
                        eprintln!("Error leyendo stdin: {e}");
                        break;
                    }
                }
                let resp = match std::str::from_utf8(&buf) {
                    Ok(linea) => {
                        let trimmed = linea.trim();
                        if trimmed.is_empty() {
                            continue;
                        }
                        match serde_json::from_str::<BatchReq>(trimmed) {
                            Ok(req) => {
                                let tema_res = if let Some(ref tv) = req.theme {
                                    let base = match req.tinta {
                                        Some(t) => Theme::para_tinta(t),
                                        None => tema_sesion.clone(),
                                    };
                                    Theme::from_value(tv, &base)
                                } else if let Some(t) = req.tinta {
                                    if theme.is_some() && t {
                                        Ok(tema_sesion.clone())
                                    } else {
                                        Ok(Theme::para_tinta(t))
                                    }
                                } else {
                                    Ok(tema_sesion.clone())
                                };
                                match tema_res.and_then(|t| reg.renderizar_con_tema(&req.block, &t)) {
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
                                }
                            }
                            Err(e) => BatchResp {
                                id: String::new(),
                                svg: None,
                                error: Some(format!("JSON inválido: {e}")),
                            },
                        }
                    }
                    Err(e) => BatchResp {
                        id: String::new(),
                        svg: None,
                        error: Some(format!("UTF-8 inválido: {e}")),
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
            theme,
        } => {
            let tema_base = match cargar_tema(theme.as_deref(), color) {
                Ok(t) => t,
                Err(e) => {
                    eprintln!("Error: {e}");
                    return ExitCode::from(2);
                }
            };
            let mut dirs: BTreeSet<PathBuf> = BTreeSet::new();
            let mut errores = 0usize;
            for r in rutas {
                if !r.exists() {
                    eprintln!("Error: ruta no encontrada: {}", r.display());
                    errores += 1;
                    continue;
                }
                if r.is_dir() {
                    match r.canonicalize() {
                        Ok(c) => {
                            dirs.insert(c);
                        }
                        Err(e) => {
                            eprintln!("Error: no se pudo resolver {}: {e}", r.display());
                            errores += 1;
                        }
                    }
                } else if r
                    .extension()
                    .and_then(|e| e.to_str())
                    .is_some_and(|e| e.eq_ignore_ascii_case("md"))
                {
                    let parent = match r.parent() {
                        Some(p) if !p.as_os_str().is_empty() => p,
                        _ => Path::new("."),
                    };
                    match parent.canonicalize() {
                        Ok(p) => {
                            dirs.insert(p);
                        }
                        Err(e) => {
                            eprintln!("Error: no se pudo resolver {}: {e}", r.display());
                            errores += 1;
                        }
                    }
                } else {
                    eprintln!(
                        "Aviso: {} no es un directorio ni un archivo .md; se ignora.",
                        r.display()
                    );
                }
            }
            let mut cambios = 0usize;
            for d in dirs {
                let (c, e) = sincronizar_dir(&reg, &d, &tema_base, !verificar);
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
