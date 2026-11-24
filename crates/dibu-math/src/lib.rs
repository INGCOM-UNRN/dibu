//! Diagramas de matemática aplicada: señales y sistemas, probabilidad y circuitos eléctricos.

pub mod circuitos;
pub mod expr;
pub mod probabilidad;
pub mod senales;

use dibu_comp::comun::envolver;
use dibu_core::Registro;

macro_rules! registrar_tipos {
    ($reg:expr, $( $tipo:literal => $f:path ),* $(,)?) => {
        $( $reg.registrar($tipo, |c, b| envolver($tipo, $f(c, b))); )*
    };
}

pub fn registrar(reg: &mut Registro) {
    registrar_tipos!(reg,
        "signal" => senales::render_signal,
        "convolution" => senales::render_convolution,
        "fourier" => senales::render_fourier,
        "sampling" => senales::render_sampling,
        "distribution" => probabilidad::render_distribution,
        "venn" => probabilidad::render_venn,
        "circuit" => circuitos::render_circuit,
        "phasor" => circuitos::render_phasor,
    );
}
