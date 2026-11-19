//! Modelo de la pieza (`instatypst-piece/v1`) y su validador.
//!
//! CONGELADO (2026-11-18): no se añaden, quitan ni renombran campos, bloques, valores ni
//! códigos de error; un cambio es `v2`. Lo describe `docs/contrato-v1.md` y lo protege
//! `tests/congelado.rs`.
//!
//! El contrato es este modelo de datos, no un formato: TOML, JSON y YAML entran
//! por `parse` y llegan al mismo `Document`. `validate` es su definición
//! ejecutable: lo que no pasa por aquí no es una pieza válida.
//!
//! Capas de validación:
//!   1. estructural (este módulo): forma, tipos, valores permitidos.
//!   2. semántica (pendiente): la pieza cumple la firma del estilo.
//!
//! Los errores no son cadenas sueltas: cada `Issue` lleva `path` y `code`, para
//! que cualquier consumidor (la CLI, una web) pueda señalar el campo exacto.

use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

use crate::util::hex_to_rgb_tuple;

pub const SCHEMA: &str = "instatypst-piece/v1";

const MIN_LADO: u32 = 100;
const MAX_LADO: u32 = 4320;
const PPI_MIN: u32 = 72;
const PPI_MAX: u32 = 1200;

/// Opciones propias de un estilo: el contrato solo las transporta.
pub type Options = BTreeMap<String, serde_json::Value>;

// ── Documento ───────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Document {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub schema: Option<String>,
    /// Valores que heredan todas las piezas (cada pieza los pisa).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub defaults: Option<PieceSpec>,
    /// Cómo se produce (no qué se produce).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub render: Option<Render>,
    /// La marca de todas las piezas (una pieza puede cambiarla o quitarla).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub brand: Option<Brand>,
    /// Paletas con nombre del proyecto, ya en roles del contrato.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub palettes: BTreeMap<String, PaletteDef>,
    #[serde(default, rename = "post")]
    pub posts: Vec<PieceSpec>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Render {
    pub ppi: Option<u32>,
    pub image: Option<ImageKind>,
    #[serde(default)]
    pub fonts: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ImageKind {
    Png,
    Webp,
}

// ── Pieza (con todos los campos opcionales: sirve para `defaults` y `post`) ──

#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PieceSpec {
    pub name: Option<String>,
    pub style: Option<Style>,
    pub variant: Option<String>,
    pub format: Option<Format>,
    pub palette: Option<PaletteSpec>,
    pub background: Option<Background>,
    pub typography: Option<Typography>,
    /// `false` quita la marca de esta pieza; una tabla la cambia.
    pub brand: Option<BrandSpec>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub options: Options,
    pub content: Option<Content>,
}

