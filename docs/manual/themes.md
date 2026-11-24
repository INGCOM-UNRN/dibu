[Manual](README.md)

# Themes

Cada diagrama se dibuja con un **theme**: una paleta semántica y una tipografía.

| Preset | Uso |
| :--- | :--- |
| `ink` (`tinta`) | Por omisión: grises y negro, pensado para imprimir. |
| `color` | Acentos azules (lo mismo que `--color`). |
| `high-contrast` (`alto-contraste`) | Todo en negro sobre blanco, para proyectar o fotocopiar. |
| `dark` (`oscuro`) | Fondo oscuro, para pantallas y presentaciones. |

## Elegir el theme

De mayor a menor prioridad:

1. `theme:` dentro del bloque: un preset o un objeto como el de abajo.
2. `inkSaver: true|false` dentro del bloque: `ink` o `color`, conservando la tipografía.
3. `--theme <preset o archivo>` en `render` y `sync-md`, o la variable `DIBU_THEME`.
4. `--color`, o `ink` si no hay nada.

En `dibu batch`, cada línea JSONL puede traer su propio `theme`.

## Themes propios

Un archivo YAML o JSON parte de un preset con `extends` y cambia sólo lo que dice:

```yaml
extends: ink
colors:
  background: "#fafaf9"
  strokeMain: "#1b365d"
  accentPrimary: "#b45309"
typography:
  sans: "Liberation Sans, Arial, sans-serif"
  mono: "Liberation Mono, monospace"
  scale: 1.2            # de 0.6 a 1.8: la letra crece y el diagrama se acomoda
  charWidthFactor: 0.62 # ancho medio de un carácter, en em (0.4 a 1.0)
```

| Color | Se usa para |
| :--- | :--- |
| `background` | El fondo del SVG. |
| `strokeMain` | Trazos principales: bordes de celdas, ejes, cables. |
| `strokeSub` | Trazos secundarios y flechas de detalle. |
| `strokeMuted` | Grillas, líneas punteadas y casilleros en blanco. |
| `fillHeader` | Encabezados de tablas. |
| `fillCell` | El relleno de celdas y cajas. |
| `fillPointerBox` | Cajas de punteros, nodos y bloques destacados. |
| `fillBadge` | Etiquetas pequeñas. |
| `accentPrimary` | El acento: curvas, flechas principales, grupos. |
| `textMain`, `textSub`, `textMuted` | Textos, de mayor a menor peso. |
| `fillHighlight` | Resaltados: celdas elegidas, fallos, áreas sombreadas. |

Se aceptan `#RGB`, `#RRGGBB`, `#RRGGBBAA`, `transparent` y `none`. Los colores y las familias
tipográficas se validan: un theme no puede inyectar atributos en el SVG.

Con el theme por defecto la salida es idéntica byte a byte a la de las versiones anteriores a los
themes, así que los SVG versionados con `sync-md` no cambian.
