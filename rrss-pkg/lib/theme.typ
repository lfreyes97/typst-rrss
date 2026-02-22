// =============================================================================
// THEME — Sistema de diseño para imágenes de redes sociales
// =============================================================================

// Nota: Las paletas (t) son inyectadas dinámicamente por la CLI rrss-cli-rs
// al inicio de cada archivo Typst generado, asegurando sincronización total.


// ─── Tipografía ──────────────────────────────────────────────────────────────

#let fonts = (
  heading: ("Fira Sans",),
  body: ("Fira Sans",),
  mono: ("Fira Mono",),
)

#let sizes = (
  hero: 3em,
  title: 2em,
  subtitle: 1.5em,
  body: 1em,
  caption: 0.8em,
  small: 0.7em,
  stat: 4em,
  stat-xl: 6em,
)

// ─── Dimensiones de página por plataforma ────────────────────────────────────
// Todas las dimensiones están en pt para 144 PPI:
// 1px ≈ 0.5pt a 144 PPI

#let platforms = (
  instagram-post: (width: 1080pt, height: 1080pt), // 1080×1080 (1:1)
  instagram-carousel: (width: 1080pt, height: 1350pt), // 1080×1350 (4:5)
  instagram-story: (width: 1080pt, height: 1920pt), // 1080×1920 (9:16)
  facebook-post: (width: 600pt, height: 315pt), // 1200×630
  twitter-post: (width: 800pt, height: 450pt), // 1600×900
  linkedin-post: (width: 600pt, height: 313.5pt), // 1200×627
  og-image: (width: 600pt, height: 315pt), // 1200×630
)

// ─── Espaciado ───────────────────────────────────────────────────────────────

#let spacing = (
  xs: 0.5em,
  sm: 0.75em,
  md: 1.25em,
  lg: 2em,
  xl: 3em,
  xxl: 4em,
)
// ─── Resolución de Fuentes ──────────────────────────────────────────────────

/// Resuelve las fuentes a usar basándose en el tema y los valores por defecto.
/// - t (dictionary): Diccionario del tema.
/// -> dictionary
#let resolve-fonts(t) = {
  let f_heading = t.at("font-heading", default: fonts.heading.first())
  let f_body = t.at("font-body", default: fonts.body.first())
  let f_mono = t.at("font-mono", default: fonts.mono.first())

  (
    heading: (f_heading,),
    body: (f_body,),
    mono: (f_mono,),
  )
}

// ─── Función de Configuración de Dimensiones ──────────────────────────────────

/// Configura las dimensiones y propiedades de la página para el post.
/// Pensada para usarse con #show: set-dimensions.with(platform: "...", theme: t)
///
/// - platform (str): Nombre de la plataforma (ej. "instagram-post")
/// - theme (dictionary): Paleta de colores del tema
/// - body (content): Contenido del documento (inyectado por Typst)
#let set-dimensions(platform: "instagram-post", theme: (:), body) = {
  let dims = platforms.at(platform, default: platforms.instagram-post)
  let f = resolve-fonts(theme)

  set page(
    width: dims.width,
    height: dims.height,
    margin: 0pt,
    fill: theme.bg,
  )

  set text(
    font: f.body.first(),
    fill: theme.text,
    size: 36pt,
  )

  body
}
