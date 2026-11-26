[Manual](../../README.md) › [Control de procesos](../README.md) › [Instrumentación y automatización](README.md)

# Diagrama escalera (`ladder`)

Dibuja los escalones de un programa en lenguaje escalera entre los dos rieles de
alimentación. Cada escalón es una secuencia de contactos (en serie, o en paralelo con `or`)
que termina en sus salidas.

## Campos

| Campo | Tipo | Valores y valor por defecto | Descripción |
| :--- | :--- | :--- | :--- |
| `type` | texto | `ladder` | Obligatorio. |
| `rungs` | lista |  | Los escalones, cada uno con `logic` y `comment`. En `logic`: contactos `no` (o `na`), `nc`, `p`, `n`; salidas `coil`, `ncoil`, `set`, `reset`; temporizadores `ton`, `tof`, `tp` y contadores `ctu`, `ctd` con `preset`; `or` (ramas en paralelo, cada una una lista). Cada elemento con `label` y `blank`. |

Además, los campos comunes a todos los tipos: `title`, `subtitle`, `id`, `inkSaver` y `theme` (ver [Campos comunes](../../campos-comunes.md)).

## Ejemplos

### Arranque y parada con enclavamiento

```yaml
type: ladder
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
  - comment: "Temporizador de llenado"
    logic:
      - {"no": "Q0.0"}
      - {ton: "T1", preset: "10 s"}
  - logic:
      - {"no": "T1"}
      - {set: "Q0.1", label: "Alarma"}
inkSaver: true
```

[![Arranque y parada con enclavamiento](img/ks_control_ladder_enclavamiento.svg)](https://ingcom-unrn.github.io/dibu-web/#z=dZC_TgMxDMb3eworM0VXxmwdGSpRtRtiMIlVIpL48OUqlaoPxHPwYiR3lKZq2fzn58_-nPYdafBoLUmTXPI5W4hg_BgI9tChoEUwHIGi8bjD4CgmbmSI2143ALPcDCHXNKglJ5ZrWGUMwPPWGT2GZYrlFE_580FFVlnksb1v1V0-6ZV80UQxb6iOL7fpVaH_mjM4RDNpzCuNp9GFOl5TDxW1-f6S4AzXnGHnT1uqm4rPEavdbyh0LO4TbX6CJfCeYo5vur88_1xOHItSOb4T6qnozlvof5f9o5L5SmOaWl2-YOFRQnmBi-9r3JFoSDJQ8wM)

[Abrir en el editor](https://ingcom-unrn.github.io/dibu-web/#z=dZC_TgMxDMb3eworM0VXxmwdGSpRtRtiMIlVIpL48OUqlaoPxHPwYiR3lKZq2fzn58_-nPYdafBoLUmTXPI5W4hg_BgI9tChoEUwHIGi8bjD4CgmbmSI2143ALPcDCHXNKglJ5ZrWGUMwPPWGT2GZYrlFE_580FFVlnksb1v1V0-6ZV80UQxb6iOL7fpVaH_mjM4RDNpzCuNp9GFOl5TDxW1-f6S4AzXnGHnT1uqm4rPEavdbyh0LO4TbX6CJfCeYo5vur88_1xOHItSOb4T6qnozlvof5f9o5L5SmOaWl2-YOFRQnmBi-9r3JFoSDJQ8wM) para explorarlo y modificarlo.

## Notas

- YAML lee `no:` como `false`; dibu lo entiende igual, y `na:` también vale.

## Ver también

[`instrumentation`](instrumentation.md), [`grafcet`](grafcet.md)
