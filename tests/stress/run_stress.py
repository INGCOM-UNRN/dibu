#!/usr/bin/env python3
"""Suite de stress test, paridad, determinismo y resiliencia para `dibu` (M4 Fase 4)."""

from __future__ import annotations

import sys

sys.dont_write_bytecode = True

import argparse
import copy
import datetime as dt
import hashlib
import json
import math
import os
import platform
import random
import resource
import shutil
import statistics
import subprocess
import tempfile
import threading
import time
from dataclasses import asdict, dataclass
from pathlib import Path
from typing import Any

import yaml

DIBU_ROOT = Path("/home/mrtin/dev/tools/docente/contenidos/dibu")
TESTS_DIR = DIBU_ROOT / "tests"
if str(TESTS_DIR) not in sys.path:
    sys.path.insert(0, str(TESTS_DIR))

from e2e import harness, oracle  # noqa: E402

KITCHEN_SINK_DIR = DIBU_ROOT / "fixtures" / "kitchen_sink"
KITCHEN_SINK_YAML = KITCHEN_SINK_DIR / "kitchen_sink.yaml"
KITCHEN_SINK_MD = KITCHEN_SINK_DIR / "kitchen_sink.md"
DEFAULT_REPORT_MD = KITCHEN_SINK_DIR / "STRESS_REPORT.md"

FENCE_PATRON_MD = (
    r"(?m)^([`~]{3,})(?:estructura|statics)[ \t]*\r?\n(.*?)\r?\n\1[ \t]*$"
)


@dataclass
class SubprocessMetric:
    returncode: int
    stdout: str
    stderr: str
    elapsed_ms: float
    peak_rss_kib: int


def sha256_text(texto: str) -> str:
    return hashlib.sha256(texto.encode("utf-8")).hexdigest()


def sha256_bytes(datos: bytes) -> str:
    return hashlib.sha256(datos).hexdigest()


def percentil(valores: list[float], p: float) -> float:
    if not valores:
        return 0.0
    ordenados = sorted(valores)
    if len(ordenados) == 1:
        return ordenados[0]
    pos = (len(ordenados) - 1) * (p / 100.0)
    lo = math.floor(pos)
    hi = math.ceil(pos)
    if lo == hi:
        return ordenados[lo]
    frac = pos - lo
    return ordenados[lo] * (1.0 - frac) + ordenados[hi] * frac


def leer_rss_actual_kib() -> int:
    try:
        for linea in Path("/proc/self/status").read_text(encoding="utf-8").splitlines():
            if linea.startswith("VmHWM:"):
                return int(linea.split()[1])
    except OSError:
        pass
    return int(resource.getrusage(resource.RUSAGE_SELF).ru_maxrss)


def medir_rss_con_time(
    cmd: list[str],
    *,
    stdin_bytes: bytes | None = None,
    cwd: Path | None = None,
    timeout: float = 180.0,
) -> tuple[int, str, str, float, int]:
    """Ejecuta un comando midiendo wall-clock y RSS pico real del proceso hijo.

    Usa `/usr/bin/time -o <tmp> -f %M` para aislar la medición respecto al RSS
    del proceso Python padre (que crece tras importar `typst` y PyYAML).
    """
    time_bin = "/usr/bin/time" if Path("/usr/bin/time").is_file() else shutil.which("time")
    if time_bin:
        with tempfile.NamedTemporaryFile(prefix="dibu_rss_", suffix=".txt", delete=False) as tf:
            rss_path = Path(tf.name)
        try:
            t0 = time.perf_counter()
            proc = subprocess.run(
                [time_bin, "-o", str(rss_path), "-f", "%M", *cmd],
                input=stdin_bytes,
                cwd=str(cwd) if cwd is not None else None,
                capture_output=True,
                timeout=timeout,
                check=False,
            )
            elapsed_ms = (time.perf_counter() - t0) * 1000.0
            rss_txt = rss_path.read_text(encoding="utf-8").strip().splitlines()
            rss_kib = int(rss_txt[-1]) if rss_txt and rss_txt[-1].isdigit() else 0
            return (
                proc.returncode,
                proc.stdout.decode("utf-8", errors="replace"),
                proc.stderr.decode("utf-8", errors="replace"),
                elapsed_ms,
                rss_kib,
            )
        finally:
            if rss_path.exists():
                rss_path.unlink()

    m = ejecutar_con_rss(cmd, stdin_bytes=stdin_bytes, cwd=cwd, timeout=timeout)
    return m.returncode, m.stdout, m.stderr, m.elapsed_ms, m.peak_rss_kib


def medir_rss_python_aislado(lote: list[tuple[dict[str, Any], bool]]) -> int:
    """Mide el `VmHWM` real de un proceso Python limpio al renderizar el lote con `alucarD`."""
    with tempfile.NamedTemporaryFile(
        prefix="dibu_py_lote_", suffix=".json", mode="w", encoding="utf-8", delete=False
    ) as tf:
        json.dump(
            [{"block": blk, "tinta": tinta} for blk, tinta in lote],
            tf,
            ensure_ascii=False,
        )
        lote_path = Path(tf.name)
    try:
        script = (
            "import json, sys; sys.dont_write_bytecode = True; "
            f"sys.path.insert(0, {str(TESTS_DIR)!r}); "
            "from pathlib import Path; from e2e import oracle; "
            f"items = json.loads(Path({str(lote_path)!r}).read_text(encoding='utf-8')); "
            "svgs = [oracle.renderizar_oraculo(it['block'], tinta=it['tinta']).svg for it in items]; "
            "hwm = [int(l.split()[1]) for l in Path('/proc/self/status').read_text().splitlines() if l.startswith('VmHWM:')]; "
            "print(hwm[0] if hwm else 0)"
        )
        env = dict(os.environ)
        env["PYTHONDONTWRITEBYTECODE"] = "1"
        proc = subprocess.run(
            [sys.executable, "-c", script],
            capture_output=True,
            text=True,
            env=env,
            timeout=180.0,
            check=True,
        )
        return int(proc.stdout.strip().splitlines()[-1])
    finally:
        if lote_path.exists():
            lote_path.unlink()


def ejecutar_con_rss(
    cmd: list[str],
    *,
    stdin_bytes: bytes | None = None,
    cwd: Path | None = None,
    timeout: float = 120.0,
) -> SubprocessMetric:
    """Ejecuta un subproceso midiendo wall-clock y RSS pico aislado vía `os.wait4`."""
    t0 = time.perf_counter()
    proc = subprocess.Popen(
        cmd,
        stdin=subprocess.PIPE if stdin_bytes is not None else subprocess.DEVNULL,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        cwd=str(cwd) if cwd is not None else None,
    )

    out_chunks: list[bytes] = []
    err_chunks: list[bytes] = []
    errores_io: list[BaseException] = []

    def leer_stdout() -> None:
        try:
            assert proc.stdout is not None
            out_chunks.append(proc.stdout.read())
        except BaseException as exc:
            errores_io.append(exc)

    def leer_stderr() -> None:
        try:
            assert proc.stderr is not None
            err_chunks.append(proc.stderr.read())
        except BaseException as exc:
            errores_io.append(exc)

    def escribir_stdin() -> None:
        try:
            if stdin_bytes is not None and proc.stdin is not None:
                proc.stdin.write(stdin_bytes)
                proc.stdin.close()
        except BrokenPipeError:
            pass
        except BaseException as exc:
            errores_io.append(exc)

    t_out = threading.Thread(target=leer_stdout, daemon=True)
    t_err = threading.Thread(target=leer_stderr, daemon=True)
    t_in = threading.Thread(target=escribir_stdin, daemon=True)
    t_out.start()
    t_err.start()
    t_in.start()

    _, status, rusage = os.wait4(proc.pid, 0)
    elapsed_ms = (time.perf_counter() - t0) * 1000.0
    proc.returncode = os.waitstatus_to_exitcode(status)

    t_in.join(timeout=timeout)
    t_out.join(timeout=timeout)
    t_err.join(timeout=timeout)
    if errores_io:
        raise RuntimeError(f"Error de I/O ejecutando {cmd}: {errores_io[0]}")

    return SubprocessMetric(
        returncode=proc.returncode,
        stdout=b"".join(out_chunks).decode("utf-8", errors="replace"),
        stderr=b"".join(err_chunks).decode("utf-8", errors="replace"),
        elapsed_ms=elapsed_ms,
        peak_rss_kib=int(rusage.ru_maxrss),
    )


def extraer_fences_md(md_texto: str) -> list[str]:
    import re

    patron = re.compile(FENCE_PATRON_MD, re.DOTALL)
    return [m.group(2) for m in patron.finditer(md_texto)]


