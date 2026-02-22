// Auto-generado por rrss-cli-rs
#import "@local/rrss:0.1.0": *

#let t = (accent: rgb("#aa2837"), bg: rgb("#0a192f"), highlight: rgb("#64ffda"), muted: rgb("#8892b0"), primary: rgb("#64ffda"), secondary: rgb("#8892b0"), surface: rgb("#112240"), text: rgb("#ccd6f6"), )

#show: set-dimensions.with(platform: "instagram-post", theme: t)

#article(
    t,
    brand: "Presuposicionalismo",
    logo: recolor-svg("assets/Logo.svg", t.text, original: "currentColor"),
    title: "La Roca de los siglos",
    quote-text: "Dios no necesita que lo defendamos. Él es la precondición de toda defensa.",
    bg-image: image("assets/bg-ciencia.jpg", width: 100%),
    overlay: recolor-svg("assets/Solid-bg.svg", t.bg, width: 100%),
    url: "Presuposicionalismo.com",
)
