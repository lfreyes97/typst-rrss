// =============================================================================
// ELEMENTS — Elementos decorativos reutilizables
// =============================================================================

#import "theme.typ": *

/// Badge / etiqueta decorativa.
///
/// - t (dictionary): Paleta de tema
/// - label (str): Texto del badge
/// - bg (color): Color de fondo (default: primary del tema)
/// -> content
#let badge(t, label, bg: none) = {
  let fill-color = if bg != none { bg } else { t.primary }
  box(
    inset: (x: spacing.sm, y: spacing.xs),
    radius: 4pt,
    fill: fill-color,
  )[
    #set text(size: sizes.small, weight: "bold", fill: white, font: fonts.heading.first())
    #upper(label)
  ]
}

/// Línea divisora decorativa.
///
/// - t (dictionary): Paleta de tema
/// - width (length): Largo de la línea
/// -> content
#let divider(t, width: 60pt) = {
  line(length: width, stroke: 2.5pt + t.primary)
}

/// Marca de agua / watermark en una esquina.
///
/// - t (dictionary): Paleta de tema
/// - text-content (str): Texto de la marca (e.g. "@usuario")
/// -> content
#let watermark(t, text-content) = {
  place(
    bottom + right,
    dx: -spacing.md,
    dy: -spacing.md,
  )[
    #set text(size: sizes.caption, fill: t.muted.transparentize(40%), font: fonts.body.first())
    #text-content
  ]
}

/// Barra lateral de acento.
///
/// - t (dictionary): Paleta de tema
/// - height (length): Altura de la barra
/// -> content
#let accent-bar(t, height: 100%) = {
  place(
    left + top,
  )[
    #rect(width: 6pt, height: height, fill: t.primary)
  ]
}

/// Círculo decorativo de fondo (para dar dinamismo visual).
///
/// - color (color): Color del círculo
/// - size (length): Diámetro del círculo
/// - x (length): Posición horizontal
/// - y (length): Posición vertical
/// -> content
#let deco-circle(color, size, x, y) = {
  place(
    top + left,
    dx: x,
    dy: y,
  )[
    #circle(radius: size, fill: color.transparentize(85%))
  ]
}

/// Bullet point estilizado para listas.
///
/// - t (dictionary): Paleta de tema
/// - items (array): Lista de strings
/// -> content
#let styled-list(t, items) = {
  set text(fill: t.text, font: fonts.body.first(), size: sizes.body)
  for item in items {
    block(spacing: spacing.sm)[
      #text(fill: t.primary, weight: "bold")[▸ ]
      #item
    ]
  }
}

/// Número con unidad — para estadísticas.
///
/// - t (dictionary): Paleta de tema
/// - number (str): Cifra principal
/// - unit (str): Unidad (%, M, K, etc.)
/// -> content
#let stat-number(t, number, unit: "") = {
  [
    #text(size: sizes.stat-xl, weight: "black", fill: t.primary, font: fonts.heading.first())[#number]
    #if unit != "" {
      text(size: sizes.title, weight: "bold", fill: t.primary, font: fonts.heading.first())[#unit]
    }
  ]
}

/// Footer con handle/marca.
///
/// - t (dictionary): Paleta de tema
/// - handle (str): Handle o nombre de marca
/// -> content
#let footer-handle(t, handle) = {
  place(
    bottom + center,
    dy: -spacing.md,
  )[
    #set text(size: sizes.caption, fill: t.muted, font: fonts.body.first(), weight: "medium")
    #handle
  ]
}

/// Carga un SVG y reemplaza un color específico.
/// Útil para iconos y fondos vectoriales que deben adaptarse al tema.
///
/// - path (str): Ruta al archivo SVG (ej: "assets/icon.svg"). Nota: asume que lib/ está a un nivel de la raíz.
/// - original (color, str): Color original a reemplazar (por defecto busca negro puro #000000).
/// - replacement (color, str): Nuevo color.
/// -> content
#let recolor-svg(path, replacement, original: "#000000", width: auto, height: auto) = {
  // Como estamos en lib/, subimos un nivel para encontrar assets/
  let real-path = "../" + path
  let data = read(real-path)

  let old-marker = if type(original) == color { original.to-hex() } else { original }
  let new-marker = if type(replacement) == color { replacement.to-hex() } else { replacement }

  image(bytes(data.replace(old-marker, new-marker)), format: "svg", width: width, height: height)
}
