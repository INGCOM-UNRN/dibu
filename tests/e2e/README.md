# Suite E2E opaque-box de `dibu`

Esta carpeta contiene la infraestructura y las suites de pruebas End-to-End (E2E) *opaque-box* de `dibu`, validadas en espejo contra el repositorio de referencia `alucarD` en modo estrictamente solo lectura.

## Arquitectura

- **`oracle.py`**: Adaptador de solo lectura sobre `/home/mrtin/dev/tools/docente/contenidos/alucarD` (`sys.dont_write_bytecode = True`). Expone `TIPOS_CONTROL` (9), `TIPOS_CHEM` (21), `TIPOS_CS` (14), `TODOS_LOS_TIPOS` (44), `renderizar_oraculo`, `renderizar_bloque_md_oraculo`, `nombre_svg_oraculo`, `sincronizar_svgs_oraculo`, `markdown_a_html_oraculo`, `html_a_typst_oraculo` y `compilar_typst_oraculo`.
- **`harness.py`**: Descubrimiento de binario (`resolver_dibu_bin`), invocación *opaque-box* por `subprocess.run` (`invocar_dibu_list_types`, `invocar_dibu_render`, `invocar_dibu_batch`, `invocar_dibu_sync_md`), validación XML/SVG con `xml.etree.ElementTree`, extracción de textos normalizando `\xa0` y `<tspan dx=...>`, verificación de determinismo byte-a-byte, detección de panics/tracebacks y comparación estructural/semántica (`comparar_con_oraculo`).
- **`run_e2e.py`**: Runner CLI único con resumen por suite, código de salida `0`/`1` y exportación opcional a JSON (`--json-report`).
- **`cases/`**: Casos declarativos YAML y escenarios Markdown reales organizados por dominio:
  - `cli.yaml`: 40 casos (5 Tier 1 + 5 Tier 2 por cada subcomando: `list-types`, `render`, `batch`, `sync-md`).
  - `control.yaml`: 90 casos (5 Tier 1 + 5 Tier 2 por cada uno de los 9 tipos de control).
  - M3.2 / M3.3 / M3.4 sumarán `chem_*.yaml`, `cs_*.yaml`, `tier3_combinaciones.yaml` y `tier4_escenarios/`.
- **`suites/`**: Módulos ejecutables (`test_cli.py`, `test_control.py`, y próximos `test_chem.py`, `test_cs.py`, `test_tier3.py`, `test_tier4.py`).

## Cómo ejecutar las pruebas

Usá siempre `PYTHONDONTWRITEBYTECODE=1` y el Python del entorno de referencia (que incluye `PyYAML`, `Markdown`, `typst`, etc. sin generar `__pycache__`):

```bash
PY=/home/mrtin/dev/tools/docente/contenidos/alucarD/.venv/bin/python
RUNNER=/home/mrtin/dev/tools/docente/contenidos/dibu/tests/e2e/run_e2e.py

# 1. Suite CLI tolerando pendientes de M1/M2
PYTHONDONTWRITEBYTECODE=1 $PY $RUNNER --suite cli --solo-implementados --verbose

# 2. Suite Control en modo estricto (90 casos, 100 % paridad dibu vs alucarD)
PYTHONDONTWRITEBYTECODE=1 $PY $RUNNER --suite control --verbose

# 3. Todas las suites disponibles sobre tipos ya implementados + reporte JSON
PYTHONDONTWRITEBYTECODE=1 $PY $RUNNER --suite all --solo-implementados --json-report /tmp/e2e_report.json

# 4. Validación completa solo contra el oráculo Python (sin requerir binario dibu)
PYTHONDONTWRITEBYTECODE=1 $PY $RUNNER --oracle-only --suite all

# 5. Modo estricto completo (exige los 44 tipos en `list-types`, fix M2 y las 6 suites; sale con rc=1 hasta completar M1–M4)
PYTHONDONTWRITEBYTECODE=1 $PY $RUNNER
```

### Flags de `run_e2e.py`

- `--tier {1,2,3,4,all}`: filtra por tier metodológico (default `all`).
- `--suite {cli,control,chem,cs,tier3,tier4,all}`: selecciona la suite a correr (default `all`).
- `--solo-implementados`: evalúa `dibu` sobre los tipos presentes en `dibu list-types` y marca `SKIP` documentado para tipos pendientes o `known_issue_m2: true` si todavía fallan.
- `--oracle-only`: valida casos y suites disponibles contra `oracle.py` sin invocar `dibu`.
- `--dibu-bin PATH`: fuerza la ruta del binario `dibu` (si no se pasa, busca `DIBU_BIN`, `target/release/dibu`, `target/debug/dibu`, o compila con `cargo build -p dibu-cli`).
- `--json-report PATH`: guarda el reporte estructurado en JSON.
- `--verbose`: imprime el estado individual de cada caso (`PASS`, `SKIP`, `FAIL`).

