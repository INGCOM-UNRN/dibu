#!/usr/bin/env python3
"""Genera el manual de dibu: una página por tipo de diagrama, jerarquizada por área y subárea.

Fuentes:
- `datos/*.yaml`: lo escrito a mano (áreas, subáreas, resumen de cada tipo, qué calcula, la
  descripción de cada campo y las notas).
- El código de los renderizadores (`crates/`): para cada tipo, las claves que lee, los valores
  admitidos (`matches!`, `match`, `opcion(...)`) y los valores por defecto (`unwrap_or`).
- `fixtures/kitchen_sink/kitchen_sink.yaml`: los ejemplos de cada tipo, que se dibujan con `dibu` y
  quedan como SVG junto a la página.

Uso:
    python3 docs/manual/generar.py               # escribe docs/manual/
    python3 docs/manual/generar.py --verificar   # falla si falta documentar algo o está desactualizado

`DIBU_BIN` elige el binario (por omisión target/release/dibu, target/debug/dibu o el del PATH).
"""

from __future__ import annotations

import argparse
import json
import os
import re
import shutil
import subprocess
import sys
from collections import OrderedDict
from pathlib import Path

import yaml

RAIZ = Path(__file__).resolve().parents[2]
MANUAL = RAIZ / "docs" / "manual"
DATOS = MANUAL / "datos"
KITCHEN_SINK = RAIZ / "fixtures" / "kitchen_sink" / "kitchen_sink.yaml"
CRATES = RAIZ / "crates"

COMUNES = OrderedDict([
    ("title", "Título del diagrama, arriba a la izquierda."),
    ("subtitle", "Subtítulo, debajo del título, en letra más chica."),
    ("id", "Nombre del archivo al exportar (`dibu render` escribe `<id>.svg`)."),
    ("inkSaver", "`true` usa el theme `ink` y `false` el `color`, sea cual sea el theme global."),
    ("theme", "Theme de este diagrama: un preset (`ink`, `color`, `high-contrast`, `dark`) o un objeto `{extends, colors, typography}`. Ver [Themes](../../themes.md)."),
])


# ---------------------------------------------------------------------------
# Esquema desde el código (port de dibu-web/scripts/esquema.mjs)
# ---------------------------------------------------------------------------

VAR_BLOQUE = r"(?:b|bloque|blk|item|it)"
RE_CLAVE = re.compile(r"\(\s*&?" + VAR_BLOQUE + r'\s*,\s*"([A-Za-z][A-Za-z0-9_]*)"')
RE_GET = re.compile(VAR_BLOQUE + r'\.get\(\s*"([A-Za-z][A-Za-z0-9_]*)"')
RE_LLAMADA = re.compile(r"\b([a-z_][a-z0-9_]*)\s*\(\s*(?:&?\s*(?:mut\s+)?c\s*,\s*)?&?" + VAR_BLOQUE + r"\b")
# Tipo de valor según la función con que el código lee la clave.
LECTORES = {
    "sí/no": ("bool_opt", "booleano", "flag_f", "flag_t", "es_bool_true"),
    "número": ("entero_b", "opt_num", "num_b", "num_req", "hex_b", "dimension", "chem_pos_o_def"),
    "texto": ("str_o", "opt_texto", "str_b", "opcion", "str_opt", "texto_opt", "texto_seq"),
    "lista": ("opt_lista", "lista", "lista_o_vacia", "lista_de", "enteros_set", "valores_num", "lista_num",
              "strings_set", "lista_str", "indices", "secuencia"),
}
RE_LECTOR = re.compile(r"\b([a-z_]+)\(\s*&?" + VAR_BLOQUE + r'\s*,\s*"([A-Za-z][A-Za-z0-9_]*)"')
RE_OPCION = re.compile(r"opcion\(\s*&?" + VAR_BLOQUE + r'\s*,\s*"([A-Za-z0-9_]+)"\s*,\s*&\[([^\]]*)\]\s*,\s*"([^"]*)"')


