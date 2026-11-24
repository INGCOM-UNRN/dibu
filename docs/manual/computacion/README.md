[Manual](../README.md)

# Sistemas de computación

Sistemas digitales, arquitectura de computadoras, sistemas operativos, redes y bases de
datos. Cada diagrama simula o calcula a partir de los datos: minimiza la función, planifica
los procesos, simula la caché, asigna las subredes.

11 tipos, en 5 grupos. Crate: `dibu-comp`.

## [Sistemas digitales](digital/README.md)

Álgebra de Boole, tablas de verdad, mapas de Karnaugh y circuitos con compuertas.

- [`truth_table`](digital/truth_table.md): Tabla de verdad. Tablas de verdad de una o varias funciones, con columnas intermedias y formas canónicas.
- [`karnaugh`](digital/karnaugh.md): Mapa de Karnaugh. Mapas de 2 a 4 variables con los agrupamientos mínimos y la expresión simplificada.
- [`logic_circuit`](digital/logic_circuit.md): Circuito lógico. El circuito con compuertas que implementa una expresión booleana.

## [Arquitectura de computadoras](arquitectura/README.md)

Segmentación, punto flotante IEEE 754 y memoria caché.

- [`pipeline`](arquitectura/pipeline.md): Pipeline. El diagrama de ciclos de un pipeline de 5 etapas, con riesgos, burbujas y forwarding.
- [`ieee754`](arquitectura/ieee754.md): IEEE 754. La codificación de un número en punto flotante, en los dos sentidos.
- [`cache`](arquitectura/cache.md): Memoria caché. La simulación de una caché directa o asociativa sobre una secuencia de accesos.

## [Sistemas operativos](sistemas-operativos/README.md)

Planificación de la CPU, paginación y reemplazo de páginas.

- [`cpu_scheduling`](sistemas-operativos/cpu_scheduling.md): Planificación de CPU. El diagrama de Gantt y los tiempos de FCFS, SJF, SRTF, Round Robin y prioridades.
- [`page_table`](sistemas-operativos/page_table.md): Tabla de páginas. La traducción de direcciones lógicas a físicas con paginación.
- [`page_replacement`](sistemas-operativos/page_replacement.md): Reemplazo de páginas. FIFO, LRU, óptimo y reloj sobre una cadena de referencias, para compararlos.

## [Redes](redes/README.md)

Direccionamiento IPv4 y subnetting.

- [`subnetting`](redes/subnetting.md): Subnetting. Subredes IPv4 de igual tamaño o con VLSM, con su mapa de direcciones.

## [Bases de datos](bases-de-datos/README.md)

Control de concurrencia y serializabilidad.

- [`transaction_schedule`](bases-de-datos/transaction_schedule.md): Planificación de transacciones. Conflictos, grafo de precedencia y serializabilidad por conflictos.
