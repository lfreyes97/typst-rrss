//! `posts.toml`: esquema, herencia de `[defaults]` y validación.
//!
//! Todas las tablas rechazan campos desconocidos (un typo falla en voz alta) y
//! `Config::resolve` entrega un `Post` ya validado: el resto del programa no
//! vuelve a mirar el TOML crudo.

use anyhow::{anyhow, bail, Context, Result};
use serde::Deserialize;
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::Path;

use crate::typst::is_ident;

/// Lado máximo de un lienzo en pt (con ppi 72, px). Corta pedidos absurdos.
const MAX_LADO: u32 = 4320;
const MIN_LADO: u32 = 100;
pub const PPI_DEFECTO: u32 = 72;

/// Formatos con nombre; con `ppi = 72`, 1 pt = 1 px.
pub const FORMATOS: &[(&str, (u32, u32))] = &[
    ("feed-4x5", (1080, 1350)),
    ("feed-1x1", (1080, 1080)),
    ("story", (1080, 1920)),
    // Los de `sizes` de instatypst, con sus mismos nombres.
    ("instagram", (1080, 1080)),
    ("twitter", (1600, 900)),
    ("linkedin", (1200, 1200)),
];

#[derive(Debug, Deserialize, Clone, PartialEq)]
#[serde(untagged)]
pub enum FormatoSpec {
    Nombre(String),
    Medidas([u32; 2]),
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    #[serde(default)]
    pub defaults: Defaults,
    /// Paletas propias: `[themes.nombre]` con bg, text, muted, primary…
    pub themes: Option<HashMap<String, HashMap<String, String>>>,
    #[serde(default, rename = "post")]
    pub posts: Vec<RawPost>,
}

#[derive(Debug, Deserialize, Default, Clone)]
#[serde(deny_unknown_fields)]
pub struct Defaults {
    pub preset: Option<String>,
    pub formato: Option<FormatoSpec>,
    pub ppi: Option<u32>,
    pub tema: Option<String>,
    pub image: Option<String>,
    /// Carpetas de fuentes que se pasan a `typst --font-path`.
    #[serde(default)]
    pub fonts: Vec<String>,
    /// Args para cualquier preset. Cada preset tiene su propia firma: lo que no
    /// vale para todos va en `por_preset`.
    #[serde(default)]
    pub args: toml::Table,
    /// `[defaults.por_preset.liso]`: args por defecto de un solo preset
    /// (se escribe `liso` o `carteles.liso`, da igual).
    #[serde(default)]
    pub por_preset: HashMap<String, toml::Table>,
    /// Variables de Typst compartidas (`#let nombre = valor`), en el orden escrito.
    /// Se ven desde `{ kw = "nombre" }` y desde los textos con formato (`md`).
    #[serde(default)]
    pub defs: toml::Table,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct RawPost {
    pub name: String,
    pub preset: Option<String>,
    pub formato: Option<FormatoSpec>,
    pub ppi: Option<u32>,
    /// Nombre de paleta, `"auto"` / `"auto-matugen"` (desde `image`) o un tema de instatypst.
    pub tema: Option<String>,
    /// Imagen de la que se extrae el tema. No es el fondo: ese va en `args`.
    pub image: Option<String>,
    /// Argumentos posicionales del preset, en orden (`frase`, `remate`…).
    #[serde(default)]
    pub pos: Vec<toml::Value>,
    /// Argumentos con nombre del preset; se combinan sobre los de `[defaults]`.
    #[serde(default)]
    pub args: toml::Table,
    /// Variables de este post; pisan las de `[defaults.defs]` con el mismo nombre.
    #[serde(default)]
    pub defs: toml::Table,
}

/// Un post con la herencia aplicada y todo validado.
#[derive(Debug, Clone)]
pub struct Post {
    pub name: String,
    /// Ruta del preset en Typst, p. ej. `carteles.suizo`.
    pub preset: String,
    /// `None`: lo decide el preset (cada uno tiene el suyo: `folio` es cuadrado, `escalera` vertical…).
    pub formato: Option<(u32, u32)>,
    pub ppi: u32,
    pub tema: Option<String>,
    pub image: Option<String>,
    pub pos: Vec<toml::Value>,
    pub args: toml::Table,
    pub defs: toml::Table,
}

impl Config {
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Self> {
        let path = path.as_ref();
        let content = fs::read_to_string(path)
            .with_context(|| format!("No se pudo leer {}", path.display()))?;
        Self::parse(&content).with_context(|| format!("{} no es válido", path.display()))
    }

