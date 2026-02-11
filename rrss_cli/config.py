
# ─── Plataformas válidas ─────────────────────────────────────────────────────

PLATFORMS = {
    "instagram-post": (1080, 1080),
    "instagram-carousel": (1080, 1350),
    "instagram-story": (1080, 1920),
    "facebook-post": (1200, 630),
    "twitter-post": (1600, 900),
    "linkedin-post": (1200, 627),
    "og-image": (1200, 630),
}

LAYOUTS = ["article", "quote", "hero", "stat", "centered-card", "split"]

# ─── Paletas (espejo de lib/theme.typ) ───────────────────────────────────────

THEME_PALETTES = {
    "dark": {
        "bg": "#0f0f0f", "surface": "#1a1a2e", "primary": "#e94560",
        "secondary": "#533483", "accent": "#0f3460", "text": "#f5f5f5",
        "muted": "#a0a0a0", "highlight": "#ffd369",
    },
    "light": {
        "bg": "#fafafa", "surface": "#ffffff", "primary": "#e94560",
        "secondary": "#6c5ce7", "accent": "#00b894", "text": "#1a1a2e",
        "muted": "#6b7280", "highlight": "#fdcb6e",
    },
    "ocean": {
        "bg": "#0a192f", "surface": "#112240", "primary": "#64ffda",
        "secondary": "#8892b0", "accent": "#233554", "text": "#ccd6f6",
        "muted": "#8892b0", "highlight": "#64ffda",
    },
    "sunset": {
        "bg": "#1a1a2e", "surface": "#16213e", "primary": "#ff6b6b",
        "secondary": "#feca57", "accent": "#ff9ff3", "text": "#f5f5f5",
        "muted": "#a0a0a0", "highlight": "#feca57",
    },
    "forest": {
        "bg": "#1b2d1b", "surface": "#2d4a2d", "primary": "#a8e6cf",
        "secondary": "#dcedc1", "accent": "#ffd3b6", "text": "#f0f0f0",
        "muted": "#98b898", "highlight": "#ffaaa5",
    },
}
