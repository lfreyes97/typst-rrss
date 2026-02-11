// =============================================================================
// CONTENIDO: Tip / Consejo
// =============================================================================
// Uso: typst compile content/tip.typ output/tip.png
// =============================================================================

#import "../templates/twitter-post.typ": *
#import "../lib/theme.typ": *
#import "../lib/layouts.typ": *
#import "../lib/elements.typ": *

#show: twitter-post.with(palette-name: "ocean")

#let t = theme("ocean")

#set align(center + horizon)

#block(
  width: 100%,
  height: 100%,
  fill: t.bg,
  inset: spacing.xl,
)[
  #set text(fill: t.text, font: fonts.heading.first())

  #badge(t, "Tip", bg: t.primary)

  #v(spacing.md)

  #text(size: sizes.title, weight: "black")[
    5 atajos de Typst que\
    deberías conocer
  ]

  #v(spacing.lg)

  #block(width: 80%)[
    #set align(left)
    #styled-list(t, (
      [Usa `#set` para aplicar estilos globalmente],
      [Importa templates con `#import`],
      [Variables con `#let` para reutilizar valores],
      [Funciones custom con `#let fn(args) = { ... }`],
      [Compila a PNG con `typst compile --format png`],
    ))
  ]
]

#footer-handle(t, "@tu_marca · typst-rrss")