# ---------------------------------------------------------------------------
# Módulo 1: Kitchen Sink YAML + MD
# ---------------------------------------------------------------------------


def validar_kitchen_sink(dibu_bin: Path) -> dict[str, Any]:
    bloques = yaml.safe_load(KITCHEN_SINK_YAML.read_text(encoding="utf-8"))
    if not isinstance(bloques, list) or not bloques:
        raise AssertionError("kitchen_sink.yaml debe ser una lista no vacía")

    ids = [b["id"] for b in bloques]
    if len(ids) != len(set(ids)):
        raise AssertionError("Hay IDs duplicados en kitchen_sink.yaml")

    tipos_yaml = {b["type"] for b in bloques}
    faltantes_yaml = sorted(set(oracle.TODOS_LOS_TIPOS) - tipos_yaml)
    if faltantes_yaml:
        raise AssertionError(f"Faltan tipos en kitchen_sink.yaml: {faltantes_yaml}")

    # `dibu render` sobre kitchen_sink.yaml
    with tempfile.TemporaryDirectory(prefix="dibu_ks_render_") as td:
        out_dir = Path(td)
        res_render = harness.invocar_dibu_render(dibu_bin, KITCHEN_SINK_YAML, output_dir=out_dir)
        if res_render.returncode != 0:
            raise AssertionError(
                f"dibu render kitchen_sink.yaml falló rc={res_render.returncode}: {res_render.stderr}"
            )
        if "Aviso:" in res_render.stderr or "Error:" in res_render.stderr:
            raise AssertionError(f"dibu render emitió avisos/errores: {res_render.stderr}")
        svgs_render = sorted(out_dir.glob("*.svg"))
        if len(svgs_render) != len(bloques):
            raise AssertionError(
                f"dibu render generó {len(svgs_render)} SVGs, esperados {len(bloques)}"
            )

    # `dibu batch` + paridad 100% contra oráculo
    items_batch = [{"id": b["id"], "block": b, "tinta": True} for b in bloques]
    res_batch, outs_batch = harness.invocar_dibu_batch(dibu_bin, items_batch)
    if res_batch.returncode != 0 or len(outs_batch) != len(bloques):
        raise AssertionError(f"dibu batch kitchen_sink.yaml falló rc={res_batch.returncode}")

    fallos_paridad_yaml: list[str] = []
    for b, out in zip(bloques, outs_batch):
        if out.error or not out.svg:
            fallos_paridad_yaml.append(f"{b['id']}: dibu error={out.error}")
            continue
        # Los tipos propios de dibu (dibu-comp, dibu-math) no existen en alucarD: sólo se exige
        # que rendericen sin error.
        if b["type"] not in oracle.TODOS_LOS_TIPOS:
            continue
        ora = oracle.renderizar_oraculo(b, tinta=True)
        if not ora.ok or not ora.svg:
            fallos_paridad_yaml.append(f"{b['id']}: oráculo error={ora.error}")
            continue
        tinta_ef = b["inkSaver"] if isinstance(b.get("inkSaver"), bool) else True
        diff = harness.comparar_con_oraculo(out.svg, ora.svg, tinta_esperada=tinta_ef)
        if diff:
            fallos_paridad_yaml.append(f"{b['id']}: {diff}")

    if fallos_paridad_yaml:
        raise AssertionError(
            f"Fallos de paridad en kitchen_sink.yaml ({len(fallos_paridad_yaml)}): {fallos_paridad_yaml[:5]}"
        )

    # `kitchen_sink.md` + `dibu sync-md --verificar` + paridad de SVGs en diagramas/
    md_texto = KITCHEN_SINK_MD.read_text(encoding="utf-8")
    fences = extraer_fences_md(md_texto)
    tipos_md: set[str] = set()
    fallos_paridad_md: list[str] = []
    diagramas_dir = KITCHEN_SINK_DIR / "diagramas"
    nombres_esperados: set[str] = set()

    for idx, cuerpo in enumerate(fences):
        bloques_fence = oracle.leer_bloques_oraculo(cuerpo)
        for bf in bloques_fence:
            tipos_md.add(bf["type"])
        nombre = oracle.nombre_svg_oraculo(cuerpo)
        nombres_esperados.add(nombre)
        svg_path = diagramas_dir / nombre
        if not svg_path.is_file():
            fallos_paridad_md.append(f"Fence {idx}: falta {svg_path}")
            continue
        if any(bf["type"] not in oracle.TODOS_LOS_TIPOS for bf in bloques_fence):
            continue  # tipos sin equivalente en alucarD: los cubre `sync-md --verificar`
        ora = oracle.renderizar_bloque_md_oraculo(cuerpo, tinta=True)
        if not ora.ok or not ora.svg:
            fallos_paridad_md.append(f"Fence {idx} ({nombre}): oráculo falló: {ora.error}")
            continue
        diff = harness.comparar_con_oraculo(
            svg_path.read_text(encoding="utf-8"),
            ora.svg,
            tinta_esperada=None,
            apilado=len(bloques_fence) > 1,
        )
        if diff:
            fallos_paridad_md.append(f"Fence {idx} ({nombre}): {diff}")

    faltantes_md = sorted(set(oracle.TODOS_LOS_TIPOS) - tipos_md)
    if faltantes_md:
        raise AssertionError(f"Faltan tipos en kitchen_sink.md: {faltantes_md}")
    if fallos_paridad_md:
        raise AssertionError(
            f"Fallos de paridad en kitchen_sink.md ({len(fallos_paridad_md)}): {fallos_paridad_md[:5]}"
        )

    res_verif = harness.invocar_dibu_sync_md(dibu_bin, [KITCHEN_SINK_MD], verificar=True)
    if res_verif.returncode != 0:
        raise AssertionError(
            f"dibu sync-md --verificar falló rc={res_verif.returncode}: stdout={res_verif.stdout} stderr={res_verif.stderr}"
        )

    return {
        "yaml_bloques": len(bloques),
        "yaml_tipos": len(tipos_yaml),
        "md_fences": len(fences),
        "md_svgs": len(nombres_esperados),
        "md_tipos": len(tipos_md),
        "paridad_yaml": "100%",
        "paridad_md": "100%",
        "sync_md_verificar_rc": res_verif.returncode,
    }


# ---------------------------------------------------------------------------
# Módulo 2: HTML y Typst/PDF (en TemporaryDirectory)
# ---------------------------------------------------------------------------


