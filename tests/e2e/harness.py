"""Harness E2E opaque-box para el binario `dibu` y comparación contra `oracle.py`."""

from __future__ import annotations

import sys

sys.dont_write_bytecode = True

import json
import os
import re
import subprocess
import time
import xml.etree.ElementTree as ET
from collections import Counter
from dataclasses import dataclass, field
from pathlib import Path
from typing import Any, Literal

import yaml

from e2e import oracle

DIBU_ROOT = Path("/home/mrtin/dev/tools/docente/contenidos/dibu")
SVG_NS = "http://www.w3.org/2000/svg"
SVG_TAG = f"{{{SVG_NS}}}svg"
TEXT_TAG = f"{{{SVG_NS}}}text"
TSPAN_TAG = f"{{{SVG_NS}}}tspan"
DEFS_TAG = f"{{{SVG_NS}}}defs"
MARKER_TAG = f"{{{SVG_NS}}}marker"
PATH_TAG = f"{{{SVG_NS}}}path"

_NAN_WORD_RE = re.compile(r"(?<![a-záéíóúñü])nan(?![a-záéíóúñü])", re.IGNORECASE)
_PANIC_RE = re.compile(r"(panicked at|RUST_BACKTRACE|Traceback \(most recent call last\))")


@dataclass(frozen=True)
class ResultadoCli:
    """Resultado crudo de ejecutar un subcomando CLI de `dibu`."""

    returncode: int
    stdout: str
    stderr: str
    cmd: list[str]
    elapsed_ms: float = 0.0


@dataclass(frozen=True)
class ResultadoBatchItem:
    """Ítem individual parseado de la salida JSONL de `dibu batch`."""

    id: str
    svg: str | None
    error: str | None
    raw: dict[str, Any]


@dataclass(frozen=True)
class ResultadoSyncMd:
    """Resultado estructurado de `dibu sync-md`."""

    returncode: int
    stdout: str
    stderr: str
    escritos: list[str]
    desactualizados: list[str]
    borrados: list[str]
    sobran: list[str]
    errores_stderr: list[str]


@dataclass(frozen=True)
class ResultadoCaso:
    """Resultado uniforme de un caso E2E."""

    case_id: str
    feature: str
    tier: int
    status: Literal["PASS", "FAIL", "SKIP"]
    mensaje: str = ""
    detalles: dict[str, Any] = field(default_factory=dict)


@dataclass
class ContextoE2E:
    """Contexto compartido que `run_e2e.py` pasa a cada suite."""

    dibu_bin: Path | None
    tipos_implementados: set[str]
    tier: str  # "1" | "2" | "3" | "4" | "all"
    solo_implementados: bool
    oracle_only: bool
    verbose: bool
    cases_dir: Path
    work_dir: Path


# ---------------------------------------------------------------------------
# Descubrimiento e invocación opaque-box de `dibu`
# ---------------------------------------------------------------------------


def resolver_dibu_bin(
    dibu_bin: str | Path | None = None,
    compilar_si_falta: bool = True,
) -> Path:
    """Resuelve la ruta del binario `dibu`.

    Prioridad:
    1. Argumento explícito (`--dibu-bin`)
    2. Variable de entorno `DIBU_BIN`
    3. `target/release/dibu`
    4. `target/debug/dibu`
    5. Compilación vía `cargo build --offline -p dibu-cli` (o `cargo build -p dibu-cli`)
    """
    candidatos: list[Path] = []
    if dibu_bin is not None:
        ruta = Path(dibu_bin)
        if not ruta.is_file():
            raise FileNotFoundError(f"Binario explícito --dibu-bin inexistente: {ruta}")
        return ruta.resolve()

    env_bin = os.environ.get("DIBU_BIN")
    if env_bin:
        ruta = Path(env_bin)
        if not ruta.is_file():
            raise FileNotFoundError(f"Binario en DIBU_BIN inexistente: {ruta}")
        return ruta.resolve()

    candidatos.append(DIBU_ROOT / "target" / "release" / "dibu")
    candidatos.append(DIBU_ROOT / "target" / "debug" / "dibu")
    for cand in candidatos:
        if cand.is_file() and os.access(cand, os.X_OK):
            return cand.resolve()

    if not compilar_si_falta:
        raise FileNotFoundError(
            "No se encontró el binario `dibu` en target/release ni target/debug y compilar_si_falta=False"
        )

    for cmd in (
        ["cargo", "build", "--offline", "-p", "dibu-cli"],
        ["cargo", "build", "-p", "dibu-cli"],
    ):
        proc = subprocess.run(
            cmd,
            cwd=str(DIBU_ROOT),
            capture_output=True,
            text=True,
            timeout=300,
            check=False,
        )
        if proc.returncode == 0 and (DIBU_ROOT / "target" / "debug" / "dibu").is_file():
            return (DIBU_ROOT / "target" / "debug" / "dibu").resolve()

    raise RuntimeError("No se pudo compilar `dibu-cli` con Cargo")


