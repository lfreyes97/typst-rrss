// use image::GenericImageView;
use palette::{FromColor, Hsl, Srgb};
use std::path::Path;
use anyhow::Result;

pub fn hex_to_rgb_tuple(hex_color: &str) -> (u8, u8, u8) {
    let hex_color = hex_color.trim_start_matches('#');
    let r = u8::from_str_radix(&hex_color[0..2], 16).unwrap_or(0);
    let g = u8::from_str_radix(&hex_color[2..4], 16).unwrap_or(0);
    let b = u8::from_str_radix(&hex_color[4..6], 16).unwrap_or(0);
    (r, g, b)
}

pub fn hex_to_hsl(hex_color: &str) -> (f32, f32, f32) {
    let (r, g, b) = hex_to_rgb_tuple(hex_color);
    let srgb = Srgb::new(r as f32 / 255.0, g as f32 / 255.0, b as f32 / 255.0);
    let hsl = Hsl::from_color(srgb);
    (hsl.hue.into_degrees(), hsl.saturation, hsl.lightness)
}

pub fn hsl_to_hex(h: f32, s: f32, l: f32) -> String {
    let hsl = Hsl::new(h, s, l);
    let srgb = Srgb::from_color(hsl);
    let r = (srgb.red * 255.0).round() as u8;
    let g = (srgb.green * 255.0).round() as u8;
    let b = (srgb.blue * 255.0).round() as u8;
    format!("#{:02x}{:02x}{:02x}", r, g, b)
}

fn clamp(val: f32, min: f32, max: f32) -> f32 {
    let v = if val < min { min } else { val };
    if v > max { max } else { v }
}

pub fn generate_palette(base_hex: &str) -> std::collections::HashMap<String, String> {
    let (h, s, l) = hex_to_hsl(base_hex);
    
    let mut palette = std::collections::HashMap::new();
    
    palette.insert("bg".to_string(), hsl_to_hex(h, clamp(s * 0.3, 0.0, 1.0), clamp(l * 0.12, 0.0, 0.1)));
    palette.insert("surface".to_string(), hsl_to_hex(h, clamp(s * 0.35, 0.0, 1.0), clamp(l * 0.18, 0.0, 0.15)));
    palette.insert("primary".to_string(), hsl_to_hex(h, s, l));
    palette.insert("secondary".to_string(), hsl_to_hex((h + 30.0) % 360.0, clamp(s * 0.7, 0.0, 1.0), clamp(l * 0.55, 0.0, 1.0)));
    palette.insert("accent".to_string(), hsl_to_hex((h + 180.0) % 360.0, clamp(s * 0.5, 0.0, 1.0), clamp(l * 0.4, 0.0, 1.0)));
    palette.insert("text".to_string(), hsl_to_hex(h, clamp(s * 0.1, 0.0, 1.0), clamp(0.95, 0.0, 1.0)));
    palette.insert("muted".to_string(), hsl_to_hex(h, clamp(s * 0.15, 0.0, 1.0), clamp(0.6, 0.0, 1.0)));
    palette.insert("highlight".to_string(), hsl_to_hex((h + 60.0) % 360.0, clamp(s * 0.8, 0.0, 1.0), clamp(l * 0.7, 0.0, 1.0)));

    palette
}

pub fn extract_from_image(image_path: &str, count: u8) -> Result<Vec<color_thief::Color>> {
    let img = image::open(Path::new(image_path))?;
    let img = img.to_rgba8();
    let pixels = img.as_raw();
    let palette = color_thief::get_palette(pixels, color_thief::ColorFormat::Rgba, 10, count)?;
    Ok(palette)
}

pub fn suggest_accent(image_path: &str) -> String {
    let colors = match extract_from_image(image_path, 12) {
        Ok(c) => c,
        Err(_) => return "#4a3f6b".to_string(),
    };
    
    if colors.is_empty() {
        return "#4a3f6b".to_string();
    }

    // Puntuar cada color: preferir saturación alta + luminosidad media
    let mut scored: Vec<(f32, String)> = Vec::new();

    for c in &colors {
        let hex = format!("#{:02x}{:02x}{:02x}", c.r, c.g, c.b);
        let (_, s, l) = hex_to_hsl(&hex);
        
        // Evitar colores muy oscuros/claros o muy desaturados
        if l < 0.08 || l > 0.92 || s < 0.05 {
            continue;
        }

        // Score: saturación alta + luminosidad entre 0.2-0.6
        let lum_score = 1.0 - (l - 0.4).abs() * 2.0; // máximo en l=0.4
        let score = s * 0.6 + lum_score * 0.3; // + proportion? color_thief no devuelve proporción fácilmente aquí
        scored.push((score, hex));
    }

    if scored.is_empty() {
        let c = &colors[0];
         return format!("#{:02x}{:02x}{:02x}", c.r, c.g, c.b);
    }

    scored.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap());
    scored[0].1.clone()
}