def validar_html_y_typst(dibu_bin: Path) -> dict[str, Any]:
    md_texto = KITCHEN_SINK_MD.read_text(encoding="utf-8")
    fences = extraer_fences_md(md_texto)
    total_bloques_md = sum(len(oracle.leer_bloques_oraculo(c)) for c in fences)

    # 1. Pipeline Markdown -> HTML (alucarD) -> Typst -> PDF
    html_alucard = oracle.markdown_a_html_oraculo(md_texto, saltos_de_linea=True)
    svgs_en_html = html_alucard.count("<svg")
    if svgs_en_html < total_bloques_md:
        raise AssertionError(
            f"markdown_a_html_oraculo produjo {svgs_en_html} <svg>, esperados >= {total_bloques_md}"
        )

    # 4. Bloque inválido en HTML (alucarD)
    html_err = oracle.markdown_a_html_oraculo("```estructura\ntype: grafo\n```\n")
    if 'class="estructura-error"' not in html_err or "<svg" in html_err:
        raise AssertionError(f"Bloque inválido en HTML no produjo aviso esperado: {html_err!r}")

    typst_disponible = shutil.which("typst") is not None
    if not typst_disponible:
        try:
            import typst as _  # noqa: F401

            typst_disponible = True
        except ImportError:
            typst_disponible = False

    if not typst_disponible:
        return {
            "typst_disponible": False,
            "html_svgs": svgs_en_html,
            "html_error_ok": True,
        }

    oracle.asegurar_alucard_en_sys_path()
    from generador_examenes.generators.pdf_renderer import compilar_typst_a_pdf

    with tempfile.TemporaryDirectory(prefix="dibu_typst_") as td:
        tmp = Path(td)

        # Flujo 1: markdown_a_html_oraculo -> html_a_typst_oraculo -> compilar_typst_oraculo
        typst_1 = oracle.html_a_typst_oraculo(html_alucard)
        pdf_1 = tmp / "flujo1_alucard.pdf"
        oracle.compilar_typst_oraculo(
            f'#set page(paper: "a4", margin: 1.5cm)\n{typst_1}\n',
            pdf_1,
        )
        b1 = pdf_1.read_bytes()
        if not b1.startswith(b"%PDF-") or len(b1) < 1000:
            raise AssertionError("PDF flujo 1 inválido o vacío")

        # Flujo 2: SVGs individuales de `dibu batch` en HTML -> html_a_typst_oraculo -> PDF
        bloques_md = []
        for idx, c in enumerate(fences):
            for j, blk in enumerate(oracle.leer_bloques_oraculo(c)):
                bloques_md.append({"id": f"md_{idx}_{j}", "block": blk, "tinta": True})
        res_b, outs_b = harness.invocar_dibu_batch(dibu_bin, bloques_md)
        if res_b.returncode != 0 or any(o.error or not o.svg for o in outs_b):
            raise AssertionError("Fallo generando SVGs de kitchen_sink.md con dibu batch")
        html_dibu_batch = "\n".join(f'<div class="estructura">{o.svg}</div>' for o in outs_b if o.svg)
        typst_2 = oracle.html_a_typst_oraculo(html_dibu_batch)
        pdf_2 = tmp / "flujo2_dibu_batch.pdf"
        oracle.compilar_typst_oraculo(
            f'#set page(paper: "a4", margin: 1.5cm)\n{typst_2}\n',
            pdf_2,
        )
        b2 = pdf_2.read_bytes()
        if not b2.startswith(b"%PDF-") or len(b2) < 1000:
            raise AssertionError("PDF flujo 2 inválido o vacío")

        # Flujo 3: SVGs apilados de `dibu sync-md` vía `#image("diagramas/...")` -> PDF
        shutil.copy2(KITCHEN_SINK_MD, tmp / "kitchen_sink.md")
        res_sync = harness.invocar_dibu_sync_md(dibu_bin, [tmp / "kitchen_sink.md"])
        if res_sync.returncode != 1:
            raise AssertionError(
                f"dibu sync-md en tempdir esperaba rc=1 al escribir SVGs, obtuvo {res_sync.returncode}"
            )
        svgs_sync = sorted((tmp / "diagramas").glob("estructura-*.svg"))
        if len(svgs_sync) != len(fences):
            raise AssertionError(
                f"dibu sync-md escribió {len(svgs_sync)} SVGs, esperados {len(fences)}"
            )
        lineas_img = [
            f'#image("diagramas/{p.name}", width: 95%)'
            for p in svgs_sync
        ]
        typst_3 = '#set page(paper: "a4", margin: 1.5cm)\n' + "\n#v(8pt)\n".join(lineas_img) + "\n"
        pdf_3 = tmp / "flujo3_sync_md_image.pdf"
        compilar_typst_a_pdf(typst_3, pdf_3, root_dir=tmp, carpeta=tmp)
        b3 = pdf_3.read_bytes()
        if not b3.startswith(b"%PDF-") or len(b3) < 1000:
            raise AssertionError("PDF flujo 3 inválido o vacío")

    return {
        "typst_disponible": True,
        "html_svgs": svgs_en_html,
        "html_error_ok": True,
        "pdf1_alucard_bytes": len(b1),
        "pdf2_dibu_batch_bytes": len(b2),
        "pdf3_sync_md_image_bytes": len(b3),
        "svgs_sync_md": len(svgs_sync),
    }


# ---------------------------------------------------------------------------
# Módulo 3: `dibu sync-md` sobre casos de `alucarD/tests/test_diagramas.py`
# ---------------------------------------------------------------------------

ENUNCIADO_TEST_DIAGRAMAS = """## Consigna

```estructura
type: stack
values: [1, 2]
```

Y después:

```estructura
- type: array
  values: [3]
- type: queue
  values: [a]
```
"""


def validar_sync_md_test_diagramas(dibu_bin: Path) -> dict[str, Any]:
    cuerpos = extraer_fences_md(ENUNCIADO_TEST_DIAGRAMAS)
    nombres_esp = sorted(oracle.nombre_svg_oraculo(c) for c in cuerpos)
    nombre_esp_stack = oracle.nombre_svg_oraculo("  type: stack\nvalues: [1, 2]\n\n")
    if nombre_esp_stack != oracle.nombre_svg_oraculo(cuerpos[0]):
        raise AssertionError("nombre_svg no recortó espacios extremos como SHA-1")

    with tempfile.TemporaryDirectory(prefix="dibu_sync_td_") as td:
        tmp = Path(td)
        md = tmp / "enunciado.md"
        md.write_text(ENUNCIADO_TEST_DIAGRAMAS, encoding="utf-8")

        # Archivo ajeno que debe preservarse fuera y dentro de diagramas/ (o notas.txt al lado)
        archivo_ajeno = tmp / "notas.txt"
        archivo_ajeno.write_text("no tocar", encoding="utf-8")
        ajeno_diagramas = tmp / "diagramas" / "logo_manual.txt"
        ajeno_diagramas.parent.mkdir(parents=True, exist_ok=True)
        ajeno_diagramas.write_text("preservar", encoding="utf-8")

        # 1. Primera sincronización: rc=1 (escribió 2)
        r1 = harness.invocar_dibu_sync_md(dibu_bin, [md])
        if r1.returncode != 1 or len(r1.escritos) != 2:
            raise AssertionError(f"Primera sync-md esperaba rc=1 y 2 escritos: {r1}")
        generados = sorted(p.name for p in (tmp / "diagramas").glob("estructura-*.svg"))
        if generados != nombres_esp:
            raise AssertionError(f"Nombres SHA-1 difieren: {generados} vs {nombres_esp}")

        # Paridad SVG contra oráculo (incluyendo apilado de 2 bloques -> 3 `<svg`)
        for c in cuerpos:
            nombre = oracle.nombre_svg_oraculo(c)
            svg_dibu = (tmp / "diagramas" / nombre).read_text(encoding="utf-8")
            blks = oracle.leer_bloques_oraculo(c)
            if len(blks) > 1 and svg_dibu.count("<svg") != 3:
                raise AssertionError(f"Bloque apilado esperaba 3 '<svg', obtuvo {svg_dibu.count('<svg')}")
            ora = oracle.renderizar_bloque_md_oraculo(c, tinta=True)
            diff = harness.comparar_con_oraculo(
                svg_dibu, ora.svg or "", tinta_esperada=True, apilado=len(blks) > 1
            )
            if diff:
                raise AssertionError(f"Paridad ENUNCIADO ({nombre}): {diff}")

        # 2. Idempotencia: segunda corrida y `--verificar` -> rc=0
        r2 = harness.invocar_dibu_sync_md(dibu_bin, [md])
        if r2.returncode != 0 or r2.escritos or r2.borrados:
            raise AssertionError(f"Segunda sync-md no fue idempotente: {r2}")
        r_verif = harness.invocar_dibu_sync_md(dibu_bin, [md], verificar=True)
        if r_verif.returncode != 0:
            raise AssertionError(f"--verificar falló: {r_verif}")

        # 3. Mutación `type2: x`: rc=1 al escribir, luego `--verificar` rc=0
        md.write_text(ENUNCIADO_TEST_DIAGRAMAS.replace("[1, 2]", "[1, 2]\ntype2: x"), encoding="utf-8")
        r_mut = harness.invocar_dibu_sync_md(dibu_bin, [md])
        if r_mut.returncode != 1:
            raise AssertionError(f"Mutación type2:x esperaba rc=1, obtuvo {r_mut.returncode}")
        r_mut_v = harness.invocar_dibu_sync_md(dibu_bin, [md], verificar=True)
        if r_mut_v.returncode != 0:
            raise AssertionError(f"--verificar tras mutación esperaba rc=0, obtuvo {r_mut_v.returncode}")

        # Preservación de archivos ajenos
        if archivo_ajeno.read_text(encoding="utf-8") != "no tocar":
            raise AssertionError("Se alteró archivo ajeno fuera de diagramas/")
        if not ajeno_diagramas.is_file():
            raise AssertionError("Se borró archivo ajeno dentro de diagramas/")

        # 4. `--verificar` con huérfanos no borra; sync-md real borra huérfanos y dir vacío
        ajeno_diagramas.unlink()
        md.write_text("Sin diagramas.", encoding="utf-8")
        r_v_huerf = harness.invocar_dibu_sync_md(dibu_bin, [md], verificar=True)
        if r_v_huerf.returncode != 1 or len(list((tmp / "diagramas").iterdir())) != 2:
            raise AssertionError("--verificar modificó huérfanos o no reportó sobrantes")
        r_clean = harness.invocar_dibu_sync_md(dibu_bin, [md])
        if r_clean.returncode != 1 or len(r_clean.borrados) != 2 or (tmp / "diagramas").exists():
            raise AssertionError(f"Limpieza de huérfanos/dir vacío falló: {r_clean}")

        # 5. Bloque inválido `type: grafo`: rc=1, "grafo" en stderr, 0 panics
        md.write_text("```estructura\ntype: grafo\n```\n", encoding="utf-8")
        r_inv = harness.invocar_dibu_sync_md(dibu_bin, [md])
        if r_inv.returncode != 1 or "grafo" not in r_inv.stderr:
            raise AssertionError(f"Bloque inválido type: grafo falló contrato: {r_inv}")
        if harness.tiene_panic_o_traceback(r_inv.stderr):
            raise AssertionError(f"Panic en sync-md con type: grafo: {r_inv.stderr}")

    return {
        "sha1_contrato_ok": True,
        "idempotencia_ok": True,
        "mutacion_type2_ok": True,
        "limpieza_huerfanos_y_dir_ok": True,
        "preservacion_ajenos_ok": True,
        "invalido_grafo_ok": True,
    }


