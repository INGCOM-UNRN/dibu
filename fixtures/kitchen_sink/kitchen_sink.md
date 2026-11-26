# Catálogo Kitchen Sink de `dibu`

En este documento reunimos ejemplos válidos de los **72 tipos** de diagramas soportados por `dibu` (`9` de control de procesos, `21` de química, `16` de ciencias de la computación, `18` de ingeniería en computación y `8` de señales, probabilidad y circuitos). Vas a encontrar bloques simples y apilados, ambas paletas (`inkSaver: true` e `inkSaver: false`), variantes `blank*`, caracteres UTF-8 con subíndices/superíndices, escape XML (`&`, `<`, `>`) y todas las variantes de fences (` ```estructura `, ` ```statics `, `~~~estructura`, `~~~statics` y ```` ````estructura ````).

---

## 1. Control de Procesos

En esta sección cubrimos los `9` tipos de `dibu-control`, combinando diagramas de bloques, respuestas temporales, análisis frecuencial y automatización industrial.

### Lazo de control y respuesta temporal

```estructura
- type: block_diagram
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

~~~estructura
- type: block_diagram
  title: Control en cascada (para completar)
  forward:
    - "Gc1(s)"
    - loop:
        forward: ["Gc2(s)", "Gp2(s)"]
        feedback: ["H2(s)"]
    - "Gp1(s)"
  feedback: ["H1(s)"]
  blankSigns: true
  inkSaver: false
- type: time_response
  title: Segundo orden subamortiguado (ζ = 0,3)
  system: {gain: 1, zeta: 0.3, wn: 2}
  annotations: [final, overshoot, peak, rise, settling]
~~~

### Frecuencia, polos/ceros y lugar de las raíces

```estructura
- type: bode
  title: Bode de L(s) = 10 / (s(s+1)(s+10))
  system: {k: 10, poles: [0, -1, -10]}
  asymptotes: true
  margins: true
- type: nyquist
  title: Nyquist con círculo unitario
  system: {k: 5, poles: [-1, -2, -3]}
  unitCircle: true
  inkSaver: false
```

````estructura
- type: pole_zero
  title: Polos y ceros con ζ y ωn
  poles: [-1, [-2, 3]]
  zeros: [-4]
  dampingLines: [0.5]
  wnCircles: [3]
  showValues: true
  inkSaver: false
- type: root_locus
  title: Lugar de las raíces K / (s(s+1)(s+2))
  system: {k: 1, poles: [0, -1, -2]}
  marks: [1]
````

### Instrumentación P&ID, Ladder y GRAFCET

```estructura
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

~~~estructura
- type: ladder
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
    - logic:
        - {"no": "Q0.0"}
        - {ton: "T1", preset: "10 s"}
- type: grafcet
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
~~~

---

## 2. Química General y Fisicoquímica

Aquí repasamos los `13` tipos generales de `dibu-chem`: estructuras de Lewis, reacciones, TREPEV, orbitales, energía, equilibrio ácido-base, cinética, fases, espectros, electroquímica y sólidos cristalinos.

### Moléculas, reacciones, TREPEV y orbitales

```estructura
- type: molecule
  title: Ion nitrato (Lewis resonante)
  atoms:
    - {id: n, el: N, at: [0, 0], charge: 1}
    - {id: o1, el: O, from: n, angle: 90, bond: 2, lonePairs: 2}
    - {id: o2, el: O, from: n, angle: 210, lonePairs: 3, charge: -1}
    - {id: o3, el: O, from: n, angle: 330, lonePairs: 3, charge: -1}
- type: reaction
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

~~~estructura
- type: vsepr
  title: XeF4 (para completar)
  central: Xe
  ligands: [F, F, F, F]
  lonePairs: 2
  blank: [name, angle, hybridization]
  inkSaver: false
- type: orbitals
  kind: molecular
  title: Diagrama OM de N2
  atoms: [N, N]
  blankBondOrder: true
~~~

### Termoquímica, ácido-base y cinética

```estructura
- type: energy_profile
  title: Reacción exotérmica catalizada
  states:
    - {label: "A + B", energy: 0}
    - {label: "ET‡", energy: 85, ts: true}
    - {label: "C + D", energy: -40}
  catalyzed:
    - {energy: 0}
    - {energy: 45, ts: true}
    - {energy: -40}
