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

## Un valor en blanco: `!blank`

Para dejar en blanco un valor concreto, cualquiera sea el tipo, se le antepone `!blank` (una
etiqueta de YAML) o se le agrega `.blank` al final. dibu **usa el valor para calcular** el diagrama
y lo **dibuja como un renglón para completar** del mismo ancho, así el dibujo no delata la
respuesta:

```yaml
type: heap
values: [50, 30, 20, !blank 40, 10]    # el 40 se inserta y sale en blanco donde queda
```

```yaml
type: tree
variant: bst
values: [50, 30, 70, 34.blank, 60]     # lo mismo con el sufijo
```

- `!blank 34` conserva el tipo: es el número 34. `!blank "007"` es el texto `007`. Con el sufijo,
  `34.blank` es un número y `ddf.blank` un texto; un texto con espacios o comillas va con
  `!blank "…"` (`"ddf".blank` no es YAML válido). En JSON, `"34.blank"`.
- `!blank` sobre una lista o un objeto deja en blanco todo lo que tiene adentro:
  `values: !blank [3, 4]`.
- Si el valor aparece en más de un lugar (en el árbol y en el arreglo de un heap, en la bitácora
  de una inserción), sale en blanco en todos. Un número igual de otro dato también sale en blanco,
  así nunca se ve la respuesta.
- Si el diagrama no puede dejar ese valor en blanco (porque no lo dibuja como texto, como una
  coordenada o un número de línea, o porque es una referencia o una opción, como `kind`), da un
  error que dice qué valor del bloque es, en lugar de mostrarlo.

Como dibu calcula la solución, el diagrama sin `blank` es la clave del ejercicio con `blank`.
