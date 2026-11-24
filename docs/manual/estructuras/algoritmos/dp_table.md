[Manual](../../README.md) › [Estructuras de datos y algoritmos](../README.md) › [Análisis de algoritmos](README.md)

# Tabla de programación dinámica (`dp_table`)

Llena la tabla de programación dinámica del problema elegido y resalta el camino de
reconstrucción de la solución. Debajo escribe el resultado: la subsecuencia, las operaciones
de edición, los objetos elegidos o las monedas.

## Qué calcula

- `lcs`: la longitud y una subsecuencia común más larga.
- `edit`: la distancia de Levenshtein y las operaciones (sustituir, insertar, borrar).
- `knapsack`: el valor máximo y los objetos elegidos.
- `coins`: la mínima cantidad de monedas y cuáles.

## Campos

| Campo | Tipo | Valores y valor por defecto | Descripción |
| :--- | :--- | :--- | :--- |
| `type` | texto | `dp_table` | Obligatorio. |
| `problem` | texto | `lcs`, `edit`, `knapsack`, `coins` · por defecto `lcs` | `lcs`, `edit`, `knapsack` o `coins`. |
| `a` | texto |  | En `lcs` y `edit`, el primer texto (hasta 24 caracteres). |
| `b` | texto |  | En `lcs` y `edit`, el segundo texto. |
| `weights` | lista |  | En `knapsack`, los pesos (de 1 a 15 objetos). |
| `values` | lista |  | En `knapsack`, los valores. |
| `capacity` | número |  | En `knapsack`, la capacidad de la mochila (de 1 a 40). |
| `coins` | lista |  | En `coins`, los valores de las monedas. |
| `amount` | número |  | En `coins`, el monto a formar (de 1 a 60). |
| `path` | sí/no |  | Resalta el camino de reconstrucción (por omisión, sí). |
| `blank` | sí/no |  | Deja la tabla en blanco (salvo la primera fila y columna) y el resultado. |

Además, los campos comunes a todos los tipos: `title`, `subtitle`, `id`, `inkSaver` y `theme` (ver [Campos comunes](../../campos-comunes.md)).

## Modo ejercicio

Campos que dejan casilleros para completar: `blank`.

## Ejemplos

### Subsecuencia común más larga

```yaml
type: dp_table
title: Subsecuencia común más larga
problem: lcs
a: ABCBDAB
b: BDCABA
```

![Subsecuencia común más larga](img/ks_alg_lcs.svg)

### Distancia de edición (Levenshtein)

```yaml
type: dp_table
title: Distancia de edición (Levenshtein)
problem: edit
a: kitten
b: sitting
```

![Distancia de edición (Levenshtein)](img/ks_alg_edicion.svg)

### Ejercicio: mochila 0/1

```yaml
type: dp_table
title: "Ejercicio: mochila 0/1"
problem: knapsack
weights: [1, 3, 4, 5]
values: [1, 4, 5, 7]
capacity: 7
blank: true
```

![Ejercicio: mochila 0/1](img/ks_alg_mochila.svg)

## Ver también

[`complexity`](complexity.md), [`flowchart`](flowchart.md), [`cost_table`](cost_table.md), [`trace_table`](trace_table.md), [`dijkstra`](dijkstra.md)
