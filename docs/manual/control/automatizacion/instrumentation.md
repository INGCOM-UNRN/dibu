[Manual](../../README.md) › [Control de procesos](../README.md) › [Instrumentación y automatización](README.md)

# P&ID (`instrumentation`)

Dibuja un diagrama de cañerías e instrumentos: equipos (tanques, bombas, válvulas,
intercambiadores), instrumentos con su tag y su ubicación (campo, panel, DCS, PLC) y las
líneas de proceso, eléctricas, neumáticas o de datos que los unen, sobre una grilla.

## Campos

| Campo | Tipo | Valores y valor por defecto | Descripción |
| :--- | :--- | :--- | :--- |
| `type` | texto | `instrumentation` | Obligatorio. |
| `equipment` | lista |  | Los equipos: `id`, `kind` (`tank`, `vessel`, `reactor`, `column`, `pump`, `control_valve`, `valve`, `heat_exchanger`), `at: [x, y]` en la grilla, `label`, `size`, `level`, `orientation: v`. |
| `instruments` | lista |  | Los instrumentos: `id`, `tag` (`"LIC 101"`), `at`, `location` (`field`, `panel`, `panel_back`, `dcs`, `plc`) y `blank`. |
| `lines` | lista |  | Las líneas: `from` y `to` (un id o `[x, y]`), `kind` (`process`, `electric`, `pneumatic`, `data`, `capillary`), `via`, `route` (`hv` o `vh`), `label`, `arrow`. |
| `grid` | número |  | Píxeles por unidad de la grilla. |
| `legend` | sí/no |  | Agrega la leyenda de los tipos de línea. |

Además, los campos comunes a todos los tipos: `title`, `subtitle`, `id`, `inkSaver` y `theme` (ver [Campos comunes](../../campos-comunes.md)).

## Ejemplos

### Control de nivel TK-101 (ISA-5.1)

```yaml
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

[![Control de nivel TK-101 (ISA-5.1)](img/ks_control_instrumentation_pid.svg)](https://ingcom-unrn.github.io/dibu-web/#z=bZHNagIxFIX38xQXVy3EwfhPdsWV1IXg4EakpDOxBGMyzcQBKb57cyczGm13IfnOueeeuEspGEhdOXs-Ce24k0YnTjrlrxdGO2sUFAK0rIWC7L1PBxRelpu3_iSlr4n4PssSdSwB6MOPLJiHCByl9ifH9ZEAdwx2IwKjPQHFP4Vi0AtGPX8harwYpNPr3WFNO4fyfCpbhymBSTqMPNaNRSRbbTtZHoJ_1FzVotXP0wmBcTqPHFbb1uK-fxUtssqIX-ELwQxC3MbJ-wz38dzl4gYuFzE5I0BxnsmbXhkUeXVNlNSim3Ow5uTBAYaj6djDzoQKu5A-lOUF95a15B7FKpHsEgQHVKASq7tzWNgjh-_IYVctNwu1PHL4jtyO0ufWKq6kz_PveKys_QJeSqW4vdzmjLG2pylZK8MGg04okTsr85sMi3mWNYWb-Mf_yJrvboRSHze8FpbBgatKJL8)

[Abrir en el editor](https://ingcom-unrn.github.io/dibu-web/#z=bZHNagIxFIX38xQXVy3EwfhPdsWV1IXg4EakpDOxBGMyzcQBKb57cyczGm13IfnOueeeuEspGEhdOXs-Ce24k0YnTjrlrxdGO2sUFAK0rIWC7L1PBxRelpu3_iSlr4n4PssSdSwB6MOPLJiHCByl9ifH9ZEAdwx2IwKjPQHFP4Vi0AtGPX8harwYpNPr3WFNO4fyfCpbhymBSTqMPNaNRSRbbTtZHoJ_1FzVotXP0wmBcTqPHFbb1uK-fxUtssqIX-ELwQxC3MbJ-wz38dzl4gYuFzE5I0BxnsmbXhkUeXVNlNSim3Ow5uTBAYaj6djDzoQKu5A-lOUF95a15B7FKpHsEgQHVKASq7tzWNgjh-_IYVctNwu1PHL4jtyO0ufWKq6kz_PveKys_QJeSqW4vdzmjLG2pylZK8MGg04okTsr85sMi3mWNYWb-Mf_yJrvboRSHze8FpbBgatKJL8) para explorarlo y modificarlo.

### Instrumento en blanco

```yaml
type: instrumentation
title: Instrumento en blanco
equipment:
  - {id: TK, kind: tank, at: [2, 2], label: "TK-1"}
instruments:
  - {id: LT, tag: "LT 1", at: [5, 1], blank: true}
lines:
  - {from: TK, to: LT, kind: capillary}
```

[![Instrumento en blanco](img/ks_control_instrumentation_blank.svg)](https://ingcom-unrn.github.io/dibu-web/#z=TY47DoMwEER7n2JEbQoipfENIijdRSkMcaIVZk3MUqCIu8eI_LrV08yblWX0BsSTpHnwLE4oshKSkPHpiyM8ow2Ou6j8Y6Zxg0YBJZ50NbC1Rk-cL3HcazgxOB80DheN4FofDApbl1Wxqt_W9CdorM7Ve441FlXxNhw1qmzYhvusTrNfVSD2n-YtxWEfl7g79ic6N1IILi2regE)

[Abrir en el editor](https://ingcom-unrn.github.io/dibu-web/#z=TY47DoMwEER7n2JEbQoipfENIijdRSkMcaIVZk3MUqCIu8eI_LrV08yblWX0BsSTpHnwLE4oshKSkPHpiyM8ow2Ou6j8Y6Zxg0YBJZ50NbC1Rk-cL3HcazgxOB80DheN4FofDApbl1Wxqt_W9CdorM7Ve441FlXxNhw1qmzYhvusTrNfVSD2n-YtxWEfl7g79ic6N1IILi2regE) para explorarlo y modificarlo.

## Ver también

[`ladder`](ladder.md), [`grafcet`](grafcet.md)
