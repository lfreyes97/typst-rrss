
// Auto-generado por rrss-cli-rs
#import "lib.typ": *
#import "templates/instagram-carousel.typ": instagram-carousel

#let t = (accent: rgb("#0f3460"), bg: rgb("#0f0f0f"), highlight: rgb("#ffd369"), muted: rgb("#a0a0a0"), primary: rgb("#e94560"), secondary: rgb("#533483"), surface: rgb("#1a1a2e"), text: rgb("#f5f5f5"), )
#let slides = ("¿Sabías que la ciencia apunta hacia un Creador?", "El argumento cosmológico Kalam demuestra que el universo tuvo un inicio absoluto.", "Si el universo tuvo un inicio, debe tener una causa trascendente, inmaterial y sumamente poderosa.", "Esta causa incausada coincide perfectamente con la descripción del Dios bíblico.",)

#instagram-carousel(theme: t)[
  #carousel-layout(
    t,
    slides: slides,
    title: "La evidencia de Dios",
    bg-image: none,
  )
]
