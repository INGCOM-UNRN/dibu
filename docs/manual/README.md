# Manual de dibu

dibu dibuja **70 tipos** de diagramas educativos como SVG a partir de bloques YAML o JSON. Cada tipo parte de los datos del problema y calcula lo que dibuja, así que el mismo bloque sirve para el apunte, el ejercicio (con los casilleros en blanco) y la clave.

## Primeros pasos

- [Formato de los bloques](bloques.md): cómo se escribe un bloque, en YAML, JSON o Markdown.
- [Campos comunes](campos-comunes.md): `title`, `subtitle`, `id`, `inkSaver` y `theme`.
- [Modo ejercicio](ejercicios.md): `blank` y los campos `blank*`.
- [Themes](themes.md): presets, themes propios y tipografía.
- [Línea de comandos](cli.md): `render`, `batch`, `sync-md`, `list-types` y `list-themes`.
- [Guía para agentes de IA](agentes/README.md): reglas, referencia compacta de todos los tipos, integración en Markdown, MyST, Typst y alucarD, y un validador de bloques.

## Tipos de diagramas

### [Estructuras de datos y algoritmos](estructuras/README.md)

- **[Estructuras lineales](estructuras/lineales/README.md)**: [`array`](estructuras/lineales/array.md), [`matrix`](estructuras/lineales/matrix.md), [`linked_list`](estructuras/lineales/linked_list.md), [`stack`](estructuras/lineales/stack.md), [`queue`](estructuras/lineales/queue.md)
- **[Memoria, tipos y recursión](estructuras/memoria/README.md)**: [`call_stack`](estructuras/memoria/call_stack.md), [`call_tree`](estructuras/memoria/call_tree.md), [`struct`](estructuras/memoria/struct.md), [`adt`](estructuras/memoria/adt.md)
- **[Árboles y tablas](estructuras/arboles/README.md)**: [`tree`](estructuras/arboles/tree.md), [`heap`](estructuras/arboles/heap.md), [`avl`](estructuras/arboles/avl.md), [`bplus_tree`](estructuras/arboles/bplus_tree.md), [`hash_table`](estructuras/arboles/hash_table.md)
- **[Análisis de algoritmos](estructuras/algoritmos/README.md)**: [`complexity`](estructuras/algoritmos/complexity.md), [`flowchart`](estructuras/algoritmos/flowchart.md), [`cost_table`](estructuras/algoritmos/cost_table.md), [`trace_table`](estructuras/algoritmos/trace_table.md), [`dp_table`](estructuras/algoritmos/dp_table.md), [`dijkstra`](estructuras/algoritmos/dijkstra.md)
- **[Computabilidad](estructuras/computabilidad/README.md)**: [`turing`](estructuras/computabilidad/turing.md)

### [Sistemas de computación](computacion/README.md)

- **[Sistemas digitales](computacion/digital/README.md)**: [`truth_table`](computacion/digital/truth_table.md), [`karnaugh`](computacion/digital/karnaugh.md), [`logic_circuit`](computacion/digital/logic_circuit.md)
- **[Arquitectura de computadoras](computacion/arquitectura/README.md)**: [`pipeline`](computacion/arquitectura/pipeline.md), [`ieee754`](computacion/arquitectura/ieee754.md), [`cache`](computacion/arquitectura/cache.md)
- **[Sistemas operativos](computacion/sistemas-operativos/README.md)**: [`cpu_scheduling`](computacion/sistemas-operativos/cpu_scheduling.md), [`page_table`](computacion/sistemas-operativos/page_table.md), [`page_replacement`](computacion/sistemas-operativos/page_replacement.md)
- **[Redes](computacion/redes/README.md)**: [`subnetting`](computacion/redes/subnetting.md)
- **[Bases de datos](computacion/bases-de-datos/README.md)**: [`transaction_schedule`](computacion/bases-de-datos/transaction_schedule.md)

### [Control de procesos](control/README.md)

