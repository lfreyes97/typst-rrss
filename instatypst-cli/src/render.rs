//! `Post` → archivo `.typ` que llama a un preset de instatypst.
//!
//! Salida:
//!
//! ```typst
//! #import "@local/instatypst:0.1.0": *
//!
//! #carteles.suizo(
//!   "frase",
//!   formato: (1080, 1350),
//!   tema: (fondo: rgb("#..."), tinta: ..., suave: ..., acento: ..., linea: ...),
//! )
//! ```

use anyhow::{anyhow, bail, Context, Result};
use std::collections::HashMap;

use crate::config::{Config, Post};
use crate::typst::{named_args, str_lit, value_lit};
use crate::{colors, themes};

/// Mínimos WCAG: texto normal y elementos gráficos.
const CONTRASTE_TEXTO: f64 = 4.5;
const CONTRASTE_ACENTO: f64 = 3.0;

/// Nombres de instatypst disponibles dentro de los textos con formato.
const ALCANCE: &[&str] = &[
    "carteles",
    "fleurones",
    "curvas",
    "formas",
    "superbg",
    "lat",
    "gr",
    "he",
    "translit",
    "interlineal",
    "orn",
    "vs",
    "ch",
    "pasaje",
    "theme",
    "palettes",
    "pairings",
    "capitular",
    "en-caja",
];

pub struct Rendered {
    pub typ: String,
    pub avisos: Vec<String>,
}

pub fn render(post: &Post, cfg: &Config) -> Result<Rendered> {
    if let Some(e) = &post.estilo_propio {
        return render_propio(post, cfg, e);
    }
    let mut avisos = Vec::new();
    let mut partes: Vec<String> = Vec::new();

    for v in &post.pos {
        partes.push(value_lit(v).context("argumento posicional")?);
    }

    // `formato` y `tema` del post, salvo que `args` ya los traiga.
    if let (Some((w, h)), false) = (post.formato, post.args.contains_key("formato")) {
        partes.push(format!("formato: ({w}, {h})"));
    }
    if let Some(tema) = resolver_tema(post, cfg, &mut avisos)? {
        let sin_tema = roles_sueltos(&post.preset);
        match (tema, sin_tema) {
            (Tema::Nombre(n), None) if !post.args.contains_key("tema") => {
                partes.push(format!("tema: {}", str_lit(&n)))
            }
            (Tema::Nombre(n), Some(_)) => bail!(
                "{} no acepta un tema de instatypst ({n:?}); usa una paleta de la CLI o `args`",
                post.preset
            ),
            (Tema::Roles(r), None) if !post.args.contains_key("tema") => {
                partes.push(format!("tema: {}", r.dict()?))
            }
            (Tema::Roles(r), Some(roles)) => {
                for (param, rol) in roles {
                    if !post.args.contains_key(*param) {
                        partes.push(format!("{param}: {}", r.rgb(*rol)?));
                    }
                }
            }
            _ => {} // `args` trae su propio `tema`
        }
    }

    let args = named_args(&post.args).context("args")?;
    if !args.is_empty() {
        partes.push(args);
    }

    let mut typ = preambulo(post)?;
    typ.push_str(&format!("#{}(\n", post.preset));
    for p in partes {
        typ.push_str(&format!("  {p},\n"));
    }
    typ.push_str(")\n");
    Ok(Rendered { typ, avisos })
}

/// Lo que va antes de la llamada: el comentario, el import de instatypst, lo que ven los
/// textos con formato (`md`) y las variables de `defs`.
fn preambulo(post: &Post) -> Result<String> {
    let mut typ = String::from("// Auto-generado por instatypst; no editar.\n");
    typ.push_str("#import \"@local/instatypst:0.1.0\": *\n\n");
    // Lo que ven los textos con formato (`md`): los espacios de nombres de instatypst
    // y, a medida que se declaran, las variables de `defs`.
    typ.push_str(&format!(
        "#let _alcance = ({})\n",
        ALCANCE
            .iter()
            .map(|n| format!("{n}: {n}"))
            .collect::<Vec<_>>()
            .join(", ")
    ));
    for (k, v) in &post.defs {
        typ.push_str(&format!(
            "#let {k} = {}\n",
            value_lit(v).with_context(|| format!("defs.{k}"))?
        ));
        typ.push_str(&format!("#let _alcance = _alcance + ({k}: {k})\n"));
    }
    typ.push('\n');
    Ok(typ)
}

