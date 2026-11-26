[Manual](../../README.md) › [Química](../README.md) › [Reacciones y cinética](README.md)

# Reacción (`reaction`)

Dibuja una ecuación química o un mecanismo: fórmulas, signos más, moléculas dibujadas (como
las de [`molecule`](../estructura/molecule.md)) y flechas de reacción, equilibrio, resonancia o
retrosíntesis con texto arriba y abajo. Un casillero en blanco deja un producto para completar.

## Campos

| Campo | Tipo | Valores y valor por defecto | Descripción |
| :--- | :--- | :--- | :--- |
| `type` | texto | `reaction` | Obligatorio. |
| `steps` | lista |  | Los elementos en orden: fórmulas, `"+"`, moléculas (bloques de `molecule`), `{arrow: "->", "<=>", "<->" (resonancia) o "=>" (retrosíntesis), above, below, blank}` y `{blank: true, width}` (un producto para completar). |
| `maxWidth` | número |  | El ancho máximo; más largo, la ecuación sigue en otro renglón. |

Además, los campos comunes a todos los tipos: `title`, `subtitle`, `id`, `inkSaver` y `theme` (ver [Campos comunes](../../campos-comunes.md)).

## Ejemplos

### Esterificación de Fischer

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

[![Esterificación de Fischer](img/ks_chem_reaction_esterificacion.svg)](https://ingcom-unrn.github.io/dibu-web/#z=K6ksSLVSKEpNTC7JzM_jKsksyQHyXYtLUosy0zKTE5MzD2_OU0hJVXDLLE7OSC3iAsoUFFtxKSjoKig5exg7-_t7KEF42koIUQ8jmHB1YlFRfrmVgpKNrZ2SjkJiUn4Z0AIlD6NgfxMgPyk1Byx7bopSLbKhQBOADDSTgaYqcQEA)

[Abrir en el editor](https://ingcom-unrn.github.io/dibu-web/#z=K6ksSLVSKEpNTC7JzM_jKsksyQHyXYtLUosy0zKTE5MzD2_OU0hJVXDLLE7OSC3iAsoUFFtxKSjoKig5exg7-_t7KEF42koIUQ8jmHB1YlFRfrmVgpKNrZ2SjkJiUn4Z0AIlD6NgfxMgPyk1Byx7bopSLbKhQBOADDSTgaYqcQEA) para explorarlo y modificarlo.

### Resonancia carboxilato y completar

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

[![Resonancia carboxilato y completar](img/ks_chem_reaction_resonancia_blank.svg)](https://ingcom-unrn.github.io/dibu-web/#z=dY_BTsMwDIbveYpfO6dSukkIRYgLDzA0joiDm2VbtDSunMCYpr47acdlQ_hk-__82y7nwVuIJ1cCJ1VCibXe-MyJkgsER9Lxd4hUGGc47ofoC4nKxQ_ZKqBBlfo5naLBJWwtnIaPFi-6qhbvRsN8jLeIXJGNxk64n0co7af17aO5Y7m9wuu_8EP17jhVaKkROflXCpJrdW-x_NdiZczN6ErDHUj2VWracf7xQiJ8slg8Nc-L31YXKR0tinx6jVPYlsN0zahCOr7RlxeLHcXs1Q8)

[Abrir en el editor](https://ingcom-unrn.github.io/dibu-web/#z=dY_BTsMwDIbveYpfO6dSukkIRYgLDzA0joiDm2VbtDSunMCYpr47acdlQ_hk-__82y7nwVuIJ1cCJ1VCibXe-MyJkgsER9Lxd4hUGGc47ofoC4nKxQ_ZKqBBlfo5naLBJWwtnIaPFi-6qhbvRsN8jLeIXJGNxk64n0co7af17aO5Y7m9wuu_8EP17jhVaKkROflXCpJrdW-x_NdiZczN6ErDHUj2VWracf7xQiJ8slg8Nc-L31YXKR0tinx6jVPYlsN0zahCOr7RlxeLHcXs1Q8) para explorarlo y modificarlo.

## Ver también

[`energy_profile`](energy_profile.md), [`kinetics`](kinetics.md)
