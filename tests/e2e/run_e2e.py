#!/usr/bin/env python3
"""Runner único de la suite E2E opaque-box para `dibu`."""

from __future__ import annotations

import sys

sys.dont_write_bytecode = True

import argparse
import importlib
import json
import tempfile
from collections import Counter
from pathlib import Path
from typing import Any

DIBU_ROOT = Path(__file__).resolve().parents[2]
TESTS_ROOT = Path(__file__).resolve().parents[1]
if str(TESTS_ROOT) not in sys.path:
    sys.path.insert(0, str(TESTS_ROOT))

from e2e import harness, oracle  # noqa: E402
from e2e.harness import ContextoE2E, ResultadoCaso  # noqa: E402

SUITES_ORDEN: tuple[tuple[str, str], ...] = (
    ("cli", "e2e.suites.test_cli"),
    ("control", "e2e.suites.test_control"),
    ("chem", "e2e.suites.test_chem"),
    ("cs", "e2e.suites.test_cs"),
    ("tier3", "e2e.suites.test_tier3"),
    ("tier4", "e2e.suites.test_tier4"),
)

SUITES_MAPA = dict(SUITES_ORDEN)


def _cargar_modulo_suite(nombre_modulo: str):
    try:
        return importlib.import_module(nombre_modulo)
    except ModuleNotFoundError as exc:
        if exc.name == nombre_modulo:
            return None
        raise


def _parsear_args(argv: list[str] | None = None) -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Runner E2E opaque-box de `dibu` contra el oráculo `alucarD`"
    )
    parser.add_argument(
        "--tier",
        choices=("1", "2", "3", "4", "all"),
        default="all",
        help="Filtrar por tier metodológico (default: all)",
    )
    parser.add_argument(
        "--suite",
        choices=("cli", "control", "chem", "cs", "tier3", "tier4", "all"),
        default="all",
        help="Suite o dominio a ejecutar (default: all)",
    )
    parser.add_argument(
        "--solo-implementados",
        action="store_true",
        help="Evalúa `dibu` solo sobre los tipos presentes en `dibu list-types` y marca SKIP en pendientes/M2",
    )
    parser.add_argument(
        "--oracle-only",
        action="store_true",
        help="Valida los casos contra `oracle.py` sin requerir el binario `dibu`",
    )
    parser.add_argument(
        "--dibu-bin",
        type=Path,
        default=None,
        help="Ruta explícita al binario `dibu`",
    )
    parser.add_argument(
        "--json-report",
        type=Path,
        default=None,
        help="Ruta donde escribir el reporte JSON estructurado",
    )
    parser.add_argument(
        "--verbose",
        action="store_true",
        help="Muestra el detalle de todos los casos (PASS/SKIP/FAIL)",
    )
    return parser.parse_args(argv)


