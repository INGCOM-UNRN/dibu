"""Suite E2E Tier 4: Escenarios realistas end-to-end (Markdown -> sync-md -> render/batch -> HTML -> Typst/PDF)."""

from __future__ import annotations

import sys

sys.dont_write_bytecode = True

import json
import re
import shutil
from pathlib import Path
from typing import Any

import yaml

from e2e import harness, oracle
from e2e.harness import ContextoE2E, ResultadoCaso

_FENCE_RE = re.compile(
    r"^(?P<fence>`{3,}|~{3,})\s*(?P<lang>estructura|statics)\s*\r?\n(?P<body>.*?)\r?\n(?P=fence)\s*$",
    re.MULTILINE | re.DOTALL,
)

_ESCENARIOS: tuple[dict[str, Any], ...] = (
    {
        "id": "tier4_examen_control",
        "archivo": "examen_control.md",
        "feature": "cli:sync-md",
        "color": True,
        "min_tipos": 9,
        "requiere_todos_44": False,
        "resiliencia_batch": False,
    },
    {
        "id": "tier4_examen_quimica",
        "archivo": "examen_quimica.md",
        "feature": "cli:sync-md",
        "color": False,
        "min_tipos": 10,
        "requiere_todos_44": False,
        "resiliencia_batch": False,
    },
    {
        "id": "tier4_examen_algoritmos",
        "archivo": "examen_algoritmos.md",
        "feature": "cli:sync-md",
        "color": False,
        "min_tipos": 14,
        "requiere_todos_44": False,
        "resiliencia_batch": False,
    },
    {
        "id": "tier4_guia_integradora_mixta",
        "archivo": "guia_integradora_mixta.md",
        "feature": "cli:sync-md",
        "color": False,
        "min_tipos": 44,
        "requiere_todos_44": True,
        "resiliencia_batch": False,
    },
    {
        "id": "tier4_parcial_errores_y_variantes",
        "archivo": "parcial_errores_y_variantes.md",
        "feature": "cli:sync-md",
        "color": False,
        "min_tipos": 4,
        "requiere_todos_44": False,
        "resiliencia_batch": True,
    },
)


def _extraer_fences_y_bloques(md_text: str) -> tuple[list[tuple[str, list[dict[str, Any]]]], list[dict[str, Any]]]:
    fences: list[tuple[str, list[dict[str, Any]]]] = []
    bloques_planos: list[dict[str, Any]] = []
    for m in _FENCE_RE.finditer(md_text):
        cuerpo = m.group("body")
        data = yaml.safe_load(cuerpo)
        if isinstance(data, dict):
            bloques = [data]
        elif isinstance(data, list):
            bloques = [dict(x) for x in data]
        else:
            raise ValueError(f"Bloque inesperado en fence: {type(data).__name__}")
        norm_bloques = [harness.normalizar_bloque_json(b) for b in bloques]
        fences.append((cuerpo, norm_bloques))
        bloques_planos.extend(norm_bloques)
    return fences, bloques_planos


def _tinta_fence(bloques: list[dict[str, Any]], tinta_cli: bool) -> bool | None:
    efectivas = {
        b["inkSaver"] if isinstance(b.get("inkSaver"), bool) else tinta_cli
        for b in bloques
    }
    if len(efectivas) == 1:
        return next(iter(efectivas))
    return None


def _pipeline_documental(md_text: str, out_pdf: Path) -> tuple[list[str], dict[str, Any]]:
    errores: list[str] = []
    detalles: dict[str, Any] = {}

    html = oracle.markdown_a_html_oraculo(md_text)
    n_svgs_html = html.count("<svg")
    if n_svgs_html == 0:
        errores.append("`markdown_a_html_oraculo` no embebió ningún <svg")
        return errores, detalles
    detalles["html_svgs"] = n_svgs_html

    typst_src = oracle.html_a_typst_oraculo(html)
    if not typst_src.strip():
        errores.append("`html_a_typst_oraculo` devolvió fuente Typst vacía")
        return errores, detalles
    detalles["typst_chars"] = len(typst_src)

    typst_bin = shutil.which("typst")
    if typst_bin:
        detalles["typst_bin"] = typst_bin
        pdf_path = oracle.compilar_typst_oraculo(typst_src, out_pdf)
        if not pdf_path.is_file():
            errores.append(f"Typst no creó el PDF esperado: {pdf_path}")
            return errores, detalles
        raw = pdf_path.read_bytes()
        if len(raw) == 0 or not raw.startswith(b"%PDF-"):
            errores.append(f"PDF inválido o vacío ({len(raw)} bytes, header={raw[:8]!r})")
            return errores, detalles
        detalles["pdf_bytes"] = len(raw)
    else:
        detalles["typst_fallback"] = (
            "Binario `typst` no instalado; validado HTML + fuente Typst sin compilar PDF"
        )

    return errores, detalles


