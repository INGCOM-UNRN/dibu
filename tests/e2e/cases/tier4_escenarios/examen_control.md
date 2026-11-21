# Primer Parcial — Control Automático y Automatización Industrial

**Instrucciones:** Resolvé los siguientes ejercicios justificando cada paso a partir de los diagramas.

## 1. Lazo de control y respuesta temporal

Analizá el siguiente lazo realimentado y su respuesta al escalón unitario (bloque apilado):

```estructura
- type: block_diagram
  title: Lazo cerrado de temperatura
  input: "R(s)"
  output: "Y(s)"
  forward:
    - {label: "Gc(s)", out: "U(s)"}
    - "Gp(s)"
  disturbance: {at: 1, label: "D(s)", block: "Gd(s)"}
  feedback: ["H(s)"]
- type: time_response
  title: Respuesta al escalón (ζ = 0,3)
  system: {gain: 1, zeta: 0.3, wn: 2}
  annotations: [final, overshoot, peak, settling]
```

## 2. Análisis frecuencial (Bode y Nyquist)

Determiná los márgenes de ganancia y fase del sistema $L(s) = \frac{10}{s(s+1)(s+10)}$:

```statics
type: bode
title: Diagrama de Bode de L(s)
system: {k: 10, poles: [0, -1, -10]}
asymptotes: true
margins: true
```

```estructura
type: nyquist
title: Traza de Nyquist
system: {k: 5, poles: [-1, -2, -3]}
unitCircle: true
margins: true
```

## 3. Polos, ceros y lugar de las raíces

Completá los valores críticos del lugar de las raíces para el sistema dado:

```estructura
type: pole_zero
title: Mapa de polos y ceros
poles: [-1, [-2, 3]]
zeros: [-4]
dampingLines: [0.5]
wnCircles: [3]
showValues: true
```

```statics
type: root_locus
title: Lugar de las raíces
system: {k: 1, poles: [0, -1, -2]}
marks: [1, 2]
blankMarks: true
```

## 4. Instrumentación P&ID, Ladder y GRAFCET

Diseñá la lógica de control del tanque TK-101 según el diagrama P&ID, el diagrama escalera y la secuencia GRAFCET:

```estructura
type: instrumentation
title: Control de nivel TK-101
equipment:
  - {id: TK, kind: tank, at: [2, 3], label: "TK-101", level: 0.6}
  - {id: LV, kind: control_valve, at: [7, 3], label: "LV-101"}
instruments:
  - {id: LT, tag: "LT 101", at: [4, 1]}
  - {id: LIC, tag: "LIC 101", at: [6, 1], location: dcs}
lines:
  - {from: TK, to: LT, kind: capillary}
  - {from: LT, to: LIC, kind: electric}
  - {from: LIC, to: LV, kind: electric, toActuator: true}
```

~~~statics
type: ladder
title: Arranque con enclavamiento
rungs:
  - comment: "Motor principal"
    logic:
      - or:
          - [{no: "I0.0", label: "Marcha"}]
          - [{no: "Q0.0"}]
      - {nc: "I0.1", label: "Parada"}
      - {coil: "Q0.0", label: "Motor"}
  - logic:
      - {no: "Q0.0"}
      - {ton: "T1", preset: "10 s"}
~~~

```estructura
type: grafcet
title: Ciclo de llenado y vaciado
steps:
  - {step: 0, initial: true}
  - {transition: "Marcha · nivel bajo"}
  - {step: 1, actions: ["Abrir V1"]}
  - {transition: "nivel alto"}
  - {step: 2, actions: ["Abrir V2"]}
  - {transition: "nivel bajo"}
```