def ejecutar_cli(
    dibu_bin: Path,
    args: list[str],
    *,
    stdin_text: str | None = None,
    cwd: Path | None = None,
    timeout: float = 30.0,
) -> ResultadoCli:
    """Ejecuta `dibu` con argumentos arbitrarios por `subprocess.run`."""
    cmd = [str(dibu_bin), *args]
    t0 = time.monotonic()
    proc = subprocess.run(
        cmd,
        input=stdin_text,
        cwd=str(cwd) if cwd is not None else None,
        capture_output=True,
        text=True,
        encoding="utf-8",
        timeout=timeout,
        check=False,
    )
    elapsed_ms = (time.monotonic() - t0) * 1000.0
    return ResultadoCli(
        returncode=proc.returncode,
        stdout=proc.stdout,
        stderr=proc.stderr,
        cmd=cmd,
        elapsed_ms=elapsed_ms,
    )


def invocar_dibu_list_types(dibu_bin: Path, timeout: float = 15.0) -> list[str]:
    """Ejecuta `dibu list-types` y retorna la lista de tipos en stdout."""
    res = ejecutar_cli(dibu_bin, ["list-types"], timeout=timeout)
    if res.returncode != 0:
        raise RuntimeError(
            f"`dibu list-types` falló con rc={res.returncode}: {res.stderr.strip()}"
        )
    return [line.strip() for line in res.stdout.splitlines() if line.strip()]


def invocar_dibu_render(
    dibu_bin: Path,
    archivo: Path | str,
    output_dir: Path | str | None = None,
    color: bool = False,
    cwd: Path | None = None,
    timeout: float = 30.0,
) -> ResultadoCli:
    """Ejecuta `dibu render <archivo> [-o <output_dir>] [--color]`."""
    args = ["render", str(archivo)]
    if output_dir is not None:
        args.extend(["-o", str(output_dir)])
    if color:
        args.append("--color")
    return ejecutar_cli(dibu_bin, args, cwd=cwd, timeout=timeout)


def normalizar_bloque_json(bloque: Any) -> Any:
    """Convierte claves `bool` de PyYAML 1.1 (`False -> 'no'`, `True -> 'yes'`) para JSON."""
    if isinstance(bloque, dict):
        nuevo: dict[str, Any] = {}
        for clave, valor in bloque.items():
            if isinstance(clave, bool):
                clave_str = "no" if clave is False else "yes"
            else:
                clave_str = str(clave)
            nuevo[clave_str] = normalizar_bloque_json(valor)
        return nuevo
    if isinstance(bloque, list):
        return [normalizar_bloque_json(item) for item in bloque]
    return bloque


def invocar_dibu_batch(
    dibu_bin: Path,
    items: list[dict[str, Any] | str],
    timeout: float = 60.0,
) -> tuple[ResultadoCli, list[ResultadoBatchItem]]:
    """Ejecuta `dibu batch` enviando líneas JSONL por stdin y parseando stdout."""
    lineas: list[str] = []
    for item in items:
        if isinstance(item, str):
            lineas.append(item)
        else:
            normalizado = normalizar_bloque_json(item)
            lineas.append(json.dumps(normalizado, ensure_ascii=False))
    stdin_payload = "\n".join(lineas) + ("\n" if lineas else "")
    res = ejecutar_cli(dibu_bin, ["batch"], stdin_text=stdin_payload, timeout=timeout)

    parsed: list[ResultadoBatchItem] = []
    for idx, raw_line in enumerate(res.stdout.splitlines()):
        linea = raw_line.strip()
        if not linea:
            continue
        try:
            obj = json.loads(linea)
        except json.JSONDecodeError as exc:
            raise ValueError(
                f"Línea {idx + 1} de `dibu batch` no es JSON válido ({exc}): {linea!r}"
            ) from exc
        parsed.append(
            ResultadoBatchItem(
                id=str(obj.get("id", "")),
                svg=obj.get("svg"),
                error=obj.get("error"),
                raw=obj,
            )
        )
    return res, parsed


def invocar_dibu_sync_md(
    dibu_bin: Path,
    rutas: list[Path | str],
    verificar: bool = False,
    color: bool = False,
    cwd: Path | None = None,
    timeout: float = 30.0,
) -> ResultadoSyncMd:
    """Ejecuta `dibu sync-md <rutas...> [--verificar] [--color]`."""
    args = ["sync-md", *[str(r) for r in rutas]]
    if verificar:
        args.append("--verificar")
    if color:
        args.append("--color")
    res = ejecutar_cli(dibu_bin, args, cwd=cwd, timeout=timeout)

    escritos: list[str] = []
    desactualizados: list[str] = []
    borrados: list[str] = []
    sobran: list[str] = []
    for linea in res.stdout.splitlines():
        s = linea.strip()
        if s.startswith("✓ Escrito: "):
            escritos.append(s.removeprefix("✓ Escrito: ").strip())
        elif s.startswith("Desactualizado: "):
            desactualizados.append(s.removeprefix("Desactualizado: ").strip())
        elif s.startswith("✓ Borrado: "):
            borrados.append(s.removeprefix("✓ Borrado: ").strip())
        elif s.startswith("Sobra: "):
            sobran.append(s.removeprefix("Sobra: ").strip())

    errores_stderr = [
        linea.strip()
        for linea in res.stderr.splitlines()
        if linea.strip().startswith("Error:")
    ]

    return ResultadoSyncMd(
        returncode=res.returncode,
        stdout=res.stdout,
        stderr=res.stderr,
        escritos=escritos,
        desactualizados=desactualizados,
        borrados=borrados,
        sobran=sobran,
        errores_stderr=errores_stderr,
    )


