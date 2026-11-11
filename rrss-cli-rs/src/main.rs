use anyhow::{Context, Result};
use clap::Parser;
use rrss_cli_rs::{build, cli, colors, config, process, render};

use std::fs;
use std::path::Path;
use tracing::info;
use tracing_subscriber::{fmt, prelude::*, EnvFilter};

fn setup_logging() {
    tracing_subscriber::registry()
        .with(fmt::layer())
        .with(EnvFilter::from_default_env().add_directive(tracing::Level::INFO.into()))
        .init();
}

fn main() -> Result<()> {
    setup_logging();
    info!("Starting rrss-cli-rs");
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

        cli::Commands::Generate { pieza, output } => {
            let (cfg, post) = pieza_desde_flags(pieza, "pieza")?;
            let r = render::render(&post, &cfg)?;
            process::validate_path(output)?;
            if let Some(dir) = output.parent().filter(|d| !d.as_os_str().is_empty()) {
                fs::create_dir_all(dir)?;
            }
            fs::write(output, r.typ).context("No se pudo escribir el .typ")?;
            for a in &r.avisos {
                println!("⚠ {a}");
            }
            println!("✓ {} ({})", output.display(), post.preset);
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
            pieza,
            output_name,
            format,
        } => {
            let (cfg, post) = pieza_desde_flags(pieza, output_name)?;
            let mut fonts: Vec<std::path::PathBuf> =
                cfg.defaults.fonts.iter().map(Into::into).collect();
            fonts.extend(pieza.font_paths.iter().cloned());
            build::construir(&post, &cfg, &root, &fonts, format)
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

/// `generate` y `full`: una pieza desde flags, validada como las del TOML.
fn pieza_desde_flags(p: &cli::PiezaArgs, name: &str) -> Result<(config::Config, config::Post)> {
    let cfg = match &p.config {
        Some(path) => config::Config::load(path)?,
        None => config::Config::parse("")?,
    };
    let post = cfg.post_desde_flags(
        name,
        &config::Flags {
            preset: &p.preset,
            pos: &p.pos,
            tema: p.tema.as_deref(),
            image: p.image.as_deref(),
            formato: p.formato.as_deref(),
            ppi: p.ppi,
            args: &p.args,
        },
    )?;
    Ok((cfg, post))
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
                if entry.path().extension().is_some_and(|e| e == "typ") {
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
        match process::compile_typst(&target, &output_path, ppi, root) {
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
            Err(e) => println!("✗ Error: {}", e),
        }
    }
    Ok(())
}
