//! Diagramas de estructuras de datos y algoritmos (Fase 3).

#![warn(clippy::missing_safety_doc)]

pub mod algos;
pub mod common;
pub mod lineal;
pub mod objetos;
pub mod recursion;

use dibu_core::Registro;

pub fn registrar(reg: &mut Registro) {
    lineal::registrar(reg);
    recursion::registrar(reg);
    algos::registrar(reg);
    objetos::registrar(reg);
}
