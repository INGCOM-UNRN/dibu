"""Suite E2E opaque-box para los 21 tipos de diagramas de Química."""

from __future__ import annotations

import sys

sys.dont_write_bytecode = True

from e2e import harness
from e2e.harness import ContextoE2E, ResultadoCaso


def run_suite(ctx: ContextoE2E) -> list[ResultadoCaso]:
    """Ejecuta los casos declarativos de química general, orgánica e inorgánica."""
    return harness.ejecutar_casos_diagrama(
        [
            ctx.cases_dir / "chem_general.yaml",
            ctx.cases_dir / "chem_organica.yaml",
            ctx.cases_dir / "chem_inorganica.yaml",
        ],
        ctx,
    )
