use serde_json::Value;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

struct TempDir(PathBuf);

impl TempDir {
    fn new(tag: &str) -> Self {
        let p = std::env::temp_dir().join(format!("dibu_theme_{tag}_{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&p);
        std::fs::create_dir_all(&p).unwrap();
        Self(p)
    }
    fn path(&self) -> &Path {
        &self.0
    }
}

impl Drop for TempDir {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

fn bin() -> &'static str {
    env!("CARGO_BIN_EXE_dibu")
}

#[test]
fn list_themes_expone_los_cuatro_presets() {
    let out = Command::new(bin()).arg("list-themes").output().unwrap();
    assert!(out.status.success());
    let lineas: Vec<&str> = std::str::from_utf8(&out.stdout).unwrap().lines().collect();
    assert_eq!(lineas, vec!["ink", "color", "high-contrast", "dark"]);
}

#[test]
fn render_con_preset_dark_y_archivo_yaml() {
    let dir = TempDir::new("render");
    let diag = dir.path().join("arr.yaml");
    std::fs::write(&diag, "id: arr\ntype: array\nvalues: [1, 2, 3]\n").unwrap();

    let out_dark = dir.path().join("dark");
    let st = Command::new(bin())
        .args(["render", "--theme", "dark", "-o"])
        .arg(&out_dark)
        .arg(&diag)
        .output()
        .unwrap();
    assert!(st.status.success(), "{}", String::from_utf8_lossy(&st.stderr));
    let svg_dark = std::fs::read_to_string(out_dark.join("arr.svg")).unwrap();
    assert!(svg_dark.contains(r##"fill="#0f172a""##));
    assert!(svg_dark.contains(r##"fill="#f8fafc""##));

    let tema_yaml = dir.path().join("tema.yaml");
    std::fs::write(
        &tema_yaml,
        "extends: ink\ncolors:\n  background: \"#fafaf9\"\n  strokeMain: \"#1b365d\"\ntypography:\n  sans: \"Liberation Sans, sans-serif\"\n  mono: \"Liberation Mono, monospace\"\n  scale: 1.2\n",
    )
    .unwrap();
    let out_custom = dir.path().join("custom");
    let st2 = Command::new(bin())
        .args(["render", "--theme"])
        .arg(&tema_yaml)
        .arg("-o")
        .arg(&out_custom)
        .arg(&diag)
        .output()
        .unwrap();
    assert!(st2.status.success(), "{}", String::from_utf8_lossy(&st2.stderr));
    let svg_custom = std::fs::read_to_string(out_custom.join("arr.svg")).unwrap();
    assert!(svg_custom.contains(r##"fill="#fafaf9""##));
    assert!(svg_custom.contains(r##"stroke="#1b365d""##));
    assert!(svg_custom.contains("Liberation Mono, monospace"));
}

#[test]
fn batch_soporta_theme_preset_inline_y_error_controlado() {
    let mut child = Command::new(bin())
        .arg("batch")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();
    {
        let stdin = child.stdin.as_mut().unwrap();
        writeln!(
            stdin,
            r#"{{"id":"hc","block":{{"type":"array","values":[1,2]}},"theme":"high-contrast"}}"#
        )
        .unwrap();
        writeln!(
            stdin,
            r#"{{"id":"custom","block":{{"type":"array","values":[1,2]}},"theme":{{"extends":"ink","typography":{{"mono":"Fira Code, monospace"}}}}}}"#
        )
        .unwrap();
        writeln!(
            stdin,
            r#"{{"id":"bad","block":{{"type":"array","values":[1,2]}},"theme":"no-existe"}}"#
        )
        .unwrap();
    }
    let out = child.wait_with_output().unwrap();
    assert!(out.status.success());
    let lineas: Vec<Value> = std::str::from_utf8(&out.stdout)
        .unwrap()
        .lines()
        .map(|l| serde_json::from_str(l).unwrap())
        .collect();
    assert_eq!(lineas.len(), 3);

    let hc_svg = lineas[0]["svg"].as_str().unwrap();
    assert!(!hc_svg.contains("#334155") && !hc_svg.contains("#475569") && !hc_svg.contains("#64748b"));

    let custom_svg = lineas[1]["svg"].as_str().unwrap();
    assert!(custom_svg.contains("Fira Code, monospace"));

    assert!(lineas[2]["error"].as_str().unwrap().contains("desconocido"));
}
