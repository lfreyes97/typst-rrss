use anyhow::Result;
use std::collections::HashMap;

use crate::cli::GenerateParams;
use crate::config::Config;
use crate::{colors, themes};

/// Resolve the theme palette from parameters
fn resolve_theme(
    params: &GenerateParams,
    cfg: Option<&Config>,
) -> Result<HashMap<String, String>> {
    let palettes = themes::predefined::get_theme_palettes(cfg);

    if let Some(map) = palettes.get(&params.theme) {
        return Ok(map.clone());
    }

    match params.theme.as_str() {
        "auto" => resolve_auto_theme(params, &palettes),
        "auto-matugen" => resolve_matugen_theme(params, &palettes),
        _ => Ok(palettes.get("dark").unwrap().clone()),
    }
}

fn resolve_auto_theme(
    params: &GenerateParams,
    palettes: &HashMap<String, HashMap<String, String>>,
) -> Result<HashMap<String, String>> {
    if let Some(path) = &params.image {
        match colors::extract_from_image(path, 8) {
            Ok(extracted) => {
                let base = format!(
                    "#{:02x}{:02x}{:02x}",
                    extracted[0].r, extracted[0].g, extracted[0].b
                );
                Ok(colors::generate_palette(&base))
            }
            Err(_) => Ok(palettes.get("dark").unwrap().clone()),
        }
    } else {
        Ok(palettes.get("dark").unwrap().clone())
    }
}

fn resolve_matugen_theme(
    params: &GenerateParams,
    palettes: &HashMap<String, HashMap<String, String>>,
) -> Result<HashMap<String, String>> {
    use material_colors::image::{FilterType, ImageReader};
    use material_colors::theme::ThemeBuilder;

    let path = match &params.image {
        Some(p) => p,
        None => return Ok(palettes.get("dark").unwrap().clone()),
    };

    let bytes = match std::fs::read(path) {
        Ok(b) => b,
        Err(e) => {
            eprintln!("⚠ Could not read image {:?}: {}", path, e);
            return Ok(palettes.get("dark").unwrap().clone());
        }
    };

    let mut data = match ImageReader::read(bytes) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("⚠ Could not decode image {:?}: {}", path, e);
            return Ok(palettes.get("dark").unwrap().clone());
        }
    };

    // Resize to 128x128 for fast & stable color extraction (recommended by material-colors docs)
    data.resize(128, 128, FilterType::Lanczos3);

    let color = ImageReader::extract_color(&data);
    let theme = ThemeBuilder::with_source(color).build();
    let scheme = &theme.schemes.dark;

    let mut m = HashMap::new();
    m.insert("bg".to_string(), scheme.background.to_hex_with_pound());
    m.insert("surface".to_string(), scheme.surface.to_hex_with_pound());
    m.insert("primary".to_string(), scheme.primary.to_hex_with_pound());
    m.insert("secondary".to_string(), scheme.secondary.to_hex_with_pound());
    m.insert("accent".to_string(), scheme.tertiary.to_hex_with_pound());
    m.insert("text".to_string(), scheme.on_background.to_hex_with_pound());
    m.insert("muted".to_string(), scheme.outline.to_hex_with_pound());
    m.insert(
        "highlight".to_string(),
        scheme.primary_container.to_hex_with_pound(),
    );

    Ok(m)
}

/// Generate a Typst file from parameters
pub fn do_generate(params: &GenerateParams, cfg: Option<&Config>) -> Result<String> {
    let mut final_accent = params.accent.clone();

    // Auto accent handling
    if params.auto_accent {
        if let Some(img_path) = &params.image {
            final_accent = colors::suggest_accent(img_path);
        }
    }

    // Theme resolution
    let theme_map = resolve_theme(params, cfg)?;

    // Start building the Typst file
    let mut out = String::new();
    out.push_str("// Auto-generado por rrss-cli-rs\n");
    out.push_str("#import \"@local/rrss:0.1.0\": *\n\n");

    // Construct Typst dictionary for theme
    out.push_str("#let t = (");
    let mut keys: Vec<_> = theme_map.keys().collect();
    keys.sort();
    for k in keys {
        let v = theme_map.get(k).unwrap();
        if k.as_str() == "accent" && final_accent != "theme" {
            out.push_str(&format!("{}: rgb(\"{}\"), ", k, final_accent));
        } else {
            out.push_str(&format!("{}: rgb(\"{}\"), ", k, v));
        }
    }

    // Inject fonts if provided
    if let Some(f) = &params.font_heading {
        out.push_str(&format!(" \"font-heading\": \"{}\", ", f));
    }
    if let Some(f) = &params.font_body {
        out.push_str(&format!(" \"font-body\": \"{}\", ", f));
    }
    if let Some(f) = &params.font_mono {
        out.push_str(&format!(" \"font-mono\": \"{}\", ", f));
    }

    out.push_str(")\n\n");

    // Configuration line (explicit dimensions)
    out.push_str(&format!(
        "#show: set-dimensions.with(platform: \"{}\", theme: t)\n\n",
        params.platform
    ));

    // Start Layout call
    out.push_str(&format!("#{}(\n", params.layout));
    out.push_str("    t,\n");

    // Required parameters
    out.push_str(&format!("    brand: \"{}\",\n", params.brand));

    // Logo
    if let Some(l) = &params.logo {
        if l.ends_with(".svg") {
            out.push_str(&format!(
                "    logo: recolor-svg(\"{}\", t.text, original: \"currentColor\"),\n",
                l
            ));
        } else {
            out.push_str(&format!("    logo: image(\"{}\", width: 100%),\n", l));
        }
    }

    // Title & Quote
    if !params.title.is_empty() {
        out.push_str(&format!("    title: \"{}\",\n", params.title));
    }
    if !params.quote.is_empty() {
        out.push_str(&format!("    quote-text: \"{}\",\n", params.quote));
    }

    // Optional metadata
    if !params.author.is_empty() {
        out.push_str(&format!("    author: \"{}\",\n", params.author));
    }
    if let Some(s) = &params.source {
        out.push_str(&format!("    source: \"{}\",\n", s));
    }

    // Background and overlay
    if let Some(img) = &params.image {
        out.push_str(&format!("    bg-image: image(\"{}\", width: 100%),\n", img));
    }
    if let Some(ov) = &params.overlay {
        if ov.ends_with(".svg") {
            out.push_str(&format!(
                "    overlay: recolor-svg(\"{}\", t.bg, width: 100%),\n",
                ov
            ));
        } else {
            out.push_str(&format!(
                "    overlay: image(\"{}\", width: 100%, height: 100%),\n",
                ov
            ));
        }
    }

    // Colors and URL
    if !params.url.is_empty() {
        out.push_str(&format!("    url: \"{}\",\n", params.url));
    }

    // Contour and tags
    if params.contour {
        out.push_str("    contour: true,\n");
    }
    if let Some(t) = &params.tag {
        out.push_str(&format!("    tag: \"{}\",\n", t));
    }

    // Slides
    if let Some(s) = &params.slides {
        if s.contains('|') {
            let list: Vec<String> = s.split('|').map(|x| format!("\"{}\"", x.trim())).collect();
            out.push_str(&format!("    slides: ({},),\n", list.join(", ")));
        } else {
            out.push_str(&format!("    slides: {},\n", s));
        }
    }

    out.push_str(")\n");

    Ok(out)
}
