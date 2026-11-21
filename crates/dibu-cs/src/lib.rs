//! Diagramas de estructuras de datos y algoritmos (Fase 3).

pub mod algos;
pub mod common;
pub mod lineal;
pub mod recursion;

use dibu_core::Registro;

pub fn registrar(reg: &mut Registro) {
    lineal::registrar(reg);
    recursion::registrar(reg);
    algos::registrar(reg);
}
