[Manual](../README.md)

# Guía de dibu para agentes de IA

Esta guía es para un agente (o una persona) que redacta apuntes, guías o exámenes y necesita
**escribir diagramas de dibu e integrarlos** en el documento. Reúne en un lugar la sintaxis, el
catálogo de los {{TOTAL}} tipos, cómo se inserta un diagrama en cada destino y cómo comprobarlo
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
   el texto que lo rodea. Para una persona, el [editor web](https://ingcom-unrn.github.io/dibu-web/)
   es el playground: abre un bloque con vista previa en vivo, y cada ejemplo del
   [manual](https://ingcom-unrn.github.io/dibu/) tiene un enlace «Abrir en el editor».

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
  bloque completo y agregarle `blank`. Para un valor concreto, en cualquier tipo: `!blank 40` (o
  `40.blank`) lo usa para calcular y lo dibuja en blanco; si ese lugar no lo admite, dibu da un
  error que dice cuál es, nunca lo muestra.
- **Tamaños:** cada tipo tiene cotas (filas, nodos, muestras). Un diagrama para un apunte entra
  en una página: preferir pocos elementos representativos.

## Campos comunes

Todos los tipos aceptan, además de los propios:

| Campo | Tipo | Descripción |
| :--- | :--- | :--- |
| `type` | texto | El tipo de diagrama. Obligatorio. |
{{COMUNES}}

## Catálogo de tipos

{{CATALOGO}}

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

En **Markdown sin MyST**, `dibu sync-md <carpeta>` dibuja cada bloque ```` ```dibu ````,
```` ```estructura ```` o ```` ```statics ```` en `<carpeta>/diagramas/estructura-<hash>.svg` (los 12 primeros dígitos del
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
