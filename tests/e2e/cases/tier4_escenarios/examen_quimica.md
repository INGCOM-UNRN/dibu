# Examen Integrador — Química General, Orgánica e Inorgánica

**Consigna:** Respondé cada apartado utilizando las representaciones moleculares, termodinámicas y espectroscópicas provistas.

## 1. Estructura molecular, geometría y orbitales (Química General)

```estructura
- type: molecule
  title: Molécula de agua
  atoms:
    - {id: o, el: O, at: [0, 0], lonePairs: 2}
    - {id: h1, el: H, at: [-1, -0.7]}
    - {id: h2, el: H, at: [1, -0.7]}
  bonds:
    - {from: o, to: h1}
    - {from: o, to: h2}
- type: vsepr
  title: Geometría del amoníaco
  central: N
  ligands: ["H", "H", "H"]
  lonePairs: 1
```

```statics
type: orbitals
title: Configuración electrónica del hierro
element: Fe
core: true
showUnpaired: true
```

## 2. Cinética, equilibrio ácido-base y electroquímica

```estructura
type: energy_profile
title: Perfil de reacción catalizada
states:
  - {label: "A + B", energy: 0}
  - {label: "ET‡", energy: 85, ts: true}
  - {label: "C + D", energy: -40}
catalyzed:
  - {energy: 0}
  - {energy: 45, ts: true}
  - {energy: -40}
```

```estructura
type: titration
title: Titulación CH3COOH 0,1 M con NaOH 0,1 M
analyte: {kind: weak_acid, pKa: 4.76, conc: 0.1, volume: 25}
titrant: {conc: 0.1, name: NaOH}
indicator: {name: fenolftaleína, range: [8.2, 10]}
annotations: [equivalence, half, initial]
```

```statics
type: speciation
title: Fracciones α del sistema fosfato
pKa: [2.15, 7.2, 12.35]
species: ["H3PO4", "H2PO4^-", "HPO4^2-", "PO4^3-"]
```

```estructura
type: kinetics
title: Cinética de primer orden
order: 1
k: 0.1
A0: 1
annotations: [halfLife]
showProduct: true
```

```estructura
type: galvanic_cell
title: Pila de Daniell
halfCells:
  - {couple: "Cu^2+/Cu", E0: 0.34, n: 2, conc: 1}
  - {couple: "Zn^2+/Zn", E0: -0.76, n: 2, conc: 0.01}
showDeltaG: true
```

## 3. Estereoquímica y reacciones (Química Orgánica)

```estructura
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

```statics
type: newman
title: Butano (C2–C3)
axis: "C2–C3"
front: ["CH3", "H", "H"]
back: ["CH3", "H", "H"]
dihedral: 180
```

```estructura
type: chair
title: Inversión de silla del metilciclohexano
substituents: [{position: 1, group: CH3, axial: true}]
flip: true
showStability: true
```

```estructura
type: fischer
title: D-gliceraldehído
top: CHO
bottom: CH2OH
centers: [{left: H, right: OH}]
showRS: true
```

## 4. Química de coordinación y termoquímica (Química Inorgánica)

```estructura
type: crystal_field
title: Campo cristalino [Co(NH3)6]3+
metal: Co
oxidation: 3
ligand: NH3
```

```statics
type: complex
title: Isómero trans-[Co(NH3)4Cl2]+
metal: Co
charge: 1
ligands: [NH3, NH3, Cl, Cl, NH3, NH3]
```

```estructura
type: latimer
title: Manganeso en medio ácido
species: ["MnO4^-", "MnO4^2-", "MnO2", "Mn^3+", "Mn^2+", "Mn"]
oxidation: [7, 6, 4, 3, 2, 0]
potentials: [0.56, 2.26, 0.95, 1.51, -1.18]
compute: [[0, 2]]
showDisproportionation: true
```

```estructura
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
