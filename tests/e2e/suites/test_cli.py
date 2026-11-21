"""Suite E2E opaque-box para los 4 subcomandos CLI de `dibu`."""

from __future__ import annotations

import sys

sys.dont_write_bytecode = True

import json
from pathlib import Path
from typing import Any

from e2e import harness, oracle
from e2e.harness import ContextoE2E, ResultadoCaso

_BLOQUE_BODE = {
    "type": "bode",
    "title": "Bode CLI",
    "system": {"k": 10, "poles": [0, -1, -10]},
    "margins": True,
}

_BLOQUE_POLOS = {
    "type": "pole_zero",
    "title": "Polos CLI",
    "poles": [-1, [-2, 3]],
    "zeros": [-4],
    "showValues": True,
}

_BLOQUE_BLOQUES = {
    "type": "block_diagram",
    "title": "Lazo CLI",
    "forward": ["Gc(s)", "Gp(s)"],
    "feedback": ["H(s)"],
}


def _ok(caso: dict[str, Any], msg: str = "") -> ResultadoCaso:
    return ResultadoCaso(
        case_id=str(caso["id"]),
        feature=str(caso["feature"]),
        tier=int(caso["tier"]),
        status="PASS",
        mensaje=msg,
    )


def _fail(caso: dict[str, Any], msg: str) -> ResultadoCaso:
    return ResultadoCaso(
        case_id=str(caso["id"]),
        feature=str(caso["feature"]),
        tier=int(caso["tier"]),
        status="FAIL",
        mensaje=msg,
    )


def _skip(caso: dict[str, Any], msg: str) -> ResultadoCaso:
    return ResultadoCaso(
        case_id=str(caso["id"]),
        feature=str(caso["feature"]),
        tier=int(caso["tier"]),
        status="SKIP",
        mensaje=msg,
    )


def _check_no_panic(res: harness.ResultadoCli, caso: dict[str, Any]) -> ResultadoCaso | None:
    if harness.tiene_panic_o_traceback(res.stdout) or harness.tiene_panic_o_traceback(res.stderr):
        return _fail(caso, f"Se detectó panic/traceback: stderr={res.stderr!r}")
    return None


def _run_oracle_action(caso: dict[str, Any], case_dir: Path) -> ResultadoCaso:
    action = str(caso["action"])

    if action in (
        "list_types_basic",
        "list_types_sorted_unique",
        "list_types_contains_control_9",
        "list_types_contains_all_44",
    ):
        tipos = oracle.listar_tipos_oraculo()
        if len(tipos) != 44 or len(set(tipos)) != 44:
            return _fail(caso, f"Oráculo esperaba 44 tipos únicos, obtuvo {len(tipos)}")
        if not set(oracle.TIPOS_CONTROL) <= set(tipos):
            return _fail(caso, "Oráculo no contiene los 9 tipos de control")
        return _ok(caso, "Validado contra `oracle.listar_tipos_oraculo`")

    if action in (
        "render_single_default_output",
        "render_multi_custom_id_and_dir",
        "render_json_blocks_wrapper",
        "render_color_flag",
        "render_ink_saver_override",
        "batch_single_default_tinta",
        "batch_tinta_false_color",
        "batch_multiple_preserves_order",
        "batch_ink_saver_overrides_tinta",
    ):
        tinta = action not in ("render_color_flag", "batch_tinta_false_color")
        res = oracle.renderizar_oraculo(_BLOQUE_BLOQUES, tinta=tinta)
        if not res.ok or not res.svg:
            return _fail(caso, f"Oráculo falló: {res.error}")
        err = harness.verificar_svg(res.svg, tinta_esperada=tinta)
        if err:
            return _fail(caso, f"Oráculo SVG inválido: {'; '.join(err)}")
        return _ok(caso, "Validado contra `oracle.renderizar_oraculo`")

    if action in ("render_malformed_yaml", "render_empty_file_and_list"):
        try:
            oracle.leer_bloques_oraculo("not: [valid")
            return _fail(caso, "Oráculo aceptó YAML malformado")
        except Exception:
            pass
        bloques = oracle.leer_bloques_oraculo("[]")
        if bloques != []:
            return _fail(caso, f"Oráculo esperaba [] para lista vacía, obtuvo {bloques}")
        return _ok(caso, "Validado contra `oracle.leer_bloques_oraculo`")

    if action in ("batch_domain_and_unknown_errors",):
        r1 = oracle.renderizar_oraculo({"type": "no_existe"})
        r2 = oracle.renderizar_oraculo({"type": "bode"})
        if r1.ok or r2.ok:
            return _fail(caso, "Oráculo aceptó bloques inválidos")
        return _ok(caso, "Validado contra `oracle.renderizar_oraculo` (errores)")

    if action in (
        "sync_md_dir_creates_hashed",
        "sync_md_idempotent_and_verificar",
        "sync_md_statics_and_tilde",
        "sync_md_multiblock_and_dedup",
        "sync_md_color_verificar_mismatch",
        "sync_md_relative_no_parent",
        "sync_md_orphan_and_empty_dir",
        "sync_md_invalid_block",
        "sync_md_unclosed_and_crlf",
    ):
        md_dir = case_dir / "ora_md"
        md_dir.mkdir(parents=True, exist_ok=True)
        cuerpo = "type: bode\nsystem: {tau: 1}\n"
        (md_dir / "doc.md").write_text(f"```estructura\n{cuerpo}```\n", encoding="utf-8")
        sync1 = oracle.sincronizar_svgs_oraculo(md_dir, tinta=True, escribir=True)
        if len(sync1.escritos) != 1 or sync1.errores:
            return _fail(caso, f"Oráculo sync inesperado: {sync1}")
        sync2 = oracle.sincronizar_svgs_oraculo(md_dir, tinta=True, escribir=False)
        if sync2.escritos or sync2.borrados or sync2.errores:
            return _fail(caso, f"Oráculo sync no idempotente: {sync2}")
        return _ok(caso, "Validado contra `oracle.sincronizar_svgs_oraculo`")

    return _skip(caso, "Contrato específico de binario CLI (omitido en --oracle-only)")


