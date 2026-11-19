# `instatypst-piece/v1` — el contrato (congelado)

**Estado: congelado el 2026-11-18.** No se añaden, quitan ni renombran campos, bloques, valores
permitidos ni códigos de error. Lo que necesite cambiar es otra versión (`instatypst-piece/v2`)
y se decide a propósito: `tests/congelado.rs` falla si la forma de v1 cambia por accidente.

Este documento describe lo que hay; si discrepa del código, manda el código y es un error de
este documento.

## Qué es y dónde vive

Un **documento de piezas** describe una o varias piezas (láminas) por lo que son, no por los
parámetros de un preset. El contrato es un modelo de datos; TOML, JSON y YAML son tres formas de
escribirlo (se elige por la extensión) y dan exactamente el mismo documento.

| Qué | Dónde |
| :--- | :--- |
| El modelo y su validación estructural | `rrss-cli-rs/src/model.rs` |
| El contenido escrito en Typst → bloques | `rrss-cli-rs/src/markup.rs` |
| Las firmas de los estilos (qué bloques acepta cada uno) | Instatypst: `carteles.firmas` → `schema/firmas.json` (copia en `rrss-cli-rs/src/firmas.json`) |
| La pieza → algo que compila | `rrss-cli-rs/src/pieza.rs` |
| La forma congelada | `rrss-cli-rs/tests/fixtures/contrato/congelado/` |

## El documento

```toml
schema = "instatypst-piece/v1"     # opcional; si está, debe ser exactamente este

[render]    # cómo se produce, no qué
ppi   = 72                          # 72–1200
image = "png"                       # png | webp   (hoy manda el flag --imagen)
fonts = ["Fonts"]                   # carpetas de fuentes

[brand]     # la marca de todas las piezas (ver «Marca»)
[palettes.NOMBRE]   # paletas con nombre del documento (ver «Paleta»)
[defaults]  # una pieza sin nombre ni contenido propio: lo que heredan todas
[[post]]    # las piezas, en orden; el orden es la serie
```

**Herencia.** Cada campo de una pieza, si falta, sale de `[defaults]`; `options` se mezcla clave
a clave. La marca de una pieza manda sobre la de `[defaults]`, y esta sobre la de `[brand]`.

## La pieza

| Campo | Qué es |
| :--- | :--- |
| `name` | obligatorio; no vacío, sin `..`, ni `/` inicial, ni bytes nulos; único |
| `style` | `"suizo"` (un estilo del catálogo) o `{ file = "x.typ", fn = "f" }` (un estilo propio) |
| `variant` | la variante del estilo (cada estilo declara si la tiene) |
| `format` | `square`, `portrait`, `story`, `landscape`, `a4`, `a5`, o `{ width, height }` (cada lado 100–4320) |
| `palette` | ver «Paleta» |
| `background` | `{ type = "solid" \| "grid" \| "rules" \| "paper" \| "contours" }` o `{ type = "image", src = "…" }` |
| `typography` | `{ heading, body }`; cada una, `"Familia"` o `{ family, weight }` |
| `brand` | `false` (sin marca en esta pieza) o una tabla (ver «Marca») |
| `options` | parámetros propios del estilo, con su nombre nativo |
| `content` | obligatorio: texto en Typst, o una lista de bloques |

`options` usa, para valores con unidad o palabra clave, las tablas de siempre: `{ kw = "center" }`,
`{ len = "96pt" }`, `{ color = "#691322" }`. `formato` y `tema` los fija el contrato: no van en `options`.

### Paleta

`palette` es un nombre (`"ocean"`, o una de `[palettes.*]`) o una tabla con **una sola fuente**
(`base`, `from_image` o `file`) y/o roles explícitos que la pisan. Sin fuente, necesita al menos
`background`. Los colores son `#RRGGBB`. Roles: `background`, `text`, `muted`, `accent`, `line`,
`good`, `bad`. Los que falten se derivan al resolver.

### Marca

