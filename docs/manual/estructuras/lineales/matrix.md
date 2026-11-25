[Manual](../../README.md) › [Estructuras de datos y algoritmos](../README.md) › [Estructuras lineales](README.md)

# Matriz (`matrix`)

Dibuja una matriz y cómo está guardada en memoria. La estática muestra la declaración
(`int m[3][4]`) y, con `showAddresses`, la dirección de cada celda en orden de filas; la
dinámica, el puntero, el arreglo de punteros a filas y cada fila; la dentada, filas de distinto
largo con el arreglo de largos o un centinela; la plana, un único bloque contiguo con la cuenta
`m[i * C + j]`; la de Java, un arreglo de arreglos (`int[][]`) con cada fila como objeto aparte.

## Qué calcula

- La dirección de cada celda en orden de filas.
- En la plana, el índice lineal de cada `(i, j)`.
- En la dentada, el arreglo de largos a partir de las filas.

## Variantes

| `variant` | Qué dibuja |
| :--- | :--- |
| `static` | Por omisión. `int m[F][C]` en un bloque, con `showLinear` también fila por fila en memoria. |
| `dynamic` | Un `int **`: el puntero, el arreglo de punteros a filas y cada fila por separado; `null` es un puntero a NULL. |
| `jagged` | Dinámica con filas de distinto largo, con el arreglo de largos (`showLengths`) o un `sentinel` al final de cada fila. |
| `flat` | Un `int *` con `F * C` elementos contiguos: cada celda con su índice `[k]` y su `(i, j)`. |
| `java` | Un `int[][]` de Java: la variable refiere al arreglo exterior, que guarda una referencia por fila; cada fila es un objeto con su `length`. `null` es una referencia nula y `[]`, una fila de largo cero; `values: []` es un arreglo exterior vacío. |

## Campos

| Campo | Tipo | Valores y valor por defecto | Descripción |
| :--- | :--- | :--- | :--- |
| `type` | texto | `matrix` | Obligatorio. |
| `values` | lista |  | Lista de filas (o lista plana junto con `cols`). En la dinámica, `null` es una fila que es NULL; en la de Java, una referencia `null`, y `[]` una fila de largo cero. |
| `rows` | número |  | Cantidad de filas, si no salen de `values` (de 1 a 10). |
| `cols` | número |  | Cantidad de columnas (de 1 a 10). |
| `variant` | texto |  | Variante, ver arriba. |
| `name` | texto |  | Nombre de la variable (por omisión `m`). |
| `elementType` | texto |  | Tipo de los elementos, para la declaración (por omisión `int`). |
| `typeSize` | número |  | Tamaño de cada elemento en bytes. |
| `showIndices` | sí/no |  | Muestra los índices de filas y columnas. |
| `showAddresses` | sí/no |  | Muestra la dirección de cada celda. |
| `baseAddr` | texto |  | Dirección de la primera celda (o del bloque, en la plana). |
| `rowAddresses` | lista |  | En la dinámica, la dirección de cada fila. |
| `rowsBaseAddr` | número |  | En la dinámica, la dirección del arreglo de punteros a filas. |
| `highlight` | lista |  | Celdas resaltadas, como `"1,2; 0,0"` o `[[1, 2]]`. |
| `highlightRows` | lista |  | Filas resaltadas. |
| `highlightCols` | lista |  | Columnas resaltadas. |
| `blankValues` | sí/no |  | Deja los valores en blanco. |
| `blankAddrs` | sí/no |  | Deja las direcciones en blanco. |
| `showLinear` | sí/no |  | En la estática, dibuja además la matriz como está en memoria, una fila detrás de la otra. |
| `showLengths` | sí/no |  | En la dentada, el arreglo de largos (por omisión, sí). |
| `lengths` | lista |  | En la dentada, los largos, si no salen de las filas. |
| `lengthsName` | texto |  | Nombre del arreglo de largos. |
| `lengthsType` | texto |  | Tipo de los largos (por omisión `size_t`). |
| `lengthsAddr` | texto |  | Dirección del arreglo de largos. |
| `blankLengths` | sí/no |  | Deja los largos en blanco (en la de Java, cada `length`). |
| `sentinel` | texto |  | En la dentada, un valor centinela al final de cada fila, como `"'\\0'"` o `-1`. |
| `showCoords` | sí/no |  | En la plana, el `(i, j)` de cada celda (por omisión, sí). |
| `showFormula` | sí/no |  | En la plana, la fórmula `m[i * C + j]` (por omisión, sí). |
| `showLogical` | sí/no |  | En la plana, agrega la matriz como la piensa el programa. |
| `zones` | sí/no |  | Enmarca el stack y el heap; `true` usa esos títulos, o `{stack, heap}` con títulos propios. |
| `owner` | objeto |  | Una estructura dueña de la matriz en lugar del puntero: `name`, `type` y `fields` (`"filas -> rows"`, `"largos -> lengths"`). |

Además, los campos comunes a todos los tipos: `title`, `subtitle`, `id`, `inkSaver` y `theme` (ver [Campos comunes](../../campos-comunes.md)).

## Modo ejercicio

Campos que dejan casilleros para completar: `blankValues`, `blankAddrs`, `blankLengths`.

## Ejemplos

### Matriz estática row-major

```yaml
type: matrix
title: Matriz estática row-major
values: [[1, 2, 3], [4, 5, 6]]
baseAddr: "0x1000"
typeSize: 4
showAddresses: true
showLinear: true
highlight: [[0, 0], [1, 2]]
```

![Matriz estática row-major](img/ks_cs_matrix_estatica_lineal.svg)

### Matriz dentada con dueña y zonas

```yaml
type: matrix
title: Matriz dentada con dueña y zonas
variant: jagged
name: filas
values: [[4, 8], [9]]
showAddresses: true
baseAddr: "@F"
rowAddresses: ["@R0", "@R1"]
lengthsType: int
lengthsAddr: "@L"
zones: true
owner:
  name: d
  type: dentada_t
  fields:
    - "n = 2"
    - "largos -> lengths"
    - "filas -> rows"
```

![Matriz dentada con dueña y zonas](img/ks_cs_matrix_jagged_duena.svg)

### Matriz plana con vista lógica

```yaml
type: matrix
title: Matriz plana con vista lógica
variant: flat
values: [[1, 2], [3, 4]]
showLogical: true
showAddresses: true
inkSaver: false
```

![Matriz plana con vista lógica](img/ks_cs_matrix_flat_color.svg)

### Arreglo de arreglos en Java

```yaml
type: matrix
title: "Arreglo de arreglos en Java"
variant: java
name: matriz
values:
  - [1, 2, 3]
  - [4]
  - null
  - []
zones: true
```

![Arreglo de arreglos en Java](img/ks_cs_matrix_java_irregular.svg)

### Arreglo exterior de largo cero (para completar)

```yaml
type: matrix
title: Arreglo exterior de largo cero (para completar)
variant: java
name: vacia
values: []
blankLengths: true
inkSaver: false
```

![Arreglo exterior de largo cero (para completar)](img/ks_cs_matrix_java_vacia.svg)

## Notas

- El máximo es 10 filas por 10 columnas; en la plana, 24 celdas.
- Las direcciones simbólicas (`"@F"`) van entre comillas en YAML.

## Ver también

[`array`](array.md), [`linked_list`](linked_list.md), [`stack`](stack.md), [`queue`](queue.md)
