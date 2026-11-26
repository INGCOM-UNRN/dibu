[Manual](../README.md)

# Estructuras de datos y algoritmos

Diagramas de memoria y de estructuras de datos de un curso de programación en C, y de los
algoritmos que se estudian sobre ellas: arreglos, matrices, listas, pilas y colas; la pila
de llamadas y la recursión; árboles, heaps, tablas de hash; complejidad, trazas y
programación dinámica. Las direcciones, los offsets y los resultados los calcula dibu.

23 tipos, en 5 grupos. Crates: `dibu-cs` y `dibu-comp`.

## [Estructuras lineales](lineales/README.md)

Arreglos, matrices, listas enlazadas, pilas y colas, con sus direcciones de memoria.

- [`array`](lineales/array.md): Arreglo. Arreglos en memoria, con índices, direcciones, offsets, punteros que los recorren y su límite excluido, cadenas de C y secuencias de Python con sus cortes.
- [`matrix`](lineales/matrix.md): Matriz. Matrices estáticas, dinámicas, dentadas, planas, con punteros a filas de un solo bloque y de Java, como están en memoria.
- [`linked_list`](lineales/linked_list.md): Lista enlazada. Listas simples, dobles y circulares, con direcciones, cursores y estructura cabecera.
- [`stack`](lineales/stack.md): Pila. Pilas en arreglo, con el tope, y su evolución paso a paso.
- [`queue`](lineales/queue.md): Cola. Colas lineales y circulares, con frente y final o con frente y cantidad, y deques circulares.

## [Memoria, tipos y recursión](memoria/README.md)

La pila de llamadas, el árbol de llamadas recursivas, la disposición de un struct y de un búfer de octetos, el stack y el heap con objetos y referencias, y los tipos abstractos de datos.

- [`call_stack`](memoria/call_stack.md): Pila de llamadas. Los marcos de activación de una ejecución, con parámetros, variables y punteros entre marcos.
- [`call_tree`](memoria/call_tree.md): Árbol de llamadas. Las llamadas de una función recursiva, su orden, sus resultados y el costo por nivel.
- [`struct`](memoria/struct.md): Struct. La disposición de un struct en memoria (offsets, relleno, sizeof), las uniones con su discriminante y los tipos opacos.
- [`byte_layout`](memoria/byte_layout.md): Disposición de octetos. Los campos de un formato binario con su ancho, offset y orden de bytes, y las filas de píxeles de un BMP con su stride y su relleno.
- [`object_heap`](memoria/object_heap.md): Stack y heap (objetos). Variables, objetos y referencias en el stack y el heap, como en Java, Python o C, también en varios instantes.
- [`adt`](memoria/adt.md): Tipo abstracto de datos. Un TAD con sus operaciones, precondiciones, invariante y representaciones posibles.

## [Árboles y tablas](arboles/README.md)

Árboles binarios y de búsqueda, montículos, AVL con sus rotaciones, árboles B+ y tablas de hash.

- [`tree`](arboles/tree.md): Árbol binario. Árboles binarios, de búsqueda, AVL y montículos, por niveles, por inserciones o anidados.
- [`heap`](arboles/heap.md): Montículo (heap). Min-heaps y max-heaps construidos por inserciones o con heapify, con sus operaciones.
- [`avl`](arboles/avl.md): Árbol AVL. Inserciones y eliminaciones en un AVL, con las rotaciones que hace cada una.
- [`bplus_tree`](arboles/bplus_tree.md): Árbol B+. Inserciones en un árbol B+, con las divisiones de hojas y nodos internos.
- [`hash_table`](arboles/hash_table.md): Tabla de hash. Inserciones en una tabla de hash, con encadenamiento o direccionamiento abierto.

## [Análisis de algoritmos](algoritmos/README.md)

Complejidad, diagramas de flujo, costos línea por línea, pruebas de escritorio, programación dinámica y caminos mínimos.

- [`complexity`](algoritmos/complexity.md): Complejidad. Curvas de crecimiento para comparar órdenes, y jerarquías de clases de complejidad.
- [`flowchart`](algoritmos/flowchart.md): Diagrama de flujo. Diagramas de flujo armados a partir de la estructura del algoritmo.
- [`cost_table`](algoritmos/cost_table.md): Tabla de costos. El análisis de un algoritmo línea por línea, con el costo y las veces de cada una.
- [`trace_table`](algoritmos/trace_table.md): Tabla de traza. Pruebas de escritorio de variables y trazas de operaciones sobre una estructura.
- [`dp_table`](algoritmos/dp_table.md): Tabla de programación dinámica. LCS, distancia de edición, mochila 0/1 y cambio de monedas, con el camino de reconstrucción.
- [`dijkstra`](algoritmos/dijkstra.md): Dijkstra. La tabla de iteraciones del algoritmo de Dijkstra y los caminos mínimos.

## [Computabilidad](computabilidad/README.md)

Máquinas de Turing con la traza de sus configuraciones.

- [`turing`](computabilidad/turing.md): Máquina de Turing. La traza de configuraciones de una máquina de Turing y su tabla de transiciones.