- type: titration
  title: Ácido poliprótico H3PO4 con NaOH
  analyte: {kind: weak_acid, pKa: [2.15, 7.2, 12.35], conc: 0.1, volume: 20}
  titrant: {conc: 0.1}
  inkSaver: false
```

```estructura
- type: speciation
  title: Especiación del ácido fosfórico
  pKa: [2.15, 7.2, 12.35]
  species: [H3PO4, H2PO4^-, HPO4^2-, PO4^3-]
- type: kinetics
  title: Órdenes 0, 1 y 2
  curves:
    - {order: 0, k: 0.05, A0: 1}
    - {order: 1, k: 0.1, A0: 1}
    - {order: 2, k: 0.2, A0: 1}
  tEnd: 30
```

### Fases, calentamiento, espectros, pilas y redes cristalinas

```estructura
- type: phase_diagram
  kind: eutectic
  title: Diagrama eutéctico Pb–Sn
  components: [Pb, Sn]
  meltingA: 327
  meltingB: 232
  eutectic: [61.9, 183]
  cooling: 30
  inkSaver: false
- type: heating_curve
  title: 100 g de hielo de −20 °C a 120 °C
  substance: water
  mass: 100
  from: -20
  to: 120
```

````estructura
- type: spectrum
  kind: nmr
  title: ¹H-RMN del etanol
  tms: true
  signals:
    - {shift: 1.22, integration: 3, neighbors: 2, J: 7, label: "a"}
    - {shift: 3.69, integration: 2, neighbors: 3, J: 7, label: "b"}
    - {shift: 2.6, integration: 1, multiplicity: s, label: "c"}
  inkSaver: false
- type: galvanic_cell
  title: Pila de Daniell (Nernst + ΔG°)
  halfCells:
    - {couple: "Cu^2+/Cu", E0: 0.34, n: 2, conc: 1}
    - {couple: "Zn^2+/Zn", E0: -0.76, n: 2, conc: 0.01}
  showDeltaG: true
````

```estructura
type: unit_cell
title: Celda unitaria Cu (fcc)
lattice: fcc
element: Cu
a: 361.5
molarMass: 63.55
```

---

## 3. Química Orgánica

Los `3` tipos conformacionales y estereoquímicos: proyecciones de Newman, sillas de ciclohexano y proyecciones de Fischer.

```estructura
- type: newman
  title: Butano conformación anti (C2–C3)
  axis: C2–C3
  front: [CH3, H, H]
  back: [CH3, H, H]
  dihedral: 180
- type: chair
  title: Metilciclohexano con flip
  substituents:
    - {position: 1, group: CH3, axial: true}
  flip: true
- type: fischer
  title: D-glucosa (4 centros quirales)
  top: CHO
  bottom: CH2OH
  centers:
    - {left: H, right: OH, priorities: [right, top, bottom, left]}
    - {left: HO, right: H, priorities: [left, top, bottom, right]}
    - {left: H, right: OH, priorities: [right, top, bottom, left]}
    - {left: H, right: OH, priorities: [right, top, bottom, left]}
```

---

## 4. Química Inorgánica y Coordinación

Los `5` tipos de inorgánica: campo cristalino, complejos de coordinación, diagramas de Latimer, Frost y ciclo de Born-Haber.

```estructura
- type: crystal_field
  title: "[Co(NH3)6]^3+ (bajo espín)"
  metal: Co
  oxidation: 3
  ligand: NH3
  inkSaver: false
- type: complex
  title: "trans-[Co(NH3)4Cl2]^+"
  metal: Co
  charge: 1
  ligands: [NH3, NH3, Cl, Cl, NH3, NH3]
  inkSaver: false
