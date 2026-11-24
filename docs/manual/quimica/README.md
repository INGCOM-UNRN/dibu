[Manual](../README.md)

# Química

Química general, fisicoquímica, analítica, orgánica e inorgánica. dibu calcula el pH de cada
punto de una titulación, las configuraciones electrónicas, el llenado de los orbitales
moleculares, la cinética, la pila con Nernst, la densidad de una celda unidad, R/S y la EECC.
Los rótulos aceptan fórmulas: `H2SO4`, `SO4^2-`, `K_a`.

21 tipos, en 7 grupos. Crate: `dibu-chem`.

## [Estructura y enlace](estructura/README.md)

Fórmulas de Lewis y esqueletales, geometría molecular, orbitales y celdas unidad.

- [`molecule`](estructura/molecule.md): Molécula. Fórmulas de Lewis y esqueletales, con cargas, pares libres, estereoquímica y flechas de mecanismo.
- [`vsepr`](estructura/vsepr.md): Geometría molecular (TRePEV). La geometría de una molécula por TRePEV, con ángulo e hibridación.
- [`orbitals`](estructura/orbitals.md): Orbitales. Diagramas de casilleros atómicos y de orbitales moleculares.
- [`unit_cell`](estructura/unit_cell.md): Celda unidad. Celdas cúbicas y redes iónicas, con átomos por celda, coordinación, empaquetamiento y densidad.

## [Reacciones y cinética](reacciones/README.md)

Ecuaciones y mecanismos, perfiles de energía y cinética.

- [`reaction`](reacciones/reaction.md): Reacción. Ecuaciones químicas, equilibrios, resonancia, mecanismos y productos para completar.
- [`energy_profile`](reacciones/energy_profile.md): Perfil de energía. El perfil de energía de una reacción, con energías de activación y ΔH, catalizada o no.
- [`kinetics`](reacciones/kinetics.md): Cinética. Concentración en el tiempo, linealizaciones, Arrhenius y reacciones consecutivas o reversibles.

## [Termodinámica](termodinamica/README.md)

Diagramas de fases y curvas de calentamiento.

- [`phase_diagram`](termodinamica/phase_diagram.md): Diagrama de fases. Diagramas de fases de una sustancia y diagramas eutécticos de dos componentes.
- [`heating_curve`](termodinamica/heating_curve.md): Curva de calentamiento. La temperatura en función del calor entregado, con los cambios de fase.

## [Equilibrio y análisis](equilibrio/README.md)

Titulaciones, especiación y espectros (IR, RMN, UV-visible, masas).

- [`titration`](equilibrio/titration.md): Titulación. Curvas de titulación de ácidos y bases, fuertes, débiles y polipróticos, con indicador.
- [`speciation`](equilibrio/speciation.md): Especiación. Las fracciones α de cada especie de un ácido en función del pH.
- [`spectrum`](equilibrio/spectrum.md): Espectro. Espectros IR, ¹H-RMN, ¹³C-RMN, UV-visible y de masas.

## [Electroquímica](electroquimica/README.md)

Pilas, y diagramas de Latimer y de Frost.

- [`galvanic_cell`](electroquimica/galvanic_cell.md): Pila. Pilas galvánicas con semirreacciones, flujo de electrones, notación, fem y ΔG.
- [`latimer`](electroquimica/latimer.md): Diagrama de Latimer. Potenciales entre estados de oxidación y potenciales entre especies no vecinas.
- [`frost`](electroquimica/frost.md): Diagrama de Frost. N·E° en función del estado de oxidación, con las especies que se desproporcionan.

## [Química orgánica](organica/README.md)

Conformaciones y estereoquímica.

- [`newman`](organica/newman.md): Proyección de Newman. Conformaciones alrededor de un enlace C–C, con el diedro y su nombre.
- [`chair`](organica/chair.md): Silla del ciclohexano. Conformaciones de silla con sustituyentes axiales y ecuatoriales, cis/trans y estabilidad.
- [`fischer`](organica/fischer.md): Proyección de Fischer. Proyecciones de Fischer con R/S por las reglas CIP y la serie D/L.

## [Química inorgánica](inorganica/README.md)

Campo cristalino, complejos de coordinación y ciclos de Born-Haber.

- [`crystal_field`](inorganica/crystal_field.md): Campo cristalino. El desdoblamiento de los orbitales d, alto y bajo espín, desapareados, μ y EECC.
- [`complex`](inorganica/complex.md): Complejo de coordinación. La geometría de un complejo con su fórmula, estado de oxidación e isomería.
- [`born_haber`](inorganica/born_haber.md): Ciclo de Born-Haber. La energía de red de un sólido iónico por la ley de Hess.
