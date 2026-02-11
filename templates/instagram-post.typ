// =============================================================================
// TEMPLATE: Instagram Post (1080×1080 px)
// =============================================================================

#import "../lib/theme.typ": *
#import "../lib/layouts.typ": *
#import "../lib/elements.typ": *

/// Crea una imagen lista para Instagram Feed (cuadrada).
///
/// - palette-name (str): Nombre del tema ("dark", "light", "ocean", etc.)
/// - body (content): Contenido del post
/// -> document
#let instagram-post(palette-name: "dark", body) = {
  let dims = platforms.at("instagram-post")
  let t = theme(palette-name)

  set page(
    width: dims.width,
    height: dims.height,
    margin: 0pt,
    fill: t.bg,
  )

  set text(font: fonts.body.first(), fill: t.text)

  // Decoraciones de fondo
  deco-circle(t.primary, 120pt, -40pt, -40pt)
  deco-circle(t.secondary, 80pt, dims.width - 100pt, dims.height - 120pt)

  body
}
