# dibu

Motor en Rust para generar diagramas educativos SVG a partir de bloques YAML/JSON compatibles con `alucarD` y `statics`.

## Arquitectura del workspace

- `dibu-core`: lienzo SVG, paletas (`tinta` / `color`), escape XML, parseo de bloques YAML/JSON, apilado de SVGs y hash SHA-1.
- `dibu-plot`: ejes cartesianos (lineal/log), grillas, curvas, guiones y leyendas.
- `dibu-control`: motor matemático de control de procesos y 9 tipos de diagramas (`block_diagram`, `time_response`, `bode`, `nyquist`, `pole_zero`, `root_locus`, `instrumentation`, `ladder`, `grafcet`).
- `dibu-chem`: motor de cálculos químicos y 21 tipos (química general, orgánica e inorgánica).
- `dibu-cs`: 14 tipos de estructuras de datos y algoritmos.
- `dibu-comp`: 18 tipos de ingeniería en computación: sistemas digitales, arquitectura, sistemas operativos, redes, bases de datos y estructuras de datos y algoritmos.
- `dibu-math`: 8 tipos de señales y sistemas, probabilidad y circuitos eléctricos.
- `dibu-cli`: binario unificado `dibu` (`render`, `batch`, `sync-md`, `list-types`) con `--features` por familia (`cs`, `control`, `chem`).

## Uso rápido

```bash
cargo build --release
./target/release/dibu list-types
./target/release/dibu render fixtures/diagramas_control.yaml -o /tmp/out
./target/release/dibu batch < pedidos.jsonl
```

Para compilar solo una familia:

```bash
cargo build --release -p dibu-cli --no-default-features --features control
# familias: cs, control, chem, comp, math (todas por defecto)
```

## Tipos de `dibu-comp` y `dibu-math`

Ejemplos completos en `fixtures/diagramas_comp.yaml` y `fixtures/diagramas_math.yaml`. Ninguno
repite lo que ya resuelven Mermaid o PlantUML (grafos genéricos, autómatas, UML, ER, secuencia,
Gantt de proyecto, timing). Todos parten de los **datos del problema** y `dibu` calcula la
solución, así un generador de exámenes puede variar los datos. Con `blank: true` se dibuja el
enunciado con los espacios para completar, a partir del mismo bloque.