def _ejecutar_caso_cli(caso: dict[str, Any], ctx: ContextoE2E) -> ResultadoCaso:
    case_dir = ctx.work_dir / str(caso["id"])
    case_dir.mkdir(parents=True, exist_ok=True)

    if ctx.oracle_only:
        return _run_oracle_action(caso, case_dir)

    assert ctx.dibu_bin is not None
    dibu_bin = ctx.dibu_bin
    action = str(caso["action"])

    # --- list-types ---------------------------------------------------------
    if action == "list_types_basic":
        res = harness.ejecutar_cli(dibu_bin, ["list-types"])
        if panic := _check_no_panic(res, caso):
            return panic
        if res.returncode != 0 or res.stderr.strip() != "":
            return _fail(caso, f"rc={res.returncode}, stderr={res.stderr!r}")
        lines = [l.strip() for l in res.stdout.splitlines() if l.strip()]
        if not lines:
            return _fail(caso, "`list-types` devolvió salida vacía")
        return _ok(caso)

    if action == "list_types_sorted_unique":
        lines_raw = harness.ejecutar_cli(dibu_bin, ["list-types"]).stdout.splitlines()
        if any(not l.strip() for l in lines_raw):
            return _fail(caso, f"Hay líneas vacías en `list-types`: {lines_raw!r}")
        if lines_raw != sorted(lines_raw):
            return _fail(caso, f"`list-types` no está ordenado: {lines_raw}")
        if len(lines_raw) != len(set(lines_raw)):
            return _fail(caso, f"`list-types` contiene duplicados: {lines_raw}")
        return _ok(caso)

    if action == "list_types_contains_control_9":
        tipos = set(harness.invocar_dibu_list_types(dibu_bin))
        faltan = sorted(set(oracle.TIPOS_CONTROL) - tipos)
        if faltan:
            return _fail(caso, f"Faltan tipos de control: {faltan}")
        return _ok(caso)

    if action == "list_types_contains_all_44":
        tipos = set(harness.invocar_dibu_list_types(dibu_bin))
        esperados = set(oracle.TODOS_LOS_TIPOS)
        faltan = sorted(esperados - tipos)
        sobran = sorted(tipos - esperados)
        if not faltan and not sobran and len(tipos) == 44:
            return _ok(caso)
        msg = f"Faltan {len(faltan)} de 44 tipos ({faltan[:8]}...), sobran={sobran}"
        if ctx.solo_implementados:
            return _skip(caso, f"Pendiente M1/M2 (--solo-implementados): {msg}")
        return _fail(caso, msg)

    if action == "list_types_help":
        res = harness.ejecutar_cli(dibu_bin, ["list-types", "--help"])
        if panic := _check_no_panic(res, caso):
            return panic
        if res.returncode != 0 or "Usage:" not in res.stdout:
            return _fail(caso, f"rc={res.returncode}, stdout={res.stdout!r}")
        return _ok(caso)

    if action == "list_types_extra_arg":
        res = harness.ejecutar_cli(dibu_bin, ["list-types", "foo"])
        if panic := _check_no_panic(res, caso):
            return panic
        if res.returncode != 2 or "unexpected argument" not in res.stderr:
            return _fail(caso, f"Esperado rc=2 + unexpected argument, obtuvo rc={res.returncode}, stderr={res.stderr!r}")
        return _ok(caso)

    if action == "list_types_unknown_flag":
        res = harness.ejecutar_cli(dibu_bin, ["list-types", "--color"])
        if panic := _check_no_panic(res, caso):
            return panic
        if res.returncode != 2 or "unexpected argument" not in res.stderr:
            return _fail(caso, f"Esperado rc=2 + unexpected argument, obtuvo rc={res.returncode}, stderr={res.stderr!r}")
        return _ok(caso)

    if action == "list_types_determinism":
        r1 = harness.ejecutar_cli(dibu_bin, ["list-types"])
        r2 = harness.ejecutar_cli(dibu_bin, ["list-types"])
        if r1.stdout != r2.stdout or r1.returncode != 0 or r2.returncode != 0:
            return _fail(caso, "Salidas distintas entre dos corridas de `list-types`")
        return _ok(caso)

    if action == "list_types_ignores_stdin":
        r_ref = harness.ejecutar_cli(dibu_bin, ["list-types"])
        r_in = harness.ejecutar_cli(dibu_bin, ["list-types"], stdin_text="{basura total\n123\n")
        if r_in.returncode != 0 or r_in.stdout != r_ref.stdout:
            return _fail(caso, f"Stdin alteró `list-types`: rc={r_in.returncode}")
        return _ok(caso)

    if action == "root_no_args":
        res = harness.ejecutar_cli(dibu_bin, [])
        if panic := _check_no_panic(res, caso):
            return panic
        if res.returncode != 2 or "Usage: dibu" not in res.stderr:
            return _fail(caso, f"Esperado rc=2 y Usage, obtuvo rc={res.returncode}, stderr={res.stderr!r}")
        return _ok(caso)

    # --- render -------------------------------------------------------------
    if action == "render_single_default_output":
        entrada = case_dir / "unico.yaml"
        entrada.write_text(
            "type: block_diagram\nforward: [\"G1(s)\", \"G2(s)\"]\n",
            encoding="utf-8",
        )
        res = harness.invocar_dibu_render(dibu_bin, entrada, output_dir=None, cwd=case_dir)
        if panic := _check_no_panic(res, caso):
            return panic
        esperado_svg = case_dir / "output" / "unico_1.svg"
        if res.returncode != 0 or "✓" not in res.stdout or not esperado_svg.is_file():
            return _fail(caso, f"rc={res.returncode}, stdout={res.stdout!r}, stderr={res.stderr!r}, exists={esperado_svg.is_file()}")
        svg_txt = esperado_svg.read_text(encoding="utf-8")
        ora = oracle.renderizar_oraculo({"type": "block_diagram", "forward": ["G1(s)", "G2(s)"]}, tinta=True)
        err = harness.comparar_con_oraculo(svg_txt, ora.svg or "", tinta_esperada=True)
        if err:
            return _fail(caso, "; ".join(err))
        return _ok(caso)

    if action == "render_multi_custom_id_and_dir":
        entrada = case_dir / "varios.yaml"
        entrada.write_text(
            "- id: mi_bode\n  type: bode\n  system: {k: 10, poles: [0, -1, -10]}\n  margins: true\n"
            "- type: pole_zero\n  poles: [-1, -2]\n  zeros: [-3]\n",
            encoding="utf-8",
        )
        out_dir = case_dir / "custom_out"
        res = harness.invocar_dibu_render(dibu_bin, entrada, output_dir=out_dir)
        if panic := _check_no_panic(res, caso):
            return panic
        f1 = out_dir / "mi_bode.svg"
        f2 = out_dir / "varios_2.svg"
        if res.returncode != 0 or not f1.is_file() or not f2.is_file():
            return _fail(caso, f"rc={res.returncode}, f1={f1.is_file()}, f2={f2.is_file()}, stderr={res.stderr!r}")
        for f in (f1, f2):
            if err := harness.verificar_svg(f.read_text(encoding="utf-8"), tinta_esperada=True):
                return _fail(caso, f"{f.name}: {'; '.join(err)}")
        return _ok(caso)

    if action == "render_json_blocks_wrapper":
        entrada = case_dir / "doc.json"
        entrada.write_text(
            json.dumps({"blocks": [{"id": "pz1", "type": "pole_zero", "poles": [-1]}]}, ensure_ascii=False),
            encoding="utf-8",
        )
        out_dir = case_dir / "out"
        res = harness.invocar_dibu_render(dibu_bin, entrada, output_dir=out_dir)
        if panic := _check_no_panic(res, caso):
            return panic
        f1 = out_dir / "pz1.svg"
        if res.returncode != 0 or not f1.is_file():
            return _fail(caso, f"rc={res.returncode}, stderr={res.stderr!r}")
        if err := harness.verificar_svg(f1.read_text(encoding="utf-8"), tinta_esperada=True):
            return _fail(caso, "; ".join(err))
        return _ok(caso)

    if action == "render_color_flag":
        entrada = case_dir / "color.yaml"
        entrada.write_text(
            "type: block_diagram\nforward: [\"G(s)\"]\nfeedback: []\n",
            encoding="utf-8",
        )
        out_dir = case_dir / "out"
        res = harness.invocar_dibu_render(dibu_bin, entrada, output_dir=out_dir, color=True)
        if panic := _check_no_panic(res, caso):
            return panic
        svg_file = out_dir / "color_1.svg"
        if res.returncode != 0 or not svg_file.is_file():
            return _fail(caso, f"rc={res.returncode}, stderr={res.stderr!r}")
        ora = oracle.renderizar_oraculo({"type": "block_diagram", "forward": ["G(s)"], "feedback": []}, tinta=False)
        err = harness.comparar_con_oraculo(svg_file.read_text(encoding="utf-8"), ora.svg or "", tinta_esperada=False)
        if err:
            return _fail(caso, "; ".join(err))
        return _ok(caso)

    if action == "render_ink_saver_override":
        entrada = case_dir / "inksaver.yaml"
        entrada.write_text(
            "- id: forzar_tinta\n  type: block_diagram\n  forward: [\"G\"]\n  inkSaver: true\n"
            "- id: forzar_color\n  type: block_diagram\n  forward: [\"G\"]\n  inkSaver: false\n",
            encoding="utf-8",
        )
        out_color = case_dir / "out_color"
        out_tinta = case_dir / "out_tinta"
        r1 = harness.invocar_dibu_render(dibu_bin, entrada, output_dir=out_color, color=True)
        r2 = harness.invocar_dibu_render(dibu_bin, entrada, output_dir=out_tinta, color=False)
        if r1.returncode != 0 or r2.returncode != 0:
            return _fail(caso, f"r1={r1.returncode}, r2={r2.returncode}")
        e1 = harness.verificar_svg((out_color / "forzar_tinta.svg").read_text(encoding="utf-8"), tinta_esperada=True)
        e2 = harness.verificar_svg((out_tinta / "forzar_color.svg").read_text(encoding="utf-8"), tinta_esperada=False)
        if e1 or e2:
            return _fail(caso, f"inkSaver override falló: e1={e1}, e2={e2}")
        return _ok(caso)

    if action == "render_missing_file":
        res = harness.invocar_dibu_render(dibu_bin, case_dir / "no_existe.yaml", output_dir=case_dir / "out")
        if panic := _check_no_panic(res, caso):
            return panic
        if res.returncode != 2 or "Error leyendo" not in res.stderr:
            return _fail(caso, f"Esperado rc=2 + 'Error leyendo', obtuvo rc={res.returncode}, stderr={res.stderr!r}")
        return _ok(caso)

    if action == "render_malformed_yaml":
        entrada = case_dir / "roto.yaml"
        entrada.write_text("not: [valid\n", encoding="utf-8")
        res = harness.invocar_dibu_render(dibu_bin, entrada, output_dir=case_dir / "out")
        if panic := _check_no_panic(res, caso):
            return panic
        if res.returncode != 2 or "YAML/JSON válido" not in res.stderr:
            return _fail(caso, f"Esperado rc=2 + YAML inválido, obtuvo rc={res.returncode}, stderr={res.stderr!r}")
        return _ok(caso)

    if action == "render_empty_file_and_list":
        f_empty = case_dir / "vacio.yaml"
        f_empty.write_text("", encoding="utf-8")
        r_empty = harness.invocar_dibu_render(dibu_bin, f_empty, output_dir=case_dir / "out1")
        if panic := _check_no_panic(r_empty, caso):
            return panic
        if r_empty.returncode != 2 or "cada bloque tiene que ser un diccionario" not in r_empty.stderr:
            return _fail(caso, f"Archivo vacío esperaba rc=2, obtuvo rc={r_empty.returncode}, stderr={r_empty.stderr!r}")

        f_list = case_dir / "lista_vacia.yaml"
        f_list.write_text("[]\n", encoding="utf-8")
        out2 = case_dir / "out2"
        r_list = harness.invocar_dibu_render(dibu_bin, f_list, output_dir=out2)
        if panic := _check_no_panic(r_list, caso):
            return panic
        if r_list.returncode != 1 or list(out2.glob("*.svg")):
            return _fail(caso, f"Lista vacía esperaba rc=1 sin SVGs, obtuvo rc={r_list.returncode}")
        return _ok(caso)

    if action == "render_unsupported_and_partial":
        f_unk = case_dir / "solo_unk.yaml"
        f_unk.write_text("type: tipo_inexistente_xyz\n", encoding="utf-8")
        r_unk = harness.invocar_dibu_render(dibu_bin, f_unk, output_dir=case_dir / "out1")
        if panic := _check_no_panic(r_unk, caso):
            return panic
        if r_unk.returncode != 1 or "Aviso:" not in r_unk.stderr:
            return _fail(caso, f"Solo desconocido esperaba rc=1 + Aviso, obtuvo rc={r_unk.returncode}, stderr={r_unk.stderr!r}")

        f_mix = case_dir / "mix.yaml"
        f_mix.write_text(
            "- id: ok1\n  type: pole_zero\n  poles: [-1]\n"
            "- type: bode\n"
            "- type: tipo_inexistente_xyz\n",
            encoding="utf-8",
        )
        out2 = case_dir / "out2"
        r_mix = harness.invocar_dibu_render(dibu_bin, f_mix, output_dir=out2)
        if panic := _check_no_panic(r_mix, caso):
            return panic
        if (
            r_mix.returncode != 0
            or not (out2 / "ok1.svg").is_file()
            or "Error: bloque 2:" not in r_mix.stderr
            or "Aviso: bloque 3" not in r_mix.stderr
        ):
            return _fail(caso, f"Mix parcial inesperado: rc={r_mix.returncode}, stderr={r_mix.stderr!r}")
        return _ok(caso)

    if action == "render_id_fallback_and_determinism":
        entrada = case_dir / "ids.yaml"
        entrada.write_text(
            "- id: \"\"\n  type: pole_zero\n  poles: [-1]\n"
            "- id: 123\n  type: bode\n  system: {tau: 1}\n",
            encoding="utf-8",
        )
        out1 = case_dir / "out1"
        out2 = case_dir / "out2"
        r1 = harness.invocar_dibu_render(dibu_bin, entrada, output_dir=out1)
        r2 = harness.invocar_dibu_render(dibu_bin, entrada, output_dir=out2)
        if r1.returncode != 0 or r2.returncode != 0:
            return _fail(caso, f"r1={r1.returncode}, r2={r2.returncode}")
        for name in ("ids_1.svg", "ids_2.svg"):
            p1, p2 = out1 / name, out2 / name
            if not p1.is_file() or not p2.is_file():
                return _fail(caso, f"Falta archivo posicional esperado {name}")
            if p1.read_bytes() != p2.read_bytes():
                return _fail(caso, f"No determinista en {name}")
        return _ok(caso)

    # --- batch --------------------------------------------------------------
    if action == "batch_single_default_tinta":
        res, items = harness.invocar_dibu_batch(
            dibu_bin,
            [{"id": "b1", "block": _BLOQUE_BLOQUES}],
        )
        if panic := _check_no_panic(res, caso):
            return panic
        if res.returncode != 0 or len(items) != 1 or items[0].id != "b1" or not items[0].svg:
            return _fail(caso, f"rc={res.returncode}, items={items}")
        if err := harness.verificar_svg(items[0].svg, tinta_esperada=True):
            return _fail(caso, "; ".join(err))
        return _ok(caso)

    if action == "batch_tinta_false_color":
        res, items = harness.invocar_dibu_batch(
            dibu_bin,
            [{"id": "b2", "block": _BLOQUE_BLOQUES, "tinta": False}],
        )
        if panic := _check_no_panic(res, caso):
            return panic
        if res.returncode != 0 or len(items) != 1 or not items[0].svg:
            return _fail(caso, f"rc={res.returncode}, items={items}")
        if err := harness.verificar_svg(items[0].svg, tinta_esperada=False):
            return _fail(caso, "; ".join(err))
        return _ok(caso)

    if action == "batch_multiple_preserves_order":
        req = [
            {"id": "k1", "block": _BLOQUE_BLOQUES, "tinta": True},
            {"id": "k2", "block": _BLOQUE_BODE, "tinta": False},
            {"id": "k3", "block": _BLOQUE_POLOS, "tinta": True},
            {"id": "k4", "block": {"type": "grafcet", "steps": [{"step": 0, "initial": True}]}, "tinta": False},
        ]
        res, items = harness.invocar_dibu_batch(dibu_bin, req)
        if panic := _check_no_panic(res, caso):
            return panic
        if res.returncode != 0 or [it.id for it in items] != ["k1", "k2", "k3", "k4"]:
            return _fail(caso, f"Orden/conteo inválido: {[it.id for it in items]}")
        for r, it in zip(req, items):
            if not it.svg:
                return _fail(caso, f"Fallo en {it.id}: {it.error}")
            ora = oracle.renderizar_oraculo(r["block"], tinta=r["tinta"])
            err = harness.comparar_con_oraculo(it.svg, ora.svg or "", tinta_esperada=r["tinta"])
            if err:
                return _fail(caso, f"{it.id}: {'; '.join(err)}")
        return _ok(caso)

    if action == "batch_ink_saver_overrides_tinta":
        req = [
            {"id": "ov1", "block": {**_BLOQUE_BLOQUES, "inkSaver": False}, "tinta": True},
            {"id": "ov2", "block": {**_BLOQUE_BLOQUES, "inkSaver": True}, "tinta": False},
        ]
        res, items = harness.invocar_dibu_batch(dibu_bin, req)
        if res.returncode != 0 or len(items) != 2:
            return _fail(caso, f"rc={res.returncode}, len={len(items)}")
        e1 = harness.verificar_svg(items[0].svg or "", tinta_esperada=False)
        e2 = harness.verificar_svg(items[1].svg or "", tinta_esperada=True)
        if e1 or e2:
            return _fail(caso, f"e1={e1}, e2={e2}")
        return _ok(caso)

    if action == "batch_empty_stdin":
        res, items = harness.invocar_dibu_batch(dibu_bin, [])
        if panic := _check_no_panic(res, caso):
            return panic
        if res.returncode != 0 or items or res.stdout != "" or res.stderr != "":
            return _fail(caso, f"rc={res.returncode}, stdout={res.stdout!r}, stderr={res.stderr!r}")
        return _ok(caso)

    if action == "batch_blank_lines_skipped":
        req = [
            "",
            "   \t  ",
            {"id": "a1", "block": _BLOQUE_BLOQUES},
            "   ",
            {"id": "a2", "block": _BLOQUE_POLOS},
            "",
        ]
        res, items = harness.invocar_dibu_batch(dibu_bin, req)
        if panic := _check_no_panic(res, caso):
            return panic
        if res.returncode != 0 or [it.id for it in items] != ["a1", "a2"]:
            return _fail(caso, f"rc={res.returncode}, ids={[it.id for it in items]}")
        return _ok(caso)

    if action == "batch_malformed_json_continues":
        req = [
            {"id": "ok1", "block": _BLOQUE_BLOQUES},
            "{malformed json",
            {"id": "ok2", "block": _BLOQUE_POLOS},
        ]
        res, items = harness.invocar_dibu_batch(dibu_bin, req)
        if panic := _check_no_panic(res, caso):
            return panic
        if res.returncode != 0 or len(items) != 3:
            return _fail(caso, f"rc={res.returncode}, len={len(items)}")
        if not items[0].svg or not items[2].svg or not items[1].error or "JSON inválido" not in items[1].error:
            return _fail(caso, f"Respuestas inesperadas: {items}")
        return _ok(caso)

    if action == "batch_missing_fields_or_wrong_id":
        req = [
            json.dumps({"id": "sin_block"}),
            json.dumps({"id": 123, "block": _BLOQUE_BLOQUES}),
        ]
        res, items = harness.invocar_dibu_batch(dibu_bin, req)
        if panic := _check_no_panic(res, caso):
            return panic
        if res.returncode != 0 or len(items) != 2:
            return _fail(caso, f"rc={res.returncode}, len={len(items)}")
        for it in items:
            if not it.error or "JSON inválido" not in it.error:
                return _fail(caso, f"Esperado JSON inválido, obtuvo {it}")
        return _ok(caso)

    if action == "batch_domain_and_unknown_errors":
        req = [
            {"id": "err_tipo", "block": {"type": "tipo_que_no_existe"}},
            {"id": "err_dom", "block": {"type": "bode"}},
            {"id": "ok_final", "block": _BLOQUE_BLOQUES},
        ]
        res, items = harness.invocar_dibu_batch(dibu_bin, req)
        if panic := _check_no_panic(res, caso):
            return panic
        if res.returncode != 0 or len(items) != 3:
            return _fail(caso, f"rc={res.returncode}, len={len(items)}")
        if items[0].id != "err_tipo" or "no soportado" not in (items[0].error or ""):
            return _fail(caso, f"Item 0 inesperado: {items[0]}")
        if items[1].id != "err_dom" or "falta system" not in (items[1].error or ""):
            return _fail(caso, f"Item 1 inesperado: {items[1]}")
        if items[2].id != "ok_final" or not items[2].svg:
            return _fail(caso, f"Item 2 inesperado: {items[2]}")
        return _ok(caso)

    if action == "batch_determinism_and_flag_rejected":
        req = [{"id": "d1", "block": _BLOQUE_BODE}, {"id": "d2", "block": _BLOQUE_POLOS}]
        r1, _ = harness.invocar_dibu_batch(dibu_bin, req)
        r2, _ = harness.invocar_dibu_batch(dibu_bin, req)
        if r1.stdout != r2.stdout or r1.returncode != 0:
            return _fail(caso, "Falla de determinismo en `batch`")
        r_flag = harness.ejecutar_cli(dibu_bin, ["batch", "--color"])
        if r_flag.returncode != 2:
            return _fail(caso, f"`dibu batch --color` esperaba rc=2, obtuvo rc={r_flag.returncode}")
        return _ok(caso)

    # --- sync-md ------------------------------------------------------------
    if action == "sync_md_dir_creates_hashed":
        cuerpo = "type: bode\nsystem: {k: 10, poles: [0, -1, -10]}\nmargins: true\n"
        (case_dir / "guia.md").write_text(f"# Guía\n\n```estructura\n{cuerpo}```\n", encoding="utf-8")
        res = harness.invocar_dibu_sync_md(dibu_bin, [case_dir])
        nombre_esp = oracle.nombre_svg_oraculo(cuerpo)
        svg_path = case_dir / "diagramas" / nombre_esp
        if res.returncode != 1 or not svg_path.is_file() or len(res.escritos) != 1:
            return _fail(caso, f"rc={res.returncode}, exists={svg_path.is_file()}, escritos={res.escritos}, stderr={res.stderr!r}")
        if "git add" not in res.stderr:
            return _fail(caso, f"Falta aviso git add en stderr: {res.stderr!r}")
        ora = oracle.renderizar_bloque_md_oraculo(cuerpo, tinta=True)
        err = harness.comparar_con_oraculo(svg_path.read_text(encoding="utf-8"), ora.svg or "", tinta_esperada=True)
        if err:
            return _fail(caso, "; ".join(err))
        return _ok(caso)

    if action == "sync_md_idempotent_and_verificar":
        cuerpo = "type: pole_zero\npoles: [-1, -2]\n"
        (case_dir / "guia.md").write_text(f"```estructura\n{cuerpo}```\n", encoding="utf-8")
        r1 = harness.invocar_dibu_sync_md(dibu_bin, [case_dir])
        r2 = harness.invocar_dibu_sync_md(dibu_bin, [case_dir])
        r3 = harness.invocar_dibu_sync_md(dibu_bin, [case_dir], verificar=True)
        if r1.returncode != 1 or r2.returncode != 0 or r3.returncode != 0:
            return _fail(caso, f"rc inesperados: r1={r1.returncode}, r2={r2.returncode}, r3={r3.returncode}")
        if r2.stdout.strip() != "" or r3.stdout.strip() != "":
            return _fail(caso, f"Esperado stdout vacío en corridas idempotentes: r2={r2.stdout!r}, r3={r3.stdout!r}")
        return _ok(caso)

    if action == "sync_md_statics_and_tilde":
        c1 = "type: block_diagram\nforward: [\"G1\"]\n"
        c2 = "type: block_diagram\nforward: [\"G2\"]\n"
        (case_dir / "fences.md").write_text(
            f"~~~statics\n{c1}~~~\n\n```statics\n{c2}```\n",
            encoding="utf-8",
        )
        res = harness.invocar_dibu_sync_md(dibu_bin, [case_dir])
        n1, n2 = oracle.nombre_svg_oraculo(c1), oracle.nombre_svg_oraculo(c2)
        if res.returncode != 1 or not (case_dir / "diagramas" / n1).is_file() or not (case_dir / "diagramas" / n2).is_file():
            return _fail(caso, f"rc={res.returncode}, escritos={res.escritos}, stderr={res.stderr!r}")
        return _ok(caso)

    if action == "sync_md_multiblock_and_dedup":
        c_multi = "- type: pole_zero\n  poles: [-1]\n- type: bode\n  system: {tau: 1}\n"
        c_dup = "  type: block_diagram\n  forward: [\"G\"]\n"
        (case_dir / "a.md").write_text(
            f"```estructura\n{c_multi}```\n\n```estructura\n{c_dup}```\n\n```statics\n{c_dup.strip()}\n```\n",
            encoding="utf-8",
        )
        res = harness.invocar_dibu_sync_md(dibu_bin, [case_dir])
        if res.returncode != 1 or len(res.escritos) != 2:
            return _fail(caso, f"Esperados 2 SVGs (1 apilado + 1 deduplicado), obtuvo escritos={res.escritos}, rc={res.returncode}")
        p_multi = case_dir / "diagramas" / oracle.nombre_svg_oraculo(c_multi)
        ora = oracle.renderizar_bloque_md_oraculo(c_multi, tinta=True)
        err = harness.comparar_con_oraculo(
            p_multi.read_text(encoding="utf-8"),
            ora.svg or "",
            tinta_esperada=True,
            apilado=True,
        )
        if err:
            return _fail(caso, "; ".join(err))
        return _ok(caso)

    if action == "sync_md_color_verificar_mismatch":
        cuerpo = "type: block_diagram\nforward: [\"G(s)\"]\n"
        (case_dir / "c.md").write_text(f"```estructura\n{cuerpo}```\n", encoding="utf-8")
        r1 = harness.invocar_dibu_sync_md(dibu_bin, [case_dir], color=False)
        svg_path = case_dir / "diagramas" / oracle.nombre_svg_oraculo(cuerpo)
        antes = svg_path.read_bytes()
        r2 = harness.invocar_dibu_sync_md(dibu_bin, [case_dir], verificar=True, color=True)
        despues = svg_path.read_bytes()
        if r1.returncode != 1 or r2.returncode != 1 or not r2.desactualizados or antes != despues:
            return _fail(caso, f"r1={r1.returncode}, r2={r2.returncode}, desact={r2.desactualizados}")
        return _ok(caso)

    if action == "sync_md_relative_no_parent":
        cuerpo = "type: block_diagram\nforward: [\"G(s)\"]\n"
        (case_dir / "doc.md").write_text(f"```estructura\n{cuerpo}```\n", encoding="utf-8")
        res = harness.invocar_dibu_sync_md(dibu_bin, ["doc.md"], cwd=case_dir)
        svg_path = case_dir / "diagramas" / oracle.nombre_svg_oraculo(cuerpo)
        if res.returncode == 1 and svg_path.is_file():
            return _ok(caso)
        msg = (
            f"Bug conocido M2 (`parent() == ''` en sync-md): "
            f"`dibu sync-md doc.md` devolvió rc={res.returncode}, creó_svg={svg_path.is_file()}"
        )
        if ctx.solo_implementados and caso.get("known_issue_m2"):
            return _skip(caso, msg)
        return _fail(caso, msg)

    if action == "sync_md_orphan_and_empty_dir":
        cuerpo = "type: pole_zero\npoles: [-1]\n"
        md_file = case_dir / "d.md"
        md_file.write_text(f"```estructura\n{cuerpo}```\n", encoding="utf-8")
        diag_dir = case_dir / "diagramas"
        diag_dir.mkdir()
        huerfano = diag_dir / "estructura-deadbeef1234.svg"
        huerfano.write_text("<svg/>", encoding="utf-8")
        ajeno = diag_dir / "otro.txt"
        ajeno.write_text("conservar", encoding="utf-8")

        r_ver = harness.invocar_dibu_sync_md(dibu_bin, [case_dir], verificar=True)
        if r_ver.returncode != 1 or not r_ver.sobran or not huerfano.is_file():
            return _fail(caso, f"--verificar debía reportar Sobra sin borrar: {r_ver}")

        r_sync = harness.invocar_dibu_sync_md(dibu_bin, [case_dir])
        if r_sync.returncode != 1 or huerfano.is_file() or not ajeno.is_file():
            return _fail(caso, f"Limpieza falló: huerfano={huerfano.is_file()}, ajeno={ajeno.is_file()}")

        # Quitar el bloque y borrar el archivo ajeno: `diagramas/` vacío debe eliminarse.
        ajeno.unlink()
        md_file.write_text("# Sin diagramas\n", encoding="utf-8")
        r_clean = harness.invocar_dibu_sync_md(dibu_bin, [case_dir])
        if r_clean.returncode != 1 or diag_dir.exists():
            return _fail(caso, f"Directorio vacío no fue eliminado: exists={diag_dir.exists()}, rc={r_clean.returncode}")
        return _ok(caso)

    if action == "sync_md_invalid_block":
        c_ok = "type: pole_zero\npoles: [-1]\n"
        (case_dir / "mix.md").write_text(
            f"```estructura\ntype: bode\n```\n\n```estructura\n{c_ok}```\n",
            encoding="utf-8",
        )
        res = harness.invocar_dibu_sync_md(dibu_bin, [case_dir])
        if res.returncode != 1 or not res.errores_stderr:
            return _fail(caso, f"Esperado rc=1 + Error en stderr: rc={res.returncode}, stderr={res.stderr!r}")
        if harness.tiene_panic_o_traceback(res.stderr):
            return _fail(caso, f"Panic detectado: {res.stderr!r}")
        svg_ok = case_dir / "diagramas" / oracle.nombre_svg_oraculo(c_ok)
        if not svg_ok.is_file():
            return _fail(caso, "El bloque válido en el mismo .md no se sincronizó")
        return _ok(caso)

    if action == "sync_md_unclosed_and_crlf":
        cuerpo = "type: block_diagram\nforward: [\"G1\"]\n"
        dir_lf = case_dir / "lf"
        dir_crlf = case_dir / "crlf"
        dir_lf.mkdir()
        dir_crlf.mkdir()
        (dir_lf / "a.md").write_text(
            f"```estructura\n{cuerpo}```\n\n```estructura\ntype: bode\n",
            encoding="utf-8",
        )
        (dir_crlf / "a.md").write_text(
            f"```estructura\r\n{cuerpo.replace(chr(10), chr(13) + chr(10))}```\r\n",
            encoding="utf-8",
        )
        r_lf = harness.invocar_dibu_sync_md(dibu_bin, [dir_lf])
        r_crlf = harness.invocar_dibu_sync_md(dibu_bin, [dir_crlf])
        if r_lf.returncode != 1 or r_crlf.returncode != 1:
            return _fail(caso, f"rc inesperados: lf={r_lf.returncode}, crlf={r_crlf.returncode}")
        if len(r_lf.escritos) != 1 or len(r_crlf.escritos) != 1:
            return _fail(caso, f"Fence abierto no se ignoró o falló CRLF: lf={r_lf.escritos}, crlf={r_crlf.escritos}")
        p_lf = Path(r_lf.escritos[0])
        p_crlf = Path(r_crlf.escritos[0])
        if p_lf.name != p_crlf.name or p_lf.read_bytes() != p_crlf.read_bytes():
            return _fail(caso, f"CRLF divergió de LF: {p_lf.name} vs {p_crlf.name}")
        return _ok(caso)

    if action == "sync_md_no_args_and_non_md":
        r_no_args = harness.ejecutar_cli(dibu_bin, ["sync-md"])
        if r_no_args.returncode != 2:
            return _fail(caso, f"`dibu sync-md` sin args esperaba rc=2, obtuvo rc={r_no_args.returncode}")
        txt_file = case_dir / "notas.txt"
        txt_file.write_text("```estructura\ntype: pole_zero\n```\n", encoding="utf-8")
        r_txt = harness.invocar_dibu_sync_md(dibu_bin, [txt_file])
        if r_txt.returncode != 0 or (case_dir / "diagramas").exists():
            return _fail(caso, f"Archivo .txt no debía crear diagramas/: rc={r_txt.returncode}")
        return _ok(caso)

    return _fail(caso, f"Acción CLI desconocida en test_cli.py: {action!r}")


def run_suite(ctx: ContextoE2E) -> list[ResultadoCaso]:
    """Ejecuta todos los casos de `cases/cli.yaml` filtrados por `ctx.tier`."""
    casos = harness.cargar_casos_yaml(ctx.cases_dir / "cli.yaml")
    if ctx.tier != "all":
        tier_num = int(ctx.tier)
        casos = [c for c in casos if int(c["tier"]) == tier_num]
    return [_ejecutar_caso_cli(c, ctx) for c in casos]
