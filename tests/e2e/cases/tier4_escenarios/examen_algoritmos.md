# Examen Final — Algoritmos y Estructuras de Datos

**Duración:** 2 horas. Respondé en forma clara usando las estructuras y tablas provistas.

## 1. Estructuras lineales y memoria dinámica

```estructura
- type: array
  title: Arreglo en memoria
  values: [10, 20, 30]
  baseAddr: "0x2000"
  typeSize: 4
  showAddresses: true
  showOffsets: true
- type: matrix
  title: Matriz estática 2×3
  values: [[1, 2, 3], [4, 5, 6]]
  baseAddr: "0x1000"
  typeSize: 4
  showAddresses: true
  showLinear: true
```

```statics
type: linked_list
title: Lista simplemente enlazada
variant: singly
values: [10, 20, 30]
showAddresses: true
```

```estructura
type: stack
title: Pila acotada
values: [5, 9]
capacity: 4
showAddresses: true
```

```estructura
type: queue
title: Cola circular
variant: circular
capacity: 6
frontIndex: 1
rearIndex: 3
values: ["", "a", "b", "c"]
```

## 2. Recursión, árboles y registros LP64

```estructura
type: call_stack
title: Pila de llamadas
showAddresses: true
showTypes: true
frames:
  - function: main
    vars:
      - {name: a, value: 3, type: int}
  - function: intercambiar
    returnTo: main
    params:
      - {name: p, value: "&a", type: "int *"}
```

```statics
type: call_tree
title: Árbol de llamadas fib(3)
showOrder: true
highlight: ["fib(3)"]
root:
  call: "fib(3)"
  returns: 2
  children:
    - {call: "fib(2)", returns: 1}
    - {call: "fib(1)", returns: 1}
```

```estructura
type: tree
title: Árbol binario de búsqueda
variant: bst
insert: [50, 30, 70, 20, 40]
highlight: [30, 70]
```

```estructura
type: struct
title: Disposición de struct en LP64
name: "struct s"
typedef: "s_t"
fields: ["char a = 'A'", "int b = 30", "char c = 'Z'"]
```

```estructura
type: adt
title: TAD Pila
name: Pila
invariant: "tope >= 0"
operations:
  - {signature: "pila_t crear(void)", kind: constructor, post: "pila vacía"}
  - {signature: "bool apilar(pila_t p, int x)", kind: mutador, pre: "p != NULL"}
```

## 3. Análisis de algoritmos y trazas

```estructura
type: complexity
title: Órdenes de crecimiento
```

```statics
type: flowchart
title: Flujo iterativo
steps:
  - "i = 0"
  - {io: "leer n"}
  - while: "i < n"
    do: ["i = i + 1"]
  - {io: "mostrar i"}
```

```estructura
type: cost_table
title: Conteo de operaciones
lines:
  - ["int s = 0;", "c1", 1]
  - ["for (int i = 0; i < n; i++)", "c2", "n + 1"]
  - ["    s += v[i];", "c3", "n"]
  - ["return s;", "c4", 1]
total: "c1 + c4 + c2(n + 1) + c3·n"
```

```estructura
type: trace_table
title: Tabla de seguimiento
columns: ["i", "suma"]
rows: [[0, 0], [1, 5], [2, 15]]
highlight: [[2, 1]]
```
