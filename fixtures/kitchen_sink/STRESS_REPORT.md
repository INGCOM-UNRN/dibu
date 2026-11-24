# Reporte de Stress Test y Validación Integral — `dibu` (M4 Fase 4)

- **Fecha UTC:** `2026-11-24T01:25:55Z`
- **Binario evaluado:** `/home/mrtin/dev/tools/docente/contenidos/dibu/target/release/dibu`
- **Entorno:** `3.11.13` / `Linux-7.2.13-200.fc44.x86_64-x86_64-with-glibc2.43` (`seed=42`, `N=1200`)

---

## 1. Catálogo Kitchen Sink (`fixtures/kitchen_sink/`)

| Artefacto | Bloques / Fences | Cobertura | Paridad vs `alucarD` | Verificación CLI |
|---|---:|---:|---:|---|
| `kitchen_sink.yaml` | 85 bloques | 44/44 tipos | 100% | `dibu render` + `dibu batch` OK |
| `kitchen_sink.md` | 23 fences (23 SVGs) | 44/44 tipos | 100% | `dibu sync-md --verificar` (`rc=0`) |

---

## 2. Pipeline HTML y Typst/PDF + Hallazgo en `typst_markup.py`

- `markdown_a_html_oraculo(kitchen_sink.md)` embebió `48` elementos `<svg>` sin errores.
- Bloque inválido en HTML (`type: grafo`) emitió `<p class="estructura-error">` sin `<svg>` (`OK=True`).
- Compilación Typst/PDF verificada (`pdf1_alucard=1038465 B`, `pdf2_dibu_batch=1005263 B`, `pdf3_sync_md_image=686160 B` sobre `23` SVGs).
- **Hallazgo documentado (solo lectura en `alucarD`)**: `_SVG` en `alucarD/generador_examenes/generators/typst_markup.py` usa un cuantificador no codicioso (`.*?`) que corta en el primer `</svg>` interno si se le pasa HTML con SVGs apilados anidados de `dibu sync-md` (`<svg><svg>...</svg></svg>`). En los tres flujos reales del sistema (`markdown_a_html_oraculo` con bloques independientes, SVGs individuales de `dibu batch` en HTML y archivos de `dibu sync-md` referenciados por `#image(...)` en Typst) la compilación a PDF pasa al 100 %.

---

## 3. Validación de `dibu sync-md` sobre casos de `alucarD/tests/test_diagramas.py`

- Contrato SHA-1 `nombre_svg`: `True`
- Idempotencia (`sync-md` + `--verificar`): `True`
- Mutación `type2: x` y re-verificación: `True`
- Limpieza de huérfanos y directorio `diagramas/` vacío: `True`
- Preservación de archivos ajenos: `True`
- Manejo de bloque inválido `type: grafo` (`rc=1`, `"grafo"` en stderr, `0` panics): `True`

---

## 4. Benchmark Comparativo (`N=1200` bloques mixtos, 44/44 tipos, ambas paletas)

Se incluyó sobremuestreo intencional de casos pesados (`time_response` con retardo/PI, `bode`, `nyquist`, `root_locus`, `titration` poliprótica `H3PO4`, `kinetics` reversible, `tree` AVL de 25 nodos, `matrix` `10×10`, `call_tree` profundo y `flowchart` doblemente anidado). La paridad estructural y semántica de los `1200` SVGs generados por `dibu batch` frente a Python fue del **100.0 %**.

| Motor / Modo | Wall-clock (ms) | ms / bloque | Throughput (blq/s) | p50 / p95 / p99 (ms) | RSS pico (KiB) | Speedup vs Python |
|---|---:|---:|---:|---|---:|---:|
| **Python (`alucarD`) in-process** | 22854.97 | 19.046 | 52.5 | 0.389 / 63.613 / 355.828 | 58200 | 1.00× |
| **`dibu batch`** (1 spawn JSONL) | 1380.30 | 1.150 | 869.4 | — | 6376 | **16.56×** |
| **`dibu render` multi-bloque** (1 spawn archivo JSON) | 1434.07 | 1.195 | 836.8 | — | 16032 | **15.94×** |
| **`dibu render` por proceso** (100 spawns estratificados) | 229.29 *(muestra)* | 2.293 *(por spawn)* | 436.1 *(spawns/s)* | 1.570 / 5.146 / 20.901 | 5584 | Proyección `N=1200`: `2751.45` ms |

> Nota: `dibu render` proceso-por-proceso paga el arranque de proceso y escritura a disco por cada bloque individual, mientras que `dibu render` multi-bloque y `dibu batch` amortizan ese costo en un único proceso (`batch` además evita I/O de archivos por bloque al operar sobre JSONL en memoria).

---

## 5. Determinismo Byte-a-Byte (`100 %` SHA-256)

- **`dibu batch` (1200 bloques, 2 corridas):** SHA-256 idéntico de `stdout` (`19740bd8f5f4c0bfc37e27ca64eddb44241552457314cfd292fc8c55a32e475d`) y de los `1200` SVGs individuales.
- **`dibu render` (`kitchen_sink.yaml`, 2 corridas):** `85/85` archivos SVG idénticos byte a byte.
- **`dibu sync-md` (`kitchen_sink.md`, 2 tempdirs):** `23/23` archivos `estructura-*.svg` con nombres y SHA-256 idénticos.

---

## 6. Resiliencia ante Entradas Malformadas (`0` Panics)

- **Probes totales ejecutados:** `129` (cubriendo los `44/44` tipos sin exclusiones).
- **Errores controlados capturados:** `101` (incluye `grafcet` con `or/and: []`, `time_response` con `delay: 1e308` / negativo / `NaN`, YAML/JSON roto, bytes no UTF-8, floats extremos, strings de 10 KB con XML `&<>"'` y emojis, matrices `100×100`, arrays de `1000` y árboles de `500` nodos).
- **Panics (`rc=101`, `SIGABRT`, `panicked at`, `RUST_BACKTRACE`):** **`0`**.
