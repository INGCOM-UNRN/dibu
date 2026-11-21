# Guía Integradora Multidisciplinaria (44 Tipos de Diagramas)

Esta guía recorre los 3 dominios del repositorio (`control`, `chem` y `cs`) con bloques individuales y apilados.

## Sección I — Control Automático (9 tipos)

```estructura
- type: block_diagram
  forward: ["G1(s)", "G2(s)"]
  feedback: ["H(s)"]
- type: time_response
  system: {gain: 1, zeta: 0.4, wn: 2}
  annotations: [final, overshoot]
```

```estructura
type: bode
system: {k: 10, poles: [0, -1, -10]}
margins: true
```

```estructura
type: nyquist
system: {k: 5, poles: [-1, -2, -3]}
unitCircle: true
```

```statics
type: pole_zero
poles: [-1, [-2, 3]]
zeros: [-4]
```

```estructura
type: root_locus
system: {k: 1, poles: [0, -1, -2]}
marks: [1]
```

```estructura
type: instrumentation
equipment: [{id: TK, kind: tank, at: [2, 2], label: "TK-1"}]
instruments: [{id: LT, tag: "LT 1", at: [5, 1]}]
lines: [{from: TK, to: LT, kind: capillary}]
```

```estructura
type: ladder
rungs:
  - logic: [{no: "I0.0"}, {coil: "Q0.0"}]
```

```estructura
type: grafcet
steps:
  - {step: 0, initial: true}
  - {transition: "Marcha"}
  - {step: 1, actions: ["Motor"]}
```

## Sección II — Química (21 tipos)

```estructura
- type: molecule
  atoms:
    - {id: o, el: O, at: [0, 0], lonePairs: 2}
    - {id: h1, el: H, at: [-1, -0.7]}
    - {id: h2, el: H, at: [1, -0.7]}
  bonds:
    - {from: o, to: h1}
    - {from: o, to: h2}
- type: reaction
  steps: ["A", {"arrow": "->"}, "B"]
```

```estructura
type: vsepr
central: N
ligands: ["H", "H", "H"]
lonePairs: 1
```

```estructura
type: orbitals
element: Fe
core: true
```

```estructura
type: energy_profile
states:
  - {label: R, energy: 0}
  - {label: ET, energy: 50, ts: true}
  - {label: P, energy: -20}
```

```estructura
type: titration
analyte: {kind: weak_acid, pKa: 4.76, conc: 0.1, volume: 25}
titrant: {conc: 0.1}
```

```estructura
type: speciation
pKa: 4.76
```

```estructura
type: kinetics
order: 1
k: 0.1
A0: 1
```

```estructura
type: phase_diagram
substance: water
points: [{T: 300, P: 0.01, label: "P"}]
```

```estructura
type: heating_curve
substance: water
mass: 100
from: -20
to: 120
```

```estructura
type: spectrum
kind: ir
peaks: [{at: 1710, depth: 0.85, label: "C=O"}]
```

```estructura
type: galvanic_cell
halfCells:
  - {couple: "Cu^2+/Cu", E0: 0.34}
  - {couple: "Zn^2+/Zn", E0: -0.76}
```

```estructura
type: unit_cell
lattice: fcc
element: Cu
a: 361.5
molarMass: 63.55
```

```estructura
type: newman
axis: "C2–C3"
front: ["CH3", "H", "H"]
back: ["CH3", "H", "H"]
```

```estructura
type: chair
substituents: [{position: 1, group: CH3, axial: true}]
flip: true
```

```estructura
type: fischer
top: CHO
bottom: CH2OH
centers: [{left: H, right: OH}]
```

```estructura
type: crystal_field
metal: Co
oxidation: 3
ligand: NH3
```

```estructura
type: complex
metal: Co
charge: 1
ligands: [NH3, NH3, Cl, Cl, NH3, NH3]
```

```estructura
type: latimer
species: ["Cu^2+", "Cu^+", "Cu"]
oxidation: [2, 1, 0]
potentials: [0.16, 0.52]
```

```estructura
type: frost
species: ["Cu^2+", "Cu^+", "Cu"]
oxidation: [2, 1, 0]
potentials: [0.16, 0.52]
showDisproportionation: true
```

```estructura
type: born_haber
formation: -300
steps:
  - {to: "M^+(g) + X^-(g)", dH: 400}
```

## Sección III — Ciencias de la Computación (14 tipos)

```estructura
- type: array
  values: [3, 1, 4]
- type: matrix
  values: [[1, 2], [3, 4]]
```

```estructura
type: linked_list
variant: singly
values: [10, 20, 30]
```

```estructura
type: stack
values: [5, 9]
capacity: 4
```

```estructura
type: call_stack
frames:
  - function: main
    vars: [{name: x, value: 1}]
```

```estructura
type: call_tree
root:
  call: "f(2)"
  returns: 1
  children:
    - {call: "f(1)", returns: 1}
```

```estructura
type: queue
variant: linear
capacity: 4
frontIndex: 0
rearIndex: 1
values: ["a", "b"]
```

```estructura
type: tree
variant: bst
insert: [50, 30, 70]
```

```estructura
type: struct
name: "struct s"
fields: ["char a", "int b"]
```

```estructura
type: adt
name: Pila
operations: ["crear()", "apilar(x)"]
```

```estructura
type: complexity
```

```estructura
type: flowchart
steps: ["x = 0", {io: "mostrar x"}]
```

```estructura
type: cost_table
lines: [["x = 1;", "c1", 1]]
total: "c1"
```

```estructura
type: trace_table
columns: ["i", "s"]
rows: [[0, 0], [1, 1]]
```
