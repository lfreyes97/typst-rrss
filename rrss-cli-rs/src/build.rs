use anyhow::Result;
use std::fs;
use std::path::Path;

use crate::cli::{GenerateParams, ImageFormat};
use crate::config::Config;
use crate::generate::do_generate;

/// Convert a PNG file to WebP format, removing the original PNG
pub fn convert_to_webp(png_path: &Path) -> Result<std::path::PathBuf> {
    let img = image::open(png_path)?;
    let webp_path = png_path.with_extension("webp");
    img.save(&webp_path)?;
    fs::remove_file(png_path)?;
    Ok(webp_path)
}

/// Run the typst compile command
pub fn run_typst_compile(root: &Path, input: &Path, output: &Path, ppi: u32) -> Result<()> {
    let status = std::process::Command::new("typst")
        .arg("compile")
        .arg("--root")
        .arg(root)
        .arg("--ppi")
        .arg(ppi.to_string())
        .arg(input)
        .arg(output)
        .status()
        .map_err(|e| anyhow::anyhow!("Failed to execute typst command: {}", e))?;

    if status.success() {
        Ok(())
    } else {
        Err(anyhow::anyhow!("Typst compilation failed"))
    }
}

/// Execute the Build command: iterate posts from config, generate .typ, compile
pub fn run_build(
    cfg: &Config,
    root: &Path,
    only: &Option<String>,
    dry_run: bool,
    format: &ImageFormat,
) -> Result<()> {
    let defaults = cfg.defaults.clone();

    let posts = match &cfg.posts {
        Some(p) => p,
        None => return Ok(()),
    };

    for (i, post) in posts.iter().enumerate() {
        let name = if post.name.is_empty() {
            format!("post_{}", i)
        } else {
            post.name.clone()
        };

        if let Some(only_name) = only {
            if &name != only_name {
                continue;
            }
        }

        // Helper closures to resolve defaults
        let get_str = |opt: &Option<String>, key: &str, def: &str| -> String {
            opt.clone().unwrap_or_else(|| {
                defaults
                    .get(key)
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string())
                    .unwrap_or(def.to_string())
            })
        };
        let get_bool = |opt: Option<bool>, key: &str, def: bool| -> bool {
            opt.unwrap_or_else(|| defaults.get(key).and_then(|v| v.as_bool()).unwrap_or(def))
        };

        let params = GenerateParams {
            brand: get_str(&post.brand, "brand", "Presuposicionalismo"),
            title: get_str(&post.title, "title", ""),
            quote: get_str(&post.quote, "quote", ""),
            image: post.image.clone().or_else(|| {
                defaults
                    .get("image")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string())
            }),
            logo: post.logo.clone().or_else(|| {
                defaults
                    .get("logo")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string())
            }),
            overlay: post
                .overlay
                .clone()
                .or_else(|| {
                    defaults
                        .get("overlay")
                        .and_then(|v| v.as_str())
                        .map(|s| s.to_string())
                })
                .or(Some("assets/Solid-bg.svg".to_string())),
            accent: get_str(&post.accent, "accent", "theme"),
            auto_accent: get_str(&post.accent, "accent", "theme") == "auto",
            url: get_str(&post.url, "url", "Presuposicionalismo.com"),
            platform: get_str(&post.platform, "platform", "instagram-post"),
            layout: get_str(&post.layout, "layout", "article"),
            theme: get_str(&post.theme, "theme", "dark"),
            author: get_str(&post.author, "author", ""),
            source: post.source.clone().or_else(|| {
                defaults
                    .get("source")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string())
            }),
            tag: post.tag.clone().or_else(|| {
                defaults
                    .get("tag")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string())
            }),
            slides: post.slides.as_ref().map(|s| s.join("|")),
            contour: get_bool(post.contour, "contour", false),
            font_heading: post.font_heading.clone().or_else(|| {
                defaults
                    .get("font-heading")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string())
            }),
            font_body: post.font_body.clone().or_else(|| {
                defaults
                    .get("font-body")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string())
            }),
            font_mono: post.font_mono.clone().or_else(|| {
                defaults
                    .get("font-mono")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string())
            }),
        };

        println!("\n  ⟩ {} — {}", name, params.title);

        if dry_run {
            println!(
                "    (dry run) layout={} platform={} theme={}",
                params.layout, params.platform, params.theme
            );
            continue;
        }

        let content = match do_generate(&params, Some(cfg)) {
            Ok(c) => c,
            Err(e) => {
                println!("    ✗ Error generating content: {}", e);
                continue;
            }
        };

        let typ_file = format!("{}.typ", name);
        if let Err(e) = fs::write(&typ_file, content) {
            println!("    ✗ Error writing file: {}", e);
            continue;
        }

        let ppi = post.ppi.unwrap_or(144);
        let output_dir = "output";
        fs::create_dir_all(output_dir)?;
        let out_filename = if params.platform == "instagram-carousel" {
            format!("{}_{{{}}}.png", name, "p")
        } else {
            format!("{}.png", name)
        };
        let output_path = Path::new(output_dir).join(&out_filename);

        match run_typst_compile(root, Path::new(&typ_file), &output_path, ppi) {
            Ok(_) => {
                if *format == ImageFormat::Webp {
                    match convert_to_webp(&output_path) {
                        Ok(webp) => println!("    ✓ Compiled → {}", webp.display()),
                        Err(e) => println!("    ✗ WebP conversion failed: {}", e),
                    }
                } else {
                    println!("    ✓ Compiled");
                }
            }
            Err(_) => println!("    ✗ Compilation failed"),
        }
    }

    Ok(())
}