    pub fn parse(content: &str) -> Result<Self> {
        Ok(toml::from_str(content)?)
    }

    /// Un post descrito con flags: pasa por `resolve`, igual que los del TOML.
    pub fn post_desde_flags(&self, name: &str, f: &Flags) -> Result<Post> {
        let mut args = toml::Table::new();
        for raw in f.args {
            let (k, v) = raw
                .split_once('=')
                .ok_or_else(|| anyhow!("--arg espera CLAVE=VALOR, no {raw:?}"))?;
            args.insert(k.to_string(), valor_de_flag(v));
        }
        let raw = RawPost {
            name: name.to_string(),
            preset: Some(f.preset.to_string()),
            formato: f.formato.map(parse_formato),
            ppi: f.ppi,
            tema: f.tema.map(String::from),
            image: f.image.map(String::from),
            pos: f
                .pos
                .iter()
                .map(|s| toml::Value::String(s.clone()))
                .collect(),
            args,
            defs: toml::Table::new(),
        };
        self.resolve(&raw)
    }

    /// Resuelve todos los posts; falla en el primero inválido o si se repite un nombre.
    pub fn resolve_all(&self) -> Result<Vec<Post>> {
        let mut vistos = HashSet::new();
        let mut out = Vec::new();
        for raw in &self.posts {
            if !vistos.insert(raw.name.as_str()) {
                bail!("el post {:?} está repetido", raw.name);
            }
            out.push(
                self.resolve(raw)
                    .with_context(|| format!("post {:?}", raw.name))?,
            );
        }
        Ok(out)
    }

    pub fn resolve(&self, raw: &RawPost) -> Result<Post> {
        let d = &self.defaults;
        validar_nombre(&raw.name)?;

        let preset = raw
            .preset
            .as_ref()
            .or(d.preset.as_ref())
            .ok_or_else(|| anyhow!("falta `preset` (ni el post ni [defaults] lo definen)"))?;
        let preset = normalizar_preset(preset)?;

        let formato = resolver_formato(raw.formato.as_ref().or(d.formato.as_ref()))?;

        let ppi = raw.ppi.or(d.ppi).unwrap_or(PPI_DEFECTO);
        if !(72..=1200).contains(&ppi) {
            bail!("ppi debe estar entre 72 y 1200, no {ppi}");
        }

        let tema = raw.tema.clone().or_else(|| d.tema.clone());
        let image = raw.image.clone().or_else(|| d.image.clone());
        if matches!(tema.as_deref(), Some("auto" | "auto-matugen")) && image.is_none() {
            bail!(
                "tema {:?} necesita `image` para extraer los colores",
                tema.unwrap()
            );
        }

        // Orden de prioridad: [defaults.args] < [defaults.por_preset.<preset>] < [post.args]
        let mut args = d.args.clone();
        for (nombre, tabla) in &d.por_preset {
            if normalizar_preset(nombre).with_context(|| format!("por_preset.{nombre}"))? == preset
            {
                args.extend(tabla.clone());
            }
        }
        for (k, v) in &raw.args {
            args.insert(k.clone(), v.clone());
        }

        let mut defs = d.defs.clone();
        defs.extend(raw.defs.clone());
        for k in defs.keys() {
            if !is_ident(k) || k.starts_with('_') {
                bail!("defs.{k}: nombre no válido (identificador sin `_` inicial)");
            }
        }

        Ok(Post {
            name: raw.name.clone(),
            preset,
            formato,
            ppi,
            tema,
            image,
            pos: raw.pos.clone(),
            args,
            defs,
        })
    }
}

/// Los datos de una pieza tal como llegan de la línea de comandos.
pub struct Flags<'a> {
    pub preset: &'a str,
    pub pos: &'a [String],
    pub tema: Option<&'a str>,
    pub image: Option<&'a str>,
    pub formato: Option<&'a str>,
    pub ppi: Option<u32>,
    pub args: &'a [String],
}

