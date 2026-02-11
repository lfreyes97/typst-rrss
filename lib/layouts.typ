// =============================================================================
// LAYOUTS — Funciones de layout reutilizables
// =============================================================================

#import "theme.typ": *

/// Layout centrado con fondo — ideal para quotes y tips.
///
/// - t (dictionary): Paleta de tema
/// - body (content): Contenido a centrar
/// -> content
#let centered-card(t, body) = {
  set align(center + horizon)
  block(
    width: 100%,
    height: 100%,
    fill: t.bg,
    inset: spacing.xl,
    body,
  )
}

/// Layout hero — texto grande centrado con subtítulo.
///
/// - t (dictionary): Paleta de tema
/// - title (str): Título principal
/// - subtitle (str): Subtítulo
/// - tag (str): Tag/etiqueta opcional
/// -> content
#let hero-layout(t, title: "", subtitle: "", tag: none) = {
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

/// Layout dividido en dos columnas — ideal para promo con imagen.
///
/// - t (dictionary): Paleta de tema
/// - left (content): Contenido del lado izquierdo
/// - right (content): Contenido del lado derecho
/// - ratio (array): Proporción de las columnas (default 1fr, 1fr)
/// -> content
#let split-layout(t, left, right, ratio: (1fr, 1fr)) = {
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

/// Layout de cita — texto con comillas decorativas y autor.
///
/// - t (dictionary): Paleta de tema
/// - quote-text (str): Texto de la cita
/// - author (str): Autor de la cita
/// - source (str): Fuente opcional (libro, discurso, etc.)
/// -> content
#let quote-layout(t, quote-text: "", author: "", source: none) = {
  set align(center + horizon)
  block(
    width: 100%,
    height: 100%,
    fill: t.bg,
    inset: spacing.xl,
  )[
    #set text(fill: t.text, font: fonts.heading.first())

    // Comilla decorativa de apertura
    #text(size: 80pt, fill: t.primary, weight: "black")["]

    #v(-spacing.lg)

    // Texto de la cita
    #block(width: 85%)[
      #set text(size: sizes.subtitle, weight: "medium", style: "italic")
      #set par(leading: 0.85em)
      #quote-text
    ]

    #v(spacing.lg)

    // Línea decorativa
    #line(length: 60pt, stroke: 2pt + t.primary)

    #v(spacing.md)

    // Autor
    #text(size: sizes.body, weight: "bold", fill: t.primary)[#author]

    #if source != none {
      v(spacing.xs)
      text(size: sizes.caption, fill: t.muted)[#source]
    }
  ]
}

/// Layout de artículo con imagen de fondo — estilo Presuposicionalismo.
///
/// - brand (str): Nombre de la marca/logo
/// - title (str): Título del artículo (se muestra en mayúsculas)
/// - quote-text (str): Cita o extracto con guillemets
/// - bg-image (content): Imagen de fondo (pasar `image("ruta")` directamente)
/// - accent (color): Color de acento para la barra del título
/// - url (str): URL del sitio para el footer
/// -> content
#let article-layout(
  brand: "PRESUPOSICIONALISMO",
  title: "",
  quote-text: "",
  bg-image: none,
  accent: rgb("#4a3f6b"),
  url: "Presuposicionalismo.com",
) = {
  let header-h = 60pt
  let title-zone-h = 190pt
  let footer-h = 45pt

  // ─── Imagen de fondo (capa base) ──────────────────────────────
  if bg-image != none {
    place(bottom + center, dy: -footer-h)[
      #block(width: 100%, clip: true)[
        #bg-image
      ]
    ]
  }

  // ─── Header: barra oscura con marca ───────────────────────────
  place(top + center)[
    #block(
      width: 100%,
      height: header-h,
      fill: rgb("#111111"),
    )[
      #set align(center + horizon)
      #set text(fill: white, font: fonts.heading.first())
      #text(size: 12pt, tracking: 2pt)[꩜]
      #v(-4pt)
      #text(size: sizes.caption - 1pt, weight: "bold", tracking: 3pt)[#upper(brand)]
    ]
  ]

  // ─── Zona de título + cita (con fondo semi-transparente) ──────
  place(top + center, dy: header-h)[
    #block(
      width: 100%,
      height: title-zone-h,
      fill: accent.transparentize(10%),
      inset: (x: spacing.lg, top: spacing.md, bottom: spacing.sm),
    )[
      #set align(center + top)
      #set text(fill: white, font: fonts.heading.first())

      // Título
      #block(width: 100%)[
        #set text(size: sizes.subtitle + 2pt, weight: "black", tracking: 1.5pt)
        #set par(leading: 0.7em)
        #upper(title)
      ]

      #v(spacing.xs)

      // Separador
      #line(length: 90%, stroke: 2pt + white.transparentize(40%))

      #v(spacing.xs)

      // Cita
      #block(width: 92%)[
        #set text(size: sizes.body - 2pt, weight: "regular", fill: white.transparentize(5%))
        #set par(leading: 0.7em)
        «#quote-text»
      ]
    ]
  ]

  // ─── Footer: barra con URL ────────────────────────────────────
  place(bottom + center)[
    #block(
      width: 100%,
      height: footer-h,
      fill: rgb("#111111"),
    )[
      #set align(center + horizon)
      #set text(fill: white, font: fonts.heading.first(), size: sizes.body - 2pt, style: "italic")
      #url
    ]
  ]
}
