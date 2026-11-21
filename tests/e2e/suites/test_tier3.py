"""Suite E2E Tier 3: Combinaciones pairwise cross-feature (YAML + composición CLI)."""

from __future__ import annotations

import sys

sys.dont_write_bytecode = True

from pathlib import Path
from typing import Any

from e2e import harness, oracle
from e2e.harness import ContextoE2E, ResultadoCaso

_BLOQUE_BODE = {
    "type": "bode",
    "title": "Bode T3",
    "system": {"k": 10, "poles": [0, -1, -10]},
    "margins": True,
}
_BLOQUE_ARRAY = {
    "type": "array",
    "values": [4, 8, 15],
    "showAddresses": True,
    "inkSaver": False,
}
_BLOQUE_TREE = {
    "type": "tree",
    "variant": "bst",
    "insert": [20, 10, 30],
    "highlight": [10],
}
_BLOQUE_VSEPR = {
    "type": "vsepr",
    "central": "N",
    "ligands": ["H", "H", "H"],
    "lonePairs": 1,
}
_BLOQUE_LINKED = {
    "type": "linked_list",
    "variant": "singly",
    "values": [1, 2, 3],
    "inkSaver": True,
}


def _ok(cid: str, feat: str, msg: str = "") -> ResultadoCaso:
    return ResultadoCaso(case_id=cid, feature=feat, tier=3, status="PASS", mensaje=msg)


def _fail(cid: str, feat: str, msg: str) -> ResultadoCaso:
    return ResultadoCaso(case_id=cid, feature=feat, tier=3, status="FAIL", mensaje=msg)


def _skip(cid: str, feat: str, msg: str) -> ResultadoCaso:
    return ResultadoCaso(case_id=cid, feature=feat, tier=3, status="SKIP", mensaje=msg)


def _faltantes_tipos(tipos_req: set[str], ctx: ContextoE2E) -> list[str]:
    return sorted(tipos_req - ctx.tipos_implementados)


def _caso_batch_control_cs_mixto(ctx: ContextoE2E) -> ResultadoCaso:
    cid = "tier3_cli_batch_control_cs_y_error"
    feat = "cli:batch"
    req_tipos = {"bode", "array", "tree"}

    items = [
        {"id": "mix_bode", "block": _BLOQUE_BODE, "tinta": True},
        {"id": "mix_arr", "block": _BLOQUE_ARRAY, "tinta": True},  # inkSaver: false -> color
        {"id": "mix_tree", "block": _BLOQUE_TREE, "tinta": False},
        {"id": "mix_err", "block": {"type": "bode"}, "tinta": True},
    ]

    if ctx.oracle_only:
        for it in items[:-1]:
            ora = oracle.renderizar_oraculo(it["block"], tinta=bool(it["tinta"]))
            if not ora.ok or not ora.svg:
                return _fail(cid, feat, f"Oráculo falló en {it['id']}: {ora.error}")
            t_ef = it["block"].get("inkSaver") if isinstance(it["block"].get("inkSaver"), bool) else bool(it["tinta"])
            err = harness.verificar_svg(ora.svg, tinta_esperada=bool(t_ef))
            if err:
                return _fail(cid, feat, f"Oráculo SVG {it['id']}: {'; '.join(err)}")
        ora_err = oracle.renderizar_oraculo(items[-1]["block"], tinta=True)
        if ora_err.ok:
            return _fail(cid, feat, "Oráculo aceptó bloque inválido")
        return _ok(cid, feat, "Validado contra oráculo (--oracle-only)")

    faltan = _faltantes_tipos(req_tipos, ctx)
    if faltan:
        if ctx.solo_implementados:
            return _skip(cid, feat, f"Faltan tipos en dibu list-types: {faltan}")
        return _fail(cid, feat, f"Faltan tipos en dibu list-types: {faltan}")

    assert ctx.dibu_bin is not None
    r1, out1 = harness.invocar_dibu_batch(ctx.dibu_bin, items)
    r2, _ = harness.invocar_dibu_batch(ctx.dibu_bin, items)
    if r1.returncode != 0 or harness.tiene_panic_o_traceback(r1.stderr):
        return _fail(cid, feat, f"batch falló rc={r1.returncode}: {r1.stderr!r}")
    if r1.stdout != r2.stdout:
        return _fail(cid, feat, "Falla de determinismo byte-a-byte en batch mixto")
    if [x.id for x in out1] != ["mix_bode", "mix_arr", "mix_tree", "mix_err"]:
        return _fail(cid, feat, f"Orden inválido: {[x.id for x in out1]}")
    for req_it, resp_it in zip(items[:-1], out1[:-1]):
        if not resp_it.svg:
            return _fail(cid, feat, f"{resp_it.id} devolvió error: {resp_it.error}")
        ora = oracle.renderizar_oraculo(req_it["block"], tinta=bool(req_it["tinta"]))
        t_ef = req_it["block"].get("inkSaver") if isinstance(req_it["block"].get("inkSaver"), bool) else bool(req_it["tinta"])
        err = harness.comparar_con_oraculo(resp_it.svg, ora.svg or "", tinta_esperada=bool(t_ef))
        if err:
            return _fail(cid, feat, f"{resp_it.id}: {'; '.join(err)}")
    if out1[-1].svg is not None or not out1[-1].error:
        return _fail(cid, feat, f"Ítem inválido no devolvió error controlado: {out1[-1]}")
    return _ok(cid, feat)


