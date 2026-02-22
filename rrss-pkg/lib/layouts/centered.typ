// =============================================================================
// LAYOUT: Base Card (Centered)
// =============================================================================

#import "../theme.typ": *

/// Layout base centrado con soporte para fondo y overlays.
/// Útil como contenedor DRY para quote, hero, y stat.
///
/// - t (dictionary): Paleta de tema
/// - bg-image (content): Imagen de fondo opcional
/// - overlay (content): Capa sobre la imagen opcional
/// - align-val (alignment): Alineación del contenido (default: center + horizon)
/// - inset-val (length): Padding interno (default: spacing.xl)
/// - body (content): Contenido a centrar
/// -> content
#let base-card(
  t,
  bg-image: none,
  overlay: none,
  align-val: center + horizon,
  inset-val: spacing.xl,
  body,
) = {
  set align(align-val)
  block(
    width: 100%,
    height: 100%,
    clip: true,
    fill: t.bg,
  )[
    // Capa 1: Background
    #if bg-image != none {
      place(center + horizon)[#bg-image]
    }

    // Capa 2: Overlay
    #if overlay != none {
      place(center + horizon)[#overlay]
    }

    // Capa 3: Content Block
    #place(align-val)[
      #block(width: 100%, height: 100%, inset: inset-val)[
        #set align(align-val)
        #let f = resolve-fonts(t)
        #set text(fill: t.text, font: f.heading.first())
        #body
      ]
    ]
  ]
}
