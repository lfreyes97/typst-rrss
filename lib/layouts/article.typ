// =============================================================================
// LAYOUT: Article (Presuposicionalismo)
// =============================================================================

#import "../theme.typ": *

/// Layout de artículo con imagen de fondo — estilo Presuposicionalismo.
///
/// - t (dictionary): Paleta de tema
/// - brand (str): Nombre de la marca/logo
/// - title (str): Título del artículo (se muestra en mayúsculas)
/// - quote-text (str): Cita o extracto con guillemets
/// - bg-image (content): Imagen de fondo (pasar `image("ruta")` directamente)
/// - accent (color): Color de acento para la barra del título
/// - url (str): URL del sitio para el footer
/// -> content
#let article-layout(
  t,
  brand: "PRESUPOSICIONALISMO",
  title: "",
  quote-text: "",
  bg-image: none,
  accent: rgb("#4a3f6b"),
  url: "Presuposicionalismo.com",
) = {
  // Porcentajes relativos a la altura de la página
  let header-h = 15%
  let title-zone-h = 45%
  let footer-h = 12%

  // ─── Imagen de fondo (capa base) ──────────────────────────────
  if bg-image != none {
    place(bottom + center, dy: -footer-h)[
      #block(width: 100%, clip: true)[
        #bg-image
      ]
    ]
  }

  // ─── Header: barra oscura con marca ───────────────────────────
  place(top + center)[
    #block(
      width: 100%,
      height: header-h,
      fill: t.surface,
    )[
      #set align(center + horizon)
      #set text(fill: t.text, font: fonts.heading.first())
      #text(size: 1.5em, tracking: 0.1em)[꩜]
      #v(-0.2em)
      #text(size: sizes.caption * 0.9, weight: "bold", tracking: 0.15em)[#upper(brand)]
    ]
  ]

  // ─── Zona de título + cita (con fondo semi-transparente) ──────
  place(top + center, dy: header-h)[
    #block(
      width: 100%,
      height: title-zone-h,
      fill: accent.transparentize(10%),
      inset: (x: spacing.lg, top: spacing.md, bottom: spacing.sm),
    )[
      #set align(center + top)
      #set text(fill: if t.bg == rgb("#fafafa") { t.bg } else { white }, font: fonts.heading.first())

      // Título
      #block(width: 100%)[
        #set text(size: sizes.subtitle * 1.1, weight: "black", tracking: 0.05em)
        #set par(leading: 0.7em)
        #upper(title)
      ]

      #v(spacing.xs)

      // Separador
      #line(length: 90%, stroke: 0.1em + (if t.bg == rgb("#fafafa") { t.bg } else { white }).transparentize(40%))

      #v(spacing.xs)

      // Cita
      #block(width: 92%)[
        #set text(
          size: sizes.body * 0.9,
          weight: "regular",
          fill: (if t.bg == rgb("#fafafa") { t.bg } else { white }).transparentize(5%),
        )
        #set par(leading: 0.7em)
        «#quote-text»
      ]
    ]
  ]

  // ─── Footer: barra con URL ────────────────────────────────────
  place(bottom + center)[
    #block(
      width: 100%,
      height: footer-h,
      fill: t.surface,
    )[
      #set align(center + horizon)
      #set text(fill: t.text, font: fonts.heading.first(), size: sizes.body * 0.8, style: "italic")
      #url
    ]
  ]
}
