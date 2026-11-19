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
