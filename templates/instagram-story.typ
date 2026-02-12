// =============================================================================
// TEMPLATE: Instagram Story (1080×1920 px)
// =============================================================================

#import "../lib.typ": *

/// Crea una imagen lista para Instagram Stories (vertical 9:16).
///
/// - theme (dict): Diccionario de tema (inyectado)
/// - body (content): Contenido del story
/// -> document
#let instagram-story(theme: (:), body) = {
  let dims = platforms.at("instagram-story")
  let t = theme

  set page(
    width: dims.width,
    height: dims.height,
    margin: 0pt,
    fill: t.bg,
  )

  set text(font: fonts.body.first(), fill: t.text, size: 40pt)

  // Decoraciones de fondo verticales
  deco-circle(t.primary, 150pt, -60pt, -60pt)
  deco-circle(t.accent, 100pt, dims.width - 80pt, 200pt)
  deco-circle(t.secondary, 120pt, -30pt, dims.height - 200pt)

  body
}