/// Un estilo propio: se importa la función del autor y se le pasa la pieza ya resuelta,
/// como un diccionario. El formato y la paleta se resuelven aquí, igual que para un preset.
fn render_propio(post: &Post, cfg: &Config, e: &crate::config::EstiloPropio) -> Result<Rendered> {
    use toml::Value;
    let mut avisos = Vec::new();
    let mut pieza = e.pieza.clone();

    pieza.insert(
        "format".into(),
        match post.formato {
            Some((w, h)) => Value::Array(vec![Value::Integer(i64::from(w)), Value::Integer(i64::from(h))]),
            None => crate::pieza::ninguno(),
        },
    );
    pieza.insert("palette".into(), crate::pieza::ninguno());
    match resolver_tema(post, cfg, &mut avisos)? {
        None => {}
        Some(Tema::Roles(r)) => {
            let color = |h: &str| {
                let mut t = toml::Table::new();
                t.insert("color".into(), Value::String(h.to_string()));
                Value::Table(t)
            };
            let mut p = toml::Table::new();
            p.insert("background".into(), color(r.hex(Rol::Fondo)));
            p.insert("text".into(), color(r.hex(Rol::Tinta)));
            p.insert("muted".into(), color(r.hex(Rol::Suave)));
            p.insert("accent".into(), color(r.hex(Rol::Acento)));
            p.insert("line".into(), color(r.hex(Rol::Linea)));
            pieza.insert("palette".into(), Value::Table(p));
        }
        Some(Tema::Nombre(n)) => bail!(
            "un estilo propio necesita una paleta que la CLI resuelva (del documento, de la CLI o de una imagen); \
             {n:?} es un tema de Instatypst y solo el preset sabe leerlo"
        ),
    }

    let mut typ = preambulo(post)?;
    typ.push_str(&format!("#import {}: {}\n\n", str_lit(&crate::typst::asset_path(&e.archivo)?), e.funcion));
    typ.push_str(&format!("#{}({})\n", e.funcion, value_lit(&Value::Table(pieza)).context("pieza")?));
    Ok(Rendered { typ, avisos })
}

/// Paleta resuelta de un post.
enum Tema {
    /// Nombre de tema de instatypst (la CLI no lo conoce; lo valida Typst).
    Nombre(String),
    Roles(Roles),
}

#[derive(Clone, Copy)]
enum Rol {
    Fondo,
    Tinta,
    Suave,
    Acento,
    Linea,
}

struct Roles {
    fondo: String,
    tinta: String,
    suave: String,
    acento: String,
    linea: String,
}

impl Roles {
    fn hex(&self, r: Rol) -> &str {
        match r {
            Rol::Fondo => &self.fondo,
            Rol::Tinta => &self.tinta,
            Rol::Suave => &self.suave,
            Rol::Acento => &self.acento,
            Rol::Linea => &self.linea,
        }
    }

    fn rgb(&self, r: Rol) -> Result<String> {
        let h = self.hex(r);
        colors::hex_to_rgb_tuple(h)?; // valida antes de emitir
        Ok(format!("rgb({})", str_lit(h)))
    }

    /// `(fondo: rgb(..), tinta: .., suave: .., acento: .., linea: ..)`, el `tema:` de instatypst.
    fn dict(&self) -> Result<String> {
        let mut v = Vec::new();
        for (k, r) in [
            ("fondo", Rol::Fondo),
            ("tinta", Rol::Tinta),
            ("suave", Rol::Suave),
            ("acento", Rol::Acento),
            ("linea", Rol::Linea),
        ] {
            v.push(format!("{k}: {}", self.rgb(r)?));
        }
        Ok(format!("({})", v.join(", ")))
    }
}

