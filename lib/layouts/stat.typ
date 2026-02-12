// =============================================================================
// LAYOUT: Stat
// =============================================================================

#import "../theme.typ": *

/// Layout para estadísticas — número grande arriba, texto abajo.
///
/// - t (dictionary): Paleta de tema
/// - number (str): Número o cifra principal
/// - label (str): Descripción de la estadística
/// - source (str): Fuente de la estadística
/// -> content
#let stat-layout(t, number: "", label: "", source: none) = {
  set align(center + horizon)
  block(
    width: 100%,
    height: 100%,
    fill: t.bg,
    inset: spacing.lg,
  )[
    #set text(fill: t.text, font: fonts.heading.first())

    // Número grande
    #text(
      size: sizes.stat,
      weight: "black",
      fill: t.primary,
      number,
    )

    #v(spacing.sm)

    // Etiqueta
    #block(width: 80%)[
      #set text(size: sizes.subtitle, weight: "bold")
      #label
    ]

    #if source != none {
      v(spacing.md)
      set text(size: sizes.caption, fill: t.muted)
      [— #source]
    }
  ]
}
