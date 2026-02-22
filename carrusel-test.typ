// Auto-generado por rrss-cli-rs
#import "@local/rrss:0.1.0": *

#let t = (accent: rgb("#0f3460"), bg: rgb("#0f0f0f"), highlight: rgb("#ffd369"), muted: rgb("#a0a0a0"), primary: rgb("#e94560"), secondary: rgb("#533483"), surface: rgb("#1a1a2e"), text: rgb("#f5f5f5"), )

#show: set-dimensions.with(platform: "instagram-carousel", theme: t)

#carousel(
    t,
    brand: "Presuposicionalismo",
    logo: recolor-svg("assets/Logo.svg", t.text, original: "currentColor"),
    title: "La evidencia de Dios",
    overlay: recolor-svg("assets/Solid-bg.svg", t.bg, width: 100%),
    url: "Presuposicionalismo.com",
    contour: true,
    slides: ("¿Sabías que la ciencia apunta hacia un Creador?", "El argumento cosmológico Kalam demuestra que el universo tuvo un inicio absoluto.", "Si el universo tuvo un inicio, debe tener una causa trascendente, inmaterial y sumamente poderosa.", "Esta causa incausada coincide perfectamente con la descripción del Dios bíblico.",),
)
