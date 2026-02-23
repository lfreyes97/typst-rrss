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

        // --- NEW BASE16 THEMES ---

        let map_base16 = |colors: [&str; 16]| {
            let mut theme = HashMap::new();
            theme.insert("bg".to_string(), colors[0].to_string());
            theme.insert("surface".to_string(), colors[1].to_string());
            theme.insert("muted".to_string(), colors[3].to_string());
            theme.insert("text".to_string(), colors[5].to_string());
            theme.insert("primary".to_string(), colors[8].to_string());
            theme.insert("highlight".to_string(), colors[9].to_string());
            theme.insert("accent".to_string(), colors[12].to_string()); // base0C/D
            theme.insert("secondary".to_string(), colors[14].to_string()); // base0E
            theme
        };

        // Nord
        m.insert(
            "nord".to_string(),
            map_base16([
                "#2e3440", "#3b4252", "#434c5e", "#4c566a", "#d8dee9", "#e5e9f0", "#eceff4",
                "#8fbcbb", "#bf616a", "#d08770", "#ebcb8b", "#a3be8c", "#88c0d0", "#81a1c1",
                "#b48ead", "#5e81ac",
            ]),
        );

        // Gruvbox Dark
        m.insert(
            "gruvbox-dark".to_string(),
            map_base16([
                "#282828", "#3c3836", "#504945", "#665c54", "#bdae93", "#d5c4a1", "#ebdbb2",
                "#fbf1c7", "#fb4934", "#fe8019", "#fabd2f", "#b8bb26", "#8ec07c", "#83a598",
                "#d3869b", "#fe8019",
            ]),
        );

        // Catppuccin Mocha
        m.insert(
            "catppuccin-mocha".to_string(),
            map_base16([
                "#1e1e2e", "#181825", "#313244", "#45475a", "#585b70", "#cdd6f4", "#f5e0dc",
                "#b4befe", "#f38ba8", "#fab387", "#f9e2af", "#a6e3a1", "#94e2d5", "#89b4fa",
                "#cba6f7", "#f2cdcd",
            ]),
        );

        // Tokyo Night
        m.insert(
            "tokyo-night".to_string(),
            map_base16([
                "#1a1b26", "#16161e", "#24283b", "#414868", "#565f89", "#cfc9c2", "#a9b1d6",
                "#c0caf5", "#f7768e", "#ff9e64", "#e0af68", "#9ece6a", "#73daca", "#7aa2f7",
                "#bb9af7", "#2ac3de",
            ]),
        );

        // Dracula
        m.insert(
            "dracula".to_string(),
            map_base16([
                "#282a36", "#44475a", "#44475a", "#6272a4", "#6272a4", "#f8f8f2", "#f8f8f2",
                "#ffffff", "#ff5555", "#ffb86c", "#f1fa8c", "#50fa7b", "#8be9fd", "#bd93f9",
                "#ff79c6", "#bd93f9",
            ]),
        );

        // One Dark
        m.insert(
            "one-dark".to_string(),
            map_base16([
                "#282c34", "#353b45", "#3e4451", "#545862", "#565c64", "#abb2bf", "#b6bdca",
                "#c8ccd4", "#e06c75", "#d19a66", "#e5c07b", "#98c379", "#56b6c2", "#61afef",
                "#c678dd", "#be5046",
            ]),
        );

        // Rose Pine
        m.insert(
            "rose-pine".to_string(),
            map_base16([
                "#191724", "#1f1d2e", "#26233a", "#6e6a86", "#908caa", "#e0def4", "#e0def4",
                "#524f67", "#eb6f92", "#f6c177", "#ebbcba", "#31748f", "#9ccfd8", "#c4a7e7",
                "#f6c177", "#524f67",
            ]),
        );

        // Catppuccin Latte (Light)
        m.insert(
            "catppuccin-latte".to_string(),
            map_base16([
                "#eff1f5", "#e6e9ef", "#ccd0da", "#bcc0cc", "#acb0be", "#4c4f69", "#5c5f77",
                "#6c6f85", "#d20f39", "#fe640b", "#df8e1d", "#40a02b", "#179299", "#1e66f5",
                "#8839ef", "#ea76cb",
            ]),
        );

        // Gruvbox Light
        m.insert(
            "gruvbox-light".to_string(),
            map_base16([
                "#fbf1c7", "#ebdbb2", "#d5c4a1", "#bdae93", "#665c54", "#3c3836", "#282828",
                "#1d2021", "#9d0006", "#af3a03", "#b57614", "#79740e", "#427b58", "#076678",
                "#8f3f71", "#af3a03",
            ]),
        );

        // Solarized Dark
        m.insert(
            "solarized-dark".to_string(),
            map_base16([
                "#002b36", "#073642", "#586e75", "#657b83", "#839496", "#93a1a1", "#eee8d5",
                "#fdf6e3", "#dc322f", "#cb4b16", "#b58900", "#859900", "#2aa198", "#268bd2",
                "#6c71c4", "#d33682",
            ]),
        );

        // Solarized Light
        m.insert(
            "solarized-light".to_string(),
            map_base16([
                "#fdf6e3", "#eee8d5", "#93a1a1", "#839496", "#657b83", "#586e75", "#073642",
                "#002b36", "#dc322f", "#cb4b16", "#b58900", "#859900", "#2aa198", "#268bd2",
                "#6c71c4", "#d33682",
            ]),
        );

        m
    }
}
