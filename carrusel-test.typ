// Auto-generado por rrss.py
#import "lib.typ": *

#let platform = "instagram-carousel"
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
#let slides = ([_¿Sabías que la ciencia apunta hacia un Creador?_], [_El argumento cosmológico Kalam demuestra que el universo tuvo un inicio absoluto._], [_Si el universo tuvo un inicio, debe tener una causa trascendente, inmaterial y sumamente poderosa._], [_Esta causa incausada coincide perfectamente con la descripción del Dios bíblico._],)

#carousel-layout(
  t,
  slides: slides,
  title: "La evidencia de Dios",
  bg-image: image("assets/contours.svg", width: 100%),
)