```

~~~estructura
- type: latimer
  title: Diagrama Latimer del Mn (medio ácido)
  species: [MnO4^-, MnO4^2-, MnO2, Mn^3+, Mn^2+, Mn]
  oxidation: [7, 6, 4, 3, 2, 0]
  potentials: [0.56, 2.26, 0.95, 1.51, -1.18]
  compute: [[0, 2], [0, 4]]
  showDisproportionation: true
- type: frost
  title: Diagrama de Frost del Mn
  species: [MnO4^-, MnO4^2-, MnO2, Mn^3+, Mn^2+, Mn]
  oxidation: [7, 6, 4, 3, 2, 0]
  potentials: [0.56, 2.26, 0.95, 1.51, -1.18]
  showDisproportionation: true
  inkSaver: false
- type: born_haber
  title: Ciclo de Born-Haber del NaCl
  start: "Na(s) + ½ Cl2(g)"
  product: "NaCl(s)"
  formation: -411
  steps:
    - {to: "Na(g) + ½ Cl2(g)", dH: 107, label: "ΔH_sub"}
    - {to: "Na^+(g) + ½ Cl2(g)", dH: 496, label: "EI"}
    - {to: "Na^+(g) + Cl(g)", dH: 122, label: "½ D"}
    - {to: "Na^+(g) + Cl^-(g)", dH: -349, label: "AE"}
~~~

---

## 5. Ciencias de la Computación

Los `14` tipos de `dibu-cs` usando también fences `statics` (tanto con backticks como con tildes).

### Estructuras lineales y memoria

```statics
- type: array
  title: "Arreglo v[0..2] con <p & q> → α·β"
  values: [3, 1, 4]
  pointers: "p:0, q:2"
  showAddresses: true
  showOffsets: true
- type: matrix
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

~~~statics
- type: linked_list
  title: Lista circular doblemente enlazada
  variant: circular_doubly
  values: ["α", "β", "γ"]
  pointers: "actual:1"
  inkSaver: false
- type: stack
  title: Pila con traza vertical
  layout: vertical_multi
  values: [1]
  traceLabels: ["Inicio", "push(2)", "pop()"]
- type: queue
  title: Cola circular en arreglo de capacidad 8
  variant: circular
  capacity: 8
  frontIndex: 5
  rearIndex: 1
  inkSaver: false
~~~

### Recursión, árboles y diseño de tipos

```statics
- type: call_stack
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
- type: call_tree
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

```estructura
- type: tree
  title: Árbol AVL rebalanceado
  variant: avl
  insert: [10, 20, 30, 40, 50, 25]
- type: struct
  title: Layout LP64 con padding
  name: struct outer
  fields:
    - {name: c, type: char}
    - {name: pos, type: struct punto, size: 8, align: 4}
- type: adt
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

```estructura
- type: object_heap
  title: Dos variables comparten un arreglo
  frames:
    - function: main
      vars:
        - {name: a, type: "int[]", ref: A}
        - {name: b, type: "int[]", ref: A}
        - {name: n, type: int, value: 3}
  objects:
    - {id: A, type: "int[]", elements: [1, 2, 3]}
- type: matrix
  title: Arreglo de arreglos en Java
  variant: java
  name: matriz
  values: [[1, 2, 3], [4], null, []]
- type: flowchart
  title: Lazo do-while
  steps:
    - "i = 0"
    - doWhile: "i < 3"
      do: ["procesar(i)", "i = i + 1"]
```

~~~estructura
- type: array
  title: "Corte: a[::-1]"
  variant: python_sequence
  values: [2, 4, 7]
  showBoundaries: true
  slice: {step: -1}
- type: object_heap
  variant: python
  title: "b = a"
  frames:
    - function: "<módulo>"
      vars: [{name: a, ref: L}, {name: b, ref: L}]
  objects:
    - {id: L, type: list, elements: [1, 2]}
- type: flowchart
  title: continue
  steps:
    - for: "j en range(3)"
      do:
        - if: "j == 1"
          then: [{continue: true}]
        - io: "MOSTRAR j"
- type: trace_table
  variant: program
  columns: [x]
  rows:
    - {step: "Leer", input: 0, values: [0]}
    - {step: "Mostrar", output: ""}
~~~

```estructura
- type: object_heap
  variant: c
  title: "free(p) deja a q colgante"
  frames:
    - function: main
      vars: [{name: p, type: "int *", ref: B}, {name: q, type: "int *", ref: "B[1]"}]
  objects:
    - {id: B, type: "int[2]", ended: true, elements: [1, 2]}
- type: array
  variant: c_string
  title: "char s[6] = \"ab\""
  text: ab
  capacity: 6
- type: queue
  variant: ring
  title: Cola envuelta
  capacity: 4
  frontIndex: 3
  values: [5, 6]
- type: byte_layout
  title: Dos bytes en orden de red
  fields:
    - {name: largo, size: 2, value: "0x1234", endian: big}
```

### Algoritmos, complejidad y seguimiento

```statics
- type: complexity
  title: Órdenes de crecimiento Θ(n²)
  curves:
    - "Θ(n²)"
    - "n log n"
    - label: medido
      points: [[2, 5], [10, 90]]
- type: flowchart
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

```estructura
- type: cost_table
  title: Análisis línea a línea (evaluación)
  lines:
    - ["for (;;)", "c1", "n"]
    - ["    x++;", "c2", "n - 1"]
  total: 2n
  blankCosts: true
  blankTotal: true
