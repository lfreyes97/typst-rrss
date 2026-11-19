//! `init`: un punto de partida editable, como `posts.toml` o como `.typ`.
//!
//! La plantilla se escribe siempre como TOML y pasa por `Config::parse` +
//! `resolve_all` antes de devolverse; el `.typ` sale de ese mismo TOML por
//! `render`. Ninguna plantilla llega al disco sin pasar la validación de `build`.
//!
//! Solo se ofrecen los presets de `POSICIONALES`: de los demás esta CLI no
//! conoce la firma y la plantilla no compilaría (`missing argument`).

use anyhow::{bail, Context, Result};

use crate::config::Config;
use crate::render;

/// Argumentos posicionales de los presets que `init` sabe escribir. Con solo
/// estos argumentos, cada preset compila tal cual.
const POSICIONALES: &[(&str, &[&str])] = &[
    ("liso", &["texto"]),
    ("suizo", &["frase", "remate"]),
    ("bauhaus", &["frase", "remate"]),
    ("epigrafe", &["cita"]),
    ("esquela", &["palabra"]),
    ("franjas", &["texto"]),
];

/// Nombre corto (`suizo`) y posicionales de un preset; error si `init` no lo conoce.
fn posicionales(preset: &str) -> Result<(&'static str, &'static [&'static str])> {
    let corto = preset.strip_prefix("carteles.").unwrap_or(preset);
    match POSICIONALES.iter().find(|(n, _)| *n == corto) {
        Some(&(n, p)) => Ok((n, p)),
        None => {
            let conocidos: Vec<_> = POSICIONALES.iter().map(|(n, _)| *n).collect();
            bail!(
                "init no conoce la firma de {preset:?}; sabe escribir: {}.\n\
                 Para otro preset de instatypst, parte de uno de estos y cambia `preset`, \
                 `pos` y `[post.args]` a mano (hay ejemplos en examples/presets.toml)",
                conocidos.join(", ")
            )
        }
    }
}

/// Un `posts.toml` con un `[[post]]` por preset, listo para `build`.
pub fn toml(presets: &[String]) -> Result<String> {
    let mut out = String::from(
        "# Punto de partida de rrss-cli-rs. Cambia los textos ‹…› y compila con:\n\
         #   rrss-cli-rs build <este archivo>\n\n\
         [defaults]\n\
         # fonts = [\"/ruta/a/Instatypst/Fonts\"]   # si no están en el sistema\n",
    );
    let mut vistos = Vec::new();
    for preset in presets {
        let (nombre, pos) = posicionales(preset)?;
        if vistos.contains(&nombre) {
            bail!("el preset {nombre:?} está repetido: cada preset sale una vez");
        }
        vistos.push(nombre);
        let pos = pos
            .iter()
            .map(|p| format!("\"‹tu {p}›\""))
            .collect::<Vec<_>>()
            .join(", ");
        out.push_str(&format!(
            "\n[[post]]\n\
             name = {nombre:?}\n\
             preset = {nombre:?}\n\
             # formato = \"feed-4x5\"   # sin él, manda el del preset\n\
             # tema = \"auto\"          # con `image`, la paleta sale de la foto\n\
             pos = [{pos}]\n\
             [post.args]\n\
             # autor = \"‹autor›\"\n"
        ));
    }
    Config::parse(&out)?
        .resolve_all()
        .context("la plantilla no pasa la validación de build")?;
    Ok(out)
}

/// El `.typ` de un solo preset, con los mismos marcadores que `toml`.
pub fn typ(presets: &[String]) -> Result<String> {
    let [_] = presets else {
        bail!("un .typ describe una sola pieza: pasa un preset (o escribe un posts.toml)");
    };
    let cfg = Config::parse(&toml(presets)?)?;
    let post = cfg.resolve(&cfg.posts[0])?;
    Ok(render::render(&post, &cfg)?.typ)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn presets() -> Vec<String> {
        POSICIONALES.iter().map(|(n, _)| n.to_string()).collect()
    }

    #[test]
    fn el_toml_pasa_la_validacion_de_build() {
        let cfg = Config::parse(&toml(&presets()).unwrap()).unwrap();
        assert_eq!(cfg.resolve_all().unwrap().len(), presets().len());
    }

    #[test]
    fn los_marcadores_no_parecen_contenido() {
        let t = toml(&["suizo".to_string()]).unwrap();
        assert!(t.contains(r#"pos = ["‹tu frase›", "‹tu remate›"]"#), "{t}");
    }

    #[test]
    fn rechaza_presets_desconocidos_invalidos_y_repetidos() {
        let mal = |ps: &[&str]| {
            let ps: Vec<String> = ps.iter().map(|p| p.to_string()).collect();
            format!("{:#}", toml(&ps).unwrap_err())
        };
        assert!(mal(&["noexiste"]).contains("firma"));
        assert!(mal(&["mosaico"]).contains("firma"));
        assert!(mal(&["fleurones.orla"]).contains("firma"));
        assert!(mal(&["li so"]).contains("firma"));
        assert!(mal(&["suizo", "suizo"]).contains("repetido"));
        assert!(mal(&["liso", "carteles.liso"]).contains("repetido"));
    }

    #[test]
    fn el_typ_llama_al_preset() {
        let typ = typ(&["carteles.suizo".to_string()]).unwrap();
        assert!(typ.contains("#carteles.suizo("));
        assert!(typ.contains("\"‹tu frase›\""));
    }

    #[test]
    fn un_typ_rechaza_varios_presets() {
        assert!(typ(&["suizo".to_string(), "liso".to_string()]).is_err());
    }
}
