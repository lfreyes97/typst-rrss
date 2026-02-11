// =============================================================================
// THEME — Sistema de diseño para imágenes de redes sociales
// =============================================================================

// ─── Paletas de colores ──────────────────────────────────────────────────────

#let palettes = (
  dark: (
    bg:        rgb("#0f0f0f"),
    surface:   rgb("#1a1a2e"),
    primary:   rgb("#e94560"),
    secondary: rgb("#533483"),
    accent:    rgb("#0f3460"),
    text:      rgb("#f5f5f5"),
    muted:     rgb("#a0a0a0"),
    highlight: rgb("#ffd369"),
  ),
  light: (
    bg:        rgb("#fafafa"),
    surface:   rgb("#ffffff"),
    primary:   rgb("#e94560"),
    secondary: rgb("#6c5ce7"),
    accent:    rgb("#00b894"),
    text:      rgb("#1a1a2e"),
    muted:     rgb("#6b7280"),
    highlight: rgb("#fdcb6e"),
  ),
  ocean: (
    bg:        rgb("#0a192f"),
    surface:   rgb("#112240"),
    primary:   rgb("#64ffda"),
    secondary: rgb("#8892b0"),
    accent:    rgb("#233554"),
    text:      rgb("#ccd6f6"),
    muted:     rgb("#8892b0"),
    highlight: rgb("#64ffda"),
  ),
  sunset: (
    bg:        rgb("#1a1a2e"),
    surface:   rgb("#16213e"),
    primary:   rgb("#ff6b6b"),
    secondary: rgb("#feca57"),
    accent:    rgb("#ff9ff3"),
    text:      rgb("#f5f5f5"),
    muted:     rgb("#a0a0a0"),
    highlight: rgb("#feca57"),
  ),
  forest: (
    bg:        rgb("#1b2d1b"),
    surface:   rgb("#2d4a2d"),
    primary:   rgb("#a8e6cf"),
    secondary: rgb("#dcedc1"),
    accent:    rgb("#ffd3b6"),
    text:      rgb("#f0f0f0"),
    muted:     rgb("#98b898"),
    highlight: rgb("#ffaaa5"),
  ),
)

/// Obtener una paleta por nombre.
/// - name (str): Nombre de la paleta (dark, light, ocean, sunset, forest)
/// -> dictionary
#let theme(name) = {
  palettes.at(name, default: palettes.dark)
}

// ─── Tipografía ──────────────────────────────────────────────────────────────

#let fonts = (
  heading: ("Fira Sans",),
  body:    ("Fira Sans",),
  mono:    ("Fira Mono",),
)

#let sizes = (
  hero:      48pt,
  title:     36pt,
  subtitle:  24pt,
  body:      18pt,
  caption:   14pt,
  small:     12pt,
  stat:      72pt,
  stat-xl:   96pt,
)

// ─── Dimensiones de página por plataforma ────────────────────────────────────
// Todas las dimensiones están en pt para 144 PPI:
// 1px ≈ 0.5pt a 144 PPI

#let platforms = (
  instagram-post:  (width: 540pt,  height: 540pt),   // 1080×1080
  instagram-story: (width: 540pt,  height: 960pt),   // 1080×1920
  facebook-post:   (width: 600pt,  height: 315pt),   // 1200×630
  twitter-post:    (width: 800pt,  height: 450pt),   // 1600×900
  linkedin-post:   (width: 600pt,  height: 313.5pt), // 1200×627
  og-image:        (width: 600pt,  height: 315pt),   // 1200×630
)

// ─── Espaciado ───────────────────────────────────────────────────────────────

#let spacing = (
  xs:  8pt,
  sm:  12pt,
  md:  20pt,
  lg:  32pt,
  xl:  48pt,
  xxl: 64pt,
)
