use dibu_core::nombre_svg;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::atomic::{AtomicU64, Ordering};

static SEQ: AtomicU64 = AtomicU64::new(0);

struct TempDir {
    path: PathBuf,
}

impl TempDir {
    fn new(prefix: &str) -> Self {
        let id = SEQ.fetch_add(1, Ordering::Relaxed);
        let path = std::env::temp_dir().join(format!(
            "dibu_sync_md_{prefix}_{}_{}",
            std::process::id(),
            id
        ));
        let _ = fs::remove_dir_all(&path);
        fs::create_dir_all(&path).expect("no se pudo crear el directorio temporal");
        Self { path }
    }

    fn path(&self) -> &Path {
        &self.path
    }
}

impl Drop for TempDir {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.path);
    }
}

fn run_dibu(cwd: &Path, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_dibu"))
        .current_dir(cwd)
        .args(args)
        .output()
        .expect("falló al ejecutar el binario dibu")
}

const BLOQUE_SIMPLE: &str = "type: block_diagram\nforward: [\"G(s)\"]\n";
const BLOQUE_REALIM: &str =
    "type: block_diagram\nforward: [\"Gc(s)\", \"Gp(s)\"]\nfeedback: [\"H(s)\"]\n";
const BLOQUE_APILADO: &str = "- type: block_diagram\n  forward: [\"A\"]\n- type: block_diagram\n  forward: [\"B\"]\n- type: block_diagram\n  forward: [\"C\"]\n";

#[test]
fn sync_md_archivo_relativo_sin_barra_en_cwd() {
    let tmp = TempDir::new("relativo");
    let md = format!("# Prueba\n\n```estructura\n{BLOQUE_SIMPLE}```\n");
    fs::write(tmp.path().join("prueba.md"), md).unwrap();

    // Primera corrida: crea diagramas/estructura-<sha1>.svg y retorna exit 1
    let out1 = run_dibu(tmp.path(), &["sync-md", "prueba.md"]);
    assert_eq!(out1.status.code(), Some(1));
    let stdout1 = String::from_utf8_lossy(&out1.stdout);
    let stderr1 = String::from_utf8_lossy(&out1.stderr);
    assert!(stdout1.contains("✓ Escrito:"), "stdout: {stdout1}");
    assert!(stderr1.contains("git add"), "stderr: {stderr1}");

    let esperado = tmp.path().join("diagramas").join(nombre_svg(BLOQUE_SIMPLE));
    assert!(esperado.is_file(), "falta SVG en {}", esperado.display());
    let svg = fs::read_to_string(&esperado).unwrap();
    assert!(svg.starts_with("<svg ") && svg.ends_with("</svg>"));

    // Segunda corrida e idempotencia con --verificar: exit 0
    let out2 = run_dibu(tmp.path(), &["sync-md", "prueba.md"]);
    assert_eq!(out2.status.code(), Some(0));

    let out_ver = run_dibu(tmp.path(), &["sync-md", "--verificar", "prueba.md"]);
    assert_eq!(out_ver.status.code(), Some(0));
}

#[test]
fn sync_md_verificar_modificado_no_escribe_cambios() {
    let tmp = TempDir::new("verificar");
    let md_path = tmp.path().join("guia.md");
    fs::write(&md_path, format!("```estructura\n{BLOQUE_SIMPLE}```\n")).unwrap();

    let init = run_dibu(tmp.path(), &["sync-md", "."]);
    assert_eq!(init.status.code(), Some(1));

    let svg_inicial_path = tmp.path().join("diagramas").join(nombre_svg(BLOQUE_SIMPLE));
    let svg_inicial_bytes = fs::read(&svg_inicial_path).unwrap();

    // Modificamos el markdown sin re-sincronizar
    fs::write(&md_path, format!("```estructura\n{BLOQUE_REALIM}```\n")).unwrap();
    let svg_nuevo_path = tmp.path().join("diagramas").join(nombre_svg(BLOQUE_REALIM));

    let ver = run_dibu(tmp.path(), &["sync-md", "--verificar", "."]);
    assert_eq!(ver.status.code(), Some(1));
    let stdout = String::from_utf8_lossy(&ver.stdout);
    let stderr = String::from_utf8_lossy(&ver.stderr);
    assert!(stdout.contains("Desactualizado:"), "stdout: {stdout}");
    assert!(stdout.contains("Sobra:"), "stdout: {stdout}");
    assert!(!stderr.contains("git add"), "stderr: {stderr}");

    // No debe haber escrito ni borrado nada en disco
    assert!(svg_inicial_path.exists());
    assert_eq!(fs::read(&svg_inicial_path).unwrap(), svg_inicial_bytes);
    assert!(!svg_nuevo_path.exists());
}

