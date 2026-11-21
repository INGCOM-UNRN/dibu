"""Suite E2E opaque-box para los 9 tipos de diagramas de Control de Procesos."""

from __future__ import annotations

import sys

sys.dont_write_bytecode = True

from e2e import harness
from e2e.harness import ContextoE2E, ResultadoCaso


def run_suite(ctx: ContextoE2E) -> list[ResultadoCaso]:
    """Ejecuta los casos declarativos de `cases/control.yaml`."""
    return harness.ejecutar_casos_diagrama(
        [ctx.cases_dir / "control.yaml"],
        ctx,
    )
