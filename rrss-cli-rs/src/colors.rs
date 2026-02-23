// use image::GenericImageView;
use anyhow::Result;
use palette::{FromColor, Hsl, OklabHue, Srgb};
use quantette::ImageRef;
use std::path::Path;
use thaimeleon_lib::scheme_builder::{ChromaBuilder, SchemeBuilder, ThemeConfig};

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
    if v > max {
        max
    } else {
        v
    }
}

fn get_default_builder() -> SchemeBuilder {
    SchemeBuilder {
        k_means_count: 255,
        light_theme_threshold: 0.55,
        run_in_parallel: false,
        light: ThemeConfig {
            base_lightness_minimum: 0.964,
            base_lightness_maximum: 0.964,
            surface_distance: 0.0225,
            set_2_dps_contrast: 7.5,
            set_2_lightness_correction: 0.01,
            set_3_dps_contrast: 40.0,
            set_4_dps_contrast: 65.0,
            set_5_dps_contrast: 80.0,
            neutral_chroma_blend: 1.0,
            bg_neutral_chroma_builder: ChromaBuilder {
                chroma_intercept: 0.67,
                lightness_to_chroma_slope: -0.667,
                low_point_chroma_intercept: 0.644,
                hue_low_point: OklabHue::from_degrees(230.0),
            },
            fg_neutral_chroma_builder: ChromaBuilder {
                chroma_intercept: 0.05,
                lightness_to_chroma_slope: 0.0,
                low_point_chroma_intercept: 0.03,
                hue_low_point: OklabHue::from_degrees(90.0),
            },
            prefered_hue_angle: 50.0,
            minimum_hue_angle: 45.0,
            chroma_weight_priority: 0.015,
            penalty_weight_priority: 2.0,
            maximum_accent_hue_center_translation: 0.005,
            high_contrast_fg_accent_radius_baseline: 0.02,
            fg_accent_radius_baseline: 0.09,
            rg_accent_radius_baseline: 0.065,
            bg_accent_radius_baseline: 0.0175,
            red_chroma_minimum: 0.12,
            orange_chroma_minimum: 0.08,
            yellow_chroma_minimum: 0.08,
            green_chroma_minimum: 0.07,
        },
        dark: ThemeConfig {
            base_lightness_minimum: 0.1,
            base_lightness_maximum: 0.185,
            surface_distance: 0.06,
            set_2_dps_contrast: 7.5,
            set_2_lightness_correction: 0.01,
            set_3_dps_contrast: 30.0,
            set_4_dps_contrast: 72.5,
            set_5_dps_contrast: 82.5,
            neutral_chroma_blend: 1.0,
            bg_neutral_chroma_builder: ChromaBuilder {
                chroma_intercept: 0.07,
                lightness_to_chroma_slope: 0.0833,
                low_point_chroma_intercept: 0.025,
                hue_low_point: OklabHue::from_degrees(270.0),
            },
            fg_neutral_chroma_builder: ChromaBuilder {
                chroma_intercept: 0.02,
                lightness_to_chroma_slope: 0.0,
                low_point_chroma_intercept: 0.01,
                hue_low_point: OklabHue::from_degrees(90.0),
            },
            prefered_hue_angle: 50.0,
            minimum_hue_angle: 45.0,
            chroma_weight_priority: 0.015,
            penalty_weight_priority: 2.0,
            maximum_accent_hue_center_translation: 0.005,
            high_contrast_fg_accent_radius_baseline: 0.035,
            fg_accent_radius_baseline: 0.08,
            rg_accent_radius_baseline: 0.07,
            bg_accent_radius_baseline: 0.06,
            red_chroma_minimum: 0.12,
            orange_chroma_minimum: 0.08,
            yellow_chroma_minimum: 0.08,
            green_chroma_minimum: 0.07,
        },
    }
}

