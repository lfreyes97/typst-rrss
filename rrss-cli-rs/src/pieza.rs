//! De una pieza (`model`, ya validada) al `Config` que `build` sabe compilar.
//!
//! La pieza dice *qué* se quiere (ejes y bloques); el preset de Instatypst
//! espera sus propios parámetros. Este módulo hace de puente: reparte los
//! bloques en los parámetros del preset según las `entradas` de su firma
//! (`schema/firmas.json`) y entrega un `Config` de siempre, de modo que el
//! escapado, los temas, los avisos de contraste y la compilación son los que
//! ya existen. Nada de lo que escribe el autor se interpola a mano.
//!
//! Lo que el puente aún no sabe traducir falla con un error que lo dice, en
//! vez de descartarse en silencio.

use anyhow::{anyhow, bail, Context, Result};
use std::collections::{BTreeMap, HashMap};
use std::fs;
use std::path::Path;

use crate::config::{Config, Defaults, FormatoSpec, RawPost};
use crate::model::{
    self, Background, Block, Brand, BrandSpec, Catalog, Content, Document, Font, Format, FormatName, Item, PaletteDef,
    PaletteSpec, PieceSpec, Role, Side, StyleSig, Style, Syntax,
};
use crate::typst::{asset_path, str_lit};
use crate::{colors, themes};

type Paleta = HashMap<String, String>;

// ── Detección y carga ───────────────────────────────────────

/// ¿Es un documento de piezas (`instatypst-piece/v1`) y no un `posts.toml` de siempre?
/// Se mira `schema` o un `style` en alguna pieza: nada de intentar leerlo de las dos formas.
pub fn es_de_piezas(texto: &str, sintaxis: Syntax) -> bool {
    let v: Option<serde_json::Value> = match sintaxis {
        Syntax::Toml => toml::from_str(texto).ok(),
        Syntax::Json => serde_json::from_str(texto).ok(),
        Syntax::Yaml => serde_yaml::from_str(texto).ok(),
    };
    let Some(v) = v else { return false };
    let por_schema = v
        .get("schema")
        .and_then(|s| s.as_str())
        .is_some_and(|s| s.starts_with("instatypst-piece"));
    let por_pieza = v
        .get("post")
        .and_then(|p| p.as_array())
        .is_some_and(|a| a.iter().any(|x| x.get("style").is_some()));
    let por_defaults = v.get("defaults").is_some_and(|d| d.get("style").is_some());
    por_schema || por_pieza || por_defaults
}

/// Las incidencias, una por línea y con su ruta.
pub fn formatear(issues: &[model::Issue]) -> String {
    let mut s = String::new();
    for i in issues {
        let ruta = if i.path.is_empty() { "(documento)" } else { &i.path };
        s.push_str(&format!("  ✗ {ruta} [{}]: {}\n", i.code, i.message));
    }
    s
}

/// Carga un archivo de posts: de piezas (TOML, JSON o YAML) o el `posts.toml` de siempre.
pub fn cargar(path: &Path) -> Result<Config> {
    let texto = fs::read_to_string(path)
        .with_context(|| format!("No se pudo leer {}", path.display()))?;
    let sintaxis = path
        .to_str()
        .and_then(Syntax::from_extension)
        .unwrap_or(Syntax::Toml);
    if !es_de_piezas(&texto, sintaxis) {
        return Config::load(path);
    }
    let cat = Catalog::embedded();
    let doc = model::load_checked(&texto, sintaxis, &cat)
        .map_err(|issues| anyhow!("{} no es válido:\n{}", path.display(), formatear(&issues)))?;
    a_config(&doc, &cat).with_context(|| format!("{} no se pudo construir", path.display()))
}

// ── Documento → Config ──────────────────────────────────────

pub fn a_config(doc: &Document, cat: &Catalog) -> Result<Config> {
    let mut defaults = Defaults::default();
    if let Some(r) = &doc.render {
        defaults.ppi = r.ppi;
        defaults.fonts = r.fonts.clone();
    }
    // Las de fábrica de la CLI más las del documento: definir una no borra las demás.
    let mut paletas = themes::predefined::get_theme_palettes(None);

    let mut posts = Vec::new();
    for i in 0..doc.posts.len() {
        let p = doc.merged(i);
        let nombre = p.name.clone().unwrap_or_default();
        posts.push(
            a_post(&p, i, doc, cat, &mut paletas).with_context(|| format!("pieza {nombre:?}"))?,
        );
    }
    Ok(Config {
        defaults,
        themes: Some(paletas),
        posts,
    })
}