# ---------------------------------------------------------------------------
# Parsing y validación XML / SVG
# ---------------------------------------------------------------------------


def tiene_panic_o_traceback(texto: str) -> bool:
    """Detecta signos de panic en Rust o traceback no controlado en Python."""
    return bool(_PANIC_RE.search(texto or ""))


def parsear_svg(svg_texto: str) -> ET.Element:
    """Parsea el SVG con `xml.etree.ElementTree`."""
    return ET.fromstring(svg_texto)


def extraer_dimensiones_svg(svg_texto: str) -> tuple[int, int]:
    """Extrae `(width, height)` numéricos del elemento raíz `<svg>`."""
    raiz = parsear_svg(svg_texto)
    w_raw = raiz.get("width")
    h_raw = raiz.get("height")
    if w_raw is None or h_raw is None:
        raise ValueError(f"Faltan atributos width/height en <svg>: width={w_raw!r}, height={h_raw!r}")
    w_f = float(w_raw)
    h_f = float(h_raw)
    return int(round(w_f)), int(round(h_f))


def _texto_de_elemento(el: ET.Element, reconstruir_tspan: bool) -> str:
    partes: list[str] = []
    if el.text:
        partes.append(el.text.replace("\xa0", " "))
    for hijo in el:
        if hijo.tag in (TSPAN_TAG, "tspan"):
            if reconstruir_tspan and hijo.get("dx") is not None:
                acumulado = "".join(partes)
                contenido_hijo = _texto_de_elemento(hijo, reconstruir_tspan)
                if (
                    acumulado
                    and not acumulado.endswith(" ")
                    and contenido_hijo
                    and not contenido_hijo.startswith(" ")
                ):
                    partes.append(" ")
            partes.append(_texto_de_elemento(hijo, reconstruir_tspan))
        else:
            partes.append(_texto_de_elemento(hijo, reconstruir_tspan))
        if hijo.tail:
            partes.append(hijo.tail.replace("\xa0", " "))
    return "".join(partes)


def extraer_textos_svg(svg_texto: str, reconstruir_tspan: bool = True) -> list[str]:
    """Extrae todos los nodos `<text>` normalizando `\\xa0` y `<tspan dx=...>`.

    Reemplaza `\\xa0` por espacio común y, cuando `reconstruir_tspan=True`,
    inserta un espacio antes de `<tspan dx=...>` para no pegar texto con
    subíndices/superíndices (ej. `H3PO4 con NaOH`, `Zn2+ 0,01 M`).
    """
    raiz = parsear_svg(svg_texto)
    textos: list[str] = []
    for elem in raiz.iter():
        if elem.tag in (TEXT_TAG, "text"):
            bruto = _texto_de_elemento(elem, reconstruir_tspan=reconstruir_tspan)
            limpio = " ".join(bruto.replace("\xa0", " ").split())
            if limpio:
                textos.append(limpio)
    return textos


def normalizar_texto_plano(texto: str) -> str:
    """Colapsa todo whitespace (incluyendo NBSP) para comparar fórmulas químicas."""
    return re.sub(r"\s+", "", texto.replace("\xa0", " "))


def _contiene_nan_invalido(svg_texto: str) -> bool:
    sin_nanosec = re.sub(r"nanosec\w*", "", svg_texto, flags=re.IGNORECASE)
    return bool(_NAN_WORD_RE.search(sin_nanosec))


