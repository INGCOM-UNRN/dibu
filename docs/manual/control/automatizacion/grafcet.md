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

[![Llenado y mezcla en paralelo](img/ks_control_grafcet_and.svg)](https://ingcom-unrn.github.io/dibu-web/#t=dHlwZTogZ3JhZmNldAp0aXRsZTogTGxlbmFkbyB5IG1lemNsYSBlbiBwYXJhbGVsbwpzdGVwczoKICAtIHtzdGVwOiAwLCBpbml0aWFsOiB0cnVlfQogIC0ge3RyYW5zaXRpb246ICJNYXJjaGEgwrcgbml2ZWwgYmFqbyJ9CiAgLSB7c3RlcDogMSwgYWN0aW9uczogWyJBYnJpciBWMSJdfQogIC0ge3RyYW5zaXRpb246ICJuaXZlbCBhbHRvIn0KICAtIGFuZDoKICAgICAgLSBbe3N0ZXA6IDIsIGFjdGlvbnM6IFsiQWdpdGFkb3IiXX0sIHt0cmFuc2l0aW9uOiAidC9YMi8zMCBzIn0sIHtzdGVwOiA0fV0KICAgICAgLSBbe3N0ZXA6IDMsIGFjdGlvbnM6IFsiQ2FsZW50YXIiXX0sIHt0cmFuc2l0aW9uOiAiVCA-IDYwIMKwQyJ9LCB7c3RlcDogNX1dCiAgLSB7dHJhbnNpdGlvbjogIjEifQogIC0ge3N0ZXA6IDYsIGFjdGlvbnM6IFsiQWJyaXIgVjIiXX0KICAtIHt0cmFuc2l0aW9uOiAibml2ZWwgYmFqbyJ9Cg)

[Abrir en el editor](https://ingcom-unrn.github.io/dibu-web/#t=dHlwZTogZ3JhZmNldAp0aXRsZTogTGxlbmFkbyB5IG1lemNsYSBlbiBwYXJhbGVsbwpzdGVwczoKICAtIHtzdGVwOiAwLCBpbml0aWFsOiB0cnVlfQogIC0ge3RyYW5zaXRpb246ICJNYXJjaGEgwrcgbml2ZWwgYmFqbyJ9CiAgLSB7c3RlcDogMSwgYWN0aW9uczogWyJBYnJpciBWMSJdfQogIC0ge3RyYW5zaXRpb246ICJuaXZlbCBhbHRvIn0KICAtIGFuZDoKICAgICAgLSBbe3N0ZXA6IDIsIGFjdGlvbnM6IFsiQWdpdGFkb3IiXX0sIHt0cmFuc2l0aW9uOiAidC9YMi8zMCBzIn0sIHtzdGVwOiA0fV0KICAgICAgLSBbe3N0ZXA6IDMsIGFjdGlvbnM6IFsiQ2FsZW50YXIiXX0sIHt0cmFuc2l0aW9uOiAiVCA-IDYwIMKwQyJ9LCB7c3RlcDogNX1dCiAgLSB7dHJhbnNpdGlvbjogIjEifQogIC0ge3N0ZXA6IDYsIGFjdGlvbnM6IFsiQWJyaXIgVjIiXX0KICAtIHt0cmFuc2l0aW9uOiAibml2ZWwgYmFqbyJ9Cg) para explorarlo y modificarlo.

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

[![Divergencia OR con campos en blanco](img/ks_control_grafcet_or_blank.svg)](https://ingcom-unrn.github.io/dibu-web/#t=dHlwZTogZ3JhZmNldAp0aXRsZTogRGl2ZXJnZW5jaWEgT1IgY29uIGNhbXBvcyBlbiBibGFuY28Kc3RlcHM6CiAgLSB7c3RlcDogMCwgaW5pdGlhbDogdHJ1ZX0KICAtIHt0cmFuc2l0aW9uOiAiQXV0byAmIDxMaXN0bz4iLCBibGFuazogdHJ1ZX0KICAtIG9yOgogICAgICAtIFt7c3RlcDogMSwgYWN0aW9uczogWyJDaWNsbyBBIl0sIGJsYW5rOiB0cnVlfV0KICAgICAgLSBbe3N0ZXA6IDIsIGFjdGlvbnM6IFsiQ2ljbG8gQiJdfV0KICAtIHt0cmFuc2l0aW9uOiAiRmluIn0KaW5rU2F2ZXI6IGZhbHNlCg)

[Abrir en el editor](https://ingcom-unrn.github.io/dibu-web/#t=dHlwZTogZ3JhZmNldAp0aXRsZTogRGl2ZXJnZW5jaWEgT1IgY29uIGNhbXBvcyBlbiBibGFuY28Kc3RlcHM6CiAgLSB7c3RlcDogMCwgaW5pdGlhbDogdHJ1ZX0KICAtIHt0cmFuc2l0aW9uOiAiQXV0byAmIDxMaXN0bz4iLCBibGFuazogdHJ1ZX0KICAtIG9yOgogICAgICAtIFt7c3RlcDogMSwgYWN0aW9uczogWyJDaWNsbyBBIl0sIGJsYW5rOiB0cnVlfV0KICAgICAgLSBbe3N0ZXA6IDIsIGFjdGlvbnM6IFsiQ2ljbG8gQiJdfV0KICAtIHt0cmFuc2l0aW9uOiAiRmluIn0KaW5rU2F2ZXI6IGZhbHNlCg) para explorarlo y modificarlo.

## Ver también

[`instrumentation`](instrumentation.md), [`ladder`](ladder.md)
