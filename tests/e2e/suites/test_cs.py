"""Suite E2E opaque-box para los 14 tipos de diagramas de Ciencias de la Computación (CS)."""

from __future__ import annotations

import sys

sys.dont_write_bytecode = True

import xml.etree.ElementTree as ET
from typing import Any, Callable

from e2e import harness, oracle
from e2e.harness import ContextoE2E, ResultadoCaso

_POLYLINE_TAG = f"{{{harness.SVG_NS}}}polyline"
_CASE_ID_SANGRIA = "cs_cost_table_t1_sangria_y_dicts_color"
_CASE_ID_COMPLEXITY_CLIP = "cs_complexity_t2_clipping_polilinea_no_vacia"


def _texto_normalizado(elem: ET.Element) -> str:
    return " ".join("".join(elem.itertext()).replace("\xa0", " ").split())


def _verificar_xml_sangria_cost_table(svg_texto: str) -> list[str]:
    """Valida que la sangría de 4 y 8 espacios desplace `x` creciente entre líneas."""
    try:
        raiz = harness.parsear_svg(svg_texto)
    except ET.ParseError as exc:
        return [f"XML inválido: {exc}"]

    objetivos = (
        "for (int i = 0; i < n; i++)",
        "if (v[i] == x)",
        "return i;",
    )
    xs_por_texto: dict[str, float] = {}
    for elem in raiz.iter():
        if elem.tag not in (harness.TEXT_TAG, "text"):
            continue
        txt = _texto_normalizado(elem)
        if txt in objetivos and txt not in xs_por_texto:
            x_raw = elem.get("x")
            if x_raw is None:
                return [f"Elemento <text> para {txt!r} sin atributo x"]
            try:
                xs_por_texto[txt] = float(x_raw)
            except ValueError:
                return [f"Atributo x no numérico en {txt!r}: {x_raw!r}"]

    faltantes = [t for t in objetivos if t not in xs_por_texto]
    if faltantes:
        return [f"Faltan nodos <text> esperados para validar sangría: {faltantes}"]

    x1 = xs_por_texto[objetivos[0]]
    x2 = xs_por_texto[objetivos[1]]
    x3 = xs_por_texto[objetivos[2]]
    errores: list[str] = []
    if not (x2 > x1 + 15.0):
        errores.append(
            f"Sangría insuficiente en nivel 1: x({objetivos[1]!r})={x2:.1f} <= x({objetivos[0]!r})={x1:.1f} + 15"
        )
    if not (x3 > x2 + 15.0):
        errores.append(
            f"Sangría insuficiente en nivel 2: x({objetivos[2]!r})={x3:.1f} <= x({objetivos[1]!r})={x2:.1f} + 15"
        )
    return errores


def _verificar_xml_clipping_complexity(svg_texto: str) -> list[str]:
    """Valida que exista al menos una <polyline> no vacía recortada verticalmente (`y >= 37`)."""
    try:
        raiz = harness.parsear_svg(svg_texto)
    except ET.ParseError as exc:
        return [f"XML inválido: {exc}"]

    polilineas = [
        elem for elem in raiz.iter() if elem.tag in (_POLYLINE_TAG, "polyline")
    ]
    if not polilineas:
        return ["No se encontró ningún elemento <polyline> en el gráfico"]

    ys_totales: list[float] = []
    for poly in polilineas:
        pts_raw = (poly.get("points") or "").strip().split()
        if len(pts_raw) < 2:
            continue
        for par in pts_raw:
            partes = par.split(",")
            if len(partes) != 2:
                return [f"Punto inválido en <polyline>: {par!r}"]
            try:
                float(partes[0])
                ys_totales.append(float(partes[1]))
            except ValueError:
                return [f"Coordenadas no numéricas en <polyline>: {par!r}"]

    if len(ys_totales) < 2:
        return ["La <polyline> no contiene al menos 2 puntos válidos"]
    y_min = min(ys_totales)
    if y_min < 37.0:
        return [f"La curva excede el tope del gráfico (y_min={y_min:.1f} < 37.0)"]
    return []


def _renderizar_y_validar_extra(
    res: ResultadoCaso,
    caso: dict[str, Any],
    ctx: ContextoE2E,
    validador: Callable[[str], list[str]],
) -> ResultadoCaso:
    bloque = dict(caso["block"])
    tinta = bool(caso.get("tinta", True))

    res_ora = oracle.renderizar_oraculo(bloque, tinta=tinta)
    if not res_ora.ok or not res_ora.svg:
        return ResultadoCaso(
            case_id=res.case_id,
            feature=res.feature,
            tier=res.tier,
            status="FAIL",
            mensaje=f"Oráculo falló inesperadamente en verificación XML: {res_ora.error}",
        )
    err_ora = validador(res_ora.svg)
    if err_ora:
        return ResultadoCaso(
            case_id=res.case_id,
            feature=res.feature,
            tier=res.tier,
            status="FAIL",
            mensaje=f"[oráculo XML] {'; '.join(err_ora)}",
        )

    if ctx.oracle_only:
        return res

    if ctx.dibu_bin is None:
        return ResultadoCaso(
            case_id=res.case_id,
            feature=res.feature,
            tier=res.tier,
            status="FAIL",
            mensaje="No se proveyó binario `dibu` para verificación XML",
        )

    cli_res, items = harness.invocar_dibu_batch(
        ctx.dibu_bin,
        [{"id": res.case_id, "block": bloque, "tinta": tinta}],
    )
    if (
        cli_res.returncode != 0
        or harness.tiene_panic_o_traceback(cli_res.stderr)
        or len(items) != 1
        or items[0].error
        or not items[0].svg
    ):
        err_det = items[0].error if items else cli_res.stderr.strip()
        return ResultadoCaso(
            case_id=res.case_id,
            feature=res.feature,
            tier=res.tier,
            status="FAIL",
            mensaje=f"`dibu batch` falló en verificación XML: {err_det}",
        )

    err_dibu = validador(items[0].svg)
    if err_dibu:
        return ResultadoCaso(
            case_id=res.case_id,
            feature=res.feature,
            tier=res.tier,
            status="FAIL",
            mensaje=f"[dibu XML] {'; '.join(err_dibu)}",
        )
    return res


def run_suite(ctx: ContextoE2E) -> list[ResultadoCaso]:
    """Ejecuta los casos declarativos de `cases/cs_lineal.yaml` y `cases/cs_arboles_algos.yaml`."""
    ruta_lineal = ctx.cases_dir / "cs_lineal.yaml"
    ruta_arboles = ctx.cases_dir / "cs_arboles_algos.yaml"
    resultados = harness.ejecutar_casos_diagrama(
        [ruta_lineal, ruta_arboles],
        ctx,
    )

    validadores: dict[str, Callable[[str], list[str]]] = {
        _CASE_ID_SANGRIA: _verificar_xml_sangria_cost_table,
        _CASE_ID_COMPLEXITY_CLIP: _verificar_xml_clipping_complexity,
    }
    necesita_extra = any(
        r.status == "PASS" and r.case_id in validadores for r in resultados
    )
    if not necesita_extra:
        return resultados

    casos_por_id = {
        str(c["id"]): c
        for ruta in (ruta_lineal, ruta_arboles)
        for c in harness.cargar_casos_yaml(ruta)
    }
    for idx, res in enumerate(resultados):
        if res.status == "PASS" and res.case_id in validadores:
            caso = casos_por_id.get(res.case_id)
            if caso is not None:
                resultados[idx] = _renderizar_y_validar_extra(
                    res,
                    caso,
                    ctx,
                    validadores[res.case_id],
                )
    return resultados
