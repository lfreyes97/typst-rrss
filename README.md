# typst-rrss

CLI en Rust para producir imágenes de redes sociales con [**instatypst**](https://github.com/lfreyes97/Instatypst), el marco de diseño en Typst (presets, paletas, tipografía). `instatypst` pone el diseño; esta herramienta lo produce: pasa datos a piezas, en lote y de forma repetible.

Lo que aporta encima de `instatypst`:

- **`posts.toml`**: describe muchas piezas y las compila de una vez.
- **Temas desde una imagen**: extrae la paleta de una foto (clustering o Material You) y avisa si el contraste texto/fondo es bajo.
- **Compilación por lote** a PNG o WebP, con errores y desbordes de página claros.
- **Escapado seguro**: nada de lo que escribes en el TOML o en los flags se interpola a mano en el Typst generado.

---

## Requisitos

- [`typst`](https://typst.app) en el `PATH`.
- `instatypst` instalado como paquete local (`@local/instatypst:0.1.0`). Mira su `README`.
- Las fuentes de `instatypst`: instálalas en el sistema (sección «Instalar las fuentes en el sistema» de su `README`) o pásalas con `fonts` en `[defaults]` / `--font-path` (una carpeta que no existe es un error). `compile` no tiene `--font-path`: usa las del sistema o `TYPST_FONT_PATHS`.
- Rust estable, para compilar la CLI.

```bash
cargo install --path rrss-cli-rs
```

---

## Comandos

| Comando | Qué hace |
| :--- | :--- |
| **`build`** | Genera y compila los posts de un `posts.toml`. |
| **`init`** | Escribe un punto de partida editable: un `posts.toml` o un `.typ`. |
| **`full`** | Una pieza desde flags: genera el `.typ` y lo compila. |
| **`generate`** | Una pieza desde flags: escribe el `.typ`, sin compilarlo. |
| **`compile`** | Compila `.typ` que ya existen (hechos a mano o con `instatypst`). |
| **`extract`** | Extrae una paleta de colores de una imagen. |
| **`colors`** | Genera una paleta armónica desde un color base. |

`rrss-cli-rs <comando> --help` describe cada flag. Los que se repiten significan lo mismo en todos los comandos:

| Flag | Significado |
| :--- | :--- |
| `-o`, `--output` | Dónde va la salida: archivo (`init`, `generate`), nombre de la pieza (`full`) o carpeta (`compile`). |
| `--formato` | Lienzo: `feed-4x5`, `feed-1x1`, `story`, `twitter`, `linkedin` o `ANCHOxALTO`. |
| `--imagen` | Formato de la imagen: `png` (por defecto) o `webp`. |
| `--solo` | `build`: compila solo el post con ese `name` (los demás se validan igual). |

```bash
# Un posts.toml de arranque (uno o varios presets); con -o *.typ, el .typ de una pieza
rrss-cli-rs init suizo liso
rrss-cli-rs init suizo -o pieza.typ

# Todo un posts.toml, o solo uno
rrss-cli-rs build
rrss-cli-rs build examples/presets.toml --solo presets/suizo
rrss-cli-rs build --dry-run --imagen webp

# Una pieza desde flags: PRESET [POS...] y --arg clave=valor
rrss-cli-rs full suizo "Dios no necesita que lo defendamos." "Él es la precondición." \
    --tema ocean --formato feed-1x1 -o creacion

# Solo el .typ, para revisarlo o editarlo; luego, compilarlo
rrss-cli-rs generate liso "Todo es gracia." --tema auto --image assets/bg-ciencia.jpg -o pieza.typ
rrss-cli-rs compile pieza.typ

rrss-cli-rs extract assets/bg-ciencia.jpg --suggest-accent
rrss-cli-rs colors "#691322" --format typst > tema.typ
```

`init` escribe solo plantillas que compilan tal cual: conoce los argumentos posicionales de `liso`, `suizo`, `bauhaus`, `epigrafe`, `esquela` y `franjas`. Para otro preset, parte de uno de estos y ajusta `preset`, `pos` y `args` (ejemplos en [`examples/presets.toml`](examples/presets.toml)). Los textos entre `‹…›` son marcadores para reemplazar.

Las imágenes salen en `output/` y los `.typ` generados en `output/typ/`. El `ppi` por defecto es **72**: un lienzo de 1080 pt sale de 1080 px.

---

## `posts.toml`

Cada `[[post]]` es una llamada a un preset de `instatypst` (`carteles.<preset>`). Los argumentos posicionales van en `pos` y los que llevan nombre, en `[post.args]`.

```toml
[defaults]
preset = "liso"
fonts = ["/ruta/a/Instatypst/Fonts"]        # opcional si las fuentes están en el sistema

[defaults.por_preset.liso]                  # args por defecto de un solo preset
fuente = "EB Garamond 12"
peso = 500

[[post]]
name = "ciencia"
tema = "auto"                               # paleta extraída de `image`
image = "assets/bg-ciencia.jpg"
pos = ["Cuando David vio las estrellas y el sol, alabó a Dios."]
[post.args]
autor = "Ciencia: una celebración y un lamento"
fondo = { image = "assets/bg-ciencia.jpg" }
```

| Campo | Significado |
| :--- | :--- |
| `preset` | Nombre del preset (`suizo`) o ruta completa (`fleurones.orla`). Si no existe, falla Typst con un error claro. |
| `formato` | `feed-4x5`, `feed-1x1`, `story`, `twitter`, `linkedin` o `[ancho, alto]`. Sin él, manda el del preset. |
| `ppi` | 72–1200. Por defecto, 72. |
| `tema` | Paleta de `[themes.*]`, `auto` / `auto-matugen` (desde `image`), o un tema de `instatypst`. |
| `image` | Imagen de la que se extrae el tema. El fondo va aparte, en `args`. |
| `pos` / `args` | Argumentos del preset; `args` se suma a `[defaults.args]` y a `por_preset`. |
| `defs` | Variables de Typst compartidas (`#let nombre = valor`). |

Un campo desconocido es un error: un typo no se ignora en silencio.

### Cómo se escriben los valores

| TOML | Typst |
| :--- | :--- |
| `"texto"` | `"texto"` (un `str`) |
| `{ txt = "texto" }` o `etiqueta_txt = "…"` | contenido de texto plano, sin interpretar nada |
| `{ md = "*texto*" }` o `etiqueta_md = "…"` | contenido con formato; ve `carteles.*` y los `defs` |
| `{ color = "#691322" }` | `rgb("#691322")` |
| `{ len = "96pt" }` | `96pt` |
| `{ kw = "center" }` | `center`, `auto`, `none`… |
| `{ image = "assets/x.jpg" }` | imagen a sangre, desde la raíz del proyecto |
| `{ typst = "3pt + red" }` | la expresión tal cual (sin escapar) |

Un `str` y un contenido **no son lo mismo** para un preset: si una pieza no sale como en los ejemplos de `instatypst`, prueba con `_txt`.

Hay un ejemplo con 8 presets en [`examples/presets.toml`](examples/presets.toml).

---

## Piezas (`instatypst-piece/v1`)

> El contrato está **congelado** (v1): ver [`docs/contrato-v1.md`](docs/contrato-v1.md), que lo
> describe entero, con su frontera (qué hay, qué el puente aún no traduce y qué no existe).

Además del `posts.toml` de siempre, `build` lee **piezas**: en vez de los
parámetros de cada preset, describes *qué* quieres (formato, paleta, fondo,
contenido) y el estilo decide cómo dibujarlo. Cambiar de preset es cambiar
una línea; el contenido se reutiliza. Sirve el mismo documento en TOML, JSON o
YAML (por la extensión). `build` distingue uno de otro por `schema` o por `style`.

```toml
schema = "instatypst-piece/v1"

[[post]]
name    = "ciencia"
style   = "suizo"              # el preset del catálogo, con su nombre de siempre
format  = "square"             # square · portrait · story · landscape · a4 · a5
palette = "ocean"              # nombre | { background, text, muted, accent } | { base = "ocean", accent = "#…" }
content = '''
#label[DEL ENSAYO]
= El calvinismo es simplemente la religión en su
#conclusion[PUREZA.]
#attribution(author: "B. B. WARFIELD", date: "1851 – 1921")
'''
```

El contenido se escribe en Typst (solo se admite una lista cerrada: títulos
`=` / `==`, párrafos, `_énfasis_`, y las funciones `#label`, `#conclusion`,
`#footer`, `#signature`, `#epigraph`, `#quote`, `#verse`, `#number`, `#date`,
`#attribution`; nada de código) o como lista de bloques `[[post.content]]`: es
equivalente. Un `@usuario` se escribe `\@usuario`.
El **rótulo** (`#label[…]`) es el texto corto que marca un quiebre, el inicio de un arco
interno o una idea enfatizada («EL ORDEN», «I · ¿Qué es una nación?»); puede aparecer varias
veces en una pieza y en cualquier punto del flujo. Un estilo de catálogo que tiene un solo
lugar para él (como `suizo`) rechaza el segundo, diciéndolo; un estilo propio recibe todos. Cada estilo declara qué
bloques acepta; lo que no entiende es un error, con la ruta exacta
(`post[2].content[1]`) y el motivo.

`options` lleva los parámetros propios del preset con su nombre nativo, y las
tablas de siempre para valores con unidad (`{ kw = "center" }`, `{ len = "96pt" }`,
`{ color = "#691322" }`).

**Epígrafe.** Un epígrafe es un elemento que cualquier cartel puede llevar, no un cartel
aparte; `liso` es el natural. Se escribe `#epigraph(source: "Efesios 2:8")[«…»]` (o el
bloque `epigraph` con `source`): sale arriba, más chico y en cursiva, y el texto
principal se centra en lo que queda. Un estilo que no lo tiene aparte lo dibuja como su
cuerpo (y rechaza la `source`, que no tendría dónde ir). Un epígrafe solo, sin texto
principal, no es una pieza válida de `liso`.

**Marca.** La marca de la serie (`[brand]`) puede ser un texto, una imagen o una
función tuya de Typst, y el preset la dibuja con **su propia tinta**, así que el mismo
logo sale blanco sobre fondo oscuro y negro sobre claro. Pesa lo mismo, respecto del
lienzo, en cualquier formato (por omisión, el alto es 1/30 de √(ancho × alto)).

```toml
[brand]
logo = { text  = "@iglesiaraah" }                 # un handle
logo = { image = "assets/Logo.svg" }              # SVG con currentColor (se recolorea) o PNG/JPG
logo = { typst = "logo.typ", fn = "logo" }        # tu función (color, lienzo) → contenido
size = 0.04                                       # opcional; no aplica a una función
```

Una pieza puede cambiarla (`brand = { logo = … }`) o quitarla (`brand = false`), y una
firma de texto escrita en la pieza (`#signature[…]`) manda sobre la marca de la serie.
`brand.typst` y `style.file` son código tuyo, de confianza: el contenido sigue siendo la
lista cerrada. Un estilo que aún no acepta una marca con color (hoy, `epigrafe`) falla
diciéndolo; se resuelve con `brand = false` en esa pieza.

**Estilo propio.** Si el diseño es tuyo (andamio, no un preset), `style = { file, fn }`
importa tu función y le pasa **la pieza ya resuelta** como un diccionario: formato en px,
paleta en sus roles (`background`, `text`, `muted`, `accent`, `line`), los bloques con su tipo
real, `options`, `brand` (una función) y la posición en la serie (`index`, `total`, que cuenta
el orden de las piezas del documento). Tu función dibuja todo lo demás.

```toml
[[post]]
name    = "derecho-01"
style   = { file = "layouts/derecho.typ", fn = "desarrollo" }
palette = { background = "#691322", text = "#f3ece2", accent = "#e3b98a" }
content = '''
= ¿Tenemos derecho a ser cristianos?
'''
```

```typst
#import "@local/instatypst:0.1.0": *          // trae `estilo`, con ayudas
#let desarrollo(pieza) = {
  let (w, h) = estilo.formato(pieza)
  let c = estilo.paleta(pieza)
  page(width: w, height: h, fill: c.background, text(fill: c.text, estilo.texto(pieza, "title")))
}
```

La paleta debe ser una que la CLI resuelva (del documento, de la CLI o `from_image`): un
nombre de tema de Instatypst solo lo sabe leer un preset. Es código tuyo, de confianza.

**Estilos que ya se construyen desde una pieza:** `liso`, `suizo`, `bauhaus`,
`epigrafe`, `esquela`, `contraste`, `evento`, `consejo` y `lamina`. Los demás
(del catálogo de 23) fallan con un mensaje claro hasta que su firma declare las
`entradas`. Tampoco se traducen aún: los bloques `list`, los fondos que no son
sólidos o imagen, `typography.heading`, los roles `line`/`good`/`bad` de la paleta,
`palette.file`.

Se comprobó que una pieza da, píxel a píxel, la misma imagen que el post
equivalente del formato de siempre (`tests/pieza.rs`: `liso`, `suizo`, `epigrafe`
y `esquela`), y la lámina 01 de la campaña, con su logo real, sale idéntica a la hecha a mano.

Las firmas de los estilos vienen de Instatypst (`schema/firmas.json`) y se
guardan aquí en `rrss-cli-rs/src/firmas.json`. Si cambian allí:
`scripts/exportar-firmas.py` y copiar el resultado.

---

## Robustez

- **Escapado**: cualquier texto pasa por un único módulo (`typst.rs`) que lo escapa para Typst; verificado con casos hostiles contra el propio `typst`.
- **`typst` se invoca con argumentos separados**, nunca interpolando una línea de shell.
- **Rutas**: se rechazan `..` y bytes nulos en nombres de post y recursos.
- **Límites**: `ppi` entre 72 y 1200; lados del lienzo entre 100 y 4320.
- **Errores**: `build` y `compile` procesan todo, muestran qué falló y terminan con código distinto de 0. Un texto que no cabe se informa como desborde de página.
- **Avisos de Typst** (una fuente que no existe, por ejemplo) salen como `⚠` bajo la pieza aunque compile.
- **Salida limpia**: stdout lleva solo el resultado; las trazas van a stderr y solo los avisos, salvo que `RUST_LOG` pida más (`RUST_LOG=info`).

Se comprobó que `build` reproduce, píxel a píxel, 88 láminas de los ejemplos de presets de `instatypst`.

---

## Estructura

- **`rrss-cli-rs/`**: la CLI.
  - `config.rs`: parser de `posts.toml` (esquema, herencia, validación).
  - `render.rs`: un post se convierte en el `.typ`.
  - `typst.rs`: valores a literales de Typst, con escapado.
  - `colors.rs`, `themes.rs`: extracción de paletas y contraste.
  - `build.rs`, `process.rs`: compilación.
- **`examples/`**: `posts.toml` de muestra.
- **`assets/`**: imágenes de ejemplo.

---

## Licencia

MIT. Consulta `LICENSE`.
