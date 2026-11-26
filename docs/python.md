# Diagramas de Python para el curso de ingreso

Propuesta de implementación para una sesión posterior. Toma como referencia el [apunte de ingreso](https://github.com/INGCOM-UNRN/apunte-ingreso), sus ejercicios de escritorio y los bancos de los capítulos 0, 2, 3 y 4. El punto de partida es dibu en `92a2680`. Los nombres y campos siguientes son propuestas de API.

| Prioridad | Cambio | Aplicación en el curso |
| --- | --- | --- |
| 1 | `array` con `variant: python_sequence` | Índices negativos, fronteras y slicing de listas, tuplas y cadenas |
| 2 | Variante Python del `object_heap` propuesto en [java.md](java.md) | Alias, copias superficiales, mutación, reasignación y paso de argumentos |
| 3 | Transferencias en `flowchart` | Ramas de `break`, `continue` y `return` en ejemplos de lazos anidados |
| 4 | `trace_table` con `variant: program` | Pruebas de escritorio con entrada, salida y condición separadas |
| 5 | Extensión de `flowchart` para `try` | Recorridos de `except`, `else`, `finally` y propagación |

## 1. Vista de secuencias e intervalos de corte

Reutilizar las celdas de `array`. Dibujar el índice positivo y el negativo de cada elemento y, en una banda opcional, las fronteras entre celdas. `slice` usa los rótulos de inicio, fin y paso y puede resaltar las celdas seleccionadas. Al marcar un corte con paso negativo, indicar el orden de visita mediante flechas o números de secuencia.

```yaml
type: array
variant: python_sequence
kind: list
values: [2, 4, 7, 1, 8]
showNegativeIndices: true
showBoundaries: true
slice:
  start: -4
  stop: -1
  step: 2
  blankSelection: true
```

`kind` acepta `list`, `tuple` y `str`. En `str` cada celda representa un punto de código Python. Las comillas y espacios visibles deben distinguirse de una celda ocultada para un ejercicio. Un corte puede omitir inicio, fin o paso. Omitir inicio y fin con paso negativo se comporta como `[::-1]`. El renderer puede calcular la selección con las reglas de `slice.indices(n)`. Los límites fuera de rango se ajustan y paso cero produce un error de validación.

**Aceptación.** Probar `a[1:4]`, `a[-4:-1:2]`, `a[::-1]`, `a[2:2]`, límites fuera de rango, paso cero y secuencia vacía. Confirmar que el límite excluido queda visible y que `blankSelection` oculta toda pista visual de la solución. El caso del banco `03_estructuras/indices` debería poder pasar del arreglo estático a esta vista sin incluir direcciones.

Fuentes del apunte — `2_control_flujo.md`, sección «Cadenas como secuencias de caracteres», y `3_estructuras/slicing.svg`.

## 2. Nombres y objetos compartidos

Extender el diseño de `object_heap` de [java.md](java.md) con `variant: python`. La columna izquierda agrupa nombres por ámbito, incluyendo `<módulo>` y funciones activas. La derecha contiene objetos con identidad simbólica. Los enlaces de nombres y elementos llegan a los mismos IDs cuando comparten objeto.

```yaml
type: object_heap
variant: python
frames:
  - function: "<módulo>"
    vars:
      - {name: original, ref: A}
      - {name: copia, ref: B}
objects:
  - id: A
    type: list
    elements: [{ref: I}]
  - id: B
    type: list
    elements: [{ref: I}]
  - id: I
    type: list
    elements: [{value: 4}, {value: 7}]
```

Usar `value` como abreviatura visual de escalares sin interés de identidad en ese ejemplo. Un ID declarado permite dibujar identidad de cualquier objeto cuando el ejercicio la necesita. `None` se muestra como objeto o valor rotulado, con un tratamiento distinto de un enlace sin destino. En instantáneas, mantener posición e identidad de los IDs conservados para que se vea cuál enlace cambió. La entrada declara los estados de cada instante.

**Aceptación.** Dibujar asignación `b = a`, copia `b = a.copy()`, copia superficial compartiendo sublista y parámetro que primero muta y luego se reasigna. Para el YAML, distinguir A de B y hacer converger sus dos flechas en I. Comprobar que un marco con un parámetro y otro con un nombre homónimo se mantienen independientes cuando se reasigna el parámetro.

Fuentes del apunte — `3_estructuras/memoria_referencias.svg` y `4_funciones/flujo_funcion.svg` y `4_funciones/scope.svg`.

## 3. Transferencias de control en diagramas de flujo

Agregar pasos explícitos `{break: true}`, `{continue: true}` y `{return: expresión}` a `flowchart`. `break` conecta con la salida del lazo más interno. `continue` conecta con la siguiente comprobación de `while` o la siguiente toma de elemento de `for`. `return` conecta con el final del ámbito de función. Mantener el símbolo de decisión y los rótulos existentes para que las preguntas comparen trazas atribuyendo cada salto a su destino.

```yaml
type: flowchart
steps:
  - for: "j en range(3)"
    do:
      - if: "j == 1"
        then:
          - continue: true
      - io: "MOSTRAR j"
```

**Aceptación.** Comparar el ejemplo con `break` en lugar de `continue`, incluyendo un lazo interno anidado y un paso posterior en el lazo externo. Validar saltos fuera de contexto y verificar que un paso después de un salto incondicional dentro de la misma rama no reciba una arista secuencial. Permitir ocultar el rótulo del paso en ejercicios.

Fuentes del apunte — `2_control_flujo.md`, secciones de manipulación y lazos anidados, y `4_funciones.md`, retornos.

## 4. Tabla de escritorio con entrada y salida

Extender `trace_table` conservando sus columnas y ocultamiento de celdas. La variante `program` agrega columnas reservadas para entrada consumida, evaluación de condición y salida emitida. Usar rótulos de etapa para «estado inicial», «iteración» y «salida». Declarar las filas como datos, con soporte para una cadena vacía que se represente de forma distinta de «sin emisión» y de una celda para completar.

```yaml
type: trace_table
variant: program
columns: [x, suma]
rows:
  - {step: "Primera lectura", input: 4, values: [4, 0]}
  - {step: "Vuelta 1", check: true, values: [4, 4]}
  - {step: "Centinela", input: 0, check: false, values: [0, 4]}
  - {step: "Mostrar", output: 4, blanks: [output]}
```

**Aceptación.** Verificar trazas de cero vueltas, lectura de centinela al comienzo y después de datos, y emisión de cero. Comprobar que una respuesta enmascarada no quede en el SVG ni en texto alternativo. Mantener compatibilidad con las filas posicionales y los `null` que ya admite la tabla.

Fuentes del apunte — `0_primeros_pasos.md`, «Pruebas de Escritorio», y `enunciados/escritorio_python.md`.

## 5. Recorrido de excepciones

Añadir a `flowchart` un bloque que declare el cuerpo del `try`, las rutas con tipos de excepción, el `else` y la limpieza. Empezar por diagramas con una sola operación potencialmente fallida y una rama de propagación explícita. `finally` recibe los caminos declarados de éxito, captura y propagación. Mantener diferenciados los rótulos normales y los excepcionales en impresión monocromática.

```yaml
type: flowchart
steps:
  - try:
      - step: "int(texto)"
    except:
      - matches: ValueError
        do:
          - io: "MOSTRAR error"
    else:
      - io: "MOSTRAR número"
    finally:
      - step: "Registrar intento"
    propagate: true
```

**Aceptación.** Cubrir conversión válida que entra por `else`, `ValueError` capturado y excepción que se propaga tras la limpieza. En un segundo incremento, contemplar retorno en el cuerpo y excepción durante la limpieza. Comprobar las aristas por destino en las pruebas, además del SVG.

Fuentes del apunte — `B_excepciones/try_except_else_finally.svg` y `B_excepciones/raise_excepcion.svg`.

## Integración

El motor actual permite usar [array](docs/manual/estructuras/lineales/array.md) con índices positivos y sin direcciones, [flowchart](docs/manual/estructuras/algoritmos/flowchart.md) con estructuras anidadas, [trace_table](docs/manual/estructuras/algoritmos/trace_table.md) para estados, [call_stack](docs/manual/estructuras/memoria/call_stack.md) para marcos y [venn](docs/manual/matematica/probabilidad/venn.md) para operaciones de conjunto. Los bancos usan PlantUML para actividad y secuencias del cuaderno. Las cinco propuestas priorizan representación de estados y recorridos con campos que admiten ocultar respuestas.

Incorporar un caso resuelto y otro para completar por variante en el manual y los fixtures. Validar por CLI y WebAssembly y, después de actualizar el motor que use AlucarD, generar un examen de muestra en HTML y PDF. Para bloques GIFT nativos, escribir los mapeos YAML en varias líneas sin llaves de flujo.