## Contrato YAML y API para M3.2, M3.3 y M3.4

### Esquema YAML para diagramas (`cases/*.yaml`)

Cada archivo YAML de dominio contiene una lista de casos con la siguiente estructura:

```yaml
- id: bode_t1_asintotas
  feature: bode
  tier: 1                     # 1 (Category-Partition), 2 (Boundary/Error), 3 (Pairwise)
  description: 'Descripción breve del caso'
  tinta: true                 # opcional, default true (false equivale a --color)
  block:
    type: bode
    system: {k: 10, poles: [0, -1, -10]}
    margins: true
  expect:
    ok: true                  # true o false
    min_width: 200            # opcional, default 1
    min_height: 150           # opcional, default 1
    text_contains: ["MG ="]   # opcional
    text_absent: []           # opcional
    svg_contains: []          # opcional
    svg_absent: []            # opcional
    compare_oracle: true      # opcional, default true
    compare_texts: true       # opcional, default true (multiset de <text>)
    check_determinism: true   # opcional, default true
    dim_tol: 0                # opcional, default 0 (tolerancia px en width/height)
    error_contains: "..."     # usado cuando ok: false (str o list[str])
```

### Interfaz de módulos `suites/test_*.py`

Cada módulo de suite exporta:

```python
def run_suite(ctx: ContextoE2E) -> list[ResultadoCaso]: ...
```

Para las suites declarativas de diagramas (M3.2 y M3.3), alcanza con delegar a `harness.ejecutar_casos_diagrama([ctx.cases_dir / "<archivo>.yaml"], ctx)`. El harness se encarga automáticamente de:
1. Validar consistencia contra `oracle.renderizar_oraculo`.
2. Respetar `--oracle-only` y `--solo-implementados`.
3. Ejecutar `dibu batch` en lote y en doble corrida para determinismo byte-a-byte.
4. Normalizar claves booleanas de YAML 1.1 (`False -> "no"` en `ladder`) con `normalizar_bloque_json`.
5. Validar XML/SVG, paletas (`tinta_efectiva` respetando `block.inkSaver`), dimensiones y multiset de textos normalizados (`\xa0` -> `' '` y espacio antes de `<tspan dx=...>`).

## Bugs conocidos y divergencias relevadas (M1 / M2 / Hardening)

1. **M2 (`dibu-cli` `sync-md`) — Ruta relativa sin directorio (`parent() == ""`)**:
   - Invocar `dibu sync-md doc.md` (sin `./`) hoy devuelve `rc=0` sin generar `diagramas/` porque `Path::new("doc.md").parent()` retorna `Some("")` y `canonicalize()` falla silenciosamente.
   - En `cases/cli.yaml`, el caso `cli_sync_md_relative_filename_no_parent` lleva `known_issue_m2: true`: marca `SKIP` bajo `--solo-implementados` y `FAIL` en modo estricto hasta que M2 aplique el fix `parent == "" -> "."`.
2. **M1 / M2 (`dibu-chem` y `dibu-cs`) — Crates pendientes de implementación**:
   - Actualmente `dibu list-types` expone los 9 tipos de control. En modo estricto (`--suite all` sin `--solo-implementados`), `cli_list_types_contains_all_44` y el gate global fallan con `rc=1` hasta que los 44 tipos estén registrados.
3. **Hardening pendiente en `dibu-control` (relevado en Step 2B.a, fuera del gate de los 90 casos válidos)**:
   - **`grafcet` panic (`rc=101`)**: `{"type": "grafcet", "steps": [{"or": []}]}` y `{"and": []}` provocan `index out of bounds` en `crates/dibu-control/src/industrial.rs:915:33`; Python responde con `ErrorDiagrama`.
   - **Divergencias de validación en casos borde extremos**:
     - `time_response` con `data: []` o `tEnd < 0` (`dibu` dibuja/auto-ajusta mientras Python rechaza `data: []` o integra `t < 0`).
     - `bode` con `wMin <= 0` (`dibu` descarta valores `<= 0` silenciosamente; Python falla).
     - `nyquist` con polo exacto sobre `jω` (`dibu` filtra `NaN`/`inf`; Python lanza `ZeroDivisionError`).
     - `pole_zero` con `xRange: [0, 0]` (`ZeroDivisionError` en Python vs ejes de ancho 0 en `dibu`), `xRange: [2, 1]` (altura negativa en ambos) y distinto orden de nodos `<text>` en `showValues: true` (cubierto comparando multiset en `comparar_con_oraculo`).
     - `ladder` con `or: []` (`dibu` dibuja rama vacía; Python falla) e `instrumentation` con ítem no-objeto en `lines` (`dibu` lo saltea; Python falla).
     - Errores de `control.py`: Python antepone `{type}: ` y `dibu` emite el mensaje interno sin prefijo (los casos de `control.yaml` validan la subcadena común compartida).