pub fn generate_palette(base_hex: &str) -> std::collections::HashMap<String, String> {
    let (r, g, b) = hex_to_rgb_tuple(base_hex);
    let builder = get_default_builder();

    // Crear una imagen 1x1 para generar desde un solo color usando el API de imagen
    let pixels = [Srgb::new(r, g, b)];
    let image_ref = ImageRef::new(1, 1, &pixels).unwrap();

    match builder.generate_from_image(image_ref) {
        Ok(scheme) => {
            let mut palette = std::collections::HashMap::new();
            let to_hex = |c: Srgb<u8>| format!("#{:02x}{:02x}{:02x}", c.red, c.green, c.blue);

            palette.insert("bg".to_string(), to_hex(scheme.base));
            palette.insert("surface".to_string(), to_hex(scheme.base_high));
            palette.insert("muted".to_string(), to_hex(scheme.muted));
            palette.insert("text".to_string(), to_hex(scheme.text));
            palette.insert("primary".to_string(), to_hex(scheme.fg_accents[0]));
            palette.insert("highlight".to_string(), to_hex(scheme.fg_accents[1]));
            palette.insert("accent".to_string(), to_hex(scheme.fg_accents[2]));
            palette.insert("secondary".to_string(), to_hex(scheme.fg_accents[3]));
            palette
        }
        Err(_) => {
            // Fallback al sistema HSL original
            let (h, s, l) = hex_to_hsl(base_hex);
            let mut palette = std::collections::HashMap::new();
            palette.insert(
                "bg".to_string(),
                hsl_to_hex(h, clamp(s * 0.3, 0.0, 1.0), clamp(l * 0.12, 0.0, 0.1)),
            );
            palette.insert(
                "surface".to_string(),
                hsl_to_hex(h, clamp(s * 0.35, 0.0, 1.0), clamp(l * 0.18, 0.0, 0.15)),
            );
            palette.insert("primary".to_string(), hsl_to_hex(h, s, l));
            palette.insert(
                "secondary".to_string(),
                hsl_to_hex(
                    (h + 30.0) % 360.0,
                    clamp(s * 0.7, 0.0, 1.0),
                    clamp(l * 0.55, 0.0, 1.0),
                ),
            );
            palette.insert(
                "accent".to_string(),
                hsl_to_hex(
                    (h + 180.0) % 360.0,
                    clamp(s * 0.5, 0.0, 1.0),
                    clamp(l * 0.4, 0.0, 1.0),
                ),
            );
            palette.insert(
                "text".to_string(),
                hsl_to_hex(h, clamp(s * 0.1, 0.0, 1.0), clamp(0.95, 0.0, 1.0)),
            );
            palette.insert(
                "muted".to_string(),
                hsl_to_hex(h, clamp(s * 0.15, 0.0, 1.0), clamp(0.6, 0.0, 1.0)),
            );
            palette.insert(
                "highlight".to_string(),
                hsl_to_hex(
                    (h + 60.0) % 360.0,
                    clamp(s * 0.8, 0.0, 1.0),
                    clamp(l * 0.7, 0.0, 1.0),
                ),
            );
            palette
        }
    }
}

pub fn extract_from_image(image_path: &str, count: u8) -> Result<Vec<color_thief::Color>> {
    let img = image::open(Path::new(image_path))?;
    let img = img.to_rgba8();
    let pixels = img.as_raw();
    let palette = color_thief::get_palette(pixels, color_thief::ColorFormat::Rgba, 10, count)?;
    Ok(palette)
}

pub fn suggest_accent(image_path: &str) -> String {
    if let Ok(img) = image::open(Path::new(image_path)) {
        let img = img.to_rgb8();
        let (width, height) = img.dimensions();
        let pixels: Vec<Srgb<u8>> = img.pixels().map(|p| Srgb::new(p[0], p[1], p[2])).collect();
        let image_ref = ImageRef::new(width, height, &pixels).unwrap();

        let builder = get_default_builder();
        if let Ok(scheme) = builder.generate_from_image(image_ref) {
            let ac = scheme.fg_accents[0];
            return format!("#{:02x}{:02x}{:02x}", ac.red, ac.green, ac.blue);
        }
    }

    let colors = match extract_from_image(image_path, 12) {
        Ok(c) => c,
        Err(_) => return "#4a3f6b".to_string(),
    };

    if colors.is_empty() {
        return "#4a3f6b".to_string();
    }

    let mut scored: Vec<(f32, String)> = Vec::new();
    for c in &colors {
        let hex = format!("#{:02x}{:02x}{:02x}", c.r, c.g, c.b);
        let (_, s, l) = hex_to_hsl(&hex);
        if l < 0.08 || l > 0.92 || s < 0.05 {
            continue;
        }
        let lum_score = 1.0 - (l - 0.4).abs() * 2.0;
        let score = s * 0.6 + lum_score * 0.3;
        scored.push((score, hex));
    }

    if scored.is_empty() {
        let c = &colors[0];
        return format!("#{:02x}{:02x}{:02x}", c.r, c.g, c.b);
    }

    scored.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap());
    scored[0].1.clone()
}
