//! Diagramas de química general, orgánica e inorgánica (Fase 2).

pub mod chem;
pub mod common;
pub mod general;
pub mod inorganica;
pub mod organica;

pub fn registrar(reg: &mut dibu_core::Registro) {
    general::registrar(reg);
    organica::registrar(reg);
    inorganica::registrar(reg);
}
