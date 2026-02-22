// =============================================================================
// LAYOUT: Split (Two Columns)
// =============================================================================

#import "../theme.typ": *

/// Layout dividido en dos columnas — ideal para promo con imagen.
///
/// - t (dictionary): Paleta de tema
/// - left (content): Contenido del lado izquierdo
/// - right (content): Contenido del lado derecho
/// - ratio (array): Proporción de las columnas (default 1fr, 1fr)
/// -> content
#let split(t, left, right, ratio: (1fr, 1fr)) = {
  block(
    width: 100%,
    height: 100%,
    fill: t.bg,
  )[
    #grid(
      columns: ratio,
      rows: 100%,
      block(
        width: 100%,
        height: 100%,
        inset: spacing.lg,
        fill: t.bg,
      )[
        #set align(horizon)
        #left
      ],
      block(
        width: 100%,
        height: 100%,
        inset: spacing.lg,
        fill: t.surface,
      )[
        #set align(center + horizon)
        #right
      ],
    )
  ]
}