fn a_post(
    p: &PieceSpec,
    i: usize,
    doc: &Document,
    cat: &Catalog,
    paletas: &mut HashMap<String, Paleta>,
) -> Result<RawPost> {
    let nombre = p.name.clone().ok_or_else(|| anyhow!("falta `name`"))?;
    let estilo = match &p.style {
        Some(Style::Catalog(s)) => s.clone(),
        Some(Style::Custom(c)) => return a_post_propio(p, i, nombre, c, doc, paletas),
        None => bail!("falta `style`"),
    };
    let sig = cat
        .styles
        .get(&estilo)
        .ok_or_else(|| anyhow!("el estilo {estilo:?} no está en el catálogo"))?;

    let bloques = bloques_de(p)?;

    let mut valores = repartir(&bloques, &estilo, sig, cat)?;

    // Marca: la de la pieza, la de los defaults o la del documento. Una firma de texto
    // (`signature`) escrita en la pieza manda sobre ella.
    let brand = match &p.brand {
        Some(BrandSpec::Off) => None,
        Some(BrandSpec::On(b)) => Some(b),
        None => doc.brand.as_ref(),
    };
    let param_marca = sig.entradas.get("signature").and_then(|x| x.as_str());
    if let (Some(b), Some(param)) = (brand, param_marca) {
        if !valores.contains_key(param) {
            if !sig.marca_funcion {
                bail!(
                    "`{estilo}` aún no acepta una marca con color (una función de Typst): \
                     ponle una firma de texto (`signature`) o quita la marca con `brand = false`"
                );
            }
            valores.insert(param.into(), marca(b)?);
        }
    }

    // Variante del estilo
    if let Some(v) = &p.variant {
        let param = sig
            .entradas
            .get("variante")
            .and_then(|x| x.as_str())
            .ok_or_else(|| anyhow!("`{estilo}` no tiene variantes (`variant`)"))?;
        valores.insert(param.into(), toml::Value::String(v.clone()));
    }

    // Tipografía → parámetros de fuente del preset
    if let Some(t) = &p.typography {
        tipografia(t, &estilo, sig, &mut valores)?;
    }

    // Fondo
    match &p.background {
        None | Some(Background::Solid) => {}
        Some(Background::Image { src }) => {
            if !sig.options.iter().any(|o| o == "fondo") {
                bail!("`{estilo}` no admite una imagen de fondo (no tiene el parámetro `fondo`)");
            }
            let mut t = toml::Table::new();
            t.insert("image".into(), toml::Value::String(src.clone()));
            valores.insert("fondo".into(), toml::Value::Table(t));
        }
        Some(otro) => bail!("el fondo {otro:?} aún no se traduce a un preset"),
    }

    // Opciones propias del estilo: pisan lo anterior. Los valores con unidad o
    // palabra clave usan las tablas de siempre: `{ kw = "center" }`, `{ len = "96pt" }`,
    // `{ color = "#691322" }`.
    for (k, v) in &p.options {
        valores.insert(k.clone(), json_a_toml(v).with_context(|| format!("options.{k}"))?);
    }

    // Posicionales (en el orden del preset) y el resto con nombre
    let mut pos = Vec::new();
    for param in &sig.pos {
        pos.push(
            valores
                .remove(param)
                .ok_or_else(|| anyhow!("a `{estilo}` le falta el argumento `{param}`"))?,
        );
    }
    let mut args = toml::Table::new();
    for (k, v) in valores {
        args.insert(k, v);
    }

    let (tema, image) = paleta(p, i, doc, paletas, &estilo)?;

    Ok(RawPost {
        name: nombre,
        preset: Some(estilo),
        formato: p.format.as_ref().map(formato),
        ppi: None,
        tema,
        image,
        pos,
        args,
        defs: toml::Table::new(),
        estilo_propio: None,
    })
}

