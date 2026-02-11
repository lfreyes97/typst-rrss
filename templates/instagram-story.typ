// =============================================================================
// TEMPLATE: Instagram Story (1080×1920 px)
// =============================================================================

#import "../lib/theme.typ": *
#import "../lib/layouts.typ": *
#import "../lib/elements.typ": *

/// Crea una imagen lista para Instagram Stories (vertical 9:16).
///
/// - palette-name (str): Nombre del tema
/// - body (content): Contenido del story
/// -> document
#let instagram-story(palette-name: "dark", body) = {
  let dims = platforms.at("instagram-story")
  let t = theme(palette-name)

  set page(
    width: dims.width,
    height: dims.height,
    margin: 0pt,
    fill: t.bg,
  )

  set text(font: fonts.body.first(), fill: t.text)

  // Decoraciones de fondo verticales
  deco-circle(t.primary, 150pt, -60pt, -60pt)
  deco-circle(t.accent, 100pt, dims.width - 80pt, 200pt)
  deco-circle(t.secondary, 120pt, -30pt, dims.height - 200pt)

  body
}