def _cuerpo_desde(src: str, i: int) -> str:
    ini = src.find("{", i)
    if ini < 0:
        return ""
    prof, j = 0, ini
    while j < len(src):
        ch = src[j]
        if ch == '"':
            j += 1
            while j < len(src) and src[j] != '"':
                j += 2 if src[j] == "\\" else 1
        elif ch == "{":
            prof += 1
        elif ch == "}":
            prof -= 1
            if prof == 0:
                return src[ini:j + 1]
        j += 1
    return src[ini:]


def esquema_codigo() -> dict:
    """{tipo: {"claves": [...], "enums": {clave: [...]}, "defs": {clave: valor}}}"""
    fns: dict[str, str] = {}
    registros: list[tuple[str, str]] = []
    for f in sorted(CRATES.rglob("*.rs")):
        if "/tests/" in str(f) or "dibu-cli" in str(f):
            continue
        src = f.read_text(encoding="utf-8")
        corte = src.find("#[cfg(test)]")
        if corte >= 0:
            src = src[:corte]
        for m in re.finditer(r"\bfn\s+([a-z_][a-z0-9_]*)\s*(?:<[^>]*>)?\s*\(", src):
            fns.setdefault(m.group(1), _cuerpo_desde(src, m.start()))
        for m in re.finditer(r'registrar\(\s*"([a-z_0-9]+)"\s*,\s*((?:[a-z_][a-z0-9_]*::)*[a-z_][a-z0-9_]*)\s*\)', src):
            registros.append((m.group(1), m.group(2).split("::")[-1]))
        for inv in re.finditer(r"registrar_tipos!\(\s*reg\s*,(.*?)\);", src, re.S):
            for m in re.finditer(r'"([a-z_0-9]+)"\s*=>\s*((?:[a-z_][a-z0-9_]*::)*[a-z_][a-z0-9_]*)', inv.group(1)):
                registros.append((m.group(1), m.group(2).split("::")[-1]))
        for m in re.finditer(r'registrar\(\s*"([a-z_0-9]+)"\s*,\s*\|', src):
            fin = src.find("reg.registrar", m.start() + 10)
            nombre = f"__closure_{m.group(1)}"
            fns[nombre] = src[m.start(): fin if fin >= 0 else src.find("\n}", m.start())]
            registros.append((m.group(1), nombre))

    tipos = {}
    for tipo, fn in registros:
        clases: dict[str, str] = {}
        claves: list[str] = []
        enums: dict[str, list[str]] = {}
        defs: dict[str, str] = {}
        vistos: set[str] = set()

        def visitar(nombre: str, prof: int) -> None:
            if nombre in vistos or prof > 3 or nombre not in fns:
                return
            vistos.add(nombre)
            cuerpo = fns[nombre]
            for m in RE_LECTOR.finditer(cuerpo):
                for nombre_clase, lectores in LECTORES.items():
                    if m.group(1) in lectores:
                        clases.setdefault(m.group(2), nombre_clase)
            for rx in (RE_CLAVE, RE_GET):
                for m in rx.finditer(cuerpo):
                    if m.group(1) not in claves:
                        claves.append(m.group(1))
            patron = re.compile(r"let\s+(?:mut\s+)?([a-z_][a-z0-9_]*)\s*(?::[^=]+)?=\s*([^;]*?\(\s*&?" + VAR_BLOQUE
                                + r'\s*,\s*"([A-Za-z0-9_]+)"[^;]*);')
            for m in patron.finditer(cuerpo):
                var, expr, clave = m.groups()
                d = re.search(r'unwrap_or\(\s*"([^"]*)"\s*\)', expr)
                if d:
                    defs[clave] = d.group(1)
                resto = cuerpo[m.start(): m.start() + 4000]
                valores: list[str] = []
                mm = re.search(r"matches!\(\s*" + re.escape(var) + r'\s*,\s*((?:"[^"]*"\s*\|?\s*)+)', resto)
                if mm:
                    valores += re.findall(r'"([^"]*)"', mm.group(1))
                mt = re.search(r"match\s+" + re.escape(var) + r"\s*\{", resto)
                if mt:
                    bloque = _cuerpo_desde(resto, mt.start())
                    for x in re.finditer(r'(?:^|[\n{,])\s*((?:"[^"]*"\s*\|?\s*)+)=>', bloque):
                        valores += re.findall(r'"([^"]*)"', x.group(1))
                if d:
                    valores.append(d.group(1))
                unicos = list(dict.fromkeys(valores))
                if len(unicos) > 1:
                    enums[clave] = unicos
            for m in RE_OPCION.finditer(cuerpo):
                if m.group(1) not in claves:
                    claves.append(m.group(1))
                enums[m.group(1)] = re.findall(r'"([^"]*)"', m.group(2))
                defs[m.group(1)] = m.group(3)
            for m in RE_LLAMADA.finditer(cuerpo):
                visitar(m.group(1), prof + 1)

        visitar(fn, 0)
        tipos[tipo] = {"claves": [c for c in claves if c != "type"], "enums": enums, "defs": defs, "clases": clases}
    return tipos


