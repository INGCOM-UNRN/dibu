"""Adaptador de solo lectura sobre el oráculo Python `alucarD`.

IMPORTANTE: Nunca debe escribir archivos ni bytecode (`__pycache__`) dentro de
`/home/mrtin/dev/tools/docente/contenidos/alucarD`.
"""

from __future__ import annotations

import sys

sys.dont_write_bytecode = True

from dataclasses import dataclass
from pathlib import Path
from typing import Any

ALUCARD_ROOT = Path("/home/mrtin/dev/tools/docente/contenidos/alucarD")

TIPOS_CONTROL: tuple[str, ...] = (
    "block_diagram",
    "time_response",
    "bode",
    "nyquist",
    "pole_zero",
    "root_locus",
    "instrumentation",
    "ladder",
    "grafcet",
)

TIPOS_CHEM: tuple[str, ...] = (
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
)

TIPOS_CS: tuple[str, ...] = (
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
)

TODOS_LOS_TIPOS: tuple[str, ...] = TIPOS_CS + TIPOS_CONTROL + TIPOS_CHEM


@dataclass(frozen=True)
class ResultadoOraculo:
    """Resultado normalizado de una invocación al oráculo `alucarD`."""

    ok: bool
    svg: str | None = None
    error: str | None = None


def asegurar_alucard_en_sys_path() -> None:
    """Agrega `ALUCARD_ROOT` a `sys.path` en modo solo lectura si no está presente."""
    sys.dont_write_bytecode = True
    root_str = str(ALUCARD_ROOT)
    if root_str not in sys.path:
        sys.path.append(root_str)


def _cargar_diagramas():
    asegurar_alucard_en_sys_path()
    from generador_examenes import diagramas

    return diagramas


def listar_tipos_oraculo() -> tuple[str, ...]:
    """Retorna la tupla de todos los tipos registrados en `alucarD`."""
    diagramas = _cargar_diagramas()
    return tuple(diagramas.TIPOS)


def paletas_oraculo() -> dict[bool, dict[str, str]]:
    """Retorna las paletas `True` (tinta) y `False` (color) definidas en `alucarD`."""
    diagramas = _cargar_diagramas()
    return {k: dict(v) for k, v in diagramas.PALETAS.items()}


def leer_bloques_oraculo(texto: str) -> list[dict[str, Any]]:
    """Parsea un bloque, lista o `{blocks: [...]}` usando `diagramas.leer_bloques`."""
    diagramas = _cargar_diagramas()
    return list(diagramas.leer_bloques(texto))


def renderizar_oraculo(bloque: dict[str, Any], tinta: bool = True) -> ResultadoOraculo:
    """Renderiza un bloque con `diagramas.renderizar`, capturando `ErrorDiagrama`."""
    diagramas = _cargar_diagramas()
    try:
        svg = diagramas.renderizar(bloque, tinta=tinta)
        return ResultadoOraculo(ok=True, svg=svg, error=None)
    except diagramas.ErrorDiagrama as exc:
        return ResultadoOraculo(ok=False, svg=None, error=str(exc))


def renderizar_bloque_md_oraculo(cuerpo: str, tinta: bool = True) -> ResultadoOraculo:
    """Renderiza el cuerpo de un fence Markdown (soporta múltiples bloques apilados)."""
    diagramas = _cargar_diagramas()
    try:
        svg = diagramas.renderizar_bloque_markdown(cuerpo, tinta=tinta)
        return ResultadoOraculo(ok=True, svg=svg, error=None)
    except diagramas.ErrorDiagrama as exc:
        return ResultadoOraculo(ok=False, svg=None, error=str(exc))


def nombre_svg_oraculo(cuerpo: str) -> str:
    """Calcula `estructura-<sha1[:12]>.svg` igual que `diagramas.nombre_svg`."""
    diagramas = _cargar_diagramas()
    return str(diagramas.nombre_svg(cuerpo))


def sincronizar_svgs_oraculo(
    directorio: Path,
    tinta: bool = True,
    escribir: bool = True,
) -> Any:
    """Delega a `diagramas.sincronizar_svgs`. Usar siempre sobre directorios temporales."""
    ruta_resuelta = Path(directorio).resolve()
    alucard_resuelto = ALUCARD_ROOT.resolve()
    if ruta_resuelta == alucard_resuelto or alucard_resuelto in ruta_resuelta.parents:
        raise ValueError(
            f"PROHIBIDO sincronizar dentro del repositorio de solo lectura: {ruta_resuelta}"
        )
    diagramas = _cargar_diagramas()
    return diagramas.sincronizar_svgs(ruta_resuelta, tinta=tinta, escribir=escribir)


def markdown_a_html_oraculo(texto_md: str, saltos_de_linea: bool = True) -> str:
    """Convierte Markdown con fences `estructura`/`statics` a HTML usando `alucarD`."""
    asegurar_alucard_en_sys_path()
    from generador_examenes.core.markdown_utils import markdown_to_html

    return str(markdown_to_html(texto_md, saltos_de_linea=saltos_de_linea))


def html_a_typst_oraculo(fragmento_html: str) -> str:
    """Convierte un fragmento HTML con SVGs embebidos a marcado Typst usando `alucarD`."""
    asegurar_alucard_en_sys_path()
    from generador_examenes.generators.typst_markup import html_a_typst

    return str(html_a_typst(fragmento_html))


def compilar_typst_oraculo(typst_src: str, output_pdf: Path) -> Path:
    """Compila código Typst a PDF usando `pdf_renderer.compilar_typst_a_pdf` en un destino temporal."""
    ruta_pdf = Path(output_pdf).resolve()
    alucard_resuelto = ALUCARD_ROOT.resolve()
    if alucard_resuelto in ruta_pdf.parents:
        raise ValueError(
            f"PROHIBIDO escribir PDF dentro del repositorio de solo lectura: {ruta_pdf}"
        )
    asegurar_alucard_en_sys_path()
    from generador_examenes.generators.pdf_renderer import compilar_typst_a_pdf

    return Path(compilar_typst_a_pdf(typst_src, ruta_pdf))
