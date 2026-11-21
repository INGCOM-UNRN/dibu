# Infraestructura de Pruebas E2E: `dibu`

## Filosofía y Metodología

- **Enfoque opaque-box**: basado en requerimientos (`ORIGINAL_REQUEST.md`, `PROJECT.md`) y en el oráculo de referencia (`alucarD`), invocando el binario `dibu` por `subprocess.run` sin depender de internals de los crates Rust.
- **Metodología de 4 Tiers**:
  - **Tier 1 (Category-Partition)**: cobertura funcional por partición de dominio y variantes (>= 5 casos por cada una de las 48 features).
  - **Tier 2 (Boundary Value Analysis & Error Handling)**: valores límite, flags `blank*`, `inkSaver`, UTF-8/escape XML y errores controlados (0 panics).
  - **Tier 3 (Pairwise Combinatorial Testing)**: combinaciones por pares cross-feature entre los 3 dominios (`control`, `chem`, `cs`), paletas (`tinta: true/false`, `inkSaver` override), flags docentes/visibilidad y composición CLI (`list-types`, `render`, `batch`, `sync-md`).
  - **Tier 4 (Real-World Application Workloads)**: 5 escenarios Markdown realistas ejecutando el flujo completo `sync-md` (`--color`, `--verificar`) -> `render` -> `batch` (determinismo byte-a-byte y resiliencia) -> HTML -> Typst -> PDF.
- **Oráculo dual**:
  1. **Contra binario `dibu`**: verifica contratos CLI, códigos de salida, XML/SVG válido, atributos `<svg>`, paletas (`tinta` vs `--color`), determinismo byte-a-byte y errores controlados.
  2. **Contra oráculo `alucarD` (`PYTHONDONTWRITEBYTECODE=1`, modo solo lectura)**: valida consistencia de todos los casos y paridad estructural/semántica (`comparar_con_oraculo`: dimensiones, multiset de `<text>`, cantidad de sub-`<svg>` y primitivas gráficas).

## Comandos de Ejecución

Ejecutar siempre con `PYTHONDONTWRITEBYTECODE=1` y el intérprete Python de `alucarD/.venv`:

```bash
PY=/home/mrtin/dev/tools/docente/contenidos/alucarD/.venv/bin/python
RUNNER=/home/mrtin/dev/tools/docente/contenidos/dibu/tests/e2e/run_e2e.py

# 1. Validación completa contra oráculo Python (sin requerir binario compilado)
PYTHONDONTWRITEBYTECODE=1 $PY $RUNNER --oracle-only --suite all

# 2. Validación contra binario debug sobre features compiladas (omite pendientes)
PYTHONDONTWRITEBYTECODE=1 $PY $RUNNER --dibu-bin /home/mrtin/dev/tools/docente/contenidos/dibu/target/debug/dibu --suite all --solo-implementados

# 3. Suites estrictas por dominio ya implementado
PYTHONDONTWRITEBYTECODE=1 $PY $RUNNER --dibu-bin /home/mrtin/dev/tools/docente/contenidos/dibu/target/debug/dibu --suite control
PYTHONDONTWRITEBYTECODE=1 $PY $RUNNER --dibu-bin /home/mrtin/dev/tools/docente/contenidos/dibu/target/debug/dibu --suite cs

# 4. Modo estricto completo (exige los 44 tipos en `dibu list-types` y 0 FAIL/0 SKIP en todas las suites)
PYTHONDONTWRITEBYTECODE=1 $PY $RUNNER
```

### Flags de `run_e2e.py`

- `--tier {1,2,3,4,all}`: filtra la ejecución por tier metodológico (default: `all`).
- `--suite {cli,control,chem,cs,tier3,tier4,all}`: selecciona una suite o todas (default: `all`).
- `--solo-implementados`: ejecuta contra `dibu` las features presentes en `dibu list-types` y marca `SKIP` documentado en tipos pendientes.
- `--oracle-only`: valida casos y pipelines contra `oracle.py` sin invocar al binario `dibu`.
- `--dibu-bin <path>`: ruta explícita al binario `dibu` (si se omite, busca `DIBU_BIN`, `target/release/dibu`, `target/debug/dibu` o compila `dibu-cli`).
- `--json-report <path>`: exporta resumen y fallos/skips en formato JSON.
- `--verbose`: imprime el resultado de cada caso individual.

