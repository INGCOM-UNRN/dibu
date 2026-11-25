#!/usr/bin/env python3
"""Valida bloques de dibu contra el esquema de la guía para agentes y, si encuentra dibu, los dibuja.

dibu es permisivo: un campo mal escrito o un valor fuera de la lista se ignoran sin avisar y el
diagrama se dibuja igual, distinto de lo pedido. Este script avisa de esos casos antes de
entregar el contenido.

Uso:
    python3 validar.py ARCHIVO...      # archivos .yaml/.yml/.json con bloques, o documentos
                                       # (.md, .typ, .gift, .txt…) con bloques ```estructura,
                                       # ```statics, ```dibu y directivas {dibu}. En los .gift,
                                       # normaliza los caracteres de ancho completo como alucarD.
    python3 validar.py -               # un bloque (o una lista) YAML/JSON por la entrada estándar

Opciones:
    --esquema RUTA   esquema.json (por omisión, el que está al lado de este script)
    --dibu RUTA      binario de dibu (por omisión DIBU_BIN o `dibu` del PATH)
    --sin-dibu       no dibuja: sólo contrasta con el esquema
    --estricto       los avisos también hacen fallar
    --json           salida en JSON: [{archivo, linea, nivel, mensaje}]

Salida: `archivo:línea: error|aviso: mensaje`. Termina con 0 si no hay errores, 1 si los hay y 2
ante un problema de uso. Requiere Python 3.9 y PyYAML.
"""

from __future__ import annotations

import argparse
import difflib
import json
import os
import re
import shutil
import subprocess
import sys
from pathlib import Path

import yaml

LENGUAJES = ("estructura", "statics", "dibu")

# ---------------------------------------------------------------------------
# YAML como lo lee dibu (YAML 1.2): sólo true/false son booleanos; `yes`, `no`, `on` y `off`
# son textos, y las fechas no se convierten.
# ---------------------------------------------------------------------------


class Cargador(yaml.SafeLoader):
    pass


_QUITAR = {"tag:yaml.org,2002:bool", "tag:yaml.org,2002:int", "tag:yaml.org,2002:float", "tag:yaml.org,2002:timestamp"}
Cargador.yaml_implicit_resolvers = {
    k: [(t, r) for t, r in v if t not in _QUITAR] for k, v in yaml.SafeLoader.yaml_implicit_resolvers.items()
}
Cargador.add_implicit_resolver("tag:yaml.org,2002:bool", re.compile(r"^(?:true|True|TRUE|false|False|FALSE)$"), list("tTfF"))
Cargador.add_implicit_resolver(
    "tag:yaml.org,2002:int", re.compile(r"^(?:[-+]?[0-9]+|0o[0-7]+|0x[0-9a-fA-F]+)$"), list("-+0123456789")
)
Cargador.add_implicit_resolver(
    "tag:yaml.org,2002:float",
    re.compile(r"^(?:[-+]?(?:\.[0-9]+|[0-9]+(?:\.[0-9]*)?)(?:[eE][-+]?[0-9]+)?|[-+]?\.(?:inf|Inf|INF)|\.(?:nan|NaN|NAN))$"),
    list("-+0123456789."),
)


def cargar(texto: str) -> tuple[object, list[int]]:
    """El documento y la línea (desde 1, relativa al texto) de cada bloque de primer nivel."""
    cargador = Cargador(texto)
    try:
        nodo = cargador.get_single_node()
        if nodo is None:
            return None, []
        valor = cargador.construct_document(nodo)
    finally:
        cargador.dispose()
    lineas = [nodo.start_mark.line + 1]
    if isinstance(nodo, yaml.SequenceNode):
        lineas = [n.start_mark.line + 1 for n in nodo.value]
    elif isinstance(nodo, yaml.MappingNode):
        for k, v in nodo.value:
            if k.value == "blocks" and isinstance(v, yaml.SequenceNode):
                lineas = [n.start_mark.line + 1 for n in v.value]
    return valor, lineas


def bloques_de(valor) -> list:
    """Un bloque, una lista de bloques o el formato de documento {blocks: [...]}."""
    if isinstance(valor, list):
        return valor
    if isinstance(valor, dict) and "type" not in valor and isinstance(valor.get("blocks"), list):
        return valor["blocks"]
    return [valor]


# ---------------------------------------------------------------------------
# Contraste con el esquema
# ---------------------------------------------------------------------------


def parecidos(palabra: str, opciones) -> str:
    cerca = difflib.get_close_matches(palabra, list(opciones), n=3, cutoff=0.6)
    return f" (¿{', '.join(repr(c) for c in cerca)}?)" if cerca else ""