fn bloques_de(p: &PieceSpec) -> Result<Vec<Block>> {
    match &p.content {
        Some(Content::Blocks(b)) => Ok(b.clone()),
        Some(Content::Markup(m)) => {
            let (b, problemas) = crate::markup::parse(m);
            if !problemas.is_empty() {
                bail!("contenido Typst no válido:\n{}", formatear(&problemas));
            }
            Ok(b)
        }
        None => bail!("falta `content`"),
    }
}

// ── Estilo propio ───────────────────────────────────────────

/// Una pieza para un estilo propio: no hay firma que consultar, así que la función del
/// autor recibe todo ya resuelto —bloques con su tipo real, sin parentesco— como un
/// diccionario con los nombres del contrato (`blocks`, `options`, `brand`, `index`…).
/// El formato y la paleta los resuelve `render`, igual que para un preset.
fn a_post_propio(
    p: &PieceSpec,
    i: usize,
    nombre: String,
    c: &model::CustomStyle,
    doc: &Document,
    paletas: &mut HashMap<String, Paleta>,
) -> Result<RawPost> {
    use toml::Value;
    let bloques = bloques_de(p)?;
    let mut pieza = toml::Table::new();
    pieza.insert("name".into(), Value::String(nombre.clone()));
    // el documento es la serie: la posición de la pieza y cuántas son
    pieza.insert("index".into(), Value::Integer(i as i64 + 1));
    pieza.insert("total".into(), Value::Integer(doc.posts.len() as i64));
    pieza.insert(
        "blocks".into(),
        Value::Array(bloques.iter().map(|b| Value::Table(bloque_propio(b))).collect()),
    );
    if let Some(v) = &p.variant {
        pieza.insert("variant".into(), Value::String(v.clone()));
    }
    let mut opciones = toml::Table::new();
    for (k, v) in &p.options {
        opciones.insert(k.clone(), json_a_toml(v).with_context(|| format!("options.{k}"))?);
    }
    pieza.insert("options".into(), Value::Table(opciones));

    if let Some(t) = &p.typography {
        let fuente = |f: &Font| match f {
            Font::Family(s) => Value::String(s.clone()),
            Font::Spec(x) => {
                let mut t = toml::Table::new();
                t.insert("family".into(), Value::String(x.family.clone()));
                if let Some(w) = x.weight {
                    t.insert("weight".into(), Value::Integer(i64::from(w)));
                }
                Value::Table(t)
            }
        };
        let mut tt = toml::Table::new();
        if let Some(f) = &t.heading {
            tt.insert("heading".into(), fuente(f));
        }
        if let Some(f) = &t.body {
            tt.insert("body".into(), fuente(f));
        }
        pieza.insert("typography".into(), Value::Table(tt));
    }

    if let Some(bg) = &p.background {
        let mut t = toml::Table::new();
        match bg {
            Background::Solid => t.insert("type".into(), Value::String("solid".into())),
            Background::Grid => t.insert("type".into(), Value::String("grid".into())),
            Background::Rules => t.insert("type".into(), Value::String("rules".into())),
            Background::Paper => t.insert("type".into(), Value::String("paper".into())),
            Background::Contours => t.insert("type".into(), Value::String("contours".into())),
            Background::Image { src } => {
                t.insert("type".into(), Value::String("image".into()));
                // la imagen ya hecha, a sangre, como en `{ image = "…" }`
                let mut im = toml::Table::new();
                im.insert("image".into(), Value::String(src.clone()));
                t.insert("content".into(), Value::Table(im))
            }
        };
        pieza.insert("background".into(), Value::Table(t));
    }

    // La marca, como función (color, lienzo) → contenido; la función del autor la llama.
    let brand = match &p.brand {
        Some(BrandSpec::Off) => None,
        Some(BrandSpec::On(b)) => Some(b),
        None => doc.brand.as_ref(),
    };
    if let Some(b) = brand {
        pieza.insert("brand".into(), marca(b)?);
    }

    // Las claves opcionales siempre existen: `none` si la pieza no las declara, para que
    // `pieza.brand` no rompa al leerla (`format` y `palette` los resuelve `render`).
    for k in ["variant", "typography", "background", "brand"] {
        pieza.entry(k.to_string()).or_insert_with(ninguno);
    }

    let (tema, image) = paleta(p, i, doc, paletas, &c.function)?;
    Ok(RawPost {
        name: nombre,
        preset: None,
        formato: p.format.as_ref().map(formato),
        ppi: None,
        tema,
        image,
        pos: Vec::new(),
        args: toml::Table::new(),
        defs: toml::Table::new(),
        estilo_propio: Some(crate::config::EstiloPropio {
            archivo: c.file.clone(),
            funcion: c.function.clone(),
            pieza,
        }),
    })
}

