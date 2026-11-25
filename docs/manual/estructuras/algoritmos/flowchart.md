[Manual](../../README.md) › [Estructuras de datos y algoritmos](../README.md) › [Análisis de algoritmos](README.md)

# Diagrama de flujo (`flowchart`)

Arma el diagrama de flujo a partir de la estructura del algoritmo: pasos, entradas y salidas,
condicionales y ciclos anidados. Cada paso con `blank: true` sale vacío para completar.

## Campos

| Campo | Tipo | Valores y valor por defecto | Descripción |
| :--- | :--- | :--- | :--- |
| `type` | texto | `flowchart` | Obligatorio. |
| `steps` | lista |  | Los pasos: un texto, `{io}` (entrada o salida), `{step}`, `{if, then, else}`, `{while, do}`, `{for, do}` o `{doWhile, do}` (el cuerpo antes de la condición); cada uno con `blank`. |
| `start` | texto |  | Rótulo del inicio (por omisión `Inicio`); `false` lo quita. |
| `end` | texto |  | Rótulo del fin (por omisión `Fin`); `false` lo quita. |
| `blankValues` | sí/no |  | Deja en blanco el texto de todos los pasos. |

Además, los campos comunes a todos los tipos: `title`, `subtitle`, `id`, `inkSaver` y `theme` (ver [Campos comunes](../../campos-comunes.md)).

## Modo ejercicio

Campos que dejan casilleros para completar: `blankValues`.

## Ejemplos

### Bucle con condición < & >

```yaml
type: flowchart
title: "Bucle con condición < & >"
steps:
  - {io: "leer n"}
  - "i = 0"
  - while: "i < n && n > 0"
    do:
      - if: "par(i)"
        then: ["s = s + i"]
        else: ["s = s - i"]
      - "i = i + 1"
```

![Bucle con condición < & >](img/ks_cs_flowchart_completo.svg)

### Lazo do-while: el cuerpo se ejecuta al menos una vez

```yaml
type: flowchart
title: "Lazo do-while: el cuerpo se ejecuta al menos una vez"
steps:
  - "i = 0"
  - doWhile: "i < 3"
    do:
      - "procesar(i)"
      - "i = i + 1"
  - {io: "mostrar i"}
```

![Lazo do-while: el cuerpo se ejecuta al menos una vez](img/ks_cs_flowchart_do_while.svg)

## Ver también

[`complexity`](complexity.md), [`cost_table`](cost_table.md), [`trace_table`](trace_table.md), [`dp_table`](dp_table.md), [`dijkstra`](dijkstra.md)
