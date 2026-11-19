//! Roles Material You (Material Color Utilities) a partir de un color de origen o de una imagen.
//!
//! El origen de una imagen lo elige el algoritmo de Material (cuantizador de Celebi + puntuación),
//! que prefiere colores vivos y presentes, distinto del selector de [`crate::roles`], que busca
//! fondo/texto/acento *de la foto*. Aquí la foto solo da el color semilla; todo lo demás es el
//! esquema de Material, claro u oscuro.

use crate::roles::Selection;
use crate::{contrast, Rgb, Roles};
use material_colors::color::Rgb as MRgb;
use material_colors::theme::ThemeBuilder;

fn de_m(c: MRgb) -> Rgb {
    Rgb([c.red, c.green, c.blue])
}

/// Los roles del esquema de Material para un color de origen. `dark` elige el esquema oscuro.
pub fn from_source(source: Rgb, dark: bool) -> Selection {
    let theme = ThemeBuilder::with_source(MRgb::new(source.0[0], source.0[1], source.0[2])).build();
    let s = if dark {
        &theme.schemes.dark
    } else {
        &theme.schemes.light
    };
    let roles = Roles {
        background: de_m(s.background),
        text: de_m(s.on_background),
        muted: de_m(s.outline),
        accent: de_m(s.primary),
        line: de_m(s.outline_variant),
    };
    let warnings = contrast::check(&roles);
    Selection { roles, warnings }
}

/// El color semilla de una imagen según Material: la imagen se reduce a 128×128 y se puntúa.
#[cfg(feature = "image")]
pub fn source_of(img: &image::RgbImage) -> Rgb {
    let small = image::imageops::thumbnail(img, 128, 128);
    let pixels: Vec<MRgb> = small
        .pixels()
        .map(|p| MRgb::new(p.0[0], p.0[1], p.0[2]))
        .collect();
    de_m(material_colors::image::extract_color(&pixels))
}

/// Los roles Material de un archivo de imagen.
#[cfg(feature = "image")]
pub fn from_image(
    path: &std::path::Path,
    dark: bool,
) -> Result<Selection, crate::extract::ExtractError> {
    let img = image::open(path)?.to_rgb8();
    Ok(from_source(source_of(&img), dark))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rgb(h: &str) -> Rgb {
        Rgb::parse(h).unwrap()
    }

    #[test]
    fn el_esquema_oscuro_es_oscuro_y_el_claro_es_claro() {
        let o = from_source(rgb("#4285f4"), true).roles;
        let c = from_source(rgb("#4285f4"), false).roles;
        assert!(o.background.oklab().l < 0.3 && o.text.oklab().l > 0.8);
        assert!(c.background.oklab().l > 0.9 && c.text.oklab().l < 0.4);
    }

    #[test]
    fn el_acento_guarda_el_tono_del_origen() {
        let hue = |c: Rgb| palette::Oklch::from_color(c.oklab()).hue.into_degrees();
        use palette::FromColor;
        let a = from_source(rgb("#2e7d32"), true).roles.accent;
        let d = (hue(a) - hue(rgb("#2e7d32"))).abs();
        assert!(d.min(360.0 - d) < 25.0, "{d}");
    }

    #[test]
    fn el_texto_es_legible_sobre_el_fondo() {
        for h in ["#4285f4", "#691322", "#f2c14e", "#2e7d32"] {
            for dark in [true, false] {
                let r = from_source(rgb(h), dark).roles;
                assert!(contrast::ratio(r.text, r.background) >= 4.5, "{h} {dark}");
            }
        }
    }

    #[test]
    fn es_determinista() {
        assert_eq!(
            from_source(rgb("#691322"), true),
            from_source(rgb("#691322"), true)
        );
    }

    #[cfg(feature = "image")]
    #[test]
    fn la_semilla_de_una_imagen_es_su_color_vivo() {
        // 80 % gris, 20 % rojo vivo: Material descarta el gris y se queda con el rojo
        let img = image::RgbImage::from_fn(50, 50, |x, _| {
            if x < 40 {
                image::Rgb([128, 128, 128])
            } else {
                image::Rgb([200, 30, 30])
            }
        });
        let s = source_of(&img);
        assert!(s.0[0] > 150 && s.0[1] < 80 && s.0[2] < 80, "{s:?}");
    }
}
