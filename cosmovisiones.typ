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
  title: "Definiendo las cosmovisiones",
  quote-text: "Una cosmovisión es una red de presuposiciones que no están verificadas por los procedimientos de la ciencia natural, acerca de la realidad, del saber y de la conducta",
  bg-image: image("assets/bg-ciencia.jpg", width: 100%),
  accent: rgb("#8b6914"),
  url: "Presuposicionalismo.com",
)