## Arquitectura de Archivos

```text
tests/e2e/
├── README.md
├── __init__.py
├── harness.py                       # Invocación CLI/batch, validación XML/SVG, unicidad de IDs, comparación dual
├── oracle.py                        # Adaptador read-only sobre alucarD (render, sync-md, html, typst, pdf)
├── run_e2e.py                       # Runner CLI unificado y reporte JSON
├── cases/
│   ├── cli.yaml                     # 40 casos (4 subcomandos CLI × 5 T1 + 5 T2)
│   ├── control.yaml                 # 90 casos (9 tipos Control × 5 T1 + 5 T2)
│   ├── chem_general.yaml            # 162 casos (13 tipos Química General)
│   ├── chem_organica.yaml           # 36 casos (3 tipos Química Orgánica)
│   ├── chem_inorganica.yaml         # 56 casos (5 tipos Química Inorgánica)
│   ├── cs_lineal.yaml               # 58 casos (5 tipos CS Lineales)
│   ├── cs_arboles_algos.yaml        # 102 casos (9 tipos CS Árboles y Algoritmos)
│   ├── tier3_combinaciones.yaml     # 50 combinaciones pairwise cross-feature (44 tipos)
│   └── tier4_escenarios/            # 5 documentos Markdown reales
│       ├── examen_control.md
│       ├── examen_quimica.md
│       ├── examen_algoritmos.md
│       ├── guia_integradora_mixta.md
│       └── parcial_errores_y_variantes.md
└── suites/
    ├── __init__.py
    ├── test_cli.py                  # Suite CLI (40 casos)
    ├── test_control.py              # Suite Control (90 casos)
    ├── test_chem.py                 # Suite Chem (254 casos)
    ├── test_cs.py                   # Suite CS (160 casos + validadores XML sangría/clipping)
    ├── test_tier3.py                # Suite Tier 3 (50 YAML + 6 composición CLI = 56 casos)
    └── test_tier4.py                # Suite Tier 4 (5 escenarios end-to-end)
```

## Pipeline Documental Tier 4 y Comportamiento de `typst`

En cada escenario de Tier 4 (`test_tier4.py`), los documentos Markdown se copian a un directorio temporal aislado bajo `ctx.work_dir` y recorren:

1. `dibu sync-md <dir>` (`--color` en `examen_control.md`, `tinta` por defecto en el resto) -> verifica `rc=1` al generar, archivos `diagramas/estructura-<sha1[:12]>.svg`, paridad estructural/semántica frente a `oracle.renderizar_bloque_md_oraculo`, y segunda pasada con `--verificar` (`rc=0`).
2. `dibu render` y `dibu batch` sobre los bloques extraídos -> verifica determinismo byte-a-byte entre corridas consecutivas y resiliencia ante bloque inválido mezclado en `parcial_errores_y_variantes.md`.
3. Conversión documental vía `oracle.markdown_a_html_oraculo` (verifica `<svg` embebidos) -> `oracle.html_a_typst_oraculo` (verifica fuente Typst no vacía) -> verificación `shutil.which("typst")`:
   - **Cuando `typst` está disponible en `PATH`** (ej. `/usr/bin/typst`): ejecuta `oracle.compilar_typst_oraculo` y valida que el PDF generado sea no vacío y comience con la cabecera `%PDF-`.
   - **Cuando `typst` no está instalado** (`shutil.which("typst") is None`): registra un fallback graceful documentado en `detalles["typst_fallback"]` validando HTML + fuente Typst sin simular éxito falso de PDF.

## Inventario de 48 Features y Conteos por Tier

