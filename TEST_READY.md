# Test Suite Ready: `dibu` E2E

## Test Runner

```bash
PYTHONDONTWRITEBYTECODE=1 /home/mrtin/dev/tools/docente/contenidos/alucarD/.venv/bin/python /home/mrtin/dev/tools/docente/contenidos/dibu/tests/e2e/run_e2e.py
```

- **Modo estricto (default)**: invoca el binario `dibu` sobre las 6 suites (`cli`, `control`, `chem`, `cs`, `tier3`, `tier4`) y exige los 44 tipos registrados en `dibu list-types`. Retornará `rc=0` con 100 % PASS en M4 cuando `dibu-chem` + `dibu-cs` + `dibu-control` estén compilados juntos en el binario `dibu`.
- **Modo `--oracle-only`**: valida los 605 casos (590 declarativos/composición + 15 contratos CLI aplicables al oráculo) contra `alucarD` en solo lectura sin requerir el binario `dibu` (`--oracle-only --suite all`).
- **Modo `--solo-implementados`**: valida contra el binario `dibu` actual (`--dibu-bin target/debug/dibu --suite all --solo-implementados`) todas las features ya compiladas (`cli`, `control`, `cs`, combinaciones Tier 3 y escenarios Tier 4 sin `chem`) y marca `SKIP` documentado en tipos aún no registrados en `dibu list-types`.

## Coverage Summary

| Tier | Descripción | Casos | Umbral requerido | Estado |
|---|---|---:|---:|---|
| Tier 1 | Category-Partition (48 features × >=5) | 252 | >= 240 | Completo |
| Tier 2 | Boundary Value Analysis & Error Handling (48 features × >=5) | 292 | >= 240 | Completo |
| Tier 3 | Pairwise Combinatorial Cross-Feature (50 YAML + 6 CLI) | 56 | >= 48 | Completo |
| Tier 4 | Real-World Markdown -> `sync-md` -> HTML -> Typst/PDF | 5 | >= 5 | Completo |
| **Total** | **Suite E2E Completa (`tests/e2e/`)** | **605** | **>= 533** | **Listo** |

Desglose por suite:
- `cli`: 40 casos (20 Tier 1 + 20 Tier 2)
- `control`: 90 casos (45 Tier 1 + 45 Tier 2)
- `chem`: 254 casos (114 Tier 1 + 140 Tier 2)
- `cs`: 160 casos (73 Tier 1 + 87 Tier 2)
- `tier3`: 56 casos (50 pairwise YAML + 6 composición CLI)
- `tier4`: 5 escenarios completos (`examen_control.md`, `examen_quimica.md`, `examen_algoritmos.md`, `guia_integradora_mixta.md`, `parcial_errores_y_variantes.md`)

## Feature Checklist

