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
  title: "La Creación bajo ataque",
  quote-text: "Cuando los incrédulos dicen que saben que el mundo es eterno o auto-existente, por un lado, están ejerciendo fe en sus presuposiciones naturalistas",
  bg-image: image("assets/bg-ciencia.jpg", width: 100%),
  accent: rgb("#e28263"),
  url: "Presuposicionalismo.com",
)
