use dibu_core::{leer_bloques, Bloque, Registro};
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};
use std::io::Write;
use std::process::{Command, Stdio};

const ALUCARD_ROOT: &str = "/home/mrtin/dev/tools/docente/contenidos/alucarD";
const ALUCARD_PYTHON: &str = "/home/mrtin/dev/tools/docente/contenidos/alucarD/.venv/bin/python";
const FIXTURES_CHEM_YAML: &str = include_str!("../../../fixtures/diagramas_quimica.yaml");

const TIPOS_CHEM: [&str; 21] = [
    "molecule",
    "reaction",
    "vsepr",
    "orbitals",
    "energy_profile",
    "titration",
    "speciation",
    "kinetics",
    "phase_diagram",
    "heating_curve",
    "spectrum",
    "galvanic_cell",
    "unit_cell",
    "newman",
    "chair",
    "fischer",
    "crystal_field",
    "complex",
    "latimer",
    "frost",
    "born_haber",
];

fn registro_chem() -> Registro {
    let mut reg = Registro::new();
    dibu_chem::registrar(&mut reg);
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

fn contiene_nan_o_inf_aislado(svg: &str) -> bool {
    let lower = svg.to_ascii_lowercase();
    let bytes = lower.as_bytes();
    for pat in ["nan", "inf"] {
        let pb = pat.as_bytes();
        let mut i = 0;
        while let Some(rel) = lower[i..].find(pat) {
            let pos = i + rel;
            let prev_es_letra = pos > 0 && bytes[pos - 1].is_ascii_lowercase();
            let next_pos = pos + pb.len();
            let next_es_letra = next_pos < bytes.len() && bytes[next_pos].is_ascii_lowercase();
            if !prev_es_letra && !next_es_letra {
                return true;
            }
            i = pos + 1;
        }
    }
    false
}

fn extraer_texto_svg(svg: &str) -> String {
    let re_text = regex::Regex::new(r"(?s)<text\b[^>]*>(.*?)</text>").unwrap();
    let re_tags = regex::Regex::new(r"<[^>]+>").unwrap();
    let mut partes = Vec::new();
    for cap in re_text.captures_iter(svg) {
        let sin_tags = re_tags.replace_all(&cap[1], "");
        let unescaped = sin_tags
            .replace("&amp;", "&")
            .replace("&lt;", "<")
            .replace("&gt;", ">")
            .replace("&quot;", "\"")
            .replace("&#39;", "'")
            .replace("&#x27;", "'");
        partes.push(unescaped);
    }
    partes.join(" ")
}

fn texto_plano_svg(svg: &str) -> String {
    extraer_texto_svg(svg)
        .split_whitespace()
        .collect::<Vec<_>>()
        .join("")
        .replace('\u{a0}', "")
}

#[test]
fn registro_21_tipos_quimica() {
    let reg = registro_chem();
    let registrados: BTreeSet<&str> = reg.tipos().into_iter().collect();
    let esperados: BTreeSet<&str> = TIPOS_CHEM.iter().copied().collect();
    assert_eq!(registrados.len(), 21);
    assert_eq!(registrados, esperados);
}

#[test]
fn paridad_74_fixtures_chem_contra_alucard_en_ambas_paletas_y_determinismo() {
    let reg = registro_chem();
    let bloques =
        leer_bloques(FIXTURES_CHEM_YAML).expect("fixtures/diagramas_quimica.yaml debe parsear");
    assert_eq!(
        bloques.len(),
        74,
        "se esperaban exactamente 74 bloques en fixtures/diagramas_quimica.yaml, hay {}",
        bloques.len()
    );

    let mut ids_vistos: BTreeSet<String> = BTreeSet::new();
    let tipos_presentes: BTreeSet<String> = bloques
        .iter()
        .filter_map(|b| b.get("type").and_then(Value::as_str).map(str::to_string))
        .collect();
    for tipo in TIPOS_CHEM {
        assert!(
            tipos_presentes.contains(tipo),
            "falta el tipo {tipo} en fixtures/diagramas_quimica.yaml"
        );
    }

    let mut casos_rust = Vec::with_capacity(bloques.len() * 2);
    for (idx, bloque) in bloques.iter().enumerate() {
        let id = bloque
            .get("id")
            .and_then(Value::as_str)
            .filter(|s| !s.is_empty())
            .map(str::to_string)
            .unwrap_or_else(|| {
                panic!("bloque {idx} sin id válido en fixtures/diagramas_quimica.yaml")
            });
        assert!(
            ids_vistos.insert(id.clone()),
            "id duplicado en fixtures/diagramas_quimica.yaml: {id}"
        );

        let mut svg_por_paleta: [Option<String>; 2] = [None, None];
        for (p_idx, tinta) in [true, false].into_iter().enumerate() {
            let svg1 = reg
                .renderizar(bloque, tinta)
                .unwrap_or_else(|e| panic!("dibu-chem falló en {id} (tinta={tinta}): {e}"));
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
                !svg1.contains(">NaN<")
                    && !svg1.contains("=\"NaN\"")
                    && !svg1.contains("Infinity")
                    && !contiene_nan_o_inf_aislado(&svg1),
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
    w = float(raiz.get("width"))
    h = float(raiz.get("height"))
    assert w > 80.0 and h > 40.0, f"dimensiones mínimas incumplidas: {w}x{h}"
    assert not re.search(r"(?<![a-z])(nan|inf)(?![a-z])", svg_txt.lower())
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
fn aserciones_semanticas_fixtures_test_diagramas_quimica() {
    let reg = registro_chem();
    let bloques =
        leer_bloques(FIXTURES_CHEM_YAML).expect("fixtures/diagramas_quimica.yaml debe parsear");
    let por_id: BTreeMap<String, &Bloque> = bloques
        .iter()
        .filter_map(|b| {
            b.get("id")
                .and_then(Value::as_str)
                .map(|id| (id.to_string(), b))
        })
        .collect();

    let render_id = |id: &str| -> String {
        let b = por_id
            .get(id)
            .unwrap_or_else(|| panic!("falta el fixture {id:?} en fixtures/diagramas_quimica.yaml"));
        reg.renderizar(b, true)
            .unwrap_or_else(|e| panic!("falló renderizar {id}: {e}"))
    };

    // 1 y 2: Titulaciones
    let tit_ac = texto_plano_svg(&render_id("titulacion-acetico"));
    assert!(tit_ac.contains("pH=8,73"), "{tit_ac}");
    assert!(tit_ac.contains("pH=pKa=4,76"), "{tit_ac}");
    assert!(tit_ac.contains("pHinicial=2,88"), "{tit_ac}");

    let tit_nh3 = extraer_texto_svg(&render_id("titulacion-amoniaco"));
    assert!(!tit_nh3.contains("5,28"), "{tit_nh3}");
    assert!(!tit_nh3.contains("9,25"), "{tit_nh3}");

    // 3 y 4: Orbitales moleculares
    let om_o2 = texto_plano_svg(&render_id("om-oxigeno"));
    assert!(om_o2.contains("Ordendeenlace=2"), "{om_o2}");
    assert!(om_o2.contains("paramagnética"), "{om_o2}");

    let om_n2 = texto_plano_svg(&render_id("om-nitrogeno"));
    assert!(!om_n2.contains("Ordendeenlace=3"), "{om_n2}");

    // 5 y 6: TRePEV
    let trepev_nh3 = extraer_texto_svg(&render_id("trepev-amoniaco"));
    assert!(trepev_nh3.contains("piramidal trigonal"), "{trepev_nh3}");
    assert!(trepev_nh3.contains("107"), "{trepev_nh3}");

    let trepev_xef4 = extraer_texto_svg(&render_id("trepev-xef4-blanco"));
    assert!(!trepev_xef4.contains("cuadrada plana"), "{trepev_xef4}");
    assert!(!trepev_xef4.contains("sp³d²"), "{trepev_xef4}");

    // 7 y 8: Pila galvánica
    let pila_blanco = extraer_texto_svg(&render_id("pila-blanco"));
    assert!(!pila_blanco.contains("Ánodo"), "{pila_blanco}");
    assert!(!pila_blanco.contains("0,46"), "{pila_blanco}");
    assert!(!pila_blanco.contains('→'), "{pila_blanco}");

    let pila_daniell = texto_plano_svg(&render_id("pila-daniell"));
    assert!(pila_daniell.contains("Ánodo(−)"), "{pila_daniell}");
    assert!(pila_daniell.contains("E°=1,10V"), "{pila_daniell}");
    assert!(pila_daniell.contains("1,159V"), "{pila_daniell}");

    // 9 y 10: Celda unidad
    let celda_fcc = texto_plano_svg(&render_id("celda-fcc"));
    assert!(celda_fcc.contains("8×1/8+6×1/2=4"), "{celda_fcc}");
    assert!(celda_fcc.contains("74%"), "{celda_fcc}");
    assert!(celda_fcc.contains("8,94"), "{celda_fcc}");

    let celda_bcc_blanco = extraer_texto_svg(&render_id("celda-bcc-blanco"));
    assert!(!celda_bcc_blanco.contains("68"), "{celda_bcc_blanco}");

    // 11, 12 y 13: Diagramas de fases
    let fases_agua = extraer_texto_svg(&render_id("fases-agua"));
    assert!(fases_agua.contains("Teb"), "{fases_agua}");
    assert!(fases_agua.contains("373,1"), "{fases_agua}");
    assert!(fases_agua.contains("Punto triple"), "{fases_agua}");

    let fases_co2 = extraer_texto_svg(&render_id("fases-co2-log"));
    assert!(fases_co2.contains("Tsub"), "{fases_co2}");

    let fases_blanco = extraer_texto_svg(&render_id("fases-blanco"));
    assert!(!fases_blanco.contains("Sólido"), "{fases_blanco}");

    // 14: Naftaleno Kekulé (anillos fusionados esqueletales: 11 enlaces = 6 simples + 5 dobles -> 16 <line>)
    let naf_svg = render_id("naftaleno-kekule");
    assert_eq!(
        naf_svg.matches("<line ").count(),
        16,
        "naftaleno-kekule debe renderizar 6 enlaces simples (6 líneas) + 5 enlaces dobles (10 líneas) = 16 <line>: {naf_svg}"
    );
    assert!(!extraer_texto_svg(&naf_svg).contains('C'), "{naf_svg}");

    // 15: Fischer glucosa
    let fischer_glu = texto_plano_svg(&render_id("fischer-glucosa"));
    assert_eq!(fischer_glu.matches("(R)").count(), 3, "{fischer_glu}");
    assert_eq!(fischer_glu.matches("(S)").count(), 1, "{fischer_glu}");
    assert!(fischer_glu.contains("Serie=D"), "{fischer_glu}");

    // 16: Silla metilo
    let silla_metil = texto_plano_svg(&render_id("silla-metil"));
    assert!(
        silla_metil.contains("laderecha,por7,3kJ/mol"),
        "{silla_metil}"
    );

    // 17: Campo cristalino Co-NH3
    let campo_co = texto_plano_svg(&render_id("campo-co-nh3"));
    assert!(campo_co.contains("bajo"), "{campo_co}");
    assert!(campo_co.contains("Desapareados:0"), "{campo_co}");
    assert!(campo_co.contains("−2,4"), "{campo_co}");

    // 18 y 19: Born-Haber
    let bh_nacl = extraer_texto_svg(&render_id("born-haber-nacl")).replace('-', "−");
    assert!(bh_nacl.contains("−787"), "{bh_nacl}");

    let bh_blanco = extraer_texto_svg(&render_id("born-haber-blanco"));
    assert!(!bh_blanco.contains("689"), "{bh_blanco}");

    // 20 y 21: Complejos
    let comp_trans = texto_plano_svg(&render_id("complejo-trans"));
    assert!(comp_trans.contains("Co(III)"), "{comp_trans}");
    assert!(comp_trans.contains("d6"), "{comp_trans}");
    assert!(comp_trans.contains("trans"), "{comp_trans}");

    let comp_fac_blanco = extraer_texto_svg(&render_id("complejo-fac-blanco"));
    assert!(!comp_fac_blanco.contains("fac"), "{comp_fac_blanco}");
}

#[test]
fn errores_controlados_test_diagramas_quimica_y_21_tipos() {
    let reg = registro_chem();

    // Los 30 casos de test_diagramas_quimica.py + casos de energy_profile y frost (cubriendo los 21 tipos)
    let casos_error: Vec<(&str, &str, &str)> = vec![
        // test_errores_claros (20)
        ("molecule", r#"{"type":"molecule"}"#, "faltan atoms"),
        (
            "molecule",
            r#"{"type":"molecule","atoms":[{"id":"a","el":"C"},{"id":"b","from":"z"}]}"#,
            "no hay un átomo",
        ),
        (
            "molecule",
            r#"{"type":"molecule","atoms":[{"el":"C"}],"bonds":[["a0","a0",4]]}"#,
            "order",
        ),
        (
            "reaction",
            r#"{"type":"reaction","steps":[{"arrow":"~>"}]}"#,
            "arrow",
        ),
        (
            "vsepr",
            r#"{"type":"vsepr","ligands":["F","F","F","F","F","F","F"]}"#,
            "geometría conocida",
        ),
        (
            "vsepr",
            r#"{"type":"vsepr","geometry":"tetrahedral","ligands":["H","H","H"]}"#,
            "lleva 4",
        ),
        (
            "orbitals",
            r#"{"type":"orbitals","element":"Xx"}"#,
            "desconocido",
        ),
        (
            "orbitals",
            r#"{"type":"orbitals","kind":"molecular","atoms":["Na","Na"]}"#,
            "dos primeros períodos",
        ),
        (
            "titration",
            r#"{"type":"titration","analyte":{"kind":"weak_acid","conc":0.1}}"#,
            "pKa",
        ),
        (
            "titration",
            r#"{"type":"titration","analyte":{"kind":"sal"}}"#,
            "analyte.kind",
        ),
        (
            "speciation",
            r#"{"type":"speciation","pKa":[2,7],"species":["A","B"]}"#,
            "3 especies",
        ),
        ("kinetics", r#"{"type":"kinetics","order":3}"#, "orden 3"),
        (
            "phase_diagram",
            r#"{"type":"phase_diagram","substance":"mercurio"}"#,
            "sustancia",
        ),
        (
            "phase_diagram",
            r#"{"type":"phase_diagram","triple":[300,1],"critical":[200,50]}"#,
            "crítico",
        ),
        (
            "phase_diagram",
            r#"{"type":"phase_diagram","kind":"eutectic","meltingA":300}"#,
            "eutéctico",
        ),
        (
            "heating_curve",
            r#"{"type":"heating_curve","from":50,"to":10}"#,
            "from < to",
        ),
        (
            "spectrum",
            r#"{"type":"spectrum","kind":"raman","peaks":[{"at":1}]}"#,
            "kind",
        ),
        (
            "spectrum",
            r#"{"type":"spectrum","kind":"nmr","signals":[{"shift":1,"multiplicity":"x"}]}"#,
            "multiplicidad",
        ),
        (
            "galvanic_cell",
            r#"{"type":"galvanic_cell","halfCells":[{"couple":"Zn","E0":1},{"couple":"Cu^2+/Cu","E0":0.34}]}"#,
            "oxidada/reducida",
        ),
        (
            "unit_cell",
            r#"{"type":"unit_cell","lattice":"hcp"}"#,
            "red 'hcp'",
        ),
        // test_errores_de_organica_e_inorganica (10)
        (
            "newman",
            r#"{"type":"newman","front":["H"],"back":["H","H","H"]}"#,
            "tres sustituyentes",
        ),
        (
            "chair",
            r#"{"type":"chair","substituents":[{"position":7,"group":"CH3"}]}"#,
            "de 1 a 6",
        ),
        (
            "fischer",
            r#"{"type":"fischer","centers":[]}"#,
            "centers",
        ),
        (
            "fischer",
            r#"{"type":"fischer","top":"tBu","bottom":"Ph","centers":[{"left":"H","right":"OH"}]}"#,
            "empatan",
        ),
        (
            "crystal_field",
            r#"{"type":"crystal_field","geometry":"octahedral"}"#,
            "electrons",
        ),
        (
            "crystal_field",
            r#"{"type":"crystal_field","metal":"Na","oxidation":1}"#,
            "transición",
        ),
        (
            "crystal_field",
            r#"{"type":"crystal_field","electrons":5,"ligand":"agua"}"#,
            "serie espectroquímica",
        ),
        (
            "complex",
            r#"{"type":"complex","ligands":["Cl","Cl","Cl","Cl","Cl"]}"#,
            "6 sitios",
        ),
        (
            "latimer",
            r#"{"type":"latimer","species":["A","B"],"oxidation":[0,2],"potentials":[1]}"#,
            "bajar",
        ),
        (
            "born_haber",
            r#"{"type":"born_haber","steps":[]}"#,
            "faltan steps",
        ),
        // Cobertura de energy_profile y frost (completando los 21 tipos)
        (
            "energy_profile",
            r#"{"type":"energy_profile","states":[{"energy":0}]}"#,
            "al menos dos estados",
        ),
        (
            "frost",
            r#"{"type":"frost","species":["A","B"],"oxidation":[1,2],"potentials":[0.5]}"#,
            "tienen que bajar de izquierda a derecha",
        ),
    ];

    assert_eq!(casos_error.len(), 32);

    let mut tipos_cubiertos: BTreeSet<&str> = BTreeSet::new();
    for (idx, (tipo, json_str, subcadena)) in casos_error.iter().enumerate() {
        tipos_cubiertos.insert(*tipo);
        let bloque: Bloque = serde_json::from_str(json_str)
            .unwrap_or_else(|e| panic!("JSON inválido en caso {idx}: {e}"));
        let err = reg
            .renderizar(&bloque, true)
            .expect_err(&format!("caso {idx} ({tipo}: {json_str}) debía dar error"));

        let prefijo = format!("{tipo}: ");
        let prefijo_punto = format!("{tipo}.");
        let doble_prefijo = format!("{tipo}: {tipo}: ");
        assert!(
            err.0.starts_with(&prefijo) || err.0.starts_with(&prefijo_punto),
            "caso {idx} ({tipo}): el error {:?} no empieza con {:?} ni {:?}",
            err.0,
            prefijo,
            prefijo_punto
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

    for tipo in TIPOS_CHEM {
        assert!(
            tipos_cubiertos.contains(tipo),
            "falta cubrir casos de error para el tipo {tipo}"
        );
    }
}
