//! Contraste WCAG entre dos colores y los avisos que de él se derivan.

use crate::{Rgb, Roles, Warning};

/// Mínimo WCAG AA para texto normal.
pub const MIN_TEXT: f64 = 4.5;
/// Mínimo WCAG para elementos gráficos (el acento).
pub const MIN_ACCENT: f64 = 3.0;

/// Luminancia relativa WCAG (0.0 – 1.0).
pub fn luminance(c: Rgb) -> f64 {
    let lin = |v: u8| {
        let v = v as f64 / 255.0;
        if v <= 0.04045 {
            v / 12.92
        } else {
            ((v + 0.055) / 1.055).powf(2.4)
        }
    };
    let [r, g, b] = c.0;
    0.2126 * lin(r) + 0.7152 * lin(g) + 0.0722 * lin(b)
}

/// Proporción de contraste WCAG (1.0 – 21.0). Es simétrica.
pub fn ratio(a: Rgb, b: Rgb) -> f64 {
    let (la, lb) = (luminance(a), luminance(b));
    (la.max(lb) + 0.05) / (la.min(lb) + 0.05)
}

/// Los avisos de contraste de unos roles: el texto contra el fondo (4,5:1) y el acento
/// contra el fondo (3:1). Mismas reglas y mismo texto que el `build` de siempre.
pub fn check(r: &Roles) -> Vec<Warning> {
    let mut avisos = Vec::new();
    for (que, a, minimo) in [
        ("texto sobre fondo", r.text, MIN_TEXT),
        ("acento sobre fondo", r.accent, MIN_ACCENT),
    ] {
        let c = ratio(a, r.background);
        if c < minimo {
            avisos.push(Warning {
                code: "contrast_low",
                message: format!("contraste bajo ({que}): {c:.1}:1, mínimo {minimo}:1"),
            });
        }
    }
    avisos
}

#[cfg(test)]
mod tests {
    use super::*;

    const NEGRO: Rgb = Rgb([0, 0, 0]);
    const BLANCO: Rgb = Rgb([255, 255, 255]);

    #[test]
    fn extremos_y_simetria() {
        assert!((ratio(NEGRO, BLANCO) - 21.0).abs() < 0.01);
        let gris = Rgb([0x77, 0x77, 0x77]);
        assert!((ratio(gris, gris) - 1.0).abs() < 1e-9);
        let (a, b) = (Rgb([0x69, 0x13, 0x22]), Rgb([0xf5, 0xf5, 0xf5]));
        assert_eq!(ratio(a, b), ratio(b, a));
    }

    fn roles(fondo: Rgb, texto: Rgb, acento: Rgb) -> Roles {
        Roles {
            background: fondo,
            text: texto,
            muted: texto,
            accent: acento,
            line: texto,
        }
    }

    #[test]
    fn avisa_con_el_mismo_texto_que_el_build() {
        let r = roles(Rgb([0x77; 3]), Rgb([0x7a; 3]), Rgb([0x78; 3]));
        let a = check(&r);
        assert_eq!(a.len(), 2);
        assert!(a.iter().all(|w| w.code == "contrast_low"));
        assert!(
            a[0].message
                .starts_with("contraste bajo (texto sobre fondo): 1.0:1, mínimo 4.5:1"),
            "{}",
            a[0].message
        );
        assert!(a[1].message.ends_with("mínimo 3:1"), "{}", a[1].message);
    }

    #[test]
    fn sin_avisos_si_hay_contraste() {
        assert!(check(&roles(NEGRO, BLANCO, Rgb([0xe9, 0x45, 0x60]))).is_empty());
    }
}