def _caso_sync_md_apilado_control_cs(ctx: ContextoE2E) -> ResultadoCaso:
    cid = "tier3_cli_sync_md_apilado_control_cs"
    feat = "cli:sync-md"
    req_tipos = {"block_diagram", "array", "pole_zero", "stack"}
    case_dir = ctx.work_dir / cid
    case_dir.mkdir(parents=True, exist_ok=True)

    c1 = (
        "- type: block_diagram\n  forward: [\"Gc(s)\"]\n  feedback: []\n"
        "- type: array\n  values: [1, 2, 3]\n"
    )
    c2 = (
        "- type: pole_zero\n  poles: [-1, -2]\n"
        "- type: stack\n  values: [10, 20]\n  capacity: 3\n"
    )
    (case_dir / "mixto.md").write_text(
        f"# Mixto Control + CS\n\n```estructura\n{c1}```\n\n~~~statics\n{c2}~~~\n",
        encoding="utf-8",
    )

    if ctx.oracle_only:
        s1 = oracle.sincronizar_svgs_oraculo(case_dir, tinta=False, escribir=True)
        if len(s1.escritos) != 2 or s1.errores:
            return _fail(cid, feat, f"Oráculo sync falló: {s1}")
        s2 = oracle.sincronizar_svgs_oraculo(case_dir, tinta=False, escribir=False)
        if s2.escritos or s2.borrados or s2.errores:
            return _fail(cid, feat, f"Oráculo sync no idempotente: {s2}")
        for cuerpo in (c1, c2):
            p = case_dir / "diagramas" / oracle.nombre_svg_oraculo(cuerpo)
            err = harness.verificar_svg(p.read_text(encoding="utf-8"), tinta_esperada=False, apilado=True)
            if err:
                return _fail(cid, feat, f"SVG apilado inválido: {'; '.join(err)}")
        return _ok(cid, feat, "Validado contra oráculo (--oracle-only)")

    faltan = _faltantes_tipos(req_tipos, ctx)
    if faltan:
        if ctx.solo_implementados:
            return _skip(cid, feat, f"Faltan tipos en dibu list-types: {faltan}")
        return _fail(cid, feat, f"Faltan tipos en dibu list-types: {faltan}")

    assert ctx.dibu_bin is not None
    r1 = harness.invocar_dibu_sync_md(ctx.dibu_bin, [case_dir], color=True)
    if r1.returncode != 1 or len(r1.escritos) != 2 or r1.errores_stderr:
        return _fail(cid, feat, f"sync-md falló: rc={r1.returncode}, escritos={r1.escritos}, stderr={r1.stderr!r}")
    r_ver = harness.invocar_dibu_sync_md(ctx.dibu_bin, [case_dir], verificar=True, color=True)
    if r_ver.returncode != 0:
        return _fail(cid, feat, f"sync-md --verificar esperaba rc=0, obtuvo rc={r_ver.returncode}")
    for cuerpo in (c1, c2):
        p = case_dir / "diagramas" / oracle.nombre_svg_oraculo(cuerpo)
        if not p.is_file():
            return _fail(cid, feat, f"Falta archivo esperado {p.name}")
        ora = oracle.renderizar_bloque_md_oraculo(cuerpo, tinta=False)
        err = harness.comparar_con_oraculo(
            p.read_text(encoding="utf-8"),
            ora.svg or "",
            tinta_esperada=False,
            apilado=True,
        )
        if err:
            return _fail(cid, feat, f"{p.name}: {'; '.join(err)}")
    return _ok(cid, feat)


