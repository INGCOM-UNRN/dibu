[Manual](../../README.md) › [Química](../README.md) › [Química inorgánica](README.md)

# Campo cristalino (`crystal_field`)

Dibuja el desdoblamiento de los orbitales d en un campo octaédrico, tetraédrico o cuadrado
plano y los llena con los electrones d del metal, en alto o bajo espín según el ligando.
Calcula los electrones desapareados, el momento magnético de solo espín y la energía de
estabilización.

## Qué calcula

- Los electrones d a partir del metal y su estado de oxidación.
- El espín por la serie espectroquímica (desde el NH₃, campo fuerte), μ = √(n(n+2)) y la EECC.

## Campos

| Campo | Tipo | Valores y valor por defecto | Descripción |
| :--- | :--- | :--- | :--- |
| `type` | texto | `crystal_field` | Obligatorio. |
| `geometry` | texto |  | `octahedral`, `tetrahedral` o `square_planar`. |
| `electrons` | número |  | Los electrones d, o `metal` y `oxidation`. |
| `metal` | texto |  | El metal. |
| `oxidation` | número |  | Su estado de oxidación. |
| `spin` | texto |  | `high` o `low`, o el que dé `ligand`. |
| `ligand` | texto |  | El ligando, de la serie espectroquímica. |
| `complex` | texto |  | La fórmula del complejo, de título. |
| `blank` | sí/no |  | Deja los orbitales vacíos. |
| `blankElectrons` | sí/no |  | Deja los electrones en blanco. |
| `blankConfig` | sí/no |  | Deja la configuración en blanco. |
| `blankSpin` | sí/no |  | Deja el espín en blanco. |
| `blankUnpaired` | sí/no |  | Deja los desapareados en blanco. |
| `blankMoment` | sí/no |  | Deja μ en blanco. |
| `blankCFSE` | sí/no |  | Deja la EECC en blanco. |

Además, los campos comunes a todos los tipos: `title`, `subtitle`, `id`, `inkSaver` y `theme` (ver [Campos comunes](../../campos-comunes.md)).

## Modo ejercicio

Campos que dejan casilleros para completar: `blank`, `blankElectrons`, `blankConfig`, `blankSpin`, `blankUnpaired`, `blankMoment`, `blankCFSE`.

## Ejemplos

### [Co(NH3)6]^3+ (bajo espín)

```yaml
type: crystal_field
title: "[Co(NH3)6]^3+ (bajo espín)"
metal: Co
oxidation: 3
ligand: NH3
inkSaver: false
```

![[Co(NH3)6]^3+ (bajo espín)](img/ks_chem_crystal_field_octa.svg)

### [Fe(CN)6]^4- (evaluación)

```yaml
type: crystal_field
title: "[Fe(CN)6]^4- (evaluación)"
metal: Fe
oxidation: 2
ligand: CN^-
blankElectrons: true
blank: true
```

![[Fe(CN)6]^4- (evaluación)](img/ks_chem_crystal_field_blanco.svg)

## Ver también

[`complex`](complex.md), [`born_haber`](born_haber.md)
