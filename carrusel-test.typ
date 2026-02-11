// Auto-generado por rrss.py
#import "lib.typ": *

#let platform = "instagram-post"
#let t = theme("dark")
#let dims = platforms.at(platform)

#set page(
  width: dims.width,
  height: dims.height,
  margin: 0pt,
  fill: t.bg,
)

#set text(font: fonts.body.first(), fill: t.text)

// Slides
#let slides = ([_El argumento cosmológico Kalam demuestra que el universo tuvo un inicio._], [_Si tuvo un inicio, debe tener una causa trascendente, inmaterial y poderosa._], [_Esta causa coincide con la descripción del Dios bíblico._],)

#carousel-layout(
  t,
  slides: slides,
  title: "La evidencia de Dios",
  bg-contour: image("assets/bg-ciencia_contour.png", width: 100%),
)
