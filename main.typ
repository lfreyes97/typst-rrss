// main.typ - Layout Vertical Estricto (Instagram Post)



// -----------------------------------------------------------------------------
// 1. CONFIGURACIÓN DEL LAYOUT
// -----------------------------------------------------------------------------

#let header-height = 150pt
#let footer-height = 100pt
#let page-width = 1080pt
#let page-height = 1080pt

// Secciones calculadas
#let body-start = header-height
#let body-height = page-height - header-height - footer-height

// -----------------------------------------------------------------------------
// 2. TEMA Y RECURSOS
// -----------------------------------------------------------------------------

#let palettes = (
  forest: (
    bg: rgb("#1b2d1b"),
    surface: rgb("#2d4a2d"),
    primary: rgb("#a8e6cf"),
    secondary: rgb("#dcedc1"),
    accent: rgb("#ffd3b6"),
    text: rgb("#f0f0f0"),
    muted: rgb("#98b898"),
    highlight: rgb("#ffaaa5"),
  ),
)

#let fonts = (
  heading: ("Fira Sans", "Roboto", "Arial"),
  body: ("Fira Sans", "Roboto", "Arial"),
)

#let sizes = (
  title: 56pt, // Ajustado para el nuevo layout grande
  subtitle: 32pt,
  body: 24pt,
  caption: 20pt,
)


// -----------------------------------------------------------------------------
// 3. LAYOUT DEFINITION
// -----------------------------------------------------------------------------

#let instagram-post-layout(
  t,
  brand: "PRESUPOSICIONALISMO",
  logo: none,
  title: "",
  quote-text: "",
  bg-image: none,
  overlay: none,
  url: "Presuposicionalismo.com",
) = {
  // --- SECCIÓN HEADER ---
  place(top + left, dy: 0pt)[
    #block(width: 100%, height: header-height, fill: t.surface)[
      #set align(center + horizon)
      #if logo != none {
        block(height: 60%, logo)
      }
    ]
  ]

  // --- SECCIÓN BODY (BACKGROUND + CONTENIDO) ---
  place(top + left, dy: body-start)[
    #block(width: 100%, height: body-height, clip: true, fill: t.bg)[

      // Capa 1: Imagen de Fondo o SVG por defecto
      #if bg-image != none {
        place(center + horizon)[
          #image(bg-image, width: 100%, height: 100%, fit: "cover")
        ]
      }

      // Capa 2: Overlay
      #if overlay != none {
        place(center + horizon)[#overlay]
      }

      // Capa 3: Contenido de Texto
      #place(center + horizon)[
        #block(width: 85%)[
          #set align(center)
          #set text(font: fonts.heading.first(), fill: t.text)

          // Título Principal
          #text(size: sizes.title, weight: "black")[#upper(title)]

          #v(1em)
          #line(length: 30%, stroke: 2pt + t.accent)
          #v(1em)

          // Cita / Texto
          #set text(font: fonts.body.first(), style: "italic", size: sizes.body)
          "#quote-text"
        ]
      ]
    ]
  ]

  // --- SECCIÓN FOOTER ---
  place(bottom + left, dy: 0pt)[
    #block(width: 100%, height: footer-height, fill: t.surface)[
      #set align(center + horizon)
      #text(fill: t.text, font: fonts.body.first(), size: sizes.caption, weight: "medium")[
        #url
      ]
    ]
  ]
}

// -----------------------------------------------------------------------------
// 5. DOCUMENTO PRINCIPAL
// -----------------------------------------------------------------------------

#let theme-active = palettes.forest

#set page(
  width: page-width,
  height: page-height,
  margin: 0pt,
  fill: theme-active.bg,
)

#instagram-post-layout(
  theme-active,

  // Datos
  brand: "Presuposicionalismo",
  title: "La luz en la Oscuridad",
  quote-text: "El temor de Jehová es el principio de la sabiduría, y el conocimiento del Santo es la inteligencia.",
  url: "@presuposicionalismo | www.presuposicionalismo.com",

  // Recursos
  logo: image("assets/Logo.svg", height: 100%),
  overlay: image("assets/Solid-bg.svg", width: 100%, height: 100%),
  bg-image: "assets/bg3_forest.jpg",
)