# ---------------------------------------------------------------------------
# Ejemplos del kitchen sink
# ---------------------------------------------------------------------------

def ejemplos_kitchen_sink() -> dict[str, list[tuple[str, str, dict]]]:
    """{tipo: [(id, yaml_sin_id, bloque)]} en el orden del archivo."""
    texto = KITCHEN_SINK.read_text(encoding="utf-8")
    trozos: list[list[str]] = []
    for linea in texto.split("\n"):
        if linea.startswith("- "):
            trozos.append([linea])
        elif trozos and not linea.startswith("#"):
            trozos[-1].append(linea)
    out: dict[str, list] = {}
    for t in trozos:
        while t and not t[-1].strip():
            t.pop()
        crudo = "\n".join(t)
        bloque = yaml.safe_load(crudo)[0]
        ident = bloque.get("id", "")
        # Sin la línea del id; el primer renglón conserva el guion.
        lineas = [l for l in t if not re.match(r"^(- |  )id:\s", l)]
        if t[0].startswith("- id:"):
            lineas[0] = "- " + lineas[0][2:]
        cuerpo = "\n".join(l[2:] if l.startswith(("- ", "  ")) else l for l in lineas)
        out.setdefault(bloque["type"], []).append((ident, cuerpo, bloque))
    return out


def binario_dibu() -> str:
    for c in (os.environ.get("DIBU_BIN"), RAIZ / "target/release/dibu", RAIZ / "target/debug/dibu", shutil.which("dibu")):
        if c and Path(c).is_file():
            return str(c)
    sys.exit("no se encontró dibu: compilá con `cargo build --release -p dibu-cli` o definí DIBU_BIN")


def renderizar(bloques: list[tuple[str, dict]]) -> dict[str, str]:
    """{id: svg} con `dibu batch`, en un solo proceso."""
    entrada = "\n".join(json.dumps({"id": i, "block": b, "tinta": True}, ensure_ascii=False) for i, b in bloques)
    proc = subprocess.run([binario_dibu(), "batch"], input=entrada + "\n", capture_output=True, text=True, check=False)
    out = {}
    for linea in proc.stdout.splitlines():
        r = json.loads(linea)
        if r.get("error"):
            sys.exit(f"el ejemplo {r['id']} no se dibuja: {r['error']}")
        out[r["id"]] = r["svg"]
    return out


# ---------------------------------------------------------------------------
# Páginas
# ---------------------------------------------------------------------------