def _caso_render_multibloque_control_cs(ctx: ContextoE2E) -> ResultadoCaso:
    cid = "tier3_cli_render_multibloque_control_cs_con_o"
    feat = "cli:render"
    req_tipos = {"time_response", "matrix", "cost_table"}
    case_dir = ctx.work_dir / cid
    case_dir.mkdir(parents=True, exist_ok=True)

    bloques = [
        {"id": "tr_color", "type": "time_response", "system": {"gain": 1, "tau": 2}},
        {"id": "mat_ink", "type": "matrix", "values": [[1, 2], [3, 4]], "inkSaver": True},
        {"id": "cost_color", "type": "cost_table", "lines": [["x = 1;", "c1", 1]], "total": "c1"},
    ]

    if ctx.oracle_only:
        for b in bloques:
            t_ef = b.get("inkSaver") if isinstance(b.get("inkSaver"), bool) else False
            ora = oracle.renderizar_oraculo(b, tinta=False)
            if not ora.ok or not ora.svg:
                return _fail(cid, feat, f"Oráculo falló en {b['id']}: {ora.error}")
            err = harness.verificar_svg(ora.svg, tinta_esperada=bool(t_ef))
            if err:
                return _fail(cid, feat, f"{b['id']}: {'; '.join(err)}")
        return _ok(cid, feat, "Validado contra oráculo (--oracle-only)")

    faltan = _faltantes_tipos(req_tipos, ctx)
    if faltan:
        if ctx.solo_implementados:
            return _skip(cid, feat, f"Faltan tipos en dibu list-types: {faltan}")
        return _fail(cid, feat, f"Faltan tipos en dibu list-types: {faltan}")

    assert ctx.dibu_bin is not None
    import json

    entrada = case_dir / "multi.json"
    entrada.write_text(json.dumps(bloques, ensure_ascii=False), encoding="utf-8")
    out1 = case_dir / "out1"
    out2 = case_dir / "out2"
    r1 = harness.invocar_dibu_render(ctx.dibu_bin, entrada, output_dir=out1, color=True)
    r2 = harness.invocar_dibu_render(ctx.dibu_bin, entrada, output_dir=out2, color=True)
    if r1.returncode != 0 or r2.returncode != 0:
        return _fail(cid, feat, f"render falló: r1={r1.returncode}, r2={r2.returncode}, stderr={r1.stderr!r}")
    for b in bloques:
        f1 = out1 / f"{b['id']}.svg"
        f2 = out2 / f"{b['id']}.svg"
        if not f1.is_file() or not f2.is_file():
            return _fail(cid, feat, f"Falta salida {b['id']}.svg")
        if f1.read_bytes() != f2.read_bytes():
            return _fail(cid, feat, f"No determinista en {b['id']}.svg")
        t_ef = b.get("inkSaver") if isinstance(b.get("inkSaver"), bool) else False
        ora = oracle.renderizar_oraculo(b, tinta=False)
        err = harness.comparar_con_oraculo(
            f1.read_text(encoding="utf-8"),
            ora.svg or "",
            tinta_esperada=bool(t_ef),
        )
        if err:
            return _fail(cid, feat, f"{b['id']}: {'; '.join(err)}")
    return _ok(cid, feat)


