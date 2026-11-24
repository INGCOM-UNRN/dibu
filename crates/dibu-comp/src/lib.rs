//! Diagramas de ingeniería en computación: sistemas digitales, arquitectura de computadoras,
//! sistemas operativos, redes y bases de datos.

pub mod algoritmos;
pub mod arquitectura;
pub mod bd;
pub mod comun;
pub mod digital;
pub mod estructuras;
pub mod logica;
pub mod redes;
pub mod so;

use comun::envolver;
use dibu_core::Registro;

macro_rules! registrar_tipos {
    ($reg:expr, $( $tipo:literal => $f:path ),* $(,)?) => {
        $( $reg.registrar($tipo, |c, b| envolver($tipo, $f(c, b))); )*
    };
}

pub fn registrar(reg: &mut Registro) {
    registrar_tipos!(reg,
        "truth_table" => digital::render_truth_table,
        "karnaugh" => digital::render_karnaugh,
        "logic_circuit" => digital::render_logic_circuit,
        "pipeline" => arquitectura::render_pipeline,
        "ieee754" => arquitectura::render_ieee754,
        "cache" => arquitectura::render_cache,
        "cpu_scheduling" => so::render_cpu_scheduling,
        "page_table" => so::render_page_table,
        "page_replacement" => so::render_page_replacement,
        "subnetting" => redes::render_subnetting,
        "transaction_schedule" => bd::render_transaction_schedule,
        "heap" => estructuras::render_heap,
        "hash_table" => estructuras::render_hash_table,
        "avl" => estructuras::render_avl,
        "bplus_tree" => estructuras::render_bplus_tree,
        "dp_table" => algoritmos::render_dp_table,
        "dijkstra" => algoritmos::render_dijkstra,
        "turing" => algoritmos::render_turing,
    );
}