/// Presets de instatypst que no tienen `tema:` y reciben los roles como
/// parámetros sueltos (firmas en src/carteles.typ y src/plano.typ).
fn roles_sueltos(preset: &str) -> Option<&'static [(&'static str, Rol)]> {
    match preset.strip_prefix("carteles.")? {
        "liso" => Some(&[("fondo", Rol::Fondo), ("tinta", Rol::Tinta)]),
        "plano" => Some(&[
            ("fondo", Rol::Fondo),
            ("linea", Rol::Linea),
            ("acento", Rol::Acento),
        ]),
        _ => None,
    }
}

fn resolver_tema(post: &Post, cfg: &Config, avisos: &mut Vec<String>) -> Result<Option<Tema>> {
    let Some(nombre) = post.tema.as_deref() else {
        return Ok(None);
    };

    let paleta: HashMap<String, String> = match nombre {
        "auto" | "auto-matugen" => {
            let img = post
                .image
                .as_deref()
                .ok_or_else(|| anyhow!("tema {nombre:?} necesita `image`"))?;
            if nombre == "auto" {
                let c = colors::extract_from_image(img, 8)
                    .with_context(|| format!("extrayendo colores de {img:?}"))?;
                let base = c
                    .first()
                    .ok_or_else(|| anyhow!("la imagen no tiene colores"))?;
                colors::generate_palette(&format!("#{:02x}{:02x}{:02x}", base.r, base.g, base.b))
            } else {
                colors::matugen_palette(img)?
            }
        }
        otro => match themes::predefined::get_theme_palettes(Some(cfg)).remove(otro) {
            Some(p) => p,
            // No es una paleta de esta CLI: se pasa como nombre de tema de instatypst
            // y, si no existe, Typst falla listando los válidos.
            None => return Ok(Some(Tema::Nombre(otro.to_string()))),
        },
    };

    let get = |k: &str| {
        paleta
            .get(k)
            .cloned()
            .ok_or_else(|| anyhow!("a la paleta {nombre:?} le falta `{k}`"))
    };
    let roles = Roles {
        fondo: get("bg")?,
        tinta: get("text")?,
        suave: get("muted")?,
        acento: get("primary")?,
        linea: get("muted")?,
    };

    for (msg, a, b, min) in [
        (
            "texto sobre fondo",
            &roles.tinta,
            &roles.fondo,
            CONTRASTE_TEXTO,
        ),
        (
            "acento sobre fondo",
            &roles.acento,
            &roles.fondo,
            CONTRASTE_ACENTO,
        ),
    ] {
        let r = colors::contrast_ratio(a, b)?;
        if r < min {
            avisos.push(format!("contraste bajo ({msg}): {r:.1}:1, mínimo {min}:1"));
        }
    }
    Ok(Some(Tema::Roles(roles)))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn render_toml(t: &str) -> Rendered {
        let cfg = Config::parse(t).unwrap();
        let post = cfg.resolve_all().unwrap().remove(0);
        render(&post, &cfg).unwrap()
    }

    #[test]
    fn llamada_completa() {
        let r = render_toml(
            r#"
            [[post]]
            name = "a"
            preset = "suizo"
            tema = "dark"
            pos = ["Ruta C:\\", { md = "*x*" }]
            [post.args]
            marca = "@yo"
            "#,
        );
        assert!(r.typ.contains("#import \"@local/instatypst:0.1.0\": *"));
        assert!(r.typ.contains("#carteles.suizo(\n"));
        assert!(r.typ.contains(r#"  "Ruta C:\\","#));
        assert!(r
            .typ
            .contains(r#"  eval("*x*", mode: "markup", scope: _alcance),"#));
        assert!(
            !r.typ.contains("formato:"),
            "sin formato manda el preset: {}",
            r.typ
        );
        assert!(r.typ.contains("tema: (fondo: rgb(\"#0f0f0f\")"));
        assert!(r.typ.contains(r#"marca: "@yo""#));
    }

    #[test]
    fn formato_explicito_se_emite() {
        let r = render_toml("[[post]]\nname='a'\npreset='folio'\nformato='story'");
        assert!(r.typ.contains("  formato: (1080, 1920),"), "{}", r.typ);
    }

    #[test]
    fn tema_desconocido_pasa_como_nombre() {
        let r = render_toml("[[post]]\nname='a'\npreset='bauhaus'\ntema='suizo'");
        assert!(r.typ.contains("tema: \"suizo\","), "{}", r.typ);
    }

    #[test]
    fn args_pisan_formato_y_tema() {
        let r = render_toml(
            "[[post]]\nname='a'\npreset='liso'\nformato='story'\ntema='dark'\n[post.args]\nformato=[10,20]\ntema='x'",
        );
        assert!(!r.typ.contains("1920"));
        assert_eq!(r.typ.matches("formato:").count(), 1);
        assert_eq!(r.typ.matches("tema:").count(), 1);
    }

    #[test]
    fn avisa_contraste_bajo() {
        let r = render_toml(
            "[themes.gris]\nbg='#777777'\ntext='#7a7a7a'\nmuted='#888888'\nprimary='#787878'\n\
             [[post]]\nname='a'\npreset='liso'\ntema='gris'",
        );
        assert_eq!(r.avisos.len(), 2, "{:?}", r.avisos);
    }

    #[test]
    fn paleta_incompleta_falla() {
        let cfg =
            Config::parse("[themes.x]\nbg='#000000'\n[[post]]\nname='a'\npreset='liso'\ntema='x'")
                .unwrap();
        let post = cfg.resolve_all().unwrap().remove(0);
        assert!(render(&post, &cfg).is_err());
    }

    #[test]
    fn liso_y_plano_reciben_roles_sueltos() {
        let r = render_toml("[[post]]\nname='a'\npreset='liso'\ntema='dark'");
        assert!(
            r.typ.contains("fondo: rgb(\"#0f0f0f\")") && r.typ.contains("tinta: rgb(\"#f5f5f5\")")
        );
        assert!(!r.typ.contains("tema:"));
        let r = render_toml("[[post]]\nname='a'\npreset='plano'\ntema='dark'");
        assert!(r.typ.contains("linea:") && r.typ.contains("acento:") && !r.typ.contains("tinta:"));
    }

    #[test]
    fn args_ganan_sobre_el_tema_en_liso() {
        let r = render_toml(
            "[[post]]\nname='a'\npreset='liso'\ntema='dark'\n[post.args]\nfondo={ color='#691322' }",
        );
        assert_eq!(r.typ.matches("fondo:").count(), 1);
        assert!(r.typ.contains("#691322") && r.typ.contains("tinta:"));
    }

    #[test]
    fn liso_rechaza_tema_de_instatypst() {
        let cfg = Config::parse("[[post]]\nname='a'\npreset='liso'\ntema='suizo'").unwrap();
        let post = cfg.resolve_all().unwrap().remove(0);
        assert!(render(&post, &cfg).is_err());
    }

    #[test]
    fn defs_se_declaran_y_entran_al_alcance() {
        let r = render_toml(
            r##"
            [defaults.defs]
            ir = { md = "#carteles.rayar[ir]" }
            borde = { typst = '3pt + rgb("#fff")' }
            [[post]]
            name = "a"
            preset = "liso"
            pos = [{ kw = "ir" }]
            "##,
        );
        let t = &r.typ;
        assert!(t.contains("#let _alcance = (carteles: carteles"));
        let i = t.find("#let ir = ").expect("declara ir");
        let j = t
            .find("#let _alcance = _alcance + (ir: ir)")
            .expect("amplía el alcance");
        let k = t.find("#let borde = (3pt + rgb").expect("declara borde");
        assert!(i < j && j < k, "{t}"); // en el orden escrito
        assert!(t.contains("  ir,"), "{t}");
    }

    #[test]
    fn defs_con_nombre_invalido_fallan() {
        for n in ["_x", "a b", "1a"] {
            let cfg = Config::parse(&format!(
                "[defaults.defs]\n\"{n}\" = 1\n[[post]]\nname='a'\npreset='liso'"
            ))
            .unwrap();
            assert!(cfg.resolve_all().is_err(), "{n}");
        }
    }
}
