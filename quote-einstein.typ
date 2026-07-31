// Auto-generado por rrss-cli-rs
#import "@local/rrss:0.1.0": *

#let t = (accent: rgb("#0f3460"), bg: rgb("#0f0f0f"), highlight: rgb("#ffd369"), muted: rgb("#a0a0a0"), primary: rgb("#e94560"), secondary: rgb("#533483"), surface: rgb("#1a1a2e"), text: rgb("#f5f5f5"), )

#show: set-dimensions.with(platform: "instagram-post", theme: t)

#quote(
    t,
    brand: "@tu_marca",
    logo: recolor-svg("assets/Logo.svg", t.text, original: "currentColor"),
    title: "La imaginación",
    quote-text: "La imaginación es más importante que el conocimiento. El conocimiento es limitado, la imaginación rodea el mundo.",
    author: "Albert Einstein",
    source: "Entrevista, 1929",
    overlay: recolor-svg("assets/Solid-bg.svg", t.bg, width: 100%),
    url: "Presuposicionalismo.com",
)