/// El valor `none` de Typst (TOML no tiene null): `{ kw = "none" }`.
pub fn ninguno() -> toml::Value {
    let mut t = toml::Table::new();
    t.insert("kw".into(), toml::Value::String("none".into()));
    toml::Value::Table(t)
}

fn poner(t: &mut toml::Table, k: &str, v: &str) {
    t.insert(k.into(), md(v));
}

/// Un bloque como diccionario de Typst: `type` y sus campos, con los textos como contenido.
fn bloque_propio(b: &Block) -> toml::Table {
    use toml::Value;
    let mut t = toml::Table::new();
    t.insert("type".into(), Value::String(b.kind().into()));
    match b {
        Block::Title { text }
        | Block::Subtitle { text }
        | Block::Label { text }
        | Block::Body { text }
        | Block::Quote { text }
        | Block::Conclusion { text }
        | Block::Signature { text }
        | Block::Footer { text } => poner(&mut t, "text", text),
        Block::Epigraph { text, source } => {
            poner(&mut t, "text", text);
            if let Some(s) = source {
                poner(&mut t, "source", s);
            }
        }
        Block::Verse { lines, role } => {
            t.insert("lines".into(), Value::Array(lines.iter().map(|l| md(l)).collect()));
            if let Some(r) = role {
                t.insert("role".into(), Value::String(r.clone()));
            }
        }
        Block::Attribution { author, role, work, date } => {
            for (k, v) in [("author", author), ("role", role), ("work", work), ("date", date)] {
                if let Some(v) = v {
                    poner(&mut t, k, v);
                }
            }
        }
        Block::Number { value, total } => {
            t.insert("value".into(), Value::Integer(i64::from(*value)));
            if let Some(x) = total {
                t.insert("total".into(), Value::Integer(i64::from(*x)));
            }
        }
        Block::Date { day, month } => {
            t.insert("day".into(), Value::Integer(i64::from(*day)));
            poner(&mut t, "month", month);
        }
        Block::List { order, items } => {
            let o = match order {
                model::ListOrder::None => "none",
                model::ListOrder::Numbered => "numbered",
                model::ListOrder::Chronological => "chronological",
            };
            t.insert("order".into(), Value::String(o.into()));
            t.insert("items".into(), Value::Array(items.iter().map(item_propio).collect()));
        }
        Block::Contrast { left, right } => {
            for (k, s) in [("left", left), ("right", right)] {
                let mut l = toml::Table::new();
                if let Some(x) = &s.label {
                    l.insert("label".into(), md(x));
                }
                if let Some(x) = &s.title {
                    l.insert("title".into(), md(x));
                }
                l.insert("items".into(), Value::Array(s.items.iter().map(item_propio).collect()));
                t.insert(k.into(), Value::Table(l));
            }
        }
    }
    t
}

fn item_propio(it: &Item) -> toml::Value {
    match it {
        Item::Text(x) => md(x),
        Item::Full(f) => {
            let mut t = toml::Table::new();
            t.insert("text".into(), md(&f.text));
            if let Some(l) = &f.label {
                t.insert("label".into(), md(l));
            }
            if let Some(r) = f.role {
                let s = match r {
                    Role::Affirm => "affirm",
                    Role::Deny => "deny",
                    Role::Neutral => "neutral",
                };
                t.insert("role".into(), toml::Value::String(s.into()));
            }
            toml::Value::Table(t)
        }
    }
}

// ── Formato ─────────────────────────────────────────────────

/// Medidas en px (con 72 ppi, 1 pt = 1 px). A4 y A5 a 150 ppi.
fn formato(f: &Format) -> FormatoSpec {
    match f {
        Format::Custom(c) => FormatoSpec::Medidas([c.width, c.height]),
        Format::Named(n) => FormatoSpec::Medidas(match n {
            FormatName::Square => [1080, 1080],
            FormatName::Portrait => [1080, 1350],
            FormatName::Story => [1080, 1920],
            FormatName::Landscape => [1600, 900],
            FormatName::A4 => [1240, 1754],
            FormatName::A5 => [874, 1240],
        }),
    }
}

