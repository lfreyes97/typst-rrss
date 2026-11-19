// Un estilo propio de prueba. Comprueba la forma de lo que recibe: si el puente
// cambia esa forma, la compilación falla.
#import "@local/instatypst:0.1.0": *

#let mi-estilo(pieza) = {
  assert(type(pieza.name) == str, message: "name")
  assert(type(pieza.index) == int and type(pieza.total) == int, message: "index/total")
  assert(pieza.index >= 1 and pieza.index <= pieza.total, message: "index fuera de 1..total")
  assert(type(pieza.blocks) == array and pieza.blocks.len() > 0, message: "blocks")
  assert(pieza.blocks.first().type == "title", message: "el primer bloque es el título")
  assert(type(pieza.options) == dictionary, message: "options")
  assert(pieza.palette == none or type(pieza.palette.background) == color, message: "palette")
  // las claves opcionales siempre existen; valen none si la pieza no las declara
  assert(pieza.format == none or (type(pieza.format) == array and pieza.format.len() == 2), message: "format")
  assert("brand" in pieza and "variant" in pieza and "background" in pieza, message: "claves opcionales")

  let (w, h) = estilo.formato(pieza)
  let c = estilo.paleta(pieza)
  page(width: w, height: h, margin: 80pt, fill: c.background, {
    set text(fill: c.text, size: 80pt, weight: 700)
    estilo.texto(pieza, "title")
    v(30pt)
    set text(size: 40pt, weight: 400)
    estilo.texto(pieza, "body", defecto: [])
    place(top + right, text(size: 30pt)[#pieza.index / #pieza.total])
    let m = estilo.marca(pieza, c.text)
    if m != none { place(bottom + left, m) }
  })
}
