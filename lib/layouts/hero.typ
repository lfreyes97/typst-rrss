// =============================================================================
// LAYOUT: Hero
// =============================================================================

#import "../theme.typ": *

/// Layout hero — texto grande centrado con subtítulo.
///
/// - t (dictionary): Paleta de tema
/// - title (str): Título principal
/// - subtitle (str): Subtítulo
/// - tag (str): Tag/etiqueta opcional
/// -> content
#let hero-layout(t, title: "", subtitle: "", tag: none, ..args) = {
  set align(center + horizon)
  block(
    width: 100%,
    height: 100%,
    fill: t.bg,
    inset: spacing.xl,
  )[
    #set text(fill: t.text, font: fonts.heading.first())

    #if tag != none {
      block(
        inset: (x: spacing.md, y: spacing.xs),
        radius: 4pt,
        fill: t.primary,
      )[
        #set text(size: sizes.caption, weight: "bold", fill: white)
        #tag
      ]
      v(spacing.md)
    }

    #block(width: 85%)[
      #set text(size: sizes.hero, weight: "black", fill: t.text)
      #set par(leading: 0.75em)
      #title
    ]

    #v(spacing.md)

    #block(width: 75%)[
      #set text(size: sizes.subtitle, fill: t.muted, weight: "regular")
      #subtitle
    ]
  ]
}
