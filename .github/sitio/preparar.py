#!/usr/bin/env python3
"""Arma el sitio de GitHub Pages del manual a partir de docs/manual (sin los datos ni el generador).

A cada página le agrega el front matter con su título (el primer `# `) y la envuelve en
`{% raw %}` para que Jekyll no tome como Liquid ningún `{{` o `{%` de los ejemplos YAML.

Uso: python3 .github/sitio/preparar.py <destino>
"""
import re
import shutil
import sys
from pathlib import Path

RAIZ = Path(__file__).resolve().parents[2]
ORIGEN = RAIZ / "docs" / "manual"
destino = Path(sys.argv[1] if len(sys.argv) > 1 else "sitio")

if destino.exists():
    shutil.rmtree(destino)
shutil.copytree(ORIGEN, destino, ignore=shutil.ignore_patterns("datos", "generar.py", "__pycache__"))
shutil.copy(Path(__file__).with_name("_config.yml"), destino / "_config.yml")

# Cada README.md pasa a index.md, para que Jekyll genere el index.html de su carpeta sin
# depender de jekyll-readme-index; los enlaces a README.md se reescriben.
for readme in list(destino.rglob("README.md")):
    readme.rename(readme.with_name("index.md"))
ENLACE_README = re.compile(r"\]\(((?:[^()\s]*/)?)README\.md(#[^)]*)?\)")

paginas = 0
for md in destino.rglob("*.md"):
    texto = ENLACE_README.sub(lambda m: f"]({m.group(1)}index.md{m.group(2) or ''})", md.read_text(encoding="utf-8"))
    m = re.search(r"^# (.+)$", texto, re.M)
    titulo = re.sub(r"[`*]", "", m.group(1)).strip() if m else "Manual de dibu"
    titulo = titulo.replace('"', '\\"')
    md.write_text(f'---\ntitle: "{titulo}"\n---\n{{% raw %}}\n{texto}\n{{% endraw %}}\n', encoding="utf-8")
    paginas += 1
print(f"✓ {destino}: {paginas} páginas")
