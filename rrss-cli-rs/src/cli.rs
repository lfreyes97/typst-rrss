use clap::{Args, Parser, Subcommand, ValueEnum};
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

    /// Genera el .typ de una pieza desde flags, sin compilarlo
    Generate {
        #[command(flatten)]
        pieza: PiezaArgs,
        /// Archivo .typ de salida
        #[arg(short, long, default_value = "pieza.typ")]
        output: PathBuf,
    },

    /// Compila archivos .typ a PNG/WebP
    Compile {
        #[arg(num_args = 0..)]
        files: Vec<PathBuf>,
        #[arg(long, default_value_t = 72)]
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

    /// Genera y compila una pieza desde flags (el .typ queda en output/typ/)
    Full {
        #[command(flatten)]
        pieza: PiezaArgs,
        /// Nombre de la pieza: output/<nombre>.png (admite subcarpetas)
        #[arg(short = 'o', long, default_value = "pieza")]
        output_name: String,
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

/// Una pieza descrita con flags; la comparten `generate` y `full`.
#[derive(Args, Clone)]
pub struct PiezaArgs {
    /// Preset de instatypst: `suizo`, `liso`, `fleurones.orla`…
    pub preset: String,
    /// Argumentos posicionales del preset, en orden (`frase`, `remate`…)
    pub pos: Vec<String>,
    /// Paleta de la CLI, `auto` / `auto-matugen` (desde --image) o un tema de instatypst
    #[arg(long)]
    pub tema: Option<String>,
    /// Imagen de la que se extrae el tema
    #[arg(long)]
    pub image: Option<String>,
    /// feed-4x5, feed-1x1, story, twitter, linkedin o ANCHOxALTO (por defecto: el del preset)
    #[arg(long)]
    pub formato: Option<String>,
    #[arg(long)]
    pub ppi: Option<u32>,
    /// Argumento con nombre del preset. El valor se lee como TOML y, si no lo es,
    /// como texto: `--arg peso=500`, `--arg marca=@yo`, `--arg 'fondo={color="#691322"}'`
    #[arg(long = "arg", value_name = "CLAVE=VALOR")]
    pub args: Vec<String>,
    /// Carpeta de fuentes extra (se repite)
    #[arg(long = "font-path")]
    pub font_paths: Vec<PathBuf>,
    /// posts.toml del que tomar [themes] y `fonts` de [defaults]
    #[arg(long)]
    pub config: Option<PathBuf>,
}
