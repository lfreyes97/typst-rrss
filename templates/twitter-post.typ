// =============================================================================
// TEMPLATE: Twitter/X Post (1600×900 px)
// =============================================================================

#import "../lib.typ": *

/// Crea una imagen lista para Twitter/X (16:9 widescreen).
///
/// - theme (dict): Diccionario de tema (inyectado)
/// - body (content): Contenido del post
/// -> document
#let twitter-post(theme: (:), body) = {
  let dims = platforms.at("twitter-post")
  let t = theme

  set page(
    width: dims.width,
    height: dims.height,
    margin: 0pt,
    fill: t.bg,
  )

  set text(font: fonts.body.first(), fill: t.text, size: 34pt)

  // Decoraciones widescreen
  deco-circle(t.primary, 130pt, -50pt, -50pt)
  deco-circle(t.accent, 90pt, dims.width - 120pt, -30pt)
  deco-circle(t.secondary, 100pt, dims.width * 0.5, dims.height - 80pt)

  body
}
