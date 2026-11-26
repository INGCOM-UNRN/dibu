[Manual](../../README.md) › [Química](../README.md) › [Química orgánica](README.md)

# Proyección de Fischer (`fischer`)

Dibuja la proyección de Fischer de una molécula con uno o varios centros quirales, de arriba
hacia abajo. Con un solo centro asigna R o S por las reglas CIP para los grupos comunes; la
serie D o L sale del último centro.

## Qué calcula

- R/S con prioridades CIP (OH, NH₂, CHO, COOH, CH₂OH, CH₃, halógenos…), o con `priorities`.
- La serie D/L.

## Campos

| Campo | Tipo | Valores y valor por defecto | Descripción |
| :--- | :--- | :--- | :--- |
| `type` | texto | `fischer` | Obligatorio. |
| `top` | texto |  | El grupo de arriba. |
| `bottom` | texto |  | El grupo de abajo. |
| `centers` | lista |  | Los centros, de arriba hacia abajo: `left`, `right`, `blank` (`left`, `right` o `true`) y `priorities` (los lugares `top`, `right`, `bottom`, `left` de mayor a menor prioridad). |
| `showConfig` | sí/no |  | Escribe R o S. |
| `showSeries` | sí/no |  | Escribe la serie D o L (por omisión, sí). |
| `blank` | sí/no |  | Deja los grupos en blanco. |
| `blankConfig` | sí/no |  | Deja R/S en blanco. |
| `blankSeries` | sí/no |  | Deja la serie en blanco. |

Además, los campos comunes a todos los tipos: `title`, `subtitle`, `id`, `inkSaver` y `theme` (ver [Campos comunes](../../campos-comunes.md)).

## Modo ejercicio

Campos que dejan casilleros para completar: `blank`, `blankConfig`, `blankSeries`.

## Ejemplos

### D-glucosa (4 centros quirales)

```yaml
type: fischer
title: D-glucosa (4 centros quirales)
top: CHO
bottom: CH2OH
centers:
  - {left: H, right: OH, priorities: [right, top, bottom, left]}
  - {left: HO, right: H, priorities: [left, top, bottom, right]}
  - {left: H, right: OH, priorities: [right, top, bottom, left]}
  - {left: H, right: OH, priorities: [right, top, bottom, left]}
```

[![D-glucosa (4 centros quirales)](img/ks_chem_fischer_glucosa.svg)](https://ingcom-unrn.github.io/dibu-web/#t=dHlwZTogZmlzY2hlcgp0aXRsZTogRC1nbHVjb3NhICg0IGNlbnRyb3MgcXVpcmFsZXMpCnRvcDogQ0hPCmJvdHRvbTogQ0gyT0gKY2VudGVyczoKICAtIHtsZWZ0OiBILCByaWdodDogT0gsIHByaW9yaXRpZXM6IFtyaWdodCwgdG9wLCBib3R0b20sIGxlZnRdfQogIC0ge2xlZnQ6IEhPLCByaWdodDogSCwgcHJpb3JpdGllczogW2xlZnQsIHRvcCwgYm90dG9tLCByaWdodF19CiAgLSB7bGVmdDogSCwgcmlnaHQ6IE9ILCBwcmlvcml0aWVzOiBbcmlnaHQsIHRvcCwgYm90dG9tLCBsZWZ0XX0KICAtIHtsZWZ0OiBILCByaWdodDogT0gsIHByaW9yaXRpZXM6IFtyaWdodCwgdG9wLCBib3R0b20sIGxlZnRdfQo)

[Abrir en el editor](https://ingcom-unrn.github.io/dibu-web/#t=dHlwZTogZmlzY2hlcgp0aXRsZTogRC1nbHVjb3NhICg0IGNlbnRyb3MgcXVpcmFsZXMpCnRvcDogQ0hPCmJvdHRvbTogQ0gyT0gKY2VudGVyczoKICAtIHtsZWZ0OiBILCByaWdodDogT0gsIHByaW9yaXRpZXM6IFtyaWdodCwgdG9wLCBib3R0b20sIGxlZnRdfQogIC0ge2xlZnQ6IEhPLCByaWdodDogSCwgcHJpb3JpdGllczogW2xlZnQsIHRvcCwgYm90dG9tLCByaWdodF19CiAgLSB7bGVmdDogSCwgcmlnaHQ6IE9ILCBwcmlvcml0aWVzOiBbcmlnaHQsIHRvcCwgYm90dG9tLCBsZWZ0XX0KICAtIHtsZWZ0OiBILCByaWdodDogT0gsIHByaW9yaXRpZXM6IFtyaWdodCwgdG9wLCBib3R0b20sIGxlZnRdfQo) para explorarlo y modificarlo.

### Alanina (configuración en blanco)

```yaml
type: fischer
title: Alanina (configuración en blanco)
top: COOH
bottom: CH3
centers:
  - {left: NH2, right: H}
blankConfig: true
blankSeries: true
inkSaver: false
```

[![Alanina (configuración en blanco)](img/ks_chem_fischer_alanina_blank.svg)](https://ingcom-unrn.github.io/dibu-web/#t=dHlwZTogZmlzY2hlcgp0aXRsZTogQWxhbmluYSAoY29uZmlndXJhY2nDs24gZW4gYmxhbmNvKQp0b3A6IENPT0gKYm90dG9tOiBDSDMKY2VudGVyczoKICAtIHtsZWZ0OiBOSDIsIHJpZ2h0OiBIfQpibGFua0NvbmZpZzogdHJ1ZQpibGFua1NlcmllczogdHJ1ZQppbmtTYXZlcjogZmFsc2UK)

[Abrir en el editor](https://ingcom-unrn.github.io/dibu-web/#t=dHlwZTogZmlzY2hlcgp0aXRsZTogQWxhbmluYSAoY29uZmlndXJhY2nDs24gZW4gYmxhbmNvKQp0b3A6IENPT0gKYm90dG9tOiBDSDMKY2VudGVyczoKICAtIHtsZWZ0OiBOSDIsIHJpZ2h0OiBIfQpibGFua0NvbmZpZzogdHJ1ZQpibGFua1NlcmllczogdHJ1ZQppbmtTYXZlcjogZmFsc2UK) para explorarlo y modificarlo.

## Ver también

[`newman`](newman.md), [`chair`](chair.md)