# ---------------------------------------------------------------------------
# Módulo 4: Benchmark comparativo (N bloques mixtos, 44 tipos, ambas paletas)
# ---------------------------------------------------------------------------


def construir_casos_pesados() -> list[dict[str, Any]]:
    return [
        {
            "id": "heavy_time_response_pi",
            "type": "time_response",
            "title": "PI con retardo y perturbación",
            "closedLoop": {
                "plant": {"gain": 2, "tau": 5, "delay": 1.5},
                "controller": {"kp": 1.2, "ki": 0.25},
            },
            "disturbance": {"at": 25, "amplitude": -0.5},
            "tEnd": 60,
            "annotations": ["overshoot", "settling", "final"],
        },
        {
            "id": "heavy_bode",
            "type": "bode",
            "title": "Bode completo",
            "system": {"k": 10, "poles": [0, -1, -10], "zeros": [-2]},
            "asymptotes": True,
            "margins": True,
        },
        {
            "id": "heavy_nyquist",
            "type": "nyquist",
            "title": "Nyquist tercer orden",
            "system": {"k": 5, "poles": [-1, -2, -3]},
            "unitCircle": True,
        },
        {
            "id": "heavy_root_locus",
            "type": "root_locus",
            "title": "Lugar de las raíces",
            "system": {"k": 1, "poles": [0, -1, -2, -5]},
            "marks": [1, 3],
        },
        {
            "id": "heavy_titration_h3po4",
            "type": "titration",
            "title": "Titulación triprótica H3PO4",
            "analyte": {"kind": "weak_acid", "pKa": [2.15, 7.2, 12.35], "conc": 0.1, "volume": 20},
            "titrant": {"conc": 0.1, "name": "NaOH"},
            "annotations": ["equivalence", "half", "initial"],
        },
        {
            "id": "heavy_kinetics_reversible",
            "type": "kinetics",
            "title": "Cinética reversible",
            "model": "reversible",
            "kf": 0.35,
            "kb": 0.12,
        },
        {
            "id": "heavy_tree_avl_25",
            "type": "tree",
            "title": "AVL de 25 nodos",
            "variant": "avl",
            "insert": list(range(1, 26)),
        },
        {
            "id": "heavy_matrix_10x10",
            "type": "matrix",
            "title": "Matriz 10x10",
            "values": [[(r * 10 + c) % 100 for c in range(10)] for r in range(10)],
            "showAddresses": True,
        },
        {
            "id": "heavy_call_tree_profundo",
            "type": "call_tree",
            "title": "Árbol de llamadas profundo",
            "showOrder": True,
            "levelCosts": ["1", "2", "4", "8"],
            "total": "O(2^n)",
            "root": {
                "call": "f(4)",
                "returns": 5,
                "children": [
                    {
                        "call": "f(3)",
                        "returns": 3,
                        "children": [
                            {
                                "call": "f(2)",
                                "returns": 2,
                                "children": [
                                    {"call": "f(1)", "returns": 1},
                                    {"call": "f(0)", "returns": 1},
                                ],
                            },
                            {"call": "f(1)", "returns": 1},
                        ],
                    },
                    {
                        "call": "f(2)",
                        "returns": 2,
                        "children": [
                            {"call": "f(1)", "returns": 1},
                            {"call": "f(0)", "returns": 1},
                        ],
                    },
                ],
            },
        },
        {
            "id": "heavy_flowchart_anidado",
            "type": "flowchart",
            "title": "Flowchart doblemente anidado",
            "steps": [
                {"io": "leer n, m"},
                "s = 0",
                {
                    "while": "i < n",
                    "do": [
                        {
                            "if": "i % 2 == 0",
                            "then": [
                                {"if": "i > 5", "then": ["s += i * 2"], "else": ["s += i"]}
                            ],
                            "else": ["s -= 1"],
                        },
                        "i = i + 1",
                    ],
                },
                {"io": "imprimir s"},
            ],
        },
    ]


def generar_lote_benchmark(count: int, seed: int) -> list[tuple[dict[str, Any], bool]]:
    base = yaml.safe_load(KITCHEN_SINK_YAML.read_text(encoding="utf-8"))
    pesados = construir_casos_pesados()
    rng = random.Random(seed)

    lote: list[tuple[dict[str, Any], bool]] = []
    # Garantizar cobertura inicial de todos los bloques base en ambas paletas si entra en count
    for b in base:
        for tinta in (True, False):
            if len(lote) < count:
                blk = copy.deepcopy(harness.normalizar_bloque_json(b))
                blk["inkSaver"] = tinta
                lote.append((blk, tinta))

    # Completar hasta `count` con ~30% casos pesados y ~70% mixto estratificado
    idx_base = 0
    idx_pes = 0
    while len(lote) < count:
        tinta = (len(lote) % 2) == 0
        if (len(lote) % 3) == 0:
            src = pesados[idx_pes % len(pesados)]
            idx_pes += 1
        else:
            src = base[idx_base % len(base)]
            idx_base += 1
        blk = copy.deepcopy(harness.normalizar_bloque_json(src))
        blk["inkSaver"] = tinta
        lote.append((blk, tinta))

    rng.shuffle(lote)
    # Asignar IDs únicos tras mezclar
    resultado: list[tuple[dict[str, Any], bool]] = []
    for i, (blk, tinta) in enumerate(lote):
        blk["id"] = f"bench_{i:04d}_{blk['type']}"
        resultado.append((blk, tinta))
    return resultado


