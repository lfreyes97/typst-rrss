// Auto-generado por rrss-cli-rs
#import "@local/rrss:0.1.0": *

#let t = (accent: rgb("#8b6914"), bg: rgb("#0f0f0f"), highlight: rgb("#ffd369"), muted: rgb("#a0a0a0"), primary: rgb("#e94560"), secondary: rgb("#533483"), surface: rgb("#1a1a2e"), text: rgb("#f5f5f5"), )

#show: set-dimensions.with(platform: "instagram-post", theme: t)

#article(
    t,
    brand: "Presuposicionalismo",
    logo: recolor-svg("assets/Logo.svg", t.text, original: "currentColor"),
    title: "Definiendo las cosmovisiones",
    quote-text: "Una cosmovisión es una red de presuposiciones que no están verificadas por los procedimientos de la ciencia natural, acerca de la realidad, del saber y de la conducta",
    bg-image: image("assets/bg-ciencia.jpg", width: 100%),
    overlay: recolor-svg("assets/Solid-bg.svg", t.bg, width: 100%),
    url: "Presuposicionalismo.com",
)
