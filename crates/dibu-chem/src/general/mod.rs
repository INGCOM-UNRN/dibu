//! Renderizadores de química general (estructurales + gráficos).

pub mod plots;
pub mod structural;

use dibu_core::Registro;

pub fn registrar(reg: &mut Registro) {
    structural::registrar(reg);
    plots::registrar(reg);
}
