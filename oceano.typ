// Auto-generado por rrss.py
#import "lib.typ": *

#let platform = "instagram-post"
#let t = theme("ocean")
#let dims = platforms.at(platform)

#set page(
  width: dims.width,
  height: dims.height,
  margin: 0pt,
  fill: rgb("#1a1a1a"),
)

#set text(font: fonts.body.first(), fill: t.text)

#article-layout(
  brand: "Presuposicionalismo",
  title: "La Roca de los siglos",
  quote-text: "Dios no necesita que lo defendamos. Él es la precondición de toda defensa.",
  bg-image: image("assets/bg-ciencia_ocean.jpg", width: 100%),
  accent: rgb("#162d38"),
  url: "Presuposicionalismo.com",
)
