[Manual](../README.md)

# Guía de dibu para agentes de IA

Esta guía es para un agente (o una persona) que redacta apuntes, guías o exámenes y necesita
**escribir diagramas de dibu e integrarlos** en el documento. Reúne en un lugar la sintaxis, el
catálogo de los 70 tipos, cómo se inserta un diagrama en cada destino y cómo comprobarlo
antes de entregar.

| Archivo | Para qué |
| :--- | :--- |
| [`README.md`](README.md) (esta página) | Reglas, catálogo de tipos, integración y validación. Leerla completa una vez. |
| [`referencia.md`](referencia.md) | Cada tipo con todos sus campos, valores admitidos, valores por defecto y ejemplos que se dibujan sin errores. Buscar ahí el tipo elegido. |
| [`esquema.json`](esquema.json) | Lo mismo en JSON, para herramientas: `tipos.<type>.campos.<campo>` con `tipo`, `valores`, `defecto` y `descripcion`. |
| [`validar.py`](validar.py) | Valida bloques sueltos o los de un documento contra el esquema y los dibuja con dibu. |

Todo se genera desde el código de dibu y desde ejemplos probados (`docs/manual/generar.py`):
coincide con la versión de dibu del repositorio.

## Flujo de trabajo

1. **Decidir si hace falta un diagrama** y cuál: el [catálogo](#catálogo-de-tipos) lista cada
   tipo con su uso. Un diagrama por idea; si el texto ya lo explica bien, no agregarlo.
2. **Leer el tipo en [`referencia.md`](referencia.md)** y partir de uno de sus ejemplos: cambiar
   los datos, no inventar campos.
3. **Escribir el bloque** con la sintaxis del destino (ver [Integración](#integración)).
4. **Validar** con `validar.py` y, si se puede, construir el documento (`myst build`, `typst
   compile`, `dibu sync-md --verificar`).
5. **Mirar el resultado** cuando haya forma de ver el SVG: que el diagrama muestre lo que dice
   el texto que lo rodea.

## Reglas

- **Un bloque es un objeto YAML (o JSON) con `type`.** Los demás campos dependen del tipo. Un
  bloque puede ser también una lista de bloques: se dibujan uno debajo del otro, en un solo
  diagrama.
- **Dar los datos del problema, no el resultado.** dibu calcula lo que dibuja: las direcciones
  de un arreglo, la minimización de un mapa de Karnaugh, el diagrama de Gantt de un planificador,
  los márgenes de un Bode, el árbol después de cada inserción. Precalcular a mano es una fuente
  de errores y no hace falta.
- **Usar sólo los campos de la referencia, con los valores que lista.** dibu es permisivo: un
  campo mal escrito o desconocido **se ignora sin avisar**, y un valor que no está en la lista
  puede dibujarse como si fuera otro. El diagrama sale, pero distinto de lo pedido. `validar.py`
  detecta estos casos.
- **Los booleanos son `true` y `false`.** dibu lee YAML 1.2: `yes`, `no`, `on` y `off` son
  textos, no booleanos.
- **Comillas para el texto con `:`, `#`, `[`, `]`, `{`, `}` o que empiece con `@`:**
  `"max = v[0]"`, `"R1 = 10k"`, `"@N1"`. Los decimales van con punto (`0.1`); dibu los muestra con
  coma.
- **`title` siempre que el diagrama no se explique solo.** Además, el título forma el texto
  alternativo de la imagen en MyST y en Typst.
- **Theme:** no poner colores ni `theme` en cada bloque. El destino fija el theme de todo el
  documento (por omisión `ink`, para imprimir). `theme` en un bloque sólo si ese diagrama
  necesita otro.
- **Modo ejercicio:** el mismo bloque con `blank: true` (o los campos `blank*` del tipo) deja en
  blanco lo que se pregunta. El bloque sin `blank` es la clave. Para un examen, escribir el
  bloque completo y agregarle `blank`.
- **Tamaños:** cada tipo tiene cotas (filas, nodos, muestras). Un diagrama para un apunte entra
  en una página: preferir pocos elementos representativos.

## Campos comunes

Todos los tipos aceptan, además de los propios:

| Campo | Tipo | Descripción |
| :--- | :--- | :--- |
| `type` | texto | El tipo de diagrama. Obligatorio. |
| `title` | texto | Título del diagrama, arriba a la izquierda. |
| `subtitle` | texto | Subtítulo, debajo del título, en letra más chica. |
| `id` | texto | Nombre del archivo al exportar (`dibu render` escribe `<id>.svg`). |
| `inkSaver` | sí/no | `true` usa el theme `ink` y `false` el `color`, sea cual sea el theme global. |
| `theme` | texto u objeto | Theme de este diagrama: un preset (`ink`, `color`, `high-contrast`, `dark`) o un objeto `{extends, colors, typography}`. |

## Catálogo de tipos

### Estructuras de datos y algoritmos

**Estructuras lineales**

- [`array`](referencia.md#array): Arreglo. Arreglos en memoria, con índices, direcciones, offsets y punteros que los recorren.
- [`matrix`](referencia.md#matrix): Matriz. Matrices estáticas, dinámicas, dentadas y planas, como están en memoria.
- [`linked_list`](referencia.md#linked_list): Lista enlazada. Listas simples, dobles y circulares, con direcciones, cursores y estructura cabecera.
- [`stack`](referencia.md#stack): Pila. Pilas en arreglo, con el tope, y su evolución paso a paso.
- [`queue`](referencia.md#queue): Cola. Colas lineales y circulares, con frente y final.

**Memoria, tipos y recursión**

- [`call_stack`](referencia.md#call_stack): Pila de llamadas. Los marcos de activación de una ejecución, con parámetros, variables y punteros entre marcos.
- [`call_tree`](referencia.md#call_tree): Árbol de llamadas. Las llamadas de una función recursiva, su orden, sus resultados y el costo por nivel.
- [`struct`](referencia.md#struct): Struct. La disposición de un struct en memoria (offsets, relleno, sizeof) y los tipos opacos.
- [`adt`](referencia.md#adt): Tipo abstracto de datos. Un TAD con sus operaciones, precondiciones, invariante y representaciones posibles.

**Árboles y tablas**

- [`tree`](referencia.md#tree): Árbol binario. Árboles binarios, de búsqueda, AVL y montículos, por niveles, por inserciones o anidados.
- [`heap`](referencia.md#heap): Montículo (heap). Min-heaps y max-heaps construidos por inserciones o con heapify, con sus operaciones.
- [`avl`](referencia.md#avl): Árbol AVL. Inserciones y eliminaciones en un AVL, con las rotaciones que hace cada una.
- [`bplus_tree`](referencia.md#bplus_tree): Árbol B+. Inserciones en un árbol B+, con las divisiones de hojas y nodos internos.
- [`hash_table`](referencia.md#hash_table): Tabla de hash. Inserciones en una tabla de hash, con encadenamiento o direccionamiento abierto.

**Análisis de algoritmos**

- [`complexity`](referencia.md#complexity): Complejidad. Curvas de crecimiento para comparar órdenes, y jerarquías de clases de complejidad.
- [`flowchart`](referencia.md#flowchart): Diagrama de flujo. Diagramas de flujo armados a partir de la estructura del algoritmo.
- [`cost_table`](referencia.md#cost_table): Tabla de costos. El análisis de un algoritmo línea por línea, con el costo y las veces de cada una.
- [`trace_table`](referencia.md#trace_table): Tabla de traza. Pruebas de escritorio de variables y trazas de operaciones sobre una estructura.
- [`dp_table`](referencia.md#dp_table): Tabla de programación dinámica. LCS, distancia de edición, mochila 0/1 y cambio de monedas, con el camino de reconstrucción.
- [`dijkstra`](referencia.md#dijkstra): Dijkstra. La tabla de iteraciones del algoritmo de Dijkstra y los caminos mínimos.

**Computabilidad**

- [`turing`](referencia.md#turing): Máquina de Turing. La traza de configuraciones de una máquina de Turing y su tabla de transiciones.

### Sistemas de computación

**Sistemas digitales**

- [`truth_table`](referencia.md#truth_table): Tabla de verdad. Tablas de verdad de una o varias funciones, con columnas intermedias y formas canónicas.
- [`karnaugh`](referencia.md#karnaugh): Mapa de Karnaugh. Mapas de 2 a 4 variables con los agrupamientos mínimos y la expresión simplificada.
- [`logic_circuit`](referencia.md#logic_circuit): Circuito lógico. El circuito con compuertas que implementa una expresión booleana.

**Arquitectura de computadoras**

- [`pipeline`](referencia.md#pipeline): Pipeline. El diagrama de ciclos de un pipeline de 5 etapas, con riesgos, burbujas y forwarding.
- [`ieee754`](referencia.md#ieee754): IEEE 754. La codificación de un número en punto flotante, en los dos sentidos.
- [`cache`](referencia.md#cache): Memoria caché. La simulación de una caché directa o asociativa sobre una secuencia de accesos.

**Sistemas operativos**

- [`cpu_scheduling`](referencia.md#cpu_scheduling): Planificación de CPU. El diagrama de Gantt y los tiempos de FCFS, SJF, SRTF, Round Robin y prioridades.
- [`page_table`](referencia.md#page_table): Tabla de páginas. La traducción de direcciones lógicas a físicas con paginación.
- [`page_replacement`](referencia.md#page_replacement): Reemplazo de páginas. FIFO, LRU, óptimo y reloj sobre una cadena de referencias, para compararlos.

**Redes**

- [`subnetting`](referencia.md#subnetting): Subnetting. Subredes IPv4 de igual tamaño o con VLSM, con su mapa de direcciones.

**Bases de datos**

- [`transaction_schedule`](referencia.md#transaction_schedule): Planificación de transacciones. Conflictos, grafo de precedencia y serializabilidad por conflictos.

### Control de procesos

**Modelado**

- [`block_diagram`](referencia.md#block_diagram): Diagrama de bloques. Lazos de control con realimentación, perturbaciones y lazos en cascada.

**Respuesta temporal**

- [`time_response`](referencia.md#time_response): Respuesta temporal. La salida de un sistema ante escalón, rampa o impulso, con sobrepico, tiempos y constantes.

**Respuesta en frecuencia**

- [`bode`](referencia.md#bode): Diagrama de Bode. Magnitud y fase en frecuencia, con asíntotas y márgenes de estabilidad.
- [`nyquist`](referencia.md#nyquist): Diagrama de Nyquist. La curva de Nyquist del lazo abierto, con el círculo unitario y las ω negativas.

**Plano s**

- [`pole_zero`](referencia.md#pole_zero): Polos y ceros. Los polos y ceros de un sistema en el plano s.
- [`root_locus`](referencia.md#root_locus): Lugar de las raíces. El lugar de las raíces con asíntotas, cruce con el eje imaginario y ganancias marcadas.

**Instrumentación y automatización**

- [`instrumentation`](referencia.md#instrumentation): P&ID. Diagramas de instrumentación con símbolos ISA-5.1.
- [`ladder`](referencia.md#ladder): Diagrama escalera. Programas de PLC en lenguaje escalera, con contactos, bobinas, temporizadores y contadores.
- [`grafcet`](referencia.md#grafcet): GRAFCET. Etapas, transiciones y divergencias en O y en Y.

### Química

**Estructura y enlace**

- [`molecule`](referencia.md#molecule): Molécula. Fórmulas de Lewis y esqueletales, con cargas, pares libres, estereoquímica y flechas de mecanismo.
- [`vsepr`](referencia.md#vsepr): Geometría molecular (TRePEV). La geometría de una molécula por TRePEV, con ángulo e hibridación.
- [`orbitals`](referencia.md#orbitals): Orbitales. Diagramas de casilleros atómicos y de orbitales moleculares.
- [`unit_cell`](referencia.md#unit_cell): Celda unidad. Celdas cúbicas y redes iónicas, con átomos por celda, coordinación, empaquetamiento y densidad.

**Reacciones y cinética**

- [`reaction`](referencia.md#reaction): Reacción. Ecuaciones químicas, equilibrios, resonancia, mecanismos y productos para completar.
- [`energy_profile`](referencia.md#energy_profile): Perfil de energía. El perfil de energía de una reacción, con energías de activación y ΔH, catalizada o no.
- [`kinetics`](referencia.md#kinetics): Cinética. Concentración en el tiempo, linealizaciones, Arrhenius y reacciones consecutivas o reversibles.

**Termodinámica**

- [`phase_diagram`](referencia.md#phase_diagram): Diagrama de fases. Diagramas de fases de una sustancia y diagramas eutécticos de dos componentes.
- [`heating_curve`](referencia.md#heating_curve): Curva de calentamiento. La temperatura en función del calor entregado, con los cambios de fase.

**Equilibrio y análisis**

- [`titration`](referencia.md#titration): Titulación. Curvas de titulación de ácidos y bases, fuertes, débiles y polipróticos, con indicador.
- [`speciation`](referencia.md#speciation): Especiación. Las fracciones α de cada especie de un ácido en función del pH.
- [`spectrum`](referencia.md#spectrum): Espectro. Espectros IR, ¹H-RMN, ¹³C-RMN, UV-visible y de masas.

**Electroquímica**

- [`galvanic_cell`](referencia.md#galvanic_cell): Pila. Pilas galvánicas con semirreacciones, flujo de electrones, notación, fem y ΔG.
- [`latimer`](referencia.md#latimer): Diagrama de Latimer. Potenciales entre estados de oxidación y potenciales entre especies no vecinas.
- [`frost`](referencia.md#frost): Diagrama de Frost. N·E° en función del estado de oxidación, con las especies que se desproporcionan.

**Química orgánica**

- [`newman`](referencia.md#newman): Proyección de Newman. Conformaciones alrededor de un enlace C–C, con el diedro y su nombre.
- [`chair`](referencia.md#chair): Silla del ciclohexano. Conformaciones de silla con sustituyentes axiales y ecuatoriales, cis/trans y estabilidad.
- [`fischer`](referencia.md#fischer): Proyección de Fischer. Proyecciones de Fischer con R/S por las reglas CIP y la serie D/L.

**Química inorgánica**

- [`crystal_field`](referencia.md#crystal_field): Campo cristalino. El desdoblamiento de los orbitales d, alto y bajo espín, desapareados, μ y EECC.
- [`complex`](referencia.md#complex): Complejo de coordinación. La geometría de un complejo con su fórmula, estado de oxidación e isomería.
- [`born_haber`](referencia.md#born_haber): Ciclo de Born-Haber. La energía de red de un sólido iónico por la ley de Hess.

### Matemática aplicada

**Señales y sistemas**

- [`signal`](referencia.md#signal): Señal. Señales continuas y discretas definidas por una expresión.
- [`convolution`](referencia.md#convolution): Convolución. La convolución discreta, con el paso de voltear y desplazar.
- [`fourier`](referencia.md#fourier): Fourier. La DFT de una secuencia y la serie de Fourier de ondas clásicas.
- [`sampling`](referencia.md#sampling): Muestreo. El muestreo de un tono, el aliasing y el criterio de Nyquist.

**Probabilidad**

- [`distribution`](referencia.md#distribution): Distribución. Densidades y funciones de masa con el área de un intervalo y su probabilidad exacta.
- [`venn`](referencia.md#venn): Diagrama de Venn. Dos o tres conjuntos con las regiones de una expresión sombreadas.

**Circuitos eléctricos**

- [`circuit`](referencia.md#circuit): Circuito. Redes serie y paralelo con su fuente, impedancia equivalente y corriente.
- [`phasor`](referencia.md#phasor): Diagrama fasorial. Fasores en el plano complejo y su suma.

## Integración

La sintaxis del bloque es la misma en todos los destinos; cambia cómo se lo marca.

### Markdown y MyST (apuntes, sitios)

Un bloque de código con el lenguaje `estructura` (también `statics`; en MyST y en Typst, también
`dibu`):

````markdown
La pila después de las tres operaciones:

```estructura
type: stack
title: Pila después de apilar 3, 7 y 1
values: [3, 7, 1]
```
````

En **MyST** (mystmd), el plugin [dibu-myst](https://github.com/INGCOM-UNRN/dibu-myst) dibuja
esos bloques al construir. Para un diagrama con epígrafe, numerado y referenciable, la directiva
`{dibu}`, con las opciones antes del cuerpo:

````markdown
:::{dibu}
:caption: Pila después de apilar 3, 7 y 1.
:label: fig-pila
:width: 60%
type: stack
values: [3, 7, 1]
:::

Como muestra la [](#fig-pila), el tope es el último elemento apilado.
````

- Opciones de `{dibu}`: `:caption:`, `:label:`, `:width:`, `:align:`, `:alt:`, `:class:` y
  `:theme:`. El argumento puede ser un archivo YAML relativo a la página:
  ```` ```{dibu} diagramas/karnaugh.yaml ````.
- Para **mostrar la sintaxis** sin dibujarla (en una guía de dibu), usar ```` ```{code} estructura ````
  o ```` ```{code-block} estructura ````: quedan como código.

En **Markdown sin MyST**, `dibu sync-md <carpeta>` dibuja cada bloque ```` ```estructura ```` o
```` ```statics ```` en `<carpeta>/diagramas/estructura-<hash>.svg` (los 12 primeros dígitos del
SHA-1 del cuerpo); la herramienta del sitio enlaza esa imagen. `dibu sync-md --verificar` falla
si algún SVG falta o está desactualizado.

### Typst

Con el paquete [dibu-typst](https://github.com/INGCOM-UNRN/dibu-typst) (`@local/dibu` o
`@preview/dibu`):

```typ
#import "@preview/dibu:0.1.0": dibu, dibu-bloques, dibu-config

#show: dibu-bloques              // dibuja los bloques ```estructura, ```statics y ```dibu
#dibu-config(theme: "ink")       // theme de los diagramas siguientes

#dibu("type: stack\nvalues: [3, 7, 1]")                      // texto YAML
#dibu((type: "karnaugh", vars: ("A", "B", "C"), minterms: (1, 3, 5, 7)), width: 60%)
```

- Preferir el bloque raw ```` ```estructura ```` con `dibu-bloques`, o un diccionario de Typst:
  el YAML dentro de un string de Typst obliga a escapar comillas y saltos de línea.
- En un diccionario, los textos van entre comillas dobles de Typst y las listas son tuplas
  (`(1, 2, 3)`; una de un elemento es `(1,)`). Una arista de `dijkstra` como `"A-B:4"` es un texto.
- Un bloque inválido es un error de compilación con el mensaje de dibu.

### Bancos de preguntas (alucarD, GIFT)

En el enunciado, un bloque ```` ```estructura ```` como en Markdown. En un banco GIFT:

- sin renglones en blanco dentro del bloque (un renglón en blanco corta la pregunta);
- sin llaves: los objetos se escriben en forma de bloque, un campo por renglón (las llaves
  `{…}` delimitan las respuestas en GIFT);
- la sangría, con espacios.

Los bancos que genera alucarD (voight) escriben el YAML con caracteres de ancho completo
(`type： stack`, `values： ［5, 9, 2］`) y marcan cada fin de renglón con `↵`, para no chocar con
la sintaxis de GIFT; alucarD los normaliza antes de dibujar. Al editar un banco así, mantener esa
forma. `validar.py` normaliza los `.gift` igual que alucarD.

````text
```estructura
type： stack↵
values： ［5, 9, 2］↵
capacity： 5↵
```
````

La sangría con `·` (la que usa voight para el código C) **no** se convierte dentro de un bloque
```` ```estructura ````: el YAML queda inválido y el diagrama no se dibuja.

### Programas y lotes

```bash
dibu render bloques.yaml -o salida/        # un SVG por bloque: <id>.svg o <archivo>_<n>.svg
echo '{"id":"a","block":{"type":"array","values":[1,2,3]}}' | dibu batch   # JSONL → {"id","svg"|"error"}
dibu list-types                            # los tipos que trae este binario
```

Un archivo para `dibu render` puede tener un bloque, una lista de bloques o `{blocks: [...]}`.

## Validar

```bash
python3 validar.py apunte.md               # los bloques y directivas de un documento
python3 validar.py bloques.yaml            # un archivo de bloques
echo 'type: stak' | python3 validar.py -   # un bloque suelto
python3 validar.py --json capitulo/*.md    # para procesar el resultado
```

Por cada problema escribe `archivo:línea: error|aviso: mensaje`:

- **error**: el bloque no se dibuja (YAML inválido, tipo inexistente, un dato que dibu rechaza:
  `cache: 'cacheSize' tiene que ser potencia de 2 (es 100)`).
- **aviso**: se dibuja, pero probablemente distinto de lo pedido (un campo que dibu ignora, un
  valor fuera de la lista, `yes` en lugar de `true`).

Termina con 1 si hay errores (con `--estricto`, también si hay avisos). Necesita Python 3.9 y
PyYAML; para dibujar, el binario `dibu` en el PATH o en `DIBU_BIN` (sin él, sólo contrasta con
el esquema).

Antes de entregar: ningún error, ningún aviso sin explicar, y el documento construye.

## Errores frecuentes

| Síntoma | Causa | Corrección |
| :--- | :--- | :--- |
| `no existe el tipo 'stak'` | Tipo mal escrito o inventado. | Elegirlo del [catálogo](#catálogo-de-tipos). |
| `dibu ignora el campo 'colour'` | Campo de otro tipo, en otro idioma o inventado. | Usar los campos de `referencia.md` para ese tipo. |
| `'variant: circularr' no es un valor admitido` | Valor fuera de la lista. | Usar uno de los valores que lista el aviso. |
| `'showAddresses' espera true o false` | `yes`, `"true"` o `1` en lugar de `true`. | `true` o `false`, sin comillas. |
| YAML inválido en `title: max: 3` | Texto con `:` sin comillas. | `title: "max: 3"`. |
| El diagrama no coincide con el texto | Datos copiados del ejemplo sin cambiar, o resultado precalculado. | Dar los datos del problema; dibu calcula. |
| `trace_table: la traza no tiene filas` | Faltan los datos que el tipo necesita. | Ver los campos del tipo: `columns` y `rows`, u `operations`. |
| El bloque aparece como código en MyST | Lenguaje mal escrito, o `{code}`. | ```` ```estructura ```` o `{dibu}`. |
