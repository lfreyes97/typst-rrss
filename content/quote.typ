// =============================================================================
// CONTENIDO: Cita / Quote
// =============================================================================
// Uso: typst compile content/quote.typ output/quote.png
//
// Para cambiar tema, editar palette-name abajo.
// Para cambiar plataforma, cambiar el import del template.
// =============================================================================

#import "../templates/instagram-post.typ": *
#import "../lib/theme.typ": *
#import "../lib/layouts.typ": *
#import "../lib/elements.typ": *

#show: instagram-post.with(palette-name: "dark")

#let t = theme("dark")

#quote-layout(
  t,
  quote-text: "La imaginación es más importante que el conocimiento. El conocimiento es limitado, la imaginación rodea el mundo.",
  author: "Albert Einstein",
  source: "Entrevista, 1929",
)

#watermark(t, "@tu_marca")
