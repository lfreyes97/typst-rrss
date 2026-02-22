// Auto-generado por rrss-cli-rs
#import "@local/rrss:0.1.0": *

#let t = (accent: rgb("#0f3460"), bg: rgb("#0f0f0f"), highlight: rgb("#ffd369"), muted: rgb("#a0a0a0"), primary: rgb("#e94560"), secondary: rgb("#533483"), surface: rgb("#1a1a2e"), text: rgb("#f5f5f5"), )

#instagram-post(theme: t)[
  #article-layout(
    t,
    brand: "RRSS Test",
    title: "Ocean Theme Test",
    quote-text: "Typst makes design programmable.",
    author: "Typst RRSS",
    bg-image: image("assets/bg-ciencia.jpg", width: 100%),
    overlay: recolor-svg("assets/Solid-bg.svg", t.bg, width: 100%),
    url: "Presuposicionalismo.com",
    tag: "@typst_rrss",
  )
]
