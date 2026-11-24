[Manual](../README.md)

# Matemática aplicada

Señales y sistemas, probabilidad y circuitos eléctricos. Las señales se escriben como
expresiones, las probabilidades se calculan exactas y los circuitos se resuelven con
impedancias complejas.

8 tipos, en 3 grupos. Crate: `dibu-math`.

## [Señales y sistemas](senales/README.md)

Señales continuas y discretas, convolución, Fourier y muestreo.

- [`signal`](senales/signal.md): Señal. Señales continuas y discretas definidas por una expresión.
- [`convolution`](senales/convolution.md): Convolución. La convolución discreta, con el paso de voltear y desplazar.
- [`fourier`](senales/fourier.md): Fourier. La DFT de una secuencia y la serie de Fourier de ondas clásicas.
- [`sampling`](senales/sampling.md): Muestreo. El muestreo de un tono, el aliasing y el criterio de Nyquist.

## [Probabilidad](probabilidad/README.md)

Distribuciones con su área sombreada y diagramas de Venn.

- [`distribution`](probabilidad/distribution.md): Distribución. Densidades y funciones de masa con el área de un intervalo y su probabilidad exacta.
- [`venn`](probabilidad/venn.md): Diagrama de Venn. Dos o tres conjuntos con las regiones de una expresión sombreadas.

## [Circuitos eléctricos](circuitos/README.md)

Redes serie y paralelo, impedancia equivalente y fasores.

- [`circuit`](circuitos/circuit.md): Circuito. Redes serie y paralelo con su fuente, impedancia equivalente y corriente.
- [`phasor`](circuitos/phasor.md): Diagrama fasorial. Fasores en el plano complejo y su suma.
