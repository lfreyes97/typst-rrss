// Estilo propio: el carrusel «La disociación» (Ellul, Propaganda 4.3), de Post/disociacion.typ.
// El diseño es el mismo, línea por línea; lo que cambia es de dónde lee: antes, de los
// parámetros de cada función; ahora, de la pieza que resuelve el contrato. Los colores
// salen de `pieza.palette` (el mismo mapa que hacía a mano con `fondo == ascua`), y el
// `n / total`, de la posición de la pieza en la serie.
//
// El énfasis (`#em[...]` en el original) es la negrita `*...*` del contenido.
#import "@local/instatypst:0.1.0": estilo

#let sans = "Bricolage Grotesque"
#let serif = "Baskervville"
#let marca = text(font: sans, size: 18pt, weight: 700, tracking: 0.12em)[LUIS F. REYES]

// la frase clave pasa a sans, recta y en negrita
#let em(x) = text(font: sans, style: "normal", weight: 800, size: 0.86em, x)

#let _pagina(pieza, m, cuerpo) = {
  let (w, h) = estilo.formato(pieza)
  let c = estilo.paleta(pieza)
  page(width: w, height: h, margin: m, fill: c.background, {
    set text(font: sans, fill: c.text, lang: "es", hyphenate: false)
    show strong: it => em(it.body)
    cuerpo(c)
  })
}

#let _pie(pieza) = {
  place(bottom + left, dy: 100pt, marca)
  place(bottom + right, dy: 100pt, text(size: 18pt, weight: 700, tracking: 0.12em)[#pieza.index / #pieza.total])
}

// ── 1 · portada ──────────────────────────────────────────────
#let portada(pieza) = {
  let (w, h) = estilo.formato(pieza)
  let c = estilo.paleta(pieza)
  page(width: w, height: h, margin: 0pt, fill: c.background, {
    set text(font: sans, fill: c.text, lang: "es", hyphenate: false)
    set par(leading: 34pt)
    place(left + top, rect(width: w, height: 20pt, fill: c.accent))
    place(left + top, dx: 80pt, dy: 100pt,
      text(size: 28pt, weight: 600, tracking: 0.16em, fill: c.accent, estilo.texto(pieza, "label")))
    place(left + top, dx: 72pt, dy: 175pt, block(width: 940pt, {
      show strong: it => text(fill: c.accent, it.body)
      text(size: 150pt, weight: 800, tracking: -0.04em, estilo.texto(pieza, "title"))
    }))
    place(left + bottom, dx: 80pt, dy: -170pt, block(width: 760pt,
      text(size: 44pt, font: serif, style: "italic", estilo.texto(pieza, "quote"))))
    place(right + bottom, dx: -80pt, dy: -178pt, text(size: 64pt, weight: 700, fill: c.accent)[→])
    place(left + bottom, dx: 80pt, dy: -80pt, marca)
  })
}

// ── lámina con desarrollo: etiqueta · título · cita opcional · cuerpo ──
#let desarrollo(pieza) = _pagina(pieza, (x: 88pt, top: 110pt, bottom: 170pt), c => {
  set par(leading: 0.55em, justify: false)
  text(size: 26pt, weight: 700, tracking: 0.16em, fill: c.accent, upper(estilo.texto(pieza, "label")))
  v(44pt)
  text(size: 76pt, weight: 800, tracking: -0.02em, estilo.texto(pieza, "title"))
  v(36pt)
  line(length: 120pt, stroke: 4pt + c.accent)
  v(36pt)
  let cita = estilo.texto(pieza, "quote")
  if cita != none {
    let fuente = estilo.bloque(pieza, "attribution")
    block(inset: (left: 28pt), stroke: (left: 4pt + c.accent), {
      text(size: 44pt, font: serif, style: "italic", cita)
      if fuente != none { v(8pt); text(size: 24pt, weight: 600, tracking: 0.04em, fuente.author) }
    })
    v(32pt)
  }
  let cuerpo = estilo.texto(pieza, "body")
  if cuerpo != none { text(size: 42pt, font: serif, weight: 400, cuerpo) }
  _pie(pieza)
})

// ── lámina central: la cita es la protagonista, el título queda como firma ──
#let central(pieza) = _pagina(pieza, (x: 88pt, top: 110pt, bottom: 170pt), c => {
  set par(leading: 0.6em, justify: false)
  place(right + top, dx: 30pt, dy: -40pt,
    text(size: 560pt, font: serif, fill: c.accent.transparentize(78%))[“])
  text(size: 26pt, weight: 700, tracking: 0.16em, fill: c.accent, upper(estilo.texto(pieza, "label")))
  v(150pt)
  text(size: 62pt, font: serif, style: "italic", estilo.texto(pieza, "quote"))
  let fuente = estilo.bloque(pieza, "attribution")
  if fuente != none { v(14pt); text(size: 24pt, weight: 600, tracking: 0.04em, fuente.author) }
  place(bottom + left, dy: -20pt, {
    line(length: 120pt, stroke: 4pt + c.accent)
    v(22pt)
    text(size: 54pt, weight: 800, tracking: -0.02em, estilo.texto(pieza, "title"))
  })
  _pie(pieza)
})

// ── lámina de filas: rótulo a la izquierda, texto a la derecha, y una cita de cierre ──
#let filas(pieza) = _pagina(pieza, (x: 88pt, top: 110pt, bottom: 170pt), c => {
  set par(leading: 0.55em, justify: false)
  text(size: 26pt, weight: 700, tracking: 0.16em, fill: c.accent, upper(estilo.texto(pieza, "label")))
  v(44pt)
  text(size: 76pt, weight: 800, tracking: -0.02em, estilo.texto(pieza, "title"))
  v(36pt)
  for fila in estilo.bloque(pieza, "list").items {
    line(length: 100%, stroke: 2pt + c.accent.transparentize(55%))
    v(22pt)
    grid(columns: (210pt, 1fr), column-gutter: 24pt,
      text(size: 30pt, weight: 800, tracking: 0.1em, fill: c.accent, upper(fila.label)),
      text(size: 38pt, font: serif, fila.text))
    v(22pt)
  }
  line(length: 100%, stroke: 2pt + c.accent.transparentize(55%))
  v(34pt)
  block(inset: (left: 28pt), stroke: (left: 4pt + c.accent), {
    text(size: 44pt, font: serif, style: "italic", estilo.texto(pieza, "quote"))
  })
  v(26pt)
  text(size: 38pt, font: serif, estilo.texto(pieza, "body"))
  _pie(pieza)
})
