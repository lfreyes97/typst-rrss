//! Roles a partir de un solo color (el de una marca): `palette.base` del contrato.
//!
//! El color base es el **acento**. El fondo y el texto son neutros con un leve tinte de su tono, y
//! el tema es oscuro si el base es oscuro o medio (L < 0,6 en Oklab) y claro si no. Si el base no
//! contrasta lo bastante con el fondo, se le mueve la luminosidad (conservando tono y chroma)
//! hasta que llegue a [`MIN_ACCENT`]; si aun así no llega, se avisa.

use crate::contrast::{ratio, MIN_ACCENT};
use crate::roles::Selection;
use crate::{Rgb, Roles, Warning};
use palette::{FromColor, Mix, Oklab, Oklch};

const BASE_OSCURA: f32 = 0.6;

fn de_oklch(l: f32, c: f32, h: palette::OklabHue<f32>) -> Rgb {
    // `from_oklab` satura al gamut sRGB al convertir a u8
    Rgb::from_oklab(Oklab::from_color(Oklch::new(l, c, h)))
}

/// El mismo color con el tono girado `grados` en Oklch (luminosidad y chroma intactos).
pub fn rotate_hue(c: Rgb, grados: f32) -> Rgb {
    let mut lch = Oklch::from_color(c.oklab());
    lch.hue = palette::OklabHue::from_degrees(lch.hue.into_degrees() + grados);
    Rgb::from_oklab(Oklab::from_color(lch))
}

/// Los roles de una pieza cuyo color de marca es `base`.
pub fn from_base(base: Rgb) -> Selection {
    let lch = Oklch::from_color(base.oklab());
    let oscuro = lch.l < BASE_OSCURA;
    let tinte = (lch.chroma * 0.25).min(0.04);

    let (fondo, texto) = if oscuro {
        (
            de_oklch(0.18, tinte, lch.hue),
            de_oklch(0.95, tinte / 4.0, lch.hue),
        )
    } else {
        (
            de_oklch(0.965, tinte / 2.0, lch.hue),
            de_oklch(0.22, tinte, lch.hue),
        )
    };

    // acento: el base, empujado en luminosidad lejos del fondo hasta que contraste
    let mut warnings = Vec::new();
    let paso = if oscuro { 0.02 } else { -0.02 };
    let mut acento = base;
    let mut luz = lch.l;
    while ratio(acento, fondo) < MIN_ACCENT && (0.0..=1.0).contains(&(luz + paso)) {
        luz += paso;
        acento = de_oklch(luz, lch.chroma, lch.hue);
    }
    if ratio(acento, fondo) < MIN_ACCENT {
        warnings.push(Warning {
            code: "contrast_low",
            message: format!(
                "el color base {} no llega a contraste {MIN_ACCENT} con su fondo",
                base.hex()
            ),
        });
    }

    let apagado = Rgb::from_oklab(fondo.oklab().mix(texto.oklab(), 0.55));
    Selection {
        roles: Roles {
            background: fondo,
            text: texto,
            muted: apagado,
            accent: acento,
            line: apagado,
        },
        warnings,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::contrast::MIN_TEXT;

    fn rgb(h: &str) -> Rgb {
        Rgb::parse(h).unwrap()
    }

    #[test]
    fn un_base_oscuro_da_tema_oscuro() {
        let s = from_base(rgb("#4a3f6b"));
        assert!(s.roles.background.oklab().l < 0.3);
        assert!(s.roles.text.oklab().l > 0.8);
    }

    #[test]
    fn un_base_claro_da_tema_claro() {
        let s = from_base(rgb("#f2c14e"));
        assert!(s.roles.background.oklab().l > 0.9);
        assert!(s.roles.text.oklab().l < 0.4);
    }

    #[test]
    fn texto_y_acento_son_legibles_sobre_el_fondo() {
        for h in [
            "#4a3f6b", "#f2c14e", "#691322", "#2e7d32", "#ffffff", "#000000",
        ] {
            let s = from_base(rgb(h));
            let r = &s.roles;
            assert!(ratio(r.text, r.background) >= MIN_TEXT, "{h} texto");
            assert!(
                ratio(r.accent, r.background) >= MIN_ACCENT || !s.warnings.is_empty(),
                "{h} acento"
            );
        }
    }

    #[test]
    fn el_acento_conserva_el_tono_del_base() {
        // un granate oscuro: se aclara para contrastar, pero sigue siendo granate
        let base = rgb("#691322");
        let a = from_base(base).roles.accent;
        let (hb, ha) = (
            Oklch::from_color(base.oklab()).hue.into_degrees(),
            Oklch::from_color(a.oklab()).hue.into_degrees(),
        );
        assert!((hb - ha).abs() < 8.0, "{hb} vs {ha}");
        assert!(a.oklab().l > base.oklab().l);
    }

    #[test]
    fn un_base_que_ya_contrasta_no_se_toca() {
        let base = rgb("#3f6fd0");
        assert_eq!(from_base(base).roles.accent, base);
    }

    #[test]
    fn girar_el_tono_conserva_la_luminosidad() {
        let c = rgb("#3f6fd0");
        let g = rotate_hue(c, 180.0);
        assert!((c.oklab().l - g.oklab().l).abs() < 0.03);
        assert_ne!(c, g);
        assert_eq!(rotate_hue(c, 360.0).hex(), c.hex());
    }

    #[test]
    fn es_determinista() {
        assert_eq!(from_base(rgb("#2e7d32")), from_base(rgb("#2e7d32")));
    }
}
