// =============================================================================
// CONTENIDO: Estadística / Dato
// =============================================================================
// Uso: typst compile content/stat.typ output/stat.png
// =============================================================================

#import "../templates/linkedin-post.typ": *
#import "../lib/theme.typ": *
#import "../lib/layouts.typ": *
#import "../lib/elements.typ": *

#show: linkedin-post.with(palette-name: "forest")

#let t = theme("forest")

#stat-layout(
  t,
  number: "73%",
  label: "de los desarrolladores prefieren herramientas basadas en código para diseño",
  source: "Developer Survey 2025 · @tu_marca",
)
