use anyhow::{anyhow, Result};
use instatypst_color::{base, roles, Rgb, Roles};
use std::collections::HashMap;
use std::path::Path;

pub use instatypst_piece::util::hex_to_rgb_tuple;

/// Luminancia relativa WCAG de un `#RRGGBB`.
fn luminancia(hex: &str) -> Result<f64> {
    let (r, g, b) = hex_to_rgb_tuple(hex)?;
    let lin = |c: u8| {
        let c = c as f64 / 255.0;
        if c <= 0.04045 {
            c / 12.92
        } else {
            ((c + 0.055) / 1.055).powf(2.4)
        }
    };
    Ok(0.2126 * lin(r) + 0.7152 * lin(g) + 0.0722 * lin(b))
}

/// Proporción de contraste WCAG entre dos colores (1.0 – 21.0).
pub fn contrast_ratio(a: &str, b: &str) -> Result<f64> {
    let (la, lb) = (luminancia(a)?, luminancia(b)?);
    Ok((la.max(lb) + 0.05) / (la.min(lb) + 0.05))
}

fn rgb(h: &str) -> Result<Rgb> {
    Rgb::parse(h).map_err(|e| anyhow!("color no válido: {h:?} ({e})"))
}

/// Los roles como el mapa de siempre de la CLI (`bg`, `text`, `primary`…). Los acentos de más
/// salen de girar el tono del principal: `highlight` +60°, `secondary` +30° y `accent` +180°.
pub fn paleta_de_roles(r: &Roles) -> HashMap<String, String> {
    let mezcla = |t: f32| {
        let (a, b) = (r.background.0, r.text.0);
        Rgb([0, 1, 2].map(|i| (a[i] as f32 * (1.0 - t) + b[i] as f32 * t).round() as u8))
    };
    let mut m = HashMap::new();
    for (k, c) in [
        ("bg", r.background),
        ("surface", mezcla(0.08)),
        ("muted", r.muted),
        ("text", r.text),
        ("primary", r.accent),
        ("highlight", base::rotate_hue(r.accent, 60.0)),
        ("secondary", base::rotate_hue(r.accent, 30.0)),
        ("accent", base::rotate_hue(r.accent, 180.0)),
    ] {
        m.insert(k.to_string(), c.hex());
    }
    m
}

/// La paleta de un solo color de marca (`palette.base`, `colors`).
pub fn generate_palette(base_hex: &str) -> Result<HashMap<String, String>> {
    Ok(paleta_de_roles(&base::from_base(rgb(base_hex)?).roles))
}

/// Los colores dominantes de una imagen, del más al menos abundante: `(#rrggbb, fracción)`.
pub fn extract_from_image(image_path: &str, count: u8) -> Result<Vec<(String, f32)>> {
    let c = instatypst_color::extract::candidates_from_path(Path::new(image_path), count)
        .map_err(|e| anyhow!("{e}"))?;
    Ok(c.into_iter().map(|c| (c.rgb.hex(), c.share)).collect())
}

/// Los roles de una foto, con los avisos del selector (`no_accent`, `contrast_low`).
pub fn roles_de_imagen(image_path: &str) -> Result<roles::Selection> {
    roles::from_image(Path::new(image_path)).map_err(|e| anyhow!("{e}"))
}

pub fn suggest_accent(image_path: &str) -> Result<String> {
    Ok(roles_de_imagen(image_path)?.roles.accent.hex())
}

/// Paleta Material You (esquema oscuro) a partir de una imagen. Falla si no se puede leer.
pub fn matugen_palette(path: &str) -> Result<HashMap<String, String>> {
    let sel = instatypst_color::material::from_image(Path::new(path), true)
        .map_err(|e| anyhow!("no se pudo leer {path:?}: {e}"))?;
    Ok(paleta_de_roles(&sel.roles))
}

#[cfg(test)]
mod contraste_tests {
    use super::contrast_ratio;

    #[test]
    fn extremos_y_simetria() {
        assert!((contrast_ratio("#000000", "#ffffff").unwrap() - 21.0).abs() < 0.01);
        assert!((contrast_ratio("#777777", "#777777").unwrap() - 1.0).abs() < 1e-9);
        let a = contrast_ratio("#691322", "#f5f5f5").unwrap();
        assert_eq!(a, contrast_ratio("#f5f5f5", "#691322").unwrap());
        assert!(contrast_ratio("rojo", "#fff").is_err());
    }
}

#[cfg(test)]
mod paleta_tests {
    use super::*;

    #[test]
    fn la_paleta_de_un_color_tiene_las_ocho_claves_y_es_legible() {
        let p = generate_palette("#691322").unwrap();
        for k in [
            "bg",
            "surface",
            "muted",
            "text",
            "primary",
            "highlight",
            "secondary",
            "accent",
        ] {
            assert!(p.contains_key(k), "falta {k}");
        }
        assert!(contrast_ratio(&p["text"], &p["bg"]).unwrap() >= 4.5);
        assert!(contrast_ratio(&p["primary"], &p["bg"]).unwrap() >= 3.0);
    }

    #[test]
    fn un_color_invalido_es_un_error() {
        assert!(generate_palette("rojo").is_err());
    }
}