# Tipos de campos que se llaman igual en varios diagramas y que no salen de los ejemplos ni de
# cómo los lee el código.
TIPOS_GENERICOS = {
    "systems": "lista", "system": "objeto", "closedLoop": "objeto", "wMin": "número", "wMax": "número",
    "limit": "número", "kMax": "número", "xRange": "lista", "yRange": "lista", "yrange": "lista",
    "range": "lista", "dampingLines": "lista", "wnCircles": "lista", "data": "lista", "amplitude": "número",
    "settlingBand": "número", "yMin": "número", "yMax": "número", "vMax": "número", "grid": "número",
    "capacity": "número", "topIndex": "número", "sign": "texto", "geometry": "texto", "oxidation": "número",
    "electrons": "número", "spin": "texto", "lambda": "número", "unit": "texto", "start": "texto",
    "end": "texto", "anode": "objeto", "cathode": "objeto", "saltBridge": "objeto", "hex": "texto",
    "notation": "texto", "vars": "lista", "bondLength": "número", "order": "número", "k": "número",
    "xStart": "número", "hStart": "número", "bands": "lista", "counts": "objeto", "root": "objeto",
    "nodeStyle": "texto", "nullLeaves": "sí/no", "packed": "sí/no", "triple": "lista", "critical": "lista",
    "meltingSlope": "número", "normalMelting": "número", "normalBoiling": "número",
    "normalSublimation": "número", "cpSolid": "número", "cpLiquid": "número", "cpGas": "número",
    "melting": "número", "boiling": "número", "hFusion": "número", "hVaporization": "número",
}


def tipo_por_nombre(clave: str) -> str:
    if clave in TIPOS_GENERICOS:
        return TIPOS_GENERICOS[clave]
    if re.match(r"^(blank|show|draw)[A-Z]", clave):
        return "sí/no"
    return ""


NOMBRE_CLASE = {"number": "número", "string": "texto", "boolean": "sí/no", "array": "lista", "object": "objeto", "null": "vacío"}


def clase(v) -> str:
    if v is None:
        return "null"
    if isinstance(v, bool):
        return "boolean"
    if isinstance(v, (int, float)):
        return "number"
    if isinstance(v, list):
        return "array"
    if isinstance(v, dict):
        return "object"
    return "string"


def tipos_valor(clave: str, ejemplos: list[dict]) -> str:
    clases = []
    for b in ejemplos:
        if clave in b:
            k = clase(b[clave])
            if k not in clases:
                clases.append(k)
    return " o ".join(NOMBRE_CLASE[k] for k in clases)


def cargar_datos() -> tuple[list, dict]:
    areas = yaml.safe_load((DATOS / "areas.yaml").read_text(encoding="utf-8"))["areas"]
    tipos: dict = {}
    for f in sorted(DATOS.glob("*.yaml")):
        if f.name == "areas.yaml":
            continue
        for k, v in (yaml.safe_load(f.read_text(encoding="utf-8")) or {}).items():
            if k in tipos:
                sys.exit(f"{f.name}: el tipo {k} está documentado dos veces")
            tipos[k] = v
    return areas, tipos


def ruta_tipo(areas, tipo: str) -> tuple[dict, dict]:
    for a in areas:
        for s in a["subareas"]:
            if tipo in s["tipos"]:
                return a, s
    raise KeyError(tipo)


def celda(t: str) -> str:
    return str(t).replace("|", "\\|").replace("\n", " ")