def validar_bloque(bloque, esquema: dict) -> list[tuple[str, str]]:
    """[(nivel, mensaje)] de un bloque: errores (no se dibuja) y avisos (se dibuja distinto)."""
    if not isinstance(bloque, dict):
        return [("error", "el bloque no es un objeto (clave: valor)")]
    tipo = bloque.get("type")
    if tipo is None:
        return [("error", "falta el campo 'type'")]
    tipos = esquema["tipos"]
    if tipo not in tipos:
        return [("error", f"no existe el tipo '{tipo}'{parecidos(str(tipo), tipos)}")]
    t = tipos[tipo]
    campos = t["campos"]
    validos = set(campos) | set(esquema["comunes"]) | set(t.get("aceptados", [])) | {"type"}
    hallazgos = []
    for clave, valor in bloque.items():
        if clave not in validos:
            hallazgos.append(("aviso", f"{tipo}: dibu ignora el campo '{clave}'{parecidos(str(clave), validos)}"))
            continue
        c = campos.get(clave)
        if not c:
            continue
        if c["valores"] and isinstance(valor, str) and valor not in c["valores"]:
            opciones = ", ".join(c["valores"])
            hallazgos.append(("aviso", f"{tipo}: '{clave}: {valor}' no es un valor admitido ({opciones})"))
        # Un texto en un campo sí/no (`yes`, `"true"`, `sí`): dibu no lo toma como verdadero.
        # Otras formas (una lista en `blank`, un objeto) las admiten algunos tipos.
        if c["tipo"].startswith("sí/no") and isinstance(valor, str):
            hallazgos.append(("aviso", f"{tipo}: '{clave}' espera true o false, no el texto {json.dumps(valor, ensure_ascii=False)}"))
    return hallazgos


# ---------------------------------------------------------------------------
# Documentos: bloques de código y directivas
# ---------------------------------------------------------------------------

RE_VALLA = re.compile(r"^(\s*)(`{3,}|~{3,})[ \t]*(" + "|".join(LENGUAJES) + r")[ \t]*$")
RE_DIRECTIVA = re.compile(r"^(\s*)(`{3,}|~{3,}|:{3,})[ \t]*\{(dibu|estructura)\}[ \t]*(.*?)\s*$")


# Los bancos GIFT de alucarD escriben el YAML con caracteres de ancho completo (`：`, `［ ］`,
# `｛ ｝`, `＃`) y marcan los fines de renglón con `↵`, para no chocar con la sintaxis de GIFT.
# alucarD los normaliza antes de dibujar (NFKC, salvo superíndices, subíndices y fracciones).
_SIN_NORMALIZAR = ("<super>", "<sub>", "<fraction>")


def normalizar_gift(texto: str) -> str:
    import unicodedata

    intactos = {c for c in set(texto) if unicodedata.decomposition(c).startswith(_SIN_NORMALIZAR)}
    if intactos:
        trozos = re.split("([" + re.escape("".join(sorted(intactos))) + "])", texto)
        texto = "".join(t if i % 2 else unicodedata.normalize("NFKC", t) for i, t in enumerate(trozos))
    else:
        texto = unicodedata.normalize("NFKC", texto)
    return re.sub(r"↵\r?\n?", "\n", texto)


def cuerpos_documento(texto: str, base: Path) -> list[tuple[int, str, str | None]]:
    """[(línea del primer renglón del cuerpo, cuerpo, error)] de los bloques de un documento."""
    lineas = texto.replace("\r", "").split("\n")
    salida = []
    i = 0
    while i < len(lineas):
        m = RE_VALLA.match(lineas[i]) or RE_DIRECTIVA.match(lineas[i])
        if not m:
            i += 1
            continue
        sangria, valla = m.group(1), m.group(2)
        directiva = m.re is RE_DIRECTIVA
        cierre = re.compile(r"^\s*" + re.escape(valla[0]) + "{" + str(len(valla)) + r",}\s*$")
        j = i + 1
        cuerpo = []
        while j < len(lineas) and not cierre.match(lineas[j]):
            cuerpo.append(lineas[j][len(sangria):] if lineas[j].startswith(sangria) else lineas[j].lstrip())
            j += 1
        inicio = i + 2
        if directiva:
            k = 0
            while k < len(cuerpo) and re.match(r"^\s*:[\w-]+:", cuerpo[k]):
                k += 1
            cuerpo, inicio = cuerpo[k:], inicio + k
            arg = m.group(4)
            if arg:
                ruta = base / arg
                try:
                    salida.append((i + 1, ruta.read_text(encoding="utf-8"), None))
                except OSError as e:
                    salida.append((i + 1, "", f"no se pudo leer {arg}: {e.strerror}"))
                i = j + 1
                continue
        salida.append((inicio, "\n".join(cuerpo), None))
        i = j + 1
    return salida


# ---------------------------------------------------------------------------
# dibu
# ---------------------------------------------------------------------------


def binario(explicito: str | None) -> str | None:
    for c in (explicito, os.environ.get("DIBU_BIN"), shutil.which("dibu")):
        if c and Path(c).is_file():
            return c
    return None


