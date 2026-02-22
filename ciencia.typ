// Auto-generado por rrss-cli-rs
#import "@local/rrss:0.1.0": *

#let t = (accent: rgb("#2d2d2d"), bg: rgb("#050506"), highlight: rgb("#201e21"), muted: rgb("#98999a"), primary: rgb("#2a2c30"), secondary: rgb("#18181a"), surface: rgb("#080808"), text: rgb("#f2f2f2"), )

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
