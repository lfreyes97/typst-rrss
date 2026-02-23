use anyhow::{Context, Result};
use clap::Parser;
use rrss_cli_rs::{build, cli, colors, config, generate};

use std::fs;
use std::path::Path;

fn main() -> Result<()> {
    let parsed = cli::Cli::parse();
    let root = std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."));

    match &parsed.command {
        cli::Commands::Colors {
            base_color,
            format,
            name,
        } => cmd_colors(base_color, format, name),

        cli::Commands::Extract {
            image,
            count,
            format,
            suggest_accent,
            name: _,
        } => cmd_extract(image, *count, format, *suggest_accent),

        cli::Commands::Generate {
            brand,
            title,
            quote,
            image,
            logo,
            overlay,
            accent,
            auto_accent,
            url,
            platform,
            layout,
            theme,
            author,
            source,
            tag,
            slides,
            contour,
            output,
            font_heading,
            font_body,
            font_mono,
        } => {
            let params = cli::GenerateParams {
                brand: brand.clone(),
                title: title.clone(),
                quote: quote.clone(),
                image: image.clone(),
                logo: logo.clone(),
                overlay: overlay.clone(),
                accent: accent.clone(),
                auto_accent: *auto_accent,
                url: url.clone(),
                platform: platform.clone(),
                layout: layout.clone(),
                theme: theme.clone(),
                author: author.clone(),
                source: source.clone(),
                tag: tag.clone(),
                slides: slides.clone(),
                contour: *contour,
                font_heading: font_heading.clone(),
                font_body: font_body.clone(),
                font_mono: font_mono.clone(),
            };
            let content = generate::do_generate(&params, None)?;
            fs::write(output, content).context("Failed to write output file")?;
            println!("✓ Generated {} ({}, {})", output, layout, platform);
            Ok(())
        }

        cli::Commands::Compile {
            files,
            ppi,
            output_dir,
            all,
            template,
            format,
        } => cmd_compile(&root, files, *ppi, output_dir, *all, template, format),

        cli::Commands::Full {
            brand,
            title,
            quote,
            image,
            logo,
            overlay,
            accent,
            auto_accent,
            url,
            platform,
            layout,
            theme,
            author,
            source,
            tag,
            ppi,
            output_name,
            font_heading,
            font_body,
            font_mono,
            format,
        } => {
            let params = cli::GenerateParams {
                brand: brand.clone(),
                title: title.clone(),
                quote: quote.clone(),
                image: image.clone(),
                logo: logo.clone(),
                overlay: overlay.clone(),
                accent: accent.clone(),
                auto_accent: *auto_accent,
                url: url.clone(),
                platform: platform.clone(),
                layout: layout.clone(),
                theme: theme.clone(),
                author: author.clone(),
                source: source.clone(),
                tag: tag.clone(),
                slides: None,
                contour: false,
                font_heading: font_heading.clone(),
                font_body: font_body.clone(),
                font_mono: font_mono.clone(),
            };
            let content = generate::do_generate(&params, None)?;
            let typ_file = format!("{}.typ", output_name);
            fs::write(&typ_file, content).context("Failed to write typ file")?;

            let out_dir = "output";
            fs::create_dir_all(out_dir)?;
            let out_filename = if params.platform == "instagram-carousel" {
                format!("{}_{{{}}}.png", output_name, "p")
            } else {
                format!("{}.png", output_name)
            };
            let output_path = Path::new(out_dir).join(&out_filename);

            println!("Compiling {} -> {}", typ_file, output_path.display());
            build::run_typst_compile(&root, Path::new(&typ_file), &output_path, *ppi)?;

            if *format == cli::ImageFormat::Webp {
                let webp = build::convert_to_webp(&output_path)?;
                println!("✓ Converted to {}", webp.display());
            }
            Ok(())
        }

        cli::Commands::Build {
            config_file,
            only,
            dry_run,
            format,
        } => {
            let cfg = config::Config::load(config_file)?;
            build::run_build(&cfg, &root, only, *dry_run, format)
        }
    }
}

