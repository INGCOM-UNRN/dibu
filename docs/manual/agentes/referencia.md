[Guía para agentes](README.md)

# Referencia de los tipos de dibu

Los 71 tipos, con todos sus campos. Los valores admitidos y por defecto salen del código; los ejemplos se dibujan sin errores y pasan `validar.py` sin avisos. Además de los campos de cada tipo, todos aceptan `title`, `subtitle`, `id`, `inkSaver` y `theme`.

- **Estructuras de datos y algoritmos**: [`array`](#array), [`matrix`](#matrix), [`linked_list`](#linked_list), [`stack`](#stack), [`queue`](#queue), [`call_stack`](#call_stack), [`call_tree`](#call_tree), [`struct`](#struct), [`object_heap`](#object_heap), [`adt`](#adt), [`tree`](#tree), [`heap`](#heap), [`avl`](#avl), [`bplus_tree`](#bplus_tree), [`hash_table`](#hash_table), [`complexity`](#complexity), [`flowchart`](#flowchart), [`cost_table`](#cost_table), [`trace_table`](#trace_table), [`dp_table`](#dp_table), [`dijkstra`](#dijkstra), [`turing`](#turing)
- **Sistemas de computación**: [`truth_table`](#truth_table), [`karnaugh`](#karnaugh), [`logic_circuit`](#logic_circuit), [`pipeline`](#pipeline), [`ieee754`](#ieee754), [`cache`](#cache), [`cpu_scheduling`](#cpu_scheduling), [`page_table`](#page_table), [`page_replacement`](#page_replacement), [`subnetting`](#subnetting), [`transaction_schedule`](#transaction_schedule)
- **Control de procesos**: [`block_diagram`](#block_diagram), [`time_response`](#time_response), [`bode`](#bode), [`nyquist`](#nyquist), [`pole_zero`](#pole_zero), [`root_locus`](#root_locus), [`instrumentation`](#instrumentation), [`ladder`](#ladder), [`grafcet`](#grafcet)
- **Química**: [`molecule`](#molecule), [`vsepr`](#vsepr), [`orbitals`](#orbitals), [`unit_cell`](#unit_cell), [`reaction`](#reaction), [`energy_profile`](#energy_profile), [`kinetics`](#kinetics), [`phase_diagram`](#phase_diagram), [`heating_curve`](#heating_curve), [`titration`](#titration), [`speciation`](#speciation), [`spectrum`](#spectrum), [`galvanic_cell`](#galvanic_cell), [`latimer`](#latimer), [`frost`](#frost), [`newman`](#newman), [`chair`](#chair), [`fischer`](#fischer), [`crystal_field`](#crystal_field), [`complex`](#complex), [`born_haber`](#born_haber)
- **Matemática aplicada**: [`signal`](#signal), [`convolution`](#convolution), [`fourier`](#fourier), [`sampling`](#sampling), [`distribution`](#distribution), [`venn`](#venn), [`circuit`](#circuit), [`phasor`](#phasor)

<a id="array"></a>

## `array`: Arreglo

*Estructuras de datos y algoritmos › Estructuras lineales.* Arreglos en memoria, con índices, direcciones, offsets y punteros que los recorren, y secuencias de Python con sus cortes.

Dibuja un arreglo como una fila de celdas contiguas, con el índice de cada una y,
opcionalmente, su dirección de memoria y su offset desde el comienzo. Los punteros o índices
que lo recorren se dibujan como flechas sobre la celda a la que apuntan. Con
`variant: python_sequence`, una lista, tupla o cadena de Python, con índices negativos,
fronteras y un corte (`slice`).

**Calcula:** La dirección de cada celda a partir de `baseAddr` y del tamaño del elemento (`typeSize`). En `python_sequence`, las celdas que elige un corte, con las reglas de `slice.indices(n)`: los límites fuera de rango se ajustan, y el resultado se escribe como en Python (`[4, 1]`, `(4,)`, `'aloh'`).

| `variant` | Qué dibuja |
| :--- | :--- |
| `python_sequence` | Una secuencia de Python (`kind`): el índice positivo y, con `showNegativeIndices`, el negativo de cada elemento; con `showBoundaries`, las fronteras entre celdas (de `0` a `n`). Con `slice`, resalta las celdas elegidas, numera el orden de visita si el paso no es 1, marca el inicio y el fin (el límite excluido) y escribe el resultado. |

| Campo | Tipo | Valores · por defecto | Descripción |
| :--- | :--- | :--- | :--- |
| `values` | lista o texto |  | Los valores, en orden. Un valor vacío (`""`) deja la celda sin contenido. |
| `capacity` | número |  | Cantidad total de celdas; las que sobran después de `values` se dibujan vacías (el espacio reservado sin usar). |
| `pointers` | texto |  | Punteros o índices sobre el arreglo, como texto `"p:0, q:3"` (nombre y posición). |
| `showIndices` | sí/no |  | Muestra el índice de cada celda (por omisión, sí). |
| `showAddresses` | sí/no |  | Muestra la dirección de cada celda. |
| `showOffsets` | sí/no |  | Muestra el desplazamiento en bytes de cada celda respecto del comienzo. |
| `baseAddr` | número |  | Dirección de la primera celda, en hexadecimal (`0x1000`) o simbólica (`"@A"`). |
| `typeSize` | número |  | Tamaño de cada elemento en bytes (por omisión 4, un `int`). |
| `blankValues` | sí/no |  | Deja los valores en blanco, para completar. |
| `blankAddrs` | sí/no |  | Deja las direcciones en blanco, para calcularlas. |
| `variant` | texto |  | Variante, ver arriba. |
| `kind` | texto |  | En `python_sequence`: `list` (por omisión), `tuple` o `str`. En `str`, `values` es un texto y cada celda es un carácter; un espacio se ve como `␣`. |
| `name` | texto |  | En `python_sequence`, el nombre de la variable (por omisión `a`, `t` o `s`). |
| `showNegativeIndices` | sí/no |  | En `python_sequence`, el índice negativo de cada elemento. |
| `showBoundaries` | sí/no |  | En `python_sequence`, las fronteras entre celdas, de `0` a `n`. |
| `slice` | objeto |  | En `python_sequence`, un corte `{start, stop, step, blankSelection}`; cualquiera se puede omitir, y `{step: -1}` es `[::-1]`. El paso 0 es un error. Con `blankSelection: true` no se ve ninguna pista de la solución: ni las celdas elegidas, ni el inicio y el fin, ni el resultado. |

**Modo ejercicio:** En `python_sequence`, `slice` con `blankSelection: true` deja el corte para resolver: se ve la
expresión (`a[-4:-1:2]`) y un casillero para el resultado. Campos: `blankValues`, `blankAddrs`.

Ejemplo: Arreglo v[0..2] con <p & q> → α·β

```yaml
type: array
title: "Arreglo v[0..2] con <p & q> → α·β"
values: [3, 1, 4]
pointers: "p:0, q:2"
showAddresses: true
showOffsets: true
```

Ejemplo: Arreglo para completar

```yaml
type: array
title: Arreglo para completar
values: [41, 42]
blankValues: true
inkSaver: false
```

Ejemplo: Corte con paso: a[-4:-1:2]

```yaml
type: array
title: "Corte con paso: a[-4:-1:2]"
variant: python_sequence
kind: list
values: [2, 4, 7, 1, 8]
showNegativeIndices: true
showBoundaries: true
slice: {start: -4, stop: -1, step: 2}
```

Ejemplo: Corte de una cadena (para completar)

```yaml
type: array
title: Corte de una cadena (para completar)
variant: python_sequence
kind: str
values: "hola mundo"
showNegativeIndices: true
slice: {step: -2, blankSelection: true}
inkSaver: false
```

- Para un arreglo de dos dimensiones, ver [`matrix`](#matrix).
- En `python_sequence` no hay direcciones de memoria; hasta 16 elementos.

<a id="matrix"></a>

## `matrix`: Matriz

*Estructuras de datos y algoritmos › Estructuras lineales.* Matrices estáticas, dinámicas, dentadas, planas y de Java, como están en memoria.

Dibuja una matriz y cómo está guardada en memoria. La estática muestra la declaración
(`int m[3][4]`) y, con `showAddresses`, la dirección de cada celda en orden de filas; la
dinámica, el puntero, el arreglo de punteros a filas y cada fila; la dentada, filas de distinto
largo con el arreglo de largos o un centinela; la plana, un único bloque contiguo con la cuenta
`m[i * C + j]`; la de Java, un arreglo de arreglos (`int[][]`) con cada fila como objeto aparte.

**Calcula:** La dirección de cada celda en orden de filas. En la plana, el índice lineal de cada `(i, j)`. En la dentada, el arreglo de largos a partir de las filas.

| `variant` | Qué dibuja |
| :--- | :--- |
| `static` | Por omisión. `int m[F][C]` en un bloque, con `showLinear` también fila por fila en memoria. |
| `dynamic` | Un `int **`: el puntero, el arreglo de punteros a filas y cada fila por separado; `null` es un puntero a NULL. |
| `jagged` | Dinámica con filas de distinto largo, con el arreglo de largos (`showLengths`) o un `sentinel` al final de cada fila. |
| `flat` | Un `int *` con `F * C` elementos contiguos: cada celda con su índice `[k]` y su `(i, j)`. |
| `java` | Un `int[][]` de Java: la variable refiere al arreglo exterior, que guarda una referencia por fila; cada fila es un objeto con su `length`. `null` es una referencia nula y `[]`, una fila de largo cero; `values: []` es un arreglo exterior vacío. |

| Campo | Tipo | Valores · por defecto | Descripción |
| :--- | :--- | :--- | :--- |
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

**Modo ejercicio:** Campos: `blankValues`, `blankAddrs`, `blankLengths`.

Ejemplo: Matriz estática row-major

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

Ejemplo: Matriz dentada con dueña y zonas

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

Ejemplo: Matriz plana con vista lógica

```yaml
type: matrix
title: Matriz plana con vista lógica
variant: flat
values: [[1, 2], [3, 4]]
showLogical: true
showAddresses: true
inkSaver: false
```

Ejemplo: Arreglo de arreglos en Java

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

Ejemplo: Arreglo exterior de largo cero (para completar)

```yaml
type: matrix
title: Arreglo exterior de largo cero (para completar)
variant: java
name: vacia
values: []
blankLengths: true
inkSaver: false
```

- El máximo es 10 filas por 10 columnas; en la plana, 24 celdas.
- Las direcciones simbólicas (`"@F"`) van entre comillas en YAML.

<a id="linked_list"></a>

## `linked_list`: Lista enlazada

*Estructuras de datos y algoritmos › Estructuras lineales.* Listas simples, dobles y circulares, con direcciones, cursores y estructura cabecera.

Dibuja los nodos de una lista enlazada con su valor y el puntero al siguiente (y al anterior,
en las dobles), la cabeza y los cursores que la recorren. Puede mostrar dónde vive cada cosa:
la cabeza o una estructura cabecera en el stack y los nodos en el heap.

| `variant` | Qué dibuja |
| :--- | :--- |
| `singly` | Por omisión. Cada nodo apunta al siguiente; el último, a NULL. |
| `doubly` | Cada nodo apunta al siguiente y al anterior. |
| `circular` | El último apunta al primero. |
| `circular_doubly` | Doble y circular. |

| Campo | Tipo | Valores · por defecto | Descripción |
| :--- | :--- | :--- | :--- |
| `values` | lista |  | Los valores de los nodos, en orden (hasta 7). `[]` es la lista vacía. |
| `variant` | texto |  | Variante, ver arriba. |
| `addresses` | lista |  | La dirección de cada nodo (hexadecimal o simbólica, como `"@N1"`). |
| `headLabel` | texto |  | El rótulo de la cabeza (por omisión `HEAD`). |
| `pointers` | texto |  | Cursores sobre la lista, como `"ant:0, actual:1"`. |
| `drawArrows` | sí/no |  | Dibuja las flechas entre nodos (por omisión, sí). |
| `nodeType` | texto |  | Rotula cada nodo con su tipo, `@N1 (nodo_t)`. |
| `header` | objeto |  | Una estructura cabecera en lugar de la cabeza: `name`, `type` y `fields` (`"primero -> 0"`, `"cant = 3"`). |
| `zones` | objeto |  | Enmarca el stack y el heap; `true` o `{stack, heap}` con títulos propios. |
| `blankValues` | sí/no |  | Deja los valores en blanco. |
| `blankNextAddrs` | sí/no |  | Deja en blanco las direcciones de los punteros al siguiente. |

**Modo ejercicio:** Campos: `blankValues`, `blankNextAddrs`.

Ejemplo: Lista enlazada con cabecera y zonas

```yaml
type: linked_list
title: Lista enlazada con cabecera y zonas
values: [12, 25, 8]
addresses: ["@N1", "@N2", "@N3"]
nodeType: nodo_t
zones:
  stack: "Stack: main"
  heap: Heap
header:
  name: mi_lista
  type: lista_t
  fields:
    - "primero -> 0"
    - "ultimo -> 2"
    - "cant = 3"
```

Ejemplo: Lista circular doblemente enlazada

```yaml
type: linked_list
title: Lista circular doblemente enlazada
variant: circular_doubly
values: ["α", "β", "γ"]
pointers: "actual:1"
inkSaver: false
```

<a id="stack"></a>

## `stack`: Pila

*Estructuras de datos y algoritmos › Estructuras lineales.* Pilas en arreglo, con el tope, y su evolución paso a paso.

Dibuja una pila de abajo hacia arriba, con el tope marcado. Con `layout: vertical_multi`
muestra varios momentos de la misma pila uno al lado del otro, para seguir una secuencia de
apilar y desapilar.

| Campo | Tipo | Valores · por defecto | Descripción |
| :--- | :--- | :--- | :--- |
| `values` | lista |  | Los valores desde la base hasta el tope. Con `vertical_multi`, una lista de estados. |
| `capacity` | número |  | Cantidad de lugares de la pila; los libres se dibujan vacíos. |
| `topIndex` | número |  | Índice del tope, si no es el último valor. |
| `topLabel` | texto |  | Rótulo del tope (por omisión `tope`). |
| `label` | texto |  | Rótulo de la pila. |
| `layout` | texto |  | `vertical_multi` dibuja varios estados uno al lado del otro. |
| `traceLabels` | lista |  | Con `vertical_multi`, el rótulo de cada estado (por ejemplo, la operación). |
| `showAddresses` | sí/no |  | Muestra la dirección de cada lugar. |
| `baseAddr` | número |  | Dirección de la base. |
| `typeSize` | número |  | Tamaño de cada elemento en bytes. |
| `blankValues` | sí/no |  | Deja los valores en blanco. |
| `blankPointers` | sí/no |  | Deja el tope en blanco. |

**Modo ejercicio:** Campos: `blankValues`, `blankPointers`.

Ejemplo: Pila con traza vertical

```yaml
type: stack
title: Pila con traza vertical
layout: vertical_multi
values: [1]
traceLabels: ["Inicio", "push(2)", "pop()"]
```

<a id="queue"></a>

## `queue`: Cola

*Estructuras de datos y algoritmos › Estructuras lineales.* Colas lineales y circulares, con frente y final.

Dibuja una cola en arreglo con los índices de frente y final. La circular se dibuja como un
anillo y, con `showPanel`, con un panel que resume frente, final y cantidad.

| `variant` | Qué dibuja |
| :--- | :--- |
| `linear` | Por omisión. Un arreglo con frente y final. |
| `circular` | Un anillo de `capacity` lugares. |

| Campo | Tipo | Valores · por defecto | Descripción |
| :--- | :--- | :--- | :--- |
| `values` | lista |  | Los valores en la cola, del frente al final. |
| `variant` | texto |  | Variante, ver arriba. |
| `capacity` | número |  | Cantidad de lugares. |
| `frontIndex` | número |  | Índice del frente. |
| `rearIndex` | número |  | Índice del final. |
| `showPanel` | sí/no |  | En la circular, un panel con frente, final y cantidad. |
| `blankValues` | sí/no |  | Deja los valores en blanco. |
| `blankPointers` | sí/no |  | Deja frente y final en blanco. |

**Modo ejercicio:** Campos: `blankValues`, `blankPointers`.

Ejemplo: Cola circular en arreglo de capacidad 8

```yaml
type: queue
title: Cola circular en arreglo de capacidad 8
variant: circular
capacity: 8
frontIndex: 5
rearIndex: 1
inkSaver: false
```

<a id="call_stack"></a>

## `call_stack`: Pila de llamadas

*Estructuras de datos y algoritmos › Memoria, tipos y recursión.* Los marcos de activación de una ejecución, con parámetros, variables y punteros entre marcos.

Dibuja los marcos de activación desde `main` (abajo) hasta el tope, que se resalta. Los
parámetros van en negrita y figuran en el título del marco. Con `steps`, la pila aparece en
varios momentos alineados en la base; un marco con `popped: true` y `returns` es el que acaba
de retornar y se dibuja punteado.

**Calcula:** Las direcciones de las variables, desde `baseAddr` hacia abajo, según el tamaño de cada una.

| Campo | Tipo | Valores · por defecto | Descripción |
| :--- | :--- | :--- | :--- |
| `frames` | lista |  | Los marcos, de `main` al tope. Cada uno con `function`, `params`, `vars`, `returnTo`, `returns` y `popped`. Las variables van como `{x: 5}`, `"x = 5"` o `{name, value, type, addr, size, blank}`. |
| `steps` | lista |  | La pila en varios momentos: cada paso con `label` y `frames` (hasta 8 pasos). |
| `pointers` | texto |  | Flechas entre variables, como `"1.p -> 0.x"`: el marco por su índice desde `main` o por el nombre de la función. |
| `showAddresses` | sí/no |  | Muestra la dirección de cada variable. |
| `baseAddr` | número |  | Dirección de la primera variable de `main` (por omisión `0x7FFC`); las siguientes bajan. |
| `typeSize` | número |  | Tamaño de una variable que no es puntero (por omisión 4); los punteros ocupan 8. |
| `showTypes` | sí/no |  | Muestra el tipo de cada variable. |
| `blankValues` | sí/no |  | Deja los valores en blanco. |
| `blankAddrs` | sí/no |  | Deja las direcciones en blanco. |

**Modo ejercicio:** Campos: `blankValues`, `blankAddrs`.

Ejemplo: Marcos de activación con punteros

```yaml
type: call_stack
title: Marcos de activación con punteros
showAddresses: true
showTypes: true
frames:
  - function: main
    vars:
      - {name: a, value: 3, type: int}
      - {name: b, value: 7, type: int}
  - function: intercambiar
    returnTo: main
    params:
      - {name: x, value: "&a", type: "int *"}
      - {name: y, value: "&b", type: "int *"}
    vars: ["aux = 3"]
pointers: "1.x -> 0.a; intercambiar.y -> main.b"
```

Ejemplo: Pila de llamadas por pasos

```yaml
type: call_stack
title: Pila de llamadas por pasos
blankValues: true
steps:
  - label: "llama a fact(2)"
    frames:
      - {function: main, vars: {r: "?"}}
      - {function: fact, params: {n: 2}}
  - label: "fact(1) devuelve 1"
    frames:
      - {function: main, vars: {r: "?"}}
      - {function: fact, params: {n: 2}}
      - {function: fact, params: {n: 1}, returns: 1, popped: true}
inkSaver: false
```

- Hasta 10 marcos.
- El árbol de las llamadas recursivas es [`call_tree`](#call_tree).

<a id="call_tree"></a>

## `call_tree`: Árbol de llamadas

*Estructuras de datos y algoritmos › Memoria, tipos y recursión.* Las llamadas de una función recursiva, su orden, sus resultados y el costo por nivel.

Dibuja el árbol de llamadas de una función recursiva con lo que devuelve cada una. Sirve para
seguir la ejecución (`showOrder` numera el orden), para mostrar subproblemas repetidos
(`highlight`) y para resolver una recurrencia sumando el costo de cada nivel.

| Campo | Tipo | Valores · por defecto | Descripción |
| :--- | :--- | :--- | :--- |
| `root` | objeto |  | La llamada inicial: `call`, `returns`, `children` (las llamadas que hace) y `blank`. Hasta 40 llamadas. |
| `showOrder` | sí/no |  | Numera las llamadas en el orden en que se hacen. |
| `highlight` | lista |  | Llamadas resaltadas, por su texto (por ejemplo `["fib(1)"]`). |
| `levelCosts` | lista |  | El costo de cada nivel, al costado. |
| `total` | texto |  | El costo total, debajo. |
| `blankReturns` | sí/no |  | Deja en blanco lo que devuelve cada llamada. |
| `blankCalls` | sí/no |  | Deja en blanco las llamadas. |
| `blankLevelCosts` | sí/no |  | Deja en blanco los costos por nivel. |
| `blankTotal` | sí/no |  | Deja en blanco el total. |

**Modo ejercicio:** Campos: `blankReturns`, `blankCalls`, `blankLevelCosts`, `blankTotal`.

Ejemplo: Árbol de llamadas fib(3)

```yaml
type: call_tree
title: Árbol de llamadas fib(3)
showOrder: true
levelCosts: ["1", "2", "2"]
total: "2^n"
root:
  call: "fib(3)"
  returns: 2
  children:
    - call: "fib(2)"
      returns: 1
      children:
        - {call: "fib(1)", returns: 1}
        - {call: "fib(0)", returns: 0}
    - {call: "fib(1)", returns: 1}
```

Ejemplo: Árbol de llamadas (retornos en blanco)

```yaml
type: call_tree
title: Árbol de llamadas (retornos en blanco)
blankReturns: true
blankLevelCosts: true
levelCosts: ["n", "2n"]
total: "n^2"
root:
  call: "f(2)"
  returns: 2
  children:
    - {call: "f(1)", returns: 1}
    - {call: "f(0)", returns: 0}
inkSaver: false
```

<a id="struct"></a>

## `struct`: Struct

*Estructuras de datos y algoritmos › Memoria, tipos y recursión.* La disposición de un struct en memoria (offsets, relleno, sizeof) y los tipos opacos.

Con la variante `layout` (por omisión) dibuja los campos de un struct como los ubica gcc en 64
bits: offset, tamaño y relleno de cada uno, y el `sizeof` total. Con `opaque` muestra qué ve
cada archivo de un tipo opaco: el `.h` con el `typedef` y las operaciones, el cliente con un
puntero y el `.c` con los campos.

**Calcula:** El offset, el relleno y el `sizeof` con los tamaños y alineaciones de gcc en 64 bits (`char` 1, `short` 2, `int` y `float` 4, `long`, `double`, `size_t` y punteros 8, y los `intN_t`).

| `variant` | Qué dibuja |
| :--- | :--- |
| `layout` | Por omisión. La disposición en memoria. |
| `opaque` | Lo que ve cada archivo de un tipo opaco. |

| Campo | Tipo | Valores · por defecto | Descripción |
| :--- | :--- | :--- | :--- |
| `fields` | lista |  | Los campos: `"char nombre[8]"`, `"int edad = 30"` o `{name, type, value, size, align, blank}` (hasta 16). `size` y `align` sirven para un tipo que dibu no conoce, como un struct anidado. |
| `name` | texto |  | Nombre del struct (`struct persona`). |
| `typedef` | texto |  | Nombre del `typedef`. |
| `variant` | texto |  | Variante, ver arriba. |
| `packed` | sí/no |  | Sin relleno, como `__attribute__((packed))`. |
| `showAddresses` | sí/no |  | Muestra la dirección de cada campo. |
| `baseAddr` | número |  | Dirección del comienzo del struct. |
| `variable` | texto |  | El nombre de la variable de ese tipo. |
| `blankValues` | sí/no |  | Deja los valores en blanco. |
| `blankOffsets` | sí/no |  | Deja los offsets en blanco. |
| `blankSizes` | sí/no |  | Deja los tamaños en blanco. |
| `blankPadding` | sí/no |  | Saca las filas de relleno, para deducirlas de los offsets. |
| `blankSizeof` | sí/no |  | Deja el `sizeof` en blanco. |
| `operations` | lista |  | En `opaque`, las operaciones del `.h`. |
| `header` | texto |  | En `opaque`, el nombre del `.h`. |
| `source` | texto |  | En `opaque`, el nombre del `.c`. |
| `client` | texto |  | En `opaque`, el código del cliente. |
| `hidden` | sí/no |  | En `opaque`, oculta los campos (lo que ve el cliente); no hace falta `fields`. |

**Modo ejercicio:** Campos: `blankValues`, `blankOffsets`, `blankSizes`, `blankPadding`, `blankSizeof`.

Ejemplo: Layout LP64 con padding

```yaml
type: struct
title: Layout LP64 con padding
name: struct outer
fields:
  - {name: c, type: char}
  - {name: pos, type: struct punto, size: 8, align: 4}
```

Ejemplo: TAD opaco en C

```yaml
type: struct
title: TAD opaco en C
variant: opaque
name: struct pila
typedef: pila_t
operations: ["void f(pila_t p)"]
fields: ["int *datos", "size_t tope"]
hidden: true
inkSaver: false
```

<a id="object_heap"></a>

## `object_heap`: Stack y heap (objetos)

*Estructuras de datos y algoritmos › Memoria, tipos y recursión.* Variables, objetos y referencias en el stack y el heap, como en Java, también en varios instantes.

Dibuja los marcos activos en el stack (del tope, arriba, a `main`) y los objetos en el heap,
con una flecha por cada referencia. Cada objeto tiene un ID: dos referencias al mismo ID llegan
a la misma caja, y dos objetos con el mismo contenido son cajas distintas. Las referencias
entre objetos (y a sí mismos) rodean el heap por la derecha. Con `snapshots`, el estado en
varios instantes, para mostrar copia, mutación y reasignación de referencias: un ID que sigue
en el instante siguiente queda en el mismo lugar, así se ve cuál flecha cambió.

**Calcula:** Qué objetos son alcanzables desde el stack, siguiendo las referencias; los demás se marcan como inalcanzables.

| Campo | Tipo | Valores · por defecto | Descripción |
| :--- | :--- | :--- | :--- |
| `frames` | lista |  | Los marcos, de `main` al tope. Cada uno con `function` y `params` y `vars`: listas de `{name, type, value}` o `{name, type, ref}` (`ref: null` es una referencia nula), o `{x: 3}` para valores. |
| `objects` | lista |  | Los objetos del heap. Cada uno con `id`, `type` y uno de `fields` (como las variables), `elements` (las celdas de un arreglo: valores sueltos, `{value}` o `{ref}`) o `value` (el contenido, como un `String`). `group` los agrupa, como el String Pool. |
| `snapshots` | lista |  | Varios instantes (hasta 6), cada uno con `label`, `frames` y `objects`. Un mismo ID en dos instantes es el mismo objeto. |
| `direction` |  |  | Con `snapshots`, `column` (por omisión) los apila y `row` los pone lado a lado. |
| `variant` | texto |  | `java` (por omisión) o `python`: en Python, los nombres se agrupan por ámbito (`<módulo>`, las funciones activas), una referencia nula es `None` (un valor rotulado, no una flecha sin destino) y los valores se escriben como en Python (`'texto'`, `True`). |
| `zones` |  |  | Los títulos del stack y del heap (`{stack, heap}`); `false` quita los recuadros. |
| `markUnreachable` |  |  | Marca los objetos inalcanzables desde el stack (por omisión, sí). |
| `blankValues` | sí/no |  | Deja los valores en blanco. |
| `blankRefs` | sí/no |  | Deja las referencias en blanco, sin flechas, para dibujarlas. |

**Modo ejercicio:** Con `blankRefs`, las cajas de las referencias quedan vacías y sin flechas, para que se dibujen;
con `blankValues`, los valores. Cada variable, campo o celda acepta además `blank: true`. Campos: `blankValues`, `blankRefs`.

Ejemplo: Dos variables comparten un arreglo

```yaml
type: object_heap
title: Dos variables comparten un arreglo
frames:
  - function: main
    vars:
      - {name: a, type: "int[]", ref: A}
      - {name: b, type: "int[]", ref: A}
      - {name: copia, type: "int[]", ref: B}
      - {name: cantidad, type: int, value: 3}
objects:
  - id: A
    type: "int[]"
    elements: [{value: 1}, {value: 2}, {value: 3}]
  - id: B
    type: "int[]"
    elements: [1, 2, 3]
```

Ejemplo: Instanciación, this y String Pool

```yaml
type: object_heap
title: "Instanciación, this y String Pool"
frames:
  - function: main
    vars:
      - {name: juan, type: Persona, ref: P1}
      - {name: maria, type: Persona, ref: P2}
  - function: saludar
    params:
      - {name: this, type: Persona, ref: P1}
objects:
  - id: P1
    type: Persona
    fields:
      - {name: nombre, type: String, ref: S1}
      - {name: edad, type: int, value: 30}
  - id: P2
    type: Persona
    fields:
      - {name: nombre, type: String, ref: S1}
      - {name: edad, type: int, value: 25}
  - {id: S1, type: String, value: "Hola", group: String Pool}
inkSaver: false
```

Ejemplo: Reasignación de un parámetro

```yaml
type: object_heap
title: Reasignación de un parámetro
snapshots:
  - label: "Al entrar a modificar(numeros)"
    frames:
      - function: main
        vars: [{name: numeros, type: "int[]", ref: A}]
      - function: modificar
        params: [{name: arr, type: "int[]", ref: A}]
    objects:
      - {id: A, type: "int[]", elements: [1, 2, 3]}
  - label: "Después de arr = new int[] {7, 8}"
    frames:
      - function: main
        vars: [{name: numeros, type: "int[]", ref: A}]
      - function: modificar
        params: [{name: arr, type: "int[]", ref: B}]
    objects:
      - {id: A, type: "int[]", elements: [1, 2, 3]}
      - {id: B, type: "int[]", elements: [7, 8]}
```

Ejemplo: Tras p = null (para completar las referencias)

```yaml
type: object_heap
title: "Tras p = null (para completar las referencias)"
frames:
  - function: main
    vars: [{name: p, type: Nodo, ref: null}, {name: lista, type: "Nodo[]", ref: L}]
objects:
  - id: L
    type: "Nodo[]"
    elements: [{ref: N2}, {ref: null}]
  - id: N1
    type: Nodo
    fields: [{name: dato, type: int, value: 1}, {name: sig, type: Nodo, ref: N2}]
  - id: N2
    type: Nodo
    fields: [{name: dato, type: int, value: 2}, {name: sig, type: Nodo, ref: N2}]
blankRefs: true
```

Ejemplo: Copia superficial: copia = original.copy()

```yaml
type: object_heap
variant: python
title: "Copia superficial: copia = original.copy()"
frames:
  - function: "<módulo>"
    vars:
      - {name: original, ref: A}
      - {name: copia, ref: B}
      - {name: resultado, value: null}
objects:
  - id: A
    type: list
    elements: [{ref: I}, "x"]
  - id: B
    type: list
    elements: [{ref: I}, "x"]
  - id: I
    type: list
    elements: [{value: 4}, {value: 7}]
```

Ejemplo: def agregar(lista): lista.append(9); lista = [0] (para completar)

```yaml
type: object_heap
variant: python
title: "def agregar(lista): lista.append(9); lista = [0] (para completar)"
blankRefs: true
snapshots:
  - label: "Después de lista.append(9)"
    frames:
      - function: "<módulo>"
        vars: [{name: lista, ref: A}]
      - function: agregar
        params: [{name: lista, ref: A}]
    objects:
      - {id: A, type: list, elements: [1, 2, 9]}
  - label: "Después de lista = [0]"
    frames:
      - function: "<módulo>"
        vars: [{name: lista, ref: A}]
      - function: agregar
        params: [{name: lista, ref: B}]
    objects:
      - {id: A, type: list, elements: [1, 2, 9]}
      - {id: B, type: list, elements: [0]}
```

- Hasta 8 marcos, 16 objetos y 12 variables, campos o celdas por marco u objeto.
- Una referencia a un ID que no está en `objects`, un ID repetido o `value` junto con `ref` son errores.
- Para la pila de llamadas de C, con direcciones, ver [`call_stack`](#call_stack); para la matriz de Java, [`matrix`](#matrix) con `variant: java`.

<a id="adt"></a>

## `adt`: Tipo abstracto de datos

*Estructuras de datos y algoritmos › Memoria, tipos y recursión.* Un TAD con sus operaciones, precondiciones, invariante y representaciones posibles.

Dibuja un tipo abstracto de datos: el cliente, la tabla de operaciones con su clase
(constructor, modificador, observador…), su precondición y su poscondición, el invariante y
las representaciones posibles con sus campos.

| Campo | Tipo | Valores · por defecto | Descripción |
| :--- | :--- | :--- | :--- |
| `name` | texto |  | Nombre del TAD. |
| `operations` | lista |  | Las operaciones: texto (la firma) o `{signature, kind, pre, post, blank}`. |
| `invariant` | texto |  | El invariante de la representación. |
| `representations` | lista |  | Las representaciones posibles, cada una `{name, fields}`. |
| `client` | lista |  | Código cliente que usa el TAD. |
| `blankOperations` | sí/no |  | Deja las operaciones en blanco. |
| `blankKinds` | sí/no |  | Deja en blanco la clase de cada operación. |

**Modo ejercicio:** Campos: `blankOperations`, `blankKinds`.

Ejemplo: Especificación TAD Pila

```yaml
type: adt
title: Especificación TAD Pila
name: Pila
blankKinds: true
operations:
  - {signature: "f()", kind: observador, pre: "p != NULL"}
  - {signature: "g()", post: "tope' > 0", blank: true}
client: ["crear();"]
invariant: "tope >= 0"
representations:
  - Arreglo
  - {name: Lista, fields: ["nodo_t *cabeza"]}
```

<a id="tree"></a>

## `tree`: Árbol binario

*Estructuras de datos y algoritmos › Árboles y tablas.* Árboles binarios, de búsqueda, AVL y montículos, por niveles, por inserciones o anidados.

Dibuja un árbol binario. Los nodos se dan por niveles (`values`), por una secuencia de
inserciones en un árbol de búsqueda (`insert`, que en la variante `avl` rebalancea) o como una
estructura anidada (`root`). El AVL muestra el factor de balance y el montículo, el índice de
cada nodo y el arreglo debajo.

| `variant` | Qué dibuja |
| :--- | :--- |
| `binary` | Por omisión. Un árbol binario cualquiera. |
| `bst` | Un árbol binario de búsqueda. |
| `avl` | Un AVL, con el factor de balance de cada nodo. |
| `heap` | Un montículo, con el arreglo que lo representa. |

| Campo | Tipo | Valores · por defecto | Descripción |
| :--- | :--- | :--- | :--- |
| `values` | lista |  | Los nodos por niveles; `""` es una posición sin nodo. |
| `insert` | lista |  | Inserciones en un árbol de búsqueda, en orden; con `avl`, rebalancea. |
| `root` | objeto |  | Un árbol anidado: `value`, `left`, `right`. |
| `variant` | texto |  | Variante, ver arriba. |
| `nodeStyle` | texto |  | `struct` dibuja cada nodo como un struct con `*izq` y `*der`. |
| `nullLeaves` | sí/no |  | Dibuja los punteros a NULL de las hojas. |
| `highlight` | lista |  | Nodos resaltados, por su valor. |
| `showBalance` | sí/no |  | En el AVL, el factor de balance de cada nodo. |
| `showArray` | sí/no |  | En el montículo, el arreglo debajo. |
| `blankValues` | sí/no |  | Deja los valores en blanco. |
| `blankBalance` | sí/no |  | Deja en blanco los factores de balance. |

**Modo ejercicio:** Campos: `blankValues`, `blankBalance`.

Ejemplo: Árbol AVL rebalanceado

```yaml
type: tree
title: Árbol AVL rebalanceado
variant: avl
insert: [10, 20, 30, 40, 50, 25]
```

Ejemplo: Max-Heap con arreglo subyacente

```yaml
type: tree
title: Max-Heap con arreglo subyacente
variant: heap
values: [90, 70, 80, 30]
showArray: true
inkSaver: false
```

- Para seguir las rotaciones de un AVL paso a paso, ver [`avl`](#avl); para las operaciones de un montículo, [`heap`](#heap).

<a id="heap"></a>

## `heap`: Montículo (heap)

*Estructuras de datos y algoritmos › Árboles y tablas.* Min-heaps y max-heaps construidos por inserciones o con heapify, con sus operaciones.

Dibuja un montículo binario como árbol y como arreglo, con el índice de cada posición. Se
construye insertando los valores de a uno (con *sift-up*), con el algoritmo de Floyd
(*heapify*) o tal como se da, y después aplica las operaciones de `ops`. Con `steps` muestra
el estado después de cada paso.

**Calcula:** La reubicación de cada inserción (flotar) y de cada extracción (hundir). El heapify de Floyd, desde la mitad del arreglo hacia la raíz.

| Campo | Tipo | Valores · por defecto | Descripción |
| :--- | :--- | :--- | :--- |
| `values` | lista |  | Los valores iniciales. |
| `kind` | texto | `min`, `max` · por defecto `min` | `min` (la raíz es el menor) o `max`. |
| `build` | texto | `none`, `insert`, `heapify` · por defecto `insert` | `insert` (de a uno), `heapify` (Floyd) o `none` (el arreglo tal como está). |
| `ops` | lista |  | Operaciones a aplicar después: `"insert 12"` y `"extract"`. |
| `steps` | sí/no |  | Dibuja el estado después de cada inserción y de cada operación (hasta 16). |
| `blank` | sí/no |  | Deja en blanco el estado final (árbol y arreglo), para completar. |

**Modo ejercicio:** Campos: `blank`.

Ejemplo: Min-heap construido por inserciones

```yaml
type: heap
title: Min-heap construido por inserciones
values: [15, 10, 20, 8, 25, 3]
ops: ["extract", "insert 12"]
steps: true
```

Ejemplo: Ejercicio: heapify de un max-heap

```yaml
type: heap
title: "Ejercicio: heapify de un max-heap"
kind: max
build: heapify
values: [4, 10, 3, 5, 1, 8, 9]
blank: true
```

<a id="avl"></a>

## `avl`: Árbol AVL

*Estructuras de datos y algoritmos › Árboles y tablas.* Inserciones y eliminaciones en un AVL, con las rotaciones que hace cada una.

Inserta las claves en un AVL vacío, de a una, y elimina las de `delete`. Dibuja el árbol
final con el factor de balance de cada nodo y, con `steps`, el árbol después de cada operación
con el nombre de la rotación que la rebalanceó.

**Calcula:** Las rotaciones simples y dobles (izquierda-derecha, derecha-izquierda) de cada operación. El factor de balance (altura izquierda menos altura derecha) de cada nodo.

| Campo | Tipo | Valores · por defecto | Descripción |
| :--- | :--- | :--- | :--- |
| `keys` | lista |  | Las claves a insertar, en orden (hasta 40). Las repetidas se ignoran. |
| `delete` | lista |  | Claves a eliminar después de las inserciones. |
| `steps` | texto | `none`, `rotations`, `all` · por defecto `none` | `none` (sólo el final), `rotations` (los pasos que rotan) o `all` (todos). |
| `balance` | sí/no |  | Muestra el factor de balance de cada nodo (por omisión, sí). |
| `blank` | sí/no |  | Deja en blanco el árbol final, para completar. |

**Modo ejercicio:** Campos: `blank`.

Ejemplo: Árbol AVL con rotaciones

```yaml
type: avl
title: Árbol AVL con rotaciones
keys: [10, 20, 30, 40, 50, 25]
steps: rotations
```

<a id="bplus_tree"></a>

## `bplus_tree`: Árbol B+

*Estructuras de datos y algoritmos › Árboles y tablas.* Inserciones en un árbol B+, con las divisiones de hojas y nodos internos.

Inserta las claves en un árbol B+ y dibuja sus niveles: los nodos internos con las claves
separadoras y las hojas, con todas las claves y enlazadas entre sí. Con `steps`, muestra el
árbol después de cada inserción que divide un nodo.

**Calcula:** La división de una hoja (sube una copia de la primera clave de la derecha) y la de un nodo interno (sube la clave del medio).

| Campo | Tipo | Valores · por defecto | Descripción |
| :--- | :--- | :--- | :--- |
| `keys` | lista |  | Las claves a insertar, en orden (hasta 60). |
| `order` | número |  | Orden del árbol, la cantidad máxima de hijos (por omisión 4). |
| `maxKeys` | número |  | Claves máximas por nodo, si no es `order − 1`. |
| `steps` | sí/no |  | Dibuja el árbol después de cada inserción que divide un nodo (hasta 12). |
| `blank` | sí/no |  | Deja en blanco las claves del árbol final. |

**Modo ejercicio:** Campos: `blank`.

Ejemplo: Árbol B+ de orden 4

```yaml
type: bplus_tree
title: Árbol B+ de orden 4
order: 4
keys: [5, 15, 25, 35, 45, 20, 30, 10, 40, 50, 55]
```

<a id="hash_table"></a>

## `hash_table`: Tabla de hash

*Estructuras de datos y algoritmos › Árboles y tablas.* Inserciones en una tabla de hash, con encadenamiento o direccionamiento abierto.

Inserta las claves en una tabla de `size` cubetas con h(k) = k mod m y dibuja la tabla
resultante junto al registro de cada inserción: su hash, la secuencia de sondeos y la
posición final. Las colisiones se resuelven por encadenamiento o con sondeo lineal,
cuadrático o doble hashing.

**Calcula:** El hash de cada clave (los textos, por la suma de sus caracteres o el polinomio de base 31). La secuencia de sondeos y la posición final, o «sin lugar» si la tabla está llena. El factor de carga α y la cantidad de inserciones con colisión.

| Campo | Tipo | Valores · por defecto | Descripción |
| :--- | :--- | :--- | :--- |
| `size` | número |  | Cantidad de cubetas m (de 1 a 101). |
| `keys` | lista |  | Las claves, enteros o textos, en el orden en que se insertan. |
| `collision` | texto | `chaining`, `linear`, `quadratic`, `double` · por defecto `linear` | `chaining`, `linear`, `quadratic` o `double`. |
| `R` | número |  | En doble hashing, el R de h₂(k) = R − (k mod R); por omisión, el primo menor que m. |
| `stringHash` | texto | `sum`, `poly` · por defecto `sum` | Hash de los textos: `sum` (suma de códigos) o `poly` (base 31). |
| `blank` | sí/no |  | Deja en blanco la tabla y el registro, para completar. |

**Modo ejercicio:** Campos: `blank`.

Ejemplo: Hashing con sondeo lineal

```yaml
type: hash_table
title: Hashing con sondeo lineal
size: 11
keys: [22, 1, 13, 11, 24, 33, 18, 42, 31]
collision: linear
```

Ejemplo: Hashing con encadenamiento

```yaml
type: hash_table
title: Hashing con encadenamiento
size: 7
keys: [50, 700, 76, 85, 92, 73, 101]
collision: chaining
```

<a id="complexity"></a>

## `complexity`: Complejidad

*Estructuras de datos y algoritmos › Análisis de algoritmos.* Curvas de crecimiento para comparar órdenes, y jerarquías de clases de complejidad.

Con la variante `growth` (por omisión) dibuja curvas de crecimiento para comparar órdenes;
con `blankLabels` salen rotuladas A, B, C… para asociarlas con su clase. Una curva con
`points` son mediciones, para clasificarlas. Con `classes`, dibuja conjuntos anidados de
clases de complejidad.

| `variant` | Qué dibuja |
| :--- | :--- |
| `growth` | Por omisión. Curvas de crecimiento. |
| `classes` | Conjuntos anidados de clases. |

| Campo | Tipo | Valores · por defecto | Descripción |
| :--- | :--- | :--- | :--- |
| `curves` | lista |  | Las curvas: `1`, `log n`, `sqrt n`, `n`, `n log n`, `n^2`, `n^3`, `2^n`, `n!` (también como `O(n²)`), o `{label, points}` para mediciones. |
| `variant` | texto |  | Variante, ver arriba. |
| `maxN` | número |  | El n máximo del eje horizontal. |
| `maxY` | número |  | El máximo del eje vertical; las curvas que se salen se cortan. |
| `xLabel` | texto |  | Rótulo del eje horizontal. |
| `yLabel` | texto |  | Rótulo del eje vertical. |
| `blankLabels` | sí/no |  | Rotula las curvas A, B, C… en el orden de `curves`. |
| `classes` | lista |  | En `classes`, la jerarquía, de la más chica a la más grande (por omisión los órdenes comunes; también `[P, NP, EXPTIME]`). |
| `items` | objeto |  | En `classes`, ejemplos dentro de cada clase: `{"O(n)": [búsqueda lineal]}`. |
| `blankClasses` | sí/no |  | Deja en blanco los nombres de las clases. |

**Modo ejercicio:** Campos: `blankLabels`, `blankClasses`.

Ejemplo: Órdenes de crecimiento Θ(n²)

```yaml
type: complexity
title: Órdenes de crecimiento Θ(n²)
curves:
  - "Θ(n²)"
  - "n log n"
  - label: medido
    points: [[2, 5], [10, 90]]
```

Ejemplo: Clases de complejidad P vs NP

```yaml
type: complexity
title: Clases de complejidad P vs NP
variant: classes
classes: ["P", "NP"]
items:
  P: ["ordenar"]
inkSaver: false
```

<a id="flowchart"></a>

## `flowchart`: Diagrama de flujo

*Estructuras de datos y algoritmos › Análisis de algoritmos.* Diagramas de flujo armados a partir de la estructura del algoritmo.

Arma el diagrama de flujo a partir de la estructura del algoritmo: pasos, entradas y salidas,
condicionales y ciclos anidados. Cada paso con `blank: true` sale vacío para completar.

Los saltos van a su destino por un carril a la derecha: `break` a la salida del lazo más
interno, `continue` a su condición y `return` al fin; después de un salto no hay arista hacia
el paso siguiente. Un `try` dibuja su región, los `except` a la derecha (a los que se llega con
flechas con guiones, las excepcionales), el `else` en el camino sin excepción y el `finally`,
al que llegan todos los caminos; con `propagate: true`, una excepción no capturada pasa por
el `finally` y se propaga.

| Campo | Tipo | Valores · por defecto | Descripción |
| :--- | :--- | :--- | :--- |
| `steps` | lista |  | Los pasos: un texto, `{io}` (entrada o salida), `{step}`, `{if, then, else}`, `{while, do}`, `{for, do}`, `{doWhile, do}` (el cuerpo antes de la condición), `{break: true}`, `{continue: true}`, `{return: expresión}` o `{try, except, else, finally, propagate}`; cada uno con `blank`. |
| `start` | texto |  | Rótulo del inicio (por omisión `Inicio`); `false` lo quita. |
| `end` | texto |  | Rótulo del fin (por omisión `Fin`); `false` lo quita. |
| `blankValues` | sí/no |  | Deja en blanco el texto de todos los pasos. |

**Modo ejercicio:** Campos: `blankValues`.

Ejemplo: Bucle con condición < & >

```yaml
type: flowchart
title: "Bucle con condición < & >"
steps:
  - {io: "leer n"}
  - "i = 0"
  - while: "i < n && n > 0"
    do:
      - if: "par(i)"
        then: ["s = s + i"]
        else: ["s = s - i"]
      - "i = i + 1"
```

Ejemplo: Lazo do-while: el cuerpo se ejecuta al menos una vez

```yaml
type: flowchart
title: "Lazo do-while: el cuerpo se ejecuta al menos una vez"
steps:
  - "i = 0"
  - doWhile: "i < 3"
    do:
      - "procesar(i)"
      - "i = i + 1"
  - {io: "mostrar i"}
```

Ejemplo: break en el lazo interno y continue en el externo

```yaml
type: flowchart
title: break en el lazo interno y continue en el externo
steps:
  - for: "i en range(3)"
    do:
      - if: "i == 1"
        then:
          - continue: true
      - for: "j en range(3)"
        do:
          - if: "j == i"
            then:
              - break: true
          - io: "MOSTRAR i, j"
      - "total = total + 1"
```

Ejemplo: def buscar(xs, x) (para completar)

```yaml
type: flowchart
title: "def buscar(xs, x) (para completar)"
steps:
  - for: "v en xs"
    do:
      - if: "v == x"
        then:
          - {return: "True", blank: true}
  - {return: "False", blank: true}
inkSaver: false
```

Ejemplo: try / except / else / finally

```yaml
type: flowchart
title: try / except / else / finally
steps:
  - {io: "LEER texto"}
  - try:
      - step: "n = int(texto)"
    except:
      - matches: ValueError
        do:
          - io: "MOSTRAR error"
    else:
      - io: "MOSTRAR n"
    finally:
      - step: "Registrar intento"
    propagate: true
```

Ejemplo: try / finally (para completar)

```yaml
type: flowchart
title: try / finally (para completar)
blankValues: true
steps:
  - try:
      - step: "f = abrir(ruta)"
      - step: "procesar(f)"
    finally:
      - step: "f.close()"
    propagate: true
```

- `break` y `continue` fuera de un lazo, y `return` sin el fin del diagrama (`end: false`), son errores.
- `try`: `except` es una lista de `{matches, do}` (`matches` es un tipo, una lista de tipos o nada); `else` necesita algún `except`, y un `try` necesita `except` o `finally`.
- Por ahora, un salto dentro de un `try` o un `try` dentro de otro no se dibujan.

<a id="cost_table"></a>

## `cost_table`: Tabla de costos

*Estructuras de datos y algoritmos › Análisis de algoritmos.* El análisis de un algoritmo línea por línea, con el costo y las veces de cada una.

Dibuja el análisis línea por línea de un algoritmo: el código (con su sangría), el costo de
cada línea y cuántas veces se ejecuta, y el total.

| Campo | Tipo | Valores · por defecto | Descripción |
| :--- | :--- | :--- | :--- |
| `lines` | lista |  | Las líneas: `[código, costo, veces]` o `{code, cost, times}`. |
| `total` | texto |  | El costo total, debajo. |
| `blankCosts` | sí/no |  | Deja los costos en blanco. |
| `blankTimes` | sí/no |  | Deja en blanco las veces. |
| `blankTotal` | sí/no |  | Deja el total en blanco. |

**Modo ejercicio:** Campos: `blankCosts`, `blankTimes`, `blankTotal`.

Ejemplo: Análisis línea a línea (evaluación)

```yaml
type: cost_table
title: Análisis línea a línea (evaluación)
lines:
  - ["for (;;)", "c1", "n"]
  - ["    x++;", "c2", "n - 1"]
total: 2n
blankCosts: true
blankTotal: true
```

<a id="trace_table"></a>

## `trace_table`: Tabla de traza

*Estructuras de datos y algoritmos › Análisis de algoritmos.* Pruebas de escritorio de variables y trazas de operaciones sobre una estructura.

Dibuja una prueba de escritorio: columnas con las variables y una fila por paso. Las filas
que faltan hasta `rowCount` y los `null` quedan para completar. También dibuja las tablas de
traza de statics, con una secuencia de operaciones sobre una estructura de capacidad fija.

| `variant` | Qué dibuja |
| :--- | :--- |
| `stack` | Sólo cambia los rótulos por omisión, para trazas de una pila. |
| `program` | La prueba de escritorio de un programa: cada fila es `{step, stage, input, check, values, output, blanks}` y la tabla suma las columnas de etapa, paso, entrada, condición y salida que se usen. Una cadena vacía se ve como `''`, sin entrada o sin emisión como `—`, y `null` es un casillero para completar. |

| Campo | Tipo | Valores · por defecto | Descripción |
| :--- | :--- | :--- | :--- |
| `columns` | lista |  | Las columnas (las variables). |
| `rows` | lista |  | Las filas; `null` o una fila corta quedan para completar. |
| `rowCount` | número |  | Cantidad total de filas; las que faltan quedan vacías. |
| `operations` | lista |  | Las operaciones de cada fila, en la primera columna (o la traza de statics). |
| `operationHeader` | texto |  | Encabezado de la columna de operaciones. |
| `capacity` | número |  | En la traza de statics, la capacidad de la estructura. |
| `pointerHeader` | texto |  | En la traza de statics, el encabezado de la columna del puntero. |
| `pointerLabel` | texto |  | En la traza de statics, el nombre del puntero. |
| `variant` | texto |  | Variante, ver arriba. |
| `blankValues` | sí/no |  | Deja los valores en blanco (en `program`, también la condición y la salida). |
| `condition` | texto |  | En `program`, el encabezado de la columna de la condición (por ejemplo `x != 0`). |
| `stepHeader` | texto |  | En `program`, el encabezado de la columna de pasos (por omisión `Paso`). |
| `inputHeader` | texto |  | En `program`, el encabezado de la entrada (por omisión `Entrada`). |
| `outputHeader` | texto |  | En `program`, el encabezado de la salida (por omisión `Salida`). |

**Modo ejercicio:** En `program`, `blanks` en una fila oculta esas celdas (`[output]`, `[check]`, o el nombre de
una variable): la respuesta no queda en el SVG. Campos: `blankValues`.

Ejemplo: Suma hasta leer 0

```yaml
type: trace_table
variant: program
title: "Suma hasta leer 0"
condition: "x != 0"
columns: [x, suma]
rows:
  - {stage: initial, step: "Primera lectura", input: 4, values: [4, 0]}
  - {stage: iteration, step: "Vuelta 1", check: true, values: [4, 4]}
  - {stage: iteration, step: "Lectura", input: 0, values: [0, 4]}
  - {stage: iteration, step: "Centinela", check: false, values: [0, 4]}
  - {stage: output, step: "Mostrar", output: 4}
```

Ejemplo: Cero vueltas (para completar)

```yaml
type: trace_table
variant: program
title: "Cero vueltas (para completar)"
condition: "x != 0"
columns: [x, suma]
rows:
  - {step: "Primera lectura", input: 0, values: [0, 0]}
  - {step: "Condición", check: false, values: [0, null], blanks: [check]}
  - {step: "Mostrar", output: 0, blanks: [output]}
  - {step: "print('')", output: ""}
inkSaver: false
```

Ejemplo: Tabla de seguimiento de variables

```yaml
type: trace_table
title: Tabla de seguimiento de variables
columns: ["i", "suma"]
rows: [[0, 7], [1, null]]
rowCount: 4
```

Ejemplo: Seguimiento de pila (statics)

```yaml
type: trace_table
title: Seguimiento de pila (statics)
variant: stack
capacity: 5
pointerLabel: "TOPE: ___"
operations: ["1. push(10)", "2. pop()"]
inkSaver: false
```

- En `program`, `stage` puede ser `initial` (estado inicial), `iteration` (iteración), `output` (salida) u otro texto; se escribe en la primera fila de cada tramo. Una fila sin `values` no muestra las variables.
- En `program` también valen las filas posicionales (una lista de valores).

<a id="dp_table"></a>

## `dp_table`: Tabla de programación dinámica

*Estructuras de datos y algoritmos › Análisis de algoritmos.* LCS, distancia de edición, mochila 0/1 y cambio de monedas, con el camino de reconstrucción.

Llena la tabla de programación dinámica del problema elegido y resalta el camino de
reconstrucción de la solución. Debajo escribe el resultado: la subsecuencia, las operaciones
de edición, los objetos elegidos o las monedas.

**Calcula:** `lcs`: la longitud y una subsecuencia común más larga. `edit`: la distancia de Levenshtein y las operaciones (sustituir, insertar, borrar). `knapsack`: el valor máximo y los objetos elegidos. `coins`: la mínima cantidad de monedas y cuáles.

| Campo | Tipo | Valores · por defecto | Descripción |
| :--- | :--- | :--- | :--- |
| `problem` | texto | `lcs`, `edit`, `knapsack`, `coins` · por defecto `lcs` | `lcs`, `edit`, `knapsack` o `coins`. |
| `a` | texto |  | En `lcs` y `edit`, el primer texto (hasta 24 caracteres). |
| `b` | texto |  | En `lcs` y `edit`, el segundo texto. |
| `weights` | lista |  | En `knapsack`, los pesos (de 1 a 15 objetos). |
| `values` | lista |  | En `knapsack`, los valores. |
| `capacity` | número |  | En `knapsack`, la capacidad de la mochila (de 1 a 40). |
| `coins` | lista |  | En `coins`, los valores de las monedas. |
| `amount` | número |  | En `coins`, el monto a formar (de 1 a 60). |
| `path` | sí/no |  | Resalta el camino de reconstrucción (por omisión, sí). |
| `blank` | sí/no |  | Deja la tabla en blanco (salvo la primera fila y columna) y el resultado. |

**Modo ejercicio:** Campos: `blank`.

Ejemplo: Subsecuencia común más larga

```yaml
type: dp_table
title: Subsecuencia común más larga
problem: lcs
a: ABCBDAB
b: BDCABA
```

Ejemplo: Distancia de edición (Levenshtein)

```yaml
type: dp_table
title: Distancia de edición (Levenshtein)
problem: edit
a: kitten
b: sitting
```

Ejemplo: Ejercicio: mochila 0/1

```yaml
type: dp_table
title: "Ejercicio: mochila 0/1"
problem: knapsack
weights: [1, 3, 4, 5]
values: [1, 4, 5, 7]
capacity: 7
blank: true
```

<a id="dijkstra"></a>

## `dijkstra`: Dijkstra

*Estructuras de datos y algoritmos › Análisis de algoritmos.* La tabla de iteraciones del algoritmo de Dijkstra y los caminos mínimos.

Ejecuta Dijkstra desde `source` y dibuja la tabla de iteraciones: en cada paso, el vértice
que se visita y la distancia y el predecesor de cada vértice (✓ si ya fue visitado). Debajo,
la distancia y el camino mínimo a cada destino.

**Calcula:** Las distancias y predecesores de cada iteración; los empates se rompen por el orden de los vértices.

| Campo | Tipo | Valores · por defecto | Descripción |
| :--- | :--- | :--- | :--- |
| `edges` | lista |  | Las aristas: `"A-B:4"` (no dirigida), `"A->B:4"` (dirigida) o `{from, to, w}`. |
| `source` | texto | por defecto `` | El vértice de origen (por omisión, el primero). |
| `vertices` | lista |  | El orden de los vértices, si no sale de las aristas. |
| `directed` | sí/no |  | Toma como dirigidas las aristas escritas con `-`. |
| `paths` | sí/no |  | Agrega la tabla de caminos mínimos (por omisión, sí). |
| `blank` | sí/no |  | Deja en blanco las iteraciones y los caminos. |

**Modo ejercicio:** Campos: `blank`.

Ejemplo: Caminos mínimos con Dijkstra

```yaml
type: dijkstra
title: Caminos mínimos con Dijkstra
source: A
edges: ["A-B:4", "A-C:2", "C-B:1", "B-D:5", "C-D:8", "C-E:10", "D-E:2"]
```

- Con pesos negativos es un error, porque Dijkstra no los admite.
- El grafo en sí no se dibuja; para eso alcanza con Mermaid o Graphviz.

<a id="turing"></a>

## `turing`: Máquina de Turing

*Estructuras de datos y algoritmos › Computabilidad.* La traza de configuraciones de una máquina de Turing y su tabla de transiciones.

Simula una máquina de Turing de una cinta sobre la entrada y dibuja cada configuración: el
paso, el estado y la cinta con el cabezal resaltado. Debajo, si acepta, rechaza o no se
detuvo, y la tabla de transiciones δ.

| Campo | Tipo | Valores · por defecto | Descripción |
| :--- | :--- | :--- | :--- |
| `transitions` | lista |  | Las transiciones: `"q0,1 -> q1,X,R"` (estado, símbolo leído, estado nuevo, símbolo escrito, movimiento `R`, `L` o `S`), o un mapa `"q0,1": "q1,X,R"`. |
| `input` | texto |  | La entrada (hasta 40 símbolos). |
| `start` | texto |  | El estado inicial (por omisión `q0`). |
| `accept` | lista |  | Los estados de aceptación. |
| `blankSymbol` | texto |  | El símbolo blanco (por omisión `_`). |
| `maxSteps` | número |  | Pasos máximos de la simulación (de 1 a 200, por omisión 40). |
| `table` | sí/no |  | Agrega la tabla de transiciones (por omisión, sí). |
| `blank` | sí/no |  | Dibuja sólo la configuración inicial, para completar la traza. |

**Modo ejercicio:** Campos: `blankSymbol`, `blank`.

Ejemplo: Máquina de Turing que suma 1 en binario

```yaml
type: turing
title: Máquina de Turing que suma 1 en binario
input: "1011"
start: q0
accept: [qf]
transitions:
  - "q0,0 -> q0,0,R"
  - "q0,1 -> q0,1,R"
  - "q0,_ -> q1,_,L"
  - "q1,1 -> q1,0,L"
  - "q1,0 -> qf,1,S"
  - "q1,_ -> qf,1,S"
```

<a id="truth_table"></a>

## `truth_table`: Tabla de verdad

*Sistemas de computación › Sistemas digitales.* Tablas de verdad de una o varias funciones, con columnas intermedias y formas canónicas.

Evalúa una o varias expresiones booleanas en todas las combinaciones de sus variables y
dibuja la tabla, con los unos resaltados. Con `steps`, agrega una columna por cada
subexpresión, para mostrar cómo se llega al resultado; con `canonical`, la suma de mintérminos
y el producto de maxtérminos.

**Calcula:** El valor de cada salida y de cada subexpresión en cada fila. Las formas canónicas Σm(…) y ΠM(…).

| Campo | Tipo | Valores · por defecto | Descripción |
| :--- | :--- | :--- | :--- |
| `expr` | texto |  | La expresión, como `A·B + !C`. NOT: `!A`, `~A`, `¬A`, `A'`; AND: `&`, `·`, `*` o yuxtaposición (`A B`, `AB'C`); OR: `+`, `\|`; XOR: `^`, `⊕`; también `AND`, `OR`, `NOT`, `XOR`, `NAND`, `NOR`, `XNOR`. |
| `outputs` | lista |  | Varias salidas que comparten variables: `[{name, expr}]` o una lista de expresiones (hasta 8). |
| `name` | texto |  | El nombre de la salida (por omisión `F`). |
| `vars` | lista |  | Las variables, en orden (por omisión, alfabético). Obligatorias con `minterms`. |
| `minterms` | lista |  | La función por sus mintérminos, en lugar de `expr`. |
| `maxterms` | lista |  | La función por sus maxtérminos. |
| `dontcares` | lista |  | Las combinaciones indiferentes, que salen como X. |
| `steps` | sí/no |  | Agrega una columna por cada subexpresión (hasta 10). |
| `canonical` | sí/no |  | Escribe las formas canónicas debajo. |
| `index` | sí/no |  | La columna con el número de mintérmino (por omisión, sí). |
| `notation` | texto |  | Cómo se escribe la negación en los encabezados: `overline` (barra), `prime` (A') o `neg` (¬A). |
| `multiLetterVars` | sí/no |  | Con `true`, `AB` es una variable; por omisión, varias mayúsculas juntas son variables de una letra. |
| `blank` | sí/no |  | Deja en blanco las salidas, las columnas intermedias y las formas canónicas. |

**Modo ejercicio:** Campos: `blank`.

Ejemplo: Función mayoría de tres entradas

```yaml
type: truth_table
title: Función mayoría de tres entradas
expr: "A·B + A·C + B·C"
name: M
canonical: true
```

Ejemplo: Tabla con columnas intermedias

```yaml
type: truth_table
title: Tabla con columnas intermedias
outputs:
  - { name: F, expr: "!(A + B) · C" }
  - { name: G, expr: "A XOR C" }
steps: true
```

Ejemplo: Ejercicio: completar la salida

```yaml
type: truth_table
title: "Ejercicio: completar la salida"
vars: [A, B, C, D]
minterms: [1, 3, 7, 11, 15]
dontcares: [0, 2, 5]
blank: true
canonical: true
```

<a id="karnaugh"></a>

## `karnaugh`: Mapa de Karnaugh

*Sistemas de computación › Sistemas digitales.* Mapas de 2 a 4 variables con los agrupamientos mínimos y la expresión simplificada.

Dibuja el mapa de Karnaugh de la función, con las filas y columnas en código Gray y el número
de mintérmino en cada celda. Minimiza la función (Quine-McCluskey con cobertura exacta),
dibuja cada grupo, también los que dan la vuelta por los bordes, y escribe la suma de
productos o el producto de sumas mínimo.

**Calcula:** Los implicantes primos y una cobertura mínima (menos términos y, a igualdad, menos literales). La expresión mínima en SOP (agrupando unos) o POS (agrupando ceros), usando las indiferentes.

| Campo | Tipo | Valores · por defecto | Descripción |
| :--- | :--- | :--- | :--- |
| `expr` | texto |  | La función como expresión (ver [`truth_table`](#truth_table)). |
| `vars` | lista |  | Las variables (de 2 a 4); las primeras van en las filas. |
| `minterms` | lista |  | La función por sus mintérminos. |
| `maxterms` | lista |  | La función por sus maxtérminos. |
| `dontcares` | lista |  | Las combinaciones indiferentes, que salen como X. |
| `name` | texto |  | El nombre de la función (por omisión `F`). |
| `form` | texto | `sop`, `pos` · por defecto `sop` | `sop` (suma de productos) o `pos` (producto de sumas). |
| `groups` | sí/no |  | Dibuja los agrupamientos (por omisión, sí). |
| `implicants` | sí/no |  | Lista debajo cada implicante con los mintérminos que cubre. |
| `result` | sí/no |  | Escribe la expresión simplificada (por omisión, sí). |
| `notation` | texto |  | `overline`, `prime` o `neg`. |
| `multiLetterVars` | sí/no |  | Con `true`, `AB` es una variable. |
| `blank` | sí/no |  | Oculta los grupos y deja la expresión en blanco; las celdas se ven. |
| `blankGroups` | sí/no |  | Oculta sólo los grupos. |
| `blankCells` | sí/no |  | Oculta también los valores de las celdas. |

**Modo ejercicio:** Campos: `blank`, `blankGroups`, `blankCells`.

Ejemplo: Mapa de Karnaugh de 4 variables

```yaml
type: karnaugh
title: Mapa de Karnaugh de 4 variables
vars: [A, B, C, D]
minterms: [0, 1, 2, 5, 6, 7, 8, 9, 10, 14]
implicants: true
```

Ejemplo: Con condiciones indiferentes (forma POS)

```yaml
type: karnaugh
title: Con condiciones indiferentes (forma POS)
vars: [A, B, C, D]
minterms: [1, 3, 7, 11, 15]
dontcares: [0, 2, 5]
form: pos
```

Ejemplo: Ejercicio: agrupar y simplificar

```yaml
type: karnaugh
title: "Ejercicio: agrupar y simplificar"
expr: "A'B'C + A'BC + AB'C + ABC'"
blank: true
```

<a id="logic_circuit"></a>

## `logic_circuit`: Circuito lógico

*Sistemas de computación › Sistemas digitales.* El circuito con compuertas que implementa una expresión booleana.

Convierte la expresión en un circuito con compuertas AND, OR, XOR, NAND, NOR, XNOR y NOT. Las
entradas bajan por rieles verticales a la izquierda y cada compuerta se ubica por su
profundidad; la negación de una AND, OR o XOR se dibuja como la compuerta negada.

| Campo | Tipo | Valores · por defecto | Descripción |
| :--- | :--- | :--- | :--- |
| `expr` | texto |  | La expresión (ver [`truth_table`](#truth_table)). |
| `output` | texto |  | El nombre de la salida (por omisión `F`). |
| `vars` | lista |  | El orden de las variables en los rieles. |
| `rails` | sí/no |  | Rieles verticales para las entradas (por omisión, sí); sin ellos, cada entrada se rotula en su compuerta. |
| `basicGates` | sí/no |  | Sólo AND, OR, XOR y NOT (una NAND sale como AND y NOT). |
| `showExpr` | sí/no |  | Escribe la expresión debajo (por omisión, sí). |
| `notation` | texto |  | `overline`, `prime` o `neg`. |
| `multiLetterVars` | sí/no |  | Con `true`, `AB` es una variable. |

Ejemplo: Circuito de la función mayoría

```yaml
type: logic_circuit
title: Circuito de la función mayoría
expr: "A·B + A·C + B·C"
output: M
```

Ejemplo: Compuertas negadas y XOR

```yaml
type: logic_circuit
title: Compuertas negadas y XOR
expr: "!(A·B) ⊕ (C + !D)"
output: Y
```

<a id="pipeline"></a>

## `pipeline`: Pipeline

*Sistemas de computación › Arquitectura de computadoras.* El diagrama de ciclos de un pipeline de 5 etapas, con riesgos, burbujas y forwarding.

Ubica cada instrucción en las etapas IF, ID, EX, MEM y WB ciclo por ciclo. Detecta las
dependencias de datos (RAW) entre instrucciones e inserta las burbujas necesarias, con o sin
forwarding; dibuja las flechas de cada dependencia y debajo los ciclos totales, las burbujas y
el CPI.

**Calcula:** Sin forwarding, el consumidor lee en ID en el mismo ciclo del WB del productor (escritura en la primera mitad, lectura en la segunda). Con forwarding, de EX a EX; después de un load, de MEM a EX, con una burbuja (load-use). Mientras una instrucción espera en ID, la siguiente queda retenida en IF.

| Campo | Tipo | Valores · por defecto | Descripción |
| :--- | :--- | :--- | :--- |
| `instructions` | lista |  | Las instrucciones (hasta 40): `"lw $t0, 0($s0)"`, `"add $t1, $t0, $t2"`. Reconoce cargas, almacenamientos, saltos y operaciones; `$zero` no genera dependencias. También `{text, dest, src, load}` para fijar los registros. |
| `forwarding` | sí/no |  | Con forwarding (por omisión, sí). |
| `arrows` | sí/no |  | Dibuja las flechas de las dependencias (por omisión, sí). |
| `summary` | sí/no |  | Escribe los ciclos totales, las burbujas y el CPI (por omisión, sí). |
| `hazards` | sí/no |  | Lista las dependencias RAW debajo (por omisión, sí). |
| `blank` | sí/no |  | Deja la grilla de ciclos vacía y los resultados en blanco. |

**Modo ejercicio:** Campos: `blank`.

Ejemplo: Pipeline MIPS de 5 etapas con forwarding

```yaml
type: pipeline
title: Pipeline MIPS de 5 etapas con forwarding
instructions:
  - "lw $t0, 0($s0)"
  - "add $t1, $t0, $t2"
  - "sub $t3, $t1, $t4"
  - "sw $t3, 4($s0)"
  - "and $t5, $t6, $t7"
```

Ejemplo: El mismo código sin forwarding

```yaml
type: pipeline
title: El mismo código sin forwarding
forwarding: false
instructions:
  - "lw $t0, 0($s0)"
  - "add $t1, $t0, $t2"
  - "sub $t3, $t1, $t4"
  - "sw $t3, 4($s0)"
  - "and $t5, $t6, $t7"
```

- No modela riesgos de control (saltos) ni estructurales.

<a id="ieee754"></a>

## `ieee754`: IEEE 754

*Sistemas de computación › Arquitectura de computadoras.* La codificación de un número en punto flotante, en los dos sentidos.

Dibuja los campos de signo, exponente y mantisa de un número en precisión media, simple o
doble, y los pasos: el número en binario y normalizado, el exponente sesgado, la mantisa, el
hexadecimal, el valor que queda guardado y el error de redondeo. Parte de un valor decimal o de
los bits.

**Calcula:** La codificación con redondeo al par más cercano, también para media precisión. Los casos especiales: cero, subnormales, infinito y NaN.

| Campo | Tipo | Valores · por defecto | Descripción |
| :--- | :--- | :--- | :--- |
| `value` | número |  | El número: `-6.75`, `0.1`, o `"inf"`, `"-inf"`, `"nan"`. |
| `bits` | texto |  | Los bits, en hexadecimal (`"0x41460000"`) o en binario con todos sus dígitos. |
| `hex` | texto |  | Lo mismo que `bits`. |
| `format` | texto | `half`, `single`, `double`, `16`, `32`, `64` · por defecto `single` | `half` (16 bits), `single` (32) o `double` (64). |
| `blank` | sí/no |  | Si se dio el valor, oculta los bits y los pasos; si se dieron los bits, oculta los pasos. |

**Modo ejercicio:** Campos: `blank`.

Ejemplo: −6,75 en precisión simple

```yaml
type: ieee754
title: "−6,75 en precisión simple"
value: -6.75
```

Ejemplo: 0,1 no es representable exactamente

```yaml
type: ieee754
title: "0,1 no es representable exactamente"
value: 0.1
format: single
```

Ejemplo: Ejercicio: decodificar

```yaml
type: ieee754
title: "Ejercicio: decodificar"
bits: "0x41460000"
blank: true
```

<a id="cache"></a>

## `cache`: Memoria caché

*Sistemas de computación › Arquitectura de computadoras.* La simulación de una caché directa o asociativa sobre una secuencia de accesos.

Divide la dirección en tag, índice y offset, simula cada acceso y dibuja una tabla con su
tag, índice, offset, si fue acierto o fallo (y de qué tipo) y qué bloque desalojó. Debajo, la
tasa de aciertos y el contenido final de la caché.

**Calcula:** Los bits de tag, índice y offset a partir de los tamaños. Aciertos y fallos con reemplazo LRU o FIFO. El tipo de cada fallo: forzoso (primer acceso al bloque), de capacidad (fallaría también en una caché totalmente asociativa del mismo tamaño) o de conflicto.

| Campo | Tipo | Valores · por defecto | Descripción |
| :--- | :--- | :--- | :--- |
| `addressBits` | número |  | Bits de la dirección (de 4 a 64, por omisión 16). |
| `cacheSize` | número |  | Tamaño de la caché en bytes, potencia de 2 (`64`, `"1KB"`). |
| `blockSize` | número |  | Tamaño de bloque en bytes, potencia de 2. |
| `associativity` | número |  | `1` (correspondencia directa), 2, 4… vías o `"full"` (totalmente asociativa). |
| `policy` | texto | `lru`, `fifo` · por defecto `lru` | `lru` o `fifo`. |
| `accesses` | lista |  | Las direcciones accedidas: `"0x040"`, `"0b1010"` o enteros (hasta 64). |
| `missTypes` | sí/no |  | La columna con el tipo de fallo (por omisión, sí). |
| `finalState` | sí/no |  | La tabla con el contenido final (por omisión, sí; hasta 64 bloques). |
| `blank` | sí/no |  | Deja en blanco tag, índice, offset, resultado y el contenido final. |

**Modo ejercicio:** Campos: `blank`.

Ejemplo: Caché de correspondencia directa

```yaml
type: cache
title: Caché de correspondencia directa
addressBits: 12
cacheSize: 64
blockSize: 8
associativity: 1
accesses: ["0x000", "0x004", "0x010", "0x040", "0x000", "0x048", "0x010", "0x080", "0x044"]
```

Ejemplo: La misma secuencia con 2 vías (LRU)

```yaml
type: cache
title: La misma secuencia con 2 vías (LRU)
addressBits: 12
cacheSize: 64
blockSize: 8
associativity: 2
accesses: ["0x000", "0x004", "0x010", "0x040", "0x000", "0x048", "0x010", "0x080", "0x044"]
finalState: false
```

<a id="cpu_scheduling"></a>

## `cpu_scheduling`: Planificación de CPU

*Sistemas de computación › Sistemas operativos.* El diagrama de Gantt y los tiempos de FCFS, SJF, SRTF, Round Robin y prioridades.

Simula la planificación de los procesos y dibuja el diagrama de Gantt, con la CPU ociosa
cuando no hay nadie listo y una flecha en cada llegada. Debajo, la tabla con el instante de
fin y los tiempos de retorno, espera y respuesta de cada proceso, y sus promedios.

**Calcula:** FCFS, SJF y prioridad no expropiativos; SRTF y prioridad expropiativos; Round Robin con quantum. En Round Robin, los que llegan durante un quantum entran a la cola antes que el expropiado. Los empates se rompen por llegada y después por el orden de la lista.

| Campo | Tipo | Valores · por defecto | Descripción |
| :--- | :--- | :--- | :--- |
| `processes` | lista |  | Los procesos (hasta 26): `{name, arrival, burst, priority}`. `arrival` es 0 por omisión; la prioridad más alta es el número menor. |
| `algorithm` | texto | `fcfs`, `sjf`, `srtf`, `rr`, `priority`, `priority_preemptive` · por defecto `fcfs` | `fcfs`, `sjf`, `srtf`, `rr`, `priority` o `priority_preemptive`. |
| `quantum` | número |  | El quantum de Round Robin (por omisión 2). |
| `blank` | sí/no |  | Deja el Gantt vacío (con la escala de tiempo) y los tiempos en blanco. |

**Modo ejercicio:** Campos: `blank`.

Ejemplo: Planificación SRTF

```yaml
type: cpu_scheduling
title: Planificación SRTF
algorithm: srtf
processes:
  - { name: P1, arrival: 0, burst: 8 }
  - { name: P2, arrival: 1, burst: 4 }
  - { name: P3, arrival: 2, burst: 9 }
  - { name: P4, arrival: 3, burst: 5 }
```

Ejemplo: Round Robin

```yaml
type: cpu_scheduling
title: Round Robin
algorithm: rr
quantum: 3
processes:
  - { name: A, arrival: 0, burst: 5 }
  - { name: B, arrival: 1, burst: 3 }
  - { name: C, arrival: 2, burst: 6 }
  - { name: D, arrival: 9, burst: 2 }
```

Ejemplo: Ejercicio: prioridad no expropiativa

```yaml
type: cpu_scheduling
title: "Ejercicio: prioridad no expropiativa"
algorithm: priority
blank: true
processes:
  - { name: P1, arrival: 0, burst: 10, priority: 3 }
  - { name: P2, arrival: 0, burst: 1, priority: 1 }
  - { name: P3, arrival: 0, burst: 2, priority: 4 }
  - { name: P4, arrival: 0, burst: 1, priority: 5 }
  - { name: P5, arrival: 0, burst: 5, priority: 2 }
```

<a id="page_table"></a>

## `page_table`: Tabla de páginas

*Sistemas de computación › Sistemas operativos.* La traducción de direcciones lógicas a físicas con paginación.

Dibuja la tabla de páginas (página, marco, bit de validez) y la traducción de cada dirección
lógica: número de página, desplazamiento, marco y dirección física, o el fallo de página.

**Calcula:** página = dirección div tamaño; offset = dirección mod tamaño; física = marco × tamaño + offset.

| Campo | Tipo | Valores · por defecto | Descripción |
| :--- | :--- | :--- | :--- |
| `pageSize` | número |  | Tamaño de página en bytes, potencia de 2. |
| `table` | lista |  | El marco de cada página; `null` es una página que no está en memoria. |
| `addresses` | lista |  | Las direcciones lógicas, en decimal o en hexadecimal. |
| `base` | texto | `dec`, `hex` · por defecto `dec` | `dec` o `hex`: cómo se escriben las direcciones. |
| `formula` | sí/no |  | Escribe las fórmulas debajo (por omisión, sí). |
| `blank` | sí/no |  | Deja en blanco la traducción. |

**Modo ejercicio:** Campos: `blank`.

Ejemplo: Traducción de direcciones con paginación

```yaml
type: page_table
title: Traducción de direcciones con paginación
pageSize: 1024
table: [5, 2, null, 7, 0]
addresses: [1500, 3100, 4095, 2100, 6000]
```

<a id="page_replacement"></a>

## `page_replacement`: Reemplazo de páginas

*Sistemas de computación › Sistemas operativos.* FIFO, LRU, óptimo y reloj sobre una cadena de referencias, para compararlos.

Simula los algoritmos de reemplazo sobre la cadena de referencias y dibuja, para cada uno,
una grilla con el contenido de cada marco en cada paso, la página que entra resaltada y los
fallos marcados. Con varios algoritmos, uno debajo del otro, con sus tasas de fallos.

**Calcula:** FIFO, LRU, óptimo (OPT, el que se usa más lejos en el futuro) y reloj (segunda oportunidad, con el bit de referencia marcado *).

| Campo | Tipo | Valores · por defecto | Descripción |
| :--- | :--- | :--- | :--- |
| `frames` | número |  | Cantidad de marcos (de 1 a 16, por omisión 3). |
| `references` | lista |  | La cadena de referencias (hasta 60). |
| `algorithm` | texto |  | Un algoritmo, `fifo`, `lru`, `opt` o `clock`. |
| `algorithms` | lista |  | Varios algoritmos para comparar. |
| `blank` | sí/no |  | Deja la grilla y los fallos en blanco. |

**Modo ejercicio:** Campos: `blank`.

Ejemplo: Algoritmos de reemplazo de páginas

```yaml
type: page_replacement
title: Algoritmos de reemplazo de páginas
frames: 3
references: [7, 0, 1, 2, 0, 3, 0, 4, 2, 3, 0, 3, 2, 1, 2, 0, 1, 7, 0, 1]
algorithms: [fifo, lru, opt]
```

Ejemplo: Algoritmo del reloj

```yaml
type: page_replacement
title: Algoritmo del reloj
frames: 3
references: [1, 2, 3, 2, 4, 1, 5, 2, 1, 2, 3, 5]
algorithm: clock
```

<a id="subnetting"></a>

## `subnetting`: Subnetting

*Sistemas de computación › Redes.* Subredes IPv4 de igual tamaño o con VLSM, con su mapa de direcciones.

Divide una red IPv4 en subredes, de igual tamaño o con VLSM según los hosts que necesita cada
una, y dibuja la tabla con la red, el prefijo, la máscara, el primer y el último host y el
broadcast de cada subred, más un mapa del espacio de direcciones.

**Calcula:** Con VLSM, el prefijo mínimo para cada pedido y la asignación alineada, de la subred más grande a la más chica. Los hosts útiles de cada subred (las /31 y /32, 2 y 1).

| Campo | Tipo | Valores · por defecto | Descripción |
| :--- | :--- | :--- | :--- |
| `network` | texto |  | La red, `A.B.C.D/n` (prefijo de 0 a 30). |
| `subnets` | lista |  | Las subredes con VLSM: `{name, hosts}` o sólo la cantidad de hosts (hasta 64). |
| `count` | número |  | Cantidad de subredes iguales, en lugar de `subnets`. |
| `sort` | sí/no |  | Con VLSM, asigna de la más grande a la más chica (por omisión, sí). |
| `summary` | sí/no |  | Escribe cuántas direcciones se usan (por omisión, sí). |
| `map` | sí/no |  | Dibuja el mapa del espacio de direcciones (por omisión, sí). |
| `binary` | sí/no |  | Escribe cada red en binario, con los bits de red y de host. |
| `blank` | sí/no |  | Deja en blanco todo salvo el nombre y los hosts pedidos. |

**Modo ejercicio:** Campos: `blank`.

Ejemplo: Subnetting VLSM

```yaml
type: subnetting
title: Subnetting VLSM
network: 192.168.10.0/24
subnets:
  - { name: Ventas, hosts: 50 }
  - { name: Desarrollo, hosts: 100 }
  - { name: Admin, hosts: 20 }
  - { name: Enlace WAN, hosts: 2 }
```

Ejemplo: Ejercicio: 8 subredes iguales

```yaml
type: subnetting
title: "Ejercicio: 8 subredes iguales"
network: 10.0.0.0/16
count: 8
blank: true
```

<a id="transaction_schedule"></a>

## `transaction_schedule`: Planificación de transacciones

*Sistemas de computación › Bases de datos.* Conflictos, grafo de precedencia y serializabilidad por conflictos.

Dibuja la planificación con una columna por transacción y una fila por operación, resaltando
las que están en conflicto. Al lado, el grafo de precedencia; debajo, la lista de conflictos y
si la planificación es serializable por conflictos, con un orden serial equivalente o el
ciclo que lo impide.

**Calcula:** Los pares en conflicto (distintas transacciones, el mismo ítem y al menos una escritura). Un orden topológico del grafo de precedencia o un ciclo.

| Campo | Tipo | Valores · por defecto | Descripción |
| :--- | :--- | :--- | :--- |
| `schedule` | lista |  | Las operaciones en orden: `R1(A)`, `W2(B)`, `C1` (commit), `A3` (abort); también `"T1: R(A)"`. Una lista o un texto separado por comas. |
| `highlight` | sí/no |  | Resalta las operaciones en conflicto (por omisión, sí). |
| `conflicts` | sí/no |  | Lista los conflictos debajo (por omisión, sí). |
| `blank` | sí/no |  | Oculta las aristas del grafo y el veredicto. |

**Modo ejercicio:** Campos: `blank`.

Ejemplo: Planificación no serializable

```yaml
type: transaction_schedule
title: Planificación no serializable
schedule: [R1(A), R2(A), W1(A), R2(B), W2(A), W1(B), C1, C2]
```

Ejemplo: Planificación serializable por conflictos

```yaml
type: transaction_schedule
title: Planificación serializable por conflictos
schedule: [R1(A), W1(A), R2(A), R1(B), W2(A), W1(B), R3(B), W3(B), C1, C2, C3]
```

<a id="block_diagram"></a>

## `block_diagram`: Diagrama de bloques

*Control de procesos › Modelado.* Lazos de control con realimentación, perturbaciones y lazos en cascada.

Dibuja un diagrama de bloques: los bloques del camino directo en serie, el sumador con su
signo, la realimentación y las señales con sus nombres. Una perturbación entra en un sumador
propio antes del bloque que se indique, y un bloque puede ser a su vez un lazo interno, para
los esquemas en cascada.

| Campo | Tipo | Valores · por defecto | Descripción |
| :--- | :--- | :--- | :--- |
| `forward` | lista |  | Los bloques del camino directo: un texto, `{label, out, blank}` (`out` es el nombre de la señal de salida) o `{loop: {forward, feedback}}` para un lazo interno. |
| `feedback` | lista |  | Los bloques de la realimentación; `[]` es realimentación unitaria. Sin la clave, el lazo queda abierto. |
| `sign` | texto |  | El signo de la realimentación en el sumador, `-` (por omisión) o `+`. |
| `input` | texto | por defecto `R(s)` | El nombre de la entrada, como `R(s)`. |
| `output` | texto | por defecto `Y(s)` | El nombre de la salida, como `Y(s)`. |
| `error` | texto |  | El nombre de la señal de error. |
| `measured` | texto |  | El nombre de la señal medida (la salida de la realimentación). |
| `disturbance` | objeto |  | Una perturbación: `at` (antes de qué bloque entra, desde 0), `label`, `block` (un bloque propio) y `sign`. |
| `blankSigns` | sí/no |  | Deja en blanco los signos de los sumadores. |

**Modo ejercicio:** Campos: `blankSigns`.

Ejemplo: Lazo con perturbación <D(s) & H(s)>

```yaml
type: block_diagram
title: "Lazo con perturbación <D(s) & H(s)>"
input: "R(s)"
output: "Y(s)"
error: "E(s)"
forward:
  - {label: "Gc(s)", out: "U(s)"}
  - {label: "Gv(s)", out: "M(s)"}
  - "Gp(s)"
disturbance: {at: 2, label: "D(s)", block: "Gd(s)"}
feedback: ["H(s)"]
measured: "B(s)"
inkSaver: true
```

Ejemplo: Control en cascada (evaluación)

```yaml
type: block_diagram
title: Control en cascada (evaluación)
forward:
  - "Gc1(s)"
  - loop:
      forward: ["Gc2(s)", "Gp2(s)"]
      feedback: ["H2(s)"]
  - "Gp1(s)"
feedback: ["H1(s)"]
blankSigns: true
inkSaver: false
```

- Los bloques son rótulos; para calcular la respuesta del lazo se usa `closedLoop` en [`time_response`](#time_response).

<a id="time_response"></a>

## `time_response`: Respuesta temporal

*Control de procesos › Respuesta temporal.* La salida de un sistema ante escalón, rampa o impulso, con sobrepico, tiempos y constantes.

Simula la respuesta de uno o varios sistemas (o de un lazo cerrado) a la entrada elegida y la
dibuja con las anotaciones pedidas: valor final, sobrepico, tiempo de pico, de subida y de
establecimiento, la constante de tiempo, el tiempo muerto y la tangente de la curva de
reacción. También dibuja puntos medidos, para comparar.

**Calcula:** La simulación con tiempo muerto; un PID se simula discreto, con el derivativo filtrado. Sobrepico, tiempos de pico, subida y establecimiento (banda del 2 % o del 5 %), τ y θ.

| Campo | Tipo | Valores · por defecto | Descripción |
| :--- | :--- | :--- | :--- |
| `system` | objeto |  | El sistema (ver la nota sobre sistemas). |
| `systems` | lista |  | Varios sistemas para comparar, cada uno con `label`. |
| `closedLoop` | objeto |  | Un lazo cerrado: `plant`, `controller` (por omisión 1), `feedback` (por omisión 1) y `sign` (por omisión `-`). |
| `input` | texto | por defecto `step` | `step`, `ramp` o `impulse`. |
| `amplitude` | número |  | La amplitud de la entrada (por omisión 1). |
| `tEnd` | número |  | El tiempo final; por omisión, el que hace falta para que se establezca. |
| `disturbance` | objeto |  | Una perturbación en escalón: `at` (el instante), `amplitude` y `where` (`output` o `input`). |
| `annotations` | lista |  | `final`, `overshoot`, `peak`, `rise`, `settling`, `tau`, `delay`, `tangent`, o `all`. |
| `settlingBand` | número |  | La banda del tiempo de establecimiento, `0.02` o `0.05`. |
| `showInput` | sí/no |  | Dibuja también la entrada. |
| `data` | lista |  | Puntos medidos `[t, y]`, que se dibujan como marcas. |
| `dataLabel` | texto | por defecto `medición` | El rótulo de los puntos medidos. |
| `yMin` | número |  | Mínimo del eje vertical. |
| `yMax` | número |  | Máximo del eje vertical. |
| `xLabel` | texto | por defecto `t (s)` | Rótulo del eje horizontal. |
| `yLabel` | texto | por defecto `y(t)` | Rótulo del eje vertical. |
| `blankAnnotations` | sí/no |  | Deja en blanco los valores de las anotaciones. |

**Modo ejercicio:** Campos: `blankAnnotations`.

Ejemplo: Segundo orden subamortiguado (ζ = 0,3)

```yaml
type: time_response
title: Segundo orden subamortiguado (ζ = 0,3)
system: {gain: 1, zeta: 0.3, wn: 2}
annotations: [final, overshoot, peak, rise, settling]
```

Ejemplo: Curva FOPDT con retardo (para completar)

```yaml
type: time_response
title: Curva FOPDT con retardo (para completar)
system: {gain: 2, num: [2], den: [15, 8, 1], delay: 1}
annotations: [final, tangent]
blankAnnotations: true
inkSaver: false
```

Ejemplo: Lazo cerrado con PI y perturbación

```yaml
type: time_response
title: Lazo cerrado con PI y perturbación
closedLoop:
  plant: {gain: 2, tau: 5, delay: 1}
  controller: {kp: 1.2, ki: 0.25}
disturbance: {at: 30, amplitude: -0.5}
tEnd: 60
annotations: [overshoot, settling]
```

- Un sistema se escribe como `num` y `den` (coeficientes en s, de la mayor potencia a la menor), `gain` y `tau` (primer orden K/(τs + 1)), `gain`, `zeta` y `wn` (segundo orden), o `k`, `poles` y `zeros` (`[re, im]` es un par complejo); todos con `delay` (tiempo muerto) opcional. Un controlador PID, como `kp`, `ki` y `kd`.

<a id="bode"></a>

## `bode`: Diagrama de Bode

*Control de procesos › Respuesta en frecuencia.* Magnitud y fase en frecuencia, con asíntotas y márgenes de estabilidad.

Evalúa la respuesta en frecuencia y dibuja la magnitud en dB y la fase en grados, en escala
logarítmica. Opcionalmente, las asíntotas y los márgenes de ganancia y de fase con sus
frecuencias de cruce. Con `closedLoop` dibuja el lazo abierto L(s) = C·G·H.

**Calcula:** El margen de ganancia (en ω180) y el de fase (en ωc). Las asíntotas a partir de los polos y ceros.

| Campo | Tipo | Valores · por defecto | Descripción |
| :--- | :--- | :--- | :--- |
| `system` | objeto |  | El sistema (ver la nota de [`time_response`](#time_response)). |
| `systems` | lista |  | Varios sistemas para comparar, cada uno con `label`. |
| `closedLoop` | objeto |  | Un lazo; se dibuja su lazo abierto. |
| `wMin` | número |  | La frecuencia mínima, en rad/s. |
| `wMax` | número |  | La frecuencia máxima, en rad/s. |
| `show` | texto | `both`, `magnitude`, `phase` · por defecto `both` | `both`, `magnitude` o `phase`. |
| `asymptotes` | sí/no |  | Dibuja las asíntotas de la magnitud. |
| `margins` | sí/no |  | Marca los márgenes de ganancia y de fase. |
| `blankMargins` | sí/no |  | Deja los márgenes en blanco. |

**Modo ejercicio:** Campos: `blankMargins`.

Ejemplo: Bode de L(s) = 10 / (s(s+1)(s+10))

```yaml
type: bode
title: Bode de L(s) = 10 / (s(s+1)(s+10))
system: {k: 10, poles: [0, -1, -10]}
asymptotes: true
margins: true
```

<a id="nyquist"></a>

## `nyquist`: Diagrama de Nyquist

*Control de procesos › Respuesta en frecuencia.* La curva de Nyquist del lazo abierto, con el círculo unitario y las ω negativas.

Dibuja la curva de Nyquist del lazo abierto en el plano complejo, con el punto −1 marcado.
Opcionalmente, el círculo unitario y la rama de las frecuencias negativas, punteada.

| Campo | Tipo | Valores · por defecto | Descripción |
| :--- | :--- | :--- | :--- |
| `system` | objeto |  | El sistema (ver la nota de [`time_response`](#time_response)). |
| `systems` | lista |  | Varios sistemas para comparar. |
| `closedLoop` | objeto |  | Un lazo; se dibuja su lazo abierto. |
| `wMin` | número |  | La frecuencia mínima. |
| `wMax` | número |  | La frecuencia máxima. |
| `limit` | número |  | El radio máximo que se dibuja (recorta las ramas que se van al infinito). |
| `unitCircle` | sí/no |  | Dibuja el círculo unitario. |
| `showNegative` | sí/no |  | Dibuja la rama de ω negativas, punteada. |

Ejemplo: Nyquist con círculo unitario

```yaml
type: nyquist
title: Nyquist con círculo unitario
system: {k: 5, poles: [-1, -2, -3]}
unitCircle: true
inkSaver: false
```

<a id="pole_zero"></a>

## `pole_zero`: Polos y ceros

*Control de procesos › Plano s.* Los polos y ceros de un sistema en el plano s.

Ubica los polos (×) y los ceros (○) en el plano s, a partir de las listas o de un sistema (o
los del lazo cerrado). Opcionalmente, las rectas de amortiguamiento constante y los círculos
de frecuencia natural constante.

| Campo | Tipo | Valores · por defecto | Descripción |
| :--- | :--- | :--- | :--- |
| `poles` | lista |  | Los polos; `[re, im]` es un par complejo. |
| `zeros` | lista |  | Los ceros. |
| `system` | objeto |  | Un sistema, del que salen polos y ceros. |
| `closedLoop` | objeto |  | Un lazo; se dibujan los polos y ceros del lazo cerrado. |
| `showValues` | sí/no |  | Escribe el valor de cada polo y cero. |
| `dampingLines` | lista |  | Rectas de amortiguamiento constante: una lista de valores de ζ (de 0 a 1). |
| `wnCircles` | lista |  | Círculos de frecuencia natural constante: una lista de valores de ωn. |
| `xRange` | lista |  | El rango del eje real. |
| `yRange` | lista |  | El rango del eje imaginario. |
| `blank` | sí/no |  | Dibuja sólo los ejes, para ubicarlos. |

**Modo ejercicio:** Campos: `blank`.

Ejemplo: Polos y ceros con ζ y ωn

```yaml
type: pole_zero
title: Polos y ceros con ζ y ωn
poles: [-1, [-2, 3]]
zeros: [-4]
dampingLines: [0.5]
wnCircles: [3]
showValues: true
inkSaver: false
```

<a id="root_locus"></a>

## `root_locus`: Lugar de las raíces

*Control de procesos › Plano s.* El lugar de las raíces con asíntotas, cruce con el eje imaginario y ganancias marcadas.

Dibuja cómo se mueven los polos del lazo cerrado al variar la ganancia desde 0 hasta `kMax`,
a partir del lazo abierto. Opcionalmente, las asíntotas, el cruce con el eje imaginario
(ganancia y frecuencia críticas) y la posición de los polos para ganancias elegidas.

**Calcula:** Las raíces para cada ganancia, las asíntotas y su centroide. La ganancia crítica y la frecuencia del cruce.

| Campo | Tipo | Valores · por defecto | Descripción |
| :--- | :--- | :--- | :--- |
| `system` | objeto |  | El lazo abierto (ver la nota de [`time_response`](#time_response)). |
| `closedLoop` | objeto |  | Un lazo; se usa su lazo abierto. |
| `kMax` | número |  | La ganancia máxima. |
| `asymptotes` | sí/no |  | Dibuja las asíntotas. |
| `showCrossing` | sí/no |  | Marca el cruce con el eje imaginario, con Kcrít y ωcrít. |
| `marks` | lista |  | Ganancias cuyos polos se marcan con ■. |
| `dampingLines` | lista |  | Rectas de amortiguamiento constante: una lista de valores de ζ (de 0 a 1). |
| `xRange` | lista |  | El rango del eje real. |
| `yRange` | lista |  | El rango del eje imaginario. |
| `blankMarks` | sí/no |  | Deja en blanco los valores de las marcas. |

**Modo ejercicio:** Campos: `blankMarks`.

Ejemplo: Lugar de las raíces K / (s(s+1)(s+2))

```yaml
type: root_locus
title: Lugar de las raíces K / (s(s+1)(s+2))
system: {k: 1, poles: [0, -1, -2]}
marks: [1]
```

<a id="instrumentation"></a>

## `instrumentation`: P&ID

*Control de procesos › Instrumentación y automatización.* Diagramas de instrumentación con símbolos ISA-5.1.

Dibuja un diagrama de cañerías e instrumentos: equipos (tanques, bombas, válvulas,
intercambiadores), instrumentos con su tag y su ubicación (campo, panel, DCS, PLC) y las
líneas de proceso, eléctricas, neumáticas o de datos que los unen, sobre una grilla.

| Campo | Tipo | Valores · por defecto | Descripción |
| :--- | :--- | :--- | :--- |
| `equipment` | lista |  | Los equipos: `id`, `kind` (`tank`, `vessel`, `reactor`, `column`, `pump`, `control_valve`, `valve`, `heat_exchanger`), `at: [x, y]` en la grilla, `label`, `size`, `level`, `orientation: v`. |
| `instruments` | lista |  | Los instrumentos: `id`, `tag` (`"LIC 101"`), `at`, `location` (`field`, `panel`, `panel_back`, `dcs`, `plc`) y `blank`. |
| `lines` | lista |  | Las líneas: `from` y `to` (un id o `[x, y]`), `kind` (`process`, `electric`, `pneumatic`, `data`, `capillary`), `via`, `route` (`hv` o `vh`), `label`, `arrow`. |
| `grid` | número |  | Píxeles por unidad de la grilla. |
| `legend` | sí/no |  | Agrega la leyenda de los tipos de línea. |

Ejemplo: Control de nivel TK-101 (ISA-5.1)

```yaml
type: instrumentation
title: Control de nivel TK-101 (ISA-5.1)
equipment:
  - {id: TK, kind: tank, at: [3, 3], label: "TK-101", level: 0.6}
  - {id: P1, kind: pump, at: [6, 5.2], label: "P-101"}
  - {id: LV, kind: control_valve, at: [8.5, 4.8], label: "LV-101"}
instruments:
  - {id: LT, tag: "LT 101", at: [5, 2]}
  - {id: LIC, tag: "LIC 101", at: [7, 1], location: dcs}
lines:
  - {from: [0.5, 1.4], to: TK, label: "entrada", via: [[3, 1.4]]}
  - {from: TK, to: P1, via: [[3, 5.2]]}
  - {from: P1, to: LV, via: [[7, 4.8]]}
  - {from: LV, to: [11, 4.8], label: "salida"}
  - {from: TK, to: LT, kind: capillary, via: [[4, 2]]}
  - {from: LT, to: LIC, kind: electric, via: [[5, 1]]}
  - {from: LIC, to: LV, kind: electric, via: [[8.5, 1]]}
inkSaver: false
```

Ejemplo: Instrumento en blanco

```yaml
type: instrumentation
title: Instrumento en blanco
equipment:
  - {id: TK, kind: tank, at: [2, 2], label: "TK-1"}
instruments:
  - {id: LT, tag: "LT 1", at: [5, 1], blank: true}
lines:
  - {from: TK, to: LT, kind: capillary}
```

<a id="ladder"></a>

## `ladder`: Diagrama escalera

*Control de procesos › Instrumentación y automatización.* Programas de PLC en lenguaje escalera, con contactos, bobinas, temporizadores y contadores.

Dibuja los escalones de un programa en lenguaje escalera entre los dos rieles de
alimentación. Cada escalón es una secuencia de contactos (en serie, o en paralelo con `or`)
que termina en sus salidas.

| Campo | Tipo | Valores · por defecto | Descripción |
| :--- | :--- | :--- | :--- |
| `rungs` | lista |  | Los escalones, cada uno con `logic` y `comment`. En `logic`: contactos `no` (o `na`), `nc`, `p`, `n`; salidas `coil`, `ncoil`, `set`, `reset`; temporizadores `ton`, `tof`, `tp` y contadores `ctu`, `ctd` con `preset`; `or` (ramas en paralelo, cada una una lista). Cada elemento con `label` y `blank`. |

Ejemplo: Arranque y parada con enclavamiento

```yaml
type: ladder
title: Arranque y parada con enclavamiento
rungs:
  - comment: "Motor con enclavamiento"
    logic:
      - or:
          - [{"no": "I0.0", label: "Marcha"}]
          - [{"no": "Q0.0"}]
      - {nc: "I0.1", label: "Parada"}
      - {nc: "I0.2", label: "Térmico"}
      - {coil: "Q0.0", label: "Motor"}
  - comment: "Temporizador de llenado"
    logic:
      - {"no": "Q0.0"}
      - {ton: "T1", preset: "10 s"}
  - logic:
      - {"no": "T1"}
      - {set: "Q0.1", label: "Alarma"}
inkSaver: true
```

- YAML lee `no:` como `false`; dibu lo entiende igual, y `na:` también vale.

<a id="grafcet"></a>

## `grafcet`: GRAFCET

*Control de procesos › Instrumentación y automatización.* Etapas, transiciones y divergencias en O y en Y.

Dibuja un GRAFCET: etapas con sus acciones, transiciones con su receptividad y divergencias
y convergencias en O y en Y. Por omisión, la última transición vuelve a la etapa inicial.

| Campo | Tipo | Valores · por defecto | Descripción |
| :--- | :--- | :--- | :--- |
| `steps` | lista |  | En orden: `step` (número, `initial`, `actions`, `blank`), `transition` (la receptividad, `blank`), `or` y `and` (divergencias: listas de secuencias). |
| `loop` | sí/no |  | Vuelve a la etapa inicial al final (por omisión, sí). |

Ejemplo: Llenado y mezcla en paralelo

```yaml
type: grafcet
title: Llenado y mezcla en paralelo
steps:
  - {step: 0, initial: true}
  - {transition: "Marcha · nivel bajo"}
  - {step: 1, actions: ["Abrir V1"]}
  - {transition: "nivel alto"}
  - and:
      - [{step: 2, actions: ["Agitador"]}, {transition: "t/X2/30 s"}, {step: 4}]
      - [{step: 3, actions: ["Calentar"]}, {transition: "T > 60 °C"}, {step: 5}]
  - {transition: "1"}
  - {step: 6, actions: ["Abrir V2"]}
  - {transition: "nivel bajo"}
```

Ejemplo: Divergencia OR con campos en blanco

```yaml
type: grafcet
title: Divergencia OR con campos en blanco
steps:
  - {step: 0, initial: true}
  - {transition: "Auto & <Listo>", blank: true}
  - or:
      - [{step: 1, actions: ["Ciclo A"], blank: true}]
      - [{step: 2, actions: ["Ciclo B"]}]
  - {transition: "Fin"}
inkSaver: false
```

<a id="molecule"></a>

## `molecule`: Molécula

*Química › Estructura y enlace.* Fórmulas de Lewis y esqueletales, con cargas, pares libres, estereoquímica y flechas de mecanismo.

Dibuja una molécula a partir de sus átomos y enlaces: en fórmula de Lewis (con pares libres,
cargas y radicales) o esqueletal (sin carbonos ni hidrógenos explícitos). Los átomos se ubican
por coordenadas o encadenados por ángulo desde otro; los anillos se arman solos y dos que
comparten vértices se fusionan. Las flechas curvas muestran un mecanismo.

| Campo | Tipo | Valores · por defecto | Descripción |
| :--- | :--- | :--- | :--- |
| `atoms` | lista |  | Los átomos: `id`, `el` (el símbolo, también grupos como `OH`), `at: [x, y]` en largos de enlace, o `from` y `angle` (con `length` y `bond`, el orden del enlace con ese átomo); `charge`, `lonePairs`, `radical`, `note` (como δ+), `show`, `blank`. |
| `rings` | lista |  | Anillos: `size`, `center`, `start`, `ids` (los ids de sus vértices), `aromatic` (con círculo) o `kekule` (dobles alternados). |
| `bonds` | lista |  | Enlaces extra: `[a, b, orden]` o `{from, to, order, stereo, dashed, blank}`. El orden es 1, 2, 3, 1.5 o 0.5; `stereo` es `wedge` (cuña) o `hash` (rayas). |
| `arrows` | lista |  | Flechas curvas de mecanismo: `from` y `to` (un átomo o un enlace `[a, b]`), `bend` (la curvatura) y `half` (un solo electrón). |
| `skeletal` | sí/no |  | Fórmula esqueletal. |
| `bondLength` | número |  | El largo de un enlace, en píxeles. |
| `caption` | texto |  | Un rótulo debajo de la molécula. |

Ejemplo: Ion nitrato (Lewis resonante)

```yaml
type: molecule
title: Ion nitrato (Lewis resonante)
atoms:
  - {id: n, el: N, at: [0, 0], charge: 1}
  - {id: o1, el: O, from: n, angle: 90, bond: 2, lonePairs: 2}
  - {id: o2, el: O, from: n, angle: 210, lonePairs: 3, charge: -1}
  - {id: o3, el: O, from: n, angle: 330, lonePairs: 3, charge: -1}
```

Ejemplo: Naftaleno (anillos fusionados)

```yaml
type: molecule
title: Naftaleno (anillos fusionados)
skeletal: true
rings:
  - {size: 6, center: [0, 0], start: 30, kekule: true}
  - {size: 6, center: [1.732, 0], start: 30}
bonds:
  - [r1_1, r1_2, 2]
  - [r1_5, r1_0, 2]
inkSaver: false
```

Ejemplo: Mecanismo S_N\2 y carbono quiral

```yaml
type: molecule
title: "Mecanismo S_N\\2 y carbono quiral"
atoms:
  - {id: oh, el: HO, at: [-2.2, 0], charge: -1, lonePairs: 2}
  - {id: c, el: C, at: [0, 0]}
  - {id: br, el: Br, from: c, angle: 0, lonePairs: 3}
  - {id: h1, el: H, from: c, angle: 90, bond: {stereo: hash}}
  - {id: cl, el: Cl, from: c, angle: 225, bond: {stereo: wedge}}
  - {id: h3, el: H, from: c, angle: 135, length: 0.8}
arrows:
  - {from: oh, to: c, bend: 1}
  - {from: [c, br], to: br, bend: -1}
```

<a id="vsepr"></a>

## `vsepr`: Geometría molecular (TRePEV)

*Química › Estructura y enlace.* La geometría de una molécula por TRePEV, con ángulo e hibridación.

Dibuja la geometría molecular según la teoría de repulsión de pares electrónicos de valencia,
con el átomo central, los ligandos y los pares libres. La geometría sale de la cantidad de
pares, o se fija con `geometry`. Debajo, la notación AXE, el nombre de la geometría, el ángulo
y la hibridación.

**Calcula:** La geometría, la notación AXE, el ángulo de enlace y la hibridación a partir de los pares.

| Campo | Tipo | Valores · por defecto | Descripción |
| :--- | :--- | :--- | :--- |
| `central` | texto |  | El átomo central. |
| `ligands` | lista |  | Los ligandos. |
| `lonePairs` | número |  | Los pares libres sobre el átomo central. |
| `geometry` | texto |  | La geometría, si no sale de los pares: `linear`, `trigonal_planar`, `bent_trigonal`, `tetrahedral`, `trigonal_pyramidal`, `bent`, `trigonal_bipyramidal`, `seesaw`, `t_shaped`, `linear_tbp`, `octahedral`, `square_pyramidal`, `square_planar`. |
| `showAngle` | sí/no |  | Escribe el ángulo. |
| `showHybridization` | sí/no |  | Escribe la hibridación. |
| `blank` | lista |  | `true` o una lista de `axe`, `name`, `angle` y `hybridization` para dejar en blanco. |

**Modo ejercicio:** Campos: `blank`.

Ejemplo: Geometría de SF4 (balancín)

```yaml
type: vsepr
title: Geometría de SF4 (balancín)
central: S
ligands: [F, F, F, F]
lonePairs: 1
```

Ejemplo: XeF4 (para completar)

```yaml
type: vsepr
title: XeF4 (para completar)
central: Xe
ligands: [F, F, F, F]
lonePairs: 2
blank: [name, angle, hybridization]
inkSaver: false
```

<a id="orbitals"></a>

## `orbitals`: Orbitales

*Química › Estructura y enlace.* Diagramas de casilleros atómicos y de orbitales moleculares.

En la variante atómica dibuja los casilleros de la configuración electrónica de un elemento o
ion, con la configuración abreviada y los electrones desapareados. En la molecular, el diagrama
de orbitales moleculares de una molécula diatómica del segundo período, con el orden de enlace
y el magnetismo.

**Calcula:** La configuración electrónica, con las excepciones del Cr y el Cu y los cationes de transición. El llenado de los orbitales moleculares, con o sin mezcla s-p, el orden de enlace y el magnetismo.

| Campo | Tipo | Valores · por defecto | Descripción |
| :--- | :--- | :--- | :--- |
| `kind` | texto |  | `atomic` (por omisión) o `molecular`. |
| `element` | texto |  | En `atomic`, el elemento. |
| `charge` | número |  | La carga del ion. |
| `core` | sí/no |  | En `atomic`, abrevia con el gas noble (`[Ar]`) y dibuja sólo la capa de valencia. |
| `showConfig` | sí/no |  | Escribe la configuración. |
| `showUnpaired` | sí/no |  | Escribe los electrones desapareados. |
| `atoms` | lista |  | En `molecular`, los dos átomos (`[O, O]`, `[C, O]`; de H a Ne). |
| `maxWidth` | número |  | El ancho máximo, en píxeles. |
| `blank` | sí/no |  | Deja los casilleros vacíos. |
| `blankConfig` | sí/no |  | Deja la configuración en blanco. |
| `blankUnpaired` | sí/no |  | Deja en blanco los desapareados. |
| `blankElectrons` | sí/no |  | En `molecular`, deja los orbitales vacíos. |
| `blankBondOrder` | sí/no |  | En `molecular`, deja el orden de enlace en blanco. |
| `blankMagnetism` | sí/no |  | En `molecular`, deja el magnetismo en blanco. |

**Modo ejercicio:** Campos: `blank`, `blankConfig`, `blankUnpaired`, `blankElectrons`, `blankBondOrder`, `blankMagnetism`.

Ejemplo: Configuración electrónica de Fe

```yaml
type: orbitals
title: Configuración electrónica de Fe
element: Fe
core: true
showUnpaired: true
```

Ejemplo: Ion Cu²⁺ (para completar)

```yaml
type: orbitals
title: Ion Cu²⁺ (para completar)
element: Cu
charge: 2
core: true
blank: true
```

Ejemplo: Diagrama OM de O2

```yaml
type: orbitals
kind: molecular
title: Diagrama OM de O2
atoms: [O, O]
inkSaver: false
```

Ejemplo: Diagrama OM de N2

```yaml
type: orbitals
kind: molecular
title: Diagrama OM de N2
atoms: [N, N]
blankBondOrder: true
```

<a id="unit_cell"></a>

## `unit_cell`: Celda unidad

*Química › Estructura y enlace.* Celdas cúbicas y redes iónicas, con átomos por celda, coordinación, empaquetamiento y densidad.

Dibuja una celda unidad cúbica en perspectiva y calcula sus propiedades: la cantidad de
átomos por celda, el número de coordinación, la fracción de empaquetamiento y, con el lado y
la masa molar, la densidad.

**Calcula:** Z, la coordinación y la fracción de empaquetamiento de cada red. La densidad ρ = Z·M / (N_A·a³).

| Campo | Tipo | Valores · por defecto | Descripción |
| :--- | :--- | :--- | :--- |
| `lattice` | texto |  | `sc`, `bcc`, `fcc`, `nacl` o `cscl`. |
| `element` | texto |  | El elemento (en las redes de un solo tipo de átomo). |
| `species` | lista |  | Las especies de una red iónica. |
| `a` | número |  | El lado de la celda, en pm. |
| `molarMass` | número |  | La masa molar, en g/mol (para la densidad). |
| `blank` | sí/no |  | `true` o una lista de `z`, `coordination`, `packing` y `density`. |

**Modo ejercicio:** Campos: `blank`.

Ejemplo: Celda unitaria Cu (fcc)

```yaml
type: unit_cell
title: Celda unitaria Cu (fcc)
lattice: fcc
element: Cu
a: 361.5
molarMass: 63.55
```

Ejemplo: Celda bcc Fe (para completar)

```yaml
type: unit_cell
title: Celda bcc Fe (para completar)
lattice: bcc
element: Fe
blank: true
inkSaver: false
```

<a id="reaction"></a>

## `reaction`: Reacción

*Química › Reacciones y cinética.* Ecuaciones químicas, equilibrios, resonancia, mecanismos y productos para completar.

Dibuja una ecuación química o un mecanismo: fórmulas, signos más, moléculas dibujadas (como
las de [`molecule`](#molecule)) y flechas de reacción, equilibrio, resonancia o
retrosíntesis con texto arriba y abajo. Un casillero en blanco deja un producto para completar.

| Campo | Tipo | Valores · por defecto | Descripción |
| :--- | :--- | :--- | :--- |
| `steps` | lista |  | Los elementos en orden: fórmulas, `"+"`, moléculas (bloques de `molecule`), `{arrow: "->", "<=>", "<->" (resonancia) o "=>" (retrosíntesis), above, below, blank}` y `{blank: true, width}` (un producto para completar). |
| `maxWidth` | número |  | El ancho máximo; más largo, la ecuación sigue en otro renglón. |

Ejemplo: Esterificación de Fischer

```yaml
type: reaction
title: Esterificación de Fischer
steps:
  - "CH3COOH"
  - "+"
  - "CH3CH2OH"
  - {arrow: "<=>", above: "H2SO4", below: "Δ"}
  - "CH3COOCH2CH3"
  - "+"
  - "H2O"
```

Ejemplo: Resonancia carboxilato y completar

```yaml
type: reaction
title: Resonancia carboxilato y completar
steps:
  - atoms:
      - {id: c, el: C, at: [0, 0]}
      - {id: r, el: R, from: c, angle: 180}
      - {id: o1, el: O, from: c, angle: 60, bond: 2, lonePairs: 2}
      - {id: o2, el: O, from: c, angle: 300, lonePairs: 3, charge: -1}
  - {arrow: "<->"}
  - {blank: true, width: 60}
inkSaver: false
```

<a id="energy_profile"></a>

## `energy_profile`: Perfil de energía

*Química › Reacciones y cinética.* El perfil de energía de una reacción, con energías de activación y ΔH, catalizada o no.

Dibuja el camino de una reacción: reactivos, intermediarios, estados de transición y
productos, unidos por curvas. Opcionalmente, un camino catalizado punteado, las energías de
activación de cada etapa y el ΔH, y la etapa determinante de la velocidad.

**Calcula:** La Ea de cada etapa, el ΔH y la etapa determinante (la de mayor Ea).

| Campo | Tipo | Valores · por defecto | Descripción |
| :--- | :--- | :--- | :--- |
| `states` | lista |  | Los estados en orden: `label`, `energy` y `ts: true` para un estado de transición. |
| `catalyzed` | lista |  | Otro camino, punteado (la misma forma que `states`). |
| `annotations` | lista |  | `Ea` (de cada etapa) y `dH`. |
| `unit` | texto |  | La unidad de energía (por omisión kJ/mol). |
| `showEnergies` | sí/no |  | Escribe la energía de cada estado. |
| `showRateDetermining` | sí/no |  | Marca la etapa determinante. |
| `xLabel` | texto |  | Rótulo del eje horizontal. |
| `yLabel` | texto |  | Rótulo del eje vertical. |
| `blankAnnotations` | sí/no |  | Deja en blanco las anotaciones. |

**Modo ejercicio:** Campos: `blankAnnotations`.

Ejemplo: Reacción exotérmica catalizada

```yaml
type: energy_profile
title: Reacción exotérmica catalizada
states:
  - {label: "A + B", energy: 0}
  - {label: "ET‡", energy: 85, ts: true}
  - {label: "C + D", energy: -40}
catalyzed:
  - {energy: 0}
  - {energy: 45, ts: true}
  - {energy: -40}
```

Ejemplo: Dos etapas con etapa determinante

```yaml
type: energy_profile
title: Dos etapas con etapa determinante
states:
  - {label: Reactivos, energy: 0}
  - {label: "ET1", energy: 60, ts: true}
  - {label: Intermediario, energy: 25}
  - {label: "ET2", energy: 90, ts: true}
  - {label: Productos, energy: 15}
showRateDetermining: true
blankAnnotations: true
inkSaver: false
```

<a id="kinetics"></a>

## `kinetics`: Cinética

*Química › Reacciones y cinética.* Concentración en el tiempo, linealizaciones, Arrhenius y reacciones consecutivas o reversibles.

Dibuja la cinética de una reacción: la concentración en función del tiempo para orden 0, 1 o
2 (o varias curvas para comparar), su linealización, la gráfica de Arrhenius, o los modelos
consecutivo (A → B → C) y reversible.

**Calcula:** Las concentraciones integradas de cada orden y la vida media. En la linealización, la recta de cada orden con su pendiente. En el consecutivo, el máximo del intermediario; en Arrhenius, k a cada temperatura.

| Campo | Tipo | Valores · por defecto | Descripción |
| :--- | :--- | :--- | :--- |
| `order` | número |  | El orden, 0, 1 o 2. |
| `k` | número |  | La constante de velocidad. |
| `A0` | número |  | La concentración inicial. |
| `curves` | lista |  | Varias curvas para comparar, cada una con su `order`, `k`, `A0` y `label`. |
| `plot` | texto |  | `linear` (la recta de cada orden) o `arrhenius`. |
| `Ea` | número |  | En Arrhenius, la energía de activación en kJ/mol. |
| `A` | número |  | En Arrhenius, el factor preexponencial. |
| `T` | lista |  | En Arrhenius, las temperaturas `[T1, T2]`. |
| `model` | texto |  | `consecutive` (A → B → C) o `reversible`. |
| `k1` | número |  | En el consecutivo, la constante de A → B. |
| `k2` | número |  | En el consecutivo, la constante de B → C. |
| `kf` | número |  | En el reversible, la constante directa. |
| `kb` | número |  | En el reversible, la constante inversa. |
| `species` | lista |  | Los nombres de las especies. |
| `showProduct` | sí/no |  | Dibuja también el producto. |
| `tEnd` | número |  | El tiempo final. |
| `annotations` | lista |  | `halfLife` marca la vida media. |
| `xLabel` | texto |  | Rótulo del eje horizontal. |
| `yLabel` | texto |  | Rótulo del eje vertical. |
| `blankAnnotations` | sí/no |  | Deja en blanco las anotaciones. |

**Modo ejercicio:** Campos: `blankAnnotations`.

Ejemplo: Órdenes 0, 1 y 2

```yaml
type: kinetics
title: Órdenes 0, 1 y 2
curves:
  - {order: 0, k: 0.05, A0: 1}
  - {order: 1, k: 0.1, A0: 1}
  - {order: 2, k: 0.2, A0: 1}
tEnd: 30
```

Ejemplo: Cinética consecutiva A → B → C

```yaml
type: kinetics
title: Cinética consecutiva A → B → C
model: consecutive
k1: 0.5
k2: 0.2
inkSaver: false
```

Ejemplo: Gráfico de Arrhenius

```yaml
type: kinetics
title: Gráfico de Arrhenius
plot: arrhenius
Ea: 75
A: 1.0e+12
T: [290, 360]
```

<a id="phase_diagram"></a>

## `phase_diagram`: Diagrama de fases

*Química › Termodinámica.* Diagramas de fases de una sustancia y diagramas eutécticos de dos componentes.

Dibuja el diagrama P-T de una sustancia con las curvas de fusión, vaporización y sublimación,
los puntos triple y crítico y las regiones sólido, líquido y gas; o, con `kind: eutectic`, el
diagrama de dos componentes con su punto eutéctico y una curva de enfriamiento.

**Calcula:** Las curvas, ajustadas a los puntos triple, crítico y normales.

| Campo | Tipo | Valores · por defecto | Descripción |
| :--- | :--- | :--- | :--- |
| `kind` | texto |  | `substance` (por omisión) o `eutectic`. |
| `substance` | texto |  | Una sustancia conocida, `water` o `co2`. |
| `triple` | lista |  | El punto triple, `[T en K, P en atm]`. |
| `critical` | lista |  | El punto crítico. |
| `meltingSlope` | número |  | La pendiente de la curva de fusión (negativa en el agua). |
| `normalMelting` | número |  | La temperatura normal de fusión. |
| `normalBoiling` | número |  | La temperatura normal de ebullición. |
| `normalSublimation` | número |  | La temperatura normal de sublimación. |
| `scale` | texto |  | `schematic` (por omisión) o `log` (presión en escala logarítmica). |
| `atm` | sí/no |  | Dibuja la recta de 1 atm. |
| `points` | lista |  | Puntos marcados: `T`, `P` y `label`. |
| `supercritical` | sí/no |  | Marca la región supercrítica. |
| `blankRegions` | sí/no |  | Deja en blanco los nombres de las regiones. |
| `blankPoints` | sí/no |  | Deja en blanco los puntos. |
| `components` | lista |  | En `eutectic`, los dos componentes. |
| `meltingA` | número |  | En `eutectic`, la fusión del componente A. |
| `meltingB` | número |  | En `eutectic`, la fusión del componente B. |
| `eutectic` | lista |  | En `eutectic`, el punto eutéctico `[x %, T]`. |
| `cooling` | número |  | En `eutectic`, una composición para dibujar su enfriamiento. |
| `blankCooling` | sí/no |  | En `eutectic`, deja en blanco el enfriamiento. |
| `yLabel` | texto |  | Rótulo del eje vertical. |

**Modo ejercicio:** Campos: `blankRegions`, `blankPoints`, `blankCooling`.

Ejemplo: CO2 en escala logarítmica

```yaml
type: phase_diagram
title: CO2 en escala logarítmica
substance: co2
scale: log
blankRegions: true
blankPoints: true
```

Ejemplo: Diagrama eutéctico Pb–Sn

```yaml
type: phase_diagram
kind: eutectic
title: Diagrama eutéctico Pb–Sn
components: [Pb, Sn]
meltingA: 327
meltingB: 232
eutectic: [61.9, 183]
cooling: 30
inkSaver: false
```

<a id="heating_curve"></a>

## `heating_curve`: Curva de calentamiento

*Química › Termodinámica.* La temperatura en función del calor entregado, con los cambios de fase.

Dibuja la curva de calentamiento de una masa de sustancia, desde una temperatura inicial hasta
una final: los tramos en cada fase y las mesetas de fusión y ebullición, con el calor de cada
tramo y el total.

**Calcula:** El calor de cada tramo (m·c·ΔT y m·ΔH) y el total.

| Campo | Tipo | Valores · por defecto | Descripción |
| :--- | :--- | :--- | :--- |
| `substance` | texto |  | Una sustancia conocida, `water`. |
| `cpSolid` | número |  | El calor específico del sólido, en J/g·K. |
| `cpLiquid` | número |  | El calor específico del líquido. |
| `cpGas` | número |  | El calor específico del gas. |
| `melting` | número |  | La temperatura de fusión, en °C. |
| `boiling` | número |  | La temperatura de ebullición. |
| `hFusion` | número |  | El calor de fusión, en J/g. |
| `hVaporization` | número |  | El calor de vaporización. |
| `mass` | número |  | La masa, en g. |
| `from` | número |  | La temperatura inicial. |
| `to` | número |  | La temperatura final. |
| `showHeat` | sí/no |  | Escribe el calor de cada tramo. |
| `showTotal` | sí/no |  | Escribe el calor total. |
| `blankHeat` | sí/no |  | Deja en blanco los calores. |
| `blankPhases` | sí/no |  | Deja en blanco los nombres de las fases. |

**Modo ejercicio:** Campos: `blankHeat`, `blankPhases`.

Ejemplo: 100 g de hielo de −20 °C a 120 °C

```yaml
type: heating_curve
title: 100 g de hielo de −20 °C a 120 °C
substance: water
mass: 100
from: -20
to: 120
```

Ejemplo: Curva de calentamiento (evaluación)

```yaml
type: heating_curve
title: Curva de calentamiento (evaluación)
substance: water
mass: 50
from: -10
to: 110
blankHeat: true
blankPhases: true
inkSaver: false
```

<a id="titration"></a>

## `titration`: Titulación

*Química › Equilibrio y análisis.* Curvas de titulación de ácidos y bases, fuertes, débiles y polipróticos, con indicador.

Calcula y dibuja el pH en función del volumen de titulante, con los puntos de equivalencia
que tienen un salto visible, la semiequivalencia, el pH inicial y la franja de viraje del
indicador. También dibuja puntos medidos.

**Calcula:** El pH de cada punto por balance de cargas, también para ácidos polipróticos. Los puntos de equivalencia, la semiequivalencia y el pH inicial.

| Campo | Tipo | Valores · por defecto | Descripción |
| :--- | :--- | :--- | :--- |
| `analyte` | objeto |  | El analito: `kind` (`strong_acid`, `strong_base`, `weak_acid`, `weak_base`), `conc`, `volume`, `pKa` (uno o una lista) o `pKb`, y `charge` de la forma agregada (`-2` para el carbonato). |
| `titrant` | objeto |  | El titulante: `conc` y `name`. |
| `vMax` | número |  | El volumen máximo de titulante. |
| `annotations` | lista |  | `equivalence`, `half`, `initial`. |
| `indicator` | objeto |  | El indicador: `name` y `range` (`[8.2, 10]`). |
| `data` | lista |  | Puntos medidos `[V, pH]`. |
| `blankAnnotations` | sí/no |  | Deja en blanco las anotaciones. |

**Modo ejercicio:** Campos: `blankAnnotations`.

Ejemplo: CH3COOH 0,1 M con NaOH 0,1 M

```yaml
type: titration
title: CH3COOH 0,1 M con NaOH 0,1 M
analyte: {kind: weak_acid, pKa: 4.76, conc: 0.1, volume: 25}
titrant: {conc: 0.1, name: NaOH}
indicator: {name: fenolftaleína, range: [8.2, 10]}
annotations: [equivalence, half, initial]
```

Ejemplo: Ácido poliprótico H3PO4 con NaOH

```yaml
type: titration
title: Ácido poliprótico H3PO4 con NaOH
analyte: {kind: weak_acid, pKa: [2.15, 7.2, 12.35], conc: 0.1, volume: 20}
titrant: {conc: 0.1}
inkSaver: false
```

Ejemplo: Base débil NH3 con HCl

```yaml
type: titration
title: Base débil NH3 con HCl
analyte: {kind: weak_base, pKb: 4.75, conc: 0.1, volume: 25}
titrant: {conc: 0.1, name: HCl}
blankAnnotations: true
```

- En la semiequivalencia, el dibujo muestra el pH calculado, que con un pKa bajo ya no es igual al pKa.

<a id="speciation"></a>

## `speciation`: Especiación

*Química › Equilibrio y análisis.* Las fracciones α de cada especie de un ácido en función del pH.

Dibuja las fracciones α de cada especie de un ácido monoprótico o poliprótico en función del
pH, con los pKa marcados donde se cruzan las curvas.

**Calcula:** Las fracciones de cada especie a partir de los pKa.

| Campo | Tipo | Valores · por defecto | Descripción |
| :--- | :--- | :--- | :--- |
| `pKa` | lista |  | Los pKa. |
| `species` | lista |  | Los nombres de las especies (por omisión H₃A, H₂A⁻…). |
| `blankPKa` | sí/no |  | Deja los pKa en blanco. |
| `blankSpecies` | sí/no |  | Deja en blanco los nombres de las especies. |

**Modo ejercicio:** Campos: `blankPKa`, `blankSpecies`.

Ejemplo: Especiación del ácido fosfórico

```yaml
type: speciation
title: Especiación del ácido fosfórico
pKa: [2.15, 7.2, 12.35]
species: [H3PO4, H2PO4^-, HPO4^2-, PO4^3-]
```

Ejemplo: Sistema carbonato (pKa en blanco)

```yaml
type: speciation
title: Sistema carbonato (pKa en blanco)
pKa: [6.35, 10.33]
species: [H2CO3, HCO3^-, CO3^2-]
blankPKa: true
inkSaver: false
```

<a id="spectrum"></a>

## `spectrum`: Espectro

*Química › Equilibrio y análisis.* Espectros IR, ¹H-RMN, ¹³C-RMN, UV-visible y de masas.

Dibuja un espectro: IR (transmitancia con bandas), RMN de protón (señales con su
multiplicidad e integración) o de carbono, UV-visible (absorbancia) y de masas (picos por
m/z), con sus ejes y rótulos.

**Calcula:** En RMN, las líneas de cada multiplete por la regla n + 1.

| Campo | Tipo | Valores · por defecto | Descripción |
| :--- | :--- | :--- | :--- |
| `kind` | texto |  | `ir`, `nmr`, `c13`, `uv` o `ms`. |
| `peaks` | lista |  | En `ir`, `at` (cm⁻¹), `depth`, `width`, `broad`, `label`; en `ms`, `mz`, `intensity`, `label`. |
| `signals` | lista |  | En `nmr`, `shift`, `integration`, `multiplicity` (s, d, t, q…) o `neighbors`, `J`, `label`; en `c13`, `shift` y `label`. |
| `bands` | lista |  | En `uv`, `at` (nm), `absorbance`, `width`. |
| `MHz` | número |  | En RMN, la frecuencia del equipo (por omisión 400). |
| `lineSpacing` | número |  | En RMN, la separación entre líneas de un multiplete (por omisión 5). |
| `showIntegration` | sí/no |  | En RMN, escribe la integración (por omisión, sí). |
| `tms` | sí/no |  | En RMN, la señal del TMS en 0. |
| `range` | lista |  | El rango del eje horizontal. |
| `blankLabels` | sí/no |  | Deja los rótulos en blanco. |

**Modo ejercicio:** Campos: `blankLabels`.

Ejemplo: Espectro IR de ácido carboxílico

```yaml
type: spectrum
kind: ir
title: Espectro IR de ácido carboxílico
peaks:
  - {at: 3000, depth: 0.75, broad: true, width: 500, label: "O–H"}
  - {at: 1710, depth: 0.85, label: "C=O"}
  - {at: 1280, depth: 0.55, label: "C–O"}
```

Ejemplo: ¹H-RMN del etanol

```yaml
type: spectrum
kind: nmr
title: ¹H-RMN del etanol
tms: true
signals:
  - {shift: 1.22, integration: 3, neighbors: 2, J: 7, label: "a"}
  - {shift: 3.69, integration: 2, neighbors: 3, J: 7, label: "b"}
  - {shift: 2.6, integration: 1, multiplicity: s, label: "c"}
inkSaver: false
```

Ejemplo: Espectro de masas del etanol

```yaml
type: spectrum
kind: ms
title: Espectro de masas del etanol
peaks:
  - {mz: 46, intensity: 20, label: "M⁺"}
  - {mz: 45, intensity: 45}
  - {mz: 31, intensity: 100, label: "pico base"}
  - {mz: 29, intensity: 25}
```

<a id="galvanic_cell"></a>

## `galvanic_cell`: Pila

*Química › Electroquímica.* Pilas galvánicas con semirreacciones, flujo de electrones, notación, fem y ΔG.

Dibuja una pila con sus dos semiceldas, el puente salino y el circuito externo. El ánodo es la
semicelda de menor E°. Opcionalmente, las semirreacciones, el sentido de los electrones y de
los iones, la notación de la pila, la fem y ΔG.

**Calcula:** El ánodo y el cátodo, la fem estándar y con Nernst, y ΔG = −nFE.

| Campo | Tipo | Valores · por defecto | Descripción |
| :--- | :--- | :--- | :--- |
| `halfCells` | lista |  | Las dos semiceldas: `{couple: "Zn^2+/Zn", E0, n, conc, electrode, solution, reaction}`. |
| `anode` | objeto |  | La semicelda del ánodo, en lugar de `halfCells`. |
| `cathode` | objeto |  | La semicelda del cátodo. |
| `saltBridge` | objeto |  | El puente salino: `name` (por omisión KNO₃), `cation` y `anion`. |
| `showIons` | sí/no |  | Dibuja el movimiento de los iones del puente. |
| `showNotation` | sí/no |  | Escribe la notación de la pila. |
| `showDeltaG` | sí/no |  | Escribe ΔG. |
| `blankPolarity` | sí/no |  | Deja en blanco el signo de los electrodos. |
| `blankReactions` | sí/no |  | Deja en blanco las semirreacciones. |
| `blankPotential` | sí/no |  | Deja la fem en blanco. |
| `blankFlow` | sí/no |  | Deja en blanco el sentido de los electrones. |
| `blankNotation` | sí/no |  | Deja la notación en blanco. |

**Modo ejercicio:** Campos: `blankPolarity`, `blankReactions`, `blankPotential`, `blankFlow`, `blankNotation`.

Ejemplo: Pila de Daniell (Nernst + ΔG°)

```yaml
type: galvanic_cell
title: Pila de Daniell (Nernst + ΔG°)
halfCells:
  - {couple: "Cu^2+/Cu", E0: 0.34, n: 2, conc: 1}
  - {couple: "Zn^2+/Zn", E0: -0.76, n: 2, conc: 0.01}
showDeltaG: true
```

Ejemplo: Celda galvánica (para completar)

```yaml
type: galvanic_cell
title: Celda galvánica (para completar)
halfCells:
  - {couple: "Ag^+/Ag", E0: 0.80, n: 1}
  - {couple: "Cu^2+/Cu", E0: 0.34, n: 2}
blankPolarity: true
blankPotential: true
blankReactions: true
blankFlow: true
inkSaver: false
```

<a id="latimer"></a>

## `latimer`: Diagrama de Latimer

*Química › Electroquímica.* Potenciales entre estados de oxidación y potenciales entre especies no vecinas.

Dibuja las especies de un elemento de la más oxidada a la más reducida con el potencial
estándar entre cada par. Calcula el potencial entre especies no vecinas y marca las que se
desproporcionan.

**Calcula:** E° entre dos especies cualesquiera, Σ nE° / Σ n. Las especies que se desproporcionan (E° a la derecha mayor que a la izquierda).

| Campo | Tipo | Valores · por defecto | Descripción |
| :--- | :--- | :--- | :--- |
| `species` | lista |  | Las especies, de la más oxidada a la más reducida. |
| `oxidation` | lista |  | El estado de oxidación de cada especie. |
| `potentials` | lista |  | El E° entre cada par de vecinas. |
| `compute` | lista |  | Pares de índices `[i, j]` cuyo potencial se calcula. |
| `showDisproportionation` | sí/no |  | Marca las especies que se desproporcionan. |
| `blankPotentials` | lista |  | Índices de los potenciales que quedan en blanco. |
| `blankComputed` | sí/no |  | Deja en blanco los potenciales calculados. |
| `blankDisproportionation` | sí/no |  | Deja en blanco la desproporción. |

**Modo ejercicio:** Campos: `blankPotentials`, `blankComputed`, `blankDisproportionation`.

Ejemplo: Diagrama Latimer del Mn (medio ácido)

```yaml
type: latimer
title: Diagrama Latimer del Mn (medio ácido)
species: [MnO4^-, MnO4^2-, MnO2, Mn^3+, Mn^2+, Mn]
oxidation: [7, 6, 4, 3, 2, 0]
potentials: [0.56, 2.26, 0.95, 1.51, -1.18]
compute: [[0, 2], [0, 4]]
showDisproportionation: true
```

Ejemplo: Latimer Cu (en blanco)

```yaml
type: latimer
title: Latimer Cu (en blanco)
species: [Cu^2+, Cu^+, Cu]
oxidation: [2, 1, 0]
potentials: [0.16, 0.52]
compute: [[0, 2]]
blankComputed: true
showDisproportionation: true
blankDisproportionation: true
inkSaver: false
```

<a id="frost"></a>

## `frost`: Diagrama de Frost

*Química › Electroquímica.* N·E° en función del estado de oxidación, con las especies que se desproporcionan.

Dibuja N·E° de cada especie respecto del estado 0 en función del estado de oxidación, con
los mismos datos que un diagrama de Latimer. Las especies que quedan por encima de la recta
entre sus vecinas se desproporcionan.

| Campo | Tipo | Valores · por defecto | Descripción |
| :--- | :--- | :--- | :--- |
| `species` | lista |  | Las especies, de la más oxidada a la más reducida. |
| `oxidation` | lista |  | El estado de oxidación de cada especie. |
| `potentials` | lista |  | El E° entre cada par de vecinas. |
| `showDisproportionation` | sí/no |  | Marca las especies que se desproporcionan. |
| `blankLabels` | sí/no |  | Deja en blanco los rótulos. |

**Modo ejercicio:** Campos: `blankLabels`.

Ejemplo: Diagrama de Frost del Mn

```yaml
type: frost
title: Diagrama de Frost del Mn
species: [MnO4^-, MnO4^2-, MnO2, Mn^3+, Mn^2+, Mn]
oxidation: [7, 6, 4, 3, 2, 0]
potentials: [0.56, 2.26, 0.95, 1.51, -1.18]
showDisproportionation: true
inkSaver: false
```

<a id="newman"></a>

## `newman`: Proyección de Newman

*Química › Química orgánica.* Conformaciones alrededor de un enlace C–C, con el diedro y su nombre.

Dibuja la proyección de Newman de un enlace: los tres sustituyentes del carbono de adelante y
los del de atrás, girados según el diedro. Nombra la conformación (anti, gauche, eclipsada).

| Campo | Tipo | Valores · por defecto | Descripción |
| :--- | :--- | :--- | :--- |
| `front` | lista |  | Los tres sustituyentes del carbono de adelante. |
| `back` | lista |  | Los tres del de atrás. |
| `dihedral` | número |  | El diedro entre el primero de adelante y el primero de atrás: 180 anti, 60 gauche, 0 eclipsada. |
| `start` | texto |  | El ángulo del primer sustituyente de adelante. |
| `axis` | texto |  | El rótulo del enlace (`C2–C3`). |
| `blank` | sí/no |  | Deja los sustituyentes en blanco. |
| `blankName` | sí/no |  | Deja en blanco el nombre de la conformación. |
| `blankDihedral` | sí/no |  | Deja el diedro en blanco. |

**Modo ejercicio:** Campos: `blank`, `blankName`, `blankDihedral`.

Ejemplo: Butano conformación anti (C2–C3)

```yaml
type: newman
title: Butano conformación anti (C2–C3)
axis: C2–C3
front: [CH3, H, H]
back: [CH3, H, H]
dihedral: 180
```

Ejemplo: Conformación gauche (en blanco)

```yaml
type: newman
title: Conformación gauche (en blanco)
front: [CH3, H, H]
back: [CH3, H, H]
dihedral: 60
blankName: true
inkSaver: false
```

<a id="chair"></a>

## `chair`: Silla del ciclohexano

*Química › Química orgánica.* Conformaciones de silla con sustituyentes axiales y ecuatoriales, cis/trans y estabilidad.

Dibuja la silla del ciclohexano con sus sustituyentes axiales y ecuatoriales y, con `flip`,
también la silla invertida. Dice si los sustituyentes son cis o trans y, con los valores A
conocidos, cuál silla es más estable y por cuánta energía.

**Calcula:** La relación cis/trans y la diferencia de energía entre las sillas por valores A (CH₃ 7,3 kJ/mol, tBu 21…).

| Campo | Tipo | Valores · por defecto | Descripción |
| :--- | :--- | :--- | :--- |
| `substituents` | lista |  | Los sustituyentes: `position` (1 a 6), `group`, `axial` y `blank`. |
| `flip` | sí/no |  | Dibuja también la silla invertida (los axiales pasan a ecuatoriales). |
| `showRelation` | sí/no |  | Escribe si es cis o trans (por omisión, sí). |
| `showStability` | sí/no |  | Escribe cuál silla es más estable. |
| `blank` | sí/no |  | Deja los sustituyentes en blanco. |
| `blankRelation` | sí/no |  | Deja en blanco la relación cis/trans. |
| `blankStability` | sí/no |  | Deja en blanco la estabilidad. |

**Modo ejercicio:** Campos: `blank`, `blankRelation`, `blankStability`.

Ejemplo: Metilciclohexano con flip

```yaml
type: chair
title: Metilciclohexano con flip
substituents:
  - {position: 1, group: CH3, axial: true}
flip: true
```

Ejemplo: Ciclohexano disustituido (blanco)

```yaml
type: chair
title: Ciclohexano disustituido (blanco)
substituents:
  - {position: 1, group: tBu, axial: false}
  - {position: 4, group: CH3, axial: false}
flip: true
blank: true
inkSaver: false
```

<a id="fischer"></a>

## `fischer`: Proyección de Fischer

*Química › Química orgánica.* Proyecciones de Fischer con R/S por las reglas CIP y la serie D/L.

Dibuja la proyección de Fischer de una molécula con uno o varios centros quirales, de arriba
hacia abajo. Con un solo centro asigna R o S por las reglas CIP para los grupos comunes; la
serie D o L sale del último centro.

**Calcula:** R/S con prioridades CIP (OH, NH₂, CHO, COOH, CH₂OH, CH₃, halógenos…), o con `priorities`. La serie D/L.

| Campo | Tipo | Valores · por defecto | Descripción |
| :--- | :--- | :--- | :--- |
| `top` | texto |  | El grupo de arriba. |
| `bottom` | texto |  | El grupo de abajo. |
| `centers` | lista |  | Los centros, de arriba hacia abajo: `left`, `right`, `blank` (`left`, `right` o `true`) y `priorities` (los lugares `top`, `right`, `bottom`, `left` de mayor a menor prioridad). |
| `showConfig` | sí/no |  | Escribe R o S. |
| `showSeries` | sí/no |  | Escribe la serie D o L (por omisión, sí). |
| `blank` | sí/no |  | Deja los grupos en blanco. |
| `blankConfig` | sí/no |  | Deja R/S en blanco. |
| `blankSeries` | sí/no |  | Deja la serie en blanco. |

**Modo ejercicio:** Campos: `blank`, `blankConfig`, `blankSeries`.

Ejemplo: D-glucosa (4 centros quirales)

```yaml
type: fischer
title: D-glucosa (4 centros quirales)
top: CHO
bottom: CH2OH
centers:
  - {left: H, right: OH, priorities: [right, top, bottom, left]}
  - {left: HO, right: H, priorities: [left, top, bottom, right]}
  - {left: H, right: OH, priorities: [right, top, bottom, left]}
  - {left: H, right: OH, priorities: [right, top, bottom, left]}
```

Ejemplo: Alanina (configuración en blanco)

```yaml
type: fischer
title: Alanina (configuración en blanco)
top: COOH
bottom: CH3
centers:
  - {left: NH2, right: H}
blankConfig: true
blankSeries: true
inkSaver: false
```

<a id="crystal_field"></a>

## `crystal_field`: Campo cristalino

*Química › Química inorgánica.* El desdoblamiento de los orbitales d, alto y bajo espín, desapareados, μ y EECC.

Dibuja el desdoblamiento de los orbitales d en un campo octaédrico, tetraédrico o cuadrado
plano y los llena con los electrones d del metal, en alto o bajo espín según el ligando.
Calcula los electrones desapareados, el momento magnético de solo espín y la energía de
estabilización.

**Calcula:** Los electrones d a partir del metal y su estado de oxidación. El espín por la serie espectroquímica (desde el NH₃, campo fuerte), μ = √(n(n+2)) y la EECC.

| Campo | Tipo | Valores · por defecto | Descripción |
| :--- | :--- | :--- | :--- |
| `geometry` | texto |  | `octahedral`, `tetrahedral` o `square_planar`. |
| `electrons` | número |  | Los electrones d, o `metal` y `oxidation`. |
| `metal` | texto |  | El metal. |
| `oxidation` | número |  | Su estado de oxidación. |
| `spin` | texto |  | `high` o `low`, o el que dé `ligand`. |
| `ligand` | texto |  | El ligando, de la serie espectroquímica. |
| `complex` | texto |  | La fórmula del complejo, de título. |
| `blank` | sí/no |  | Deja los orbitales vacíos. |
| `blankElectrons` | sí/no |  | Deja los electrones en blanco. |
| `blankConfig` | sí/no |  | Deja la configuración en blanco. |
| `blankSpin` | sí/no |  | Deja el espín en blanco. |
| `blankUnpaired` | sí/no |  | Deja los desapareados en blanco. |
| `blankMoment` | sí/no |  | Deja μ en blanco. |
| `blankCFSE` | sí/no |  | Deja la EECC en blanco. |

**Modo ejercicio:** Campos: `blank`, `blankElectrons`, `blankConfig`, `blankSpin`, `blankUnpaired`, `blankMoment`, `blankCFSE`.

Ejemplo: [Co(NH3)6]^3+ (bajo espín)

```yaml
type: crystal_field
title: "[Co(NH3)6]^3+ (bajo espín)"
metal: Co
oxidation: 3
ligand: NH3
inkSaver: false
```

Ejemplo: [Fe(CN)6]^4- (evaluación)

```yaml
type: crystal_field
title: "[Fe(CN)6]^4- (evaluación)"
metal: Fe
oxidation: 2
ligand: CN^-
blankElectrons: true
blank: true
```

<a id="complex"></a>

## `complex`: Complejo de coordinación

*Química › Química inorgánica.* La geometría de un complejo con su fórmula, estado de oxidación e isomería.

Dibuja un complejo de coordinación octaédrico, tetraédrico o cuadrado plano con sus ligandos
en su lugar, y calcula la fórmula, el estado de oxidación del metal, la coordinación, los
electrones d y el isómero (cis/trans, fac/mer).

**Calcula:** La fórmula y el estado de oxidación a partir de las cargas de los ligandos. El isómero geométrico.

| Campo | Tipo | Valores · por defecto | Descripción |
| :--- | :--- | :--- | :--- |
| `geometry` | texto |  | `octahedral`, `tetrahedral` o `square_planar`. |
| `metal` | texto |  | El metal. |
| `ligands` | lista |  | Los ligandos, en orden: arriba, abajo, izquierda, derecha, adelante, atrás (en el cuadrado plano: izquierda, derecha, adelante, atrás). |
| `charge` | número |  | La carga del complejo. |
| `oxidation` | número |  | El estado de oxidación, si un ligando no es de la tabla. |
| `blank` | lista |  | `true` o una lista de `formula`, `oxidation`, `coordination`, `d` e `isomer`. |

**Modo ejercicio:** Campos: `blank`.

Ejemplo: trans-[Co(NH3)4Cl2]^+

```yaml
type: complex
title: "trans-[Co(NH3)4Cl2]^+"
metal: Co
charge: 1
ligands: [NH3, NH3, Cl, Cl, NH3, NH3]
inkSaver: false
```

Ejemplo: Complejo fac (para completar)

```yaml
type: complex
title: Complejo fac (para completar)
metal: Co
ligands: [Cl, NH3, Cl, NH3, Cl, NH3]
blank: [isomer, oxidation]
```

<a id="born_haber"></a>

## `born_haber`: Ciclo de Born-Haber

*Química › Química inorgánica.* La energía de red de un sólido iónico por la ley de Hess.

Dibuja el ciclo de Born-Haber como un diagrama de niveles: desde los elementos, cada paso
hasta los iones gaseosos y la formación del sólido, y calcula la energía de red por la ley de
Hess.

**Calcula:** La energía de red, como la diferencia entre ΔHf y la suma de los pasos.

| Campo | Tipo | Valores · por defecto | Descripción |
| :--- | :--- | :--- | :--- |
| `start` | texto |  | El estado inicial (los elementos). |
| `steps` | lista |  | Los pasos hasta los iones gaseosos: `to`, `dH`, `label` y `blank`. |
| `formation` | número |  | La entalpía de formación ΔHf. |
| `product` | texto |  | El sólido. |
| `blankLattice` | sí/no |  | Deja la energía de red en blanco. |

**Modo ejercicio:** Campos: `blankLattice`.

Ejemplo: Ciclo de Born-Haber del NaCl

```yaml
type: born_haber
title: Ciclo de Born-Haber del NaCl
start: "Na(s) + ½ Cl2(g)"
product: "NaCl(s)"
formation: -411
steps:
  - {to: "Na(g) + ½ Cl2(g)", dH: 107, label: "ΔH_sub"}
  - {to: "Na^+(g) + ½ Cl2(g)", dH: 496, label: "EI"}
  - {to: "Na^+(g) + Cl(g)", dH: 122, label: "½ D"}
  - {to: "Na^+(g) + Cl^-(g)", dH: -349, label: "AE"}
```

Ejemplo: Ciclo KBr (energía reticular en blanco)

```yaml
type: born_haber
title: Ciclo KBr (energía reticular en blanco)
start: "K(s) + ½ Br2(l)"
product: "KBr(s)"
formation: -394
steps:
  - {to: "K(g)", dH: 89, label: "ΔH_sub"}
  - {to: "K^+(g)", dH: 419, label: "EI"}
  - {to: "K^+(g) + Br(g)", dH: 112, label: "½ D + vap"}
  - {to: "K^+(g) + Br^-(g)", dH: -325, label: "AE"}
blankLattice: true
inkSaver: false
```

<a id="signal"></a>

## `signal`: Señal

*Matemática aplicada › Señales y sistemas.* Señales continuas y discretas definidas por una expresión.

Evalúa una o varias expresiones de t (continuas) o de n (discretas) y las grafica. Las
continuas se dibujan como curvas, juntas con su leyenda o una debajo de la otra; las
discretas, como varillas, una por gráfico.

| Campo | Tipo | Valores · por defecto | Descripción |
| :--- | :--- | :--- | :--- |
| `expr` | texto |  | La expresión: `+ − * / ^`, multiplicación implícita (`2t`, `3(t+1)`), `pi` y `e`, y las funciones `sin`, `cos`, `tan`, `exp`, `log` (ln), `log10`, `sqrt`, `abs`, `sgn`, `u` (escalón), `rect`, `tri`, `sinc`, `ramp`, `floor`, `ceil`, `round`, `min`, `max`, `pow`, `mod`, `delta`. |
| `label` | texto |  | El nombre de la señal (`x(t)`). |
| `signals` | lista |  | Varias señales: `{expr, label}` o una lista de expresiones (hasta 6). |
| `domain` | texto | `continuous`, `discrete` · por defecto `continuous` | `continuous` o `discrete`. |
| `variable` | texto |  | La variable (por omisión `t`, o `n` en las discretas). |
| `range` | lista |  | El intervalo de la variable, `[desde, hasta]` (hasta 400 muestras en las discretas). |
| `yrange` | lista |  | El rango del eje vertical, si no se calcula solo. |
| `stack` | sí/no |  | Un gráfico por señal (por omisión, en las discretas). |
| `height` | número |  | La altura de cada gráfico, en píxeles. |
| `blank` | sí/no |  | Dibuja los ejes sin las curvas, para graficarlas. |

**Modo ejercicio:** Campos: `blank`.

Ejemplo: Operaciones sobre señales continuas

```yaml
type: signal
title: Operaciones sobre señales continuas
signals:
  - { expr: "rect((t-1)/2)", label: "x(t)" }
  - { expr: "rect((2t-1)/2)", label: "x(2t)" }
range: [-1, 4]
```

Ejemplo: Señal discreta exponencial

```yaml
type: signal
title: Señal discreta exponencial
domain: discrete
expr: "0.8^n u(n)"
label: "x[n]"
range: [-4, 12]
```

<a id="convolution"></a>

## `convolution`: Convolución

*Matemática aplicada › Señales y sistemas.* La convolución discreta, con el paso de voltear y desplazar.

Calcula y dibuja la convolución discreta y[n] = Σ x[k]·h[n−k]: las secuencias x[k] y h[k] y
el resultado y[n]. Con `step`, agrega el paso intermedio: h[n−k] volteada y desplazada sobre
x[k], con la cuenta de esa muestra.

**Calcula:** y[n] completa y, con `step`, los productos de la muestra elegida.

| Campo | Tipo | Valores · por defecto | Descripción |
| :--- | :--- | :--- | :--- |
| `x` | lista |  | Las muestras de x (hasta 64). |
| `xStart` | número |  | El índice de la primera muestra de x (por omisión 0). |
| `h` | lista |  | Las muestras de h. |
| `hStart` | número |  | El índice de la primera muestra de h. |
| `step` | número |  | Un n para mostrar el paso de voltear y desplazar. |
| `values` | sí/no |  | Escribe y[n] como lista debajo (por omisión, sí). |
| `blank` | sí/no |  | Oculta y[n] y el resultado del paso. |

**Modo ejercicio:** Campos: `blank`.

Ejemplo: Convolución discreta paso a paso

```yaml
type: convolution
title: Convolución discreta paso a paso
x: [1, 2, 1]
h: [1, 1, 1, 1]
step: 2
```

<a id="fourier"></a>

## `fourier`: Fourier

*Matemática aplicada › Señales y sistemas.* La DFT de una secuencia y la serie de Fourier de ondas clásicas.

Con `sequence`, calcula la DFT de una secuencia y dibuja la secuencia, el módulo |X[k]| (y la
fase) y una tabla con cada X[k]. Con `wave`, dibuja el espectro de amplitud de la serie de
Fourier de una onda cuadrada, diente de sierra o triangular, y la suma parcial frente a la
onda ideal.

**Calcula:** La DFT de N puntos. Los coeficientes de la serie y la suma parcial con los primeros armónicos.

| Campo | Tipo | Valores · por defecto | Descripción |
| :--- | :--- | :--- | :--- |
| `sequence` | lista |  | La secuencia, para la DFT (hasta 128 muestras). |
| `phase` | sí/no |  | Agrega la fase de X[k]. |
| `table` | sí/no |  | La tabla de X[k] (por omisión, sí, hasta 16 muestras). |
| `wave` | texto |  | `square`, `sawtooth` o `triangle`. |
| `harmonics` | número |  | La cantidad de armónicos (de 1 a 60, por omisión 7). |
| `amplitude` | número |  | La amplitud A de la onda. |
| `period` | número |  | El período T. |
| `blank` | sí/no |  | Oculta el espectro (y la suma parcial) y los valores de la tabla. |

**Modo ejercicio:** Campos: `blank`.

Ejemplo: Serie de Fourier de una onda cuadrada

```yaml
type: fourier
title: Serie de Fourier de una onda cuadrada
wave: square
harmonics: 9
```

Ejemplo: DFT de 8 puntos

```yaml
type: fourier
title: DFT de 8 puntos
sequence: [1, 1, 1, 1, 0, 0, 0, 0]
```

<a id="sampling"></a>

## `sampling`: Muestreo

*Matemática aplicada › Señales y sistemas.* El muestreo de un tono, el aliasing y el criterio de Nyquist.

Dibuja un tono senoidal, sus muestras y, si no se cumple Nyquist, el alias que pasa por las
mismas muestras. Debajo, si hay aliasing y la frecuencia aparente. Con `spectrum`, agrega las
réplicas del espectro en k·fs ± f₀.

**Calcula:** La frecuencia aparente del tono muestreado.

| Campo | Tipo | Valores · por defecto | Descripción |
| :--- | :--- | :--- | :--- |
| `frequency` | número |  | La frecuencia del tono, en Hz. |
| `fs` | número |  | La frecuencia de muestreo, en Hz. |
| `amplitude` | número |  | La amplitud. |
| `phase` | número |  | La fase, en grados. |
| `periods` | número |  | Cuántos períodos del tono se dibujan (por omisión 3). |
| `reconstruction` | sí/no |  | Dibuja el alias (por omisión, sí). |
| `spectrum` | sí/no |  | Agrega el espectro con las réplicas. |
| `blank` | sí/no |  | Oculta el alias y deja en blanco las conclusiones. |

**Modo ejercicio:** Campos: `blank`.

Ejemplo: Muestreo con aliasing

```yaml
type: sampling
title: Muestreo con aliasing
frequency: 9
fs: 12
periods: 3
```

Ejemplo: Muestreo que cumple Nyquist

```yaml
type: sampling
title: Muestreo que cumple Nyquist
frequency: 2
fs: 10
periods: 2
spectrum: true
```

<a id="distribution"></a>

## `distribution`: Distribución

*Matemática aplicada › Probabilidad.* Densidades y funciones de masa con el área de un intervalo y su probabilidad exacta.

Dibuja la densidad de una distribución continua o la función de masa de una discreta, sombrea
el intervalo pedido y calcula su probabilidad, además de la esperanza y la varianza.

**Calcula:** La probabilidad exacta con la función de distribución: la normal por la función gamma incompleta, la t por la beta incompleta, la χ² por la gamma incompleta y las discretas sumando la masa.

| Campo | Tipo | Valores · por defecto | Descripción |
| :--- | :--- | :--- | :--- |
| `dist` | texto | `normal`, `uniform`, `exponential`, `t`, `chi2`, `binomial`, `poisson`, `geometric` · por defecto `normal` | `normal`, `uniform`, `exponential`, `t`, `chi2`, `binomial`, `poisson` o `geometric`. |
| `mean` | número |  | En la normal, la media. |
| `sd` | número |  | En la normal, el desvío. |
| `a` | número |  | En la uniforme, el extremo inferior. |
| `b` | número |  | En la uniforme, el extremo superior. |
| `lambda` | número |  | En la exponencial y la Poisson, λ. |
| `df` | número |  | En la t y la χ², los grados de libertad. |
| `n` | número |  | En la binomial, los ensayos. |
| `p` | número |  | En la binomial y la geométrica, la probabilidad de éxito. |
| `interval` | lista |  | El intervalo `[a, b]`; `null` es −∞ o +∞. En las discretas, los extremos van incluidos. |
| `showMean` | sí/no |  | Marca la media. |
| `blank` | sí/no |  | Oculta el área sombreada y la probabilidad. |

**Modo ejercicio:** Campos: `blank`.

Ejemplo: Distribución normal estándar

```yaml
type: distribution
title: Distribución normal estándar
dist: normal
mean: 0
sd: 1
interval: [-1.96, 1.96]
```

Ejemplo: Distribución binomial

```yaml
type: distribution
title: Distribución binomial
dist: binomial
n: 10
p: 0.3
interval: [2, 4]
```

Ejemplo: Ejercicio: cola derecha de una t

```yaml
type: distribution
title: "Ejercicio: cola derecha de una t"
dist: t
df: 8
interval: [2.306, null]
blank: true
```

<a id="venn"></a>

## `venn`: Diagrama de Venn

*Matemática aplicada › Probabilidad.* Dos o tres conjuntos con las regiones de una expresión sombreadas.

Dibuja dos o tres conjuntos dentro del universo y sombrea las regiones que forman una
expresión de conjuntos. Con varias expresiones, un diagrama por cada una, rotulados a), b)…
Los elementos o cantidades de cada región se pueden escribir adentro.

**Calcula:** Las regiones de la expresión, con unión, intersección, diferencia, diferencia simétrica y complemento.

| Campo | Tipo | Valores · por defecto | Descripción |
| :--- | :--- | :--- | :--- |
| `sets` | lista |  | Los nombres de los conjuntos (2 o 3). |
| `shade` | lista o texto |  | Una expresión o una lista (hasta 8): `∪` (`\|`, `+`), `∩` (`&`, `·` o yuxtaposición), `−` (`\`), `Δ` (`^`) y el complemento con `'` o `¬`. `U` es el universo. |
| `regions` | objeto |  | Lo que va en cada región, por los conjuntos que la forman: `"A": [1, 3]`, `"A,B": [7]`, `U` (afuera de todos). |
| `counts` | objeto |  | Cantidades por región, con las mismas claves que `regions`. |
| `blank` | sí/no |  | Dibuja los conjuntos sin sombrear, para sombrear la expresión. |

**Modo ejercicio:** Campos: `blank`.

Ejemplo: Operaciones con conjuntos

```yaml
type: venn
title: Operaciones con conjuntos
sets: [A, B, C]
shade: ["A ∩ B", "(A ∪ B) − C", "A Δ B", "C'"]
```

Ejemplo: Elementos por región

```yaml
type: venn
title: Elementos por región
sets: [A, B]
shade: "A − B"
regions:
  A: [1, 3, 5]
  "A,B": [7, 9]
  B: [2, 4]
  U: [6, 8]
```

<a id="circuit"></a>

## `circuit`: Circuito

*Matemática aplicada › Circuitos eléctricos.* Redes serie y paralelo con su fuente, impedancia equivalente y corriente.

Dibuja un circuito con una fuente y una red de resistencias, inductores, capacitores y
fuentes armada como serie y paralelo anidados. Calcula la resistencia o la impedancia
equivalente (a la frecuencia dada) y la corriente de la fuente.

**Calcula:** La impedancia de cada elemento (R, jωL, 1/(jωC)) y la equivalente, en forma rectangular y polar. La corriente I = V/Z.

| Campo | Tipo | Valores · por defecto | Descripción |
| :--- | :--- | :--- | :--- |
| `network` | objeto |  | La red: `{series: […]}` o `{parallel: […]}` anidados; cada elemento es un texto (`"R1 = 10k"`, `"C2: 100uF"`) o `{type, name, value}`. Tipos: R, L, C, V, I y Z. |
| `source` | texto |  | La fuente, como un elemento: `"V1 = 12V"` o `{type: I, name, value}`. |
| `frequency` | número |  | La frecuencia, en Hz (para los inductores y capacitores). |
| `style` | texto | `ansi`, `iec` · por defecto `ansi` | `ansi` (resistencias en zigzag) o `iec` (rectángulos). |
| `results` | sí/no |  | Escribe la impedancia equivalente y la corriente (por omisión, sí). |
| `blank` | sí/no |  | Deja los resultados en blanco. |
| `blankValues` | sí/no |  | Con `blank`, oculta también los valores de los elementos. |

**Modo ejercicio:** Campos: `blank`, `blankValues`.

Ejemplo: Circuito serie-paralelo

```yaml
type: circuit
title: Circuito serie-paralelo
source: "V1 = 12V"
network:
  series:
    - "R1 = 100Ω"
    - parallel: ["R2 = 220Ω", "R3 = 330Ω"]
    - "R4 = 47Ω"
```

Ejemplo: RLC en alterna

```yaml
type: circuit
title: RLC en alterna
frequency: 50
source: "V = 220"
network:
  series:
    - "R = 10Ω"
    - "L = 100mH"
    - parallel: ["C = 100uF", "R2 = 50Ω"]
```

- Los valores aceptan prefijos del SI: `4.7k`, `10mH`, `100uF`, `2,2µF`, `1M`.

<a id="phasor"></a>

## `phasor`: Diagrama fasorial

*Matemática aplicada › Circuitos eléctricos.* Fasores en el plano complejo y su suma.

Dibuja los fasores como flechas en el plano complejo, sobre una grilla polar, y opcionalmente
su suma (desde el origen o encadenados punta con cola). Debajo, una tabla con cada fasor en
forma polar y rectangular.

**Calcula:** La suma, en forma polar y rectangular.

| Campo | Tipo | Valores · por defecto | Descripción |
| :--- | :--- | :--- | :--- |
| `phasors` | lista |  | Los fasores (hasta 12): `{label, magnitude, angle}` (el ángulo en grados) o `{label, re, im}`. |
| `sum` | sí/no |  | Dibuja la suma. |
| `tipToTail` | sí/no |  | Encadena los fasores punta con cola. |
| `sumLabel` | texto |  | El rótulo de la suma (por omisión Σ). |
| `unit` | texto |  | La unidad, para la tabla. |
| `table` | sí/no |  | La tabla de fasores (por omisión, sí). |
| `blank` | sí/no |  | Oculta la suma y sus valores. |

**Modo ejercicio:** Campos: `blank`.

Ejemplo: Suma de fasores

```yaml
type: phasor
title: Suma de fasores
phasors:
  - { label: V1, magnitude: 10, angle: 30 }
  - { label: V2, magnitude: 6, angle: -45 }
  - { label: V3, magnitude: 4, angle: 120 }
sum: true
unit: V
```

---

Generado por `docs/manual/generar.py`; no se edita a mano.
