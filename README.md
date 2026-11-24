# dibu

Motor en Rust para generar diagramas educativos SVG a partir de bloques YAML/JSON compatibles con `alucarD` y `statics`.

## Arquitectura del workspace

- `dibu-core`: lienzo SVG, paletas (`tinta` / `color`), escape XML, parseo de bloques YAML/JSON, apilado de SVGs y hash SHA-1.
- `dibu-plot`: ejes cartesianos (lineal/log), grillas, curvas, guiones y leyendas.
- `dibu-control`: motor matemático de control de procesos y 9 tipos de diagramas (`block_diagram`, `time_response`, `bode`, `nyquist`, `pole_zero`, `root_locus`, `instrumentation`, `ladder`, `grafcet`).
- `dibu-chem`: motor de cálculos químicos y 21 tipos (química general, orgánica e inorgánica).
- `dibu-cs`: 14 tipos de estructuras de datos y algoritmos.
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
```

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
