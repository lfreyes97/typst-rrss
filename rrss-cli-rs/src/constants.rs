use std::collections::HashMap;
use std::path::PathBuf;

pub fn project_root() -> PathBuf {
    std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."))
}

pub fn get_platforms() -> HashMap<&'static str, (u32, u32)> {
    let mut m = HashMap::new();
    m.insert("instagram-post", (1080, 1080));
    m.insert("instagram-carousel", (1080, 1350));
    m.insert("instagram-story", (1080, 1920));
    m.insert("facebook-post", (1200, 630));
    m.insert("twitter-post", (1600, 900));
    m.insert("linkedin-post", (1200, 627));
    m.insert("og-image", (1200, 630));
    m
}

pub fn get_theme_palettes(config: Option<&crate::config::Config>) -> HashMap<String, HashMap<String, String>> {
    if let Some(cfg) = config {
        if let Some(themes) = &cfg.themes {
            return themes.clone();
        }
    }

    let mut m = HashMap::new();
    
    // Fallback: Dark
    let mut dark = HashMap::new();
    dark.insert("bg".to_string(), "#0f0f0f".to_string());
    dark.insert("surface".to_string(), "#1a1a2e".to_string());
    dark.insert("primary".to_string(), "#e94560".to_string());
    dark.insert("secondary".to_string(), "#533483".to_string());
    dark.insert("accent".to_string(), "#0f3460".to_string());
    dark.insert("text".to_string(), "#f5f5f5".to_string());
    dark.insert("muted".to_string(), "#a0a0a0".to_string());
    dark.insert("highlight".to_string(), "#ffd369".to_string());
    m.insert("dark".to_string(), dark);

    // Fallback: Light
    let mut light = HashMap::new();
    light.insert("bg".to_string(), "#fafafa".to_string());
    light.insert("surface".to_string(), "#ffffff".to_string());
    light.insert("primary".to_string(), "#e94560".to_string());
    light.insert("secondary".to_string(), "#6c5ce7".to_string());
    light.insert("accent".to_string(), "#00b894".to_string());
    light.insert("text".to_string(), "#1a1a2e".to_string());
    light.insert("muted".to_string(), "#6b7280".to_string());
    light.insert("highlight".to_string(), "#fdcb6e".to_string());
    m.insert("light".to_string(), light);

    m
}
