[Manual](../../README.md) › [Control de procesos](../README.md) › [Instrumentación y automatización](README.md)

# GRAFCET (`grafcet`)

Dibuja un GRAFCET: etapas con sus acciones, transiciones con su receptividad y divergencias
y convergencias en O y en Y. Por omisión, la última transición vuelve a la etapa inicial.

## Campos

| Campo | Tipo | Valores y valor por defecto | Descripción |
| :--- | :--- | :--- | :--- |
| `type` | texto | `grafcet` | Obligatorio. |
| `steps` | lista |  | En orden: `step` (número, `initial`, `actions`, `blank`), `transition` (la receptividad, `blank`), `or` y `and` (divergencias: listas de secuencias). |
| `loop` | sí/no |  | Vuelve a la etapa inicial al final (por omisión, sí). |

Además, los campos comunes a todos los tipos: `title`, `subtitle`, `id`, `inkSaver` y `theme` (ver [Campos comunes](../../campos-comunes.md)).

## Ejemplos

### Llenado y mezcla en paralelo

```yaml
type: grafcet
title: Llenado y mezcla en paralelo
steps:
  - {step: 0, initial: true}
  - {transition: "Marcha · nivel bajo"}
  - {step: 1, actions: ["Abrir V1"]}
  - {transition: "nivel alto"}
  - and:
      - [{step: 2, actions: ["Agitador"]}, {transition: "t/X2/30 s"}, {step: 4}]
      - [{step: 3, actions: ["Calentar"]}, {transition: "T > 60 °C"}, {step: 5}]
  - {transition: "1"}
  - {step: 6, actions: ["Abrir V2"]}
  - {transition: "nivel bajo"}
```

![Llenado y mezcla en paralelo](img/ks_control_grafcet_and.svg)

### Divergencia OR con campos en blanco

```yaml
type: grafcet
title: Divergencia OR con campos en blanco
steps:
  - {step: 0, initial: true}
  - {transition: "Auto & <Listo>", blank: true}
  - or:
      - [{step: 1, actions: ["Ciclo A"], blank: true}]
      - [{step: 2, actions: ["Ciclo B"]}]
  - {transition: "Fin"}
inkSaver: false
```

![Divergencia OR con campos en blanco](img/ks_control_grafcet_or_blank.svg)

## Ver también

[`instrumentation`](instrumentation.md), [`ladder`](ladder.md)
