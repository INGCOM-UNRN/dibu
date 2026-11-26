# Diagramas de C para implementar en Dibu

Propuesta para una sesión posterior en `/home/mrtin/dev/tools/docente/contenidos/dibu`.
Basada en el apunte de C, los bancos 2027 y Dibu en `7b98f49`. Los nombres de
API en la columna «Cambio propuesto» son tentativos.

El `c.md` indicado todavía no existe en Dibu. Para el inventario se usaron
el manual, el registro de tipos, los renderizadores y el binario local. Los
bancos GIFT de 2027 todavía carecen de bloques `estructura`. En 2026 sí hay
preguntas con diagramas nativos, por ejemplo las de matrices estáticas,
planas y dentadas en `parcial_3_diagramas`.

## Qué se puede usar hoy

| Tema | Tipo disponible | Variantes para el examen |
| --- | --- | --- |
| Trazas de C | `trace_table` con `variant: program` y `flowchart` | Rellenar el estado de la bandera y el contador, la salida o la condición. Usar filas distintas para «falso» y «división omitida». Comparar entrada vacía, centinela inicial y corte intermedio. |
| Funciones y recursión | `call_stack` con `steps` y `pointers`, y `call_tree` | Mostrar marcos de llamadas sucesivas, ocultamiento entre marcos, mutación por puntero y retorno. Para dividir y vencer, ocultar retornos o costo por nivel. |
| Arreglos y matrices | `array` y `matrix` con `static`, `flat`, `dynamic` o `jagged` | Ocultar direcciones, largos o valores. Comparar una matriz por filas, un bloque plano y filas independientes. Para límites de búsqueda, indicar el límite excluido en la traza. |
| Registros y TDA | `struct` con `layout` u `opaque`, opción `packed: true`, y `adt` | Comparar orden de campos, padding interior y final, vista del cliente y precondiciones. Declarar los tamaños y alineamientos del ejercicio cuando dependen de la ABI. |
| Listas, pilas y colas | `linked_list`, `stack` y `queue` | Dibujar listas vacías, de un nodo y con cursores sobre nodos duplicados. Para colas circulares, usar celdas en orden físico en el renderer actual y explicitar el significado de `FINAL`. |
| Testing con Treadstone | `trace_table` con columnas elegidas para el caso | Dar entrada, estado inicial y un resultado observado. Pedir oráculo, aserción, estado final y caso que detecte un mutante. Usar una tabla para fallo de reserva, cola vacía, escritura corta o estabilidad. |
| Compilación, GDB y toolchain | `flowchart` y `call_stack`; PlantUML del apunte | Dibujar el flujo lineal de compilación o una sesión de GDB. El tipo `pipeline` de Dibu modela instrucciones de CPU. El apunte ya tiene un DAG de Make en SVG y estados de GDB en PlantUML. |

Las preguntas de prueba deben suministrar los datos necesarios y pedir
la suite en Treadstone. Una tabla con oráculos en blanco puede acompañar
la consigna. La clave puede mostrar la tabla completa junto con los
criterios de implementación y pruebas.

## Prioridades de implementación

| Orden | Cambio propuesto | Preguntas que habilita |
| --- | --- | --- |
| 1 | `object_heap` con `variant: c` y punteros a objetos, campos y celdas | Ubicar auto, static y heap. Rastrear pasaje por valor frente a `T **`, alias de registros, préstamos y estados tras `free` o `realloc`. |
| 2 | Vista de `array` para límites y cadenas de C | Distinguir `a+n`, primer NUL, capacidad disponible y celdas posteriores al terminador. Rastrear índices de lectura y escritura. |
| 3 | `queue` con representación circular basada en frente y cantidad, extensible a deque | Deducir orden lógico con vuelta de índice y comparar vacío, lleno y extracción del último elemento. |
| 4 | `byte_layout` para campos serializados y filas BMP | Completar endianness, offset, stride y padding. Comparar entrada corta, índice de paleta inválido y cabecera válida. |
| 5 | `matrix` con `variant: row_pointers`, y `struct` con `variant: union` | Dibujar un bloque único con punteros interiores, y campos superpuestos con discriminante separado. |
| 6 | Secuencias de concurrencia y E/S | Deducir sincronización, bloqueo, envíos parciales y cierre. Generalizar solo si las figuras de PlantUML o la traza tabular resultan insuficientes para el modo ejercicio. |

