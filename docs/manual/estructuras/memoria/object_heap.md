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

[![Dos variables comparten un arreglo](img/ks_cs_object_heap_alias.svg)](https://ingcom-unrn.github.io/dibu-web/#t=dHlwZTogb2JqZWN0X2hlYXAKdGl0bGU6IERvcyB2YXJpYWJsZXMgY29tcGFydGVuIHVuIGFycmVnbG8KZnJhbWVzOgogIC0gZnVuY3Rpb246IG1haW4KICAgIHZhcnM6CiAgICAgIC0ge25hbWU6IGEsIHR5cGU6ICJpbnRbXSIsIHJlZjogQX0KICAgICAgLSB7bmFtZTogYiwgdHlwZTogImludFtdIiwgcmVmOiBBfQogICAgICAtIHtuYW1lOiBjb3BpYSwgdHlwZTogImludFtdIiwgcmVmOiBCfQogICAgICAtIHtuYW1lOiBjYW50aWRhZCwgdHlwZTogaW50LCB2YWx1ZTogM30Kb2JqZWN0czoKICAtIGlkOiBBCiAgICB0eXBlOiAiaW50W10iCiAgICBlbGVtZW50czogW3t2YWx1ZTogMX0sIHt2YWx1ZTogMn0sIHt2YWx1ZTogM31dCiAgLSBpZDogQgogICAgdHlwZTogImludFtdIgogICAgZWxlbWVudHM6IFsxLCAyLCAzXQo)

[Abrir en el editor](https://ingcom-unrn.github.io/dibu-web/#t=dHlwZTogb2JqZWN0X2hlYXAKdGl0bGU6IERvcyB2YXJpYWJsZXMgY29tcGFydGVuIHVuIGFycmVnbG8KZnJhbWVzOgogIC0gZnVuY3Rpb246IG1haW4KICAgIHZhcnM6CiAgICAgIC0ge25hbWU6IGEsIHR5cGU6ICJpbnRbXSIsIHJlZjogQX0KICAgICAgLSB7bmFtZTogYiwgdHlwZTogImludFtdIiwgcmVmOiBBfQogICAgICAtIHtuYW1lOiBjb3BpYSwgdHlwZTogImludFtdIiwgcmVmOiBCfQogICAgICAtIHtuYW1lOiBjYW50aWRhZCwgdHlwZTogaW50LCB2YWx1ZTogM30Kb2JqZWN0czoKICAtIGlkOiBBCiAgICB0eXBlOiAiaW50W10iCiAgICBlbGVtZW50czogW3t2YWx1ZTogMX0sIHt2YWx1ZTogMn0sIHt2YWx1ZTogM31dCiAgLSBpZDogQgogICAgdHlwZTogImludFtdIgogICAgZWxlbWVudHM6IFsxLCAyLCAzXQo) para explorarlo y modificarlo.

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

[![Instanciación, this y String Pool](img/ks_cs_object_heap_this_pool.svg)](https://ingcom-unrn.github.io/dibu-web/#t=dHlwZTogb2JqZWN0X2hlYXAKdGl0bGU6ICJJbnN0YW5jaWFjacOzbiwgdGhpcyB5IFN0cmluZyBQb29sIgpmcmFtZXM6CiAgLSBmdW5jdGlvbjogbWFpbgogICAgdmFyczoKICAgICAgLSB7bmFtZToganVhbiwgdHlwZTogUGVyc29uYSwgcmVmOiBQMX0KICAgICAgLSB7bmFtZTogbWFyaWEsIHR5cGU6IFBlcnNvbmEsIHJlZjogUDJ9CiAgLSBmdW5jdGlvbjogc2FsdWRhcgogICAgcGFyYW1zOgogICAgICAtIHtuYW1lOiB0aGlzLCB0eXBlOiBQZXJzb25hLCByZWY6IFAxfQpvYmplY3RzOgogIC0gaWQ6IFAxCiAgICB0eXBlOiBQZXJzb25hCiAgICBmaWVsZHM6CiAgICAgIC0ge25hbWU6IG5vbWJyZSwgdHlwZTogU3RyaW5nLCByZWY6IFMxfQogICAgICAtIHtuYW1lOiBlZGFkLCB0eXBlOiBpbnQsIHZhbHVlOiAzMH0KICAtIGlkOiBQMgogICAgdHlwZTogUGVyc29uYQogICAgZmllbGRzOgogICAgICAtIHtuYW1lOiBub21icmUsIHR5cGU6IFN0cmluZywgcmVmOiBTMX0KICAgICAgLSB7bmFtZTogZWRhZCwgdHlwZTogaW50LCB2YWx1ZTogMjV9CiAgLSB7aWQ6IFMxLCB0eXBlOiBTdHJpbmcsIHZhbHVlOiAiSG9sYSIsIGdyb3VwOiBTdHJpbmcgUG9vbH0KaW5rU2F2ZXI6IGZhbHNlCg)

[Abrir en el editor](https://ingcom-unrn.github.io/dibu-web/#t=dHlwZTogb2JqZWN0X2hlYXAKdGl0bGU6ICJJbnN0YW5jaWFjacOzbiwgdGhpcyB5IFN0cmluZyBQb29sIgpmcmFtZXM6CiAgLSBmdW5jdGlvbjogbWFpbgogICAgdmFyczoKICAgICAgLSB7bmFtZToganVhbiwgdHlwZTogUGVyc29uYSwgcmVmOiBQMX0KICAgICAgLSB7bmFtZTogbWFyaWEsIHR5cGU6IFBlcnNvbmEsIHJlZjogUDJ9CiAgLSBmdW5jdGlvbjogc2FsdWRhcgogICAgcGFyYW1zOgogICAgICAtIHtuYW1lOiB0aGlzLCB0eXBlOiBQZXJzb25hLCByZWY6IFAxfQpvYmplY3RzOgogIC0gaWQ6IFAxCiAgICB0eXBlOiBQZXJzb25hCiAgICBmaWVsZHM6CiAgICAgIC0ge25hbWU6IG5vbWJyZSwgdHlwZTogU3RyaW5nLCByZWY6IFMxfQogICAgICAtIHtuYW1lOiBlZGFkLCB0eXBlOiBpbnQsIHZhbHVlOiAzMH0KICAtIGlkOiBQMgogICAgdHlwZTogUGVyc29uYQogICAgZmllbGRzOgogICAgICAtIHtuYW1lOiBub21icmUsIHR5cGU6IFN0cmluZywgcmVmOiBTMX0KICAgICAgLSB7bmFtZTogZWRhZCwgdHlwZTogaW50LCB2YWx1ZTogMjV9CiAgLSB7aWQ6IFMxLCB0eXBlOiBTdHJpbmcsIHZhbHVlOiAiSG9sYSIsIGdyb3VwOiBTdHJpbmcgUG9vbH0KaW5rU2F2ZXI6IGZhbHNlCg) para explorarlo y modificarlo.

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

[![Reasignación de un parámetro](img/ks_cs_object_heap_instantes.svg)](https://ingcom-unrn.github.io/dibu-web/#t=dHlwZTogb2JqZWN0X2hlYXAKdGl0bGU6IFJlYXNpZ25hY2nDs24gZGUgdW4gcGFyw6FtZXRybwpzbmFwc2hvdHM6CiAgLSBsYWJlbDogIkFsIGVudHJhciBhIG1vZGlmaWNhcihudW1lcm9zKSIKICAgIGZyYW1lczoKICAgICAgLSBmdW5jdGlvbjogbWFpbgogICAgICAgIHZhcnM6IFt7bmFtZTogbnVtZXJvcywgdHlwZTogImludFtdIiwgcmVmOiBBfV0KICAgICAgLSBmdW5jdGlvbjogbW9kaWZpY2FyCiAgICAgICAgcGFyYW1zOiBbe25hbWU6IGFyciwgdHlwZTogImludFtdIiwgcmVmOiBBfV0KICAgIG9iamVjdHM6CiAgICAgIC0ge2lkOiBBLCB0eXBlOiAiaW50W10iLCBlbGVtZW50czogWzEsIDIsIDNdfQogIC0gbGFiZWw6ICJEZXNwdcOpcyBkZSBhcnIgPSBuZXcgaW50W10gezcsIDh9IgogICAgZnJhbWVzOgogICAgICAtIGZ1bmN0aW9uOiBtYWluCiAgICAgICAgdmFyczogW3tuYW1lOiBudW1lcm9zLCB0eXBlOiAiaW50W10iLCByZWY6IEF9XQogICAgICAtIGZ1bmN0aW9uOiBtb2RpZmljYXIKICAgICAgICBwYXJhbXM6IFt7bmFtZTogYXJyLCB0eXBlOiAiaW50W10iLCByZWY6IEJ9XQogICAgb2JqZWN0czoKICAgICAgLSB7aWQ6IEEsIHR5cGU6ICJpbnRbXSIsIGVsZW1lbnRzOiBbMSwgMiwgM119CiAgICAgIC0ge2lkOiBCLCB0eXBlOiAiaW50W10iLCBlbGVtZW50czogWzcsIDhdfQo)

[Abrir en el editor](https://ingcom-unrn.github.io/dibu-web/#t=dHlwZTogb2JqZWN0X2hlYXAKdGl0bGU6IFJlYXNpZ25hY2nDs24gZGUgdW4gcGFyw6FtZXRybwpzbmFwc2hvdHM6CiAgLSBsYWJlbDogIkFsIGVudHJhciBhIG1vZGlmaWNhcihudW1lcm9zKSIKICAgIGZyYW1lczoKICAgICAgLSBmdW5jdGlvbjogbWFpbgogICAgICAgIHZhcnM6IFt7bmFtZTogbnVtZXJvcywgdHlwZTogImludFtdIiwgcmVmOiBBfV0KICAgICAgLSBmdW5jdGlvbjogbW9kaWZpY2FyCiAgICAgICAgcGFyYW1zOiBbe25hbWU6IGFyciwgdHlwZTogImludFtdIiwgcmVmOiBBfV0KICAgIG9iamVjdHM6CiAgICAgIC0ge2lkOiBBLCB0eXBlOiAiaW50W10iLCBlbGVtZW50czogWzEsIDIsIDNdfQogIC0gbGFiZWw6ICJEZXNwdcOpcyBkZSBhcnIgPSBuZXcgaW50W10gezcsIDh9IgogICAgZnJhbWVzOgogICAgICAtIGZ1bmN0aW9uOiBtYWluCiAgICAgICAgdmFyczogW3tuYW1lOiBudW1lcm9zLCB0eXBlOiAiaW50W10iLCByZWY6IEF9XQogICAgICAtIGZ1bmN0aW9uOiBtb2RpZmljYXIKICAgICAgICBwYXJhbXM6IFt7bmFtZTogYXJyLCB0eXBlOiAiaW50W10iLCByZWY6IEJ9XQogICAgb2JqZWN0czoKICAgICAgLSB7aWQ6IEEsIHR5cGU6ICJpbnRbXSIsIGVsZW1lbnRzOiBbMSwgMiwgM119CiAgICAgIC0ge2lkOiBCLCB0eXBlOiAiaW50W10iLCBlbGVtZW50czogWzcsIDhdfQo) para explorarlo y modificarlo.

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

[![Tras p = null (para completar las referencias)](img/ks_cs_object_heap_alcanzable_blank.svg)](https://ingcom-unrn.github.io/dibu-web/#t=dHlwZTogb2JqZWN0X2hlYXAKdGl0bGU6ICJUcmFzIHAgPSBudWxsIChwYXJhIGNvbXBsZXRhciBsYXMgcmVmZXJlbmNpYXMpIgpmcmFtZXM6CiAgLSBmdW5jdGlvbjogbWFpbgogICAgdmFyczogW3tuYW1lOiBwLCB0eXBlOiBOb2RvLCByZWY6IG51bGx9LCB7bmFtZTogbGlzdGEsIHR5cGU6ICJOb2RvW10iLCByZWY6IEx9XQpvYmplY3RzOgogIC0gaWQ6IEwKICAgIHR5cGU6ICJOb2RvW10iCiAgICBlbGVtZW50czogW3tyZWY6IE4yfSwge3JlZjogbnVsbH1dCiAgLSBpZDogTjEKICAgIHR5cGU6IE5vZG8KICAgIGZpZWxkczogW3tuYW1lOiBkYXRvLCB0eXBlOiBpbnQsIHZhbHVlOiAxfSwge25hbWU6IHNpZywgdHlwZTogTm9kbywgcmVmOiBOMn1dCiAgLSBpZDogTjIKICAgIHR5cGU6IE5vZG8KICAgIGZpZWxkczogW3tuYW1lOiBkYXRvLCB0eXBlOiBpbnQsIHZhbHVlOiAyfSwge25hbWU6IHNpZywgdHlwZTogTm9kbywgcmVmOiBOMn1dCmJsYW5rUmVmczogdHJ1ZQo)

[Abrir en el editor](https://ingcom-unrn.github.io/dibu-web/#t=dHlwZTogb2JqZWN0X2hlYXAKdGl0bGU6ICJUcmFzIHAgPSBudWxsIChwYXJhIGNvbXBsZXRhciBsYXMgcmVmZXJlbmNpYXMpIgpmcmFtZXM6CiAgLSBmdW5jdGlvbjogbWFpbgogICAgdmFyczogW3tuYW1lOiBwLCB0eXBlOiBOb2RvLCByZWY6IG51bGx9LCB7bmFtZTogbGlzdGEsIHR5cGU6ICJOb2RvW10iLCByZWY6IEx9XQpvYmplY3RzOgogIC0gaWQ6IEwKICAgIHR5cGU6ICJOb2RvW10iCiAgICBlbGVtZW50czogW3tyZWY6IE4yfSwge3JlZjogbnVsbH1dCiAgLSBpZDogTjEKICAgIHR5cGU6IE5vZG8KICAgIGZpZWxkczogW3tuYW1lOiBkYXRvLCB0eXBlOiBpbnQsIHZhbHVlOiAxfSwge25hbWU6IHNpZywgdHlwZTogTm9kbywgcmVmOiBOMn1dCiAgLSBpZDogTjIKICAgIHR5cGU6IE5vZG8KICAgIGZpZWxkczogW3tuYW1lOiBkYXRvLCB0eXBlOiBpbnQsIHZhbHVlOiAyfSwge25hbWU6IHNpZywgdHlwZTogTm9kbywgcmVmOiBOMn1dCmJsYW5rUmVmczogdHJ1ZQo) para explorarlo y modificarlo.

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

[![Copia superficial: copia = original.copy()](img/ks_cs_object_heap_python_copia.svg)](https://ingcom-unrn.github.io/dibu-web/#t=dHlwZTogb2JqZWN0X2hlYXAKdmFyaWFudDogcHl0aG9uCnRpdGxlOiAiQ29waWEgc3VwZXJmaWNpYWw6IGNvcGlhID0gb3JpZ2luYWwuY29weSgpIgpmcmFtZXM6CiAgLSBmdW5jdGlvbjogIjxtw7NkdWxvPiIKICAgIHZhcnM6CiAgICAgIC0ge25hbWU6IG9yaWdpbmFsLCByZWY6IEF9CiAgICAgIC0ge25hbWU6IGNvcGlhLCByZWY6IEJ9CiAgICAgIC0ge25hbWU6IHJlc3VsdGFkbywgdmFsdWU6IG51bGx9Cm9iamVjdHM6CiAgLSBpZDogQQogICAgdHlwZTogbGlzdAogICAgZWxlbWVudHM6IFt7cmVmOiBJfSwgIngiXQogIC0gaWQ6IEIKICAgIHR5cGU6IGxpc3QKICAgIGVsZW1lbnRzOiBbe3JlZjogSX0sICJ4Il0KICAtIGlkOiBJCiAgICB0eXBlOiBsaXN0CiAgICBlbGVtZW50czogW3t2YWx1ZTogNH0sIHt2YWx1ZTogN31dCg)

[Abrir en el editor](https://ingcom-unrn.github.io/dibu-web/#t=dHlwZTogb2JqZWN0X2hlYXAKdmFyaWFudDogcHl0aG9uCnRpdGxlOiAiQ29waWEgc3VwZXJmaWNpYWw6IGNvcGlhID0gb3JpZ2luYWwuY29weSgpIgpmcmFtZXM6CiAgLSBmdW5jdGlvbjogIjxtw7NkdWxvPiIKICAgIHZhcnM6CiAgICAgIC0ge25hbWU6IG9yaWdpbmFsLCByZWY6IEF9CiAgICAgIC0ge25hbWU6IGNvcGlhLCByZWY6IEJ9CiAgICAgIC0ge25hbWU6IHJlc3VsdGFkbywgdmFsdWU6IG51bGx9Cm9iamVjdHM6CiAgLSBpZDogQQogICAgdHlwZTogbGlzdAogICAgZWxlbWVudHM6IFt7cmVmOiBJfSwgIngiXQogIC0gaWQ6IEIKICAgIHR5cGU6IGxpc3QKICAgIGVsZW1lbnRzOiBbe3JlZjogSX0sICJ4Il0KICAtIGlkOiBJCiAgICB0eXBlOiBsaXN0CiAgICBlbGVtZW50czogW3t2YWx1ZTogNH0sIHt2YWx1ZTogN31dCg) para explorarlo y modificarlo.

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

[![def agregar(lista): lista.append(9); lista = [0] (para completar)](img/ks_cs_object_heap_python_parametro_blank.svg)](https://ingcom-unrn.github.io/dibu-web/#t=dHlwZTogb2JqZWN0X2hlYXAKdmFyaWFudDogcHl0aG9uCnRpdGxlOiAiZGVmIGFncmVnYXIobGlzdGEpOiBsaXN0YS5hcHBlbmQoOSk7IGxpc3RhID0gWzBdIChwYXJhIGNvbXBsZXRhcikiCmJsYW5rUmVmczogdHJ1ZQpzbmFwc2hvdHM6CiAgLSBsYWJlbDogIkRlc3B1w6lzIGRlIGxpc3RhLmFwcGVuZCg5KSIKICAgIGZyYW1lczoKICAgICAgLSBmdW5jdGlvbjogIjxtw7NkdWxvPiIKICAgICAgICB2YXJzOiBbe25hbWU6IGxpc3RhLCByZWY6IEF9XQogICAgICAtIGZ1bmN0aW9uOiBhZ3JlZ2FyCiAgICAgICAgcGFyYW1zOiBbe25hbWU6IGxpc3RhLCByZWY6IEF9XQogICAgb2JqZWN0czoKICAgICAgLSB7aWQ6IEEsIHR5cGU6IGxpc3QsIGVsZW1lbnRzOiBbMSwgMiwgOV19CiAgLSBsYWJlbDogIkRlc3B1w6lzIGRlIGxpc3RhID0gWzBdIgogICAgZnJhbWVzOgogICAgICAtIGZ1bmN0aW9uOiAiPG3Ds2R1bG8-IgogICAgICAgIHZhcnM6IFt7bmFtZTogbGlzdGEsIHJlZjogQX1dCiAgICAgIC0gZnVuY3Rpb246IGFncmVnYXIKICAgICAgICBwYXJhbXM6IFt7bmFtZTogbGlzdGEsIHJlZjogQn1dCiAgICBvYmplY3RzOgogICAgICAtIHtpZDogQSwgdHlwZTogbGlzdCwgZWxlbWVudHM6IFsxLCAyLCA5XX0KICAgICAgLSB7aWQ6IEIsIHR5cGU6IGxpc3QsIGVsZW1lbnRzOiBbMF19Cg)

[Abrir en el editor](https://ingcom-unrn.github.io/dibu-web/#t=dHlwZTogb2JqZWN0X2hlYXAKdmFyaWFudDogcHl0aG9uCnRpdGxlOiAiZGVmIGFncmVnYXIobGlzdGEpOiBsaXN0YS5hcHBlbmQoOSk7IGxpc3RhID0gWzBdIChwYXJhIGNvbXBsZXRhcikiCmJsYW5rUmVmczogdHJ1ZQpzbmFwc2hvdHM6CiAgLSBsYWJlbDogIkRlc3B1w6lzIGRlIGxpc3RhLmFwcGVuZCg5KSIKICAgIGZyYW1lczoKICAgICAgLSBmdW5jdGlvbjogIjxtw7NkdWxvPiIKICAgICAgICB2YXJzOiBbe25hbWU6IGxpc3RhLCByZWY6IEF9XQogICAgICAtIGZ1bmN0aW9uOiBhZ3JlZ2FyCiAgICAgICAgcGFyYW1zOiBbe25hbWU6IGxpc3RhLCByZWY6IEF9XQogICAgb2JqZWN0czoKICAgICAgLSB7aWQ6IEEsIHR5cGU6IGxpc3QsIGVsZW1lbnRzOiBbMSwgMiwgOV19CiAgLSBsYWJlbDogIkRlc3B1w6lzIGRlIGxpc3RhID0gWzBdIgogICAgZnJhbWVzOgogICAgICAtIGZ1bmN0aW9uOiAiPG3Ds2R1bG8-IgogICAgICAgIHZhcnM6IFt7bmFtZTogbGlzdGEsIHJlZjogQX1dCiAgICAgIC0gZnVuY3Rpb246IGFncmVnYXIKICAgICAgICBwYXJhbXM6IFt7bmFtZTogbGlzdGEsIHJlZjogQn1dCiAgICBvYmplY3RzOgogICAgICAtIHtpZDogQSwgdHlwZTogbGlzdCwgZWxlbWVudHM6IFsxLCAyLCA5XX0KICAgICAgLSB7aWQ6IEIsIHR5cGU6IGxpc3QsIGVsZW1lbnRzOiBbMF19Cg) para explorarlo y modificarlo.

## Notas

- Hasta 8 marcos, 16 objetos y 12 variables, campos o celdas por marco u objeto.
- Una referencia a un ID que no está en `objects`, un ID repetido o `value` junto con `ref` son errores.
- Para la pila de llamadas de C, con direcciones, ver [`call_stack`](call_stack.md); para la matriz de Java, [`matrix`](../lineales/matrix.md) con `variant: java`.

## Ver también

[`call_stack`](call_stack.md), [`call_tree`](call_tree.md), [`struct`](struct.md), [`adt`](adt.md)