/// `ANCHOxALTO` son medidas; cualquier otra cosa, un nombre de formato.
pub fn parse_formato(s: &str) -> FormatoSpec {
    let medidas = s
        .split_once('x')
        .and_then(|(w, h)| Some([w.parse().ok()?, h.parse().ok()?]));
    match medidas {
        Some(m) => FormatoSpec::Medidas(m),
        None => FormatoSpec::Nombre(s.to_string()),
    }
}

/// El valor de `--arg`: TOML si lo es (número, booleano, array, tabla), texto si no.
fn valor_de_flag(raw: &str) -> toml::Value {
    toml::from_str::<toml::Table>(&format!("v = {raw}"))
        .ok()
        .and_then(|mut t| t.remove("v"))
        .unwrap_or_else(|| toml::Value::String(raw.to_string()))
}

/// El nombre acaba en rutas de archivo: sin `..`, sin absolutas, sin nulos.
fn validar_nombre(n: &str) -> Result<()> {
    let ok = !n.is_empty()
        && !n.starts_with('/')
        && n.split('/').all(|c| !c.is_empty() && c != "." && c != "..")
        && n.chars().all(|c| c.is_alphanumeric() || "-_/.".contains(c));
    if !ok {
        bail!("nombre de post no válido: {n:?} (letras, números, `-`, `_`, `.`; `/` para subcarpetas)");
    }
    Ok(())
}

/// `suizo` → `carteles.suizo`; `fleurones.orla` se respeta tal cual.
fn normalizar_preset(p: &str) -> Result<String> {
    if p.is_empty() || !p.split('.').all(is_ident) {
        bail!("preset no válido: {p:?} (se espera un nombre como `suizo` o `fleurones.orla`)");
    }
    Ok(if p.contains('.') {
        p.to_string()
    } else {
        format!("carteles.{p}")
    })
}

