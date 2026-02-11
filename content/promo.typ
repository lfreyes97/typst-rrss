// =============================================================================
// CONTENIDO: Anuncio / Promoción
// =============================================================================
// Uso: typst compile content/promo.typ output/promo.png
// =============================================================================

#import "../templates/facebook-post.typ": *
#import "../lib/theme.typ": *
#import "../lib/layouts.typ": *
#import "../lib/elements.typ": *

#show: facebook-post.with(palette-name: "sunset")

#let t = theme("sunset")

#split-layout(
  t,
  // Columna izquierda: texto
  [
    #set text(fill: t.text, font: fonts.heading.first())
    #badge(t, "Nuevo lanzamiento")
    #v(spacing.md)

    #text(size: sizes.title, weight: "black")[
      Curso de Diseño\
      con Typst
    ]
    #v(spacing.sm)

    #text(size: sizes.body, fill: t.muted)[
      Aprende a crear imágenes profesionales para redes sociales usando código.
    ]
    #v(spacing.md)

    #box(
      inset: (x: spacing.md, y: spacing.sm),
      radius: 6pt,
      fill: t.primary,
    )[
      #set text(size: sizes.body, weight: "bold", fill: white)
      Inscríbete ahora →
    ]
  ],
  // Columna derecha: visual
  [
    #set text(fill: t.text, font: fonts.heading.first())
    #set align(center + horizon)

    #block(
      width: 80%,
      inset: spacing.lg,
      radius: 12pt,
      fill: t.accent.transparentize(60%),
    )[
      #text(size: 60pt)[🎨]
      #v(spacing.sm)
      #text(size: sizes.subtitle, weight: "bold")[typst-rrss]
      #v(spacing.xs)
      #text(size: sizes.caption, fill: t.muted)[v1.0]
    ]
  ],
)

#watermark(t, "@tu_marca")