- **[Modelado](control/modelado/README.md)**: [`block_diagram`](control/modelado/block_diagram.md)
- **[Respuesta temporal](control/respuesta/README.md)**: [`time_response`](control/respuesta/time_response.md)
- **[Respuesta en frecuencia](control/frecuencia/README.md)**: [`bode`](control/frecuencia/bode.md), [`nyquist`](control/frecuencia/nyquist.md)
- **[Plano s](control/plano-s/README.md)**: [`pole_zero`](control/plano-s/pole_zero.md), [`root_locus`](control/plano-s/root_locus.md)
- **[Instrumentación y automatización](control/automatizacion/README.md)**: [`instrumentation`](control/automatizacion/instrumentation.md), [`ladder`](control/automatizacion/ladder.md), [`grafcet`](control/automatizacion/grafcet.md)

### [Química](quimica/README.md)

- **[Estructura y enlace](quimica/estructura/README.md)**: [`molecule`](quimica/estructura/molecule.md), [`vsepr`](quimica/estructura/vsepr.md), [`orbitals`](quimica/estructura/orbitals.md), [`unit_cell`](quimica/estructura/unit_cell.md)
- **[Reacciones y cinética](quimica/reacciones/README.md)**: [`reaction`](quimica/reacciones/reaction.md), [`energy_profile`](quimica/reacciones/energy_profile.md), [`kinetics`](quimica/reacciones/kinetics.md)
- **[Termodinámica](quimica/termodinamica/README.md)**: [`phase_diagram`](quimica/termodinamica/phase_diagram.md), [`heating_curve`](quimica/termodinamica/heating_curve.md)
- **[Equilibrio y análisis](quimica/equilibrio/README.md)**: [`titration`](quimica/equilibrio/titration.md), [`speciation`](quimica/equilibrio/speciation.md), [`spectrum`](quimica/equilibrio/spectrum.md)
- **[Electroquímica](quimica/electroquimica/README.md)**: [`galvanic_cell`](quimica/electroquimica/galvanic_cell.md), [`latimer`](quimica/electroquimica/latimer.md), [`frost`](quimica/electroquimica/frost.md)
- **[Química orgánica](quimica/organica/README.md)**: [`newman`](quimica/organica/newman.md), [`chair`](quimica/organica/chair.md), [`fischer`](quimica/organica/fischer.md)
- **[Química inorgánica](quimica/inorganica/README.md)**: [`crystal_field`](quimica/inorganica/crystal_field.md), [`complex`](quimica/inorganica/complex.md), [`born_haber`](quimica/inorganica/born_haber.md)

### [Matemática aplicada](matematica/README.md)

- **[Señales y sistemas](matematica/senales/README.md)**: [`signal`](matematica/senales/signal.md), [`convolution`](matematica/senales/convolution.md), [`fourier`](matematica/senales/fourier.md), [`sampling`](matematica/senales/sampling.md)
- **[Probabilidad](matematica/probabilidad/README.md)**: [`distribution`](matematica/probabilidad/distribution.md), [`venn`](matematica/probabilidad/venn.md)
- **[Circuitos eléctricos](matematica/circuitos/README.md)**: [`circuit`](matematica/circuitos/circuit.md), [`phasor`](matematica/circuitos/phasor.md)

## Índice alfabético