def pagina_tipo(areas, tipo: str, d: dict, codigo: dict, ejemplos: list, svgs: dict, archivos: dict) -> str:
    a, s = ruta_tipo(areas, tipo)
    hermanos = [t for t in s["tipos"] if t != tipo]
    bloques = [b for _, _, b in ejemplos]
    esq = codigo.get(tipo, {"enums": {}, "defs": {}})
    p = [f"[Manual](../../README.md) › [{a['nombre']}](../README.md) › [{s['nombre']}](README.md)", ""]
    p.append(f"# {d['nombre']} (`{tipo}`)")
    p.append("")
    p.append(d["resumen"].strip())
    p.append("")
    if d.get("calcula"):
        p.append("## Qué calcula")
        p.append("")
        p += [f"- {c}" for c in d["calcula"]]
        p.append("")
    if d.get("variantes"):
        p.append("## Variantes")
        p.append("")
        p.append("| `variant` | Qué dibuja |")
        p.append("| :--- | :--- |")
        p += [f"| `{k}` | {celda(v)} |" for k, v in d["variantes"].items()]
        p.append("")
    p.append("## Campos")
    p.append("")
    p.append("| Campo | Tipo | Valores y valor por defecto | Descripción |")
    p.append("| :--- | :--- | :--- | :--- |")
    p.append(f"| `type` | texto | `{tipo}` | Obligatorio. |")
    for clave, desc in d["campos"].items():
        raiz = clave.split(".")[0].split("[")[0]
        enum = esq["enums"].get(clave) if clave == raiz else None
        defecto = esq["defs"].get(clave) if clave == raiz else None
        valores = ", ".join(f"`{v}`" for v in enum) if enum else ""
        if defecto is not None:
            valores += (" · " if valores else "") + f"por defecto `{defecto}`"
        tv = (d.get("tipos", {}).get(clave) or tipos_valor(clave, bloques) or esq.get("clases", {}).get(clave, "")) if clave == raiz else ""
        if not tv and clave == raiz:
            tv = "texto" if enum else tipo_por_nombre(clave)
        p.append(f"| `{clave}` | {tv} | {valores} | {celda(desc)} |")
    p.append("")
    p.append("Además, los campos comunes a todos los tipos: `title`, `subtitle`, `id`, `inkSaver` y `theme` "
             "(ver [Campos comunes](../../campos-comunes.md)).")
    p.append("")
    blanks = [c for c in d["campos"] if c.startswith("blank") or c == "blank"]
    if blanks or d.get("ejercicio"):
        p.append("## Modo ejercicio")
        p.append("")
        if d.get("ejercicio"):
            p.append(d["ejercicio"].strip())
            p.append("")
        if blanks:
            p.append("Campos que dejan casilleros para completar: " + ", ".join(f"`{b}`" for b in blanks) + ".")
            p.append("")
    p.append("## Ejemplos")
    p.append("")
    for ident, cuerpo, bloque in ejemplos:
        titulo = bloque.get("title") or ident
        p.append(f"### {titulo}")
        p.append("")
        p.append("```yaml")
        p.append(cuerpo)
        p.append("```")
        p.append("")
        nombre = f"img/{ident}.svg"
        archivos[nombre] = svgs[ident]
        p.append(f"![{celda(titulo)}]({nombre})")
        p.append("")
    if d.get("notas"):
        p.append("## Notas")
        p.append("")
        p += [f"- {n.strip()}" for n in d["notas"]]
        p.append("")
    if hermanos:
        p.append("## Ver también")
        p.append("")
        p.append(", ".join(f"[`{h}`]({h}.md)" for h in hermanos))
        p.append("")
    return "\n".join(p)


def pagina_subarea(a, s, tipos) -> str:
    p = [f"[Manual](../../README.md) › [{a['nombre']}](../README.md)", "", f"# {s['nombre']}", "", s["descripcion"].strip(), ""]
    p.append("| Tipo | Diagrama | Para qué |")
    p.append("| :--- | :--- | :--- |")
    for t in s["tipos"]:
        p.append(f"| [`{t}`]({t}.md) | {tipos[t]['nombre']} | {celda(tipos[t]['para'])} |")
    p.append("")
    return "\n".join(p)


def crates_de(texto: str) -> str:
    nombres = [f"`{c.strip()}`" for c in texto.split(" y ")]
    return ("Crates: " + " y ".join(nombres) if len(nombres) > 1 else f"Crate: {nombres[0]}") + "."