### 1. Memoria y vida de objetos

Extender el plano de `object_heap`, que ya ofrece instantáneas y enlaces a
objetos en Java y Python. En C los objetos pueden vivir en un marco, en
almacenamiento estático o en memoria dinámica. Un mismo identificador de
objeto conserva su identidad entre instantes. Los nombres de variables,
los identificadores y las direcciones simbólicas tienen propósitos distintos.

- Resolver destinos por objeto, campo o índice. Las celdas y los campos pueden ser objetos de punteros válidos. Modelar `NULL`, valor indeterminado, puntero colgante y límite excluido con rótulos distintos.
- Mantener visible un bloque liberado como referencia histórica, con el rótulo «vida terminada». Una flecha de diagnóstico hacia ese bloque debe distinguirse de una referencia válida. No dibujar un valor legible dentro de memoria liberada.
- Declarar los estados de cada instante. Para `realloc` con tamaño positivo, el fallo conserva bloque y alias. El éxito termina el bloque anterior. Cuando se ilustra la reutilización de la dirección, el nuevo objeto mantiene identidad distinta aunque se dibuje en la misma dirección simbólica.
- En ejercicios, ocultar por separado enlaces y estados. El modo resuelto conserva la misma geometría y datos de entrada.
- Cuatro primeras variantes: parámetro `T **` que cambia un puntero del llamador, copia superficial de un registro, `free` con dos alias y `realloc` con éxito frente a fallo. Reservas: retorno de dirección de un local, vista prestada invalidada por mutación, local `static` y pérdida del último propietario.

**Aceptación.** Comprobar que `p` y `q` converjan cuando comparten objeto y que el
render no mezcle dos objetos distintos con la misma dirección. Un destino
inexistente en un enlace válido se rechaza. Una referencia colgante apunta
solo a la identidad histórica declarada. Probar las versiones Java y Python
existentes junto a la nueva.

Ejemplos 2027 — `lectura: cambiar-por-puntero`, `alias-registro`, `ampliar-bloque`,
`copia-y-prestamo`. Apunte — `2_punteros.md`, `5_memoria_dinamica.md`,
`uml/ciclo_vida_bloque.puml`.

### 2. Límites, capacidad y cadenas en `array`

El arreglo actual dibuja índices y offsets, pero descarta la marca
`pointers: "fin:n"` cuando hay `n` celdas. Incorporar un marcador sobre
la frontera derecha, sin crear una celda `a[n]`. Permitir un intervalo
semiabierto con límites visibles y un orden de visita opcional. Para
cadenas C, admitir bytes declarados, separar longitud lógica de capacidad
y rotular el primer NUL como `\0`. Diferenciar bytes posteriores a ese NUL,
capacidad sin datos declarados y casilleros que el alumno debe completar.

- Primeras variantes: recorrido de `[a+1,a+n)`, cadena terminada con espacio sobrante, arreglo sin terminador y compactación con cursores de lectura y escritura.
- Si la entrada no contiene NUL, el resultado es «sin terminador en la capacidad indicada». El diagrama evita mostrar un resultado de `strlen` para ese caso.
- Para `trace_table`, hace falta distinguir «no evaluado» de «falso» si se quiere codificación visual propia. Antes de agregar API se puede escribir el estado como texto en una columna ordinaria.

**Aceptación.** Marcar el límite exactamente después de la última celda,
admitir intervalo vacío y ocultar los resultados solicitados. En una
secuencia que contenga `a, \0, b, \0` el primer NUL fija la longitud y las
cuatro celdas siguen siendo visibles.

Ejemplos 2027 — `lectura: caminar-cadena`, `programacion: recortar-extremos`,
`compactar-pares`. Apunte — `3_secuencias.md`, `6_aritmetica.md`.

