[Manual](../../README.md) › [Química](../README.md) › [Química orgánica](README.md)

# Proyección de Newman (`newman`)

Dibuja la proyección de Newman de un enlace: los tres sustituyentes del carbono de adelante y
los del de atrás, girados según el diedro. Nombra la conformación (anti, gauche, eclipsada).

## Campos

| Campo | Tipo | Valores y valor por defecto | Descripción |
| :--- | :--- | :--- | :--- |
| `type` | texto | `newman` | Obligatorio. |
| `front` | lista |  | Los tres sustituyentes del carbono de adelante. |
| `back` | lista |  | Los tres del de atrás. |
| `dihedral` | número |  | El diedro entre el primero de adelante y el primero de atrás: 180 anti, 60 gauche, 0 eclipsada. |
| `start` | texto |  | El ángulo del primer sustituyente de adelante. |
| `axis` | texto |  | El rótulo del enlace (`C2–C3`). |
| `blank` | sí/no |  | Deja los sustituyentes en blanco. |
| `blankName` | sí/no |  | Deja en blanco el nombre de la conformación. |
| `blankDihedral` | sí/no |  | Deja el diedro en blanco. |

Además, los campos comunes a todos los tipos: `title`, `subtitle`, `id`, `inkSaver` y `theme` (ver [Campos comunes](../../campos-comunes.md)).

## Modo ejercicio

Campos que dejan casilleros para completar: `blank`, `blankName`, `blankDihedral`.

## Ejemplos

### Butano conformación anti (C2–C3)

```yaml
type: newman
title: Butano conformación anti (C2–C3)
axis: C2–C3
front: [CH3, H, H]
back: [CH3, H, H]
dihedral: 180
```

[![Butano conformación anti (C2–C3)](img/ks_chem_newman_butano_anti.svg)](https://ingcom-unrn.github.io/dibu-web/#z=K6ksSLVSyEstz03M4yrJLMkB8pxKSxLz8hWS8_PS8otyE5MzD2_OU0jMK8lU0HA2etQw2dlYkyuxIrPYSgHK5Uorys8rsVKIdvYw1lHwAKJYrqTE5GxUkZTMjNSUosQcKwVDCwMuAA)

[Abrir en el editor](https://ingcom-unrn.github.io/dibu-web/#z=K6ksSLVSyEstz03M4yrJLMkB8pxKSxLz8hWS8_PS8otyE5MzD2_OU0jMK8lU0HA2etQw2dlYkyuxIrPYSgHK5Uorys8rsVKIdvYw1lHwAKJYrqTE5GxUkZTMjNSUosQcKwVDCwMuAA) para explorarlo y modificarlo.

### Conformación gauche (en blanco)

```yaml
type: newman
title: Conformación gauche (en blanco)
front: [CH3, H, H]
back: [CH3, H, H]
dihedral: 60
blankName: true
inkSaver: false
```

[![Conformación gauche (en blanco)](img/ks_chem_newman_gauche_blank.svg)](https://ingcom-unrn.github.io/dibu-web/#z=K6ksSLVSyEstz03M4yrJLMkB8pzz89Lyi3ITkzMPb85TSE8sTc5IVdBIzVNIyknMS87X5Eorys8rsVKIdvYw1lHwAKJYrqTE5GxUkZTMjNSUosQcKwUzAy6Qzmy_xFyg6SVFpalcmXnZwYllqUVWCmmJOcWpXAA)

[Abrir en el editor](https://ingcom-unrn.github.io/dibu-web/#z=K6ksSLVSyEstz03M4yrJLMkB8pzz89Lyi3ITkzMPb85TSE8sTc5IVdBIzVNIyknMS87X5Eorys8rsVKIdvYw1lHwAKJYrqTE5GxUkZTMjNSUosQcKwUzAy6Qzmy_xFyg6SVFpalcmXnZwYllqUVWCmmJOcWpXAA) para explorarlo y modificarlo.

## Ver también

[`chair`](chair.md), [`fischer`](fischer.md)
