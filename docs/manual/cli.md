[Manual](README.md)

# Línea de comandos

```bash
dibu list-types                         # los tipos compilados en este binario
dibu list-themes                        # los presets de theme
dibu render examen.yaml -o salida/      # un SVG por bloque: <id>.svg o <archivo>_<n>.svg
dibu render examen.yaml --color         # theme color
dibu render examen.yaml --theme dark    # o --theme mi-tema.yaml
dibu sync-md apuntes/                   # los bloques ```estructura de los .md como SVG versionados
dibu sync-md --verificar apuntes/       # falla si hay SVG faltantes, desactualizados o sobrantes
dibu batch < pedidos.jsonl              # un proceso para muchos diagramas
```

## `batch`

Cada línea de la entrada es un JSON `{"id": "...", "block": {...}}`, con `tinta` (`true` o `false`)
y `theme` (preset u objeto) opcionales; cada línea de la salida es `{"id", "svg"}` o
`{"id", "error"}`. Así un programa (alucarD, un generador de exámenes) dibuja cientos de diagramas
con un solo proceso.

```bash
echo '{"id":"a","block":{"type":"array","values":[1,2,3]}}' | dibu batch
```

## Familias

El binario trae todas las familias. Para uno más chico se compila sólo lo necesario:

```bash
cargo build --release -p dibu-cli --no-default-features --features cs,comp
```

Familias: `cs` (estructuras), `control`, `chem` (química), `comp` (computación, que también trae
heap, hash, AVL, B+, programación dinámica, Dijkstra y Turing) y `math` (señales, probabilidad y
circuitos).

## Editor web

[dibu-web](https://ingcom-unrn.github.io/dibu-web/) tiene los mismos tipos en el navegador, con
vista previa en vivo, autocompletado de campos y un editor visual: sirve de playground para
probar un bloque antes de llevarlo a un apunte o a un banco. «Compartir» copia un enlace con el
diagrama en la URL (`#z=…` o `#t=…`), que lo abre tal cual; así enlaza el manual cada uno de sus ejemplos.