// ─── Command handlers ──────────────────────────────────────────────────────────

fn cmd_colors(base_color: &str, format: &cli::OutputFormat, name: &str) -> Result<()> {
    let palette = colors::generate_palette(base_color);
    match format {
        cli::OutputFormat::Json => println!("{}", serde_json::to_string_pretty(&palette)?),
        cli::OutputFormat::Typst => {
            println!("  {}: (", name);
            let mut keys: Vec<_> = palette.keys().collect();
            keys.sort();
            for key in keys {
                println!("    {}: rgb(\"{}\"),", key, palette[key]);
            }
            println!("  ),");
        }
        cli::OutputFormat::Table => {
            println!("Palette: {}", base_color);
            for (k, v) in &palette {
                println!("{}: {}", k, v);
            }
        }
    }
    Ok(())
}

fn cmd_extract(
    image: &std::path::PathBuf,
    count: u8,
    format: &cli::ExtractFormat,
    suggest_accent: bool,
) -> Result<()> {
    println!("Analyzing {:?}...", image);
    let img_str = image.to_str().context("Invalid image path")?;

    if suggest_accent {
        let accent = colors::suggest_accent(img_str);
        println!("Suggested accent: {}", accent);
        return Ok(());
    }

    let extracted = colors::extract_from_image(img_str, count)?;
    match format {
        cli::ExtractFormat::Json => println!("JSON output not fully implemented"),
        cli::ExtractFormat::Palette => {
            let base = format!(
                "#{:02x}{:02x}{:02x}",
                extracted[0].r, extracted[0].g, extracted[0].b
            );
            let palette = colors::generate_palette(&base);
            println!("Palette from {}:", base);
            for (k, v) in palette {
                println!("{}: {}", k, v);
            }
        }
        cli::ExtractFormat::Table => {
            for (i, c) in extracted.iter().enumerate() {
                println!("{}: #{:02x}{:02x}{:02x}", i + 1, c.r, c.g, c.b);
            }
        }
    }
    Ok(())
}

fn cmd_compile(
    root: &Path,
    files: &[std::path::PathBuf],
    ppi: u32,
    output_dir: &str,
    all: bool,
    template: &Option<String>,
    format: &cli::ImageFormat,
) -> Result<()> {
    let targets = if all {
        let content_dir = root.join("content");
        let mut found = Vec::new();
        if content_dir.exists() {
            for entry in walkdir::WalkDir::new(content_dir).max_depth(1) {
                let entry = entry?;
                if entry.path().extension().map_or(false, |e| e == "typ") {
                    found.push(entry.path().to_path_buf());
                }
            }
        }
        found.sort();
        if found.is_empty() && root.join("main.typ").exists() {
            found.push(root.join("main.typ"));
        }
        found
    } else if !files.is_empty() {
        files.to_vec()
    } else {
        vec![root.join("main.typ")]
    };

    if targets.is_empty() {
        println!("⚠ No files specified to compile.");
        return Ok(());
    }

    fs::create_dir_all(output_dir)?;
    println!(
        "PPI: {}  Files: {}  Output: {}/",
        ppi,
        targets.len(),
        output_dir
    );

    for target in targets {
        let stem = target
            .file_stem()
            .context("No file stem")?
            .to_str()
            .context("Invalid stem")?;
        let out_filename = if let Some(tmpl) = template {
            format!("{}-{}.png", stem, tmpl)
        } else {
            format!("{}.png", stem)
        };
        let output_path = Path::new(output_dir).join(&out_filename);

        print!("  ⟩ {}... ", stem);
        match build::run_typst_compile(root, &target, &output_path, ppi) {
            Ok(_) => {
                if *format == cli::ImageFormat::Webp {
                    match build::convert_to_webp(&output_path) {
                        Ok(webp) => println!("✓ → {}", webp.display()),
                        Err(e) => println!("✗ WebP error: {}", e),
                    }
                } else {
                    println!("✓");
                }
            }
            Err(_) => println!("✗ Error"),
        }
    }
    Ok(())
}
