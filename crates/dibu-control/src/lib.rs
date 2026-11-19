//! Diagramas de control de procesos (9 tipos).

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
}
