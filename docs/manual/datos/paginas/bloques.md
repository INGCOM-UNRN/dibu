[Manual](README.md)

# Formato de los bloques

Un diagrama es un **bloque**: un objeto YAML o JSON con un campo `type` que elige el tipo de
diagrama y los demás campos, que dependen del tipo.

```yaml
type: stack
title: Pila después de tres apilar
values: [3, 7, 1]
```

Para probar un bloque sin instalar nada, el [editor web](https://ingcom-unrn.github.io/dibu-web/) es un playground con vista
previa en vivo: [este ejemplo, abierto ahí](https://ingcom-unrn.github.io/dibu-web/#t=dHlwZTogc3RhY2sKdGl0bGU6IFBpbGEgZGVzcHXDqXMgZGUgdHJlcyBhcGlsYXIKdmFsdWVzOiBbMywgNywgMV0K). Cada ejemplo del manual tiene un enlace
«Abrir en el editor» para ver cómo está construido y modificarlo.

## Un archivo, muchos diagramas

`dibu render` y `dibu batch` aceptan:

- un bloque solo (como el de arriba);
- una lista de bloques: cada elemento es un diagrama;
- un objeto con `blocks: [...]`, que es el formato de documento de statics.

```yaml
- id: pila
  type: stack
  values: [3, 7, 1]
- id: cola
  type: queue
  variant: circular
  values: [4, 9]
  capacity: 6
```

## En Markdown

En un apunte, cada bloque de código con la etiqueta `estructura` (o `statics`) es un diagrama. Si
tiene una lista de bloques, se dibujan uno debajo del otro en un solo SVG.

````markdown
La pila queda así:

```estructura
type: stack
values: [3, 7, 1]
```
````

`dibu sync-md <archivos o carpetas>` dibuja cada bloque en
`<carpeta>/diagramas/estructura-<hash>.svg`, donde el hash son los 12 primeros dígitos del SHA-1
del cuerpo del bloque, y borra los SVG que ningún bloque usa. Con `--verificar` no escribe: termina
con error si algo está desactualizado, para usarlo en CI o como hook de pre-commit.

## Cuidados con el YAML

- Un texto con `:`, `#`, corchetes o llaves va entre comillas: `"max = v[0]"`, `"R1 = 10k"`.
- Una dirección que empieza con `@` va entre comillas: `"@N1"`.
- En un banco GIFT (alucarD), el bloque va sin renglones en blanco y sin llaves: los objetos se
  escriben en forma de bloque, un campo por renglón.
- Los números con coma decimal se escriben con punto (`0.1`); dibu los muestra con coma.

## Errores

Un bloque inválido no se dibuja: dibu informa el tipo y el motivo (`cache: 'cacheSize' tiene que
ser potencia de 2 (es 100)`). En `dibu render` el resto de los bloques se dibuja igual. Los tamaños
tienen cotas (filas, nodos, muestras) para que un bloque con errores no produzca un SVG gigante.
