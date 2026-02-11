// =============================================================================
// TEMPLATE: Facebook Post (1200×630 px)
// =============================================================================

#import "../lib/theme.typ": *
#import "../lib/layouts.typ": *
#import "../lib/elements.typ": *

/// Crea una imagen lista para Facebook Feed (horizontal).
///
/// - palette-name (str): Nombre del tema
/// - body (content): Contenido del post
/// -> document
#let facebook-post(palette-name: "dark", body) = {
  let dims = platforms.at("facebook-post")
  let t = theme(palette-name)

  set page(
    width: dims.width,
    height: dims.height,
    margin: 0pt,
    fill: t.bg,
  )

  set text(font: fonts.body.first(), fill: t.text)

  // Decoraciones horizontales
  deco-circle(t.primary, 100pt, -30pt, -30pt)
  deco-circle(t.secondary, 70pt, dims.width - 90pt, dims.height - 100pt)

  body
}
