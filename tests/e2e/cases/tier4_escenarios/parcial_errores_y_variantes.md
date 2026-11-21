# Parcial Docente — Variantes `blank*`, `inkSaver` y UTF-8

Este escenario combina diagramas de Control y CS con variantes docentes y overrides de paleta.

## 1. Bloque en blanco y `inkSaver: true` con caracteres UTF-8

```estructura
type: block_diagram
title: "Lazo α/β — ñ & <regulación>"
inkSaver: true
forward: [{blank: true}, {label: "Gp(s)", out: "Y(s)"}]
feedback: [{blank: true}]
blankSigns: true
```

## 2. Arreglo y tabla de costos en blanco (`inkSaver: false`)

```statics
- type: array
  title: "Arreglo ñ — ≤ n"
  inkSaver: false
  values: [11, 22, 33]
  showAddresses: true
  blankValues: true
  blankAddrs: true
- type: cost_table
  inkSaver: false
  lines:
    - ["for (int i = 0; i < n; i++)", "c1", "n + 1"]
    - ["    suma += v[i];", "c2", "n"]
  total: "c1(n + 1) + c2·n"
  blankCosts: true
  blankTotal: true
```

## 3. Árbol AVL y tabla de seguimiento con `blank*`

```estructura
type: tree
title: "AVL — rotación derecha"
variant: avl
insert: [30, 20, 10]
blankBalance: true
blankValues: true
```

~~~statics
type: trace_table
title: "Traza α ≤ β"
columns: ["i", "acum"]
rows: [[0, 0], [1, 10], [2, 30]]
blankValues: true
~~~
