[Manual](README.md)

# Campos comunes

Todos los tipos aceptan estos campos, además de los propios.

| Campo | Tipo | Descripción |
| :--- | :--- | :--- |
| `type` | texto | El tipo de diagrama. Obligatorio. |
| `title` | texto | Título, arriba a la izquierda. En química acepta fórmulas (`H2SO4`, `Fe^3+`). |
| `subtitle` | texto | Subtítulo, debajo del título. |
| `id` | texto | Nombre del archivo al exportar: `dibu render` escribe `<id>.svg`. Sin `id`, `<archivo>_<n>.svg`. |
| `inkSaver` | sí/no | `true` usa el theme `ink` y `false` el `color`, sea cual sea el theme global. |
| `theme` | texto u objeto | Theme de este diagrama: un preset o `{extends, colors, typography}`. Manda sobre `inkSaver` y sobre el theme global. Ver [Themes](themes.md). |

El título y el subtítulo agrandan el ancho del diagrama si hace falta para que entren.
