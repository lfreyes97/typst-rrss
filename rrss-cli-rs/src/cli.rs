use clap::{Parser, Subcommand, ValueEnum};
use std::path::PathBuf;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
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
        #[arg(long)]
        font_heading: Option<String>,
        #[arg(long)]
        font_body: Option<String>,
        #[arg(long)]
        font_mono: Option<String>,
    },

    /// Compila archivos .typ a PNG/WebP
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
        #[arg(long, default_value = "png")]
        format: ImageFormat,
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
        #[arg(long)]
        font_heading: Option<String>,
        #[arg(long)]
        font_body: Option<String>,
        #[arg(long)]
        font_mono: Option<String>,
        #[arg(long, default_value = "png")]
        format: ImageFormat,
    },

    /// Genera posts desde TOML
    Build {
        #[arg(default_value = "posts.toml")]
        config_file: PathBuf,
        #[arg(short, long)]
        only: Option<String>,
        #[arg(long)]
        dry_run: bool,
        #[arg(long, default_value = "png")]
        format: ImageFormat,
    },
}

#[derive(ValueEnum, Clone)]
pub enum OutputFormat {
    Table,
    Json,
    Typst,
}

#[derive(ValueEnum, Clone)]
pub enum ExtractFormat {
    Table,
    Json,
    Palette,
}

#[derive(ValueEnum, Clone, PartialEq)]
pub enum ImageFormat {
    Png,
    Webp,
}

pub struct GenerateParams {
    pub brand: String,
    pub title: String,
    pub quote: String,
    pub image: Option<String>,
    pub logo: Option<String>,
    pub overlay: Option<String>,
    pub accent: String,
    pub auto_accent: bool,
    pub url: String,
    pub platform: String,
    pub layout: String,
    pub theme: String,
    pub author: String,
    pub source: Option<String>,
    pub tag: Option<String>,
    pub slides: Option<String>,
    pub contour: bool,
    pub font_heading: Option<String>,
    pub font_body: Option<String>,
    pub font_mono: Option<String>,
}
