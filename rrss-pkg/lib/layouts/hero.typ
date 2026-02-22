// =============================================================================
// LAYOUT: Hero
// =============================================================================

#import "../theme.typ": *
#import "centered.typ": base-card

/// Layout hero — texto grande centrado con subtítulo.
///
/// - t (dictionary): Paleta de tema
/// - title (str): Título principal
/// - subtitle (str): Subtítulo
/// - tag (str): Tag/etiqueta opcional
/// - bg-image (content): Imagen de fondo opcional
/// - overlay (content): Capa sobre la imagen opcional
/// -> content
#let hero(
  t,
  title: "",
  subtitle: "",
  tag: none,
  bg-image: none,
  overlay: none,
  ..args,
) = {
  base-card(
    t,
    bg-image: bg-image,
    overlay: overlay,
  )[
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
