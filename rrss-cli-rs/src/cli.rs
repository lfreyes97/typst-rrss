use clap::{ArgAction, Args, Parser, Subcommand, ValueEnum};
use std::path::PathBuf;

/// Produce piezas para redes sociales con los presets de instatypst.
///
/// La vía principal: `init` escribe un posts.toml de partida y `build` lo compila.
#[derive(Parser)]
#[command(
    author,
    version,
    about,
    disable_help_flag = true,
    disable_version_flag = true,
    disable_help_subcommand = true
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
    /// Muestra la ayuda
    #[arg(short, long, action = ArgAction::Help, global = true, display_order = 999)]
    help: Option<bool>,
    /// Muestra la versión
    #[arg(short = 'V', long, action = ArgAction::Version)]
    version: Option<bool>,
}

// El orden de las variantes es el de `--help`: primero la vía principal.
#[derive(Subcommand)]
pub enum Commands {
    /// Genera y compila los posts de un posts.toml
    Build {
        /// posts.toml que se compila
        #[arg(default_value = "posts.toml")]
        config_file: PathBuf,
        /// Compila solo el post con este `name` (el resto se valida igual)
        #[arg(long, value_name = "NOMBRE")]
        solo: Option<String>,
        /// Valida y muestra qué se haría, sin compilar nada
        #[arg(long)]
        dry_run: bool,
        /// Formato de las imágenes
        #[arg(long, default_value = "png")]
        imagen: ImageFormat,
    },

    /// Escribe un punto de partida editable: un posts.toml o un .typ
    Init {
        /// Presets de instatypst (por defecto, `liso`). Un .typ admite uno solo
        presets: Vec<String>,
        /// Archivo de salida: si acaba en .typ, se escribe la pieza en Typst;
        /// si no, un posts.toml
        #[arg(short, long, value_name = "ARCHIVO", default_value = "posts.toml")]
        output: PathBuf,
        /// Sobrescribe el archivo si ya existe
        #[arg(long)]
        force: bool,
    },

    /// Genera y compila una pieza desde flags (el .typ queda en output/typ/)
    Full {
        #[command(flatten)]
        pieza: PiezaArgs,
        /// Nombre de la pieza: output/<nombre>.png (admite subcarpetas)
        #[arg(short, long, value_name = "NOMBRE", default_value = "pieza")]
        output: String,
        /// Formato de la imagen
        #[arg(long, default_value = "png")]
        imagen: ImageFormat,
    },

    /// Genera el .typ de una pieza desde flags, sin compilarlo
    Generate {
        #[command(flatten)]
        pieza: PiezaArgs,
        /// Archivo .typ de salida
        #[arg(short, long, value_name = "ARCHIVO", default_value = "pieza.typ")]
        output: PathBuf,
    },

    /// Compila archivos .typ ya escritos a PNG/WebP
    Compile {
        /// Archivos .typ que se compilan
        files: Vec<PathBuf>,
        /// Resolución: 72 da 1 px por pt (entre 72 y 1200)
        #[arg(long, default_value_t = 72)]
        ppi: u32,
        /// Carpeta de las imágenes
        #[arg(short, long, value_name = "CARPETA", default_value = "output")]
        output: String,
        /// Formato de las imágenes
        #[arg(long, default_value = "png")]
        imagen: ImageFormat,
    },

    /// Extrae una paleta de colores de una imagen
    Extract {
        /// Imagen de la que se extraen los colores
        image: PathBuf,
        /// Cuántos colores extraer (2–255)
        #[arg(short, long, value_name = "N", default_value_t = 8, value_parser = clap::value_parser!(u8).range(2..))]
        count: u8,
        /// Salida: lista de colores, JSON o paleta armónica desde el dominante
        #[arg(short, long, default_value = "table")]
        format: ExtractFormat,
        /// Solo sugiere un color de acento para la imagen
        #[arg(long)]
        suggest_accent: bool,
    },

    /// Genera una paleta armónica desde un color base
    Colors {
        /// Color base en #RRGGBB
        base_color: String,
        /// Salida: tabla, JSON o diccionario de Typst
        #[arg(short, long, default_value = "table")]
        format: OutputFormat,
        /// Nombre del diccionario con --format typst
        #[arg(short, long, default_value = "custom")]
        name: String,
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
    /// Resolución: 72 da 1 px por pt (entre 72 y 1200; por defecto, 72)
    #[arg(long)]
    pub ppi: Option<u32>,
    /// Argumento con nombre del preset. El valor se lee como TOML y, si no lo es,
    /// como texto: `--arg peso=500`, `--arg marca=@yo`, `--arg 'fondo={color="#691322"}'`
    #[arg(long = "arg", value_name = "CLAVE=VALOR")]
    pub args: Vec<String>,
    /// Carpeta de fuentes extra (se repite)
    #[arg(long = "font-path", value_name = "CARPETA")]
    pub font_paths: Vec<PathBuf>,
    /// posts.toml del que tomar [themes] y `fonts` de [defaults]
    #[arg(long)]
    pub config: Option<PathBuf>,
}
