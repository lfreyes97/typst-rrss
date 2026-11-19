use anyhow::{Context, Result};
use clap::Parser;
use rrss_cli_rs::{build, cli, colors, config, pieza, process, render, scaffold};

use std::collections::BTreeMap;
use std::fs;
use std::path::Path;
use tracing_subscriber::{fmt, prelude::*, EnvFilter};

/// Trazas a stderr, solo avisos salvo que `RUST_LOG` pida más: stdout es del resultado.
fn setup_logging() {
    tracing_subscriber::registry()
        .with(fmt::layer().with_writer(std::io::stderr))
        .with(EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("warn")))
        .init();
}

fn main() -> Result<()> {
    setup_logging();
    let parsed = cli::Cli::parse();
    let root = std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."));

    match &parsed.command {
        cli::Commands::Build {
            config_file,
            solo,
            dry_run,
            imagen,
        } => {
            // un documento de piezas (instatypst-piece/v1) o el posts.toml de siempre
            let cfg = pieza::cargar(config_file)?;
            build::run_build(&cfg, &root, solo, *dry_run, imagen)
        }

        cli::Commands::Init {
            presets,
            output,
            force,
        } => {
            let presets = if presets.is_empty() {
                vec!["liso".to_string()]
            } else {
                presets.clone()
            };
            // La extensión decide qué se escribe; la plantilla se valida antes de tocar el disco.
            let es_typ = output.extension().is_some_and(|e| e == "typ");
            let (contenido, siguiente) = if es_typ {
                (scaffold::typ(&presets)?, "compile")
            } else {
                (scaffold::toml(&presets)?, "build")
            };
            process::validate_path(output)?;
            if output.exists() && !force {
                anyhow::bail!(
                    "{} ya existe (usa --force para sobrescribirlo)",
                    output.display()
                );
            }
            if let Some(dir) = output.parent().filter(|d| !d.as_os_str().is_empty()) {
                fs::create_dir_all(dir)?;
            }
            fs::write(output, contenido).context("No se pudo escribir la plantilla")?;
            println!("✓ {} ({})", output.display(), presets.join(", "));
            println!("Siguiente: rrss-cli-rs {siguiente} {}", output.display());
            Ok(())
        }

        cli::Commands::Full {
            pieza,
            output,
            imagen,
        } => {
            let (cfg, post) = pieza_desde_flags(pieza, output)?;
            let fonts = build::fuentes(&cfg, &pieza.font_paths)?;
            build::construir(&post, &cfg, &root, &fonts, imagen)
        }

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
            output,
            imagen,
        } => cmd_compile(&root, files, *ppi, output, imagen),

        cli::Commands::Extract {
            image,
            count,
            format,
            suggest_accent,
        } => cmd_extract(image, *count, format, *suggest_accent),

        cli::Commands::Colors {
            base_color,
            format,
            name,
        } => cmd_colors(base_color, format, name),
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

/// La paleta en orden alfabético, igual en cada ejecución.
fn paleta(base: &str) -> BTreeMap<String, String> {
    colors::generate_palette(base).into_iter().collect()
}

fn cmd_colors(base_color: &str, format: &cli::OutputFormat, name: &str) -> Result<()> {
    colors::hex_to_rgb_tuple(base_color)?;
    let palette = paleta(base_color);
    match format {
        cli::OutputFormat::Json => println!("{}", serde_json::to_string_pretty(&palette)?),
        cli::OutputFormat::Typst => {
            println!("  {}: (", name);
            for (key, value) in &palette {
                println!("    {}: rgb(\"{}\"),", key, value);
            }
            println!("  ),");
        }
        cli::OutputFormat::Table => {
            println!("Paleta: {}", base_color);
            for (k, v) in &palette {
                println!("{}: {}", k, v);
            }
        }
    }
    Ok(())
}

fn cmd_extract(
    image: &Path,
    count: u8,
    format: &cli::ExtractFormat,
    suggest_accent: bool,
) -> Result<()> {
    if !image.is_file() {
        anyhow::bail!("no existe la imagen {}", image.display());
    }
    let img_str = image.to_str().context("La ruta de la imagen no es UTF-8")?;

    if suggest_accent {
        println!("Acento sugerido: {}", colors::suggest_accent(img_str));
        return Ok(());
    }

    let extracted = colors::extract_from_image(img_str, count)
        .with_context(|| format!("no se pudieron extraer colores de {}", image.display()))?;
    let hex: Vec<String> = extracted
        .iter()
        .map(|c| format!("#{:02x}{:02x}{:02x}", c.r, c.g, c.b))
        .collect();
    match format {
        cli::ExtractFormat::Json => println!("{}", serde_json::to_string_pretty(&hex)?),
        cli::ExtractFormat::Palette => {
            let base = hex.first().context("la imagen no tiene colores")?;
            println!("Paleta desde {}:", base);
            for (k, v) in paleta(base) {
                println!("{}: {}", k, v);
            }
        }
        cli::ExtractFormat::Table => {
            for (i, h) in hex.iter().enumerate() {
                println!("{}: {}", i + 1, h);
            }
        }
    }
    Ok(())
}

/// Compila cada archivo aunque otro falle; termina con error si alguno falló.
fn cmd_compile(
    root: &Path,
    files: &[std::path::PathBuf],
    ppi: u32,
    output_dir: &str,
    format: &cli::ImageFormat,
) -> Result<()> {
    if files.is_empty() {
        anyhow::bail!(
            "falta qué compilar.\nUso: rrss-cli-rs compile PIEZA.typ [MÁS.typ…] [--ppi N] [-o CARPETA]"
        );
    }

    fs::create_dir_all(output_dir)?;
    println!(
        "ppi: {}  archivos: {}  salida: {}/",
        ppi,
        files.len(),
        output_dir
    );

    let mut fallos = 0;
    for target in files {
        let stem = target
            .file_stem()
            .and_then(|s| s.to_str())
            .with_context(|| format!("nombre de archivo no válido: {}", target.display()))?;
        let output_path = Path::new(output_dir).join(format!("{stem}.png"));

        println!("\n  ⟩ {stem}");
        let res = process::compile_typst(target, &output_path, ppi, root).and_then(|avisos| {
            process::mostrar_avisos(&avisos, "    ");
            if *format == cli::ImageFormat::Webp {
                build::convert_to_webp(&output_path)
            } else {
                Ok(output_path.clone())
            }
        });
        match res {
            Ok(img) => println!("    ✓ → {}", img.display()),
            Err(e) => {
                println!("    ✗ {e:#}");
                fallos += 1;
            }
        }
    }

    if fallos > 0 {
        anyhow::bail!("{fallos} de {} archivos fallaron", files.len());
    }
    Ok(())
}
