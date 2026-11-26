[Manual](../../README.md) › [Estructuras de datos y algoritmos](../README.md) › [Memoria, tipos y recursión](README.md)

# Tipo abstracto de datos (`adt`)

Dibuja un tipo abstracto de datos: el cliente, la tabla de operaciones con su clase
(constructor, modificador, observador…), su precondición y su poscondición, el invariante y
las representaciones posibles con sus campos.

## Campos

| Campo | Tipo | Valores y valor por defecto | Descripción |
| :--- | :--- | :--- | :--- |
| `type` | texto | `adt` | Obligatorio. |
| `name` | texto |  | Nombre del TAD. |
| `operations` | lista |  | Las operaciones: texto (la firma) o `{signature, kind, pre, post, blank}`. |
| `invariant` | texto |  | El invariante de la representación. |
| `representations` | lista |  | Las representaciones posibles, cada una `{name, fields}`. |
| `client` | lista |  | Código cliente que usa el TAD. |
| `blankOperations` | sí/no |  | Deja las operaciones en blanco. |
| `blankKinds` | sí/no |  | Deja en blanco la clase de cada operación. |

Además, los campos comunes a todos los tipos: `title`, `subtitle`, `id`, `inkSaver` y `theme` (ver [Campos comunes](../../campos-comunes.md)).

## Modo ejercicio

Campos que dejan casilleros para completar: `blankOperations`, `blankKinds`.

## Ejemplos

### Especificación TAD Pila

```yaml
type: adt
title: Especificación TAD Pila
name: Pila
blankKinds: true
operations:
  - {signature: "f()", kind: observador, pre: "p != NULL"}
  - {signature: "g()", post: "tope' > 0", blank: true}
client: ["crear();"]
invariant: "tope >= 0"
representations:
  - Arreglo
  - {name: Lista, fields: ["nodo_t *cabeza"]}
```

[![Especificación TAD Pila](img/ks_cs_adt_pila.svg)](https://ingcom-unrn.github.io/dibu-web/#z=ZY_BSgNBDIbv8xRxLraygueVFgr11EU86KkUyc5ml9DpzJCZFlT2qXwEX8y4iyB4-5P8H_-f8paoBuyKKVy8yoecyHHPDh1_fQZ43mzhiT2agCc9T7L1GI47Dl2uociZTEwkWDiGXBuAW_jIPAQsZ1HC9oulreCo9hpim0ku2EWpIE3XBFcreHxpGjv-R4cJTTEXHYqmXMMa7nQ1NZjDR-M8U1DH3johlMXy3h4MhwsKY_glYb1S0ghpbFb737obERp8nPPnPxvOBSvomfzPl3sbYhdfC9w4bOkd7WE03w)

[Abrir en el editor](https://ingcom-unrn.github.io/dibu-web/#z=ZY_BSgNBDIbv8xRxLraygueVFgr11EU86KkUyc5ml9DpzJCZFlT2qXwEX8y4iyB4-5P8H_-f8paoBuyKKVy8yoecyHHPDh1_fQZ43mzhiT2agCc9T7L1GI47Dl2uociZTEwkWDiGXBuAW_jIPAQsZ1HC9oulreCo9hpim0ku2EWpIE3XBFcreHxpGjv-R4cJTTEXHYqmXMMa7nQ1NZjDR-M8U1DH3johlMXy3h4MhwsKY_glYb1S0ghpbFb737obERp8nPPnPxvOBSvomfzPl3sbYhdfC9w4bOkd7WE03w) para explorarlo y modificarlo.

## Ver también

[`call_stack`](call_stack.md), [`call_tree`](call_tree.md), [`struct`](struct.md), [`object_heap`](object_heap.md)
