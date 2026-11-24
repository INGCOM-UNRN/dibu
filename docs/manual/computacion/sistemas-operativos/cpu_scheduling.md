[Manual](../../README.md) › [Sistemas de computación](../README.md) › [Sistemas operativos](README.md)

# Planificación de CPU (`cpu_scheduling`)

Simula la planificación de los procesos y dibuja el diagrama de Gantt, con la CPU ociosa
cuando no hay nadie listo y una flecha en cada llegada. Debajo, la tabla con el instante de
fin y los tiempos de retorno, espera y respuesta de cada proceso, y sus promedios.

## Qué calcula

- FCFS, SJF y prioridad no expropiativos; SRTF y prioridad expropiativos; Round Robin con quantum.
- En Round Robin, los que llegan durante un quantum entran a la cola antes que el expropiado.
- Los empates se rompen por llegada y después por el orden de la lista.

## Campos

| Campo | Tipo | Valores y valor por defecto | Descripción |
| :--- | :--- | :--- | :--- |
| `type` | texto | `cpu_scheduling` | Obligatorio. |
| `processes` | lista |  | Los procesos (hasta 26): `{name, arrival, burst, priority}`. `arrival` es 0 por omisión; la prioridad más alta es el número menor. |
| `algorithm` | texto | `fcfs`, `sjf`, `srtf`, `rr`, `priority`, `priority_preemptive` · por defecto `fcfs` | `fcfs`, `sjf`, `srtf`, `rr`, `priority` o `priority_preemptive`. |
| `quantum` | número |  | El quantum de Round Robin (por omisión 2). |
| `blank` | sí/no |  | Deja el Gantt vacío (con la escala de tiempo) y los tiempos en blanco. |

Además, los campos comunes a todos los tipos: `title`, `subtitle`, `id`, `inkSaver` y `theme` (ver [Campos comunes](../../campos-comunes.md)).

## Modo ejercicio

Campos que dejan casilleros para completar: `blank`.

## Ejemplos

### Planificación SRTF

```yaml
type: cpu_scheduling
title: Planificación SRTF
algorithm: srtf
processes:
  - { name: P1, arrival: 0, burst: 8 }
  - { name: P2, arrival: 1, burst: 4 }
  - { name: P3, arrival: 2, burst: 9 }
  - { name: P4, arrival: 3, burst: 5 }
```

![Planificación SRTF](img/ks_so_srtf.svg)

### Round Robin

```yaml
type: cpu_scheduling
title: Round Robin
algorithm: rr
quantum: 3
processes:
  - { name: A, arrival: 0, burst: 5 }
  - { name: B, arrival: 1, burst: 3 }
  - { name: C, arrival: 2, burst: 6 }
  - { name: D, arrival: 9, burst: 2 }
```

![Round Robin](img/ks_so_round_robin.svg)

### Ejercicio: prioridad no expropiativa

```yaml
type: cpu_scheduling
title: "Ejercicio: prioridad no expropiativa"
algorithm: priority
blank: true
processes:
  - { name: P1, arrival: 0, burst: 10, priority: 3 }
  - { name: P2, arrival: 0, burst: 1, priority: 1 }
  - { name: P3, arrival: 0, burst: 2, priority: 4 }
  - { name: P4, arrival: 0, burst: 1, priority: 5 }
  - { name: P5, arrival: 0, burst: 5, priority: 2 }
```

![Ejercicio: prioridad no expropiativa](img/ks_so_prioridad_ejercicio.svg)

## Ver también

[`page_table`](page_table.md), [`page_replacement`](page_replacement.md)