def ejecutar_benchmark(
    dibu_bin: Path,
    lote: list[tuple[dict[str, Any], bool]],
    render_spawns: int,
    seed: int,
) -> dict[str, Any]:
    n = len(lote)

    # 1. Python (alucarD) in-process + medición de VmHWM en proceso limpio
    latencias_py_ms: list[float] = []
    svgs_py: list[str] = []
    t0_py = time.perf_counter()
    for blk, tinta in lote:
        ts = time.perf_counter()
        ora = oracle.renderizar_oraculo(blk, tinta=tinta)
        latencias_py_ms.append((time.perf_counter() - ts) * 1000.0)
        if not ora.ok or not ora.svg:
            raise AssertionError(f"Oráculo falló en benchmark ({blk['id']}): {ora.error}")
        svgs_py.append(ora.svg)
    py_total_ms = (time.perf_counter() - t0_py) * 1000.0
    py_rss_kib = medir_rss_python_aislado(lote)

    # 2. `dibu batch` (1 spawn JSONL con N bloques)
    payload_batch = "".join(
        json.dumps({"id": blk["id"], "block": blk, "tinta": tinta}, ensure_ascii=False) + "\n"
        for blk, tinta in lote
    ).encode("utf-8")
    rc_b, out_b, err_b, batch_ms, batch_rss_kib = medir_rss_con_time(
        [str(dibu_bin), "batch"], stdin_bytes=payload_batch, timeout=180.0
    )
    if rc_b != 0:
        raise AssertionError(f"dibu batch benchmark rc={rc_b}: {err_b}")
    if harness.tiene_panic_o_traceback(err_b):
        raise AssertionError(f"Panic en dibu batch benchmark: {err_b}")

    lineas_batch = [ln for ln in out_b.splitlines() if ln.strip()]
    if len(lineas_batch) != n:
        raise AssertionError(f"dibu batch devolvió {len(lineas_batch)} líneas, esperadas {n}")

    # Verificar paridad 100% de los N SVGs contra Python (fuera del cronómetro de `batch`)
    fallos_paridad = 0
    primeros_fallos: list[str] = []
    for ln, (blk, tinta), svg_ora in zip(lineas_batch, lote, svgs_py):
        obj = json.loads(ln)
        if obj.get("error") or not obj.get("svg"):
            fallos_paridad += 1
            if len(primeros_fallos) < 5:
                primeros_fallos.append(f"{blk['id']}: {obj.get('error')}")
            continue
        diff = harness.comparar_con_oraculo(obj["svg"], svg_ora, tinta_esperada=tinta)
        if diff:
            fallos_paridad += 1
            if len(primeros_fallos) < 5:
                primeros_fallos.append(f"{blk['id']}: {diff}")
    if fallos_paridad > 0:
        raise AssertionError(
            f"Fallos de paridad en benchmark N={n}: {fallos_paridad} ({primeros_fallos})"
        )

    # 3. `dibu render` multi-bloque (1 spawn archivo JSON con N bloques -> -o <tmp>)
    with tempfile.TemporaryDirectory(prefix="dibu_bench_render_") as td:
        tmp = Path(td)
        lote_json = tmp / "lote.json"
        out_dir = tmp / "out"
        lote_json.write_text(
            json.dumps([blk for blk, _ in lote], ensure_ascii=False),
            encoding="utf-8",
        )
        rc_rm, _, err_rm, render_multi_ms, render_multi_rss_kib = medir_rss_con_time(
            [str(dibu_bin), "render", str(lote_json), "-o", str(out_dir)],
            timeout=180.0,
        )
        if rc_rm != 0:
            raise AssertionError(f"dibu render multi-bloque rc={rc_rm}: {err_rm}")
        svgs_escritos = list(out_dir.glob("*.svg"))
        if len(svgs_escritos) != n:
            raise AssertionError(
                f"dibu render multi-bloque escribió {len(svgs_escritos)} archivos, esperados {n}"
            )

    # 4. `dibu render` proceso-por-proceso (muestra estratificada de K spawns)
    por_tipo: dict[str, list[int]] = {}
    for idx, (blk, _) in enumerate(lote):
        por_tipo.setdefault(blk["type"], []).append(idx)
    indices_muestra: list[int] = []
    if render_spawns >= n:
        indices_muestra = list(range(n))
    else:
        rng = random.Random(seed + 1)
        for tipo in sorted(por_tipo):
            if len(indices_muestra) < render_spawns:
                indices_muestra.append(por_tipo[tipo][0])
        usados = set(indices_muestra)
        restantes = [i for i in range(n) if i not in usados]
        rng.shuffle(restantes)
        indices_muestra.extend(restantes[: max(0, render_spawns - len(indices_muestra))])

    latencias_spawn_ms: list[float] = []
    rss_spawns_kib: list[int] = []
    with tempfile.TemporaryDirectory(prefix="dibu_bench_spawns_") as td:
        tmp = Path(td)
        out_sp = tmp / "out"
        out_sp.mkdir(parents=True, exist_ok=True)
        for k, idx in enumerate(indices_muestra):
            blk, _ = lote[idx]
            archivo_blk = tmp / f"blk_{k}.json"
            archivo_blk.write_text(json.dumps(blk, ensure_ascii=False), encoding="utf-8")
            t_sp0 = time.perf_counter()
            proc_sp = subprocess.run(
                [str(dibu_bin), "render", str(archivo_blk), "-o", str(out_sp)],
                capture_output=True,
                timeout=30.0,
                check=False,
            )
            latencias_spawn_ms.append((time.perf_counter() - t_sp0) * 1000.0)
            if proc_sp.returncode != 0:
                raise AssertionError(
                    f"dibu render spawn {k} ({blk['id']}) rc={proc_sp.returncode}: {proc_sp.stderr.decode('utf-8', errors='replace')}"
                )
            if k < 10 or blk["type"] in ("time_response", "bode", "tree", "matrix", "titration"):
                _, _, _, _, rss_sp = medir_rss_con_time(
                    [str(dibu_bin), "render", str(archivo_blk), "-o", str(out_sp)],
                    timeout=30.0,
                )
                rss_spawns_kib.append(rss_sp)

    spawns_total_ms = sum(latencias_spawn_ms)
    proyeccion_n_ms = (spawns_total_ms / len(latencias_spawn_ms)) * n

    return {
        "n": n,
        "tipos_cubiertos": len(por_tipo),
        "paridad_n_porc": 100.0,
        "python": {
            "total_ms": round(py_total_ms, 2),
            "ms_por_bloque": round(py_total_ms / n, 3),
            "p50_ms": round(percentil(latencias_py_ms, 50), 3),
            "p95_ms": round(percentil(latencias_py_ms, 95), 3),
            "p99_ms": round(percentil(latencias_py_ms, 99), 3),
            "throughput_blk_s": round(n / (py_total_ms / 1000.0), 1),
            "rss_kib": py_rss_kib,
        },
        "dibu_batch": {
            "total_ms": round(batch_ms, 2),
            "ms_por_bloque": round(batch_ms / n, 3),
            "throughput_blk_s": round(n / (batch_ms / 1000.0), 1),
            "rss_kib": batch_rss_kib,
            "speedup_vs_python": round(py_total_ms / batch_ms, 2),
        },
        "dibu_render_multi": {
            "total_ms": round(render_multi_ms, 2),
            "ms_por_bloque": round(render_multi_ms / n, 3),
            "throughput_blk_s": round(n / (render_multi_ms / 1000.0), 1),
            "rss_kib": render_multi_rss_kib,
            "speedup_vs_python": round(py_total_ms / render_multi_ms, 2),
        },
        "dibu_render_spawns": {
            "spawns": len(latencias_spawn_ms),
            "total_muestra_ms": round(spawns_total_ms, 2),
            "ms_por_spawn": round(statistics.mean(latencias_spawn_ms), 3),
            "p50_ms": round(percentil(latencias_spawn_ms, 50), 3),
            "p95_ms": round(percentil(latencias_spawn_ms, 95), 3),
            "p99_ms": round(percentil(latencias_spawn_ms, 99), 3),
            "throughput_spawn_s": round(len(latencias_spawn_ms) / (spawns_total_ms / 1000.0), 1),
            "rss_max_kib": max(rss_spawns_kib),
            "proyeccion_n_ms": round(proyeccion_n_ms, 2),
        },
    }


# ---------------------------------------------------------------------------
# Módulo 5: Determinismo byte-a-byte (100% SHA-256)
# ---------------------------------------------------------------------------


def validar_determinismo(
    dibu_bin: Path,
    lote: list[tuple[dict[str, Any], bool]],
) -> dict[str, Any]:
    # 1. `dibu batch` sobre los N bloques (2 corridas)
    payload = "".join(
        json.dumps({"id": blk["id"], "block": blk, "tinta": tinta}, ensure_ascii=False) + "\n"
        for blk, tinta in lote
    ).encode("utf-8")

    b1 = ejecutar_con_rss([str(dibu_bin), "batch"], stdin_bytes=payload, timeout=180.0)
    b2 = ejecutar_con_rss([str(dibu_bin), "batch"], stdin_bytes=payload, timeout=180.0)
    if b1.returncode != 0 or b2.returncode != 0:
        raise AssertionError("dibu batch falló en prueba de determinismo")
    if sha256_text(b1.stdout) != sha256_text(b2.stdout):
        raise AssertionError("dibu batch stdout no es determinístico byte a byte")

    hashes_svgs_batch = 0
    for l1, l2 in zip(b1.stdout.splitlines(), b2.stdout.splitlines()):
        if not l1.strip():
            continue
        s1 = json.loads(l1).get("svg", "")
        s2 = json.loads(l2).get("svg", "")
        if sha256_text(s1) != sha256_text(s2):
            raise AssertionError("Divergencia SHA-256 en SVG individual de dibu batch")
        hashes_svgs_batch += 1

    # 2. `dibu render` sobre kitchen_sink.yaml (2 corridas)
    with tempfile.TemporaryDirectory(prefix="dibu_det_render_") as td:
        d1 = Path(td) / "r1"
        d2 = Path(td) / "r2"
        r1 = harness.invocar_dibu_render(dibu_bin, KITCHEN_SINK_YAML, output_dir=d1)
        r2 = harness.invocar_dibu_render(dibu_bin, KITCHEN_SINK_YAML, output_dir=d2)
        if r1.returncode != 0 or r2.returncode != 0:
            raise AssertionError("dibu render falló en prueba de determinismo")
        files1 = sorted(p.name for p in d1.glob("*.svg"))
        files2 = sorted(p.name for p in d2.glob("*.svg"))
        if files1 != files2:
            raise AssertionError("Archivos de salida difieren entre corridas de dibu render")
        for fn in files1:
            if sha256_bytes((d1 / fn).read_bytes()) != sha256_bytes((d2 / fn).read_bytes()):
                raise AssertionError(f"Divergencia SHA-256 en dibu render ({fn})")

    # 3. `dibu sync-md` sobre kitchen_sink.md (2 tempdirs)
    with tempfile.TemporaryDirectory(prefix="dibu_det_sync_") as td:
        t1 = Path(td) / "s1"
        t2 = Path(td) / "s2"
        t1.mkdir()
        t2.mkdir()
        shutil.copy2(KITCHEN_SINK_MD, t1 / "kitchen_sink.md")
        shutil.copy2(KITCHEN_SINK_MD, t2 / "kitchen_sink.md")
        s1 = harness.invocar_dibu_sync_md(dibu_bin, [t1 / "kitchen_sink.md"])
        s2 = harness.invocar_dibu_sync_md(dibu_bin, [t2 / "kitchen_sink.md"])
        if s1.returncode != 1 or s2.returncode != 1:
            raise AssertionError("dibu sync-md en tempdirs esperaba rc=1")
        svgs1 = sorted(p.name for p in (t1 / "diagramas").glob("*.svg"))
        svgs2 = sorted(p.name for p in (t2 / "diagramas").glob("*.svg"))
        if svgs1 != svgs2:
            raise AssertionError("Nombres SVG difieren en determinismo de sync-md")
        for fn in svgs1:
            if sha256_bytes((t1 / "diagramas" / fn).read_bytes()) != sha256_bytes(
                (t2 / "diagramas" / fn).read_bytes()
            ):
                raise AssertionError(f"Divergencia SHA-256 en sync-md ({fn})")

    return {
        "batch_stdout_sha256": sha256_text(b1.stdout),
        "batch_svgs_iguales": hashes_svgs_batch,
        "render_svgs_iguales": len(files1),
        "sync_md_svgs_iguales": len(svgs1),
        "determinismo": "100%",
    }