def _verificar_svg_hoja(
    elem: ET.Element,
    *,
    tinta_esperada: bool | None,
    min_width: int,
    min_height: int,
    prefijo: str = "",
) -> list[str]:
    errores: list[str] = []
    if elem.tag != SVG_TAG:
        errores.append(f"{prefijo}Tag raíz inválido: esperado {SVG_TAG!r}, obtenido {elem.tag!r}")
        return errores

    if elem.get("class") != "estructura":
        errores.append(f"{prefijo}Falta class='estructura' (obtenido {elem.get('class')!r})")
    if elem.get("role") != "img":
        errores.append(f"{prefijo}Falta role='img' (obtenido {elem.get('role')!r})")
    aria_label = elem.get("aria-label")
    if not aria_label:
        errores.append(f"{prefijo}Falta aria-label no vacío")

    view_box = elem.get("viewBox")
    if not view_box or not view_box.strip():
        errores.append(f"{prefijo}Falta viewBox")

    try:
        w = float(elem.get("width", "nan"))
        h = float(elem.get("height", "nan"))
        if not (w > 0 and h > 0):
            errores.append(f"{prefijo}Dimensiones no positivas: width={w}, height={h}")
        if w < min_width:
            errores.append(f"{prefijo}width={w} < min_width={min_width}")
        if h < min_height:
            errores.append(f"{prefijo}height={h} < min_height={min_height}")
    except ValueError:
        errores.append(
            f"{prefijo}width/height no numéricos: width={elem.get('width')!r}, height={elem.get('height')!r}"
        )

    defs_elems = [h for h in elem if h.tag == DEFS_TAG]
    if not defs_elems:
        errores.append(f"{prefijo}Falta elemento <defs>")
    else:
        markers = [m for d in defs_elems for m in d if m.tag == MARKER_TAG]
        ids = [m.get("id", "") for m in markers]
        main_markers = [m for m in markers if m.get("id", "").startswith("flecha-main-")]
        sub_markers = [m for m in markers if m.get("id", "").startswith("flecha-sub-")]
        if not main_markers:
            errores.append(f"{prefijo}Falta marker 'flecha-main-*' en <defs> (ids={ids})")
        if not sub_markers:
            errores.append(f"{prefijo}Falta marker 'flecha-sub-*' en <defs> (ids={ids})")

        if tinta_esperada is not None and main_markers:
            fills_main = [
                p.get("fill", "").lower()
                for m in main_markers
                for p in m
                if p.tag == PATH_TAG
            ]
            serial_hoja = ET.tostring(elem, encoding="unicode").lower()
            if tinta_esperada:
                if "#1d4ed8" in serial_hoja:
                    errores.append(f"{prefijo}Paleta tinta=True no debe contener #1d4ed8")
                if "#0f172a" in serial_hoja:
                    errores.append(f"{prefijo}Paleta tinta=True no debe contener #0f172a")
                if "#000000" not in fills_main:
                    errores.append(
                        f"{prefijo}Paleta tinta=True esperaba flecha-main con fill=#000000 (obtenido {fills_main})"
                    )
            else:
                if "#1d4ed8" not in fills_main:
                    errores.append(
                        f"{prefijo}Paleta tinta=False esperaba flecha-main con fill=#1d4ed8 (obtenido {fills_main})"
                    )
                if "#0f172a" not in serial_hoja:
                    errores.append(f"{prefijo}Paleta tinta=False esperaba #0f172a (strokeMain/textMain)")

    return errores


def verificar_svg(
    svg_texto: str,
    *,
    tinta_esperada: bool | None = None,
    min_width: int = 1,
    min_height: int = 1,
    text_contains: list[str] | None = None,
    text_absent: list[str] | None = None,
    svg_contains: list[str] | None = None,
    svg_absent: list[str] | None = None,
    apilado: bool = False,
) -> list[str]:
    """Valida estructural y semánticamente un SVG generado por `dibu` o `alucarD`."""
    errores: list[str] = []
    if not svg_texto or not svg_texto.strip():
        return ["SVG vacío"]

    try:
        raiz = parsear_svg(svg_texto)
    except ET.ParseError as exc:
        return [f"XML inválido: {exc}"]

    if raiz.tag != SVG_TAG:
        errores.append(f"Raíz debe ser {SVG_TAG!r}, obtenido {raiz.tag!r}")
        return errores

    if _contiene_nan_invalido(svg_texto):
        errores.append("El SVG contiene 'nan' numérico")

    hijos_svg = [h for h in raiz if h.tag == SVG_TAG]
    if apilado and len(hijos_svg) > 0:
        view_box = raiz.get("viewBox")
        if not view_box or not view_box.strip():
            errores.append("SVG apilado sin viewBox en raíz")
        try:
            w = float(raiz.get("width", "nan"))
            h = float(raiz.get("height", "nan"))
            if not (w > 0 and h > 0):
                errores.append(f"SVG apilado con dimensiones no positivas: width={w}, height={h}")
            if w < min_width:
                errores.append(f"SVG apilado width={w} < min_width={min_width}")
            if h < min_height:
                errores.append(f"SVG apilado height={h} < min_height={min_height}")
        except ValueError:
            errores.append("SVG apilado con width/height no numéricos")

        for idx, hijo in enumerate(hijos_svg):
            errores.extend(
                _verificar_svg_hoja(
                    hijo,
                    tinta_esperada=tinta_esperada,
                    min_width=1,
                    min_height=1,
                    prefijo=f"[hijo {idx}] ",
                )
            )
    else:
        errores.extend(
            _verificar_svg_hoja(
                raiz,
                tinta_esperada=tinta_esperada,
                min_width=min_width,
                min_height=min_height,
            )
        )

    textos = extraer_textos_svg(svg_texto, reconstruir_tspan=True)
    texto_unido = " | ".join(textos)
    texto_plano = normalizar_texto_plano(" ".join(textos))

    for sub in text_contains or []:
        sub_norm = " ".join(str(sub).replace("\xa0", " ").split())
        if sub_norm not in texto_unido and normalizar_texto_plano(sub_norm) not in texto_plano:
            errores.append(
                f"Falta texto esperado {sub!r} en textos extraídos: {textos[:20]}"
            )

    for sub in text_absent or []:
        sub_norm = " ".join(str(sub).replace("\xa0", " ").split())
        if sub_norm in texto_unido:
            errores.append(f"Texto prohibido presente {sub!r} en textos extraídos")

    for sub in svg_contains or []:
        if str(sub) not in svg_texto:
            errores.append(f"Falta fragmento SVG esperado {sub!r}")

    for sub in svg_absent or []:
        if str(sub) in svg_texto:
            errores.append(f"Fragmento SVG prohibido presente {sub!r}")

    return errores


