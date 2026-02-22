pub mod predefined {
    use std::collections::HashMap;

    pub fn get_theme_palettes(
        config: Option<&crate::config::Config>,
    ) -> HashMap<String, HashMap<String, String>> {
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

        // Restored: Ocean
        let mut ocean = HashMap::new();
        ocean.insert("bg".to_string(), "#0a192f".to_string());
        ocean.insert("surface".to_string(), "#112240".to_string());
        ocean.insert("primary".to_string(), "#64ffda".to_string());
        ocean.insert("secondary".to_string(), "#8892b0".to_string());
        ocean.insert("accent".to_string(), "#233554".to_string());
        ocean.insert("text".to_string(), "#ccd6f6".to_string());
        ocean.insert("muted".to_string(), "#8892b0".to_string());
        ocean.insert("highlight".to_string(), "#64ffda".to_string());
        m.insert("ocean".to_string(), ocean);

        // Restored: Sunset
        let mut sunset = HashMap::new();
        sunset.insert("bg".to_string(), "#1a1a2e".to_string());
        sunset.insert("surface".to_string(), "#16213e".to_string());
        sunset.insert("primary".to_string(), "#ff6b6b".to_string());
        sunset.insert("secondary".to_string(), "#feca57".to_string());
        sunset.insert("accent".to_string(), "#ff9ff3".to_string());
        sunset.insert("text".to_string(), "#f5f5f5".to_string());
        sunset.insert("muted".to_string(), "#a0a0a0".to_string());
        sunset.insert("highlight".to_string(), "#feca57".to_string());
        m.insert("sunset".to_string(), sunset);

        // Restored: Forest
        let mut forest = HashMap::new();
        forest.insert("bg".to_string(), "#1b2d1b".to_string());
        forest.insert("surface".to_string(), "#2d4a2d".to_string());
        forest.insert("primary".to_string(), "#a8e6cf".to_string());
        forest.insert("secondary".to_string(), "#dcedc1".to_string());
        forest.insert("accent".to_string(), "#ffd3b6".to_string());
        forest.insert("text".to_string(), "#f0f0f0".to_string());
        forest.insert("muted".to_string(), "#98b898".to_string());
        forest.insert("highlight".to_string(), "#ffaaa5".to_string());
        m.insert("forest".to_string(), forest);

        m
    }
}
