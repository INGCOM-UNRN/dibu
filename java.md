# Diagramas para enseñar Java con dibu

Propuesta para implementar en una sesión posterior. Parte de las figuras del apunte de Programación II y del motor `4e62d0a` que usa su plugin de MyST. Los nombres y campos de las variantes nuevas son propuestas de API.

## Prioridades

| Orden | Cambio | Figuras del apunte que habilita |
| --- | --- | --- |
| 1 | `matrix` con `variant: java` | Arreglo de arreglos `int[][]` con filas independientes |
| 2 | Nuevo tipo `object_heap` | Stack y heap, copia de referencias, pasaje por valor, `this`, instanciación y String Pool |
| 3 | Paso `doWhile` en `flowchart` | Comparación de `for`, `while` y `do-while` |

Las jerarquías, multiplicidades, realizaciones y secuencias entre objetos tienen representación en PlantUML. Las variantes de dibu de esta propuesta se centran en la representación de datos y en el flujo de ejecución.

## 1. Matriz de Java

**Propuesta.** Agregar `variant: java` a `matrix`. El caso rectangular y el irregular comparten el mismo dibujo. Un arreglo exterior contiene referencias indexadas a las filas y cada fila tiene su propio largo. La variante `dynamic` existente renderiza la declaración `int **matriz`. La nueva usa `int[][] matriz`.

```yaml
type: matrix
variant: java
name: matriz
elementType: int
values:
  - [1, 2, 3]
  - [4]
  - null
  - []
zones: true
```

- Dibujar `matriz` en el stack como una referencia al arreglo exterior en el heap. Mostrar un enlace desde cada posición que contiene una referencia y rotular `null` en la tercera posición.
- Mostrar las filas como objetos separados. La cuarta posición apunta a un arreglo válido de largo cero. Con `values: []` se muestra un arreglo exterior de largo cero.
- Usar identificadores simbólicos para los objetos. El dibujo representa relaciones e índices. El orden y la posición geométrica de las filas carecen de significado físico.
- Reutilizar el trazado de filas y flechas de `matrix` donde corresponda. Los casos de dos posiciones que apuntan a la misma fila se pueden expresar con `object_heap`.

**Aceptación.** Renderizar la matriz rectangular del apunte, la irregular del YAML y el arreglo exterior vacío. Verificar que `null` y `[]` sean distinguibles, que los enlaces salgan de sus índices correspondientes y que la declaración visible tenga sintaxis Java. Cubrir la compatibilidad de las variantes `static`, `dynamic`, `jagged` y `flat` con una regresión de renderizado.

Fuente en el apunte — `parte_1/09_arreglos.md`, `fig-matriz-memoria`.

## 2. Marcos y objetos por referencia

**Propuesta.** Incorporar `object_heap` como vista didáctica del stack y el heap. El primer caso a implementar es una vista de un solo instante. Los campos `frames` y `objects` declaran los marcos activos y los objetos que entran en el ejemplo. Cada objeto tiene un ID explícito. Las variables, los campos y las celdas usan `value` para valores o `ref` para referencias. `ref: null` dibuja una referencia nula.

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
    elements: [{value: 1}, {value: 2}, {value: 3}]