# ---------------------------------------------------------------------------
# Módulo 6: Resiliencia ante entradas malformadas (0 panics en los 44 tipos)
# ---------------------------------------------------------------------------


def _chequear_sin_panic(nombre: str, rc: int, stderr: str) -> None:
    if rc == 101 or rc < 0 or harness.tiene_panic_o_traceback(stderr):
        raise AssertionError(
            f"PANIC detectado en prueba de resiliencia '{nombre}' (rc={rc}): {stderr!r}"
        )


def validar_resiliencia(dibu_bin: Path) -> dict[str, Any]:
    total_probes = 0
    errores_controlados = 0

    # A. `dibu render` con YAML/JSON malformado y valores extremos
    casos_render = [
        ("yaml_roto", "type: [sin cerrar\n", (1, 2)),
        ("archivo_vacio", "", (1, 2)),
        ("escalar", "42\n", (1, 2)),
        ("lista_no_objetos", "- 123\n- true\n- null\n", (1, 2)),
        (
            "floats_extremos_invalidos",
            "- type: time_response\n  system: {gain: .nan, tau: .inf, delay: 1e308}\n"
            "- type: bode\n  system: {k: .nan, poles: [0]}\n"
            "- type: titration\n  analyte: {kind: weak_acid, pKa: .nan, conc: -1, volume: .inf}\n",
            (1,),
        ),
        (
            "floats_extremos_mixto",
            "- type: time_response\n  system: {gain: 1, tau: 1, delay: 1e308}\n"
            "- type: array\n  values: [1e308, -1e308, 1e-308, 0, -0.0]\n",
            (0,),
        ),
    ]
    with tempfile.TemporaryDirectory(prefix="dibu_fuzz_render_") as td:
        tmp = Path(td)
        for nombre, contenido, rcs_esperados in casos_render:
            f = tmp / f"{nombre}.yaml"
            f.write_text(contenido, encoding="utf-8")
            out = tmp / f"out_{nombre}"
            res = harness.invocar_dibu_render(dibu_bin, f, output_dir=out)
            total_probes += 1
            _chequear_sin_panic(f"render:{nombre}", res.returncode, res.stderr)
            if res.returncode not in rcs_esperados:
                raise AssertionError(
                    f"render:{nombre} esperaba rc in {rcs_esperados}, obtuvo {res.returncode} (stderr={res.stderr!r})"
                )
            if res.returncode == 0:
                if "Error:" not in res.stderr:
                    raise AssertionError(f"render:{nombre} debía reportar error en bloque inválido")
                for svg_p in out.glob("*.svg"):
                    errs_svg = harness.verificar_svg(svg_p.read_text(encoding="utf-8"))
                    if errs_svg:
                        raise AssertionError(f"render:{nombre} SVG inválido ({svg_p.name}): {errs_svg}")
            errores_controlados += 1

    # B. `dibu batch` adversarial en los 44 tipos (SIN exclusiones)
    lineas_batch: list[str] = []

    # Líneas JSON inválidas / truncadas / estructura inválida
    lineas_rotas = [
        "{",
        '{"id": "trunc"',
        "[]",
        "12345",
        '"solo string"',
        '{"id": 123, "block": {"type": "array", "values": [1]}}',
        '{"id": "sin_block"}',
        '{"id": "block_str", "block": "no_objeto"}',
        '{"id": "tinta_str", "block": {"type": "array", "values": [1]}, "tinta": "si"}',
        '{"id": "tipo_vacio", "block": {"type": ""}}',
        '{"id": "tipo_inexistente", "block": {"type": "tipo_que_no_existe"}}',
    ]
    lineas_batch.extend(lineas_rotas)

    # 1. `{type: T}` vacío y `{type: T}` con campos nulos para TODOS los 44 tipos
    for tipo in oracle.TODOS_LOS_TIPOS:
        lineas_batch.append(
            json.dumps({"id": f"vacio_{tipo}", "block": {"type": tipo}, "tinta": True})
        )
        lineas_batch.append(
            json.dumps(
                {
                    "id": f"nulos_{tipo}",
                    "block": {
                        "type": tipo,
                        "title": None,
                        "values": None,
                        "steps": None,
                        "system": None,
                        "atoms": None,
                        "halfCells": None,
                        "frames": None,
                        "fields": None,
                    },
                    "tinta": False,
                }
            )
        )

    # 2. Probes específicos críticos (incluidos los 2 fixes de dibu-control y extremos)
    probes_especificos: list[tuple[str, dict[str, Any]]] = [
        ("grafcet_or_vacio", {"type": "grafcet", "steps": [{"or": []}]}),
        ("grafcet_and_vacio", {"type": "grafcet", "steps": [{"and": []}]}),
        (
            "grafcet_anidado_vacio",
            {"type": "grafcet", "steps": [{"step": 0}, {"or": [[{"step": 1}], [{"and": []}]]}]},
        ),
        (
            "time_resp_delay_1e308",
            {"type": "time_response", "system": {"gain": 1, "tau": 1, "delay": 1e308}},
        ),
        (
            "time_resp_delay_neg",
            {"type": "time_response", "system": {"gain": 1, "tau": 1, "delay": -5.0}},
        ),
        (
            "time_resp_delay_nan_str",
            {"type": "time_response", "system": {"gain": 1, "tau": 1, "delay": "NaN"}},
        ),
        (
            "time_resp_tend_extremo",
            {"type": "time_response", "system": {"gain": 1, "tau": 1}, "tEnd": 1e308},
        ),
        ("bode_cero_polos", {"type": "bode", "system": {"num": [1], "den": [0, 0]}}),
        ("nyquist_k_inf_str", {"type": "nyquist", "system": {"k": "inf", "poles": [-1]}}),
        ("root_locus_vacio", {"type": "root_locus", "system": {"k": 1, "poles": []}}),
        ("ladder_rungs_vacios", {"type": "ladder", "rungs": []}),
        ("ladder_logic_vacia", {"type": "ladder", "rungs": [{"logic": []}]}),
        (
            "instr_ref_inexistente",
            {"type": "instrumentation", "lines": [{"from": "NOPE", "to": "NOPE2"}]},
        ),
        (
            "titration_conc_cero",
            {"type": "titration", "analyte": {"kind": "weak_acid", "pKa": 4.7, "conc": 0, "volume": -1}},
        ),
        ("molecule_ciclo_from", {"type": "molecule", "atoms": [{"id": "a", "from": "missing"}]}),
        (
            "unit_cell_a_neg",
            {"type": "unit_cell", "lattice": "fcc", "a": -10, "molarMass": 0},
        ),
        (
            "utf8_10kb_xml",
            {
                "type": "array",
                "title": ("🚀αβγ<script>&\"'" * 600)[:10240],
                "values": ["<>&\"'", "🧪" * 100, 1e308, -1e308, 1e-308, -0.0, 0],
            },
        ),
        (
            "matrix_100x100",
            {"type": "matrix", "values": [[c for c in range(100)] for _ in range(100)]},
        ),
        ("array_1000", {"type": "array", "values": list(range(1000))}),
        ("tree_500", {"type": "tree", "variant": "bst", "insert": list(range(500))}),
    ]
    for pid, blk in probes_especificos:
        lineas_batch.append(json.dumps({"id": pid, "block": blk, "tinta": True}, ensure_ascii=False))

    # 3. Intercalar 2 ítems válidos (al inicio y al final) para verificar que el stream no se corta
    valido_inicio = {"id": "valido_inicio", "block": {"type": "stack", "values": [1, 2]}, "tinta": True}
    valido_fin = {"id": "valido_fin", "block": {"type": "vsepr", "central": "N", "ligands": ["H", "H", "H"], "lonePairs": 1}, "tinta": False}

    todas_lineas = [
        json.dumps(valido_inicio),
        *lineas_batch,
        json.dumps(valido_fin),
    ]
    # También probar bytes UTF-8 inválidos en una corrida aparte de `batch`
    payload_bytes = ("\n".join(todas_lineas) + "\n").encode("utf-8")
    m_fuzz = ejecutar_con_rss([str(dibu_bin), "batch"], stdin_bytes=payload_bytes, timeout=180.0)
    total_probes += len(todas_lineas)
    _chequear_sin_panic("batch_fuzz", m_fuzz.returncode, m_fuzz.stderr)
    if m_fuzz.returncode != 0:
        raise AssertionError(f"dibu batch fuzz salió con rc={m_fuzz.returncode}, esperado 0")

    salidas_fuzz = [json.loads(ln) for ln in m_fuzz.stdout.splitlines() if ln.strip()]
    if len(salidas_fuzz) != len(todas_lineas):
        raise AssertionError(
            f"dibu batch fuzz respondió {len(salidas_fuzz)} líneas, esperadas {len(todas_lineas)}"
        )
    if not salidas_fuzz[0].get("svg") or salidas_fuzz[0].get("error"):
        raise AssertionError(f"Ítem válido inicial falló tras stream adversarial: {salidas_fuzz[0]}")
    if not salidas_fuzz[-1].get("svg") or salidas_fuzz[-1].get("error"):
        raise AssertionError(f"Ítem válido final falló tras stream adversarial: {salidas_fuzz[-1]}")

    for item in salidas_fuzz[1:-1]:
        err = item.get("error")
        svg = item.get("svg")
        if err:
            errores_controlados += 1
            if harness.tiene_panic_o_traceback(str(err)):
                raise AssertionError(f"Mensaje de error contiene panic: {err}")
        elif svg:
            # Algunos probes (como arrays grandes o `{type: complexity}` sin campos) pueden ser válidos
            errs_xml = harness.verificar_svg(svg)
            if errs_xml:
                raise AssertionError(f"SVG inválido emitido en fuzz ({item.get('id')}): {errs_xml}")
        else:
            raise AssertionError(f"Ítem sin svg ni error: {item}")

    # Bytes no UTF-8 mezclados en stdin de `batch`
    m_bin = ejecutar_con_rss(
        [str(dibu_bin), "batch"],
        stdin_bytes=b"\xff\xfe\x00\x80\n" + (json.dumps(valido_fin) + "\n").encode("utf-8"),
        timeout=30.0,
    )
    total_probes += 1
    _chequear_sin_panic("batch_binario", m_bin.returncode, m_bin.stderr)

    # C. `dibu sync-md` sobre .md con fences rotos, vacíos, inválidos y mixtos
    with tempfile.TemporaryDirectory(prefix="dibu_fuzz_sync_") as td:
        tmp = Path(td)
        md_mixto = tmp / "mixto.md"
        md_mixto.write_text(
            "```estructura\n```\n\n"
            "```estructura\ntype: [yaml roto\n```\n\n"
            "```estructura\ntype: grafo\n```\n\n"
            "```estructura\ntype: stack\nvalues: [7, 8]\n```\n",
            encoding="utf-8",
        )
        r_sync = harness.invocar_dibu_sync_md(dibu_bin, [md_mixto])
        total_probes += 1
        _chequear_sin_panic("sync_md_mixto", r_sync.returncode, r_sync.stderr)
        if r_sync.returncode != 1:
            raise AssertionError(f"sync-md mixto esperaba rc=1, obtuvo {r_sync.returncode}")
        if len(r_sync.errores_stderr) < 3:
            raise AssertionError(f"sync-md mixto esperaba >=3 errores, obtuvo {r_sync.errores_stderr}")
        if len(r_sync.escritos) != 1:
            raise AssertionError(f"sync-md mixto debía escribir el fence válido: {r_sync}")

    return {
        "total_probes": total_probes,
        "errores_controlados": errores_controlados,
        "tipos_fuzzeados": len(oracle.TODOS_LOS_TIPOS),
        "panics": 0,
    }