def _caso_batch_tres_dominios(ctx: ContextoE2E) -> ResultadoCaso:
    cid = "tier3_cli_batch_tres_dominios_con_error"
    feat = "cli:batch"
    req_tipos = {"bode", "vsepr", "linked_list"}

    items = [
        {"id": "d_ctrl", "block": _BLOQUE_BODE, "tinta": False},
        {"id": "d_chem", "block": _BLOQUE_VSEPR, "tinta": True},
        {"id": "d_cs", "block": _BLOQUE_LINKED, "tinta": False},  # inkSaver: true
        {"id": "d_err", "block": {"type": "vsepr"}, "tinta": True},
    ]

    if ctx.oracle_only:
        for it in items[:-1]:
            ora = oracle.renderizar_oraculo(it["block"], tinta=bool(it["tinta"]))
            if not ora.ok or not ora.svg:
                return _fail(cid, feat, f"Oráculo falló en {it['id']}: {ora.error}")
            t_ef = it["block"].get("inkSaver") if isinstance(it["block"].get("inkSaver"), bool) else bool(it["tinta"])
            err = harness.verificar_svg(ora.svg, tinta_esperada=bool(t_ef))
            if err:
                return _fail(cid, feat, f"{it['id']}: {'; '.join(err)}")
        if oracle.renderizar_oraculo(items[-1]["block"], tinta=True).ok:
            return _fail(cid, feat, "Oráculo aceptó bloque vsepr inválido")
        return _ok(cid, feat, "Validado contra oráculo (--oracle-only)")

    faltan = _faltantes_tipos(req_tipos, ctx)
    if faltan:
        if ctx.solo_implementados:
            return _skip(cid, feat, f"Pendiente tipos aún no implementados en `dibu`: {faltan}")
        return _fail(cid, feat, f"Faltan tipos en `dibu list-types`: {faltan}")

    assert ctx.dibu_bin is not None
    r1, out1 = harness.invocar_dibu_batch(ctx.dibu_bin, items)
    r2, _ = harness.invocar_dibu_batch(ctx.dibu_bin, items)
    if r1.returncode != 0 or r1.stdout != r2.stdout:
        return _fail(cid, feat, f"batch falló o no determinista: rc={r1.returncode}")
    for req_it, resp_it in zip(items[:-1], out1[:-1]):
        if not resp_it.svg:
            return _fail(cid, feat, f"{resp_it.id} error: {resp_it.error}")
        ora = oracle.renderizar_oraculo(req_it["block"], tinta=bool(req_it["tinta"]))
        t_ef = req_it["block"].get("inkSaver") if isinstance(req_it["block"].get("inkSaver"), bool) else bool(req_it["tinta"])
        err = harness.comparar_con_oraculo(resp_it.svg, ora.svg or "", tinta_esperada=bool(t_ef))
        if err:
            return _fail(cid, feat, f"{resp_it.id}: {'; '.join(err)}")
    if out1[-1].svg is not None or not out1[-1].error:
        return _fail(cid, feat, f"Ítem inválido no devolvió error: {out1[-1]}")
    return _ok(cid, feat)


