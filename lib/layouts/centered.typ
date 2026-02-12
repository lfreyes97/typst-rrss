// =============================================================================
// LAYOUT: Centered Card
// =============================================================================

#import "../theme.typ": *

/// Layout centrado con fondo — ideal para quotes y tips.
///
/// - t (dictionary): Paleta de tema
/// - body (content): Contenido a centrar
/// -> content
#let centered-card(t, body) = {
  set align(center + horizon)
  block(
    width: 100%,
    height: 100%,
    fill: t.bg,
    inset: spacing.xl,
    body,
  )
}
