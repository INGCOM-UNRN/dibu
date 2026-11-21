use dibu_core::{leer_bloques, nombre_svg, Bloque, Registro};
use dibu_cs::algos::{campos_struct_desde_strs, disposicion_struct};
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicU64, Ordering};

const ALUCARD_ROOT: &str = "/home/mrtin/dev/tools/docente/contenidos/alucarD";
const ALUCARD_PYTHON: &str = "/home/mrtin/dev/tools/docente/contenidos/alucarD/.venv/bin/python";
const FIXTURES_CS_YAML: &str = include_str!("../../../fixtures/diagramas_cs.yaml");

const TIPOS_CS: [&str; 14] = [
    "array",
    "matrix",
    "linked_list",
    "stack",
    "call_stack",
    "call_tree",
    "queue",
    "tree",
    "struct",
    "adt",
    "complexity",
    "flowchart",
    "cost_table",
    "trace_table",
];

static TEMP_SEQ: AtomicU64 = AtomicU64::new(0);

struct TempDir {
    path: PathBuf,
}

impl TempDir {
    fn new(prefix: &str) -> Self {
        let seq = TEMP_SEQ.fetch_add(1, Ordering::Relaxed);
        let path = std::env::temp_dir().join(format!(
            "dibu_paridad_cs_{prefix}_{}_{}",
            std::process::id(),
            seq
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

fn registro_cs() -> Registro {
    let mut reg = Registro::new();
    dibu_cs::registrar(&mut reg);
    reg
}

fn ejecutar_python_json(script: &str, stdin_json: &str) -> Value {
    let mut child = Command::new(ALUCARD_PYTHON)
        .args(["-c", script])
        .env("PYTHONDONTWRITEBYTECODE", "1")
        .env("PYTHONPATH", ALUCARD_ROOT)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("falló al lanzar python de alucarD");

    {
        let stdin = child.stdin.as_mut().expect("stdin disponible");
        stdin
            .write_all(stdin_json.as_bytes())
            .expect("falló al escribir stdin a python");
    }

    let out = child
        .wait_with_output()
        .expect("falló al esperar salida de python");
    if !out.status.success() {
        panic!(
            "Python falló (status={:?}):\nstdout:\n{}\nstderr:\n{}",
            out.status.code(),
            String::from_utf8_lossy(&out.stdout),
            String::from_utf8_lossy(&out.stderr)
        );
    }
    serde_json::from_slice(&out.stdout).expect("salida Python no es JSON válido")
}

fn normalizar_marcadores(svg: &str) -> String {
    let re_id = regex::Regex::new(r"flecha-(main|sub)-[0-9a-f]{8}").unwrap();
    re_id.replace_all(svg, "flecha-$1-HASH").into_owned()
}

#[test]
fn paridad_14_tipos_fixtures_cs_contra_alucard_en_ambas_paletas_y_determinismo() {
    let reg = registro_cs();
    let bloques = leer_bloques(FIXTURES_CS_YAML).expect("fixtures/diagramas_cs.yaml debe parsear");
    assert!(
        bloques.len() >= 98,
        "se esperaban >= 98 casos válidos de test_diagramas.py, hay {}",
        bloques.len()
    );

    let mut ids_vistos: BTreeSet<String> = BTreeSet::new();
    let tipos_presentes: BTreeSet<String> = bloques
        .iter()
        .filter_map(|b| b.get("type").and_then(Value::as_str).map(str::to_string))
        .collect();
    for tipo in TIPOS_CS {
        assert!(
            tipos_presentes.contains(tipo),
            "falta el tipo {tipo} en fixtures/diagramas_cs.yaml"
        );
    }

    let mut casos_rust = Vec::with_capacity(bloques.len() * 2);
    for (idx, bloque) in bloques.iter().enumerate() {
        let id = bloque
            .get("id")
            .and_then(Value::as_str)
            .filter(|s| !s.is_empty())
            .map(str::to_string)
            .unwrap_or_else(|| panic!("bloque {idx} sin id válido en fixtures/diagramas_cs.yaml"));
        assert!(
            ids_vistos.insert(id.clone()),
            "id duplicado en fixtures/diagramas_cs.yaml: {id}"
        );
        let tipo = bloque.get("type").and_then(Value::as_str).unwrap_or("");

        let mut svg_por_paleta: [Option<String>; 2] = [None, None];
        for (p_idx, tinta) in [true, false].into_iter().enumerate() {
            let svg1 = reg
                .renderizar(bloque, tinta)
                .unwrap_or_else(|e| panic!("dibu-cs falló en {id} (tinta={tinta}): {e}"));
            let svg2 = reg
                .renderizar(bloque, tinta)
                .unwrap_or_else(|e| panic!("segunda corrida falló en {id} (tinta={tinta}): {e}"));

            // Determinismo byte a byte
            assert_eq!(
                svg1, svg2,
                "no determinismo byte a byte en {id} (tinta={tinta})"
            );
            assert!(
                svg1.starts_with("<svg ") && svg1.ends_with("</svg>"),
                "SVG mal delimitado en {id} (tinta={tinta})"
            );
            assert!(
                !svg1.contains(">NaN<") && !svg1.contains("=\"NaN\"") && !svg1.contains("Infinity"),
                "NaN/Infinity detectado en {id} (tinta={tinta})"
            );

            svg_por_paleta[p_idx] = Some(svg1.clone());
            casos_rust.push(serde_json::json!({
                "id": id,
                "tinta": tinta,
                "block": bloque,
                "rust_svg_norm": normalizar_marcadores(&svg1),
                "rust_svg_raw": svg1,
            }));
        }

        let svg_tinta = svg_por_paleta[0].as_deref().unwrap();
        let svg_color = svg_por_paleta[1].as_deref().unwrap();
        assert_ne!(
            svg_tinta, svg_color,
            "la paleta tinta y la paleta color no deben producir el mismo SVG en {id}"
        );
        if tipo == "linked_list" {
            assert!(
                !svg_tinta.contains("#1d4ed8") && svg_color.contains("#1d4ed8"),
                "{id}: se esperaba #1d4ed8 ausente en tinta y presente en color"
            );
        }
    }

    let py_script = r#"
import json
import re
import sys
import xml.etree.ElementTree as ET
from collections import Counter
from generador_examenes.diagramas import renderizar

SVG_NS = "{http://www.w3.org/2000/svg}"
RE_MARCADOR = re.compile(r"flecha-(main|sub)-[0-9a-f]{8}")

def analizar(svg_txt):
    raiz = ET.fromstring(svg_txt)
    assert raiz.tag == f"{SVG_NS}svg", f"root tag inesperado: {raiz.tag}"
    conteos = Counter()
    textos = []
    markers = []
    color_attrs = []
    for el in raiz.iter():
        tag = el.tag.replace(SVG_NS, "")
        conteos[tag] += 1
        fill = el.get("fill")
        stroke = el.get("stroke")
        color = el.get("color")
        stop_color = el.get("stop-color")
        if any(v is not None for v in (fill, stroke, color, stop_color)):
            color_attrs.append([tag, fill, stroke, color, stop_color])
        if tag == "text":
            txt = "".join(el.itertext()).replace("\xa0", " ")
            textos.append(txt)
        elif tag == "marker":
            mid = el.get("id", "")
            markers.append(RE_MARCADOR.sub(r"flecha-\1-HASH", mid))
    return {
        "width": raiz.get("width"),
        "height": raiz.get("height"),
        "viewBox": raiz.get("viewBox"),
        "counts": dict(sorted(conteos.items())),
        "texts": textos,
        "markers": sorted(markers),
        "color_attrs": color_attrs,
    }

entrada = json.load(sys.stdin)
resultados = []
for item in entrada:
    py_svg = renderizar(item["block"], tinta=item["tinta"])
    py_info = analizar(py_svg)
    rust_info = analizar(item["rust_svg_raw"])
    # Validar también que la normalización de marcadores no dejó hashes sueltos en Rust
    assert RE_MARCADOR.search(item["rust_svg_norm"]) is None
    resultados.append({
        "id": item["id"],
        "tinta": item["tinta"],
        "py": py_info,
        "rust": rust_info,
    })
print(json.dumps(resultados, ensure_ascii=False))
"#;

    let stdin_json = serde_json::to_string(&casos_rust).unwrap();
    let salida = ejecutar_python_json(py_script, &stdin_json);
    let items = salida.as_array().expect("lista JSON de resultados");
    assert_eq!(items.len(), casos_rust.len());

    let mut diferencias = Vec::new();
    for item in items {
        let id = item["id"].as_str().unwrap();
        let tinta = item["tinta"].as_bool().unwrap();
        let py = &item["py"];
        let rust = &item["rust"];

        if py["width"] != rust["width"]
            || py["height"] != rust["height"]
            || py["viewBox"] != rust["viewBox"]
        {
            diferencias.push(format!(
                "[{id} tinta={tinta}] dimensiones distintas: py=({}, {}, {}) vs rust=({}, {}, {})",
                py["width"], py["height"], py["viewBox"], rust["width"], rust["height"], rust["viewBox"]
            ));
        }

        if py["texts"] != rust["texts"] {
            diferencias.push(format!(
                "[{id} tinta={tinta}] textos distintos:\n  py:   {}\n  rust: {}",
                py["texts"], rust["texts"]
            ));
        }

        if py["counts"] != rust["counts"] {
            diferencias.push(format!(
                "[{id} tinta={tinta}] conteos SVG distintos:\n  py:   {}\n  rust: {}",
                py["counts"], rust["counts"]
            ));
        }

        if py["markers"] != rust["markers"] {
            diferencias.push(format!(
                "[{id} tinta={tinta}] marcadores SVG distintos:\n  py:   {}\n  rust: {}",
                py["markers"], rust["markers"]
            ));
        }

        if py["color_attrs"] != rust["color_attrs"] {
            diferencias.push(format!(
                "[{id} tinta={tinta}] atributos de color (fill/stroke/color/stop-color) distintos:\n  py:   {}\n  rust: {}",
                py["color_attrs"], rust["color_attrs"]
            ));
        }
    }

    let (pares, resto) = items.as_chunks::<2>();
    assert!(resto.is_empty(), "se esperaba cantidad par de resultados");
    for [item_tinta, item_color] in pares {
        assert_eq!(item_tinta["id"], item_color["id"]);
        assert_eq!(item_tinta["tinta"], Value::Bool(true));
        assert_eq!(item_color["tinta"], Value::Bool(false));
        let id = item_tinta["id"].as_str().unwrap();
        if item_tinta["rust"]["color_attrs"] == item_color["rust"]["color_attrs"] {
            diferencias.push(format!(
                "[{id}] color_attrs idénticos entre paleta tinta y paleta color"
            ));
        }
    }

    assert!(
        diferencias.is_empty(),
        "Se encontraron {} diferencias de paridad:\n{}",
        diferencias.len(),
        diferencias.join("\n")
    );
}

#[test]
fn errores_controlados_en_14_tipos_con_prefijo_unico() {
    let reg = registro_cs();

    let casos_error: Vec<(&str, &str, &str)> = vec![
        // array (3)
        ("array", r#"{"type":"array","values":"1 2 3"}"#, "'values' tiene que ser una lista"),
        ("array", r#"{"type":"array","values":5}"#, "'values' tiene que ser una lista"),
        ("array", r#"{"type":"array","values":{"a":1}}"#, "'values' tiene que ser una lista"),
        // matrix (14)
        ("matrix", r#"{"type":"matrix","variant":"triangular","values":[[1]]}"#, "variante de matriz"),
        ("matrix", r#"{"type":"matrix","values":"1 2 3"}"#, "lista de filas"),
        ("matrix", r#"{"type":"matrix","values":[[1]],"highlight":"a,b"}"#, "highlight"),
        ("matrix", r#"{"type":"matrix","values":[[1],[1],[1],[1],[1],[1],[1],[1],[1],[1],[1]]}"#, "11 filas; el máximo es 10"),
        ("matrix", r#"{"type":"matrix","values":[[1,2],[0,1,2,3,4,5,6,7,8,9,10]]}"#, "fila 1 de la matriz tiene 11 columnas"),
        ("matrix", r#"{"type":"matrix","values":[0,1,2,3,4,5,6,7,8,9,10,11,12,13,14,15,16,17,18,19,20,21,22,23,24,25,26,27,28,29],"cols":2}"#, "15 filas"),
        ("matrix", r#"{"type":"matrix","values":[[1]],"rows":11}"#, "'rows' de una matriz tiene que estar entre 1 y 10: 11"),
        ("matrix", r#"{"type":"matrix","values":[[1]],"cols":12}"#, "'cols' de una matriz tiene que estar entre 1 y 10: 12"),
        ("matrix", r#"{"type":"matrix","values":[[1]],"cols":0}"#, "'cols'"),
        ("matrix", r#"{"type":"matrix","values":[[1]],"rows":"muchas"}"#, "tiene que ser un entero"),
        ("matrix", r#"{"type":"matrix","variant":"dynamic","values":[[1],[1],[1],[1],[1],[1],[1],[1],[1],[1],[1]]}"#, "11 filas"),
        ("matrix", r#"{"type":"matrix","variant":"dynamic","values":[[1]],"cols":11}"#, "'cols'"),
        ("matrix", r#"{"type":"matrix","variant":"flat","rows":5,"cols":5}"#, "máximo"),
        ("matrix", r#"{"type":"matrix","variant":"jagged","name":"filas","values":[[4,8],[9]],"owner":{"name":"d","fields":["x -> otra"]}}"#, "se espera rows"),
        // linked_list (7)
        ("linked_list", r#"{"type":"linked_list","values":5}"#, "'values' tiene que ser una lista"),
        ("linked_list", r#"{"type":"linked_list","values":[1,2],"addresses":"0x1000"}"#, "'addresses' tiene que ser una lista"),
        ("linked_list", r#"{"type":"linked_list","values":[1,2],"header":{"name":"l"}}"#, "necesita 'fields'"),
        ("linked_list", r#"{"type":"linked_list","values":[1,2],"header":{"fields":[]}}"#, "necesita 'fields'"),
        ("linked_list", r#"{"type":"linked_list","values":[1,2],"header":"texto"}"#, "necesita 'fields'"),
        ("linked_list", r#"{"type":"linked_list","values":[1,2],"header":{"name":"l","fields":["ultimo -> 3"]}}"#, "apunta al nodo"),
        ("linked_list", r#"{"type":"linked_list","values":[1,2],"header":{"fields":["sin separador"]}}"#, "campo inválido"),
        // stack (2)
        ("stack", r#"{"type":"stack","values":5}"#, "'values' tiene que ser una lista"),
        ("stack", r#"{"type":"stack","layout":"vertical_multi","values":[1],"traceLabels":"mal"}"#, "'traceLabels' tiene que ser una lista"),
        // call_stack (6)
        ("call_stack", r#"{"type":"call_stack","frames":[]}"#, "lista de marcos"),
        ("call_stack", r#"{"type":"call_stack","frames":[{"vars":{"x":1}}]}"#, "necesita 'function'"),
        ("call_stack", r#"{"type":"call_stack","frames":[{"function":"main","vars":{"x":1}}],"pointers":"0.x -> 0.z"}"#, "no tiene la variable 'z'"),
        ("call_stack", r#"{"type":"call_stack","frames":[{"function":"main","vars":{"x":1}}],"pointers":"0.x"}"#, "flecha"),
        ("call_stack", r#"{"type":"call_stack","frames":[{"function":"f"},{"function":"f"},{"function":"f"},{"function":"f"},{"function":"f"},{"function":"f"},{"function":"f"},{"function":"f"},{"function":"f"},{"function":"f"},{"function":"f"}]}"#, "el máximo es 10"),
        ("call_stack", r#"{"type":"call_stack","steps":[{"frames":[{"function":"m"}]},{"frames":[{"function":"m"}]},{"frames":[{"function":"m"}]},{"frames":[{"function":"m"}]},{"frames":[{"function":"m"}]},{"frames":[{"function":"m"}]},{"frames":[{"function":"m"}]},{"frames":[{"function":"m"}]},{"frames":[{"function":"m"}]}]}"#, "el máximo es 8"),
        // call_tree (4)
        ("call_tree", r#"{"type":"call_tree"}"#, "root"),
        ("call_tree", r#"{"type":"call_tree","root":{"returns":1}}"#, "call"),
        ("call_tree", r#"{"type":"call_tree","root":{"call":"f","children":["g","g","g","g","g","g","g","g","g","g","g","g","g","g","g","g","g","g","g","g","g","g","g","g","g","g","g","g","g","g","g","g","g","g","g","g","g","g","g","g"]}}"#, "máximo"),
        ("call_tree", r#"{"type":"call_tree","root":{"call":"f"},"levelCosts":5}"#, "'levelCosts'"),
        // queue (2)
        ("queue", r#"{"type":"queue","values":5}"#, "'values' tiene que ser una lista"),
        ("queue", r#"{"type":"queue","values":"abc"}"#, "'values' tiene que ser una lista"),
        // tree (2)
        ("tree", r#"{"type":"tree","values":[]}"#, "no tiene nodos"),
        ("tree", r#"{"type":"tree","insert":"no_lista"}"#, "'insert' tiene que ser una lista"),
        // struct (6)
        ("struct", r#"{"type":"struct"}"#, "fields"),
        ("struct", r#"{"type":"struct","fields":["punto_t p"]}"#, "tamaño"),
        ("struct", r#"{"type":"struct","fields":["?"]}"#, "no entiendo"),
        ("struct", r#"{"type":"struct","fields":[{"name":"x"}]}"#, "'type'"),
        ("struct", r#"{"type":"struct","variant":"otra","fields":["int a"]}"#, "variante"),
        ("struct", r#"{"type":"struct","fields":["int a","int a","int a","int a","int a","int a","int a","int a","int a","int a","int a","int a","int a","int a","int a","int a","int a"]}"#, "máximo"),
        // adt (2)
        ("adt", r#"{"type":"adt"}"#, "operations"),
        ("adt", r#"{"type":"adt","operations":[{"kind":"x"}]}"#, "signature"),
        // complexity (6)
        ("complexity", r#"{"type":"complexity","curves":["n^4"]}"#, "desconocido"),
        ("complexity", r#"{"type":"complexity","curves":[{"label":"x"}]}"#, "points"),
        ("complexity", r#"{"type":"complexity","curves":[{"points":[[1]]}]}"#, "points"),
        ("complexity", r#"{"type":"complexity","curves":5}"#, "curves"),
        ("complexity", r#"{"type":"complexity","variant":"otra"}"#, "variante"),
        ("complexity", r#"{"type":"complexity","variant":"classes","classes":["P"],"items":{"NP":["SAT"]}}"#, "NP"),
        // flowchart (3)
        ("flowchart", r#"{"type":"flowchart"}"#, "steps"),
        ("flowchart", r#"{"type":"flowchart","steps":[{"x":1}]}"#, "paso 0"),
        ("flowchart", r#"{"type":"flowchart","steps":[{"while":"a"}]}"#, "cuerpo"),
        // cost_table (2)
        ("cost_table", r#"{"type":"cost_table"}"#, "lines"),
        ("cost_table", r#"{"type":"cost_table","lines":"mal"}"#, "lines"),
        // trace_table (3)
        ("trace_table", r#"{"type":"trace_table"}"#, "filas"),
        ("trace_table", r#"{"type":"trace_table","columns":["a"]}"#, "filas"),
        ("trace_table", r#"{"type":"trace_table","columns":["a"],"rows":[[1,2]]}"#, "columnas"),
    ];

    assert!(
        casos_error.len() >= 34,
        "se requieren >= 34 casos de error, hay {}",
        casos_error.len()
    );

    let mut tipos_cubiertos: BTreeSet<&str> = BTreeSet::new();
    for (idx, (tipo, json_str, subcadena)) in casos_error.iter().enumerate() {
        tipos_cubiertos.insert(*tipo);
        let bloque: Bloque = serde_json::from_str(json_str)
            .unwrap_or_else(|e| panic!("JSON inválido en caso {idx}: {e}"));
        let err = reg
            .renderizar(&bloque, true)
            .expect_err(&format!("caso {idx} ({tipo}: {json_str}) debía dar error"));

        let prefijo = format!("{tipo}: ");
        let doble_prefijo = format!("{tipo}: {tipo}: ");
        assert!(
            err.0.starts_with(&prefijo),
            "caso {idx} ({tipo}): el error {:?} no empieza con {:?}",
            err.0,
            prefijo
        );
        assert!(
            !err.0.starts_with(&doble_prefijo),
            "caso {idx} ({tipo}): el error {:?} tiene doble prefijo {:?}",
            err.0,
            doble_prefijo
        );
        assert!(
            err.0.contains(subcadena),
            "caso {idx} ({tipo}): el error {:?} no contiene {:?}",
            err.0,
            subcadena
        );
    }

    for tipo in TIPOS_CS {
        assert!(
            tipos_cubiertos.contains(tipo),
            "falta cubrir casos de error para el tipo {tipo}"
        );
    }

    // Casos de resiliencia (sin duplicar los de `casos_error`): no producen panic, o renderizan SVG válido o dan ErrorDiagrama prefijado
    let resiliencia = [
        ("adt", r#"{"type":"adt","operations":[1,2],"representations":5}"#),
        ("call_tree", r#"{"type":"call_tree","root":{"call":"f"},"total":5}"#),
        ("complexity", r#"{"type":"complexity","variant":"classes","classes":["P"],"items":{"P":3},"maxN":[]}"#),
        ("array", r#"{"type":"array","values":[1],"pointers":123}"#),
    ];
    let casos_error_jsons: BTreeSet<&str> = casos_error.iter().map(|(_, j, _)| *j).collect();
    for (tipo, json_str) in resiliencia {
        assert!(
            !casos_error_jsons.contains(json_str),
            "caso de resiliencia duplicado en casos_error: {json_str}"
        );
        let bloque: Bloque = serde_json::from_str(json_str).unwrap();
        match reg.renderizar(&bloque, true) {
            Ok(svg) => assert!(svg.starts_with("<svg ") && svg.ends_with("</svg>")),
            Err(e) => assert!(e.0.starts_with(&format!("{tipo}: "))),
        }
    }
}

#[test]
fn struct_lp64_coincide_con_gcc_y_svg() {
    if Command::new("gcc").arg("--version").output().is_err() {
        panic!("gcc es obligatorio para verificar LP64");
    }

    struct CasoStruct<'a> {
        nombre: &'a str,
        campos: &'a [&'a str],
        packed: bool,
    }

    let casos = [
        CasoStruct {
            nombre: "relleno_interno_y_final",
            campos: &["char a", "int b", "char c"],
            packed: false,
        },
        CasoStruct {
            nombre: "char_double_short",
            campos: &["char a", "double d", "short s"],
            packed: false,
        },
        CasoStruct {
            nombre: "arrays_y_puntero",
            campos: &["char n[10]", "int *p", "short m[3]"],
            packed: false,
        },
        CasoStruct {
            nombre: "short_char_long_char",
            campos: &["short a", "char b", "long c", "char d"],
            packed: false,
        },
        CasoStruct {
            nombre: "float_array_sizet",
            campos: &["float f", "char c[5]", "size_t t"],
            packed: false,
        },
        CasoStruct {
            nombre: "dos_chars",
            campos: &["char a", "char b"],
            packed: false,
        },
        CasoStruct {
            nombre: "orden_ahorra_relleno",
            campos: &["double d", "int i", "char a", "char b"],
            packed: false,
        },
        CasoStruct {
            nombre: "orden_caro",
            campos: &["char a", "double d", "char b", "int i"],
            packed: false,
        },
        CasoStruct {
            nombre: "modificadores_y_2d",
            campos: &[
                "char nombre[10]",
                "int *p",
                "const unsigned long x",
                "short m[2][3]",
            ],
            packed: false,
        },
        CasoStruct {
            nombre: "punteros_array_y_void",
            campos: &["char flag", "int *ptrs[3]", "void *ctx", "uint8_t tag"],
            packed: false,
        },
        CasoStruct {
            nombre: "packed_sin_relleno",
            campos: &["char a", "int b", "char c"],
            packed: true,
        },
        CasoStruct {
            nombre: "packed_con_double_y_puntero",
            campos: &["char a", "double d", "int *p", "short s"],
            packed: true,
        },
    ];

    let tmp = TempDir::new("gcc_lp64");
    let mut fuente = String::from(
        "#include <stdio.h>\n#include <stddef.h>\n#include <stdint.h>\n#include <stdbool.h>\n\n",
    );
    let mut main_body = String::new();
    let mut esperados_rust: Vec<Vec<usize>> = Vec::with_capacity(casos.len());

    let reg = registro_cs();

    for (i, caso) in casos.iter().enumerate() {
        let mut campos = campos_struct_desde_strs(caso.campos)
            .unwrap_or_else(|e| panic!("caso {} falló al parsear campos: {e}", caso.nombre));
        let disp = disposicion_struct(&mut campos, caso.packed);

        // Validar también el SVG renderizado para este struct
        let bloque_val = serde_json::json!({
            "type": "struct",
            "name": format!("struct s{i}"),
            "fields": caso.campos,
            "packed": caso.packed,
        });
        let bloque: Bloque = bloque_val.as_object().unwrap().clone();
        let svg = reg
            .renderizar(&bloque, true)
            .unwrap_or_else(|e| panic!("SVG struct falló en {}: {e}", caso.nombre));
        assert!(
            svg.contains(&format!("sizeof(struct s{i}) = {} bytes", disp.sizeof)),
            "SVG de {} no contiene sizeof={} bytes",
            caso.nombre,
            disp.sizeof
        );
        for c in &campos {
            assert!(
                svg.contains(&format!(">+{}<", c.offset)),
                "SVG de {} no contiene offset +{}",
                caso.nombre,
                c.offset
            );
        }

        let attr = if caso.packed {
            " __attribute__((packed))"
        } else {
            ""
        };
        let decls: String = caso
            .campos
            .iter()
            .map(|c| format!("{c};"))
            .collect::<Vec<_>>()
            .join(" ");
        fuente.push_str(&format!("struct{attr} s{i} {{ {decls} }};\n"));

        main_body.push_str(&format!(
            "    printf(\"%zu %zu\", sizeof(struct s{i}), _Alignof(struct s{i}));\n"
        ));
        for c in &campos {
            main_body.push_str(&format!(
                "    printf(\" %zu\", offsetof(struct s{i}, {}));\n",
                c.nombre
            ));
        }
        main_body.push_str("    puts(\"\");\n");

        let mut fila = vec![disp.sizeof, disp.alineacion];
        fila.extend(campos.iter().map(|c| c.offset));
        esperados_rust.push(fila);
    }

    fuente.push_str(&format!("\nint main(void) {{\n{main_body}    return 0;\n}}\n"));
    let src_path = tmp.path().join("lp64.c");
    let bin_path = tmp.path().join("lp64");
    fs::write(&src_path, fuente).expect("no se pudo escribir lp64.c");

    let comp = Command::new("gcc")
        .args(["-std=c11", "-Wall", "-Wextra"])
        .arg(&src_path)
        .arg("-o")
        .arg(&bin_path)
        .output()
        .expect("falló al invocar gcc");
    assert!(
        comp.status.success(),
        "gcc falló:\n{}",
        String::from_utf8_lossy(&comp.stderr)
    );

    let run = Command::new(&bin_path)
        .output()
        .expect("falló al ejecutar binario lp64");
    assert!(
        run.status.success(),
        "lp64 falló:\n{}",
        String::from_utf8_lossy(&run.stderr)
    );
    let stdout = String::from_utf8(run.stdout).unwrap();
    let lineas: Vec<&str> = stdout.lines().collect();
    assert_eq!(lineas.len(), casos.len());

    for (i, linea) in lineas.iter().enumerate() {
        let gcc_vals: Vec<usize> = linea
            .split_whitespace()
            .map(|s| s.parse::<usize>().unwrap())
            .collect();
        assert_eq!(
            gcc_vals, esperados_rust[i],
            "discrepancia LP64 vs gcc en caso '{}' (gcc={gcc_vals:?}, rust={:?})",
            casos[i].nombre, esperados_rust[i]
        );
    }
}

fn resolver_o_compilar_dibu(tmp_build: &Path) -> PathBuf {
    if let Some(bin) = option_env!("CARGO_BIN_EXE_dibu") {
        let p = PathBuf::from(bin);
        if p.is_file() {
            return p;
        }
    }
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .and_then(|p| p.parent())
        .expect("raíz del workspace");
    for perfil in ["debug", "release"] {
        let cand = workspace_root.join("target").join(perfil).join("dibu");
        if cand.is_file() {
            return cand;
        }
    }
    // Si se corrió `cargo test -p dibu-cs` en un target limpio, compilamos `dibu-cli`
    // en un `--target-dir` dentro de `tmp_build` para no bloquear el `.cargo-lock` padre.
    let target_dir = tmp_build.join("cargo_target");
    let out = Command::new("cargo")
        .current_dir(workspace_root)
        .args([
            "build",
            "--offline",
            "-p",
            "dibu-cli",
            "--target-dir",
            target_dir.to_str().unwrap(),
        ])
        .output()
        .expect("falló al invocar cargo build -p dibu-cli");
    assert!(
        out.status.success(),
        "no se pudo compilar dibu-cli:\n{}",
        String::from_utf8_lossy(&out.stderr)
    );
    target_dir.join("debug").join("dibu")
}

#[test]
fn dibu_sync_md_sobre_los_14_tipos_cs_y_paridad_nombre_svg() {
    let tmp = TempDir::new("sync_md_cs");
    let dibu_bin = resolver_o_compilar_dibu(tmp.path());

    let bloques_md: [(&str, &str); 15] = [
        (
            "```estructura",
            "type: array\nvalues: [3, 1, 4]\npointers: \"p:0, q:2\"\n",
        ),
        (
            "~~~estructura",
            "type: matrix\nvalues: [[1, 2], [3, 4]]\n",
        ),
        (
            "```statics",
            "type: linked_list\nvalues: [10, 20, 30]\n",
        ),
        (
            "~~~statics",
            "type: stack\nvalues: [5, 9]\ncapacity: 4\n",
        ),
        (
            "```estructura",
            "type: call_stack\nframes:\n  - function: main\n    vars: {a: 3}\n",
        ),
        (
            "~~~estructura",
            "type: call_tree\nroot:\n  call: f(2)\n  returns: 2\n",
        ),
        (
            "```statics",
            "type: queue\nvariant: circular\ncapacity: 6\nfrontIndex: 1\nrearIndex: 3\n",
        ),
        (
            "```estructura",
            "type: tree\nvariant: bst\ninsert: [8, 3, 10]\n",
        ),
        (
            "~~~estructura",
            "type: struct\nname: struct s\nfields: [\"char a\", \"int b\"]\n",
        ),
        (
            "```statics",
            "type: adt\nname: Pila\noperations: [\"pila_t crear(void)\"]\n",
        ),
        (
            "```estructura",
            "type: complexity\ncurves: [\"n\", \"n^2\"]\n",
        ),
        (
            "~~~estructura",
            "type: flowchart\nsteps:\n  - \"x = 1\"\n",
        ),
        (
            "```statics",
            "type: cost_table\nlines:\n  - [\"int s = 0;\", \"c1\", 1]\ntotal: c1\n",
        ),
        (
            "```estructura",
            "type: trace_table\ncolumns: [\"i\", \"s\"]\nrows: [[0, 1]]\n",
        ),
        (
            "~~~estructura",
            "- type: array\n  values: [1, 2]\n- type: stack\n  values: [7]\n",
        ),
    ];

    let mut md_texto = String::from("# Guía CS\n\n");
    let mut cuerpos = Vec::with_capacity(bloques_md.len());
    for (fence, cuerpo) in &bloques_md {
        let cierre = if fence.starts_with("~~~") {
            "~~~"
        } else {
            "```"
        };
        md_texto.push_str(fence);
        md_texto.push('\n');
        md_texto.push_str(cuerpo);
        md_texto.push_str(cierre);
        md_texto.push_str("\n\n");
        cuerpos.push(*cuerpo);
    }

    let md_path = tmp.path().join("cs_guia.md");
    fs::write(&md_path, &md_texto).expect("no se pudo escribir cs_guia.md");

    // Verificar nombres `estructura-<sha1[:12]>.svg` contra Python `nombre_svg` y `dibu_core::nombre_svg`
    let py_script = r#"
import json, sys
from generador_examenes.diagramas import nombre_svg
cuerpos = json.load(sys.stdin)
print(json.dumps([nombre_svg(c) for c in cuerpos]))
"#;
    let py_nombres_val =
        ejecutar_python_json(py_script, &serde_json::to_string(&cuerpos).unwrap());
    let mut nombres_esperados: Vec<String> = py_nombres_val
        .as_array()
        .unwrap()
        .iter()
        .map(|v| v.as_str().unwrap().to_string())
        .collect();
    for (cuerpo, py_nombre) in cuerpos.iter().zip(nombres_esperados.iter()) {
        assert_eq!(nombre_svg(cuerpo), *py_nombre);
    }
    nombres_esperados.sort();

    // Primera corrida: genera todos los SVG y sale con exit 1
    let out1 = Command::new(&dibu_bin)
        .current_dir(tmp.path())
        .args(["sync-md", "cs_guia.md"])
        .output()
        .expect("falló al ejecutar dibu sync-md");
    assert_eq!(
        out1.status.code(),
        Some(1),
        "primera corrida debía salir con 1.\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&out1.stdout),
        String::from_utf8_lossy(&out1.stderr)
    );

    let diagramas_dir = tmp.path().join("diagramas");
    let mut archivos: Vec<String> = fs::read_dir(&diagramas_dir)
        .expect("diagramas/ debe existir")
        .map(|e| e.unwrap().file_name().to_string_lossy().into_owned())
        .collect();
    archivos.sort();
    assert_eq!(archivos, nombres_esperados);

    // Validar que todos los SVGs generados son XML válido (y el apilado tiene 1 + 2 = 3 `<svg`)
    let mut svgs_leidos: BTreeMap<String, String> = BTreeMap::new();
    for nombre in &archivos {
        let svg = fs::read_to_string(diagramas_dir.join(nombre)).unwrap();
        assert!(
            svg.starts_with("<svg ") && svg.ends_with("</svg>"),
            "SVG inválido en {nombre}"
        );
        svgs_leidos.insert(nombre.clone(), svg);
    }
    let nombre_apilado = nombre_svg(cuerpos.last().unwrap());
    assert_eq!(svgs_leidos[&nombre_apilado].matches("<svg").count(), 3);

    // Validar XML estricto en Python sobre todos los SVGs producidos por `dibu sync-md`
    let py_xml_check = r#"
import json, sys, xml.etree.ElementTree as ET
svgs = json.load(sys.stdin)
for name, content in svgs.items():
    root = ET.fromstring(content)
    assert root.tag == "{http://www.w3.org/2000/svg}svg", name
print("true")
"#;
    let xml_ok = ejecutar_python_json(py_xml_check, &serde_json::to_string(&svgs_leidos).unwrap());
    assert_eq!(xml_ok, Value::Bool(true));

    // Segunda corrida con --verificar: exit 0
    let out_ver = Command::new(&dibu_bin)
        .current_dir(tmp.path())
        .args(["sync-md", "--verificar", "cs_guia.md"])
        .output()
        .expect("falló al ejecutar dibu sync-md --verificar");
    assert_eq!(
        out_ver.status.code(),
        Some(0),
        "--verificar debía salir con 0.\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&out_ver.stdout),
        String::from_utf8_lossy(&out_ver.stderr)
    );
}
