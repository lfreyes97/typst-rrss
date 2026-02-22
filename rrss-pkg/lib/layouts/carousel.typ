// =============================================================================
// LAYOUT: Carousel
// =============================================================================

#import "../theme.typ": *

/// Layout de carrusel — múltiples slides con fondo continuo.
///
/// - t (dictionary): Paleta de tema
/// - slides (array): Lista de contenidos. Puede ser string o content.
/// - title (str): Título principal (para la slide de portada).
/// - bg-image (content): Imagen de fondo (SVG/PNG) que se extenderá por todo el carrusel.
/// - brand (str): Marca para el header.
/// -> content
#let carousel(
  t,
  platform: "instagram-carousel",
  slides: (),
  title: "",
  bg-image: none,
  brand: "Presuposicionalismo",
  ..args,
) = {
  let dims = platforms.at(platform)
  let total = slides.len()

  // Iterar sobre slides
  for (i, slide) in slides.enumerate() {
    // Salto de página entre slides
    if i > 0 { pagebreak(weak: true) }

    // Fondo base

    // ─── Fondo Continuo (Sliding Window) ──────────────────────────
    // La imagen se escala al ancho total del carrusel y se desplaza
    if bg-image != none {
      place(top + left)[
        #block(width: 100%, height: 100%, clip: true)[
          #place(top + left, dx: -100% * i)[
            #block(width: 100% * total, height: 100%)[
              #set image(fit: "cover")
              #bg-image
            ]
          ]
          // Capa de oscurecimiento para legibilidad
          #block(width: 100%, height: 100%, fill: t.bg.transparentize(10%))
        ]
      ]
    }

    // ─── Contenido ──────────────────────────────────────────────

    // Slide 1: PORTADA (si hay título)
    if i == 0 and title != "" {
      place(center + horizon)[
        #block(width: 100%, height: 100%, inset: spacing.xl)[
          #set align(center + horizon)
          #let f = resolve-fonts(t)

          // Header Marca
          #place(top + center)[
            #set text(fill: t.text.transparentize(40%), size: sizes.small, weight: "bold", tracking: 3pt)
            #upper(brand)
          ]

          // Título Principal
          #block(width: 100%)[
            #set text(fill: t.text, font: f.heading.first(), size: sizes.hero * 1.2, weight: "black")
            #set par(leading: 0.65em)
            #upper(title)
          ]

          #v(spacing.lg)

          // Separador
          #line(length: 60pt, stroke: 4pt + t.primary)

          #v(spacing.lg)

          // Hook / Subtítulo (primer slide content)
          #block(width: 85%)[
            #set text(fill: t.text.transparentize(20%), size: sizes.subtitle, weight: "medium")
            #slide
          ]

          // Indicador "Swipe"
          #place(bottom + center)[
            #text(fill: t.primary, size: sizes.small, weight: "bold", tracking: 2pt)[DESLIZA →]
          ]
        ]
      ]
    } else {
      // Slide 2+: CONTENIDO
      place(center + horizon)[
        #block(
          width: 100%,
          height: 100%,
          inset: (x: spacing.xl, y: spacing.xxl), // Más margen vertical
        )[
          #let f = resolve-fonts(t)
          #set align(left + horizon)
          #set text(fill: t.text, font: f.heading.first(), size: sizes.title * 0.9)
          #set par(leading: 0.8em)

          // Header discreto
          #place(top + left)[
            #set text(fill: t.text.transparentize(60%), size: sizes.small, weight: "bold", tracking: 2pt)
            #upper(title)
          ]

          // Contenido principal
          #slide
        ]
      ]

      // Footer: Numeración
      place(bottom + right, dx: -spacing.md, dy: -spacing.md)[
        block(
        fill: t.surface,
        inset: (x: 12pt, y: 6pt),
        radius: 4pt,
        )[
        #set text(size: sizes.small, weight: "bold", fill: t.muted)
        #(i + 1) / #total
        ]
      ]
    }
  }
}