// ── Bloques → parámetros del preset ─────────────────────────

/// Texto con formato (Typst en línea). Un salto de línea del texto es un salto de línea.
fn md(s: &str) -> toml::Value {
    let mut t = toml::Table::new();
    t.insert("md".into(), toml::Value::String(s.replace('\n', "\\\n")));
    toml::Value::Table(t)
}

fn texto_de(b: &Block) -> Option<String> {
    Some(match b {
        Block::Title { text }
        | Block::Subtitle { text }
        | Block::Label { text }
        | Block::Body { text }
        | Block::Quote { text }
        | Block::Epigraph { text, .. }
        | Block::Conclusion { text }
        | Block::Signature { text }
        | Block::Footer { text } => text.clone(),
        Block::Verse { lines, .. } => lines.join("\n"),
        _ => return None,
    })
}

fn repartir(
    bloques: &[Block],
    estilo: &str,
    sig: &StyleSig,
    cat: &Catalog,
) -> Result<BTreeMap<String, toml::Value>> {
    let entradas = sig
        .entradas
        .as_object()
        .filter(|o| !o.is_empty())
        .ok_or_else(|| {
            anyhow!("`{estilo}` aún no se puede construir desde una pieza (su firma no tiene `entradas`)")
        })?;

    let mut valores: BTreeMap<String, toml::Value> = BTreeMap::new();
    let mut textos: BTreeMap<String, Vec<String>> = BTreeMap::new();

    for b in bloques {
        let tipo = b.kind();
        // El bloque, o el más general de su cadena que el preset sepa colocar.
        let clave = cat
            .cadena(tipo)
            .into_iter()
            .find(|k| entradas.contains_key(*k))
            .ok_or_else(|| anyhow!("`{estilo}` no tiene dónde poner un bloque `{tipo}`"))?;
        let destino = &entradas[clave];

        // campo → parámetro, para los bloques con campos
        let campo = |nombre: &str| -> Result<&str> {
            destino
                .get(nombre)
                .and_then(|x| x.as_str())
                .ok_or_else(|| anyhow!("`{estilo}` no tiene dónde poner `{tipo}.{nombre}`"))
        };

        match b {
            Block::Attribution { author, role, work, date } => {
                for (nombre, v) in [("author", author), ("role", role), ("work", work), ("date", date)] {
                    if let Some(v) = v {
                        valores.insert(campo(nombre)?.into(), md(v));
                    }
                }
            }
            Block::Date { day, month } => {
                valores.insert(campo("day")?.into(), toml::Value::Integer(i64::from(*day)));
                valores.insert(campo("month")?.into(), md(month));
            }
            Block::Number { value, total } => {
                valores.insert(campo("value")?.into(), toml::Value::Integer(i64::from(*value)));
                if let Some(t) = total {
                    valores.insert(campo("total")?.into(), toml::Value::Integer(i64::from(*t)));
                }
            }
            Block::Contrast { left, right } => {
                valores.insert(campo("left")?.into(), lado(left, estilo)?);
                valores.insert(campo("right")?.into(), lado(right, estilo)?);
            }
            // Un estilo con entrada propia para el epígrafe lo coloca aparte (con su fuente);
            // uno que lo trata como cuerpo solo recibe el texto.
            Block::Epigraph { text, source } if destino.is_object() => {
                valores.insert(campo("text")?.into(), md(text));
                if let Some(s) = source {
                    valores.insert(campo("source")?.into(), md(s));
                }
            }
            Block::Epigraph { source: Some(_), .. } => bail!(
                "`{estilo}` no tiene dónde poner la fuente de un epígrafe: quítala o ponla en el texto"
            ),
            Block::List { .. } => bail!("el bloque `list` aún no se traduce a un preset"),
            otro => {
                let param = destino
                    .as_str()
                    .ok_or_else(|| anyhow!("`{estilo}`: la entrada de `{clave}` debería ser un parámetro"))?;
                if let Some(t) = texto_de(otro) {
                    textos.entry(param.into()).or_default().push(t);
                }
            }
        }
    }

    // Varios cuerpos en el mismo parámetro: párrafos.
    for (param, partes) in textos {
        valores.insert(param, md(&partes.join("\n\n")));
    }
    Ok(valores)
}