fn resolver_formato(f: Option<&FormatoSpec>) -> Result<Option<(u32, u32)>> {
    let (w, h) = match f {
        None => return Ok(None),
        Some(FormatoSpec::Medidas([w, h])) => (*w, *h),
        Some(FormatoSpec::Nombre(n)) => FORMATOS
            .iter()
            .find(|(k, _)| k == n)
            .map(|(_, d)| *d)
            .ok_or_else(|| {
                let validos: Vec<_> = FORMATOS.iter().map(|(k, _)| *k).collect();
                anyhow!(
                    "formato desconocido {n:?}; usa {} o [ancho, alto]",
                    validos.join(", ")
                )
            })?,
    };
    if !(MIN_LADO..=MAX_LADO).contains(&w) || !(MIN_LADO..=MAX_LADO).contains(&h) {
        bail!("formato {w}×{h} fuera de rango ({MIN_LADO}–{MAX_LADO})");
    }
    Ok(Some((w, h)))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn una(toml: &str) -> Result<Post> {
        let cfg = Config::parse(toml)?;
        cfg.resolve_all()?
            .into_iter()
            .next()
            .ok_or_else(|| anyhow!("sin posts"))
    }

    #[test]
    fn herencia_y_valores_por_defecto() {
        let p = una(r#"
            [defaults]
            preset = "suizo"
            [defaults.args]
            marca = "@yo"
            [[post]]
            name = "a"
            pos = ["frase", "remate"]
            [post.args]
            papel = "x"
        "#)
        .unwrap();
        assert_eq!(p.preset, "carteles.suizo");
        assert_eq!(p.formato, None); // lo decide el preset
        assert_eq!(p.ppi, 72);
        assert_eq!(p.args.len(), 2); // marca heredada + papel propio
        assert_eq!(p.pos.len(), 2);
    }

    #[test]
    fn el_post_pisa_los_defaults() {
        let p = una(r#"
            [defaults]
            preset = "liso"
            formato = "story"
            [defaults.args]
            peso = 500
            [[post]]
            name = "a"
            preset = "fleurones.orla"
            formato = [1000, 500]
            [post.args]
            peso = 700
        "#)
        .unwrap();
        assert_eq!(p.preset, "fleurones.orla");
        assert_eq!(p.formato, Some((1000, 500)));
        assert_eq!(p.args["peso"].as_integer(), Some(700));
    }

    #[test]
    fn campo_desconocido_falla() {
        // `recolor` existía en el posts.toml viejo y se ignoraba sin avisar.
        let e = una(r#"
            [[post]]
            name = "a"
            preset = "liso"
            recolor = true
        "#)
        .unwrap_err();
        assert!(format!("{e:#}").contains("recolor"), "{e:#}");
        assert!(Config::parse("[defaults]\nbrand = \"x\"").is_err());
        assert!(Config::parse("colores = 1").is_err());
    }

    #[test]
    fn errores_de_validacion() {
        let mal = |t: &str| format!("{:#}", una(t).unwrap_err());
        assert!(mal("[[post]]\nname='a'").contains("preset"));
        assert!(mal("[[post]]\nname='a'\npreset='liso'\nformato='cuadrado'").contains("formato"));
        assert!(mal("[[post]]\nname='a'\npreset='liso'\nformato=[99999,10]").contains("rango"));
        assert!(mal("[[post]]\nname='a'\npreset='liso'\nppi=10").contains("ppi"));
        assert!(mal("[[post]]\nname='a'\npreset='li so'").contains("preset"));
        assert!(mal("[[post]]\nname='a'\npreset='liso'\ntema='auto'").contains("image"));
    }

    #[test]
    fn nombres_seguros() {
        for n in ["../x", "/abs", "a//b", "a b", ""] {
            assert!(validar_nombre(n).is_err(), "{n:?}");
        }
        assert!(validar_nombre("serie/post-1.v2").is_ok());
    }

    #[test]
    fn nombre_repetido() {
        let cfg =
            Config::parse("[defaults]\npreset='liso'\n[[post]]\nname='a'\n[[post]]\nname='a'")
                .unwrap();
        assert!(cfg
            .resolve_all()
            .unwrap_err()
            .to_string()
            .contains("repetido"));
    }

    #[test]
    fn flags_pasan_por_la_misma_validacion() {
        let cfg = Config::parse("").unwrap();
        let pos = vec!["hola".to_string()];
        let args = vec![
            "peso=500".into(),
            "marca=@yo".into(),
            "fondo={color=\"#691322\"}".into(),
        ];
        let f = Flags {
            preset: "liso",
            pos: &pos,
            tema: None,
            image: None,
            formato: Some("800x600"),
            ppi: None,
            args: &args,
        };
        let p = cfg.post_desde_flags("a", &f).unwrap();
        assert_eq!(p.preset, "carteles.liso");
        assert_eq!(p.formato, Some((800, 600)));
        assert_eq!(p.args["peso"].as_integer(), Some(500));
        assert_eq!(p.args["marca"].as_str(), Some("@yo"));
        assert!(p.args["fondo"].is_table());

        let mal = Flags {
            formato: Some("cuadrado"),
            ..f
        };
        assert!(cfg.post_desde_flags("a", &mal).is_err());
        let sin_igual = vec!["peso".to_string()];
        let mal = Flags {
            args: &sin_igual,
            ..f
        };
        assert!(cfg.post_desde_flags("a", &mal).is_err());
        let mal = Flags {
            preset: "li so",
            ..f
        };
        assert!(cfg.post_desde_flags("a", &mal).is_err());
        assert!(cfg.post_desde_flags("../x", &f).is_err());
    }

    #[test]
    fn por_preset_solo_llega_a_su_preset() {
        let cfg = Config::parse(
            r#"
            [defaults]
            preset = "liso"
            [defaults.args]
            marca = "@yo"
            [defaults.por_preset.liso]
            fuente = "EB Garamond 12"
            peso = 500
            [[post]]
            name = "a"
            [[post]]
            name = "b"
            preset = "epigrafe"
            [[post]]
            name = "c"
            [post.args]
            peso = 900
            "#,
        )
        .unwrap();
        let ps = cfg.resolve_all().unwrap();
        assert!(ps[0].args.contains_key("fuente") && ps[0].args.contains_key("marca"));
        assert!(!ps[1].args.contains_key("fuente") && ps[1].args.contains_key("marca"));
        assert_eq!(ps[2].args["peso"].as_integer(), Some(900)); // el post gana
        assert_eq!(ps[2].args["fuente"].as_str(), Some("EB Garamond 12"));
    }
}
