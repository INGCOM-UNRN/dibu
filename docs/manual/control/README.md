[Manual](../README.md)

# Control de procesos

Los temas de una asignatura de control: modelado con diagramas de bloques, respuesta
temporal, análisis en frecuencia, el plano s y la automatización. dibu simula la respuesta,
evalúa la respuesta en frecuencia y calcula raíces y márgenes, así que el dibujo coincide con
la teoría.

9 tipos, en 5 grupos. Crate: `dibu-control`.

## [Modelado](modelado/README.md)

Diagramas de bloques con realimentación, perturbaciones y lazos en cascada.

- [`block_diagram`](modelado/block_diagram.md): Diagrama de bloques. Lazos de control con realimentación, perturbaciones y lazos en cascada.

## [Respuesta temporal](respuesta/README.md)

La salida de un sistema ante escalón, rampa o impulso, con sus parámetros.

- [`time_response`](respuesta/time_response.md): Respuesta temporal. La salida de un sistema ante escalón, rampa o impulso, con sobrepico, tiempos y constantes.

## [Respuesta en frecuencia](frecuencia/README.md)

Diagramas de Bode y de Nyquist, con márgenes de estabilidad.

- [`bode`](frecuencia/bode.md): Diagrama de Bode. Magnitud y fase en frecuencia, con asíntotas y márgenes de estabilidad.
- [`nyquist`](frecuencia/nyquist.md): Diagrama de Nyquist. La curva de Nyquist del lazo abierto, con el círculo unitario y las ω negativas.

## [Plano s](plano-s/README.md)

Polos y ceros, y lugar de las raíces.

- [`pole_zero`](plano-s/pole_zero.md): Polos y ceros. Los polos y ceros de un sistema en el plano s.
- [`root_locus`](plano-s/root_locus.md): Lugar de las raíces. El lugar de las raíces con asíntotas, cruce con el eje imaginario y ganancias marcadas.

## [Instrumentación y automatización](automatizacion/README.md)

P&ID con símbolos ISA-5.1, diagramas escalera de PLC y GRAFCET.

- [`instrumentation`](automatizacion/instrumentation.md): P&ID. Diagramas de instrumentación con símbolos ISA-5.1.
- [`ladder`](automatizacion/ladder.md): Diagrama escalera. Programas de PLC en lenguaje escalera, con contactos, bobinas, temporizadores y contadores.
- [`grafcet`](automatizacion/grafcet.md): GRAFCET. Etapas, transiciones y divergencias en O y en Y.
