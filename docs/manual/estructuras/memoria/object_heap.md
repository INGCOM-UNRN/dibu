[Manual](../../README.md) › [Estructuras de datos y algoritmos](../README.md) › [Memoria, tipos y recursión](README.md)

# Stack y heap (objetos) (`object_heap`)

Dibuja los marcos activos en el stack (del tope, arriba, a `main`) y los objetos en el heap,
con una flecha por cada referencia. Cada objeto tiene un ID: dos referencias al mismo ID llegan
a la misma caja, y dos objetos con el mismo contenido son cajas distintas. Las referencias
entre objetos (y a sí mismos) rodean el heap por la derecha. Con `snapshots`, el estado en
varios instantes, para mostrar copia, mutación y reasignación de referencias: un ID que sigue
en el instante siguiente queda en el mismo lugar, así se ve cuál flecha cambió.

## Qué calcula

- Qué objetos son alcanzables desde el stack, siguiendo las referencias; los demás se marcan como inalcanzables.

## Campos

| Campo | Tipo | Valores y valor por defecto | Descripción |
| :--- | :--- | :--- | :--- |
| `type` | texto | `object_heap` | Obligatorio. |
| `frames` | lista |  | Los marcos, de `main` al tope. Cada uno con `function` y `params` y `vars`: listas de `{name, type, value}` o `{name, type, ref}` (`ref: null` es una referencia nula), o `{x: 3}` para valores. |
| `objects` | lista |  | Los objetos del heap. Cada uno con `id`, `type` y uno de `fields` (como las variables), `elements` (las celdas de un arreglo: valores sueltos, `{value}` o `{ref}`) o `value` (el contenido, como un `String`). `group` los agrupa, como el String Pool. |
| `snapshots` | lista |  | Varios instantes (hasta 6), cada uno con `label`, `frames` y `objects`. Un mismo ID en dos instantes es el mismo objeto. |
| `direction` |  |  | Con `snapshots`, `column` (por omisión) los apila y `row` los pone lado a lado. |
| `variant` | texto |  | `java` (por omisión) o `python`: en Python, los nombres se agrupan por ámbito (`<módulo>`, las funciones activas), una referencia nula es `None` (un valor rotulado, no una flecha sin destino) y los valores se escriben como en Python (`'texto'`, `True`). |
| `zones` |  |  | Los títulos del stack y del heap (`{stack, heap}`); `false` quita los recuadros. |
| `markUnreachable` |  |  | Marca los objetos inalcanzables desde el stack (por omisión, sí). |
| `blankValues` | sí/no |  | Deja los valores en blanco. |
| `blankRefs` | sí/no |  | Deja las referencias en blanco, sin flechas, para dibujarlas. |

Además, los campos comunes a todos los tipos: `title`, `subtitle`, `id`, `inkSaver` y `theme` (ver [Campos comunes](../../campos-comunes.md)).

## Modo ejercicio

Con `blankRefs`, las cajas de las referencias quedan vacías y sin flechas, para que se dibujen;
con `blankValues`, los valores. Cada variable, campo o celda acepta además `blank: true`.

Campos que dejan casilleros para completar: `blankValues`, `blankRefs`.

## Ejemplos

### Dos variables comparten un arreglo

```yaml
type: object_heap
title: Dos variables comparten un arreglo
frames:
  - function: main
    vars:
      - {name: a, type: "int[]", ref: A}
      - {name: b, type: "int[]", ref: A}
      - {name: copia, type: "int[]", ref: B}
      - {name: cantidad, type: int, value: 3}
objects:
  - id: A
    type: "int[]"
    elements: [{value: 1}, {value: 2}, {value: 3}]
  - id: B
    type: "int[]"
    elements: [1, 2, 3]
```

