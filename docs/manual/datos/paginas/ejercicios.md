[Manual](README.md)

# Modo ejercicio

El mismo bloque sirve para la explicación, el ejercicio y la clave: los campos `blank` y `blank*`
dibujan el diagrama con los casilleros de la respuesta vacíos (una línea punteada donde va el
valor), sin tocar los datos.

```yaml
type: cache
cacheSize: 64
blockSize: 8
accesses: ["0x000", "0x040", "0x000"]
blank: true          # tag, índice, resultado y contenido final, para completar
```

- **`blank: true`** oculta, en cada tipo, lo que se pregunta habitualmente: la salida de la tabla
  de verdad, el diagrama de Gantt y los tiempos, la traza de la máquina de Turing, el área de una
  distribución. Lo que se ve son los datos del problema.
- **`blank*`** son más finos: `blankValues`, `blankAddrs`, `blankOffsets`, `blankMargins`,
  `blankAnnotations`… Cada página de tipo los lista en su sección «Modo ejercicio».
- Algunos aceptan una lista de qué ocultar: `blank: [name, angle]` en `vsepr`, `blank: [z, density]`
  en `unit_cell`.
- En listas de elementos (pasos de un diagrama de flujo, llamadas de un árbol, ranuras de un
  GRAFCET), cada elemento puede tener su propio `blank: true`.

Como dibu calcula la solución, el diagrama sin `blank` es la clave del ejercicio con `blank`.
