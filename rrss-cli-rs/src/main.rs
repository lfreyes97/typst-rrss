use anyhow::{anyhow, Context, Result};
use clap::{Parser, Subcommand, ValueEnum};
use rrss_cli_rs::{colors, config, constants, themes};

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Genera un esquema de colores
    Colors {
        base_color: String,
        #[arg(short, long, default_value = "table")]
        format: OutputFormat,
        #[arg(short, long, default_value = "custom")]
        name: String,
    },

    /// Extrae colores de una imagen
    Extract {
        image: PathBuf,
        #[arg(short, long, default_value_t = 8)]
        count: u8,
        #[arg(short, long, default_value = "table")]
        format: ExtractFormat,
        #[arg(long)]
        suggest_accent: bool,
        #[arg(short, long, default_value = "extracted")]
        name: String,
    },

    /// Genera main.typ (comando único)
    Generate {
        #[arg(short, long, default_value = "Presuposicionalismo")]
        brand: String,
        #[arg(short, long)]
        title: String,
        #[arg(short, long)]
        quote: String,
        #[arg(short, long)]
        image: Option<String>,
        #[arg(long)]
        logo: Option<String>,
        #[arg(long)]
        overlay: Option<String>,
        #[arg(short, long, default_value = "theme")]
        accent: String,
        #[arg(long)]
        auto_accent: bool,
        #[arg(short, long, default_value = "Presuposicionalismo.com")]
        url: String,
        #[arg(short, long, default_value = "instagram-post")]
        platform: String,
        #[arg(short, long, default_value = "article")]
        layout: String,
        #[arg(long, default_value = "dark")]
        theme: String,
        #[arg(long, default_value = "")]
        author: String,
        #[arg(long)]
        source: Option<String>,
        #[arg(long)]
        tag: Option<String>,
        #[arg(long)]
        slides: Option<String>,
        #[arg(long)]
        contour: bool,
        #[arg(short, long, default_value = "main.typ")]
        output: String,
    },

    /// Compila archivos .typ a PNG
    Compile {
        #[arg(num_args = 0..)]
        files: Vec<PathBuf>,
        #[arg(long, default_value_t = 144)]
        ppi: u32,
        #[arg(short = 'd', long, default_value = "output")]
        output_dir: String,
        #[arg(short, long)]
        all: bool,
        #[arg(long)]
        template: Option<String>,
    },

    /// Pipeline completo: genera y compila
    Full {
        #[arg(short, long, default_value = "Presuposicionalismo")]
        brand: String,
        #[arg(short, long)]
        title: String,
        #[arg(short, long)]
        quote: String,
        #[arg(short, long)]
        image: Option<String>,
        #[arg(long)]
        logo: Option<String>,
        #[arg(long)]
        overlay: Option<String>,
        #[arg(short, long, default_value = "theme")]
        accent: String,
        #[arg(long)]
        auto_accent: bool,
        #[arg(short, long, default_value = "Presuposicionalismo.com")]
        url: String,
        #[arg(short, long, default_value = "instagram-post")]
        platform: String,
        #[arg(short, long, default_value = "article")]
        layout: String,
        #[arg(long, default_value = "dark")]
        theme: String,
        #[arg(long, default_value = "")]
        author: String,
        #[arg(long)]
        source: Option<String>,
        #[arg(long)]
        tag: Option<String>,
        #[arg(long, default_value_t = 144)]
        ppi: u32,
        #[arg(short, long, default_value = "main")]
        output_name: String,
    },

    /// Genera posts desde TOML
    Build {
        #[arg(default_value = "posts.toml")]
        config_file: PathBuf,
        #[arg(short, long)]
        only: Option<String>,
        #[arg(long)]
        dry_run: bool,
    },
}

#[derive(ValueEnum, Clone)]
enum OutputFormat {
    Table,
    Json,
    Typst,
}

#[derive(ValueEnum, Clone)]
enum ExtractFormat {
    Table,
    Json,
    Palette,
}

struct GenerateParams {
    brand: String,
    title: String,
    quote: String,
    image: Option<String>,
    logo: Option<String>,
    overlay: Option<String>,
    accent: String,
    auto_accent: bool,
    url: String,
    platform: String,
    layout: String,
    theme: String,
    author: String,
    source: Option<String>,
    tag: Option<String>,
    slides: Option<String>,
    contour: bool,
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let root = constants::project_root();

