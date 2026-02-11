// Auto-generado por rrss.py
#import "lib.typ": *

#let platform = "instagram-post"
#let t = theme("sunset")
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
  title: "La luz en la oscuridad",
  quote-text: "El temor de Jehová es el principio de la sabiduría, y el conocimiento del Santo es la inteligencia.",
  bg-image: image("assets/bg-ciencia_sunset.jpg", width: 100%),
  accent: rgb("#7d414b"),
  url: "Presuposicionalismo.com",
)
