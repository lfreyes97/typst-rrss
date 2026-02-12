use serde::Deserialize;
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use anyhow::{Context, Result};

#[derive(Debug, Deserialize)]
pub struct Config {
    pub defaults: HashMap<String, toml::Value>,
    pub themes: Option<HashMap<String, HashMap<String, String>>>,
    #[serde(rename = "post")]
    pub posts: Option<Vec<PostConfig>>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct PostConfig {
    pub name: String,
    pub title: Option<String>,
    pub quote: Option<String>,
    pub image: Option<String>,
    pub logo: Option<String>,
    pub overlay: Option<String>,
    pub accent: Option<String>,
    pub brand: Option<String>,
    pub url: Option<String>,
    pub platform: Option<String>,
    pub layout: Option<String>,
    pub theme: Option<String>,
    pub author: Option<String>,
    pub tag: Option<String>,
    pub ppi: Option<u32>,
    pub slides: Option<Vec<String>>,
    pub contour: Option<bool>,
    pub recolor: Option<bool>,
    pub recolor_intensity: Option<f32>,
    pub source: Option<String>,
}

impl Config {
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Self> {
        let content = fs::read_to_string(path).context("Failed to read config file")?;
        let config: Config = toml::from_str(&content).context("Failed to parse config file")?;
        Ok(config)
    }
}