```

- Ubicar los marcos en el stack y los objetos en el heap. Las flechas llegan al objeto identificado en `ref`. Dos referencias a `A` llegan a la misma caja aunque se originen en marcos distintos.
- Para instancias como `Persona`, usar `fields` con la misma forma que `vars`. Para arreglos, usar `elements` y numerar las celdas. Los arreglos de referencias usan `ref` en sus celdas. Así también se puede mostrar un enlace de un objeto a otro objeto o a sí mismo.
- Exponer IDs simbólicos como `A` y `B`. El renderer valida IDs duplicados, destinos inexistentes y entradas que mezclen `value` con `ref`. Las cadenas se dibujan como contenido. La identidad sale del ID declarado.
- Incorporar `group: String Pool` para agrupar los objetos declarados dentro del pool en el heap. La pertenencia al pool proviene del YAML.
- Reutilizar el dibujo de marcos de `call_stack` y el de celdas de `array` en cuanto acepten geometría y conectores. El tipo nuevo necesita que todas las flechas se resuelvan dentro del mismo plano.

### Variante por instantáneas

Agregar `snapshots` después de validar el caso de una sola vista. Cada entrada usa `label`, `frames` y `objects`. Los IDs se resuelven dentro de cada instantánea. Repetir un ID en otro panel representa la misma identidad entre instantes. Los paneles se pueden apilar para conservar el ancho de lectura.

| Ejemplo | Qué debe verse |
| --- | --- |
| Paso de un primitivo | El parámetro recibe su valor. Al cambiarlo, la variable del llamador conserva el valor anterior. |
| Mutación de un arreglo | `numeros` y `arr` llegan a `A`. Tras `arr[0] = 999`, ambos observan la nueva celda. |
| Reasignación de un parámetro | Al comienzo ambas referencias llegan a `A`. Tras `arr = new int[] {...}`, solo `arr` apunta a `B`. |
| Instanciación y `this` | `juan` y el `this` del marco de su método llegan al mismo objeto. `maria` llega a otra instancia. |
| String Pool | `s1` y `s2` llegan al literal `"Hola"` del pool. `s3` llega a otra instancia que contiene el mismo texto. |
| Alcanzabilidad | Tras asignar `null` a la última raíz, el objeto queda sin camino desde el stack y puede permanecer visible como candidato a recolección. |

**Aceptación.** Probar alias, dos objetos con contenido idéntico, una referencia nula, conexiones entre objetos y copia frente a reasignación. En las vistas sucesivas, comprobar que se conserva la identidad de un objeto al cambiar su contenido y que las flechas solo cambian al reasignar las referencias correspondientes. El dibujo describe los estados declarados y permite ilustrar alcanzabilidad sin modelar el momento de recolección.

Fuentes en el apunte — `parte_1/04_metodos.md` (`fig-pasaje-parametros`), `parte_1/09_arreglos.md` (`fig-referencias-copia`), `parte_1/13_memoria.md` y `parte_2/03_sintaxis_clases.md` (`fig-modelo-memoria-this`, `fig-instanciacion-memoria`).

## 3. Lazo con prueba al final

**Propuesta.** Extender `flowchart` con un paso `{doWhile, do}` que use la misma lista anidable de pasos que `while` y `for`.

```yaml
type: flowchart
title: Lazo do-while
steps:
  - "i = 0"
  - doWhile: "i < 3"
    do:
      - "procesar(i)"
      - "i = i + 1"
```

- Conectar la entrada al cuerpo y el final del cuerpo al rombo de la condición. La rama verdadera vuelve al cuerpo y la falsa llega al paso siguiente.
- Usar tres bloques con la misma acción para comparar `for`, `while` y `do-while`. Los casos de cortocircuito se pueden representar con condicionales anidados del `flowchart` actual. Para la bandera, `trace_table` permite registrar el índice, el valor leído y si ya se encontró el objetivo.

**Aceptación.** Verificar que el camino desde la entrada alcance el cuerpo antes del rombo, incluso cuando la condición inicial es falsa. Probar un `doWhile` dentro de otro lazo y verificar que los enlaces verdadero y falso conserven su destino. Probar el modo `blank` con el mismo comportamiento visual que los demás pasos.

Fuente en el apunte — `parte_1/05_sintaxis_control.md`, `fig-comparacion-lazos`, `fig-cortocircuito` y `fig-patron-bandera`.

## Figuras que ya se pueden generar

| Tema | Tipo de dibu | Ejemplo |
| --- | --- | --- |
| Representación de `double` | `ieee754` | `format: double` con `value: 9.5`. |
| Celdas e índices de un arreglo | `array` | Mostrar la estructura con índices. Usar `object_heap` cuando también haga falta mostrar dónde vive la referencia. |
| Creación y retiro de marcos | `call_stack` | `steps` y `popped` muestran llamadas y retornos. `showAddresses: false` sirve para el modelo conceptual de Java. |
| Lista simple, doble y circular | `linked_list` | Usar las variantes `singly`, `doubly` y `circular` con los mismos nodos para comparar enlaces. |
| Contrato de una pila | `adt` | Declarar precondiciones, poscondiciones, invariante y las dos representaciones del TAD. |
| Órdenes de crecimiento | `complexity` | Comparar las curvas desde constante hasta factorial si se recupera la figura de complejidad. |

## Implementación y verificación

- Los puntos de entrada actuales están en [lineal.rs](crates/dibu-cs/src/lineal.rs), [recursion.rs](crates/dibu-cs/src/recursion.rs) y [algos.rs](crates/dibu-cs/src/algos.rs). Usar los parsers, el registro y el theme compartidos.
- Agregar un ejemplo al manual y un fixture por variante. Los tests deberían verificar el contenido y el destino de los enlaces, además de que se produzca un SVG válido. Validar por CLI y con el motor WebAssembly para cubrir el uso desde MyST.
- Al integrar en el apunte, actualizar el motor embebido del plugin y usar `:::{dibu}` con `:caption:`, `:label:` y `:width:`. Reutilizar los identificadores `fig-…` y comprobar que `[](#fig-…)` resuelva al renderizar. Retirar un SVG manual cuando su sustituto conserve la explicación y el archivo haya quedado sin referencias.

Manual local de dibu — [matrix](docs/manual/estructuras/lineales/matrix.md), [array](docs/manual/estructuras/lineales/array.md), [call_stack](docs/manual/estructuras/memoria/call_stack.md), [flowchart](docs/manual/estructuras/algoritmos/flowchart.md), [ieee754](docs/manual/computacion/arquitectura/ieee754.md).