    match &cli.command {
        Commands::Colors {
            base_color,
            format,
            name,
        } => {
            let palette = colors::generate_palette(base_color);
            match format {
                OutputFormat::Json => println!("{}", serde_json::to_string_pretty(&palette)?),
                OutputFormat::Typst => {
                    println!("  {}: (", name);
                    let mut keys: Vec<_> = palette.keys().collect();
                    keys.sort();
                    for key in keys {
                        println!("    {}: rgb(\"{}\"),", key, palette[key]);
                    }
                    println!("  ),");
                }
                OutputFormat::Table => {
                    println!("Palette: {}", base_color);
                    for (k, v) in &palette {
                        println!("{}: {}", k, v);
                    }
                }
            }
        }
        Commands::Extract {
            image,
            count,
            format,
            suggest_accent,
            name: _,
        } => {
            println!("Analyzing {:?}...", image);
            let img_str = image.to_str().context("Invalid image path")?;

            if *suggest_accent {
                let accent = colors::suggest_accent(img_str);
                println!("Suggested accent: {}", accent);
                return Ok(());
            }

            let extracted = colors::extract_from_image(img_str, *count)?;
            match format {
                ExtractFormat::Json => println!("JSON output not fully implemented"),
                ExtractFormat::Palette => {
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
                ExtractFormat::Table => {
                    for (i, c) in extracted.iter().enumerate() {
                        println!("{}: #{:02x}{:02x}{:02x}", i + 1, c.r, c.g, c.b);
                    }
                }
            }
        }
        Commands::Generate {
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
        } => {
            let params = GenerateParams {
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
            };
            let content = do_generate(&params, None)?;
            fs::write(output, content).context("Failed to write output file")?;
            println!("✓ Generated {} ({}, {})", output, layout, platform);
        }
        Commands::Full {
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
        } => {
            let params = GenerateParams {
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
            };
            let content = do_generate(&params, None)?;
            let typ_file = format!("{}.typ", output_name);
            fs::write(&typ_file, content).context("Failed to write typ file")?;

            let output_dir = "output";
            fs::create_dir_all(output_dir)?;
            let out_filename = if params.platform == "instagram-carousel" {
                format!("{}_{{p}}.png", output_name)
            } else {
                format!("{}.png", output_name)
            };
            let output_path = Path::new(output_dir).join(&out_filename);

            println!("Compiling {} -> {}", typ_file, output_path.display());
            run_typst_compile(&root, &Path::new(&typ_file), &output_path, *ppi)?;
        }
        Commands::Compile {
            files,
            ppi,
            output_dir,
            all,
            template,
        } => {
            let targets = if *all {
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
                if found.is_empty() {
                    // If content dir empty, checking root main.typ
                    if root.join("main.typ").exists() {
                        found.push(root.join("main.typ"));
                    }
                }
                found
            } else if !files.is_empty() {
                files.clone()
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
                match run_typst_compile(&root, &target, &output_path, *ppi) {
                    Ok(_) => println!("✓"),
                    Err(_) => println!("✗ Error"),
                }
            }
        }
        Commands::Build {
            config_file,
            only,
            dry_run,
        } => {
            let cfg = config::Config::load(config_file)?;
            let defaults = cfg.defaults.clone();

            if let Some(ref posts) = cfg.posts {
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
                        opt.unwrap_or_else(|| {
                            defaults.get(key).and_then(|v| v.as_bool()).unwrap_or(def)
                        })
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
                        slides: if let Some(s) = &post.slides {
                            Some(s.join("|"))
                        } else {
                            None
                        }, // Simplified
                        contour: get_bool(post.contour, "contour", false),
                    };

                    println!("\n  ⟩ {} — {}", name, params.title);

                    if *dry_run {
                        println!(
                            "    (dry run) layout={} platform={} theme={}",
                            params.layout, params.platform, params.theme
                        );
                        continue;
                    }

                    let content = match do_generate(&params, Some(&cfg)) {
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
                        format!("{}_{{p}}.png", name)
                    } else {
                        format!("{}.png", name)
                    };
                    let output_path = Path::new(output_dir).join(&out_filename);

                    match run_typst_compile(&root, &Path::new(&typ_file), &output_path, ppi) {
                        Ok(_) => println!("    ✓ Compiled"),
                        Err(_) => println!("    ✗ Compilation failed"),
                    }
                }
            }
        }
    }
    Ok(())
}

// Logic extracted to reuse
fn do_generate(params: &GenerateParams, cfg: Option<&config::Config>) -> Result<String> {
    let mut final_accent = params.accent.clone();

    // Auto accent handling
    if params.auto_accent {
        if let Some(img_path) = &params.image {
            // Try to suggest from image
            final_accent = colors::suggest_accent(img_path);
        } else {
            // Or fallback to theme accent if no image
            // For now just keep default
        }
    }

    // Theme resolution
    let palettes = themes::predefined::get_theme_palettes(cfg);
    let theme_map = if let Some(map) = palettes.get(&params.theme) {
        map.clone()
    } else if params.theme == "auto" {
        if let Some(path) = &params.image {
            match colors::extract_from_image(path, 8) {
                Ok(extracted) => {
                    let base = format!(
                        "#{:02x}{:02x}{:02x}",
                        extracted[0].r, extracted[0].g, extracted[0].b
                    );
                    colors::generate_palette(&base)
                }
                Err(_) => palettes.get("dark").unwrap().clone(),
            }
        } else {
            palettes.get("dark").unwrap().clone()
        }
    } else {
        palettes.get("dark").unwrap().clone()
    };

    // Start building the Typst file
    let mut out = String::new();
    out.push_str("// Auto-generado por rrss-cli-rs\n");
    out.push_str("#import \"lib.typ\": *\n");

    // Clean platform import
    out.push_str(&format!(
        "#import \"templates/{}.typ\": {}\n\n",
        params.platform, params.platform
    ));

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
    out.push_str(")\n\n");

    // Start Template wrapper
    out.push_str(&format!("#{}(theme: t)[\n", params.platform));

    // Start Layout call
    out.push_str(&format!("  #{}-layout(\n", params.layout));
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
        if params.layout == "quote" {
            out.push_str(&format!("    quote-text: \"{}\",\n", params.quote));
        } else {
            // Keep backwards compatibility with variable naming in layouts if needed
            // Most laytous seem to use `quote-text` based on user's oceano.typ
            out.push_str(&format!("    quote-text: \"{}\",\n", params.quote));
        }
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
    // Accent is now handled within the #let t = (...) constructor directly
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

    out.push_str("  )\n");
    out.push_str("]\n");

    Ok(out)
}

// Security fix: Command injection prevention
fn run_typst_compile(root: &Path, input: &Path, output: &Path, ppi: u32) -> Result<()> {
    let status = Command::new("typst")
        .arg("compile")
        .arg("--root")
        .arg(root)
        .arg("--ppi")
        .arg(ppi.to_string())
        .arg(input)
        .arg(output)
        .status()
        .context("Failed to execute typst command")?;

    if status.success() {
        Ok(())
    } else {
        Err(anyhow!("Typst compilation failed"))
    }
}