```toml
[brand]
logo = { text  = "@iglesiaraah" }              # un handle
logo = { image = "assets/Logo.svg" }           # svg (con currentColor: se recolorea), png, jpg, jpeg o webp
logo = { typst = "logo.typ", fn = "logo" }     # tu función (color, lienzo) → contenido
size = 0.04                                    # 0,01–0,25 de √(ancho × alto); por omisión 1/30; no aplica a `typst`
```

Exactamente un `logo`: `text`, `image` o `typst` (este siempre con `fn`). El preset dibuja la
marca con su propia tinta. Una firma de texto escrita en la pieza (`signature`) manda sobre ella.

## Los bloques de contenido

Una lista en el orden escrito. Cada bloque es una tabla con `type` y sus campos; los textos
admiten Typst en línea (`_énfasis_`, `*negrita*`, `\ ` salto de línea) y un `\n` del texto es un
salto de línea.

| `type` | Campos | Notas |
| :--- | :--- | :--- |
| `title` | `text` | único |
| `subtitle` | `text` | único |
| `label` | `text` | rótulo corto: marca un quiebre, el inicio de un arco interno o enfatiza una idea. **Puede repetirse** |
| `body` | `text` | |
| `quote` | `text` | |
| `epigraph` | `text`, `source?` | `source` es de quién es el epígrafe, no la atribución de la pieza |
| `verse` | `lines` (lista), `role?` | no se alinea con el resto del estilo |
| `conclusion` | `text` | único |
| `attribution` | `author?`, `role?`, `work?`, `date?` | al menos uno de `author`, `work`, `date` |
| `list` | `order?` (`none` \| `numbered` \| `chronological`), `items` | `items` no vacío |
| `contrast` | `left`, `right`: `{ label?, title?, items }` | cada lado con ítems |
| `signature` | `text` | único |
| `footer` | `text` | único; línea de cierre bajo el contenido |
| `number` | `value`, `total?` | único; `total` ≥ `value` |
| `date` | `day` (1–31), `month` | único |

Un **ítem** de `list` o de un lado de `contrast` es un texto, o `{ text, label?, role? }` con
`role` = `neutral` \| `affirm` \| `deny`.

**Parentesco.** `verse → quote → body` y `epigraph → quote → body`. Un estilo que acepta `body`
acepta también `quote`, `verse` y `epigraph` (los dibuja como cuerpo). Un bloque cuenta como **el
más específico de su cadena que el estilo acepta**, y solo como ese: en `liso`, que tiene el
epígrafe aparte, un epígrafe no cubre el `body` que exige.

### El contenido en Typst

`content` puede ser un texto en Typst, con una **lista cerrada** de lo que se admite (todo lo
demás es un error con su línea y columna; nunca se ejecuta código):

| Escribes | Resulta |
| :--- | :--- |
| `= x` · `== x` | `title` · `subtitle` (otros niveles: error) |
| un párrafo | `body` |
| `- a` · `+ a` (seguidos) | `list` sin orden · numerada |
| `#label[…]` `#conclusion[…]` `#footer[…]` `#signature[…]` | el del mismo nombre |
| `#epigraph[…]` · `#epigraph(source: "x")[…]` | `epigraph` |
| `#quote[…]` · `#quote(attribution: "x")[…]` | `quote` (+ `attribution`) |
| `#verse[a \ b]` | `verse` |
| `#number(3)` · `#number(3, total: 8)` | `number` |
| `#date(day: 12, month: "NOV")` | `date` |
| `#attribution(author:, role:, work:, date:)` | `attribution` |

Un `@usuario` se escribe `\@usuario`. Las listas con etiqueta por ítem solo se escriben en la
forma de bloques. Las dos formas son equivalentes.

## Validación

Dos capas; cada incidencia es `{ path, code, message }` (la ruta señala el campo: `post[2].content[1]`).

1. **Estructural** (`model::validate`): la forma, los tipos, los valores permitidos; un campo
   desconocido es un error (un typo no se ignora).