def comparar_con_oraculo(
    svg_dibu: str,
    svg_oraculo: str,
    *,
    tinta_esperada: bool | None = None,
    dim_tol: int = 0,
    exigir_mismos_textos: bool = True,
    apilado: bool = False,
) -> list[str]:
    """Compara paridad estructural y semántica entre SVG de `dibu` y `alucarD`."""
    errores: list[str] = []

    err_dibu = verificar_svg(svg_dibu, tinta_esperada=tinta_esperada, apilado=apilado)
    if err_dibu:
        errores.extend([f"[dibu SVG] {e}" for e in err_dibu])

    err_ora = verificar_svg(svg_oraculo, tinta_esperada=tinta_esperada, apilado=apilado)
    if err_ora:
        errores.extend([f"[oráculo SVG] {e}" for e in err_ora])

    if err_dibu or err_ora:
        return errores

    w_dibu, h_dibu = extraer_dimensiones_svg(svg_dibu)
    w_ora, h_ora = extraer_dimensiones_svg(svg_oraculo)
    if abs(w_dibu - w_ora) > dim_tol:
        errores.append(
            f"Ancho diverge: dibu={w_dibu}, oráculo={w_ora} (tolerancia={dim_tol})"
        )
    if abs(h_dibu - h_ora) > dim_tol:
        errores.append(
            f"Alto diverge: dibu={h_dibu}, oráculo={h_ora} (tolerancia={dim_tol})"
        )

    if exigir_mismos_textos:
        c_dibu = Counter(extraer_textos_svg(svg_dibu, reconstruir_tspan=True))
        c_ora = Counter(extraer_textos_svg(svg_oraculo, reconstruir_tspan=True))
        if c_dibu != c_ora:
            faltan = list((c_ora - c_dibu).elements())
            sobran = list((c_dibu - c_ora).elements())
            errores.append(
                f"Multiset de textos difiere: faltan={faltan[:10]}, sobran={sobran[:10]}"
            )

    raiz_dibu = parsear_svg(svg_dibu)
    raiz_ora = parsear_svg(svg_oraculo)
    hijos_dibu = sum(1 for h in raiz_dibu if h.tag == SVG_TAG)
    hijos_ora = sum(1 for h in raiz_ora if h.tag == SVG_TAG)
    if hijos_dibu != hijos_ora:
        errores.append(
            f"Cantidad de <svg> hijos difiere: dibu={hijos_dibu}, oráculo={hijos_ora}"
        )

    graficos_tags = {
        f"{{{SVG_NS}}}{t}"
        for t in ("path", "rect", "circle", "line", "polyline", "polygon", "ellipse")
    }
    n_graf_dibu = sum(1 for e in raiz_dibu.iter() if e.tag in graficos_tags)
    n_graf_ora = sum(1 for e in raiz_ora.iter() if e.tag in graficos_tags)
    if (n_graf_dibu == 0) != (n_graf_ora == 0):
        errores.append(
            f"Consistencia gráfica difiere: dibu={n_graf_dibu} primitivas, oráculo={n_graf_ora}"
        )

    return errores


# ---------------------------------------------------------------------------
# Carga y ejecución de casos YAML declarativos
# ---------------------------------------------------------------------------


def cargar_casos_yaml(ruta: Path) -> list[dict[str, Any]]:
    """Carga una lista de casos declarativos desde YAML validando unicidad de IDs."""
    contenido = ruta.read_text(encoding="utf-8")
    data = yaml.safe_load(contenido)
    if data is None:
        return []
    if not isinstance(data, list):
        raise ValueError(f"Se esperaba una lista de casos en {ruta}, obtenido {type(data).__name__}")
    vistos: set[str] = set()
    for idx, caso in enumerate(data):
        if isinstance(caso, dict) and "id" in caso:
            cid = str(caso["id"])
            if cid in vistos:
                raise ValueError(f"ID duplicado en {ruta} (índice {idx}): {cid!r}")
            vistos.add(cid)
    return data


def _tinta_efectiva(bloque: dict[str, Any], tinta_cli: bool) -> bool:
    ink_saver = bloque.get("inkSaver")
    if isinstance(ink_saver, bool):
        return ink_saver
    return tinta_cli


