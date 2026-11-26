[Manual](../../README.md) › [Estructuras de datos y algoritmos](../README.md)

# Memoria, tipos y recursión

La pila de llamadas, el árbol de llamadas recursivas, la disposición de un struct y de un búfer de octetos, el stack y el heap con objetos y referencias, y los tipos abstractos de datos.

| Tipo | Diagrama | Para qué |
| :--- | :--- | :--- |
| [`call_stack`](call_stack.md) | Pila de llamadas | Los marcos de activación de una ejecución, con parámetros, variables y punteros entre marcos. |
| [`call_tree`](call_tree.md) | Árbol de llamadas | Las llamadas de una función recursiva, su orden, sus resultados y el costo por nivel. |
| [`struct`](struct.md) | Struct | La disposición de un struct en memoria (offsets, relleno, sizeof), las uniones con su discriminante y los tipos opacos. |
| [`byte_layout`](byte_layout.md) | Disposición de octetos | Los campos de un formato binario con su ancho, offset y orden de bytes, y las filas de píxeles de un BMP con su stride y su relleno. |
| [`object_heap`](object_heap.md) | Stack y heap (objetos) | Variables, objetos y referencias en el stack y el heap, como en Java, Python o C, también en varios instantes. |
| [`adt`](adt.md) | Tipo abstracto de datos | Un TAD con sus operaciones, precondiciones, invariante y representaciones posibles. |
