// Auto-generado por rrss.py
#import "lib.typ": *

#let platform = "instagram-post"
#let t = theme("dark")
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
  title: "Ciencia: Una celebración y un lamento",
  quote-text: "Cuando David vio las estrellas y el sol, alabó a Dios. ¡Qué nuevas y espectaculares bellezas tenemos ahora a la vista, gracias a los descubrimientos del siglo XX!",
  bg-image: image("assets/bg-ciencia.jpg", width: 100%),
  accent: rgb("#2d2d2d"),
  url: "Presuposicionalismo.com",
)