def _verificar_error_esperado(error_obtenido: str | None, expect: dict[str, Any], motor: str) -> list[str]:
    if not error_obtenido:
        return [f"{motor}: se esperaba error controlado y no devolvió error"]
    errores: list[str] = []
    if tiene_panic_o_traceback(error_obtenido):
        errores.append(f"{motor}: contiene panic/traceback inesperado: {error_obtenido!r}")
    err_sub = expect.get("error_contains")
    if err_sub:
        if isinstance(err_sub, list):
            for sub in err_sub:
                if str(sub) not in error_obtenido:
                    errores.append(
                        f"{motor}: error {error_obtenido!r} no contiene {sub!r}"
                    )
        else:
            if str(err_sub) not in error_obtenido:
                errores.append(
                    f"{motor}: error {error_obtenido!r} no contiene {err_sub!r}"
                )
    return errores


def ejecutar_caso_diagrama(
    caso: dict[str, Any],
    *,
    dibu_bin: Path | None,
    tipos_implementados: set[str],
    solo_implementados: bool = False,
    oracle_only: bool = False,
) -> ResultadoCaso:
    """Ejecuta un único caso YAML declarativo contra oráculo y (opcionalmente) `dibu batch`."""
    case_id = str(caso["id"])
    feature = str(caso["feature"])
    tier = int(caso["tier"])
    tinta = bool(caso.get("tinta", True))
    expect = dict(caso.get("expect") or {})
    ok_esperado = bool(expect.get("ok", True))
    bloque = dict(caso["block"])
    tinta_ef = _tinta_efectiva(bloque, tinta)

    res_ora = oracle.renderizar_oraculo(bloque, tinta=tinta)
    if ok_esperado:
        if not res_ora.ok or not res_ora.svg:
            return ResultadoCaso(
                case_id=case_id,
                feature=feature,
                tier=tier,
                status="FAIL",
                mensaje=f"Oráculo falló inesperadamente: {res_ora.error}",
            )
        err_ora = verificar_svg(
            res_ora.svg,
            tinta_esperada=tinta_ef,
            min_width=int(expect.get("min_width", 1)),
            min_height=int(expect.get("min_height", 1)),
            text_contains=expect.get("text_contains"),
            text_absent=expect.get("text_absent"),
            svg_contains=expect.get("svg_contains"),
            svg_absent=expect.get("svg_absent"),
        )
        if err_ora:
            return ResultadoCaso(
                case_id=case_id,
                feature=feature,
                tier=tier,
                status="FAIL",
                mensaje=f"Oráculo SVG inválido: {'; '.join(err_ora)}",
            )
    else:
        if res_ora.ok:
            return ResultadoCaso(
                case_id=case_id,
                feature=feature,
                tier=tier,
                status="FAIL",
                mensaje="Oráculo aceptó un bloque que debía fallar",
            )
        err_msg_ora = _verificar_error_esperado(res_ora.error, expect, "oráculo")
        if err_msg_ora:
            return ResultadoCaso(
                case_id=case_id,
                feature=feature,
                tier=tier,
                status="FAIL",
                mensaje="; ".join(err_msg_ora),
            )

    if oracle_only:
        return ResultadoCaso(
            case_id=case_id,
            feature=feature,
            tier=tier,
            status="PASS",
            mensaje="Validado contra oráculo (--oracle-only)",
        )

    if feature not in tipos_implementados:
        if solo_implementados:
            return ResultadoCaso(
                case_id=case_id,
                feature=feature,
                tier=tier,
                status="SKIP",
                mensaje=f"Tipo {feature!r} aún no registrado en `dibu list-types`",
            )
        return ResultadoCaso(
            case_id=case_id,
            feature=feature,
            tier=tier,
            status="FAIL",
            mensaje=f"Tipo {feature!r} faltante en `dibu list-types` (modo estricto)",
        )

    if dibu_bin is None:
        return ResultadoCaso(
            case_id=case_id,
            feature=feature,
            tier=tier,
            status="FAIL",
            mensaje="No se proveyó binario `dibu`",
        )

    items = [{"id": case_id, "block": bloque, "tinta": tinta}]
    res_cli_1, out_1 = invocar_dibu_batch(dibu_bin, items)
    if res_cli_1.returncode != 0 or tiene_panic_o_traceback(res_cli_1.stderr):
        return ResultadoCaso(
            case_id=case_id,
            feature=feature,
            tier=tier,
            status="FAIL",
            mensaje=f"`dibu batch` falló (rc={res_cli_1.returncode}): {res_cli_1.stderr.strip()}",
        )
    if len(out_1) != 1:
        return ResultadoCaso(
            case_id=case_id,
            feature=feature,
            tier=tier,
            status="FAIL",
            mensaje=f"`dibu batch` devolvió {len(out_1)} ítems para 1 entrada",
        )
    item_1 = out_1[0]

    if expect.get("check_determinism", True):
        res_cli_2, out_2 = invocar_dibu_batch(dibu_bin, items)
        if res_cli_2.returncode != 0 or len(out_2) != 1:
            return ResultadoCaso(
                case_id=case_id,
                feature=feature,
                tier=tier,
                status="FAIL",
                mensaje="Segunda corrida de determinismo falló",
            )
        if res_cli_1.stdout != res_cli_2.stdout:
            return ResultadoCaso(
                case_id=case_id,
                feature=feature,
                tier=tier,
                status="FAIL",
                mensaje="Falla de determinismo byte-a-byte entre dos corridas de `dibu batch`",
            )

    return _evaluar_item_dibu_contra_caso(
        caso=caso,
        item=item_1,
        svg_oraculo=res_ora.svg,
        tinta_ef=tinta_ef,
    )


