[Manual](../../README.md) › [Sistemas de computación](../README.md) › [Arquitectura de computadoras](README.md)

# Memoria caché (`cache`)

Divide la dirección en tag, índice y offset, simula cada acceso y dibuja una tabla con su
tag, índice, offset, si fue acierto o fallo (y de qué tipo) y qué bloque desalojó. Debajo, la
tasa de aciertos y el contenido final de la caché.

## Qué calcula

- Los bits de tag, índice y offset a partir de los tamaños.
- Aciertos y fallos con reemplazo LRU o FIFO.
- El tipo de cada fallo: forzoso (primer acceso al bloque), de capacidad (fallaría también en una caché totalmente asociativa del mismo tamaño) o de conflicto.

## Campos

| Campo | Tipo | Valores y valor por defecto | Descripción |
| :--- | :--- | :--- | :--- |
| `type` | texto | `cache` | Obligatorio. |
| `addressBits` | número |  | Bits de la dirección (de 4 a 64, por omisión 16). |
| `cacheSize` | número |  | Tamaño de la caché en bytes, potencia de 2 (`64`, `"1KB"`). |
| `blockSize` | número |  | Tamaño de bloque en bytes, potencia de 2. |
| `associativity` | número |  | `1` (correspondencia directa), 2, 4… vías o `"full"` (totalmente asociativa). |
| `policy` | texto | `lru`, `fifo` · por defecto `lru` | `lru` o `fifo`. |
| `accesses` | lista |  | Las direcciones accedidas: `"0x040"`, `"0b1010"` o enteros (hasta 64). |
| `missTypes` | sí/no |  | La columna con el tipo de fallo (por omisión, sí). |
| `finalState` | sí/no |  | La tabla con el contenido final (por omisión, sí; hasta 64 bloques). |
| `blank` | sí/no |  | Deja en blanco tag, índice, offset, resultado y el contenido final. |

Además, los campos comunes a todos los tipos: `title`, `subtitle`, `id`, `inkSaver` y `theme` (ver [Campos comunes](../../campos-comunes.md)).

## Modo ejercicio

Campos que dejan casilleros para completar: `blank`.

## Ejemplos

### Caché de correspondencia directa

```yaml
type: cache
title: Caché de correspondencia directa
addressBits: 12
cacheSize: 64
blockSize: 8
associativity: 1
accesses: ["0x000", "0x004", "0x010", "0x040", "0x000", "0x048", "0x010", "0x080", "0x044"]
```

[![Caché de correspondencia directa](img/ks_arq_cache_directa.svg)](https://ingcom-unrn.github.io/dibu-web/#z=XYsxDsIwEAT7e4XlmsJBFrJSwhMoEYU5n8SJKI7sU0T4Ee_gY5wCuKDb2d2RZaLeYMQrgbAMCgeF19MkMphLoTrlMdGIHE3iQigRYkra1z1L7U23hVU_8kPlnYfLkPH2oQCx1qyq8Myy6Bkioqqk4sm6u3PObswa_Dd0v8a3qTXh_xPa5O0Z3g)

[Abrir en el editor](https://ingcom-unrn.github.io/dibu-web/#z=XYsxDsIwEAT7e4XlmsJBFrJSwhMoEYU5n8SJKI7sU0T4Ee_gY5wCuKDb2d2RZaLeYMQrgbAMCgeF19MkMphLoTrlMdGIHE3iQigRYkra1z1L7U23hVU_8kPlnYfLkPH2oQCx1qyq8Myy6Bkioqqk4sm6u3PObswa_Dd0v8a3qTXh_xPa5O0Z3g) para explorarlo y modificarlo.

### La misma secuencia con 2 vías (LRU)

```yaml
type: cache
title: La misma secuencia con 2 vías (LRU)
addressBits: 12
cacheSize: 64
blockSize: 8
associativity: 2
accesses: ["0x000", "0x004", "0x010", "0x040", "0x000", "0x048", "0x010", "0x080", "0x044"]
finalState: false
```

[![La misma secuencia con 2 vías (LRU)](img/ks_arq_cache_2vias.svg)](https://ingcom-unrn.github.io/dibu-web/#z=XY09DoJAEIX7OcWEShOLhWwMobSmklgZi3FY4sYFjDMS8U6ewou5QaWwez_zvdHx4gpk4pMD9RqiKQlbLy2hOL65jj0h9x1mOLyeJLgot7slUF1fncjGqxSYZjAtVP4R-bWFY-j5_HE5kEgfR9QPXscCMyDmiLoI7hNzN8YkK5yE_Yr0l9i5mpP8_yafK5scoPEdhUpJ4-uGgjh4Aw)

[Abrir en el editor](https://ingcom-unrn.github.io/dibu-web/#z=XY09DoJAEIX7OcWEShOLhWwMobSmklgZi3FY4sYFjDMS8U6ewou5QaWwez_zvdHx4gpk4pMD9RqiKQlbLy2hOL65jj0h9x1mOLyeJLgot7slUF1fncjGqxSYZjAtVP4R-bWFY-j5_HE5kEgfR9QPXscCMyDmiLoI7hNzN8YkK5yE_Yr0l9i5mpP8_yafK5scoPEdhUpJ4-uGgjh4Aw) para explorarlo y modificarlo.

## Ver también

[`pipeline`](pipeline.md), [`ieee754`](ieee754.md)