[![Dos variables comparten un arreglo](img/ks_cs_object_heap_alias.svg)](https://ingcom-unrn.github.io/dibu-web/#z=jZDBjsIwDETv-YoR53CA3nID7V-gCrmtywalTpW6SAj13zd0YbULQktOo_Hz2I6ee3aI1ZFr3X8y9Ua9hmx9xAEnSp6qwAPq2PWUlAWjgFLiQ4imTdTx4AywRDtKrT6KQ0desoVr91y8viUukmEHstB55MKL7sqFReLWYTM9gtW7YB17_yJ1-wyTqG-oufMZt3nRMGZdTOb7H24n-SaPmwP-ZM8OB-5YMond5da-mizuev1LF1P5E7f9P25lsbYoSvMF)

[Abrir en el editor](https://ingcom-unrn.github.io/dibu-web/#z=jZDBjsIwDETv-YoR53CA3nID7V-gCrmtywalTpW6SAj13zd0YbULQktOo_Hz2I6ee3aI1ZFr3X8y9Ua9hmx9xAEnSp6qwAPq2PWUlAWjgFLiQ4imTdTx4AywRDtKrT6KQ0desoVr91y8viUukmEHstB55MKL7sqFReLWYTM9gtW7YB17_yJ1-wyTqG-oufMZt3nRMGZdTOb7H24n-SaPmwP-ZM8OB-5YMond5da-mizuev1LF1P5E7f9P25lsbYoSvMF) para explorarlo y modificarlo.

### Instanciación, this y String Pool

```yaml
type: object_heap
title: "Instanciación, this y String Pool"
frames:
  - function: main
    vars:
      - {name: juan, type: Persona, ref: P1}
      - {name: maria, type: Persona, ref: P2}
  - function: saludar
    params:
      - {name: this, type: Persona, ref: P1}
objects:
  - id: P1
    type: Persona
    fields:
      - {name: nombre, type: String, ref: S1}
      - {name: edad, type: int, value: 30}
  - id: P2
    type: Persona
    fields:
      - {name: nombre, type: String, ref: S1}
      - {name: edad, type: int, value: 25}
  - {id: S1, type: String, value: "Hola", group: String Pool}
inkSaver: false
```

[![Instanciación, this y String Pool](img/ks_cs_object_heap_this_pool.svg)](https://ingcom-unrn.github.io/dibu-web/#z=xVLLasMwELzrKxafHWhcetEXtLeAP6Bs7FWyqbwykhwIwV-VT8iPVX6EtjY5VydpdpiZHRQvLWlw-xNV8fNI2KrI0SYo-5AQUSrGiu83ySEeOcAFyuhZDrBzzmbKeGwoaAWwAdNJFdmJhgZZEgRwRj8Oh7OBqySyhlOHg9rouyMfnGAOnkx6bfsluUHP-IRd9AvfgLar0Y8aLaZoa_NhiefmUw3zPlwP4Cjwhz8ihsnWa3lxzd7Tw2CqatYv18tRjfWDyxLz1Jft0v31pf9JUPxHguJtSnAdIpTbpdzMyt6dxSyHg3ddq3__jF6xfJV4Jq_BoA2kvgE)

[Abrir en el editor](https://ingcom-unrn.github.io/dibu-web/#z=xVLLasMwELzrKxafHWhcetEXtLeAP6Bs7FWyqbwykhwIwV-VT8iPVX6EtjY5VydpdpiZHRQvLWlw-xNV8fNI2KrI0SYo-5AQUSrGiu83ySEeOcAFyuhZDrBzzmbKeGwoaAWwAdNJFdmJhgZZEgRwRj8Oh7OBqySyhlOHg9rouyMfnGAOnkx6bfsluUHP-IRd9AvfgLar0Y8aLaZoa_NhiefmUw3zPlwP4Cjwhz8ihsnWa3lxzd7Tw2CqatYv18tRjfWDyxLz1Jft0v31pf9JUPxHguJtSnAdIpTbpdzMyt6dxSyHg3ddq3__jF6xfJV4Jq_BoA2kvgE) para explorarlo y modificarlo.

### Reasignación de un parámetro

```yaml
type: object_heap
title: Reasignación de un parámetro
snapshots:
  - label: "Al entrar a modificar(numeros)"
    frames:
      - function: main
        vars: [{name: numeros, type: "int[]", ref: A}]
      - function: modificar
        params: [{name: arr, type: "int[]", ref: A}]
    objects:
      - {id: A, type: "int[]", elements: [1, 2, 3]}
  - label: "Después de arr = new int[] {7, 8}"
    frames:
      - function: main
        vars: [{name: numeros, type: "int[]", ref: A}]
      - function: modificar
        params: [{name: arr, type: "int[]", ref: B}]
    objects:
      - {id: A, type: "int[]", elements: [1, 2, 3]}
      - {id: B, type: "int[]", elements: [7, 8]}
```

[![Reasignación de un parámetro](img/ks_cs_object_heap_instantes.svg)](https://ingcom-unrn.github.io/dibu-web/#z=1ZAxTsQwEEX7nOIrFUjZAihAlih2xQloVxGaTSaskT2ObAeEohyGkpoj5GI4YUFBoK1ocDme_2bmxeeWFdzugat4t2dqs6ijSaVbpqDvhSo9vglqRidoyY8vlqN3WRBqw97FoDJgBUM7Ngr52oAlevIgWFfrRlfkT6Sz7F04zVMv0HiyPOemt0LTSRW1EwVLWg5l4JF8UNj2kroVDogCcV441xK3ZV7Ac6OwHsrfaJ_zv5Bpf7ILKHl_HPjhZbFrr-v0-yPEhm26e0KfFTgvcFEO37zccGi78TVMItNUXEP4CXMc_WWBq-HfuNn8jZtFZnMsM8kph-wd)

[Abrir en el editor](https://ingcom-unrn.github.io/dibu-web/#z=1ZAxTsQwEEX7nOIrFUjZAihAlih2xQloVxGaTSaskT2ObAeEohyGkpoj5GI4YUFBoK1ocDme_2bmxeeWFdzugat4t2dqs6ijSaVbpqDvhSo9vglqRidoyY8vlqN3WRBqw97FoDJgBUM7Ngr52oAlevIgWFfrRlfkT6Sz7F04zVMv0HiyPOemt0LTSRW1EwVLWg5l4JF8UNj2kroVDogCcV441xK3ZV7Ac6OwHsrfaJ_zv5Bpf7ILKHl_HPjhZbFrr-v0-yPEhm26e0KfFTgvcFEO37zccGi78TVMItNUXEP4CXMc_WWBq-HfuNn8jZtFZnMsM8kph-wd) para explorarlo y modificarlo.

### Tras p = null (para completar las referencias)

```yaml
type: object_heap
title: "Tras p = null (para completar las referencias)"
frames:
  - function: main
    vars: [{name: p, type: Nodo, ref: null}, {name: lista, type: "Nodo[]", ref: L}]
objects:
  - id: L
    type: "Nodo[]"
    elements: [{ref: N2}, {ref: null}]
  - id: N1
    type: Nodo
    fields: [{name: dato, type: int, value: 1}, {name: sig, type: Nodo, ref: N2}]
  - id: N2
    type: Nodo
    fields: [{name: dato, type: int, value: 2}, {name: sig, type: Nodo, ref: N2}]
blankRefs: true
```

[![Tras p = null (para completar las referencias)](img/ks_cs_object_heap_alcanzable_blank.svg)](https://ingcom-unrn.github.io/dibu-web/#z=pY_BasMwEETv-orFpxTcQ3wU5A-KD6W3YMrGXrVq15KQ1oVQ8u-VFadxIYdAb6vR7LxZOQbS4A8f1MvrO2FQYoWzVL1ETBBgB25ihk3AiND7MTAJRuD8GclQJNdbTA-VMhFHSloBPIKZXC_WOw0jWpclgC-MScP-22WXhlCDFHLrB1_PSbpwTjUsDrZJ8OKqZtu-qxbn06lT58oLzw5ZLJi__iIR00hOCr2st82MuTK734x2uwqZM8rTWOJhVX5A8Zdm1kmdb-Mpz9tr-2TfblyYwStW8x9WcxfrwOg-n8nkQIkTqR8)

[Abrir en el editor](https://ingcom-unrn.github.io/dibu-web/#z=pY_BasMwEETv-orFpxTcQ3wU5A-KD6W3YMrGXrVq15KQ1oVQ8u-VFadxIYdAb6vR7LxZOQbS4A8f1MvrO2FQYoWzVL1ETBBgB25ihk3AiND7MTAJRuD8GclQJNdbTA-VMhFHSloBPIKZXC_WOw0jWpclgC-MScP-22WXhlCDFHLrB1_PSbpwTjUsDrZJ8OKqZtu-qxbn06lT58oLzw5ZLJi__iIR00hOCr2st82MuTK734x2uwqZM8rTWOJhVX5A8Zdm1kmdb-Mpz9tr-2TfblyYwStW8x9WcxfrwOg-n8nkQIkTqR8) para explorarlo y modificarlo.

### Copia superficial: copia = original.copy()

```yaml
type: object_heap
variant: python
title: "Copia superficial: copia = original.copy()"
frames:
  - function: "<módulo>"
    vars:
      - {name: original, ref: A}
      - {name: copia, ref: B}
      - {name: resultado, value: null}
objects:
  - id: A
    type: list
    elements: [{ref: I}, "x"]
  - id: B
    type: list
    elements: [{ref: I}, "x"]
  - id: I
    type: list
    elements: [{value: 4}, {value: 7}]
```

[![Copia superficial: copia = original.copy()](img/ks_cs_object_heap_python_copia.svg)](https://ingcom-unrn.github.io/dibu-web/#z=nU_NSgMxEL7nKYacKmw9FYSggu2pzyBF4u6sHZlNQnYiLmWfykfwxRzj2oMeBHOafL8zMiV0EB-fsZWHI_pkXnwmH8RBmuQYgxESVondxUQexpIw99SSZwdthW4gZnqi4PlSgWl1YU2f_YCjMwBr6EtohWLQiOvh_a0rHG-tMgDaVDWfbw2noB53zmogY-_gbv4pqKULu_3FZhwLi-9io_FcFAmFeTZfFy4rUafB1Sn1fKZR6hcZBwwqg_tTLdjPDdhXezjbtv-z7f-yLctu1Pg9X80H8wE)

[Abrir en el editor](https://ingcom-unrn.github.io/dibu-web/#z=nU_NSgMxEL7nKYacKmw9FYSggu2pzyBF4u6sHZlNQnYiLmWfykfwxRzj2oMeBHOafL8zMiV0EB-fsZWHI_pkXnwmH8RBmuQYgxESVondxUQexpIw99SSZwdthW4gZnqi4PlSgWl1YU2f_YCjMwBr6EtohWLQiOvh_a0rHG-tMgDaVDWfbw2noB53zmogY-_gbv4pqKULu_3FZhwLi-9io_FcFAmFeTZfFy4rUafB1Sn1fKZR6hcZBwwqg_tTLdjPDdhXezjbtv-z7f-yLctu1Pg9X80H8wE) para explorarlo y modificarlo.

### def agregar(lista): lista.append(9); lista = [0] (para completar)

```yaml
type: object_heap
variant: python
title: "def agregar(lista): lista.append(9); lista = [0] (para completar)"
blankRefs: true
snapshots:
  - label: "Después de lista.append(9)"
    frames:
      - function: "<módulo>"
        vars: [{name: lista, ref: A}]
      - function: agregar
        params: [{name: lista, ref: A}]
    objects:
      - {id: A, type: list, elements: [1, 2, 9]}
  - label: "Después de lista = [0]"
    frames:
      - function: "<módulo>"
        vars: [{name: lista, ref: A}]
      - function: agregar
        params: [{name: lista, ref: B}]
    objects:
      - {id: A, type: list, elements: [1, 2, 9]}
      - {id: B, type: list, elements: [0]}
```

[![def agregar(lista): lista.append(9); lista = [0] (para completar)](img/ks_cs_object_heap_python_parametro_blank.svg)](https://ingcom-unrn.github.io/dibu-web/#z=zZBNasNADIX3PsXDKxucknYX9wcaeoJugymyLcdux-NhRi6E4AN13SPkYh3boYTSn00WnY1gpO89PcnOcIouf-ZCnmomE7ySbUhLCrOTutOBNKL8SFhyBdpa3pKNVOOE4hRTvSBjWJfRKr6eP3CLzTJDZMgSiq41ioVsHAa5Iv3yyJVLIbbnwGkyru7EpQGwgKKclXd6YGf6w5tDyV8dQj8IVJZanqDxLVD1upCm0569aQ_vZa-6u_DYBXwgb7jZaw8dV05guUpxP2TfaBxDfvJjjPYPhfmAJyvtm9J3E8h03xFJwIpb1jJKXSa4SrDKht-Dz5f8b6HXZwh9Aqx_BJbZEHwA)

[Abrir en el editor](https://ingcom-unrn.github.io/dibu-web/#z=zZBNasNADIX3PsXDKxucknYX9wcaeoJugymyLcdux-NhRi6E4AN13SPkYh3boYTSn00WnY1gpO89PcnOcIouf-ZCnmomE7ySbUhLCrOTutOBNKL8SFhyBdpa3pKNVOOE4hRTvSBjWJfRKr6eP3CLzTJDZMgSiq41ioVsHAa5Iv3yyJVLIbbnwGkyru7EpQGwgKKclXd6YGf6w5tDyV8dQj8IVJZanqDxLVD1upCm0569aQ_vZa-6u_DYBXwgb7jZaw8dV05guUpxP2TfaBxDfvJjjPYPhfmAJyvtm9J3E8h03xFJwIpb1jJKXSa4SrDKht-Dz5f8b6HXZwh9Aqx_BJbZEHwA) para explorarlo y modificarlo.

## Notas

- Hasta 8 marcos, 16 objetos y 12 variables, campos o celdas por marco u objeto.
- Una referencia a un ID que no está en `objects`, un ID repetido o `value` junto con `ref` son errores.
- Para la pila de llamadas de C, con direcciones, ver [`call_stack`](call_stack.md); para la matriz de Java, [`matrix`](../lineales/matrix.md) con `variant: java`.

## Ver también

[`call_stack`](call_stack.md), [`call_tree`](call_tree.md), [`struct`](struct.md), [`adt`](adt.md)