/// Un lado de `contraste`: `(rotulo:, titulo:, items:)`. Los nombres son los del preset.
fn lado(s: &Side, estilo: &str) -> Result<toml::Value> {
    let mut t = toml::Table::new();
    if let Some(l) = &s.label {
        t.insert("rotulo".into(), md(l));
    }
    if let Some(l) = &s.title {
        t.insert("titulo".into(), md(l));
    }
    let mut items = Vec::new();
    for it in &s.items {
        items.push(match it {
            Item::Text(x) => md(x),
            Item::Full(f) => {
                if f.label.is_some() {
                    bail!("`{estilo}` no tiene dónde poner la etiqueta de un ítem");
                }
                let mut i = toml::Table::new();
                i.insert("texto".into(), md(&f.text));
                if let Some(r) = f.role {
                    let rol = match r {
                        Role::Affirm => "afirma",
                        Role::Deny => "niega",
                        Role::Neutral => "neutro",
                    };
                    i.insert("rol".into(), toml::Value::String(rol.into()));
                }
                toml::Value::Table(i)
            }
        });
    }
    t.insert("items".into(), toml::Value::Array(items));
    Ok(toml::Value::Table(t))
}

// ── Marca ───────────────────────────────────────────────────

/// La marca como expresión de Typst: una función `(color, lienzo) -> contenido`, que el
/// preset llama con su propia tinta. Los textos y rutas pasan por `str_lit` y `asset_path`;
/// el único código que se emite es el que escribe esta función.
fn marca(b: &Brand) -> Result<toml::Value> {
    let tam = b.size.map(|s| format!(", proporcion: {s}")).unwrap_or_default();
    let l = &b.logo;
    let expr = if let Some(t) = &l.text {
        format!("marca.texto({}{tam})", str_lit(t))
    } else if let Some(img) = &l.image {
        let ruta = str_lit(&asset_path(img)?);
        if img.to_ascii_lowercase().ends_with(".svg") {
            format!("marca.svg(read({ruta}){tam})")
        } else {
            format!("marca.imagen(read({ruta}, encoding: none){tam})")
        }
    } else if let (Some(archivo), Some(f)) = (&l.typst, &l.function) {
        // tu función (color, lienzo) -> contenido, importada del archivo indicado
        format!("{{ import {}: {f}; {f} }}", str_lit(&asset_path(archivo)?))
    } else {
        bail!("la marca no tiene logo");
    };
    let mut t = toml::Table::new();
    t.insert("typst".into(), toml::Value::String(expr));
    Ok(toml::Value::Table(t))
}

// ── Tipografía ──────────────────────────────────────────────

fn tipografia(
    t: &model::Typography,
    estilo: &str,
    sig: &StyleSig,
    valores: &mut BTreeMap<String, toml::Value>,
) -> Result<()> {
    let tiene = |p: &str| sig.options.iter().any(|o| o == p);
    let (familia, peso) = match &t.body {
        None => (None, None),
        Some(Font::Family(f)) => (Some(f.clone()), None),
        Some(Font::Spec(x)) => (Some(x.family.clone()), x.weight),
    };
    if let Some(fam) = familia {
        if !tiene("fuente") {
            bail!("`{estilo}` no admite cambiar la tipografía del cuerpo (no tiene `fuente`)");
        }
        valores.insert("fuente".into(), toml::Value::String(fam));
    }
    if let Some(w) = peso {
        if !tiene("peso") {
            bail!("`{estilo}` no admite cambiar el peso (no tiene `peso`)");
        }
        valores.insert("peso".into(), toml::Value::Integer(i64::from(w)));
    }
    if t.heading.is_some() {
        bail!("`typography.heading` aún no se traduce: cada preset nombra distinto su fuente de títulos");
    }
    Ok(())
}

// ── Paleta ──────────────────────────────────────────────────