### 3. Cola circular con cantidad y deque

El manual describe `values` en orden lógico. El renderer circular coloca
`values[i]` en la posición física `i`. El panel usa `FINAL` como último
insertado y deja siempre en blanco la cantidad. Reproducido en el binario
local con capacidad 5 y frente 4.

Agregar un modo explícito basado en `capacity`, `frontIndex` y una
secuencia lógica. Su longitud da `count`; con longitud positiva se
deriva el último índice ocupado como `(frontIndex+count-1)%capacity`.
El siguiente índice de inserción se presenta por separado y solo es
utilizable cuando hay espacio. Mantener el comportamiento de los YAML
existentes y documentar las dos convenciones. El modo deque reutiliza la
misma representación y cambia las operaciones permitidas.

**Aceptación.** Con capacidad 5, frente 4 y secuencia `[7,9]`, el 7 ocupa
la celda 4 y el 9 la 0. Probar vacío, singleton, lleno y vuelta después de
quitar o agregar en cualquiera de los extremos. Con cantidad igual a la
capacidad, frente y siguiente índice coinciden, pero la cola está llena.
Se deben poder ocultar frente, último índice y cantidad de forma independiente.

Ejemplos 2027 — `lectura: cola-envuelta`, `deque-extremos`, `programacion:
quitar-frente-deque`, `quitar-final-deque`, `cola-con-modelo`.

### 4. Layout de octetos y BMP

Un tipo `byte_layout` puede leer campos explícitos con ancho, offset y
orden de bytes. Usar celdas de arreglo cuando corresponda y permitir
agruparlas por campo. Una vista de filas BMP agrega ancho lógico, stride,
orden de filas y un marcador de padding. El tamaño de archivo y el
layout en memoria de C son datos distintos. El búfer dibujado debe tener
octetos explícitos o derivarse de campos con tamaños y orden definidos.

| Variante | Ejemplo de comprobación |
| --- | --- |
| Cabecera binaria | `0x1234` produce `12 34` en orden de red y `34 12` en little-endian. |
| BMP de 24 bits | Ancho 3 produce 9 octetos útiles, 3 de relleno y stride 12. Un píxel rojo se escribe como `00 00 FF`. |
| BMP indexado | Ancho 5 produce stride 8. Un índice fuera de la paleta se marca como inválido y el padding queda fuera de la validación. |
| Dos órdenes de filas | Con offset 54, ancho 2, altura 3 y stride 8, el inicio de la fila visual superior está en 54 o 70 según el orden. |

**Aceptación.** Ocultar campos, offsets o bytes por separado. Probar una fila
sin padding y otra truncada. Para tamaños fuera del intervalo de
representación, devolver un diagnóstico sin dibujar un rango imposible.
Los rótulos «dato», «relleno» y «truncado» sobreviven al monocromo.

Ejemplos 2027 — `lectura: stride-24`, `centrar-fila`, `relleno-intacto`,
`longitud-de-red`. Apunte — `4_binarios_archivos.md` y `6_bitmaps.md`.

### 5. Dos extensiones acotadas de memoria

- **Matriz con punteros a filas y un solo bloque de datos.** Dibujar
  una tabla de punteros que apunta a los índices `0, C, 2*C, …` del
  mismo bloque. Para 2×3, la fila 1 comienza en el índice 3. Comparar con
  `dynamic`, que usa reservas de fila independientes. En la clave deben
  verse los dos recursos que se liberan. Una segunda variante aborda la
  reserva parcial de filas con estados explícitos en `object_heap`.
- **Unión con discriminante.** Dibujar las vistas de los campos desde
  el offset 0 con tamaños y alineamientos declarados. Mostrar la etiqueta
  del registro exterior y el miembro que selecciona el ejemplo. Probar
  `union` con miembros char e int bajo supuestos dados y la comparación
  con una `struct` equivalente. Admitir ocultar layout y discriminante
  independientemente. El tamaño final respeta la alineación declarada.