def pagina_area(a, tipos) -> str:
    n = sum(len(s["tipos"]) for s in a["subareas"])
    p = ["[Manual](../README.md)", "", f"# {a['nombre']}", "", a["descripcion"].strip(), "",
         f"{n} tipos, en {len(a['subareas'])} grupos. " + crates_de(a["crate"]), ""]
    for s in a["subareas"]:
        p.append(f"## [{s['nombre']}]({s['clave']}/README.md)")
        p.append("")
        p.append(s["descripcion"].strip())
        p.append("")
        for t in s["tipos"]:
            p.append(f"- [`{t}`]({s['clave']}/{t}.md): {tipos[t]['nombre']}. {tipos[t]['para']}")
        p.append("")
    return "\n".join(p)


def indice(areas, tipos) -> str:
    total = sum(len(s["tipos"]) for a in areas for s in a["subareas"])
    p = ["# Manual de dibu", "",
         f"dibu dibuja **{total} tipos** de diagramas educativos como SVG a partir de bloques YAML o JSON. "
         "Cada tipo parte de los datos del problema y calcula lo que dibuja, así que el mismo bloque sirve "
         "para el apunte, el ejercicio (con los casilleros en blanco) y la clave.", "",
         "## Primeros pasos", "",
         "- [Formato de los bloques](bloques.md): cómo se escribe un bloque, en YAML, JSON o Markdown.",
         "- [Campos comunes](campos-comunes.md): `title`, `subtitle`, `id`, `inkSaver` y `theme`.",
         "- [Modo ejercicio](ejercicios.md): `blank` y los campos `blank*`.",
         "- [Themes](themes.md): presets, themes propios y tipografía.",
         "- [Línea de comandos](cli.md): `render`, `batch`, `sync-md`, `list-types` y `list-themes`.", "",
         "## Tipos de diagramas", ""]
    for a in areas:
        p.append(f"### [{a['nombre']}]({a['clave']}/README.md)")
        p.append("")
        for s in a["subareas"]:
            enlaces = ", ".join(f"[`{t}`]({a['clave']}/{s['clave']}/{t}.md)" for t in s["tipos"])
            p.append(f"- **[{s['nombre']}]({a['clave']}/{s['clave']}/README.md)**: {enlaces}")
        p.append("")
    p.append("## Índice alfabético")
    p.append("")
    filas = []
    for a in areas:
        for s in a["subareas"]:
            for t in s["tipos"]:
                filas.append((t, f"| [`{t}`]({a['clave']}/{s['clave']}/{t}.md) | {tipos[t]['nombre']} | {a['nombre']} › {s['nombre']} |"))
    p.append("| Tipo | Diagrama | Área |")
    p.append("| :--- | :--- | :--- |")
    p += [f for _, f in sorted(filas)]
    p.append("")
    p.append("---")
    p.append("")
    p.append("Este manual se genera con `python3 docs/manual/generar.py` a partir de `docs/manual/datos/`, "
             "del código y de los ejemplos de `fixtures/kitchen_sink/kitchen_sink.yaml`; no se edita a mano.")
    p.append("")
    return "\n".join(p)