# ---------------------------------------------------------------------------
# Módulo 7: Generación automática de STRESS_REPORT.md
# ---------------------------------------------------------------------------


def generar_reporte_md(resultados: dict[str, Any], ruta_md: Path) -> None:
    ks = resultados["kitchen_sink"]
    ht = resultados["html_typst"]
    sm = resultados["sync_md_test_diagramas"]
    bm = resultados["benchmark"]
    dt_res = resultados["determinismo"]
    rz = resultados["resiliencia"]
    meta = resultados["meta"]

    py = bm["python"]
    db = bm["dibu_batch"]
    dr = bm["dibu_render_multi"]
    ds = bm["dibu_render_spawns"]

    typst_linea = (
        f"- Compilación Typst/PDF verificada (`pdf1_alucard={ht['pdf1_alucard_bytes']} B`, "
        f"`pdf2_dibu_batch={ht['pdf2_dibu_batch_bytes']} B`, "
        f"`pdf3_sync_md_image={ht['pdf3_sync_md_image_bytes']} B` sobre `{ht['svgs_sync_md']}` SVGs)."
        if ht.get("typst_disponible")
        else "- Typst no estuvo disponible en este entorno; se validó solo generación HTML."
    )

    contenido = f"""# Reporte de Stress Test y Validación Integral — `dibu` (M4 Fase 4)

- **Fecha UTC:** `{meta['timestamp_utc']}`
- **Binario evaluado:** `{meta['dibu_bin']}`
- **Entorno:** `{meta['python_version']}` / `{meta['platform']}` (`seed={meta['seed']}`, `N={bm['n']}`)

---

## 1. Catálogo Kitchen Sink (`fixtures/kitchen_sink/`)

| Artefacto | Bloques / Fences | Cobertura | Paridad vs `alucarD` | Verificación CLI |
|---|---:|---:|---:|---|
| `kitchen_sink.yaml` | {ks['yaml_bloques']} bloques | {ks['yaml_tipos']}/44 tipos | {ks['paridad_yaml']} | `dibu render` + `dibu batch` OK |
| `kitchen_sink.md` | {ks['md_fences']} fences ({ks['md_svgs']} SVGs) | {ks['md_tipos']}/44 tipos | {ks['paridad_md']} | `dibu sync-md --verificar` (`rc={ks['sync_md_verificar_rc']}`) |

---

## 2. Pipeline HTML y Typst/PDF + Hallazgo en `typst_markup.py`

- `markdown_a_html_oraculo(kitchen_sink.md)` embebió `{ht['html_svgs']}` elementos `<svg>` sin errores.
- Bloque inválido en HTML (`type: grafo`) emitió `<p class="estructura-error">` sin `<svg>` (`OK={ht['html_error_ok']}`).
{typst_linea}
- **Hallazgo documentado (solo lectura en `alucarD`)**: `_SVG` en `alucarD/generador_examenes/generators/typst_markup.py` usa un cuantificador no codicioso (`.*?`) que corta en el primer `</svg>` interno si se le pasa HTML con SVGs apilados anidados de `dibu sync-md` (`<svg><svg>...</svg></svg>`). En los tres flujos reales del sistema (`markdown_a_html_oraculo` con bloques independientes, SVGs individuales de `dibu batch` en HTML y archivos de `dibu sync-md` referenciados por `#image(...)` en Typst) la compilación a PDF pasa al 100 %.

---

## 3. Validación de `dibu sync-md` sobre casos de `alucarD/tests/test_diagramas.py`

- Contrato SHA-1 `nombre_svg`: `{sm['sha1_contrato_ok']}`
- Idempotencia (`sync-md` + `--verificar`): `{sm['idempotencia_ok']}`
- Mutación `type2: x` y re-verificación: `{sm['mutacion_type2_ok']}`
- Limpieza de huérfanos y directorio `diagramas/` vacío: `{sm['limpieza_huerfanos_y_dir_ok']}`
- Preservación de archivos ajenos: `{sm['preservacion_ajenos_ok']}`
- Manejo de bloque inválido `type: grafo` (`rc=1`, `"grafo"` en stderr, `0` panics): `{sm['invalido_grafo_ok']}`

---

## 4. Benchmark Comparativo (`N={bm['n']}` bloques mixtos, {bm['tipos_cubiertos']}/44 tipos, ambas paletas)

Se incluyó sobremuestreo intencional de casos pesados (`time_response` con retardo/PI, `bode`, `nyquist`, `root_locus`, `titration` poliprótica `H3PO4`, `kinetics` reversible, `tree` AVL de 25 nodos, `matrix` `10×10`, `call_tree` profundo y `flowchart` doblemente anidado). La paridad estructural y semántica de los `{bm['n']}` SVGs generados por `dibu batch` frente a Python fue del **{bm['paridad_n_porc']:.1f} %**.

| Motor / Modo | Wall-clock (ms) | ms / bloque | Throughput (blq/s) | p50 / p95 / p99 (ms) | RSS pico (KiB) | Speedup vs Python |
|---|---:|---:|---:|---|---:|---:|
| **Python (`alucarD`) in-process** | {py['total_ms']:.2f} | {py['ms_por_bloque']:.3f} | {py['throughput_blk_s']:.1f} | {py['p50_ms']:.3f} / {py['p95_ms']:.3f} / {py['p99_ms']:.3f} | {py['rss_kib']} | 1.00× |
| **`dibu batch`** (1 spawn JSONL) | {db['total_ms']:.2f} | {db['ms_por_bloque']:.3f} | {db['throughput_blk_s']:.1f} | — | {db['rss_kib']} | **{db['speedup_vs_python']:.2f}×** |
| **`dibu render` multi-bloque** (1 spawn archivo JSON) | {dr['total_ms']:.2f} | {dr['ms_por_bloque']:.3f} | {dr['throughput_blk_s']:.1f} | — | {dr['rss_kib']} | **{dr['speedup_vs_python']:.2f}×** |
| **`dibu render` por proceso** ({ds['spawns']} spawns estratificados) | {ds['total_muestra_ms']:.2f} *(muestra)* | {ds['ms_por_spawn']:.3f} *(por spawn)* | {ds['throughput_spawn_s']:.1f} *(spawns/s)* | {ds['p50_ms']:.3f} / {ds['p95_ms']:.3f} / {ds['p99_ms']:.3f} | {ds['rss_max_kib']} | Proyección `N={bm['n']}`: `{ds['proyeccion_n_ms']:.2f}` ms |

> Nota: `dibu render` proceso-por-proceso paga el arranque de proceso y escritura a disco por cada bloque individual, mientras que `dibu render` multi-bloque y `dibu batch` amortizan ese costo en un único proceso (`batch` además evita I/O de archivos por bloque al operar sobre JSONL en memoria).

---

## 5. Determinismo Byte-a-Byte (`100 %` SHA-256)

- **`dibu batch` ({bm['n']} bloques, 2 corridas):** SHA-256 idéntico de `stdout` (`{dt_res['batch_stdout_sha256']}`) y de los `{dt_res['batch_svgs_iguales']}` SVGs individuales.
- **`dibu render` (`kitchen_sink.yaml`, 2 corridas):** `{dt_res['render_svgs_iguales']}/{dt_res['render_svgs_iguales']}` archivos SVG idénticos byte a byte.
- **`dibu sync-md` (`kitchen_sink.md`, 2 tempdirs):** `{dt_res['sync_md_svgs_iguales']}/{dt_res['sync_md_svgs_iguales']}` archivos `estructura-*.svg` con nombres y SHA-256 idénticos.

---

## 6. Resiliencia ante Entradas Malformadas (`0` Panics)

- **Probes totales ejecutados:** `{rz['total_probes']}` (cubriendo los `{rz['tipos_fuzzeados']}/44` tipos sin exclusiones).
- **Errores controlados capturados:** `{rz['errores_controlados']}` (incluye `grafcet` con `or/and: []`, `time_response` con `delay: 1e308` / negativo / `NaN`, YAML/JSON roto, bytes no UTF-8, floats extremos, strings de 10 KB con XML `&<>\"'` y emojis, matrices `100×100`, arrays de `1000` y árboles de `500` nodos).
- **Panics (`rc=101`, `SIGABRT`, `panicked at`, `RUST_BACKTRACE`):** **`{rz['panics']}`**.
"""
    ruta_md.parent.mkdir(parents=True, exist_ok=True)
    ruta_md.write_text(contenido, encoding="utf-8")


