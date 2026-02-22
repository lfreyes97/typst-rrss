// =============================================================================
// LAYOUT: Stat
// =============================================================================

#import "../theme.typ": *
#import "centered.typ": base-card

/// Layout para estadísticas — número grande arriba, texto abajo.
///
/// - t (dictionary): Paleta de tema
/// - number (str): Número o cifra principal
/// - label (str): Descripción de la estadística
/// - source (str): Fuente de la estadística
/// - bg-image (content): Imagen de fondo opcional
/// - overlay (content): Capa sobre la imagen opcional
/// -> content
#let stat(
  t,
  number: "",
  label: "",
  source: none,
  bg-image: none,
  overlay: none,
  ..args,
) = {
  base-card(
    t,
    bg-image: bg-image,
    overlay: overlay,
  )[
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
