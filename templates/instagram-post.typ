// =============================================================================
// TEMPLATE: Instagram Post (1080×1080 px)
// =============================================================================

#import "../lib.typ": *

/// Crea una imagen lista para Instagram Feed (cuadrada).
///
/// - theme (dict): Diccionario de tema (inyectado)
/// - body (content): Contenido del post
/// -> document
#let instagram-post(theme: (:), body) = {
  let dims = platforms.at("instagram-post")
  let t = theme

  set page(
    width: dims.width,
    height: dims.height,
    margin: 0pt,
    fill: t.bg,
  )

  set text(font: fonts.body.first(), fill: t.text, size: 36pt)

  // Decoraciones de fondo
  deco-circle(t.primary, 120pt, -40pt, -40pt)
  deco-circle(t.secondary, 80pt, dims.width - 100pt, dims.height - 120pt)

  body
}