def _caso_sync_md_tres_dominios(ctx: ContextoE2E) -> ResultadoCaso:
    cid = "tier3_cli_sync_md_apilado_tres_dominios"
    feat = "cli:sync-md"
    req_tipos = {"bode", "reaction", "flowchart"}
    case_dir = ctx.work_dir / cid
    case_dir.mkdir(parents=True, exist_ok=True)

    cuerpo = (
        "- type: bode\n  system: {tau: 1}\n"
        "- type: reaction\n  steps: [\"A\", {\"arrow\": \"->\"}, \"B\"]\n"
        "- type: flowchart\n  steps: [\"inicio\", {\"io\": \"leer x\"}]\n"
    )
    (case_dir / "tri.md").write_text(f"```estructura\n{cuerpo}```\n", encoding="utf-8")

    if ctx.oracle_only:
        s1 = oracle.sincronizar_svgs_oraculo(case_dir, tinta=True, escribir=True)
        if len(s1.escritos) != 1 or s1.errores:
            return _fail(cid, feat, f"Oráculo sync falló: {s1}")
        p = case_dir / "diagramas" / oracle.nombre_svg_oraculo(cuerpo)
        err = harness.verificar_svg(p.read_text(encoding="utf-8"), tinta_esperada=True, apilado=True)
        if err:
            return _fail(cid, feat, "; ".join(err))
        s2 = oracle.sincronizar_svgs_oraculo(case_dir, tinta=True, escribir=False)
        if s2.escritos or s2.borrados or s2.errores:
            return _fail(cid, feat, f"Oráculo sync no idempotente: {s2}")
        return _ok(cid, feat, "Validado contra oráculo (--oracle-only)")

    faltan = _faltantes_tipos(req_tipos, ctx)
    if faltan:
        if ctx.solo_implementados:
            return _skip(cid, feat, f"Pendiente tipos aún no implementados en `dibu`: {faltan}")
        return _fail(cid, feat, f"Faltan tipos en `dibu list-types`: {faltan}")

    assert ctx.dibu_bin is not None
    r1 = harness.invocar_dibu_sync_md(ctx.dibu_bin, [case_dir])
    if r1.returncode != 1 or len(r1.escritos) != 1 or r1.errores_stderr:
        return _fail(cid, feat, f"sync-md falló: rc={r1.returncode}, escritos={r1.escritos}, stderr={r1.stderr!r}")
    r_ver = harness.invocar_dibu_sync_md(ctx.dibu_bin, [case_dir], verificar=True)
    if r_ver.returncode != 0:
        return _fail(cid, feat, f"sync-md --verificar esperaba rc=0, obtuvo rc={r_ver.returncode}")
    p = case_dir / "diagramas" / oracle.nombre_svg_oraculo(cuerpo)
    ora = oracle.renderizar_bloque_md_oraculo(cuerpo, tinta=True)
    err = harness.comparar_con_oraculo(p.read_text(encoding="utf-8"), ora.svg or "", tinta_esperada=True, apilado=True)
    if err:
        return _fail(cid, feat, "; ".join(err))
    return _ok(cid, feat)


def _caso_list_types_cobertura_dominios(ctx: ContextoE2E) -> ResultadoCaso:
    cid = "tier3_cli_list_types_dominios_y_oracle"
    feat = "cli:list-types"
    tipos_ora = set(oracle.listar_tipos_oraculo())
    if len(tipos_ora) != 44:
        return _fail(cid, feat, f"Oráculo no expone 44 tipos: {len(tipos_ora)}")

    if ctx.oracle_only:
        return _ok(cid, feat, "44 tipos en 3 dominios verificados en oráculo (--oracle-only)")

    faltan = _faltantes_tipos(tipos_ora, ctx)
    if faltan:
        if ctx.solo_implementados:
            return _skip(cid, feat, f"Faltan {len(faltan)} tipos en `dibu list-types`: {faltan[:6]}...")
        return _fail(cid, feat, f"Faltan {len(faltan)} tipos en `dibu list-types`: {faltan[:10]}")
    return _ok(cid, feat)


def run_suite(ctx: ContextoE2E) -> list[ResultadoCaso]:
    """Ejecuta Tier 3 (`tier3_combinaciones.yaml` + composición CLI cross-feature)."""
    if ctx.tier not in ("all", "3"):
        return []

    resultados = harness.ejecutar_casos_diagrama(
        [ctx.cases_dir / "tier3_combinaciones.yaml"],
        ctx,
    )
    resultados.extend(
        [
            _caso_list_types_cobertura_dominios(ctx),
            _caso_batch_control_cs_mixto(ctx),
            _caso_sync_md_apilado_control_cs(ctx),
            _caso_render_multibloque_control_cs(ctx),
            _caso_batch_tres_dominios(ctx),
            _caso_sync_md_tres_dominios(ctx),
        ]
    )
    return resultados
