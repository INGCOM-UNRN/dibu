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