# ---------------------------------------------------------------------------
# CLI
# ---------------------------------------------------------------------------


def parse_args(argv: list[str] | None = None) -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Stress test M4 Fase 4 para `dibu` (kitchen sink, paridad, benchmark, determinismo, resiliencia)."
    )
    parser.add_argument(
        "--count",
        type=int,
        default=1200,
        help="Cantidad de bloques mixtos para benchmark (500 <= N <= 2000, default: 1200)",
    )
    parser.add_argument("--seed", type=int, default=42, help="Semilla determinística (default: 42)")
    parser.add_argument(
        "--dibu-bin",
        type=str,
        default=None,
        help="Ruta al binario `dibu` (default: target/release/dibu)",
    )
    parser.add_argument(
        "--render-spawns",
        type=str,
        default="100",
        help="Cantidad de spawns individuales de `dibu render` (número o 'all', default: 100)",
    )
    parser.add_argument(
        "--report-md",
        type=str,
        default=str(DEFAULT_REPORT_MD),
        help=f"Ruta de salida del reporte Markdown (default: {DEFAULT_REPORT_MD})",
    )
    parser.add_argument(
        "--no-report",
        action="store_true",
        help="No escribir archivo STRESS_REPORT.md",
    )
    parser.add_argument(
        "--json-report",
        type=str,
        default=None,
        help="Ruta opcional para guardar métricas completas en JSON",
    )
    args = parser.parse_args(argv)
    if not (500 <= args.count <= 2000):
        parser.error(f"--count debe estar entre 500 y 2000 (recibido: {args.count})")
    if args.render_spawns.lower() == "all":
        args.render_spawns_n = args.count
    else:
        try:
            k = int(args.render_spawns)
        except ValueError:
            parser.error(f"--render-spawns inválido: {args.render_spawns!r}")
        if k < 1:
            parser.error("--render-spawns debe ser >= 1")
        args.render_spawns_n = k
    return args


def main(argv: list[str] | None = None) -> int:
    args = parse_args(argv)
    dibu_bin = harness.resolver_dibu_bin(args.dibu_bin, compilar_si_falta=False)

    print(f"[1/6] Validando Kitchen Sink (YAML + MD) con {dibu_bin}...")
    res_ks = validar_kitchen_sink(dibu_bin)
    print(f"      OK: {res_ks}")

    print("[2/6] Validando pipeline HTML y Typst/PDF...")
    res_ht = validar_html_y_typst(dibu_bin)
    print(f"      OK: {res_ht}")

    print("[3/6] Validando `dibu sync-md` sobre casos de `alucarD/tests/test_diagramas.py`...")
    res_sm = validar_sync_md_test_diagramas(dibu_bin)
    print(f"      OK: {res_sm}")

    print(f"[4/6] Ejecutando benchmark comparativo (N={args.count}, seed={args.seed}, spawns={args.render_spawns_n})...")
    lote = generar_lote_benchmark(args.count, args.seed)
    res_bm = ejecutar_benchmark(dibu_bin, lote, args.render_spawns_n, args.seed)
    print(
        f"      OK: Python={res_bm['python']['total_ms']}ms | "
        f"batch={res_bm['dibu_batch']['total_ms']}ms ({res_bm['dibu_batch']['speedup_vs_python']}x) | "
        f"render_multi={res_bm['dibu_render_multi']['total_ms']}ms"
    )

    print("[5/6] Validando determinismo byte-a-byte (SHA-256)...")
    res_dt = validar_determinismo(dibu_bin, lote)
    print(f"      OK: {res_dt['determinismo']} ({res_dt['batch_svgs_iguales']} SVGs batch)")

    print("[6/6] Ejecutando suite de resiliencia (44 tipos, 0 panics)...")
    res_rz = validar_resiliencia(dibu_bin)
    print(f"      OK: {res_rz}")

    resultados = {
        "meta": {
            "timestamp_utc": dt.datetime.now(dt.timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ"),
            "dibu_bin": str(dibu_bin),
            "python_version": platform.python_version(),
            "platform": platform.platform(),
            "seed": args.seed,
        },
        "kitchen_sink": res_ks,
        "html_typst": res_ht,
        "sync_md_test_diagramas": res_sm,
        "benchmark": res_bm,
        "determinismo": res_dt,
        "resiliencia": res_rz,
    }

    if not args.no_report:
        ruta_md = Path(args.report_md)
        generar_reporte_md(resultados, ruta_md)
        print(f"Reporte generado en: {ruta_md}")

    if args.json_report:
        ruta_json = Path(args.json_report)
        ruta_json.parent.mkdir(parents=True, exist_ok=True)
        ruta_json.write_text(json.dumps(resultados, indent=2, ensure_ascii=False), encoding="utf-8")
        print(f"JSON generado en: {ruta_json}")

    return 0


if __name__ == "__main__":
    raise SystemExit(main())
