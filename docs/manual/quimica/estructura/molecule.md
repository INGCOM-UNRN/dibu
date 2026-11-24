[Manual](../../README.md) › [Química](../README.md) › [Estructura y enlace](README.md)

# Molécula (`molecule`)

Dibuja una molécula a partir de sus átomos y enlaces: en fórmula de Lewis (con pares libres,
cargas y radicales) o esqueletal (sin carbonos ni hidrógenos explícitos). Los átomos se ubican
por coordenadas o encadenados por ángulo desde otro; los anillos se arman solos y dos que
comparten vértices se fusionan. Las flechas curvas muestran un mecanismo.

## Campos

| Campo | Tipo | Valores y valor por defecto | Descripción |
| :--- | :--- | :--- | :--- |
| `type` | texto | `molecule` | Obligatorio. |
| `atoms` | lista |  | Los átomos: `id`, `el` (el símbolo, también grupos como `OH`), `at: [x, y]` en largos de enlace, o `from` y `angle` (con `length` y `bond`, el orden del enlace con ese átomo); `charge`, `lonePairs`, `radical`, `note` (como δ+), `show`, `blank`. |
| `rings` | lista |  | Anillos: `size`, `center`, `start`, `ids` (los ids de sus vértices), `aromatic` (con círculo) o `kekule` (dobles alternados). |
| `bonds` | lista |  | Enlaces extra: `[a, b, orden]` o `{from, to, order, stereo, dashed, blank}`. El orden es 1, 2, 3, 1.5 o 0.5; `stereo` es `wedge` (cuña) o `hash` (rayas). |
| `arrows` | lista |  | Flechas curvas de mecanismo: `from` y `to` (un átomo o un enlace `[a, b]`), `bend` (la curvatura) y `half` (un solo electrón). |
| `skeletal` | sí/no |  | Fórmula esqueletal. |
| `bondLength` | número |  | El largo de un enlace, en píxeles. |
| `caption` | texto |  | Un rótulo debajo de la molécula. |

Además, los campos comunes a todos los tipos: `title`, `subtitle`, `id`, `inkSaver` y `theme` (ver [Campos comunes](../../campos-comunes.md)).

## Ejemplos

### Ion nitrato (Lewis resonante)

```yaml
type: molecule
title: Ion nitrato (Lewis resonante)
atoms:
  - {id: n, el: N, at: [0, 0], charge: 1}
  - {id: o1, el: O, from: n, angle: 90, bond: 2, lonePairs: 2}
  - {id: o2, el: O, from: n, angle: 210, lonePairs: 3, charge: -1}
  - {id: o3, el: O, from: n, angle: 330, lonePairs: 3, charge: -1}
```

![Ion nitrato (Lewis resonante)](img/ks_chem_molecule_nitrato.svg)

### Naftaleno (anillos fusionados)

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

![Naftaleno (anillos fusionados)](img/ks_chem_molecule_naftaleno_skeletal.svg)

### Mecanismo S_N\2 y carbono quiral

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

![Mecanismo S_N\2 y carbono quiral](img/ks_chem_molecule_sn2_estereo.svg)

## Ver también

[`vsepr`](vsepr.md), [`orbitals`](orbitals.md), [`unit_cell`](unit_cell.md)
