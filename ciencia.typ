// Auto-generado por rrss-cli-rs
#import "@local/rrss:0.1.0": *

#let t = (accent: rgb("#2d2d2d"), bg: rgb("#1c1e21"), highlight: rgb("#debdee"), muted: rgb("#7a7d81"), primary: rgb("#b2cbff"), secondary: rgb("#f5b8c5"), surface: rgb("#232529"), text: rgb("#dcdee4"), )

#show: set-dimensions.with(platform: "instagram-post", theme: t)

#article(
    t,
    brand: "Presuposicionalismo",
    logo: recolor-svg("assets/Logo.svg", t.text, original: "currentColor"),
    title: "Ciencia: Una celebración y un lamento",
    quote-text: "Cuando David vio las estrellas y el sol, alabó a Dios. ¡Qué nuevas y espectaculares bellezas tenemos ahora a la vista, gracias a los descubrimientos del siglo XX!",
    bg-image: image("assets/bg-ciencia.jpg", width: 100%),
    overlay: recolor-svg("assets/Solid-bg.svg", t.bg, width: 100%),
    url: "Presuposicionalismo.com",
)