def _evaluar_item_dibu_contra_caso(
    *,
    caso: dict[str, Any],
    item: ResultadoBatchItem,
    svg_oraculo: str | None,
    tinta_ef: bool,
) -> ResultadoCaso:
    case_id = str(caso["id"])
    feature = str(caso["feature"])
    tier = int(caso["tier"])
    expect = dict(caso.get("expect") or {})
    ok_esperado = bool(expect.get("ok", True))

    if ok_esperado:
        if item.error or not item.svg:
            return ResultadoCaso(
                case_id=case_id,
                feature=feature,
                tier=tier,
                status="FAIL",
                mensaje=f"`dibu` devolvió error inesperado: {item.error}",
            )
        errores_svg = verificar_svg(
            item.svg,
            tinta_esperada=tinta_ef,
            min_width=int(expect.get("min_width", 1)),
            min_height=int(expect.get("min_height", 1)),
            text_contains=expect.get("text_contains"),
            text_absent=expect.get("text_absent"),
            svg_contains=expect.get("svg_contains"),
            svg_absent=expect.get("svg_absent"),
        )
        if errores_svg:
            return ResultadoCaso(
                case_id=case_id,
                feature=feature,
                tier=tier,
                status="FAIL",
                mensaje=f"`dibu` SVG inválido: {'; '.join(errores_svg)}",
            )
        if expect.get("compare_oracle", True) and svg_oraculo is not None:
            err_cmp = comparar_con_oraculo(
                item.svg,
                svg_oraculo,
                tinta_esperada=tinta_ef,
                dim_tol=int(expect.get("dim_tol", 0)),
                exigir_mismos_textos=bool(expect.get("compare_texts", True)),
            )
            if err_cmp:
                return ResultadoCaso(
                    case_id=case_id,
                    feature=feature,
                    tier=tier,
                    status="FAIL",
                    mensaje=f"Divergencia vs oráculo: {'; '.join(err_cmp)}",
                )
        return ResultadoCaso(
            case_id=case_id,
            feature=feature,
            tier=tier,
            status="PASS",
        )

    if item.svg is not None:
        return ResultadoCaso(
            case_id=case_id,
            feature=feature,
            tier=tier,
            status="FAIL",
            mensaje="`dibu` renderizó SVG cuando se esperaba error",
        )
    err_check = _verificar_error_esperado(item.error, expect, "dibu")
    if err_check:
        return ResultadoCaso(
            case_id=case_id,
            feature=feature,
            tier=tier,
            status="FAIL",
            mensaje="; ".join(err_check),
        )
    return ResultadoCaso(case_id=case_id, feature=feature, tier=tier, status="PASS")