- type: trace_table
  title: Seguimiento de variables
  columns: ["i", "suma"]
  rows: [[0, 7], [1, null]]
  rowCount: 4
```

---

## 6. Sistemas digitales

Tablas de verdad con columnas intermedias y formas canónicas, mapas de Karnaugh minimizados por Quine-McCluskey y circuitos con compuertas generados a partir de la expresión.

```estructura
- type: truth_table
  title: Función mayoría de tres entradas
  expr: "A·B + A·C + B·C"
  name: M
  canonical: true
```

```estructura
- type: karnaugh
  title: Mapa de Karnaugh de 4 variables
  vars: [A, B, C, D]
  minterms: [0, 1, 2, 5, 6, 7, 8, 9, 10, 14]
  implicants: true
```

~~~estructura
- type: logic_circuit
  title: Circuito de la función mayoría
  expr: "A·B + A·C + B·C"
  output: M
~~~

---

## 7. Arquitectura de computadoras

El pipeline de 5 etapas detecta riesgos RAW e inserta burbujas; `ieee754` muestra la codificación paso a paso y `cache` simula aciertos y fallos clasificados (forzosos, de capacidad y de conflicto).

```estructura
- type: pipeline
  title: Pipeline MIPS de 5 etapas con forwarding
  instructions:
    - "lw $t0, 0($s0)"
    - "add $t1, $t0, $t2"
    - "sub $t3, $t1, $t4"
    - "sw $t3, 4($s0)"
    - "and $t5, $t6, $t7"
```

```estructura
- type: ieee754
  title: "−6,75 en precisión simple"
  value: -6.75
```

~~~estructura
- type: cache
  title: Caché de correspondencia directa
  addressBits: 12
  cacheSize: 64
  blockSize: 8
  associativity: 1
  accesses: ["0x000", "0x004", "0x010", "0x040", "0x000", "0x048", "0x010", "0x080", "0x044"]
~~~

---

## 8. Sistemas operativos

Planificación de CPU con diagrama de Gantt y métricas, traducción de direcciones con tabla de páginas y algoritmos de reemplazo comparados sobre la misma cadena de referencias.

```estructura
- type: cpu_scheduling
  title: Planificación SRTF
  algorithm: srtf
  processes:
    - { name: P1, arrival: 0, burst: 8 }
    - { name: P2, arrival: 1, burst: 4 }
    - { name: P3, arrival: 2, burst: 9 }
    - { name: P4, arrival: 3, burst: 5 }
```

```estructura
- type: page_table
  title: Traducción de direcciones con paginación
  pageSize: 1024
  table: [5, 2, null, 7, 0]
  addresses: [1500, 3100, 4095, 2100, 6000]
```

~~~estructura
- type: page_replacement
  title: Algoritmos de reemplazo de páginas
  frames: 3
  references: [7, 0, 1, 2, 0, 3, 0, 4, 2, 3, 0, 3, 2, 1, 2, 0, 1, 7, 0, 1]
  algorithms: [fifo, lru, opt]
~~~

---

## 9. Redes

Subnetting con VLSM: `dibu` asigna los bloques en orden decreciente y calcula red, máscara, rango de hosts y broadcast.

```estructura
- type: subnetting
  title: Subnetting VLSM
  network: 192.168.10.0/24
  subnets:
    - { name: Ventas, hosts: 50 }
    - { name: Desarrollo, hosts: 100 }
    - { name: Admin, hosts: 20 }
    - { name: Enlace WAN, hosts: 2 }
```

```estructura
- type: subnetting
  title: "Ejercicio: 8 subredes iguales"
  network: 10.0.0.0/16
  count: 8
  blank: true
```

---

## 10. Bases de datos

Planificaciones de transacciones con sus conflictos, el grafo de precedencia y el veredicto de serializabilidad.

```estructura
- type: transaction_schedule
  title: Planificación no serializable
  schedule: [R1(A), R2(A), W1(A), R2(B), W2(A), W1(B), C1, C2]
```

```estructura
- type: transaction_schedule
  title: Planificación serializable por conflictos
  schedule: [R1(A), W1(A), R2(A), R1(B), W2(A), W1(B), R3(B), W3(B), C1, C2, C3]