def dibujar(dibu: str, bloques: list[dict]) -> list[str | None]:
    """El error de dibu de cada bloque (None si se dibujó)."""
    entrada = "\n".join(json.dumps({"id": str(n), "block": b}, ensure_ascii=False) for n, b in enumerate(bloques))
    proc = subprocess.run([dibu, "batch"], input=entrada + "\n", capture_output=True, text=True, check=False)
    errores: list[str | None] = [None] * len(bloques)
    for linea in proc.stdout.splitlines():
        r = json.loads(linea)
        if r.get("error"):
            errores[int(r["id"])] = r["error"]
    return errores


# ---------------------------------------------------------------------------


def main() -> int:
    ap = argparse.ArgumentParser(description=__doc__.split("\n")[0])
    ap.add_argument("archivos", nargs="+")
    ap.add_argument("--esquema", default=str(Path(__file__).with_name("esquema.json")))
    ap.add_argument("--dibu")
    ap.add_argument("--sin-dibu", action="store_true")
    ap.add_argument("--estricto", action="store_true")
    ap.add_argument("--json", action="store_true")
    args = ap.parse_args()
    try:
        esquema = json.loads(Path(args.esquema).read_text(encoding="utf-8"))
    except OSError as e:
        print(f"no se pudo leer el esquema {args.esquema}: {e.strerror}", file=sys.stderr)
        return 2

    hallazgos: list[dict] = []
    pendientes: list[tuple[str, int, dict]] = []  # (archivo, línea, bloque) para dibujar

    def anotar(archivo, linea, nivel, mensaje):
        hallazgos.append({"archivo": archivo, "linea": linea, "nivel": nivel, "mensaje": mensaje})

    def revisar(archivo: str, texto: str, desplazamiento: int) -> None:
        try:
            valor, lineas = cargar(texto)
        except yaml.YAMLError as e:
            marca = getattr(e, "problem_mark", None)
            linea = desplazamiento + (marca.line if marca else 0)
            pista = ""
            if re.search(r"^·", texto, re.M):
                pista = " (la sangría está escrita con '·', que alucarD no convierte en espacios dentro de un bloque)"
            anotar(archivo, linea, "error", f"YAML inválido: {getattr(e, 'problem', None) or e}{pista}")
            return
        if valor is None:
            anotar(archivo, desplazamiento, "error", "el bloque está vacío")
            return
        for n, bloque in enumerate(bloques_de(valor)):
            linea = desplazamiento + (lineas[n] if n < len(lineas) else 1) - 1
            for nivel, mensaje in validar_bloque(bloque, esquema):
                anotar(archivo, linea, nivel, mensaje)
            if isinstance(bloque, dict) and bloque.get("type") in esquema["tipos"]:
                pendientes.append((archivo, linea, bloque))

    for nombre in args.archivos:
        if nombre == "-":
            revisar("<stdin>", sys.stdin.read(), 1)
            continue
        ruta = Path(nombre)
        try:
            texto = ruta.read_text(encoding="utf-8")
        except OSError as e:
            anotar(nombre, 0, "error", f"no se pudo leer: {e.strerror}")
            continue
        if ruta.suffix.lower() in (".yaml", ".yml", ".json"):
            revisar(nombre, texto, 1)
        else:
            gift = ruta.suffix.lower() == ".gift"
            for linea, cuerpo, error in cuerpos_documento(texto, ruta.parent):
                if error:
                    anotar(nombre, linea, "error", error)
                else:
                    revisar(nombre, normalizar_gift(cuerpo) if gift else cuerpo, linea)

    dibu = None if args.sin_dibu else binario(args.dibu)
    if pendientes and dibu:
        for (archivo, linea, _), error in zip(pendientes, dibujar(dibu, [b for _, _, b in pendientes])):
            if error:
                anotar(archivo, linea, "error", f"dibu: {error}")
    elif pendientes and not args.sin_dibu:
        print("aviso: no se encontró dibu (DIBU_BIN o PATH); sólo se contrastó con el esquema", file=sys.stderr)

    hallazgos.sort(key=lambda h: (h["archivo"], h["linea"]))
    if args.json:
        print(json.dumps(hallazgos, ensure_ascii=False, indent=1))
    else:
        for h in hallazgos:
            print(f"{h['archivo']}:{h['linea']}: {h['nivel']}: {h['mensaje']}")
        errores = sum(h["nivel"] == "error" for h in hallazgos)
        avisos = len(hallazgos) - errores
        print(f"{len(pendientes)} bloques revisados: {errores} errores, {avisos} avisos", file=sys.stderr)
    falla = any(h["nivel"] == "error" or args.estricto for h in hallazgos)
    return 1 if falla else 0


if __name__ == "__main__":
    sys.exit(main())