| Tipo | Campos principales | Qué calcula |
|---|---|---|
| `truth_table` | `expr` u `outputs: [{name, expr}]`, o `vars` + `minterms`/`maxterms`; `steps`, `canonical`, `index` | tabla de verdad, columnas intermedias, Σm / ΠM |
| `karnaugh` | `expr` o `vars` + `minterms`, `dontcares`; `form: sop\|pos`, `implicants`, `notation: overline\|prime\|neg` | mapa de 2 a 4 variables, agrupamientos mínimos (Quine-McCluskey) y expresión simplificada |
| `logic_circuit` | `expr`, `output`; `rails`, `basicGates` | circuito con compuertas AND/OR/XOR/NAND/NOR/XNOR/NOT |
| `pipeline` | `instructions` (MIPS/RISC-V), `forwarding` | diagrama de ciclos de 5 etapas, burbujas, riesgos RAW y CPI |
| `ieee754` | `value` o `bits`, `format: half\|single\|double` | campos de bits, normalización, exponente sesgado y error de redondeo |
| `cache` | `addressBits`, `cacheSize`, `blockSize`, `associativity`, `policy: lru\|fifo`, `accesses` | tag/índice/offset, aciertos y fallos (forzosos, de capacidad, de conflicto), contenido final |
| `cpu_scheduling` | `processes: [{name, arrival, burst, priority}]`, `algorithm: fcfs\|sjf\|srtf\|rr\|priority\|priority_preemptive`, `quantum` | diagrama de Gantt y tiempos de retorno, espera y respuesta |
| `page_table` | `pageSize`, `table` (marcos, `null` = no presente), `addresses` | traducción de direcciones lógicas a físicas y fallos de página |
| `page_replacement` | `frames`, `references`, `algorithms: [fifo, lru, opt, clock]` | marcos en cada paso, fallos y tasa de fallos |
| `subnetting` | `network` (`A.B.C.D/n`), `subnets: [{name, hosts}]` (VLSM) o `count` | red, máscara, rango de hosts y broadcast de cada subred |
| `transaction_schedule` | `schedule: [R1(A), W2(A), C1, …]` | conflictos, grafo de precedencia y serializabilidad por conflictos |
| `heap` | `values`, `kind: min\|max`, `build: insert\|heapify\|none`, `ops: [insert X, extract]`, `steps` | árbol y arreglo después de cada operación |
| `hash_table` | `size`, `keys`, `collision: chaining\|linear\|quadratic\|double` | cubetas, secuencia de sondeos y factor de carga |
| `avl` | `keys`, `delete`, `steps: none\|rotations\|all` | árbol balanceado, rotaciones y factores de balance |
| `bplus_tree` | `order` (o `maxKeys`), `keys`, `steps` | divisiones de nodos y hojas enlazadas |
| `dp_table` | `problem: lcs\|edit\|knapsack\|coins` y sus datos | tabla de programación dinámica con el camino de reconstrucción |
| `dijkstra` | `edges: ["A-B:4", "A->C:2"]`, `source` | tabla de iteraciones (distancia, predecesor) y caminos mínimos |
| `turing` | `transitions: ["q0,1 -> q1,X,R"]`, `input`, `start`, `accept` | traza de configuraciones y tabla δ |
| `signal` | `expr` o `signals: [{expr, label}]`, `domain: continuous\|discrete`, `range` | gráfico de x(t) o x[n] (u, rect, tri, sinc, ramp, …) |
| `convolution` | `x`, `xStart`, `h`, `hStart`, `step` | x[k], h[k], h[n−k] superpuesta y y[n] |
| `fourier` | `sequence` (DFT) o `wave: square\|sawtooth\|triangle` + `harmonics` | espectro y suma parcial de la serie |
| `sampling` | `frequency`, `fs`, `periods`, `spectrum` | muestras, alias y criterio de Nyquist |
| `distribution` | `dist: normal\|uniform\|exponential\|t\|chi2\|binomial\|poisson\|geometric` y parámetros, `interval: [a, b]` | densidad o masa, área sombreada y probabilidad exacta |
| `venn` | `sets` (2 o 3), `shade` (expresión o lista), `regions` | regiones sombreadas por operaciones de conjuntos |
| `circuit` | `network: {series\|parallel: […]}`, `source`, `frequency`, `style: ansi\|iec` | esquema, impedancia equivalente y corriente |
| `phasor` | `phasors: [{label, magnitude, angle}]`, `sum`, `tipToTail` | diagrama fasorial, suma en forma polar y rectangular |

## Releases e integración continua

Los binarios precompilados de cada versión están en la página de
[Releases](https://github.com/INGCOM-UNRN/dibu/releases): Linux (glibc y musl estático,
x86_64 y aarch64), macOS (Intel y Apple Silicon) y Windows x86_64, más wheels de Python
(`uv tool install ./dibu-*.whl`) y un `SHA256SUMS`.

Para publicar una versión, subir `version` en `Cargo.toml` (`[workspace.package]`) y en
`pyproject.toml`, y empujar el tag correspondiente:

```bash
git tag v0.2.0 && git push origin v0.2.0
```

Workflows en `.github/workflows/`:

- `ci.yml`: en cada push y PR. Corre los tests en Linux, macOS y Windows, renderiza todos
  los fixtures con la CLI y verifica con `sync-md --verificar` que los SVG versionados estén
  al día. Los tests `paridad_*` se omiten porque necesitan el oráculo local `alucarD`.
  `cargo fmt` y `clippy` corren en modo informativo hasta que el código pase sin avisos.
- `release.yml`: al empujar un tag `vX.Y.Z`. Valida que el tag coincida con las versiones
  declaradas, corre el CI, compila los binarios y wheels, y crea el release con notas
  agrupadas a partir de los commits semánticos. Si existe el secreto
  `DIBU_WEB_DISPATCH_TOKEN`, avisa a `dibu-web` para que republique el editor.
- `mantenimiento.yml`: los lunes y a mano. `cargo audit` (RustSec), tests con Rust beta y
  un resumen de las dependencias con versiones nuevas. Dependabot (`.github/dependabot.yml`)
  abre PRs semanales para Cargo y mensuales para las Actions.
