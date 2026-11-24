[Manual](../../README.md) › [Química](../README.md) › [Química orgánica](README.md)

# Silla del ciclohexano (`chair`)

Dibuja la silla del ciclohexano con sus sustituyentes axiales y ecuatoriales y, con `flip`,
también la silla invertida. Dice si los sustituyentes son cis o trans y, con los valores A
conocidos, cuál silla es más estable y por cuánta energía.

## Qué calcula

- La relación cis/trans y la diferencia de energía entre las sillas por valores A (CH₃ 7,3 kJ/mol, tBu 21…).

## Campos

| Campo | Tipo | Valores y valor por defecto | Descripción |
| :--- | :--- | :--- | :--- |
| `type` | texto | `chair` | Obligatorio. |
| `substituents` | lista |  | Los sustituyentes: `position` (1 a 6), `group`, `axial` y `blank`. |
| `flip` | sí/no |  | Dibuja también la silla invertida (los axiales pasan a ecuatoriales). |
| `showRelation` | sí/no |  | Escribe si es cis o trans (por omisión, sí). |
| `showStability` | sí/no |  | Escribe cuál silla es más estable. |
| `blank` | sí/no |  | Deja los sustituyentes en blanco. |
| `blankRelation` | sí/no |  | Deja en blanco la relación cis/trans. |
| `blankStability` | sí/no |  | Deja en blanco la estabilidad. |

Además, los campos comunes a todos los tipos: `title`, `subtitle`, `id`, `inkSaver` y `theme` (ver [Campos comunes](../../campos-comunes.md)).

## Modo ejercicio

Campos que dejan casilleros para completar: `blank`, `blankRelation`, `blankStability`.

## Ejemplos

### Metilciclohexano con flip

```yaml
type: chair
title: Metilciclohexano con flip
substituents:
  - {position: 1, group: CH3, axial: true}
flip: true
```

![Metilciclohexano con flip](img/ks_chem_chair_metil.svg)

### Ciclohexano disustituido (blanco)

```yaml
type: chair
title: Ciclohexano disustituido (blanco)
substituents:
  - {position: 1, group: tBu, axial: false}
  - {position: 4, group: CH3, axial: false}
flip: true
blank: true
inkSaver: false
```

![Ciclohexano disustituido (blanco)](img/ks_chem_chair_terbutil_blank.svg)

## Ver también

[`newman`](newman.md), [`fischer`](fischer.md)
