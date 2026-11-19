// Una marca como función propia: (color, lienzo) -> contenido.
#let logo(color, lienzo) = {
  let a = calc.sqrt(lienzo.at(0) * lienzo.at(1)) / 30 * 1pt
  box(width: a * 3, height: a, fill: color)
}