| # | Feature | Dominio | Tier 1 | Tier 2 | Tier 3 | Tier 4 | Total |
|---|---------|---------|-------:|-------:|-------:|-------:|------:|
| 1 | `cli:list-types` | CLI | 5 | 5 | 1 | 0 | 11 |
| 2 | `cli:render` | CLI | 5 | 5 | 1 | 0 | 11 |
| 3 | `cli:batch` | CLI | 5 | 5 | 2 | 0 | 12 |
| 4 | `cli:sync-md` | CLI | 5 | 5 | 2 | 5 | 17 |
| 5 | `block_diagram` | Control | 5 | 5 | 2 | 0 | 12 |
| 6 | `time_response` | Control | 5 | 5 | 1 | 0 | 11 |
| 7 | `bode` | Control | 5 | 5 | 2 | 0 | 12 |
| 8 | `nyquist` | Control | 5 | 5 | 1 | 0 | 11 |
| 9 | `pole_zero` | Control | 5 | 5 | 1 | 0 | 11 |
| 10 | `root_locus` | Control | 5 | 5 | 1 | 0 | 11 |
| 11 | `instrumentation` | Control | 5 | 5 | 1 | 0 | 11 |
| 12 | `ladder` | Control | 5 | 5 | 1 | 0 | 11 |
| 13 | `grafcet` | Control | 5 | 5 | 1 | 0 | 11 |
| 14 | `molecule` | Chem | 6 | 7 | 1 | 0 | 14 |
| 15 | `reaction` | Chem | 5 | 6 | 1 | 0 | 12 |
| 16 | `vsepr` | Chem | 5 | 7 | 1 | 0 | 13 |
| 17 | `orbitals` | Chem | 9 | 7 | 1 | 0 | 17 |
| 18 | `energy_profile` | Chem | 5 | 6 | 1 | 0 | 12 |
| 19 | `titration` | Chem | 6 | 7 | 2 | 0 | 15 |
| 20 | `speciation` | Chem | 5 | 6 | 1 | 0 | 12 |
| 21 | `kinetics` | Chem | 6 | 8 | 1 | 0 | 15 |
| 22 | `phase_diagram` | Chem | 5 | 8 | 1 | 0 | 14 |
| 23 | `heating_curve` | Chem | 5 | 7 | 1 | 0 | 13 |
| 24 | `spectrum` | Chem | 5 | 7 | 1 | 0 | 13 |
| 25 | `galvanic_cell` | Chem | 5 | 7 | 1 | 0 | 13 |
| 26 | `unit_cell` | Chem | 6 | 6 | 1 | 0 | 13 |
| 27 | `newman` | Chem | 5 | 6 | 1 | 0 | 12 |
| 28 | `chair` | Chem | 5 | 7 | 1 | 0 | 13 |
| 29 | `fischer` | Chem | 5 | 8 | 1 | 0 | 14 |
| 30 | `crystal_field` | Chem | 5 | 7 | 1 | 0 | 13 |
| 31 | `complex` | Chem | 7 | 6 | 1 | 0 | 14 |
| 32 | `latimer` | Chem | 5 | 5 | 1 | 0 | 11 |
| 33 | `frost` | Chem | 5 | 5 | 1 | 0 | 11 |
| 34 | `born_haber` | Chem | 5 | 6 | 2 | 0 | 13 |
| 35 | `array` | CS | 5 | 6 | 1 | 0 | 12 |
| 36 | `matrix` | CS | 6 | 9 | 1 | 0 | 16 |
| 37 | `linked_list` | CS | 6 | 6 | 1 | 0 | 13 |
| 38 | `stack` | CS | 5 | 5 | 1 | 0 | 11 |
| 39 | `call_stack` | CS | 5 | 6 | 1 | 0 | 12 |
| 40 | `call_tree` | CS | 5 | 6 | 1 | 0 | 12 |
| 41 | `queue` | CS | 5 | 5 | 1 | 0 | 11 |
| 42 | `tree` | CS | 5 | 6 | 2 | 0 | 13 |
| 43 | `struct` | CS | 5 | 6 | 2 | 0 | 13 |
| 44 | `adt` | CS | 5 | 6 | 1 | 0 | 12 |
| 45 | `complexity` | CS | 5 | 8 | 1 | 0 | 14 |
| 46 | `flowchart` | CS | 5 | 6 | 1 | 0 | 12 |
| 47 | `cost_table` | CS | 5 | 6 | 1 | 0 | 12 |
| 48 | `trace_table` | CS | 5 | 7 | 1 | 0 | 13 |
| **Total** | **48 features** | **Todos** | **252** | **292** | **56** | **5** | **605** |