| Tipo | Diagrama | Área |
| :--- | :--- | :--- |
| [`adt`](estructuras/memoria/adt.md) | Tipo abstracto de datos | Estructuras de datos y algoritmos › Memoria, tipos y recursión |
| [`array`](estructuras/lineales/array.md) | Arreglo | Estructuras de datos y algoritmos › Estructuras lineales |
| [`avl`](estructuras/arboles/avl.md) | Árbol AVL | Estructuras de datos y algoritmos › Árboles y tablas |
| [`block_diagram`](control/modelado/block_diagram.md) | Diagrama de bloques | Control de procesos › Modelado |
| [`bode`](control/frecuencia/bode.md) | Diagrama de Bode | Control de procesos › Respuesta en frecuencia |
| [`born_haber`](quimica/inorganica/born_haber.md) | Ciclo de Born-Haber | Química › Química inorgánica |
| [`bplus_tree`](estructuras/arboles/bplus_tree.md) | Árbol B+ | Estructuras de datos y algoritmos › Árboles y tablas |
| [`cache`](computacion/arquitectura/cache.md) | Memoria caché | Sistemas de computación › Arquitectura de computadoras |
| [`call_stack`](estructuras/memoria/call_stack.md) | Pila de llamadas | Estructuras de datos y algoritmos › Memoria, tipos y recursión |
| [`call_tree`](estructuras/memoria/call_tree.md) | Árbol de llamadas | Estructuras de datos y algoritmos › Memoria, tipos y recursión |
| [`chair`](quimica/organica/chair.md) | Silla del ciclohexano | Química › Química orgánica |
| [`circuit`](matematica/circuitos/circuit.md) | Circuito | Matemática aplicada › Circuitos eléctricos |
| [`complex`](quimica/inorganica/complex.md) | Complejo de coordinación | Química › Química inorgánica |
| [`complexity`](estructuras/algoritmos/complexity.md) | Complejidad | Estructuras de datos y algoritmos › Análisis de algoritmos |
| [`convolution`](matematica/senales/convolution.md) | Convolución | Matemática aplicada › Señales y sistemas |
| [`cost_table`](estructuras/algoritmos/cost_table.md) | Tabla de costos | Estructuras de datos y algoritmos › Análisis de algoritmos |
| [`cpu_scheduling`](computacion/sistemas-operativos/cpu_scheduling.md) | Planificación de CPU | Sistemas de computación › Sistemas operativos |
| [`crystal_field`](quimica/inorganica/crystal_field.md) | Campo cristalino | Química › Química inorgánica |
| [`dijkstra`](estructuras/algoritmos/dijkstra.md) | Dijkstra | Estructuras de datos y algoritmos › Análisis de algoritmos |
| [`distribution`](matematica/probabilidad/distribution.md) | Distribución | Matemática aplicada › Probabilidad |
| [`dp_table`](estructuras/algoritmos/dp_table.md) | Tabla de programación dinámica | Estructuras de datos y algoritmos › Análisis de algoritmos |
| [`energy_profile`](quimica/reacciones/energy_profile.md) | Perfil de energía | Química › Reacciones y cinética |
| [`fischer`](quimica/organica/fischer.md) | Proyección de Fischer | Química › Química orgánica |
| [`flowchart`](estructuras/algoritmos/flowchart.md) | Diagrama de flujo | Estructuras de datos y algoritmos › Análisis de algoritmos |
| [`fourier`](matematica/senales/fourier.md) | Fourier | Matemática aplicada › Señales y sistemas |
| [`frost`](quimica/electroquimica/frost.md) | Diagrama de Frost | Química › Electroquímica |
| [`galvanic_cell`](quimica/electroquimica/galvanic_cell.md) | Pila | Química › Electroquímica |
| [`grafcet`](control/automatizacion/grafcet.md) | GRAFCET | Control de procesos › Instrumentación y automatización |
| [`hash_table`](estructuras/arboles/hash_table.md) | Tabla de hash | Estructuras de datos y algoritmos › Árboles y tablas |
| [`heap`](estructuras/arboles/heap.md) | Montículo (heap) | Estructuras de datos y algoritmos › Árboles y tablas |
| [`heating_curve`](quimica/termodinamica/heating_curve.md) | Curva de calentamiento | Química › Termodinámica |
| [`ieee754`](computacion/arquitectura/ieee754.md) | IEEE 754 | Sistemas de computación › Arquitectura de computadoras |
| [`instrumentation`](control/automatizacion/instrumentation.md) | P&ID | Control de procesos › Instrumentación y automatización |
| [`karnaugh`](computacion/digital/karnaugh.md) | Mapa de Karnaugh | Sistemas de computación › Sistemas digitales |
| [`kinetics`](quimica/reacciones/kinetics.md) | Cinética | Química › Reacciones y cinética |
| [`ladder`](control/automatizacion/ladder.md) | Diagrama escalera | Control de procesos › Instrumentación y automatización |
| [`latimer`](quimica/electroquimica/latimer.md) | Diagrama de Latimer | Química › Electroquímica |
| [`linked_list`](estructuras/lineales/linked_list.md) | Lista enlazada | Estructuras de datos y algoritmos › Estructuras lineales |
| [`logic_circuit`](computacion/digital/logic_circuit.md) | Circuito lógico | Sistemas de computación › Sistemas digitales |
| [`matrix`](estructuras/lineales/matrix.md) | Matriz | Estructuras de datos y algoritmos › Estructuras lineales |
| [`molecule`](quimica/estructura/molecule.md) | Molécula | Química › Estructura y enlace |
| [`newman`](quimica/organica/newman.md) | Proyección de Newman | Química › Química orgánica |
| [`nyquist`](control/frecuencia/nyquist.md) | Diagrama de Nyquist | Control de procesos › Respuesta en frecuencia |
| [`orbitals`](quimica/estructura/orbitals.md) | Orbitales | Química › Estructura y enlace |
| [`page_replacement`](computacion/sistemas-operativos/page_replacement.md) | Reemplazo de páginas | Sistemas de computación › Sistemas operativos |
| [`page_table`](computacion/sistemas-operativos/page_table.md) | Tabla de páginas | Sistemas de computación › Sistemas operativos |
| [`phase_diagram`](quimica/termodinamica/phase_diagram.md) | Diagrama de fases | Química › Termodinámica |
| [`phasor`](matematica/circuitos/phasor.md) | Diagrama fasorial | Matemática aplicada › Circuitos eléctricos |
| [`pipeline`](computacion/arquitectura/pipeline.md) | Pipeline | Sistemas de computación › Arquitectura de computadoras |
| [`pole_zero`](control/plano-s/pole_zero.md) | Polos y ceros | Control de procesos › Plano s |
| [`queue`](estructuras/lineales/queue.md) | Cola | Estructuras de datos y algoritmos › Estructuras lineales |
| [`reaction`](quimica/reacciones/reaction.md) | Reacción | Química › Reacciones y cinética |
| [`root_locus`](control/plano-s/root_locus.md) | Lugar de las raíces | Control de procesos › Plano s |
| [`sampling`](matematica/senales/sampling.md) | Muestreo | Matemática aplicada › Señales y sistemas |
| [`signal`](matematica/senales/signal.md) | Señal | Matemática aplicada › Señales y sistemas |
| [`speciation`](quimica/equilibrio/speciation.md) | Especiación | Química › Equilibrio y análisis |
| [`spectrum`](quimica/equilibrio/spectrum.md) | Espectro | Química › Equilibrio y análisis |
| [`stack`](estructuras/lineales/stack.md) | Pila | Estructuras de datos y algoritmos › Estructuras lineales |
| [`struct`](estructuras/memoria/struct.md) | Struct | Estructuras de datos y algoritmos › Memoria, tipos y recursión |
| [`subnetting`](computacion/redes/subnetting.md) | Subnetting | Sistemas de computación › Redes |
| [`time_response`](control/respuesta/time_response.md) | Respuesta temporal | Control de procesos › Respuesta temporal |
| [`titration`](quimica/equilibrio/titration.md) | Titulación | Química › Equilibrio y análisis |
| [`trace_table`](estructuras/algoritmos/trace_table.md) | Tabla de traza | Estructuras de datos y algoritmos › Análisis de algoritmos |
| [`transaction_schedule`](computacion/bases-de-datos/transaction_schedule.md) | Planificación de transacciones | Sistemas de computación › Bases de datos |
| [`tree`](estructuras/arboles/tree.md) | Árbol binario | Estructuras de datos y algoritmos › Árboles y tablas |
| [`truth_table`](computacion/digital/truth_table.md) | Tabla de verdad | Sistemas de computación › Sistemas digitales |
| [`turing`](estructuras/computabilidad/turing.md) | Máquina de Turing | Estructuras de datos y algoritmos › Computabilidad |
| [`unit_cell`](quimica/estructura/unit_cell.md) | Celda unidad | Química › Estructura y enlace |
| [`venn`](matematica/probabilidad/venn.md) | Diagrama de Venn | Matemática aplicada › Probabilidad |
| [`vsepr`](quimica/estructura/vsepr.md) | Geometría molecular (TRePEV) | Química › Estructura y enlace |

---

Este manual se genera con `python3 docs/manual/generar.py` a partir de `docs/manual/datos/`, del código y de los ejemplos de `fixtures/kitchen_sink/kitchen_sink.yaml`; no se edita a mano.
