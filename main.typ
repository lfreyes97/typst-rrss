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
  title: "El autoengaño del incrédulo",
  quote-text: "Por supuesto, todos los hombres tienen fe. Los incrédulos tienen fe tanto como los creyentes.",
  bg-image: image("assets/bg-ciencia.jpg", width: 100%),
  accent: rgb("#1a1c19"),
  url: "Presuposicionalismo.com",
)