/// Un estilo es un preset del catálogo (`"swiss"`) o una función propia.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(untagged)]
pub enum Style {
    Catalog(String),
    Custom(CustomStyle),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CustomStyle {
    pub file: String,
    #[serde(rename = "fn")]
    pub function: String,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(untagged)]
pub enum Format {
    Named(FormatName),
    Custom(CustomSize),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CustomSize {
    pub width: u32,
    pub height: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum FormatName {
    Square,
    Portrait,
    Story,
    Landscape,
    A4,
    A5,
}

// ── Paleta ──────────────────────────────────────────────────

// `Def` pesa mucho más que `Name`; meterlo en un `Box` cambiaría un tipo público del contrato
// congelado para ahorrar unos bytes por pieza, así que se deja como está.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(untagged)]
#[allow(clippy::large_enum_variant)]
pub enum PaletteSpec {
    Name(String),
    Def(PaletteDef),
}

/// Los roles del contrato. Una fuente (`base`, `from_image`, `file`) y/o roles
/// explícitos que la pisan; los roles que falten se derivan al resolver.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PaletteDef {
    pub base: Option<String>,
    pub from_image: Option<String>,
    pub file: Option<String>,
    pub background: Option<String>,
    pub text: Option<String>,
    pub muted: Option<String>,
    pub accent: Option<String>,
    pub line: Option<String>,
    pub good: Option<String>,
    pub bad: Option<String>,
}

impl PaletteDef {
    fn roles(&self) -> [(&'static str, &Option<String>); 7] {
        [
            ("background", &self.background),
            ("text", &self.text),
            ("muted", &self.muted),
            ("accent", &self.accent),
            ("line", &self.line),
            ("good", &self.good),
            ("bad", &self.bad),
        ]
    }

    fn fuentes(&self) -> usize {
        [&self.base, &self.from_image, &self.file]
            .iter()
            .filter(|f| f.is_some())
            .count()
    }
}

// ── Fondo y tipografía ──────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase", deny_unknown_fields)]
pub enum Background {
    Solid,
    Grid,
    Rules,
    Paper,
    Contours,
    Image { src: String },
}

#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Typography {
    pub heading: Option<Font>,
    pub body: Option<Font>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(untagged)]
pub enum Font {
    Family(String),
    Spec(FontSpec),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FontSpec {
    pub family: String,
    pub weight: Option<u16>,
}

// ── Marca ───────────────────────────────────────────────────

/// La marca de la serie: un handle en texto, una imagen (el SVG se recolorea con la
/// tinta de cada preset) o una función de Typst `(color, lienzo) -> contenido`.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Brand {
    pub logo: Logo,
    /// Alto de la marca relativo a √(ancho × alto) del lienzo; por omisión 1/30.
    /// No aplica a una función de Typst: ahí el tamaño lo decide la función.
    pub size: Option<f64>,
}

/// Exactamente una de: `text`, `image` o `typst` (con su `fn`).
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Logo {
    pub text: Option<String>,
    pub image: Option<String>,
    pub typst: Option<String>,
    #[serde(rename = "fn")]
    pub function: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum BrandSpec {
    Off,
    On(Brand),
}

impl Serialize for BrandSpec {
    fn serialize<S: serde::Serializer>(&self, s: S) -> std::result::Result<S::Ok, S::Error> {
        match self {
            BrandSpec::Off => s.serialize_bool(false),
            BrandSpec::On(b) => b.serialize(s),
        }
    }
}

impl<'de> Deserialize<'de> for BrandSpec {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> std::result::Result<Self, D::Error> {
        use serde::de::Error;
        match serde_json::Value::deserialize(d)? {
            serde_json::Value::Bool(false) => Ok(BrandSpec::Off),
            serde_json::Value::Bool(true) => Err(D::Error::custom(
                "`brand = true` no dice qué marca: usa `false` para quitarla o una tabla con `logo`",
            )),
            v => serde_json::from_value::<Brand>(v).map(BrandSpec::On).map_err(D::Error::custom),
        }
    }
}

// ── Contenido ───────────────────────────────────────────────

/// Typst (azúcar sintáctico) o la lista de bloques equivalente.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(untagged)]
pub enum Content {
    Markup(String),
    Blocks(Vec<Block>),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase", deny_unknown_fields)]
pub enum Block {
    Title {
        text: String,
    },
    Subtitle {
        text: String,
    },
    Label {
        text: String,
    },
    Body {
        text: String,
    },
    Quote {
        text: String,
    },
    /// Una cita breve que abre la pieza; `source` es de quién es (no es la atribución
    /// de la pieza). En un estilo que no tiene dónde ponerla aparte, se dibuja como el cuerpo.
    Epigraph {
        text: String,
        source: Option<String>,
    },
    Verse {
        lines: Vec<String>,
        role: Option<String>,
    },
    Conclusion {
        text: String,
    },
    Attribution {
        author: Option<String>,
        /// Cargo o descripción de quien habla.
        role: Option<String>,
        work: Option<String>,
        date: Option<String>,
    },
    List {
        #[serde(default)]
        order: ListOrder,
        items: Vec<Item>,
    },
    /// Dos lados enfrentados (versus, ley/evangelio…).
    Contrast {
        left: Side,
        right: Side,
    },
    Signature {
        text: String,
    },
    /// Línea de cierre bajo el contenido (pie, coda, pista).
    Footer {
        text: String,
    },
    /// Numeral propio (p. ej. el número del capítulo).
    Number {
        value: u32,
        /// Total, para «3 / 8».
        total: Option<u32>,
    },
    /// Una fecha de evento: día y mes (el mes, como se quiera mostrar).
    Date {
        day: u8,
        month: String,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ListOrder {
    #[default]
    None,
    Numbered,
    Chronological,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Side {
    /// Rótulo corto del lado («antes», «después»).
    pub label: Option<String>,
    pub title: Option<String>,
    pub items: Vec<Item>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(untagged)]
pub enum Item {
    Text(String),
    Full(ItemFull),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ItemFull {
    pub text: String,
    pub label: Option<String>,
    pub role: Option<Role>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Role {
    Neutral,
    Affirm,
    Deny,
}

impl Block {
    pub fn kind(&self) -> &'static str {
        match self {
            Block::Title { .. } => "title",
            Block::Subtitle { .. } => "subtitle",
            Block::Label { .. } => "label",
            Block::Body { .. } => "body",
            Block::Quote { .. } => "quote",
            Block::Epigraph { .. } => "epigraph",
            Block::Verse { .. } => "verse",
            Block::Conclusion { .. } => "conclusion",
            Block::Attribution { .. } => "attribution",
            Block::List { .. } => "list",
            Block::Contrast { .. } => "contrast",
            Block::Signature { .. } => "signature",
            Block::Footer { .. } => "footer",
            Block::Number { .. } => "number",
            Block::Date { .. } => "date",
        }
    }

    /// Bloques que, por función, aparecen una sola vez en la pieza. `label` no: marca quiebres,
    /// inicios de arcos internos o una idea enfatizada, y puede haber varios.
    fn es_unico(&self) -> bool {
        matches!(
            self,
            Block::Title { .. }
                | Block::Subtitle { .. }
                | Block::Conclusion { .. }
                | Block::Signature { .. }
                | Block::Footer { .. }
                | Block::Number { .. }
                | Block::Date { .. }
        )
    }
}

// ── Lectura de "texto o tabla" con errores que nombran el campo ──
//
// `#[serde(untagged)]` solo sabe decir «no coincide con ninguna variante». Aquí
// se decide por el tipo del valor (cadena o no) y se delega en el tipo exacto,
// para que el error diga qué campo o variante sobra («unknown field `tinta`»).
macro_rules! cadena_o_estructura {
    ($t:ident { $s:ident($sty:ty), $o:ident($oty:ty) }) => {
        impl<'de> Deserialize<'de> for $t {
            fn deserialize<D: serde::Deserializer<'de>>(
                d: D,
            ) -> std::result::Result<Self, D::Error> {
                use serde::de::Error;
                let v = serde_json::Value::deserialize(d)?;
                if v.is_string() {
                    serde_json::from_value::<$sty>(v)
                        .map($t::$s)
                        .map_err(D::Error::custom)
                } else {
                    serde_json::from_value::<$oty>(v)
                        .map($t::$o)
                        .map_err(D::Error::custom)
                }
            }
        }
    };
}

cadena_o_estructura!(Style { Catalog(String), Custom(CustomStyle) });
cadena_o_estructura!(Format { Named(FormatName), Custom(CustomSize) });
cadena_o_estructura!(PaletteSpec { Name(String), Def(PaletteDef) });
cadena_o_estructura!(Font { Family(String), Spec(FontSpec) });
cadena_o_estructura!(Item { Text(String), Full(ItemFull) });
cadena_o_estructura!(Content { Markup(String), Blocks(Vec<Block>) });

// ── Lectura ─────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Syntax {
    Toml,
    Json,
    Yaml,
}

impl Syntax {
    pub fn from_extension(path: &str) -> Option<Self> {
        match path.rsplit('.').next()?.to_ascii_lowercase().as_str() {
            "toml" => Some(Syntax::Toml),
            "json" => Some(Syntax::Json),
            "yaml" | "yml" => Some(Syntax::Yaml),
            _ => None,
        }
    }
}

/// Texto en cualquiera de las tres sintaxis → `Document`. Aún sin validar.
pub fn parse(texto: &str, sintaxis: Syntax) -> Result<Document> {
    match sintaxis {
        Syntax::Toml => toml::from_str(texto).map_err(|e| anyhow!("{e}")),
        Syntax::Json => serde_json::from_str(texto).map_err(|e| anyhow!("{e}")),
        Syntax::Yaml => serde_yaml::from_str(texto).map_err(|e| anyhow!("{e}")),
    }
}

/// `Document` → texto en la sintaxis pedida (para `--emit` y para depurar).
pub fn write(doc: &Document, sintaxis: Syntax) -> Result<String> {
    match sintaxis {
        Syntax::Toml => toml::to_string(doc).map_err(|e| anyhow!("{e}")),
        Syntax::Json => serde_json::to_string_pretty(doc).map_err(|e| anyhow!("{e}")),
        Syntax::Yaml => serde_yaml::to_string(doc).map_err(|e| anyhow!("{e}")),
    }
}

// ── Validación estructural ──────────────────────────────────

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Issue {
    pub path: String,
    pub code: &'static str,
    pub message: String,
}

fn issue(path: impl Into<String>, code: &'static str, message: impl Into<String>) -> Issue {
    Issue {
        path: path.into(),
        code,
        message: message.into(),
    }
}

/// Todas las incidencias de un documento. Vacío = estructuralmente válido.
pub fn validate(doc: &Document) -> Vec<Issue> {
    let mut v = Vec::new();

    if let Some(s) = &doc.schema {
        if s != SCHEMA {
            v.push(issue(
                "schema",
                "schema_version",
                format!("se esperaba {SCHEMA:?}, llegó {s:?}"),
            ));
        }
    }

    if let Some(r) = &doc.render {
        if let Some(ppi) = r.ppi {
            if !(PPI_MIN..=PPI_MAX).contains(&ppi) {
                v.push(issue(
                    "render.ppi",
                    "ppi_range",
                    format!("ppi {ppi} fuera de {PPI_MIN}–{PPI_MAX}"),
                ));
            }
        }
    }

    for (nombre, def) in &doc.palettes {
        validar_paleta(def, &format!("palettes.{nombre}"), &mut v);
    }
    if let Some(b) = &doc.brand {
        validar_marca(b, "brand", &mut v);
    }

    if let Some(d) = &doc.defaults {
        if d.name.is_some() {
            v.push(issue(
                "defaults.name",
                "name_in_defaults",
                "`name` identifica una pieza; no va en defaults",
            ));
        }
        if let Some(BrandSpec::On(b)) = &d.brand {
            validar_marca(b, "defaults.brand", &mut v);
        }
        validar_campos(d, "defaults", doc, &mut v);
    }

    let mut vistos: Vec<&str> = Vec::new();
    for (i, p) in doc.posts.iter().enumerate() {
        let base = format!("post[{i}]");
        match p.name.as_deref() {
            None => v.push(issue(
                format!("{base}.name"),
                "name_missing",
                "falta `name`",
            )),
            Some(n) => {
                if !nombre_valido(n) {
                    v.push(issue(
                        format!("{base}.name"),
                        "name_invalid",
                        format!("nombre no válido: {n:?} (sin `..`, ni rutas absolutas, ni vacío)"),
                    ));
                } else if vistos.contains(&n) {
                    v.push(issue(
                        format!("{base}.name"),
                        "name_duplicate",
                        format!("nombre repetido: {n:?}"),
                    ));
                } else {
                    vistos.push(n);
                }
            }
        }
        if let Some(BrandSpec::On(b)) = &p.brand {
            validar_marca(b, &format!("{base}.brand"), &mut v);
        }
        validar_campos(p, &base, doc, &mut v);

        // Obligatorios, contando lo que hereda de defaults.
        let hereda = |f: fn(&PieceSpec) -> bool| doc.defaults.as_ref().is_some_and(f);
        if p.style.is_none() && !hereda(|d| d.style.is_some()) {
            v.push(issue(
                format!("{base}.style"),
                "style_missing",
                "falta `style`",
            ));
        }
        if p.content.is_none() && !hereda(|d| d.content.is_some()) {
            v.push(issue(
                format!("{base}.content"),
                "content_missing",
                "falta `content`",
            ));
        }
    }

    v
}

fn ruta_de_recurso_valida(r: &str) -> bool {
    !r.trim().is_empty() && !r.contains('\0') && !r.split('/').any(|c| c == "..")
}

fn validar_marca(b: &Brand, base: &str, v: &mut Vec<Issue>) {
    let l = &b.logo;
    let fuentes = [l.text.is_some(), l.image.is_some(), l.typst.is_some()]
        .iter()
        .filter(|x| **x)
        .count();
    if fuentes != 1 {
        v.push(issue(
            format!("{base}.logo"),
            "brand_logo",
            "el logo es exactamente una de: `text`, `image` o `typst` (con `fn`)",
        ));
    }
    if l.typst.is_some() != l.function.is_some() {
        v.push(issue(
            format!("{base}.logo"),
            "brand_logo",
            "`typst` (el archivo) y `fn` (la función) van juntos",
        ));
    }
    if let Some(t) = &l.text {
        if t.trim().is_empty() {
            v.push(issue(
                format!("{base}.logo.text"),
                "brand_logo",
                "el texto de la marca está vacío",
            ));
        }
    }
    if let Some(f) = &l.function {
        if !crate::util::is_ident(f) {
            v.push(issue(
                format!("{base}.logo.fn"),
                "brand_logo",
                format!("{f:?} no es un nombre de función de Typst"),
            ));
        }
    }
    for (campo, ruta) in [("image", &l.image), ("typst", &l.typst)] {
        if let Some(r) = ruta {
            if !ruta_de_recurso_valida(r) {
                v.push(issue(
                    format!("{base}.logo.{campo}"),
                    "brand_path",
                    format!("ruta no válida: {r:?} (vacía, con `..` o con bytes nulos)"),
                ));
            }
        }
    }
    if let Some(img) = &l.image {
        let ext = img.rsplit('.').next().unwrap_or("").to_ascii_lowercase();
        if !["svg", "png", "jpg", "jpeg", "webp"].contains(&ext.as_str()) {
            v.push(issue(
                format!("{base}.logo.image"),
                "brand_image",
                format!("{img:?}: se admite svg, png, jpg, jpeg o webp"),
            ));
        }
    }
    if let Some(sz) = b.size {
        if !(0.01..=0.25).contains(&sz) {
            v.push(issue(
                format!("{base}.size"),
                "brand_size",
                format!("{sz} fuera de 0,01–0,25 (fracción de √(ancho × alto); por omisión 1/30 ≈ 0,033)"),
            ));
        }
        if l.typst.is_some() {
            v.push(issue(
                format!("{base}.size"),
                "brand_size",
                "el tamaño de una función de Typst lo decide la función",
            ));
        }
    }
}

fn nombre_valido(n: &str) -> bool {
    !n.is_empty() && !n.contains("..") && !n.contains('\0') && !n.starts_with('/')
}

fn validar_campos(p: &PieceSpec, base: &str, doc: &Document, v: &mut Vec<Issue>) {
    if let Some(Style::Catalog(s)) = &p.style {
        if s.trim().is_empty() {
            v.push(issue(
                format!("{base}.style"),
                "style_empty",
                "`style` vacío",
            ));
        }
    }
    if let Some(Style::Custom(c)) = &p.style {
        if !ruta_de_recurso_valida(&c.file) || !c.file.ends_with(".typ") {
            v.push(issue(
                format!("{base}.style.file"),
                "style_file",
                format!(
                    "{:?}: se espera un archivo .typ dentro del proyecto (sin `..`)",
                    c.file
                ),
            ));
        }
        if !crate::util::is_ident(&c.function) {
            v.push(issue(
                format!("{base}.style.fn"),
                "style_fn",
                format!("{:?} no es un nombre de función de Typst", c.function),
            ));
        }
    }

    if let Some(Format::Custom(CustomSize { width, height })) = &p.format {
        for (lado, n) in [("width", width), ("height", height)] {
            if !(MIN_LADO..=MAX_LADO).contains(n) {
                v.push(issue(
                    format!("{base}.format.{lado}"),
                    "format_size",
                    format!("{n} fuera de {MIN_LADO}–{MAX_LADO}"),
                ));
            }
        }
    }

    match &p.palette {
        Some(PaletteSpec::Def(d)) => validar_paleta(d, &format!("{base}.palette"), v),
        Some(PaletteSpec::Name(n)) if n.trim().is_empty() => v.push(issue(
            format!("{base}.palette"),
            "palette_empty",
            "paleta sin nombre",
        )),
        _ => {}
    }
    let _ = doc; // el nombre de paleta se resuelve contra el catálogo en la capa semántica

    if let Some(Content::Markup(m)) = &p.content {
        if m.trim().is_empty() {
            v.push(issue(
                format!("{base}.content"),
                "content_empty",
                "el contenido está vacío",
            ));
        } else {
            // Sintaxis Typst: lista cerrada, y los bloques que resultan siguen
            // las mismas reglas que si se hubieran escrito a mano.
            let (bloques, problemas) = crate::markup::parse(m);
            if problemas.is_empty() {
                validar_bloques(&bloques, &format!("{base}.content"), v);
            }
            for i in problemas {
                v.push(Issue {
                    path: format!("{base}.content{}", i.path),
                    ..i
                });
            }
        }
    }
    if let Some(Content::Blocks(bs)) = &p.content {
        validar_bloques(bs, &format!("{base}.content"), v);
    }
}

fn validar_paleta(d: &PaletteDef, base: &str, v: &mut Vec<Issue>) {
    if d.fuentes() > 1 {
        v.push(issue(
            base,
            "palette_source_conflict",
            "usa una sola fuente: `base`, `from_image` o `file`",
        ));
    }
    if d.fuentes() == 0 && d.background.is_none() {
        v.push(issue(
            format!("{base}.background"),
            "palette_incomplete",
            "sin fuente, la paleta necesita al menos `background`",
        ));
    }
    for (rol, valor) in d.roles() {
        if let Some(hex) = valor {
            if hex_to_rgb_tuple(hex).is_err() {
                v.push(issue(
                    format!("{base}.{rol}"),
                    "color_invalid",
                    format!("color no válido: {hex:?} (se espera #RRGGBB)"),
                ));
            }
        }
    }
}

fn validar_bloques(bs: &[Block], base: &str, v: &mut Vec<Issue>) {
    if bs.is_empty() {
        v.push(issue(base, "content_empty", "no hay bloques"));
    }
    let mut vistos: Vec<&'static str> = Vec::new();
    for (i, b) in bs.iter().enumerate() {
        let ruta = format!("{base}[{i}]");
        if b.es_unico() {
            if vistos.contains(&b.kind()) {
                v.push(issue(
                    &ruta,
                    "block_duplicate",
                    format!("`{}` solo puede aparecer una vez", b.kind()),
                ));
            }
            vistos.push(b.kind());
        }
        match b {
            Block::Title { text }
            | Block::Subtitle { text }
            | Block::Label { text }
            | Block::Body { text }
            | Block::Quote { text }
            | Block::Epigraph { text, .. }
            | Block::Conclusion { text }
            | Block::Signature { text }
            | Block::Footer { text } => {
                if text.trim().is_empty() {
                    v.push(issue(
                        format!("{ruta}.text"),
                        "text_empty",
                        format!("`{}` sin texto", b.kind()),
                    ));
                }
            }
            Block::Number { total, value } => {
                if let Some(t) = total {
                    if t < value {
                        v.push(issue(
                            format!("{ruta}.total"),
                            "number_total",
                            format!("el total ({t}) no puede ser menor que el número ({value})"),
                        ));
                    }
                }
            }
            Block::Date { day, month } => {
                if !(1..=31).contains(day) {
                    v.push(issue(
                        format!("{ruta}.day"),
                        "date_invalid",
                        format!("día {day} fuera de 1–31"),
                    ));
                }
                if month.trim().is_empty() {
                    v.push(issue(
                        format!("{ruta}.month"),
                        "date_invalid",
                        "`month` vacío",
                    ));
                }
            }
            Block::Verse { lines, .. } => {
                if lines.is_empty() {
                    v.push(issue(
                        format!("{ruta}.lines"),
                        "verse_empty",
                        "verso sin líneas",
                    ));
                }
            }
            Block::Attribution {
                author, work, date, ..
            } => {
                if author.is_none() && work.is_none() && date.is_none() {
                    v.push(issue(
                        &ruta,
                        "attribution_empty",
                        "atribución sin `author`, `work` ni `date`",
                    ));
                }
            }
            Block::List { items, .. } => {
                if items.is_empty() {
                    v.push(issue(
                        format!("{ruta}.items"),
                        "list_empty",
                        "lista sin ítems",
                    ));
                }
            }
            Block::Contrast { left, right } => {
                for (lado, s) in [("left", left), ("right", right)] {
                    if s.items.is_empty() {
                        v.push(issue(
                            format!("{ruta}.{lado}.items"),
                            "list_empty",
                            "lado del contraste sin ítems",
                        ));
                    }
                }
            }
        }
    }
}

/// Lee, valida y devuelve el documento, o todas las incidencias juntas.
pub fn load(texto: &str, sintaxis: Syntax) -> std::result::Result<Document, Vec<Issue>> {
    let doc = parse(texto, sintaxis).map_err(|e| vec![issue("", "parse", e.to_string())])?;
    let v = validate(&doc);
    if v.is_empty() {
        Ok(doc)
    } else {
        Err(v)
    }
}

// ── Herencia ────────────────────────────────────────────────

impl Document {
    /// La pieza `i` con lo que hereda de `defaults`: lo que la pieza declara
    /// manda; `options` se mezcla clave a clave.
    pub fn merged(&self, i: usize) -> PieceSpec {
        let p = &self.posts[i];
        let Some(d) = &self.defaults else {
            return p.clone();
        };
        let mut options = d.options.clone();
        options.extend(p.options.clone());
        PieceSpec {
            name: p.name.clone(),
            style: p.style.clone().or_else(|| d.style.clone()),
            variant: p.variant.clone().or_else(|| d.variant.clone()),
            format: p.format.clone().or_else(|| d.format.clone()),
            palette: p.palette.clone().or_else(|| d.palette.clone()),
            background: p.background.clone().or_else(|| d.background.clone()),
            typography: p.typography.clone().or_else(|| d.typography.clone()),
            brand: p.brand.clone().or_else(|| d.brand.clone()),
            options,
            content: p.content.clone().or_else(|| d.content.clone()),
        }
    }
}

// ── Capa semántica: la pieza contra la firma del estilo ─────

/// Las firmas de los estilos, tal como las exporta Instatypst
/// (`scripts/exportar-firmas.py` → `schema/firmas.json`).
#[derive(Debug, Clone, Deserialize)]
pub struct Catalog {
    pub blocks: Vec<String>,
    /// «X es un caso de Y» (verse → quote → body).
    #[serde(default)]
    pub block_parents: BTreeMap<String, String>,
    pub styles: BTreeMap<String, StyleSig>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct StyleSig {
    #[serde(default)]
    pub uso: String,
    /// Parámetros posicionales del preset, en orden.
    #[serde(default)]
    pub pos: Vec<String>,
    pub bloques: Slots,
    /// Cómo se reparte una pieza en los parámetros del preset (ver `firmas.typ`).
    /// Vacío: el preset aún no se puede construir desde una pieza.
    #[serde(default)]
    pub entradas: serde_json::Value,
    /// Parámetros con nombre del preset, derivados de su código.
    #[serde(default)]
    pub options: Vec<String>,
    /// `marca:` acepta una función `(color, lienzo) -> contenido`, además de un contenido.
    #[serde(default, rename = "marca-funcion")]
    pub marca_funcion: bool,
}

/// Parámetros que el compilador fija a partir de los ejes de la pieza; no se
/// pasan en `options` (se usaría `format` / `palette` / `variant`).
const OPCIONES_RESERVADAS: &[(&str, &str)] =
    &[("formato", "format"), ("tema", "palette o variant")];

#[derive(Debug, Clone, Deserialize)]
pub struct Slots {
    pub requeridos: Vec<Slot>,
    pub opcionales: Vec<String>,
}

/// Un requisito: un bloque, o «uno de» varios.
#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
pub enum Slot {
    One(String),
    AnyOf(Vec<String>),
}

impl Slot {
    fn alternativas(&self) -> Vec<&str> {
        match self {
            Slot::One(s) => vec![s.as_str()],
            Slot::AnyOf(v) => v.iter().map(String::as_str).collect(),
        }
    }
}

impl Catalog {
    /// El bloque y sus ancestros, del más específico al más general.
    pub fn cadena<'a>(&'a self, bloque: &'a str) -> Vec<&'a str> {
        let mut v = vec![bloque];
        let mut actual = bloque;
        while let Some(p) = self.block_parents.get(actual) {
            if v.contains(&p.as_str()) {
                break; // ciclo: lo impide el test de Instatypst; aquí, a salvo
            }
            v.push(p);
            actual = p;
        }
        v
    }

    /// Las firmas que vienen con la CLI (copia de `schema/firmas.json` de Instatypst).
    pub fn embedded() -> Self {
        serde_json::from_str(include_str!("firmas.json")).expect("src/firmas.json es válido")
    }
}

/// Incidencias de la pieza frente al catálogo. El contenido en sintaxis Typst
/// se convierte antes en bloques (`markup::parse`), así que se comprueba igual.
pub fn validate_semantic(doc: &Document, cat: &Catalog) -> Vec<Issue> {
    let mut v = Vec::new();
    for i in 0..doc.posts.len() {
        let base = format!("post[{i}]");
        let p = doc.merged(i);

        let Some(Style::Catalog(nombre)) = &p.style else {
            continue; // sin estilo (ya lo dice validate) o estilo propio: no hay firma
        };
        let Some(sig) = cat.styles.get(nombre) else {
            let validos: Vec<&str> = cat.styles.keys().map(String::as_str).collect();
            v.push(issue(
                format!("{base}.style"),
                "style_unknown",
                format!(
                    "el estilo {nombre:?} no existe; los del catálogo: {}",
                    validos.join(", ")
                ),
            ));
            continue;
        };

        for clave in p.options.keys() {
            if let Some((_, eje)) = OPCIONES_RESERVADAS.iter().find(|(r, _)| r == clave) {
                v.push(issue(
                    format!("{base}.options.{clave}"),
                    "option_reserved",
                    format!("{clave:?} lo fija el contrato: usa `{eje}`"),
                ));
            } else if !sig.options.iter().any(|o| o == clave) {
                v.push(issue(
                    format!("{base}.options.{clave}"),
                    "option_unknown",
                    format!(
                        "`{nombre}` no tiene la opción {clave:?}; acepta: {}",
                        sig.options.join(", ")
                    ),
                ));
            }
        }

        let bs = match &p.content {
            Some(Content::Blocks(b)) => b.clone(),
            Some(Content::Markup(m)) => {
                let (b, problemas) = crate::markup::parse(m);
                if !problemas.is_empty() {
                    continue; // ya lo señala `validate`
                }
                b
            }
            None => continue,
        };
        let bs = &bs;
        let aceptados: Vec<&str> = sig
            .bloques
            .requeridos
            .iter()
            .flat_map(Slot::alternativas)
            .chain(sig.bloques.opcionales.iter().map(String::as_str))
            .collect();

        // Cada bloque cuenta como el más específico de su cadena que el estilo
        // acepta: un `verse` vale como `quote` o, si no, como `body`. Y solo como
        // ese: un `epigraph` en un estilo que lo tiene aparte (`liso`) no cubre
        // el `body` que ese mismo estilo exige.
        let resueltos: Vec<Option<&str>> = bs
            .iter()
            .map(|b| {
                cat.cadena(b.kind())
                    .into_iter()
                    .find(|k| aceptados.contains(k))
            })
            .collect();

        for req in &sig.bloques.requeridos {
            let alt = req.alternativas();
            if !resueltos.iter().flatten().any(|r| alt.contains(r)) {
                v.push(issue(
                    format!("{base}.content"),
                    "block_missing",
                    format!("`{nombre}` necesita {}", alt.join(" o ")),
                ));
            }
        }

        for (j, b) in bs.iter().enumerate() {
            if resueltos[j].is_none() {
                v.push(issue(
                    format!("{base}.content[{j}]"),
                    "block_unsupported",
                    format!(
                        "`{nombre}` no acepta `{}`; acepta: {}",
                        b.kind(),
                        aceptados.join(", ")
                    ),
                ));
            }
        }

        // Un bloque que el estilo coloca en un solo parámetro (un `label` en su única
        // etiqueta) no puede repetirse: no hay a dónde mandar el segundo. Los textos que
        // heredan de `body` (cuerpo, cita, epígrafe, verso) sí se unen en párrafos.
        let mut ocupados: Vec<&str> = Vec::new();
        for (j, b) in bs.iter().enumerate() {
            let cadena = cat.cadena(b.kind());
            if cadena.contains(&"body") {
                continue;
            }
            let Some(clave) = cadena.into_iter().find(|k| sig.entradas.get(*k).is_some()) else {
                continue;
            };
            if !sig.entradas[clave].is_string() {
                continue;
            }
            if ocupados.contains(&clave) {
                v.push(issue(
                    format!("{base}.content[{j}]"),
                    "block_repeated",
                    format!(
                        "`{nombre}` solo tiene un lugar para `{}`; esta pieza trae más de uno",
                        b.kind()
                    ),
                ));
            }
            ocupados.push(clave);
        }
    }
    v
}

/// `load` más la capa semántica: una sola lista de incidencias.
pub fn load_checked(
    texto: &str,
    sintaxis: Syntax,
    cat: &Catalog,
) -> std::result::Result<Document, Vec<Issue>> {
    let doc = load(texto, sintaxis)?;
    let v = validate_semantic(&doc, cat);
    if v.is_empty() {
        Ok(doc)
    } else {
        Err(v)
    }
}