def generar() -> tuple[dict[str, str], list[str]]:
    """{ruta relativa: contenido} y la lista de problemas encontrados."""
    areas, tipos = cargar_datos()
    codigo = esquema_codigo()
    ejemplos = ejemplos_kitchen_sink()
    problemas: list[str] = []

    registrados = set(codigo)
    en_areas = [t for a in areas for s in a["subareas"] for t in s["tipos"]]
    for t in sorted(registrados - set(en_areas)):
        problemas.append(f"el tipo {t} no está en ninguna subárea de areas.yaml")
    for t in sorted(set(en_areas) - registrados):
        problemas.append(f"areas.yaml nombra {t}, que dibu no registra")
    for t in en_areas:
        if t not in tipos:
            problemas.append(f"falta la descripción de {t} en datos/")
            continue
        d = tipos[t]
        for campo in ("nombre", "para", "resumen", "campos"):
            if not d.get(campo):
                problemas.append(f"{t}: falta '{campo}'")
        documentados = {c.split(".")[0].split("[")[0] for c in d.get("campos", {})}
        ignorar = set(d.get("ignorar", [])) | set(COMUNES)
        for c in codigo.get(t, {}).get("claves", []):
            if c not in documentados and c not in ignorar:
                problemas.append(f"{t}: el código lee '{c}', que no está documentado")
        for c in documentados - set(codigo.get(t, {}).get("claves", [])) - set(d.get("anidados", [])):
            if not any(c in b for _, _, b in ejemplos.get(t, [])) and c not in d.get("sin_ejemplo", []):
                problemas.append(f"{t}: documenta '{c}', que el código no lee ni aparece en los ejemplos")
        if not ejemplos.get(t):
            problemas.append(f"{t}: no hay ejemplos en el kitchen sink")
    if problemas:
        return {}, problemas

    pendientes = [(i, b) for t in en_areas for i, _, b in ejemplos[t]]
    svgs = renderizar(pendientes)
    salida: dict[str, str] = {"README.md": indice(areas, tipos)}
    for a in areas:
        salida[f"{a['clave']}/README.md"] = pagina_area(a, tipos)
        for s in a["subareas"]:
            base = f"{a['clave']}/{s['clave']}"
            salida[f"{base}/README.md"] = pagina_subarea(a, s, tipos)
            for t in s["tipos"]:
                archivos: dict[str, str] = {}
                salida[f"{base}/{t}.md"] = pagina_tipo(areas, t, tipos[t], codigo, ejemplos[t], svgs, archivos)
                for nombre, svg in archivos.items():
                    salida[f"{base}/{nombre}"] = svg
    for f in sorted((DATOS / "paginas").glob("*.md")):
        salida[f.name] = f.read_text(encoding="utf-8")
    return salida, problemas


def main() -> int:
    ap = argparse.ArgumentParser(description=__doc__.split("\n")[0])
    ap.add_argument("--verificar", action="store_true", help="no escribe: falla si algo falta o está desactualizado")
    args = ap.parse_args()
    salida, problemas = generar()
    if problemas:
        print("✗ Falta documentar:", *problemas, sep="\n  ", file=sys.stderr)
        return 1
    # Lo generado: todo lo de docs/manual salvo datos/ y este script.
    existentes = {str(p.relative_to(MANUAL)) for p in MANUAL.rglob("*")
                  if p.is_file() and not str(p.relative_to(MANUAL)).startswith("datos/") and p.name != "generar.py"
                  and "__pycache__" not in str(p)}
    distintos = [r for r, c in salida.items() if not (MANUAL / r).is_file() or (MANUAL / r).read_text(encoding="utf-8") != c]
    sobrantes = sorted(existentes - set(salida))
    if args.verificar:
        if distintos or sobrantes:
            print("✗ El manual está desactualizado (correr python3 docs/manual/generar.py):", file=sys.stderr)
            for r in (distintos + sobrantes)[:20]:
                print(f"  {r}", file=sys.stderr)
            if distintos and (MANUAL / distintos[0]).is_file():
                import difflib
                viejo = (MANUAL / distintos[0]).read_text(encoding="utf-8").replace(">", ">\n").splitlines()
                nuevo = salida[distintos[0]].replace(">", ">\n").splitlines()
                diff = list(difflib.unified_diff(viejo, nuevo, "versionado", "generado", n=1, lineterm=""))
                print(f"Diferencias en {distintos[0]}:", *diff[:30], sep="\n", file=sys.stderr)
            return 1
        print(f"✓ Manual al día: {len(salida)} archivos")
        return 0
    for r in distintos:
        destino = MANUAL / r
        destino.parent.mkdir(parents=True, exist_ok=True)
        destino.write_text(salida[r], encoding="utf-8")
    for r in sobrantes:
        (MANUAL / r).unlink()
    paginas = sum(1 for r in salida if r.endswith(".md"))
    print(f"✓ Manual: {paginas} páginas, {len(salida) - paginas} imágenes ({len(distintos)} archivos cambiados, {len(sobrantes)} borrados)")
    return 0


if __name__ == "__main__":
    sys.exit(main())
