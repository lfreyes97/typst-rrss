use anyhow::{bail, Result};
use std::fs;
use std::path::{Path, PathBuf};

use crate::cli::ImageFormat;
use crate::config::{Config, Post};
use crate::process;
use crate::render::render;

/// Convert a PNG file to WebP format, removing the original PNG
pub fn convert_to_webp(png_path: &Path) -> Result<PathBuf> {
    let img = image::open(png_path)?;
    let webp_path = png_path.with_extension("webp");
    img.save(&webp_path)?;
    fs::remove_file(png_path)?;
    Ok(webp_path)
}

/// Las carpetas de `fonts` de [defaults] más las de `--font-path`. Typst ignora
/// en silencio una carpeta que no existe; aquí es un error.
pub fn fuentes(cfg: &Config, extra: &[PathBuf]) -> Result<Vec<PathBuf>> {
    let fonts: Vec<PathBuf> = cfg
        .defaults
        .fonts
        .iter()
        .map(PathBuf::from)
        .chain(extra.iter().cloned())
        .collect();
    for f in &fonts {
        if !f.is_dir() {
            bail!("la carpeta de fuentes {} no existe", f.display());
        }
    }
    Ok(fonts)
}

pub const SALIDA: &str = "output";
pub const TYP_DIR: &str = "output/typ";

/// Genera y compila los posts de `posts.toml`. Devuelve error si alguno falla,
/// pero procesa todos antes de rendirse.
pub fn run_build(
    cfg: &Config,
    root: &Path,
    solo: &Option<String>,
    dry_run: bool,
    format: &ImageFormat,
) -> Result<()> {
    // Toda la configuración se valida antes de compilar nada, incluso los
    // posts que `--solo` va a saltar: un typo no debe esperar a su turno.
    let posts = cfg.resolve_all()?;
    let posts: Vec<&Post> = posts
        .iter()
        .filter(|p| match solo {
            Some(o) => &p.name == o,
            None => true,
        })
        .collect();
    if let Some(o) = solo {
        if posts.is_empty() {
            bail!("no hay ningún post llamado {o:?}");
        }
    }

    let fonts = fuentes(cfg, &[])?;
    let mut fallos = 0;

    for post in &posts {
        println!("\n  ⟩ {} — {}", post.name, post.preset);
        if dry_run {
            let formato = post
                .formato
                .map_or("formato del preset".to_string(), |(w, h)| {
                    format!("{w}×{h}")
                });
            println!(
                "    (dry run) {formato} @ {} ppi, tema={}",
                post.ppi,
                post.tema.as_deref().unwrap_or("-")
            );
            continue;
        }
        if let Err(e) = construir(post, cfg, root, &fonts, format) {
            println!("    ✗ {e:#}");
            fallos += 1;
        }
    }

    if fallos > 0 {
        bail!("{fallos} de {} posts fallaron", posts.len());
    }
    Ok(())
}

/// Renderiza, escribe `output/typ/<name>.typ` y compila a `output/<name>.png`.
pub fn construir(
    post: &Post,
    cfg: &Config,
    root: &Path,
    fonts: &[PathBuf],
    format: &ImageFormat,
) -> Result<()> {
    let r = render(post, cfg)?;
    for a in &r.avisos {
        println!("    ⚠ {a}");
    }

    let typ = Path::new(TYP_DIR).join(format!("{}.typ", post.name));
    let png = Path::new(SALIDA).join(format!("{}.png", post.name));
    for f in [&typ, &png] {
        if let Some(parent) = f.parent() {
            fs::create_dir_all(parent)?;
        }
    }
    fs::write(&typ, r.typ)?;

    let avisos = process::compile_typst_with(&typ, &png, post.ppi, root, fonts)?;
    process::mostrar_avisos(&avisos, "    ");
    if *format == ImageFormat::Webp {
        println!("    ✓ → {}", convert_to_webp(&png)?.display());
    } else {
        println!("    ✓ → {}", png.display());
    }
    Ok(())
}