2. **Semántica** (`model::validate_semantic`): la pieza frente a la firma de su estilo. Un
   estilo propio no tiene firma: recibe todo.

Cada estilo del catálogo declara en su firma: `bloques` (`requeridos`, con «uno de» entre
paréntesis, y `opcionales`), `entradas` (a qué parámetro del preset va cada bloque), `options`
(sus parámetros, derivados de su código) y `marca-funcion`. Un bloque que el estilo no acepta,
o que falta, es un error; no se descarta en silencio.

**Códigos de error** (40, congelados):

| Grupo | Códigos |
| :--- | :--- |
| lectura | `parse`, `schema_version` |
| documento | `ppi_range`, `name_in_defaults` |
| pieza | `name_missing`, `name_invalid`, `name_duplicate`, `style_missing`, `style_empty`, `style_file`, `style_fn`, `content_missing`, `content_empty`, `format_size` |
| paleta | `palette_empty`, `palette_incomplete`, `palette_source_conflict`, `color_invalid` |
| marca | `brand_logo`, `brand_path`, `brand_image`, `brand_size` |
| bloques | `block_duplicate`, `text_empty`, `verse_empty`, `attribution_empty`, `list_empty`, `number_total`, `date_invalid` |
| catálogo | `style_unknown`, `block_missing`, `block_unsupported`, `block_repeated`, `option_unknown`, `option_reserved` |
| Typst | `markup_syntax`, `markup_forbidden`, `markup_function`, `markup_args`, `heading_level` |

## Estilo propio

`style = { file, fn }` importa tu función y le pasa **una sola cosa**: la pieza ya resuelta, como
un diccionario de Typst (con las ayudas `estilo.*` de Instatypst). Es código tuyo, de confianza.

| Clave | Valor |
| :--- | :--- |
| `name`, `index`, `total` | el nombre, y la posición de la pieza en el documento (1…) y cuántas son |
| `format` | `(ancho, alto)` en px, o `none` |
| `palette` | `(background, text, muted, accent, line)` como colores, o `none` |
| `blocks` | los bloques, con su tipo real (sin parentesco), textos como contenido |
| `options` | tal cual |
| `variant`, `typography`, `background`, `brand` | lo declarado, o `none`; `brand` es una función `(color, lienzo) → contenido` |

Las claves opcionales siempre existen (valen `none`). La paleta debe ser una que la CLI resuelva
(del documento, de la CLI o `from_image`); el nombre de un tema de Instatypst solo lo sabe leer un preset.

## La frontera de v1

**Está en el contrato, pero el puente aún no lo traduce a un preset** (con un estilo propio sí
llega todo; con un preset falla diciéndolo):

- el bloque `list` y los fondos `grid`, `rules`, `paper` y `contours`;
- `typography.heading`, y los roles `line`, `good` y `bad` de la paleta;
- `palette.file`, y combinar `palette.from_image` con roles explícitos;
- los estilos del catálogo sin `entradas` (14 de 23), y la marca en `epigrafe`.

**No existe en v1** (candidatos a v2, según lo pidan piezas reales): el rol de una página
(`cover`, `list`, `epigraph`…), el marco y el rótulo de serie (`[series]`), una regla para
invertir fondo y acento, y recolorear una imagen de fondo con la paleta de la pieza (un
`background.recolor`; la crate [`lutgen`](https://github.com/ozwaldorf/lutgen-rs) hace justo
eso con una tabla de color en Oklab, y ya hay fondos recoloreados a mano en `assets/`).

## Cómo evoluciona

- Añadir un estilo al catálogo, o `entradas` a uno existente, **no** cambia el contrato: viven en
  las firmas. Las firmas sí se versionan con Instatypst (`scripts/exportar-firmas.py`).
- Añadir o cambiar un campo, un bloque, un valor permitido o un código de error **sí** lo
  cambia: es `v2`. Lo que v1 escribió tiene que seguir leyéndose (lo comprueba el golden).
