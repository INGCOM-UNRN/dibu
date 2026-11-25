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
- `dibu-wasm`: el motor para WebAssembly, con todas las familias. Tiene una API pura (texto → JSON)
  y, con la feature `abi-c`, el módulo `dibu.wasm` sin imports que usan las integraciones.

## Integraciones

Cada una vive en su propio repositorio y usa el motor de `crates/dibu-wasm`:

- [dibu-web](https://github.com/INGCOM-UNRN/dibu-web): el editor en el navegador.
- [dibu-myst](https://github.com/INGCOM-UNRN/dibu-myst): plugin de [MyST](https://mystmd.org) que
  dibuja los bloques ```` ```estructura ```` y la directiva `{dibu}` al construir el sitio o el PDF.
- [dibu-typst](https://github.com/INGCOM-UNRN/dibu-typst): paquete de [Typst](https://typst.app)
  con la función `#dibu(...)` y los bloques ```` ```estructura ```` dentro del documento.

```bash
cargo build --profile wasm -p dibu-wasm --target wasm32-unknown-unknown
# → target/wasm32-unknown-unknown/wasm/dibu_wasm.wasm
```

## Manual

El [manual](docs/manual/README.md) tiene una página por cada tipo de diagrama, agrupadas por área
(estructuras de datos y algoritmos, sistemas de computación, control de procesos, química y
matemática aplicada), con todos sus campos, lo que calcula, el modo ejercicio y ejemplos dibujados.
Se genera con `python3 docs/manual/generar.py` a partir de `docs/manual/datos/`, del código y del
kitchen sink; `--verificar` falla si falta documentar un tipo o un campo, y corre en CI.
El workflow `manual.yml` lo publica en GitHub Pages con cada cambio en `docs/manual/` y con cada
release (hay que activar Pages con origen «GitHub Actions» en Settings → Pages).

### Para agentes de IA

La [guía para agentes](docs/manual/agentes/README.md) reúne lo que necesita un agente (o una
persona) que redacta apuntes o exámenes con diagramas de dibu: reglas, catálogo, cómo integrar los
bloques en Markdown, MyST, Typst y bancos de alucarD, y errores frecuentes. La acompañan una
[referencia compacta](docs/manual/agentes/referencia.md) de todos los tipos, un
[esquema JSON](docs/manual/agentes/esquema.json) y un validador:

```bash
python3 docs/manual/agentes/validar.py apunte.md   # avisa campos que dibu ignora, valores fuera de la lista…
```

Como dibu ignora sin avisar un campo mal escrito, el validador es la forma de detectarlo. Se genera
junto con el manual, y `--verificar` comprueba que todos los ejemplos lo pasen sin avisos.
[`llms.txt`](llms.txt) apunta a estos archivos.

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

## Themes

Cada diagrama se dibuja con un **theme**: una paleta semántica (fondo, trazos, rellenos, acento
y textos) más una tipografía (familias sans y mono, escala y factor de ancho por carácter).

| Preset | Uso |
|---|---|
| `ink` (`tinta`) | por defecto: grises y negro, pensado para imprimir |
| `color` | acentos azules (lo mismo que `--color`) |
| `high-contrast` (`alto-contraste`) | todo en negro sobre blanco, para proyectar o fotocopiar |
| `dark` (`oscuro`) | fondo oscuro, para pantallas y presentaciones |

```bash
dibu list-themes
dibu render examen.yaml --theme dark
dibu render examen.yaml --theme mi-tema.yaml     # archivo YAML/JSON
DIBU_THEME=high-contrast dibu sync-md apuntes/  # también por variable de entorno
```

Un archivo de theme parte de un preset con `extends` y pisa sólo lo que cambia:

```yaml
extends: ink
colors:
  background: "#fafaf9"
  strokeMain: "#1b365d"
  accentPrimary: "#b45309"
typography:
  sans: "Liberation Sans, Arial, sans-serif"
  mono: "Liberation Mono, monospace"
  scale: 1.2            # 0.6 a 1.8: agranda la letra y el diagrama se acomoda
  charWidthFactor: 0.62 # ancho medio de un carácter, en em (0.4 a 1.0)
```

Colores disponibles: `background`, `strokeMain`, `strokeSub`, `strokeMuted`, `fillHeader`,
`fillCell`, `fillPointerBox`, `fillBadge`, `accentPrimary`, `textMain`, `textSub`, `textMuted`
y `fillHighlight`. Se aceptan `#RGB`, `#RRGGBB`, `#RRGGBBAA`, `transparent` y `none`; los colores
y las familias tipográficas se validan, así que un theme no puede inyectar atributos en el SVG.

Prioridad, de mayor a menor:
1. `theme:` dentro del bloque (un preset o un objeto como el de arriba).
2. `inkSaver: true|false` dentro del bloque (elige `ink` o `color` y conserva la tipografía).
3. `--theme` o `DIBU_THEME`.
4. `--color` (`color`) o, si no hay nada, `ink`.

En `dibu batch` cada línea JSONL puede traer su propio `theme` (preset u objeto); `tinta` sigue
funcionando. Con el theme por defecto la salida es idéntica byte a byte a la de versiones
anteriores, así que los SVG versionados con `sync-md` no cambian.

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
(`uv tool install ./dibu-*.whl`), el motor `dibu-X.Y.Z.wasm` y un `SHA256SUMS`.

Para publicar una versión, subir `version` en `Cargo.toml` (`[workspace.package]`) y en
`pyproject.toml`, y empujar el tag correspondiente:

```bash
git tag v0.2.0 && git push origin v0.2.0
```

Workflows en `.github/workflows/`:

- `ci.yml`: en cada push y PR. Corre los tests en Linux, macOS y Windows, renderiza todos
  los fixtures con la CLI y verifica con `sync-md --verificar` que los SVG versionados estén
  al día. Los tests `paridad_*` se omiten porque necesitan el oráculo local `alucarD`.
  Compila el motor WebAssembly y verifica que no tenga imports.
  `cargo fmt` y `clippy` corren en modo informativo hasta que el código pase sin avisos.
- `release.yml`: al empujar un tag `vX.Y.Z`. Valida que el tag coincida con las versiones
  declaradas, corre el CI, compila los binarios y wheels, y crea el release con notas
  agrupadas a partir de los commits semánticos. Si existe el secreto
  `DIBU_WEB_DISPATCH_TOKEN`, avisa a `dibu-web` para que republique el editor.
- `mantenimiento.yml`: los lunes y a mano. `cargo audit` (RustSec), tests con Rust beta y
  un resumen de las dependencias con versiones nuevas. Dependabot (`.github/dependabot.yml`)
  abre PRs semanales para Cargo y mensuales para las Actions.
