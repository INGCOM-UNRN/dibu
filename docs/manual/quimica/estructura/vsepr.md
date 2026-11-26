[Manual](../../README.md) › [Química](../README.md) › [Estructura y enlace](README.md)

# Geometría molecular (TRePEV) (`vsepr`)

Dibuja la geometría molecular según la teoría de repulsión de pares electrónicos de valencia,
con el átomo central, los ligandos y los pares libres. La geometría sale de la cantidad de
pares, o se fija con `geometry`. Debajo, la notación AXE, el nombre de la geometría, el ángulo
y la hibridación.

## Qué calcula

- La geometría, la notación AXE, el ángulo de enlace y la hibridación a partir de los pares.

## Campos

| Campo | Tipo | Valores y valor por defecto | Descripción |
| :--- | :--- | :--- | :--- |
| `type` | texto | `vsepr` | Obligatorio. |
| `central` | texto |  | El átomo central. |
| `ligands` | lista |  | Los ligandos. |
| `lonePairs` | número |  | Los pares libres sobre el átomo central. |
| `geometry` | texto |  | La geometría, si no sale de los pares: `linear`, `trigonal_planar`, `bent_trigonal`, `tetrahedral`, `trigonal_pyramidal`, `bent`, `trigonal_bipyramidal`, `seesaw`, `t_shaped`, `linear_tbp`, `octahedral`, `square_pyramidal`, `square_planar`. |
| `showAngle` | sí/no |  | Escribe el ángulo. |
| `showHybridization` | sí/no |  | Escribe la hibridación. |
| `blank` | lista |  | `true` o una lista de `axe`, `name`, `angle` y `hybridization` para dejar en blanco. |

Además, los campos comunes a todos los tipos: `title`, `subtitle`, `id`, `inkSaver` y `theme` (ver [Campos comunes](../../campos-comunes.md)).

## Modo ejercicio

Campos que dejan casilleros para completar: `blank`.

## Ejemplos

### Geometría de SF4 (balancín)

```yaml
type: vsepr
title: Geometría de SF4 (balancín)
central: S
ligands: [F, F, F, F]
lonePairs: 1
```

[![Geometría de SF4 (balancín)](img/ks_chem_vsepr_sf4.svg)](https://ingcom-unrn.github.io/dibu-web/#z=K6ksSLVSKCtOLSjiKsksyQFy3FPzc1NLig6vTVRISVUIdjNR0EhKzEnMSz68Nk-TKzk1r6QoMcdKIZgrJzM9MS-l2Eoh2k1HAYpiuXLy81IDEjOLgOKGXAA)

[Abrir en el editor](https://ingcom-unrn.github.io/dibu-web/#z=K6ksSLVSKCtOLSjiKsksyQFy3FPzc1NLig6vTVRISVUIdjNR0EhKzEnMSz68Nk-TKzk1r6QoMcdKIZgrJzM9MS-l2Eoh2k1HAYpiuXLy81IDEjOLgOKGXAA) para explorarlo y modificarlo.

### XeF4 (para completar)

```yaml
type: vsepr
title: XeF4 (para completar)
central: Xe
ligands: [F, F, F, F]
lonePairs: 2
blank: [name, angle, hybridization]
inkSaver: false
```

[![XeF4 (para completar)](img/ks_chem_vsepr_xef4_blank.svg)](https://ingcom-unrn.github.io/dibu-web/#z=LYrBCsJADETv-YocFXoST_sBPQteBOkhbWMNTbNLNhTq17uCMDC8eRNH4YR75eIQEtrgwf0VT4WccMpbUQ7yM0xs4aQ_DSoL2VwTPvsO_xlAs_GNxNt-gVHJ1nYw2rhDskVbvY_RZZYPhWQbQGy9086e8EVaGb4)

[Abrir en el editor](https://ingcom-unrn.github.io/dibu-web/#z=LYrBCsJADETv-YocFXoST_sBPQteBOkhbWMNTbNLNhTq17uCMDC8eRNH4YR75eIQEtrgwf0VT4WccMpbUQ7yM0xs4aQ_DSoL2VwTPvsO_xlAs_GNxNt-gVHJ1nYw2rhDskVbvY_RZZYPhWQbQGy9086e8EVaGb4) para explorarlo y modificarlo.

## Ver también

[`molecule`](molecule.md), [`orbitals`](orbitals.md), [`unit_cell`](unit_cell.md)
