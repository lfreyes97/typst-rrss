// =============================================================================
// LAYOUT: Quote
// =============================================================================

#import "../theme.typ": *

/// Layout de cita — texto con comillas decorativas y autor.
///
/// - t (dictionary): Paleta de tema
/// - quote-text (str): Texto de la cita
/// - author (str): Autor de la cita
/// - source (str): Fuente opcional (libro, discurso, etc.)
/// -> content
#let quote-layout(t, quote-text: "", author: "", source: none, ..args) = {
  set align(center + horizon)
  block(
    width: 100%,
    height: 100%,
    fill: t.bg,
    inset: spacing.xl,
  )[
    #set text(fill: t.text, font: fonts.heading.first())

    // Comilla decorativa de apertura
    #text(size: 5em, fill: t.primary, weight: "black")["]

    #v(-spacing.lg)

    // Texto de la cita
    #block(width: 85%)[
      #set text(size: sizes.subtitle, weight: "medium", style: "italic")
      #set par(leading: 0.85em)
      #quote-text
    ]

    #v(spacing.lg)

    // Línea decorativa
    #line(length: 4em, stroke: 0.15em + t.primary)

    #v(spacing.md)

    // Autor
    #text(size: sizes.body, weight: "bold", fill: t.primary)[#author]

    #if source != none {
      v(spacing.xs)
      text(size: sizes.caption, fill: t.muted)[#source]
    }
  ]
}