def _ejecutar_escenario(meta: dict[str, Any], ctx: ContextoE2E) -> ResultadoCaso:
    cid = str(meta["id"])
    feat = str(meta["feature"])
    archivo = str(meta["archivo"])
    color = bool(meta["color"])
    tinta_cli = not color
    src_md = ctx.cases_dir / "tier4_escenarios" / archivo

    if not src_md.is_file():
        return ResultadoCaso(
            case_id=cid,
            feature=feat,
            tier=4,
            status="FAIL",
            mensaje=f"No existe el archivo de escenario {src_md}",
        )

    md_text = src_md.read_text(encoding="utf-8")
    fences, bloques = _extraer_fences_y_bloques(md_text)
    if not fences or not bloques:
        return ResultadoCaso(
            case_id=cid,
            feature=feat,
            tier=4,
            status="FAIL",
            mensaje=f"Escenario {archivo} no contiene fences válidos",
        )

    tipos_usados = {str(b.get("type", "")) for b in bloques}
    if len(tipos_usados) < int(meta["min_tipos"]):
        return ResultadoCaso(
            case_id=cid,
            feature=feat,
            tier=4,
            status="FAIL",
            mensaje=f"{archivo} cubre {len(tipos_usados)} tipos < mínimo {meta['min_tipos']}",
        )
    if meta["requiere_todos_44"]:
        faltan_44 = sorted(set(oracle.TODOS_LOS_TIPOS) - tipos_usados)
        if faltan_44:
            return ResultadoCaso(
                case_id=cid,
                feature=feat,
                tier=4,
                status="FAIL",
                mensaje=f"{archivo} omite tipos del catálogo 44: {faltan_44}",
            )

    case_dir = ctx.work_dir / cid
    case_dir.mkdir(parents=True, exist_ok=True)
    md_aislado = case_dir / archivo
    md_aislado.write_text(md_text, encoding="utf-8")

    # Modo --oracle-only: sync-md oráculo + render/batch simulado + pipeline HTML -> Typst/PDF
    if ctx.oracle_only:
        sync1 = oracle.sincronizar_svgs_oraculo(case_dir, tinta=tinta_cli, escribir=True)
        if sync1.errores or len(sync1.escritos) != len(fences):
            return ResultadoCaso(
                case_id=cid,
                feature=feat,
                tier=4,
                status="FAIL",
                mensaje=f"Oráculo sync falló: escritos={len(sync1.escritos)}/{len(fences)}, errores={sync1.errores}",
            )
        sync2 = oracle.sincronizar_svgs_oraculo(case_dir, tinta=tinta_cli, escribir=False)
        if sync2.escritos or sync2.borrados or sync2.errores:
            return ResultadoCaso(
                case_id=cid,
                feature=feat,
                tier=4,
                status="FAIL",
                mensaje=f"Oráculo sync no idempotente: {sync2}",
            )

        for cuerpo, b_lista in fences:
            svg_file = case_dir / "diagramas" / oracle.nombre_svg_oraculo(cuerpo)
            if not svg_file.is_file():
                return ResultadoCaso(
                    case_id=cid,
                    feature=feat,
                    tier=4,
                    status="FAIL",
                    mensaje=f"Falta SVG esperado en oráculo: {svg_file.name}",
                )
            err = harness.verificar_svg(
                svg_file.read_text(encoding="utf-8"),
                tinta_esperada=_tinta_fence(b_lista, tinta_cli),
                apilado=len(b_lista) > 1,
            )
            if err:
                return ResultadoCaso(
                    case_id=cid,
                    feature=feat,
                    tier=4,
                    status="FAIL",
                    mensaje=f"{svg_file.name}: {'; '.join(err)}",
                )

        for idx, blk in enumerate(bloques):
            r_blk = oracle.renderizar_oraculo(blk, tinta=tinta_cli)
            if not r_blk.ok or not r_blk.svg:
                return ResultadoCaso(
                    case_id=cid,
                    feature=feat,
                    tier=4,
                    status="FAIL",
                    mensaje=f"Bloque {idx} falló en oráculo: {r_blk.error}",
                )

        if meta["resiliencia_batch"]:
            r_err = oracle.renderizar_oraculo({"type": "bode"}, tinta=tinta_cli)
            if r_err.ok:
                return ResultadoCaso(
                    case_id=cid,
                    feature=feat,
                    tier=4,
                    status="FAIL",
                    mensaje="Oráculo aceptó bloque inválido en prueba de resiliencia",
                )

        err_doc, det_doc = _pipeline_documental(md_text, case_dir / f"{cid}.pdf")
        if err_doc:
            return ResultadoCaso(
                case_id=cid,
                feature=feat,
                tier=4,
                status="FAIL",
                mensaje="; ".join(err_doc),
                detalles=det_doc,
            )
        return ResultadoCaso(
            case_id=cid,
            feature=feat,
            tier=4,
            status="PASS",
            mensaje=f"Oráculo OK ({len(fences)} fences, {len(bloques)} bloques, {len(tipos_usados)} tipos)",
            detalles=det_doc,
        )

    # Modo binario `dibu`
    faltantes = sorted(tipos_usados - ctx.tipos_implementados)
    if faltantes:
        if ctx.solo_implementados:
            return ResultadoCaso(
                case_id=cid,
                feature=feat,
                tier=4,
                status="SKIP",
                mensaje=f"Escenario requiere tipos aún no implementados en `dibu`: {faltantes[:8]}"
                + ("..." if len(faltantes) > 8 else ""),
            )
        return ResultadoCaso(
            case_id=cid,
            feature=feat,
            tier=4,
            status="FAIL",
            mensaje=f"Faltan tipos en `dibu list-types`: {faltantes[:10]}",
        )

    assert ctx.dibu_bin is not None
    dibu_bin = ctx.dibu_bin

    # 1. `dibu sync-md` + paridad vs oráculo + `--verificar` idempotente
    r_sync = harness.invocar_dibu_sync_md(dibu_bin, [case_dir], color=color)
    if harness.tiene_panic_o_traceback(r_sync.stderr):
        return ResultadoCaso(
            case_id=cid,
            feature=feat,
            tier=4,
            status="FAIL",
            mensaje=f"Panic en `dibu sync-md`: {r_sync.stderr!r}",
        )
    if r_sync.returncode != 1 or r_sync.errores_stderr or len(r_sync.escritos) != len(fences):
        return ResultadoCaso(
            case_id=cid,
            feature=feat,
            tier=4,
            status="FAIL",
            mensaje=(
                f"`dibu sync-md` inesperado: rc={r_sync.returncode}, "
                f"escritos={len(r_sync.escritos)}/{len(fences)}, stderr={r_sync.stderr!r}"
            ),
        )

    r_ver = harness.invocar_dibu_sync_md(dibu_bin, [case_dir], verificar=True, color=color)
    if r_ver.returncode != 0 or r_ver.stdout.strip() != "":
        return ResultadoCaso(
            case_id=cid,
            feature=feat,
            tier=4,
            status="FAIL",
            mensaje=f"`dibu sync-md --verificar` no idempotente: rc={r_ver.returncode}, stdout={r_ver.stdout!r}",
        )

    for cuerpo, b_lista in fences:
        nombre_svg = oracle.nombre_svg_oraculo(cuerpo)
        svg_path = case_dir / "diagramas" / nombre_svg
        if not svg_path.is_file():
            return ResultadoCaso(
                case_id=cid,
                feature=feat,
                tier=4,
                status="FAIL",
                mensaje=f"Falta {nombre_svg} en diagramas/",
            )
        ora_md = oracle.renderizar_bloque_md_oraculo(cuerpo, tinta=tinta_cli)
        if not ora_md.ok or not ora_md.svg:
            return ResultadoCaso(
                case_id=cid,
                feature=feat,
                tier=4,
                status="FAIL",
                mensaje=f"Oráculo bloque md falló en {nombre_svg}: {ora_md.error}",
            )
        err_cmp = harness.comparar_con_oraculo(
            svg_path.read_text(encoding="utf-8"),
            ora_md.svg,
            tinta_esperada=_tinta_fence(b_lista, tinta_cli),
            apilado=len(b_lista) > 1,
        )
        if err_cmp:
            return ResultadoCaso(
                case_id=cid,
                feature=feat,
                tier=4,
                status="FAIL",
                mensaje=f"{nombre_svg}: {'; '.join(err_cmp)}",
            )

    # 2. `dibu render` y `dibu batch` sobre bloques extraídos + determinismo + resiliencia
    lote_bloques = [
        {"id": f"{cid}_{idx}", **blk} for idx, blk in enumerate(bloques)
    ]
    json_in = case_dir / "bloques_extraidos.json"
    json_in.write_text(json.dumps(lote_bloques, ensure_ascii=False), encoding="utf-8")

    out_r1 = case_dir / "render_out_1"
    out_r2 = case_dir / "render_out_2"
    rr1 = harness.invocar_dibu_render(dibu_bin, json_in, output_dir=out_r1, color=color)
    rr2 = harness.invocar_dibu_render(dibu_bin, json_in, output_dir=out_r2, color=color)
    if rr1.returncode != 0 or rr2.returncode != 0 or harness.tiene_panic_o_traceback(rr1.stderr):
        return ResultadoCaso(
            case_id=cid,
            feature=feat,
            tier=4,
            status="FAIL",
            mensaje=f"`dibu render` falló: r1={rr1.returncode}, r2={rr2.returncode}, stderr={rr1.stderr!r}",
        )
    for b in lote_bloques:
        p1 = out_r1 / f"{b['id']}.svg"
        p2 = out_r2 / f"{b['id']}.svg"
        if not p1.is_file() or not p2.is_file():
            return ResultadoCaso(
                case_id=cid,
                feature=feat,
                tier=4,
                status="FAIL",
                mensaje=f"`dibu render` no generó {b['id']}.svg",
            )
        if p1.read_bytes() != p2.read_bytes():
            return ResultadoCaso(
                case_id=cid,
                feature=feat,
                tier=4,
                status="FAIL",
                mensaje=f"`dibu render` no determinista en {b['id']}.svg",
            )

    batch_items: list[dict[str, Any]] = [
        {"id": b["id"], "block": blk, "tinta": tinta_cli}
        for b, blk in zip(lote_bloques, bloques)
    ]
    if meta["resiliencia_batch"]:
        batch_items.insert(
            len(batch_items) // 2,
            {"id": f"{cid}_invalido", "block": {"type": "bode"}, "tinta": tinta_cli},
        )

    rb1, out_b1 = harness.invocar_dibu_batch(dibu_bin, batch_items)
    rb2, _ = harness.invocar_dibu_batch(dibu_bin, batch_items)
    if rb1.returncode != 0 or harness.tiene_panic_o_traceback(rb1.stderr):
        return ResultadoCaso(
            case_id=cid,
            feature=feat,
            tier=4,
            status="FAIL",
            mensaje=f"`dibu batch` falló: rc={rb1.returncode}, stderr={rb1.stderr!r}",
        )
    if rb1.stdout != rb2.stdout:
        return ResultadoCaso(
            case_id=cid,
            feature=feat,
            tier=4,
            status="FAIL",
            mensaje="`dibu batch` no determinista byte-a-byte",
        )
    if len(out_b1) != len(batch_items):
        return ResultadoCaso(
            case_id=cid,
            feature=feat,
            tier=4,
            status="FAIL",
            mensaje=f"`dibu batch` devolvió {len(out_b1)} ítems de {len(batch_items)}",
        )
    for req_it, resp_it in zip(batch_items, out_b1):
        if req_it["id"].endswith("_invalido"):
            if resp_it.svg is not None or not resp_it.error:
                return ResultadoCaso(
                    case_id=cid,
                    feature=feat,
                    tier=4,
                    status="FAIL",
                    mensaje=f"Ítem inválido en batch no devolvió error controlado: {resp_it}",
                )
        else:
            if not resp_it.svg:
                return ResultadoCaso(
                    case_id=cid,
                    feature=feat,
                    tier=4,
                    status="FAIL",
                    mensaje=f"Ítem válido {resp_it.id} falló en batch: {resp_it.error}",
                )

    # 3. Pipeline documental HTML -> Typst -> PDF
    err_doc, det_doc = _pipeline_documental(md_text, case_dir / f"{cid}.pdf")
    if err_doc:
        return ResultadoCaso(
            case_id=cid,
            feature=feat,
            tier=4,
            status="FAIL",
            mensaje="; ".join(err_doc),
            detalles=det_doc,
        )

    return ResultadoCaso(
        case_id=cid,
        feature=feat,
        tier=4,
        status="PASS",
        mensaje=f"E2E completo OK ({len(fences)} fences, {len(bloques)} bloques, {len(tipos_usados)} tipos)",
        detalles=det_doc,
    )


def run_suite(ctx: ContextoE2E) -> list[ResultadoCaso]:
    """Ejecuta los 5 escenarios realistas de Tier 4 bajo `cases/tier4_escenarios/`."""
    if ctx.tier not in ("all", "4"):
        return []
    return [_ejecutar_escenario(meta, ctx) for meta in _ESCENARIOS]
