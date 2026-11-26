//! Diagramas de control de procesos (9 tipos).

#![warn(clippy::missing_safety_doc)]

pub mod blocks;
pub mod industrial;
pub mod math;
pub mod plots;

use dibu_core::Registro;

pub fn registrar(reg: &mut Registro) {
    reg.registrar("block_diagram", blocks::render_block_diagram);
    reg.registrar("time_response", plots::render_time_response);
    reg.registrar("bode", plots::render_bode);
    reg.registrar("nyquist", plots::render_nyquist);
    reg.registrar("pole_zero", plots::render_pole_zero);
    reg.registrar("root_locus", plots::render_root_locus);
    reg.registrar("instrumentation", industrial::render_instrumentation);
    reg.registrar("ladder", industrial::render_ladder);
    reg.registrar("grafcet", industrial::render_grafcet);
}

#[cfg(test)]
mod tests {
    use super::*;
    use dibu_core::leer_bloques;

    #[test]
    fn todos_los_fixtures_de_control_generan_svg_valido() {
        let mut reg = Registro::new();
        registrar(&mut reg);
        let yaml = include_str!("../../../fixtures/diagramas_control.yaml");
        let bloques = leer_bloques(yaml).expect("fixture YAML válido");
        assert_eq!(bloques.len(), 13);
        for (i, b) in bloques.iter().enumerate() {
            for tinta in [true, false] {
                let svg = reg
                    .renderizar(b, tinta)
                    .unwrap_or_else(|e| panic!("bloque {i} ({:?}) falló: {e}", b.get("id")));
                assert!(svg.starts_with("<svg "));
                assert!(svg.ends_with("</svg>"));
                assert!(svg.contains(r#"class="estructura""#));
            }
        }
    }

    #[test]
    fn regresion_adversarial_control_sin_nan_inf_ni_negativos() {
        let mut reg = Registro::new();
        registrar(&mut reg);

        let deben_fallar = [
            serde_json::json!({"type": "pole_zero", "zeros": [-1e308, 1e308]}),
            serde_json::json!({"type": "pole_zero", "poles": [-1], "xRange": [5, 5]}),
            serde_json::json!({"type": "pole_zero", "poles": [-1], "yRange": [0, 0]}),
            serde_json::json!({"type": "root_locus", "system": {"num": [1], "den": [1, 1]}, "xRange": [2, -2]}),
            serde_json::json!({"type": "bode", "system": {"num": [1e308], "den": [1e-10]}}),
            serde_json::json!({"type": "nyquist", "system": {"num": [1e308], "den": [1]}, "limit": 1e308}),
            serde_json::json!({"type": "time_response", "system": {"num": [1], "den": [1, -10]}, "tEnd": 100, "annotations": "all"}),
            serde_json::json!({"type": "time_response", "system": {"tau": 1}, "tEnd": -5}),
            serde_json::json!({"type": "time_response", "system": {"tau": 1}, "annotations": [123]}),
            serde_json::json!({"type": "time_response", "system": {"tau": 1}, "annotations": ["settling"], "settlingBand": 0}),
            serde_json::json!({"type": "instrumentation", "grid": -50, "equipment": [{"id": "V1", "kind": "vessel", "at": [1, 1]}]}),
            serde_json::json!({"type": "instrumentation", "equipment": [{"id": "T1", "kind": "tank", "at": [0, 1e30]}]}),
        ];
        for v in deben_fallar {
            let b = v.as_object().unwrap();
            assert!(reg.renderizar(b, true).is_err(), "debía fallar: {v}");
        }

        let deben_pasar = [
            serde_json::json!({"type": "pole_zero", "zeros": [-1e150, 1e150]}),
            serde_json::json!({"type": "time_response", "system": {"tau": 1}, "yMin": 10, "yMax": 0}),
            serde_json::json!({"type": "nyquist", "system": {"tau": 1}, "wMin": 10, "wMax": 1}),
            serde_json::json!({"type": "instrumentation", "grid": 0, "instruments": [{"id": "a", "tag": "T1", "at": [1, 1]}]}),
        ];
        for v in deben_pasar {
            let b = v.as_object().unwrap();
            let svg = reg.renderizar(b, true).unwrap_or_else(|e| panic!("falló {v}: {e}"));
            assert!(!svg.contains("NaN"), "NaN en {v}: {svg}");
            assert!(!svg.contains("inf"), "inf en {v}: {svg}");
        }
    }

    #[test]
    fn regresion_tier5_f1_f2_visibilidad_y_cota_grado() {
        let mut reg = Registro::new();
        registrar(&mut reg);

        // F1: wnCircles extremos y polos/ceros fuera de xRange/yRange fijos no emiten inf/-inf ni coords gigantes
        for v in [
            serde_json::json!({"type": "pole_zero", "poles": [-1], "wnCircles": [1e308]}),
            serde_json::json!({"type": "pole_zero", "poles": [-1], "wnCircles": [1e305]}),
            serde_json::json!({"type": "root_locus", "system": {"k": 1, "poles": [-1, -2]}, "wnCircles": [1e308]}),
            serde_json::json!({"type": "pole_zero", "poles": [-1e307], "zeros": [1e307], "xRange": [-1, 1], "yRange": [-1, 1], "showValues": true}),
            serde_json::json!({"type": "pole_zero", "poles": [-1e305], "xRange": [-1, 1], "yRange": [-1, 1], "showValues": true}),
            serde_json::json!({"type": "root_locus", "system": {"k": 1, "poles": [-1e307]}, "xRange": [-1, 1], "yRange": [-1, 1]}),
            serde_json::json!({"type": "root_locus", "system": {"k": 1, "poles": [-1e150, -2e150]}, "xRange": [-1, 1], "yRange": [-1, 1]}),
        ] {
            let svg = reg
                .renderizar(v.as_object().unwrap(), true)
                .unwrap_or_else(|e| panic!("falló {v}: {e}"));
            assert!(!svg.contains("NaN") && !svg.contains("inf"), "inf/NaN en {v}: {svg}");
            assert!(svg.len() < 20_000, "SVG desproporcionado para {v}: {} bytes", svg.len());
        }

        // F2: grados polinomiales > MAX_GRADO_SISTEMA (ej. 40, 60) se rechazan al instante (< 50 ms)
        for grado in [25usize, 40, 60] {
            let den = vec![1.0; grado + 1];
            let v = serde_json::json!({"type": "root_locus", "system": {"num": [1], "den": den}});
            let t0 = std::time::Instant::now();
            let res = reg.renderizar(v.as_object().unwrap(), true);
            assert!(res.is_err(), "grado {grado} debía rechazarse");
            assert!(
                t0.elapsed().as_millis() < 50,
                "rechazo de grado {grado} tardó demasiado: {:?}",
                t0.elapsed()
            );
        }

        // F2: closedLoop combinado que excede MAX_GRADO_SISTEMA también se rechaza rápido
        let p15 = vec![1.0; 16];
        let cl = serde_json::json!({
            "type": "root_locus",
            "closedLoop": {
                "plant": {"num": [1], "den": p15},
                "controller": {"num": [1], "den": p15}
            }
        });
        assert!(reg.renderizar(cl.as_object().unwrap(), true).is_err());

        // Frontera válida: grado 12 en root_locus sigue funcionando rápido y sin errores
        let den12 = vec![1.0; 13];
        let v12 = serde_json::json!({"type": "root_locus", "system": {"num": [1], "den": den12}});
        let svg12 = reg.renderizar(v12.as_object().unwrap(), true).unwrap();
        assert!(!svg12.contains("NaN") && !svg12.contains("inf"));
    }
}
