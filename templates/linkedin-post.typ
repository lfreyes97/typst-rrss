// =============================================================================
// TEMPLATE: LinkedIn Post (1200×627 px)
// =============================================================================

#import "../lib/theme.typ": *
#import "../lib/layouts.typ": *
#import "../lib/elements.typ": *

/// Crea una imagen lista para LinkedIn Feed.
///
/// - palette-name (str): Nombre del tema
/// - body (content): Contenido del post
/// -> document
#let linkedin-post(palette-name: "dark", body) = {
  let dims = platforms.at("linkedin-post")
  let t = theme(palette-name)

  set page(
    width: dims.width,
    height: dims.height,
    margin: 0pt,
    fill: t.bg,
  )

  set text(font: fonts.body.first(), fill: t.text)

  // Decoraciones profesionales sutiles
  deco-circle(t.primary, 80pt, -20pt, -20pt)
  deco-circle(t.secondary, 60pt, dims.width - 80pt, dims.height - 80pt)

  // Barra superior de acento
  place(top + left)[
    #rect(width: 100%, height: 4pt, fill: t.primary)
  ]

  body
}