#[test]
fn sync_md_fences_estructura_statics_tilde_cuatro_backticks_y_sin_cerrar() {
    let tmp = TempDir::new("fences");
    let c1 = "type: block_diagram\nforward: [\"F1\"]\n";
    let c2 = "type: block_diagram\nforward: [\"F2\"]\n";
    let c3 = "type: block_diagram\nforward: [\"F3\"]\n";
    let c4 = "type: block_diagram\nforward: [\"F4\"]\n";
    let c5 = "type: block_diagram\n# ``` ejemplo interno\nforward: [\"F5\"]\n";
    let c_sin_cerrar = "type: block_diagram\nforward: [\"SIN_CERRAR\"]\n";

    let contenido = format!(
        "```estructura\n{c1}```\n\n\
         ```statics\n{c2}```\n\n\
         ~~~estructura\n{c3}~~~\n\n\
         ~~~statics\n{c4}~~~\n\n\
         ````estructura\n{c5}````\n\n\
         ```estructura\n{c_sin_cerrar}~~~\n"
    );
    fs::write(tmp.path().join("fences.md"), contenido).unwrap();

    let out = run_dibu(tmp.path(), &["sync-md", "fences.md"]);
    assert_eq!(out.status.code(), Some(1));

    let diagramas = tmp.path().join("diagramas");
    let mut archivos: Vec<String> = fs::read_dir(&diagramas)
        .unwrap()
        .map(|e| e.unwrap().file_name().to_string_lossy().into_owned())
        .collect();
    archivos.sort();

    let mut esperados = vec![
        nombre_svg(c1),
        nombre_svg(c2),
        nombre_svg(c3),
        nombre_svg(c4),
        nombre_svg(c5),
    ];
    esperados.sort();

    assert_eq!(archivos, esperados);
    assert!(!diagramas.join(nombre_svg(c_sin_cerrar)).exists());
}

#[test]
fn sync_md_bloques_apilados_en_unico_svg() {
    let tmp = TempDir::new("apilado");
    fs::write(
        tmp.path().join("apilado.md"),
        format!("```estructura\n{BLOQUE_APILADO}```\n"),
    )
    .unwrap();

    let out = run_dibu(tmp.path(), &["sync-md", "apilado.md"]);
    assert_eq!(out.status.code(), Some(1));

    let svg_path = tmp.path().join("diagramas").join(nombre_svg(BLOQUE_APILADO));
    let svg = fs::read_to_string(&svg_path).unwrap();
    // N = 3 bloques => 1 contenedor exterior + 3 hijos = 4 tags `<svg`
    assert_eq!(svg.matches("<svg").count(), 4);
}

#[test]
fn sync_md_paleta_color_vs_tinta() {
    let tmp = TempDir::new("color");
    fs::write(
        tmp.path().join("paleta.md"),
        format!("```estructura\n{BLOQUE_REALIM}```\n"),
    )
    .unwrap();

    let svg_path = tmp.path().join("diagramas").join(nombre_svg(BLOQUE_REALIM));

    // Tinta por defecto
    let out_tinta = run_dibu(tmp.path(), &["sync-md", "paleta.md"]);
    assert_eq!(out_tinta.status.code(), Some(1));
    let svg_tinta = fs::read_to_string(&svg_path).unwrap();
    assert!(svg_tinta.contains("#000000"));
    assert!(!svg_tinta.contains("#1d4ed8"));

    // Modo --color (tinta = false)
    let out_color = run_dibu(tmp.path(), &["sync-md", "--color", "paleta.md"]);
    assert_eq!(out_color.status.code(), Some(1));
    let svg_color = fs::read_to_string(&svg_path).unwrap();
    assert!(svg_color.contains("#1d4ed8"));
    assert!(svg_color.contains("#0f172a"));

    // Idempotente en modo --color
    let out_ver = run_dibu(tmp.path(), &["sync-md", "--color", "--verificar", "paleta.md"]);
    assert_eq!(out_ver.status.code(), Some(0));
}

