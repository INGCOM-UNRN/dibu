[Manual](../../README.md) › [Estructuras de datos y algoritmos](../README.md) › [Memoria, tipos y recursión](README.md)

# Árbol de llamadas (`call_tree`)

Dibuja el árbol de llamadas de una función recursiva con lo que devuelve cada una. Sirve para
seguir la ejecución (`showOrder` numera el orden), para mostrar subproblemas repetidos
(`highlight`) y para resolver una recurrencia sumando el costo de cada nivel.

## Campos

| Campo | Tipo | Valores y valor por defecto | Descripción |
| :--- | :--- | :--- | :--- |
| `type` | texto | `call_tree` | Obligatorio. |
| `root` | objeto |  | La llamada inicial: `call`, `returns`, `children` (las llamadas que hace) y `blank`. Hasta 40 llamadas. |
| `showOrder` | sí/no |  | Numera las llamadas en el orden en que se hacen. |
| `highlight` | lista |  | Llamadas resaltadas, por su texto (por ejemplo `["fib(1)"]`). |
| `levelCosts` | lista |  | El costo de cada nivel, al costado. |
| `total` | texto |  | El costo total, debajo. |
| `blankReturns` | sí/no |  | Deja en blanco lo que devuelve cada llamada. |
| `blankCalls` | sí/no |  | Deja en blanco las llamadas. |
| `blankLevelCosts` | sí/no |  | Deja en blanco los costos por nivel. |
| `blankTotal` | sí/no |  | Deja en blanco el total. |

Además, los campos comunes a todos los tipos: `title`, `subtitle`, `id`, `inkSaver` y `theme` (ver [Campos comunes](../../campos-comunes.md)).

## Modo ejercicio

Campos que dejan casilleros para completar: `blankReturns`, `blankCalls`, `blankLevelCosts`, `blankTotal`.

## Ejemplos

### Árbol de llamadas fib(3)

```yaml
type: call_tree
title: Árbol de llamadas fib(3)
showOrder: true
levelCosts: ["1", "2", "2"]
total: "2^n"
root:
  call: "fib(3)"
  returns: 2
  children:
    - call: "fib(2)"
      returns: 1
      children:
        - {call: "fib(1)", returns: 1}
        - {call: "fib(0)", returns: 0}
    - {call: "fib(1)", returns: 1}
```

[![Árbol de llamadas fib(3)](img/ks_cs_call_tree_fib3_costos.svg)](https://ingcom-unrn.github.io/dibu-web/#z=jY5BCsIwEEX3OcWQlQWFtu6y9QAeQFRSM9LC2Mhkqoh04dm8mGkr2i4EZxHCz3s_I7czGjhYor0wopJKKAbPBxeewCEQ2ZN1NsCxKmbLRIXSX9fskA0IN6gIL0grHyQY2OhMz0Hnw7FV4sWSifddrRV7L0ZB_1fMhjodA0ZpuI563r2WFTnGuiMBFmM67-luPkb2DqbWYN5HapbElb5W-wNLJ1jaqj-aXg)

[Abrir en el editor](https://ingcom-unrn.github.io/dibu-web/#z=jY5BCsIwEEX3OcWQlQWFtu6y9QAeQFRSM9LC2Mhkqoh04dm8mGkr2i4EZxHCz3s_I7czGjhYor0wopJKKAbPBxeewCEQ2ZN1NsCxKmbLRIXSX9fskA0IN6gIL0grHyQY2OhMz0Hnw7FV4sWSifddrRV7L0ZB_1fMhjodA0ZpuI563r2WFTnGuiMBFmM67-luPkb2DqbWYN5HapbElb5W-wNLJ1jaqj-aXg) para explorarlo y modificarlo.

### Árbol de llamadas (retornos en blanco)

```yaml
type: call_tree
title: Árbol de llamadas (retornos en blanco)
blankReturns: true
blankLevelCosts: true
levelCosts: ["n", "2n"]
total: "n^2"
root:
  call: "f(2)"
  returns: 2
  children:
    - {call: "f(1)", returns: 1}
    - {call: "f(0)", returns: 0}
inkSaver: false
```

[![Árbol de llamadas (retornos en blanco)](img/ks_cs_call_tree_blank.svg)](https://ingcom-unrn.github.io/dibu-web/#z=ZY6xCsJQDEX39xXhTS0otB3f6uqko6i82ojFmEheFEQ6-G3-mK2iKE6XnBwu1y5HDLCJRGtTRGetUQ_uN62FoEEgiofYxASZoomyJECGmiJvJHdD7mdoJ-UUwPSELzTFM9JEkr0pfYGFZz8CX7FfOhOLFMDzqvJORSw4eM7p2Tarct-f-u6vht-upUaRBw9gDNePXOZ968ctuz-h-BGKzrW8n8czaoBtpITuAQ)

[Abrir en el editor](https://ingcom-unrn.github.io/dibu-web/#z=ZY6xCsJQDEX39xXhTS0otB3f6uqko6i82ojFmEheFEQ6-G3-mK2iKE6XnBwu1y5HDLCJRGtTRGetUQ_uN62FoEEgiofYxASZoomyJECGmiJvJHdD7mdoJ-UUwPSELzTFM9JEkr0pfYGFZz8CX7FfOhOLFMDzqvJORSw4eM7p2Tarct-f-u6vht-upUaRBw9gDNePXOZ968ctuz-h-BGKzrW8n8czaoBtpITuAQ) para explorarlo y modificarlo.

## Ver también

[`call_stack`](call_stack.md), [`struct`](struct.md), [`object_heap`](object_heap.md), [`adt`](adt.md)