def main(argv: list[str] | None = None) -> int:
    args = _parsear_args(argv)

    dibu_bin: Path | None = None
    tipos_implementados: set[str] = set(oracle.TODOS_LOS_TIPOS)

    if not args.oracle_only:
        try:
            dibu_bin = harness.resolver_dibu_bin(args.dibu_bin, compilar_si_falta=True)
            tipos_implementados = set(harness.invocar_dibu_list_types(dibu_bin))
        except Exception as exc:
            print(f"[FATAL] No se pudo inicializar el binario `dibu`: {exc}", file=sys.stderr)
            return 1

    suites_objetivo = (
        [nombre for nombre, _ in SUITES_ORDEN]
        if args.suite == "all"
        else [args.suite]
    )

    resultados_por_suite: dict[str, list[ResultadoCaso]] = {}
    todos_resultados: list[ResultadoCaso] = []

    with tempfile.TemporaryDirectory(prefix="dibu_e2e_") as tmp_str:
        work_dir = Path(tmp_str)
        ctx = ContextoE2E(
            dibu_bin=dibu_bin,
            tipos_implementados=tipos_implementados,
            tier=args.tier,
            solo_implementados=args.solo_implementados,
            oracle_only=args.oracle_only,
            verbose=args.verbose,
            cases_dir=DIBU_ROOT / "tests" / "e2e" / "cases",
            work_dir=work_dir,
        )

        # En modo estricto con --suite all, verificar que los 44 tipos estén compilados.
        if (
            args.suite == "all"
            and not args.oracle_only
            and not args.solo_implementados
        ):
            faltantes = sorted(set(oracle.TODOS_LOS_TIPOS) - tipos_implementados)
            if faltantes:
                res_gate = ResultadoCaso(
                    case_id="gate_all_44_types",
                    feature="cli:list-types",
                    tier=1,
                    status="FAIL",
                    mensaje=(
                        f"Faltan {len(faltantes)} de 44 tipos en `dibu list-types`: "
                        f"{', '.join(faltantes[:15])}"
                        + ("..." if len(faltantes) > 15 else "")
                    ),
                )
                resultados_por_suite["global_gate"] = [res_gate]
                todos_resultados.append(res_gate)

        for nombre_suite in suites_objetivo:
            mod_name = SUITES_MAPA[nombre_suite]
            modulo = _cargar_modulo_suite(mod_name)
            if modulo is None:
                estricto = not args.solo_implementados and not args.oracle_only
                status_missing = "FAIL" if (estricto or args.suite != "all") else "SKIP"
                res_missing = ResultadoCaso(
                    case_id=f"suite_{nombre_suite}_disponible",
                    feature=f"suite:{nombre_suite}",
                    tier=0,
                    status=status_missing,
                    mensaje=f"Suite {mod_name} aún no implementada",
                )
                resultados_por_suite[nombre_suite] = [res_missing]
                todos_resultados.append(res_missing)
                continue

            suite_dir = work_dir / nombre_suite
            suite_dir.mkdir(parents=True, exist_ok=True)
            ctx_suite = ContextoE2E(
                dibu_bin=ctx.dibu_bin,
                tipos_implementados=ctx.tipos_implementados,
                tier=ctx.tier,
                solo_implementados=ctx.solo_implementados,
                oracle_only=ctx.oracle_only,
                verbose=ctx.verbose,
                cases_dir=ctx.cases_dir,
                work_dir=suite_dir,
            )
            try:
                res_suite = modulo.run_suite(ctx_suite)
            except Exception as exc:
                res_suite = [
                    ResultadoCaso(
                        case_id=f"suite_{nombre_suite}_exception",
                        feature=f"suite:{nombre_suite}",
                        tier=0,
                        status="FAIL",
                        mensaje=f"Excepción no controlada ejecutando {mod_name}: {type(exc).__name__}: {exc}",
                    )
                ]
            resultados_por_suite[nombre_suite] = res_suite
            todos_resultados.extend(res_suite)

    conteo = Counter(r.status for r in todos_resultados)
    total = len(todos_resultados)
    n_pass = conteo["PASS"]
    n_fail = conteo["FAIL"]
    n_skip = conteo["SKIP"]

    modo_desc = (
        "oracle-only"
        if args.oracle_only
        else ("solo-implementados" if args.solo_implementados else "estricto")
    )
    print(
        f"=== E2E dibu | suite={args.suite} | tier={args.tier} | modo={modo_desc} ==="
    )
    if dibu_bin is not None:
        print(f"Binario: {dibu_bin} ({len(tipos_implementados)} tipos en list-types)")
    else:
        print("Binario: <ninguno (oracle-only)>")

    for nombre_suite, lista in resultados_por_suite.items():
        c_s = Counter(r.status for r in lista)
        print(
            f"  - {nombre_suite}: total={len(lista)} | PASS={c_s['PASS']} | FAIL={c_s['FAIL']} | SKIP={c_s['SKIP']}"
        )
        for r in lista:
            if r.status == "FAIL" or args.verbose:
                extra = f" -> {r.mensaje}" if r.mensaje else ""
                print(f"      [{r.status}] (T{r.tier}) {r.case_id} [{r.feature}]{extra}")

    print(
        f"Resumen total: total={total} | PASS={n_pass} | FAIL={n_fail} | SKIP={n_skip}"
    )

    if args.json_report is not None:
        args.json_report.parent.mkdir(parents=True, exist_ok=True)
        payload: dict[str, Any] = {
            "config": {
                "suite": args.suite,
                "tier": args.tier,
                "modo": modo_desc,
                "dibu_bin": str(dibu_bin) if dibu_bin else None,
                "tipos_implementados": sorted(tipos_implementados),
            },
            "summary": {
                "total": total,
                "pass": n_pass,
                "fail": n_fail,
                "skip": n_skip,
            },
            "suites": {
                nombre: {
                    "total": len(lista),
                    "pass": sum(1 for r in lista if r.status == "PASS"),
                    "fail": sum(1 for r in lista if r.status == "FAIL"),
                    "skip": sum(1 for r in lista if r.status == "SKIP"),
                }
                for nombre, lista in resultados_por_suite.items()
            },
            "failures": [
                {
                    "case_id": r.case_id,
                    "feature": r.feature,
                    "tier": r.tier,
                    "mensaje": r.mensaje,
                }
                for r in todos_resultados
                if r.status == "FAIL"
            ],
            "skips": [
                {
                    "case_id": r.case_id,
                    "feature": r.feature,
                    "tier": r.tier,
                    "mensaje": r.mensaje,
                }
                for r in todos_resultados
                if r.status == "SKIP"
            ],
        }
        args.json_report.write_text(
            json.dumps(payload, indent=2, ensure_ascii=False) + "\n",
            encoding="utf-8",
        )

    return 0 if n_fail == 0 else 1


if __name__ == "__main__":
    raise SystemExit(main())
