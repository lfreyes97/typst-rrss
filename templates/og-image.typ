// =============================================================================
// TEMPLATE: OG Image / Open Graph (1200×630 px)
// =============================================================================

#import "../lib/theme.typ": *
#import "../lib/layouts.typ": *
#import "../lib/elements.typ": *

/// Crea una imagen Open Graph (link preview).
///
/// - palette-name (str): Nombre del tema
/// - body (content): Contenido de la imagen
/// -> document
#let og-image(palette-name: "dark", body) = {
  let dims = platforms.at("og-image")
  let t = theme(palette-name)

  set page(
    width: dims.width,
    height: dims.height,
    margin: 0pt,
    fill: t.bg,
  )

  set text(font: fonts.body.first(), fill: t.text)

  // Decoraciones para link previews
  deco-circle(t.primary, 100pt, -30pt, -30pt)
  deco-circle(t.secondary, 70pt, dims.width - 90pt, dims.height - 100pt)

  // Borde inferior de marca
  place(bottom + left)[
    #rect(width: 100%, height: 5pt, fill: t.primary)
  ]

  body
}