| # | Feature | Dominio | Tier 1 | Tier 2 | Tier 3 | Tier 4 (en escenarios) | Total directo |
|---|---------|---------|-------:|-------:|-------:|-----------------------:|--------------:|
| 1 | `cli:list-types` | CLI | 5 | 5 | 1 | — | 11 |
| 2 | `cli:render` | CLI | 5 | 5 | 1 | 5 | 11 (+5 T4) |
| 3 | `cli:batch` | CLI | 5 | 5 | 2 | 5 | 12 (+5 T4) |
| 4 | `cli:sync-md` | CLI | 5 | 5 | 2 | 5 | 17 |
| 5 | `block_diagram` | Control | 5 | 5 | 2 | ✓ (`examen_control`, `guia_integradora`, `parcial`) | 12 |
| 6 | `time_response` | Control | 5 | 5 | 1 | ✓ (`examen_control`, `guia_integradora`) | 11 |
| 7 | `bode` | Control | 5 | 5 | 2 | ✓ (`examen_control`, `guia_integradora`) | 12 |
| 8 | `nyquist` | Control | 5 | 5 | 1 | ✓ (`examen_control`, `guia_integradora`) | 11 |
| 9 | `pole_zero` | Control | 5 | 5 | 1 | ✓ (`examen_control`, `guia_integradora`) | 11 |
| 10 | `root_locus` | Control | 5 | 5 | 1 | ✓ (`examen_control`, `guia_integradora`) | 11 |
| 11 | `instrumentation` | Control | 5 | 5 | 1 | ✓ (`examen_control`, `guia_integradora`) | 11 |
| 12 | `ladder` | Control | 5 | 5 | 1 | ✓ (`examen_control`, `guia_integradora`) | 11 |
| 13 | `grafcet` | Control | 5 | 5 | 1 | ✓ (`examen_control`, `guia_integradora`) | 11 |
| 14 | `molecule` | Chem | 6 | 7 | 1 | ✓ (`examen_quimica`, `guia_integradora`) | 14 |
| 15 | `reaction` | Chem | 5 | 6 | 1 | ✓ (`examen_quimica`, `guia_integradora`) | 12 |
| 16 | `vsepr` | Chem | 5 | 7 | 1 | ✓ (`examen_quimica`, `guia_integradora`) | 13 |
| 17 | `orbitals` | Chem | 9 | 7 | 1 | ✓ (`examen_quimica`, `guia_integradora`) | 17 |
| 18 | `energy_profile` | Chem | 5 | 6 | 1 | ✓ (`examen_quimica`, `guia_integradora`) | 12 |
| 19 | `titration` | Chem | 6 | 7 | 2 | ✓ (`examen_quimica`, `guia_integradora`) | 15 |
| 20 | `speciation` | Chem | 5 | 6 | 1 | ✓ (`examen_quimica`, `guia_integradora`) | 12 |
| 21 | `kinetics` | Chem | 6 | 8 | 1 | ✓ (`examen_quimica`, `guia_integradora`) | 15 |
| 22 | `phase_diagram` | Chem | 5 | 8 | 1 | ✓ (`guia_integradora`) | 14 |
| 23 | `heating_curve` | Chem | 5 | 7 | 1 | ✓ (`guia_integradora`) | 13 |
| 24 | `spectrum` | Chem | 5 | 7 | 1 | ✓ (`guia_integradora`) | 13 |
| 25 | `galvanic_cell` | Chem | 5 | 7 | 1 | ✓ (`examen_quimica`, `guia_integradora`) | 13 |
| 26 | `unit_cell` | Chem | 6 | 6 | 1 | ✓ (`guia_integradora`) | 13 |
| 27 | `newman` | Chem | 5 | 6 | 1 | ✓ (`examen_quimica`, `guia_integradora`) | 12 |
| 28 | `chair` | Chem | 5 | 7 | 1 | ✓ (`examen_quimica`, `guia_integradora`) | 13 |
| 29 | `fischer` | Chem | 5 | 8 | 1 | ✓ (`examen_quimica`, `guia_integradora`) | 14 |
| 30 | `crystal_field` | Chem | 5 | 7 | 1 | ✓ (`examen_quimica`, `guia_integradora`) | 13 |
| 31 | `complex` | Chem | 7 | 6 | 1 | ✓ (`examen_quimica`, `guia_integradora`) | 14 |
| 32 | `latimer` | Chem | 5 | 5 | 1 | ✓ (`examen_quimica`, `guia_integradora`) | 11 |
| 33 | `frost` | Chem | 5 | 5 | 1 | ✓ (`guia_integradora`) | 11 |
| 34 | `born_haber` | Chem | 5 | 6 | 2 | ✓ (`examen_quimica`, `guia_integradora`) | 13 |
| 35 | `array` | CS | 5 | 6 | 1 | ✓ (`examen_algoritmos`, `guia_integradora`, `parcial`) | 12 |
| 36 | `matrix` | CS | 6 | 9 | 1 | ✓ (`examen_algoritmos`, `guia_integradora`) | 16 |
| 37 | `linked_list` | CS | 6 | 6 | 1 | ✓ (`examen_algoritmos`, `guia_integradora`) | 13 |
| 38 | `stack` | CS | 5 | 5 | 1 | ✓ (`examen_algoritmos`, `guia_integradora`) | 11 |
| 39 | `call_stack` | CS | 5 | 6 | 1 | ✓ (`examen_algoritmos`, `guia_integradora`) | 12 |
| 40 | `call_tree` | CS | 5 | 6 | 1 | ✓ (`examen_algoritmos`, `guia_integradora`) | 12 |
| 41 | `queue` | CS | 5 | 5 | 1 | ✓ (`examen_algoritmos`, `guia_integradora`) | 11 |
| 42 | `tree` | CS | 5 | 6 | 2 | ✓ (`examen_algoritmos`, `guia_integradora`, `parcial`) | 13 |
| 43 | `struct` | CS | 5 | 6 | 2 | ✓ (`examen_algoritmos`, `guia_integradora`) | 13 |
| 44 | `adt` | CS | 5 | 6 | 1 | ✓ (`examen_algoritmos`, `guia_integradora`) | 12 |
| 45 | `complexity` | CS | 5 | 8 | 1 | ✓ (`examen_algoritmos`, `guia_integradora`) | 14 |
| 46 | `flowchart` | CS | 5 | 6 | 1 | ✓ (`examen_algoritmos`, `guia_integradora`) | 12 |
| 47 | `cost_table` | CS | 5 | 6 | 1 | ✓ (`examen_algoritmos`, `guia_integradora`, `parcial`) | 12 |
| 48 | `trace_table` | CS | 5 | 7 | 1 | ✓ (`examen_algoritmos`, `guia_integradora`, `parcial`) | 13 |
