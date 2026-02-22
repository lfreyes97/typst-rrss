// =============================================================================
// LAYOUT: Article (Presuposicionalismo)
// =============================================================================

#import "../theme.typ": *

/// Layout de artículo con imagen de fondo — estilo Presuposicionalismo.
/// Refactorizado para usar sistema de partición vertical estricto (Header/Body/Footer).
///
/// - t (dictionary): Paleta de tema
/// - brand (str): Nombre de la marca/logo
/// - title (str): Título del artículo
/// - quote-text (str): Cita o extracto
/// - bg-image (content|str): Ruta a imagen de fondo o content
/// - overlay (content|str): Ruta a overlay SVG o content
/// - logo (content|str): Ruta a logo SVG o content
/// - url (str): URL del sitio para el footer
/// -> content
#let article(
  t,
  brand: "PRESUPOSICIONALISMO",
  logo: none,
  title: "",
  quote-text: "",
  bg-image: none,
  overlay: none,
  accent: none,
  url: "Presuposicionalismo.com",
  ..args,
) = {
  // Override accent color if provided
  let t = if accent != none {
    t + (accent: accent)
  } else {
    t
  }

  // 1. CONFIGURACIÓN DEL LAYOUT
  // Nota: Al usar #show: set-dimensions, las dimensiones ya están en la página.
  // Pero para colocar elementos con 'place' en posiciones fijas (footer),
  // necesitamos saber la altura. Si no se provee en args, usamos 100% o el valor de IG por defecto.
  let page-height = args.at("page-height", default: 1080pt)
  let header-height = 150pt
  let footer-height = 100pt

  // Secciones calculadas
  let body-start = header-height
  let body-height = page-height - header-height - footer-height

  // --- SECCIÓN HEADER ---
  place(top + left, dy: 0pt)[
    #block(width: 100%, height: header-height, fill: t.surface)[
      #set align(center + horizon)
      #let f = resolve-fonts(t)
      #if logo != none {
        block(height: 60%, logo)
      } else {
        stack(dir: ltr, spacing: 1em)[
          #text(size: 40pt, fill: t.text, font: f.heading.first())[꩜]
          #text(size: 32pt, weight: "bold", fill: t.text, tracking: 0.1em)[#upper(brand)]
        ]
      }
    ]
  ]

  // --- SECCIÓN BODY (BACKGROUND + CONTENIDO) ---
  place(top + left, dy: body-start)[
    #block(width: 100%, height: body-height, clip: true, fill: t.bg)[
      #let f = resolve-fonts(t)
      // Capa 1: Imagen de Fondo
      #if bg-image != none {
        place(center + horizon)[
          #bg-image
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
          #set text(font: f.heading.first(), fill: t.text)

          // Título Principal
          #set par(leading: 0.8em)
          #text(size: sizes.title, weight: "black")[#upper(title)]

          #v(1em)
          #line(length: 30%, stroke: 2pt + t.accent)
          #v(1em)

          // Cita / Texto
          #set text(font: f.body.first(), style: "italic", size: sizes.body)
          "#quote-text"
        ]
      ]
    ]
  ]

  // --- SECCIÓN FOOTER ---
  place(bottom + left, dy: 0pt)[
    #block(width: 100%, height: footer-height, fill: t.surface)[
      #set align(center + horizon)
      #let f = resolve-fonts(t)
      #text(fill: t.text, font: f.body.first(), size: sizes.caption, weight: "medium")[
        #url
      ]
    ]
  ]
}
