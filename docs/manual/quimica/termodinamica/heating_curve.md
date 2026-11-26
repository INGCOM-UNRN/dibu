[Manual](../../README.md) › [Química](../README.md) › [Termodinámica](README.md)

# Curva de calentamiento (`heating_curve`)

Dibuja la curva de calentamiento de una masa de sustancia, desde una temperatura inicial hasta
una final: los tramos en cada fase y las mesetas de fusión y ebullición, con el calor de cada
tramo y el total.

## Qué calcula

- El calor de cada tramo (m·c·ΔT y m·ΔH) y el total.

## Campos

| Campo | Tipo | Valores y valor por defecto | Descripción |
| :--- | :--- | :--- | :--- |
| `type` | texto | `heating_curve` | Obligatorio. |
| `substance` | texto |  | Una sustancia conocida, `water`. |
| `cpSolid` | número |  | El calor específico del sólido, en J/g·K. |
| `cpLiquid` | número |  | El calor específico del líquido. |
| `cpGas` | número |  | El calor específico del gas. |
| `melting` | número |  | La temperatura de fusión, en °C. |
| `boiling` | número |  | La temperatura de ebullición. |
| `hFusion` | número |  | El calor de fusión, en J/g. |
| `hVaporization` | número |  | El calor de vaporización. |
| `mass` | número |  | La masa, en g. |
| `from` | número |  | La temperatura inicial. |
| `to` | número |  | La temperatura final. |
| `showHeat` | sí/no |  | Escribe el calor de cada tramo. |
| `showTotal` | sí/no |  | Escribe el calor total. |
| `blankHeat` | sí/no |  | Deja en blanco los calores. |
| `blankPhases` | sí/no |  | Deja en blanco los nombres de las fases. |

Además, los campos comunes a todos los tipos: `title`, `subtitle`, `id`, `inkSaver` y `theme` (ver [Campos comunes](../../campos-comunes.md)).

## Modo ejercicio

Campos que dejan casilleros para completar: `blankHeat`, `blankPhases`.

## Ejemplos

### 100 g de hielo de −20 °C a 120 °C

```yaml
type: heating_curve
title: 100 g de hielo de −20 °C a 120 °C
substance: water
mass: 100
from: -20
to: 120
```

[![100 g de hielo de −20 °C a 120 °C](img/ks_chem_heating_curve_agua.svg)](https://ingcom-unrn.github.io/dibu-web/#t=dHlwZTogaGVhdGluZ19jdXJ2ZQp0aXRsZTogMTAwIGcgZGUgaGllbG8gZGUg4oiSMjAgwrBDIGEgMTIwIMKwQwpzdWJzdGFuY2U6IHdhdGVyCm1hc3M6IDEwMApmcm9tOiAtMjAKdG86IDEyMAo)

[Abrir en el editor](https://ingcom-unrn.github.io/dibu-web/#t=dHlwZTogaGVhdGluZ19jdXJ2ZQp0aXRsZTogMTAwIGcgZGUgaGllbG8gZGUg4oiSMjAgwrBDIGEgMTIwIMKwQwpzdWJzdGFuY2U6IHdhdGVyCm1hc3M6IDEwMApmcm9tOiAtMjAKdG86IDEyMAo) para explorarlo y modificarlo.

### Curva de calentamiento (evaluación)

```yaml
type: heating_curve
title: Curva de calentamiento (evaluación)
substance: water
mass: 50
from: -10
to: 110
blankHeat: true
blankPhases: true
inkSaver: false
```

[![Curva de calentamiento (evaluación)](img/ks_chem_heating_curve_blank.svg)](https://ingcom-unrn.github.io/dibu-web/#t=dHlwZTogaGVhdGluZ19jdXJ2ZQp0aXRsZTogQ3VydmEgZGUgY2FsZW50YW1pZW50byAoZXZhbHVhY2nDs24pCnN1YnN0YW5jZTogd2F0ZXIKbWFzczogNTAKZnJvbTogLTEwCnRvOiAxMTAKYmxhbmtIZWF0OiB0cnVlCmJsYW5rUGhhc2VzOiB0cnVlCmlua1NhdmVyOiBmYWxzZQo)

[Abrir en el editor](https://ingcom-unrn.github.io/dibu-web/#t=dHlwZTogaGVhdGluZ19jdXJ2ZQp0aXRsZTogQ3VydmEgZGUgY2FsZW50YW1pZW50byAoZXZhbHVhY2nDs24pCnN1YnN0YW5jZTogd2F0ZXIKbWFzczogNTAKZnJvbTogLTEwCnRvOiAxMTAKYmxhbmtIZWF0OiB0cnVlCmJsYW5rUGhhc2VzOiB0cnVlCmlua1NhdmVyOiBmYWxzZQo) para explorarlo y modificarlo.

## Ver también

[`phase_diagram`](phase_diagram.md)