Ejemplos 2027 — `matrices-dinamicas-06`, `lectura: union-con-marca` y
`prefijo-config`. Apunte — `1_matrices.md`, `7_estructuras.md` y
`13E_padding_abi.md`.

### 6. Concurrencia, E/S y variantes posteriores

- **Carriles para hilos.** Eventos declarados de lectura, escritura, lock,
  unlock, espera y join. Variantes mínimas de incremento sin sincronizar,
  incremento protegido, bloqueo por orden inverso y condición con
  despertar y nueva comprobación del predicado. El resultado muestra el
  orden declarado. Una intercalación sin sincronización ilustra un fallo
  posible y se rotula como carrera de datos, sin predecir una salida C.
- **Carriles para flujo TCP.** Cliente y servidor, descriptor de escucha
  y conexión. Variantes de lectura fragmentada, dos tramas en una
  recepción, cierre ordenado y cierre a mitad de trama. Una misma
  escritura puede alimentar varias lecturas, por lo que las flechas de
  E/S y los límites de trama necesitan rótulos separados.
- **Control de flujo de C.** Si se reutiliza `flowchart` para ampliar la
  cobertura, agregar `switch` con `case`, `default` y caída al caso
  siguiente. Probar que `break` salga del switch interno y que
  `continue` vaya al lazo circundante. Los lazos y retornos actuales
  se pueden usar directamente.
- **Trazas de algoritmos y dependencias.** Empezar con `trace_table` para
  búsqueda binaria, mezcla estable y compilar en orden. Considerar una
  vista de intervalos `[lo,hi)` si hace falta seguirlos sobre el arreglo.
  Para un grafo de compilación, conservar el DAG en SVG y la figura de
  compilación separada en PlantUML hasta que se justifique un modo de
  ejercicio que oculte aristas o marque las recompilaciones. Lo mismo aplica a la sesión de GDB.

Fuentes del apunte — `uml/carrera_contador.puml`, `uml/deadlock.puml`,
`uml/sockets_tcp.puml`, `uml/compilacion_separada.puml` y `uml/gdb_estados.puml`.

## Primer incremento y verificación

El primer incremento puede implementar `object_heap: c` y la frontera
de `array`, con una figura resuelta y otra para completar por familia.
Después, probar la cola circular con cantidad y el layout binario en los
bloques B y D. En 2027 sirve empezar por lectura de código, donde la figura
se puede vincular con un estado intermedio ya pedido. Para usar las figuras
en los cuatro temas sin cambiar el sorteo, preparar ocho alternativas
equivalentes por familia cuando la pregunta lleve el diagrama.

- En Dibu, los puntos de entrada son `crates/dibu-cs/src/objetos.rs`,
  `lineal.rs`, `recursion.rs` y `algos.rs`. Documentar ejemplos en
  `docs/manual/datos/` y regenerar manual y esquema con
  `python3 docs/manual/generar.py`.
- Probar invariantes del grafo o la ubicación de celdas y rótulos mediante
  el SVG, además de validar su XML. Incluir fixtures de error para índices
  inválidos, referencias inexistentes y tamaños fuera de rango.
- Validar el mismo YAML con CLI y WebAssembly. En modo examen, verificar
  por separado que la respuesta oculta no aparezca en texto, descripción ni
  atributos del SVG y que tampoco la revelen aristas o resaltados.
- Integrar un bloque piloto en AlucarD con el binario elegido mediante
  `ALUCARD_DIBU`. En GIFT, usar YAML multilínea sin llaves de flujo ni
  renglones en blanco dentro del bloque. Validar la versión para completar
  en el examen y la resuelta en la clave. Revisarlas en HTML y en PDF
  monocromo, especialmente a lo ancho de una página.

La auditoría local ejecutó `dibu batch` con trazas, matrices, pila de
llamadas, layout de registros y cola circular. Verificó que el valor
oculto de una traza no apareciera en el SVG y que el marcador de fin del
arreglo se omitiera. También confirmó que `object_heap` con `variant: c`
y las variantes propuestas de punteros a filas y de unión todavía son
rechazadas por el binario.