```

---

## 11. Estructuras de datos y algoritmos

Estructuras construidas operación por operación (heap, hash, AVL, B+), tablas de programación dinámica con su camino de reconstrucción, Dijkstra y una máquina de Turing.

```estructura
- type: heap
  title: Min-heap construido por inserciones
  values: [15, 10, 20, 8, 25, 3]
  ops: ["extract", "insert 12"]
  steps: true
```

```estructura
- type: hash_table
  title: Hashing con sondeo lineal
  size: 11
  keys: [22, 1, 13, 11, 24, 33, 18, 42, 31]
  collision: linear
```

~~~estructura
- type: avl
  title: Árbol AVL con rotaciones
  keys: [10, 20, 30, 40, 50, 25]
  steps: rotations
~~~

```estructura
- type: bplus_tree
  title: Árbol B+ de orden 4
  order: 4
  keys: [5, 15, 25, 35, 45, 20, 30, 10, 40, 50, 55]
```

```estructura
- type: dp_table
  title: Subsecuencia común más larga
  problem: lcs
  a: ABCBDAB
  b: BDCABA
```

~~~estructura
- type: dijkstra
  title: Caminos mínimos con Dijkstra
  source: A
  edges: ["A-B:4", "A-C:2", "C-B:1", "B-D:5", "C-D:8", "C-E:10", "D-E:2"]
~~~

```estructura
- type: turing
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

---

## 12. Señales y sistemas

Señales continuas y discretas definidas por expresiones, convolución paso a paso, series de Fourier, DFT y muestreo con aliasing.

```estructura
- type: signal
  title: Operaciones sobre señales continuas
  signals:
    - { expr: "rect((t-1)/2)", label: "x(t)" }
    - { expr: "rect((2t-1)/2)", label: "x(2t)" }
  range: [-1, 4]
```

```estructura
- type: convolution
  title: Convolución discreta paso a paso
  x: [1, 2, 1]
  h: [1, 1, 1, 1]
  step: 2
```

~~~estructura
- type: fourier
  title: Serie de Fourier de una onda cuadrada
  wave: square
  harmonics: 9
~~~

```estructura
- type: sampling
  title: Muestreo con aliasing
  frequency: 9
  fs: 12
  periods: 3
```

---

## 13. Probabilidad

Distribuciones con el área sombreada y la probabilidad exacta, y diagramas de Venn con operaciones de conjuntos.

```estructura
- type: distribution
  title: Distribución normal estándar
  dist: normal
  mean: 0
  sd: 1
  interval: [-1.96, 1.96]
```

```estructura
- type: venn
  title: Operaciones con conjuntos
  sets: [A, B, C]
  shade: ["A ∩ B", "(A ∪ B) − C", "A Δ B", "C'"]
```

---

## 14. Circuitos eléctricos

Redes serie-paralelo con su impedancia equivalente y diagramas fasoriales.

```estructura
- type: circuit
  title: Circuito serie-paralelo
  source: "V1 = 12V"
  network:
    series:
      - "R1 = 100Ω"
      - parallel: ["R2 = 220Ω", "R3 = 330Ω"]
      - "R4 = 47Ω"
```

```estructura
- type: circuit
  title: RLC en alterna
  frequency: 50
  source: "V = 220"
  network:
    series:
      - "R = 10Ω"
      - "L = 100mH"
      - parallel: ["C = 100uF", "R2 = 50Ω"]
```

~~~estructura
- type: phasor
  title: Suma de fasores
  phasors:
    - { label: V1, magnitude: 10, angle: 30 }
    - { label: V2, magnitude: 6, angle: -45 }
    - { label: V3, magnitude: 4, angle: 120 }
  sum: true
  unit: V
~~~

---

## 15. Integración Cross-Dominio (Control + Química + CS apilados)

En este último bloque apilamos los tres dominios dentro de un mismo fence para verificar la composición vertical con `apilar_svgs` y las transiciones de paleta:

````estructura
- type: block_diagram
  title: Planta química controlada (Control)
  forward: ["PI(s)", "Reactor"]
  feedback: ["Sensor"]
  inkSaver: false
- type: reaction
  title: Síntesis acoplada (Química)
  steps: ["A", "+", "B", {arrow: "->", above: "cat."}, "C"]
  inkSaver: true
- type: array
  title: Buffer de telemetría (CS)
  values: [10, 20, 30, 40]
  pointers: "head:0, tail:3"
  showAddresses: true
  inkSaver: false
````