/// `(tema, image)` del post. Los nombres de paleta del documento pasan a la CLI en su
/// vocabulario de siempre (`bg`, `text`, `muted`, `primary`); los demás se dejan tal cual
/// (paletas de la CLI o temas de Instatypst, que valida Typst).
fn paleta(
    p: &PieceSpec,
    i: usize,
    doc: &Document,
    paletas: &mut HashMap<String, Paleta>,
    estilo: &str,
) -> Result<(Option<String>, Option<String>)> {
    match &p.palette {
        None => Ok((None, None)),
        Some(PaletteSpec::Name(n)) => {
            if let Some(def) = doc.palettes.get(n) {
                let m = paleta_legacy(def, paletas).with_context(|| format!("palettes.{n}"))?;
                paletas.insert(n.clone(), m);
            }
            Ok((Some(n.clone()), None))
        }
        Some(PaletteSpec::Def(d)) => {
            if let Some(f) = &d.file {
                bail!("`palette.file` ({f:?}) aún no se lee");
            }
            if let Some(img) = &d.from_image {
                if tiene_roles(d) {
                    bail!("`palette.from_image` no se combina todavía con roles explícitos");
                }
                return Ok((Some("auto".into()), Some(img.clone())));
            }
            let nombre = format!("__pieza_{i}");
            let m = paleta_legacy(d, paletas).with_context(|| format!("palette de `{estilo}`"))?;
            paletas.insert(nombre.clone(), m);
            Ok((Some(nombre), None))
        }
    }
}

fn tiene_roles(d: &PaletteDef) -> bool {
    [&d.background, &d.text, &d.muted, &d.accent, &d.line, &d.good, &d.bad]
        .iter()
        .any(|r| r.is_some())
}

/// Roles del contrato → las claves de siempre de la CLI. Los que faltan se derivan.
fn paleta_legacy(d: &PaletteDef, existentes: &HashMap<String, Paleta>) -> Result<Paleta> {
    for (rol, v) in [("line", &d.line), ("good", &d.good), ("bad", &d.bad)] {
        if v.is_some() {
            bail!(
                "el rol `{rol}` aún no pasa a los presets (el puente usa background, text, muted y accent)"
            );
        }
    }
    let mut m: Paleta = match &d.base {
        Some(b) => existentes
            .get(b)
            .cloned()
            .ok_or_else(|| anyhow!("`base = {b:?}` no es una paleta de la CLI ni del documento"))?,
        None => Paleta::new(),
    };
    if let Some(x) = &d.background {
        m.insert("bg".into(), x.clone());
    }
    if let Some(x) = &d.text {
        m.insert("text".into(), x.clone());
    }
    if let Some(x) = &d.muted {
        m.insert("muted".into(), x.clone());
    }
    if let Some(x) = &d.accent {
        m.insert("primary".into(), x.clone());
    }
    let bg = m
        .get("bg")
        .cloned()
        .ok_or_else(|| anyhow!("la paleta necesita `background`"))?;
    if !m.contains_key("text") {
        // el de más contraste con el fondo
        let claro = "#f5f5f5";
        let oscuro = "#111111";
        let t = if colors::contrast_ratio(claro, &bg)? >= colors::contrast_ratio(oscuro, &bg)? {
            claro
        } else {
            oscuro
        };
        m.insert("text".into(), t.into());
    }
    let texto = m["text"].clone();
    m.entry("muted".into()).or_insert_with(|| texto.clone());
    m.entry("primary".into()).or_insert(texto);
    Ok(m)
}

fn json_a_toml(v: &serde_json::Value) -> Result<toml::Value> {
    use serde_json::Value as J;
    Ok(match v {
        J::Null => bail!("TOML no tiene `null`"),
        J::Bool(b) => toml::Value::Boolean(*b),
        J::Number(n) => match (n.as_i64(), n.as_f64()) {
            (Some(i), _) => toml::Value::Integer(i),
            (_, Some(f)) => toml::Value::Float(f),
            _ => bail!("número no válido: {n}"),
        },
        J::String(s) => toml::Value::String(s.clone()),
        J::Array(a) => toml::Value::Array(a.iter().map(json_a_toml).collect::<Result<_>>()?),
        J::Object(o) => {
            let mut t = toml::Table::new();
            for (k, x) in o {
                t.insert(k.clone(), json_a_toml(x)?);
            }
            toml::Value::Table(t)
        }
    })
}