def ejecutar_casos_diagrama(
    rutas_yaml: list[Path],
    ctx: ContextoE2E,
) -> list[ResultadoCaso]:
    """Ejecuta todos los casos declarativos YAML filtrados por `ctx.tier` en lote."""
    casos: list[dict[str, Any]] = []
    vistos_global: set[str] = set()
    for ruta in rutas_yaml:
        for caso in cargar_casos_yaml(ruta):
            cid = str(caso["id"])
            if cid in vistos_global:
                raise ValueError(f"ID duplicado al cargar {ruta}: {cid!r}")
            vistos_global.add(cid)
            casos.append(caso)

    if ctx.tier != "all":
        tier_num = int(ctx.tier)
        casos = [c for c in casos if int(c["tier"]) == tier_num]

    if not casos:
        return []

    resultados: list[ResultadoCaso] = []
    pendientes_dibu: list[tuple[dict[str, Any], str | None, bool]] = []

    for caso in casos:
        case_id = str(caso["id"])
        feature = str(caso["feature"])
        tier = int(caso["tier"])
        tinta = bool(caso.get("tinta", True))
        expect = dict(caso.get("expect") or {})
        ok_esperado = bool(expect.get("ok", True))
        bloque = dict(caso["block"])
        tinta_ef = _tinta_efectiva(bloque, tinta)

        res_ora = oracle.renderizar_oraculo(bloque, tinta=tinta)
        if ok_esperado:
            if not res_ora.ok or not res_ora.svg:
                resultados.append(
                    ResultadoCaso(
                        case_id=case_id,
                        feature=feature,
                        tier=tier,
                        status="FAIL",
                        mensaje=f"Oráculo falló inesperadamente: {res_ora.error}",
                    )
                )
                continue
            err_ora = verificar_svg(
                res_ora.svg,
                tinta_esperada=tinta_ef,
                min_width=int(expect.get("min_width", 1)),
                min_height=int(expect.get("min_height", 1)),
                text_contains=expect.get("text_contains"),
                text_absent=expect.get("text_absent"),
                svg_contains=expect.get("svg_contains"),
                svg_absent=expect.get("svg_absent"),
            )
            if err_ora:
                resultados.append(
                    ResultadoCaso(
                        case_id=case_id,
                        feature=feature,
                        tier=tier,
                        status="FAIL",
                        mensaje=f"Oráculo SVG inválido: {'; '.join(err_ora)}",
                    )
                )
                continue
        else:
            if res_ora.ok:
                resultados.append(
                    ResultadoCaso(
                        case_id=case_id,
                        feature=feature,
                        tier=tier,
                        status="FAIL",
                        mensaje="Oráculo aceptó un bloque que debía fallar",
                    )
                )
                continue
            err_msg_ora = _verificar_error_esperado(res_ora.error, expect, "oráculo")
            if err_msg_ora:
                resultados.append(
                    ResultadoCaso(
                        case_id=case_id,
                        feature=feature,
                        tier=tier,
                        status="FAIL",
                        mensaje="; ".join(err_msg_ora),
                    )
                )
                continue

        if ctx.oracle_only:
            resultados.append(
                ResultadoCaso(
                    case_id=case_id,
                    feature=feature,
                    tier=tier,
                    status="PASS",
                    mensaje="Validado contra oráculo (--oracle-only)",
                )
            )
            continue

        if feature not in ctx.tipos_implementados:
            if ctx.solo_implementados:
                resultados.append(
                    ResultadoCaso(
                        case_id=case_id,
                        feature=feature,
                        tier=tier,
                        status="SKIP",
                        mensaje=f"Tipo {feature!r} aún no registrado en `dibu list-types`",
                    )
                )
            else:
                resultados.append(
                    ResultadoCaso(
                        case_id=case_id,
                        feature=feature,
                        tier=tier,
                        status="FAIL",
                        mensaje=f"Tipo {feature!r} faltante en `dibu list-types` (modo estricto)",
                    )
                )
            continue

        pendientes_dibu.append((caso, res_ora.svg, tinta_ef))

    if not pendientes_dibu:
        return resultados

    if ctx.dibu_bin is None:
        for caso, _, _ in pendientes_dibu:
            resultados.append(
                ResultadoCaso(
                    case_id=str(caso["id"]),
                    feature=str(caso["feature"]),
                    tier=int(caso["tier"]),
                    status="FAIL",
                    mensaje="No se proveyó binario `dibu`",
                )
            )
        return resultados

    batch_req = [
        {
            "id": str(caso["id"]),
            "block": dict(caso["block"]),
            "tinta": bool(caso.get("tinta", True)),
        }
        for caso, _, _ in pendientes_dibu
    ]
    res_1, items_1 = invocar_dibu_batch(ctx.dibu_bin, batch_req)
    res_2, items_2 = invocar_dibu_batch(ctx.dibu_bin, batch_req)

    if (
        res_1.returncode != 0
        or res_2.returncode != 0
        or tiene_panic_o_traceback(res_1.stderr)
        or tiene_panic_o_traceback(res_2.stderr)
    ):
        # Si el lote completo crasheó, aislar por caso para señalar exactamente cuál hizo panic.
        for caso, svg_ora, tinta_ef in pendientes_dibu:
            resultados.append(
                ejecutar_caso_diagrama(
                    caso,
                    dibu_bin=ctx.dibu_bin,
                    tipos_implementados=ctx.tipos_implementados,
                    solo_implementados=ctx.solo_implementados,
                    oracle_only=False,
                )
            )
        return resultados

    mapa_1: dict[str, ResultadoBatchItem] = {}
    for it in items_1:
        if it.id in mapa_1:
            raise ValueError(f"ID duplicado en respuesta de `dibu batch`: {it.id!r}")
        mapa_1[it.id] = it
    mapa_2 = {it.id: it for it in items_2}

    for caso, svg_ora, tinta_ef in pendientes_dibu:
        cid = str(caso["id"])
        expect = dict(caso.get("expect") or {})
        it1 = mapa_1.get(cid)
        it2 = mapa_2.get(cid)
        if it1 is None:
            resultados.append(
                ResultadoCaso(
                    case_id=cid,
                    feature=str(caso["feature"]),
                    tier=int(caso["tier"]),
                    status="FAIL",
                    mensaje="Ítem faltante en respuesta de `dibu batch`",
                )
            )
            continue
        if expect.get("check_determinism", True):
            if it2 is None or it1.svg != it2.svg or it1.error != it2.error:
                resultados.append(
                    ResultadoCaso(
                        case_id=cid,
                        feature=str(caso["feature"]),
                        tier=int(caso["tier"]),
                        status="FAIL",
                        mensaje="Falla de determinismo byte-a-byte entre dos corridas de `dibu batch`",
                    )
                )
                continue
        resultados.append(
            _evaluar_item_dibu_contra_caso(
                caso=caso,
                item=it1,
                svg_oraculo=svg_ora,
                tinta_ef=tinta_ef,
            )
        )

    return resultados