#[test]
fn sync_md_limpia_obsoletos_remueve_dir_vacio_y_preserva_ajenos() {
    let tmp = TempDir::new("limpieza");
    let md_path = tmp.path().join("doc.md");
    fs::write(
        &md_path,
        format!("```estructura\n{BLOQUE_SIMPLE}```\n\n```estructura\n{BLOQUE_REALIM}```\n"),
    )
    .unwrap();

    assert_eq!(run_dibu(tmp.path(), &["sync-md", "."]).status.code(), Some(1));
    let diagramas = tmp.path().join("diagramas");
    assert!(diagramas.join(nombre_svg(BLOQUE_SIMPLE)).is_file());
    assert!(diagramas.join(nombre_svg(BLOQUE_REALIM)).is_file());

    // Quitamos todos los bloques => borra ambos SVGs y remueve diagramas/ vacío
    fs::write(&md_path, "# Sin diagramas\n").unwrap();
    let out_rm = run_dibu(tmp.path(), &["sync-md", "."]);
    assert_eq!(out_rm.status.code(), Some(1));
    assert!(String::from_utf8_lossy(&out_rm.stdout).contains("✓ Borrado:"));
    assert!(!diagramas.exists(), "diagramas/ vacío debía eliminarse");

    // Ahora probamos preservación de archivo ajeno dentro de diagramas/
    fs::write(&md_path, format!("```estructura\n{BLOQUE_SIMPLE}```\n")).unwrap();
    assert_eq!(run_dibu(tmp.path(), &["sync-md", "."]).status.code(), Some(1));
    let nota = diagramas.join("nota.txt");
    fs::write(&nota, "conservar").unwrap();

    fs::write(&md_path, "# Vacio otra vez\n").unwrap();
    let out_ajeno = run_dibu(tmp.path(), &["sync-md", "."]);
    assert_eq!(out_ajeno.status.code(), Some(1));
    assert!(!diagramas.join(nombre_svg(BLOQUE_SIMPLE)).exists());
    assert!(diagramas.is_dir(), "diagramas/ con nota.txt debe seguir existiendo");
    assert_eq!(fs::read_to_string(&nota).unwrap(), "conservar");
}

#[test]
fn sync_md_errores_controlados_bloques_invalidos_y_rutas_inexistentes() {
    let casos_bloque = [
        ("tipo_invalido", "```estructura\ntype: grafo\nvalues: [1, 2]\n```\n", "grafo"),
        ("yaml_roto", "```estructura\ntype: [sin cerrar\n```\n", "Error:"),
        ("bloque_vacio", "```estructura\n```\n", "Error:"),
    ];

    for (nombre, contenido, esperado_stderr) in casos_bloque {
        let tmp = TempDir::new(nombre);
        fs::write(tmp.path().join("mal.md"), contenido).unwrap();
        let out = run_dibu(tmp.path(), &["sync-md", "mal.md"]);
        assert_eq!(out.status.code(), Some(1), "caso {nombre}");
        let stderr = String::from_utf8_lossy(&out.stderr);
        assert!(stderr.contains("Error:"), "caso {nombre}, stderr: {stderr}");
        assert!(
            stderr.contains(esperado_stderr),
            "caso {nombre}, stderr: {stderr}"
        );
        assert!(
            !stderr.contains("panicked"),
            "caso {nombre} produjo panic: {stderr}"
        );
        assert!(!tmp.path().join("diagramas").exists());
    }

    // Rutas inexistentes: archivo y directorio
    let tmp_rutas = TempDir::new("rutas_inexistentes");
    for ruta in ["no_existe.md", "no_existe_dir/"] {
        let out = run_dibu(tmp_rutas.path(), &["sync-md", ruta]);
        assert_eq!(out.status.code(), Some(1), "ruta {ruta}");
        let stderr = String::from_utf8_lossy(&out.stderr);
        assert!(stderr.contains("Error:"), "ruta {ruta}, stderr: {stderr}");
        assert!(
            !stderr.contains("panicked"),
            "ruta {ruta} produjo panic: {stderr}"
        );
    }
}

#[test]
fn sync_md_deduplica_bloques_identicos_entre_archivos() {
    let tmp = TempDir::new("dedup");
    fs::write(
        tmp.path().join("a.md"),
        format!("```estructura\n{BLOQUE_SIMPLE}```\n"),
    )
    .unwrap();
    fs::write(
        tmp.path().join("b.md"),
        format!("```estructura\n\n{BLOQUE_SIMPLE}\n```\n"),
    )
    .unwrap();

    let out = run_dibu(tmp.path(), &["sync-md", "."]);
    assert_eq!(out.status.code(), Some(1));
    let entradas: Vec<_> = fs::read_dir(tmp.path().join("diagramas"))
        .unwrap()
        .collect();
    assert_eq!(entradas.len(), 1);
}
