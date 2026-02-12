use clap::{Parser, Subcommand, ValueEnum};
use std::path::{Path, PathBuf};
use anyhow::Result;
use rrss_cli_rs::{colors, config, constants, templates, images};
use std::fs;
use std::process::Command;
use std::collections::HashMap;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Genera un esquema de colores a partir de un color hex base
    Colors {
        /// Color hexadecimal base (ej: #FF0000)
        base_color: String,
        
        /// Formato de salida
        #[arg(short, long, default_value = "table")]
        format: OutputFormat,
        
        /// Nombre de la paleta (para formato typst)
        #[arg(short, long, default_value = "custom")]
        name: String,
    },
    
    /// Extrae colores dominantes de una imagen
    Extract {
        /// Ruta a la imagen
        image: PathBuf,
        
        /// Cantidad de colores a extraer
        #[arg(short, long, default_value_t = 8)]
        count: u8,
        
        /// Formato de salida
        #[arg(short, long, default_value = "table")]
        format: ExtractFormat,
        
        /// Sugerir el mejor color de acento
        #[arg(long)]
        suggest_accent: bool,
        
        /// Nombre de la paleta (para formato palette)
        #[arg(short, long, default_value = "extracted")]
        name: String,
    },
    
    /// Genera main.typ con los datos proporcionados
    Generate {
        #[arg(short, long, default_value = "Presuposicionalismo")]
        brand: String,
        
        #[arg(short, long)]
        title: String,
        
        #[arg(short, long)]
        quote: String,
        
        #[arg(short, long)]
        image: Option<String>,
        
        #[arg(short, long, default_value = "#4a3f6b")]
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
    
    /// Pipeline completo: genera main.typ + compila a PNG
    Full {
        #[arg(short, long, default_value = "Presuposicionalismo")]
        brand: String,
        
        #[arg(short, long)]
        title: String,
        
        #[arg(short, long)]
        quote: String,
        
        #[arg(short, long)]
        image: Option<String>,
        
        #[arg(short, long, default_value = "#4a3f6b")]
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
        tag: Option<String>,
        
        #[arg(long, default_value_t = 144)]
        ppi: u32,
        
        #[arg(short, long, default_value = "main")]
        output_name: String,
    },
    
    /// Genera posts desde un archivo TOML declarativo
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

fn main() -> Result<()> {
    let cli = Cli::parse();
    
    let root = constants::project_root();

    match &cli.command {
        Commands::Colors { base_color, format, name } => {
            let palette = colors::generate_palette(base_color);
            match format {
                OutputFormat::Json => {
                    println!("{}", serde_json::to_string_pretty(&palette)?);
                }
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
                    let mut keys: Vec<_> = palette.keys().collect();
                    keys.sort();
                    for key in keys {
                        println!("{}: {}", key, palette[key]);
                    }
                }
            }
        }
        Commands::Extract { image, count, format, suggest_accent, name: _ } => {
            println!("Analyzing {:?}...", image);
            if *suggest_accent {
                 let accent = colors::suggest_accent(image.to_str().unwrap());
                 println!("Suggested accent: {}", accent);
                 return Ok(());
            }

            let extracted = colors::extract_from_image(image.to_str().unwrap(), *count)?;

            match format {
                 ExtractFormat::Json => {
                     // Need a serializable struct for this if we want full JSON output
                     println!("JSON output not fully implemented yet");
                 }
                 ExtractFormat::Palette => {
                     let base = format!("#{:02x}{:02x}{:02x}", extracted[0].r, extracted[0].g, extracted[0].b);
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
        Commands::Generate { brand, title, quote, image, accent, auto_accent, url, platform, layout, theme, author, tag, slides, contour, output } => {
            let mut final_accent = accent.clone();
            if *auto_accent {
                if let Some(img_path) = image {
                     final_accent = colors::suggest_accent(img_path);
                     println!("Auto-extracted accent: {}", final_accent);
                }
            }
            
            // Resolve theme for single generation (using defaults)
            let theme_map = resolve_theme(&theme, image.as_deref(), None);

            // Generate content
            let content = generate_typst_content(
                brand, title, quote, image.as_deref(), &final_accent, url, platform, layout, &theme_map, author, tag.as_deref(), slides.as_deref(), *contour
            )?;
            
            fs::write(output, content)?;
            println!("✓ Generated {} ({}, {})", output, layout, platform);
        }
        Commands::Compile { files, ppi, output_dir, all, template } => {
            let targets = if *all {
                let content_dir = root.join("content");
                let mut found = Vec::new();
                for entry in walkdir::WalkDir::new(content_dir).max_depth(1) {
                    let entry = entry?;
                    if entry.path().extension().map_or(false, |e| e == "typ") {
                        found.push(entry.path().to_path_buf());
                    }
                }
                found.sort();
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
            println!("PPI: {}  Files: {}  Output: {}/", ppi, targets.len(), output_dir);

            for target in targets {
                let name = target.file_stem().unwrap().to_str().unwrap();
                let out_filename = if let Some(tmpl) = template {
                    format!("{}-{}.png", name, tmpl)
                } else {
                    format!("{}.png", name)
                };
                let output_path = Path::new(output_dir).join(&out_filename);

                print!("  ⟩ {}... ", name);
                
                let status = Command::new("typst")
                    .arg("compile")
                    .arg("--root")
                    .arg(&root)
                    .arg(format!("--ppi={}", ppi))
                    .arg(&target)
                    .arg(&output_path)
                    .status();

                match status {
                    Ok(s) if s.success() => {
                        println!("✓");
                    }
                    _ => {
                        println!("✗ Error");
                    }
                }
            }
        }
        Commands::Full { brand, title, quote, image, accent, auto_accent, url, platform, layout, theme, author, tag, ppi, output_name } => {
            let typ_file = format!("{}.typ", output_name);
            
            // Re-use logic from Generate (simplification: calling recursively implies cloning arguments or extracting logic)
            // Ideally we separate logic from CLI command structs.
            // For now, let's just duplicate the generate call logic or extract it.
            let mut final_accent = accent.clone();
            if *auto_accent {
                 if let Some(img_path) = image {
                      final_accent = colors::suggest_accent(img_path);
                 }
            }
             // Resolve theme (using defaults)
             let theme_map = resolve_theme(&theme, image.as_deref(), None);

             let content = generate_typst_content(
                brand, title, quote, image.as_deref(), &final_accent, url, platform, layout, &theme_map, author, tag.as_deref(), None, false
            )?;
            fs::write(&typ_file, content)?;
            
            // Compile
            let output_dir = "output";
            fs::create_dir_all(output_dir)?;
            let output_path = Path::new(output_dir).join(format!("{}.png", output_name));
            
             let status = Command::new("typst")
                    .arg("compile")
                    .arg("--root")
                    .arg(&root)
                    .arg(format!("--ppi={}", ppi))
                    .arg(&typ_file)
                    .arg(&output_path)
                    .status()?;
                    
            if status.success() {
                 println!("✓ Full pipeline success: {}", output_path.display());
            } else {
                 println!("✗ Full pipeline failed");
            }
        }
        Commands::Build { config_file, only, dry_run } => {
            let cfg = config::Config::load(config_file)?;
            let defaults = cfg.defaults.clone();
            
            if let Some(ref posts) = cfg.posts {
                for (i, post) in posts.iter().enumerate() {
                     let name = if post.name.is_empty() { format!("post_{}", i) } else { post.name.clone() };
                     
                     if let Some(only_name) = only {
                         if &name != only_name { continue; }
                     }
                     
                     // Resolving fields with defaults
                     let get_str = |opt: &Option<String>, key: &str, def: &str| -> String {
                         opt.clone().unwrap_or_else(|| {
                             defaults.get(key).and_then(|v| v.as_str()).map(|s| s.to_string()).unwrap_or(def.to_string())
                         })
                     };
                     
                     let get_bool = |opt: Option<bool>, key: &str, def: bool| -> bool {
                         opt.unwrap_or_else(|| {
                             defaults.get(key).and_then(|v| v.as_bool()).unwrap_or(def)
                         })
                     };
                     
                     let get_f32 = |opt: Option<f32>, key: &str, def: f32| -> f32 {
                         opt.unwrap_or_else(|| {
                             defaults.get(key).and_then(|v| v.as_float()).map(|f| f as f32).unwrap_or(def)
                         })
                     };

                     let get_u32 = |opt: Option<u32>, key: &str, def: u32| -> u32 {
                         opt.unwrap_or_else(|| {
                             defaults.get(key).and_then(|v| v.as_integer()).map(|i| i as u32).unwrap_or(def)
                         })
                     };

                     let title = get_str(&post.title, "title", "");
                     let quote = get_str(&post.quote, "quote", "");
                     let brand = get_str(&post.brand, "brand", "Presuposicionalismo");
                     let url = get_str(&post.url, "url", "Presuposicionalismo.com");
                     let platform = get_str(&post.platform, "platform", "instagram-post");
                     let layout = get_str(&post.layout, "layout", "article");
                     let theme = get_str(&post.theme, "theme", "dark");
                     let author = get_str(&post.author, "author", "");
                     let tag = post.tag.clone().or_else(|| defaults.get("tag").and_then(|v| v.as_str()).map(|s| s.to_string()));
                     
                     let mut image = post.image.clone().or_else(|| defaults.get("image").and_then(|v| v.as_str()).map(|s| s.to_string()));
                     let accent_def = get_str(&post.accent, "accent", "#4a3f6b");
                     let auto_accent = accent_def == "auto";
                     
                     let ppi = get_u32(post.ppi, "ppi", 144);
                     let contour = get_bool(post.contour, "contour", false);
                     let recolor = get_bool(post.recolor, "recolor", false);
                     let recolor_intensity = get_f32(post.recolor_intensity, "recolor_intensity", 0.7);
                     
                     // Slides (special handling for list)
                     // If explicit slides, use them. Else check defaults.
                     let slides_str: Option<String> = if let Some(s) = &post.slides {
                         Some(s.join("|"))
                     } else if let Some(val) = defaults.get("slides") {
                         if let Some(arr) = val.as_array() {
                             Some(arr.iter().filter_map(|v| v.as_str()).collect::<Vec<_>>().join("|"))
                         } else {
                             None
                         }
                     } else {
                         None
                     };

                     println!("\n  ⟩ {} — {}", name, title);
                     
                     let mut final_accent = if auto_accent { "#4a3f6b".to_string() } else { accent_def };

                     if *dry_run {
                         println!("    (dry run) layout={} platform={} theme={} image={:?}", layout, platform, theme, image);
                         continue;
                     }
                     
                     // Resolve Theme
                     let theme_map = resolve_theme(&theme, image.as_deref(), Some(&cfg));
                     
                     // Check if it's a preset for recoloring
                     let palettes = constants::get_theme_palettes(Some(&cfg));
                     if palettes.contains_key(&theme) && recolor {
                          if let Some(img_path) = &image {
                               println!("    🎨 Recoloring with theme {} (intensity: {})", theme, recolor_intensity);
                               match images::recolor_image(img_path, &theme, &theme_map, None, recolor_intensity) {
                                   Ok(new_path) => image = Some(new_path),
                                   Err(e) => println!("    ⚠ Failed to recolor: {}", e),
                               }
                          }
                     }

                     let final_theme_map = theme_map;
                     
                     // Update final_accent from map if it was "auto", or keep it if it was overridden?
                     // If auto_accent was requested, we might want to pick from the map's accent/primary.
                     if auto_accent {
                         if let Some(img_path) = &image {
                              // If we extracted, we probably have a good accent in the map.
                              // But let's keep the suggest_accent logic for now as it uses color_thief specific logic.
                              final_accent = colors::suggest_accent(img_path);
                         } else {
                              final_accent = final_theme_map.get("accent").unwrap_or(&"#4a3f6b".to_string()).clone();
                         }
                     }

                     // Generate .typ
                     let typ_file = format!("{}.typ", name);
                     match generate_typst_content(
                         &brand, &title, &quote, image.as_deref(), &final_accent, &url, &platform, &layout, &final_theme_map, &author, tag.as_deref(), slides_str.as_deref(), contour
                     ) {
                         Ok(content) => {
                             if let Err(e) = fs::write(&typ_file, content) {
                                 println!("    ✗ Error writing file: {}", e);
                                 continue;
                             }
                         },
                         Err(e) => {
                             println!("    ✗ Error generating content: {}", e);
                             continue;
                         }
                     }
                     
                     // Compile
                     let output_dir = "output";
                     fs::create_dir_all(output_dir)?;
                     let template_pattern = if layout == "carousel" { Some("{0p}".to_string()) } else { None };
                     
                     let out_filename = if let Some(tmpl) = &template_pattern {
                        format!("{}-{}.png", name, tmpl)
                     } else {
                        format!("{}.png", name)
                     };
                     
                     let output_path = Path::new(output_dir).join(&out_filename);
                     
                     let status = Command::new("typst")
                        .arg("compile")
                        .arg("--root")
                        .arg(&root)
                        .arg(format!("--ppi={}", ppi))
                        .arg(&typ_file)
                        .arg(&output_path)
                        .status();
                        
                     match status {
                         Ok(s) if s.success() => println!("    ✓ Compiled"),
                         _ => println!("    ✗ Compilation failed"),
                     }
                }
            }
        }
    }

    Ok(())
}

// This tool call is just for main.rs, will do templates.rs in next step or parallel.

fn generate_typst_content(
    brand: &str, title: &str, quote: &str, image: Option<&str>, accent: &str, url: &str, 
    platform: &str, layout: &str, theme: &HashMap<String, String>, author: &str, tag: Option<&str>, slides: Option<&str>, contour: bool
) -> Result<String> {
    let bg_image_line = if let Some(img) = image {
        format!("bg-image: image(\"{}\", width: 100%),", img)
    } else {
        "// sin imagen de fondo".to_string()
    };
    
    // Construct Typst dictionary for theme
    let mut theme_str = "(".to_string();
    let mut keys: Vec<_> = theme.keys().collect();
    keys.sort(); // Deterministic output
    for k in keys {
        let v = theme.get(k).unwrap();
        // Assuming values are hex strings like "#123456"
        theme_str.push_str(&format!("{}: rgb(\"{}\"), ", k, v));
    }
    theme_str.push(')');

    // Simple template replacement
    let content = match layout {
        "article" => templates::MAIN_TEMPLATE_ARTICLE
            .replace("{platform}", platform)
            .replace("{theme}", &theme_str)
            .replace("{brand}", brand)
            .replace("{title}", title)
            .replace("{quote}", quote)
            .replace("{bg_image_line}", &bg_image_line)
            .replace("{accent}", accent)
            .replace("{url}", url),
        "quote" => templates::MAIN_TEMPLATE_QUOTE
             .replace("{platform}", platform)
             .replace("{theme}", &theme_str)
             .replace("{quote}", quote)
             .replace("{author}", if author.is_empty() { brand } else { author })
             .replace("{brand}", brand),
        "hero" => templates::MAIN_TEMPLATE_HERO
             .replace("{platform}", platform)
             .replace("{theme}", &theme_str)
             .replace("{title}", title)
             .replace("{quote}", quote)
             .replace("{tag_line}", &tag.map(|t| format!("tag: \"{}\",", t)).unwrap_or("// sin tag".to_string()))
             .replace("{brand}", brand),
         "carousel" => {
              // Handle slides parsing if string
              let slides_content = if let Some(s) = slides {
                  if s.contains('|') {
                      let list: Vec<String> = s.split('|').map(|x| format!("\"{}\"", x.trim())).collect();
                      format!("({},)", list.join(", "))
                  } else {
                      s.to_string()
                  }
              } else {
                  "()".to_string()
              };
               
              templates::MAIN_TEMPLATE_CAROUSEL
                .replace("{platform}", platform)
                .replace("{theme}", &theme_str)
                .replace("{title}", title)
                .replace("{slides_content}", &slides_content)
                .replace("{bg_image_line}", &if contour {
                     // Contour logic would go here, simplistic fallback
                     if let Some(img) = image {
                         format!("image(\"{}\", width: 100%)", img)
                     } else {
                         "none".to_string()
                     }
                } else if let Some(img) = image {
                     format!("image(\"{}\", width: 100%)", img)
                } else {
                     "none".to_string()
                })
         },
         _ => return Err(anyhow::anyhow!("Unknown layout: {}", layout)),
    };
    
    Ok(content)
}

fn resolve_theme(
    theme_name: &str, 
    image_path: Option<&str>, 
    cfg: Option<&config::Config>
) -> HashMap<String, String> {
    let palettes = constants::get_theme_palettes(cfg);
    
    // 1. Try to get named theme
    if let Some(map) = palettes.get(theme_name) {
        return map.clone();
    }
    
    // 2. Auto / Extraction mode
    if theme_name == "auto" {
        if let Some(path) = image_path {
            println!("    🎨 Extracting palette from image (auto)...");
            if let Ok(colors) = colors::extract_from_image(path, 8) {
                 let mut dyn_map: HashMap<String, String> = HashMap::new();
                 let bg = colors[0]; 
                 
                 dyn_map.insert("bg".to_string(), format!("#{:02x}{:02x}{:02x}", bg.r, bg.g, bg.b));
                 dyn_map.insert("surface".to_string(), format!("#{:02x}{:02x}{:02x}", bg.r, bg.g, bg.b)); // Simplify for now
                 dyn_map.insert("text".to_string(), "#ffffff".to_string());
                 
                 let primary = if colors.len() > 1 { colors[1] } else { bg };
                 dyn_map.insert("primary".to_string(), format!("#{:02x}{:02x}{:02x}", primary.r, primary.g, primary.b));
                 
                 let secondary = if colors.len() > 2 { colors[2] } else { primary };
                 dyn_map.insert("secondary".to_string(), format!("#{:02x}{:02x}{:02x}", secondary.r, secondary.g, secondary.b));
                 
                 let accent = if colors.len() > 3 { colors[3] } else { secondary };
                 dyn_map.insert("accent".to_string(), format!("#{:02x}{:02x}{:02x}", accent.r, accent.g, accent.b));
                 
                 return dyn_map;
            } else {
                 println!("    ⚠ Failed to extract colors, falling back to dark.");
            }
        }
    }

    // 3. Fallback to dark
    palettes.get("dark").unwrap_or_else(|| {
        // Should not happen if constants.rs is correct
        panic!("Default dark theme missing");
    }).clone()
}
