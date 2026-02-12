
// Auto-generado por rrss-cli-rs
#import "lib.typ": *
#import "templates/instagram-post.typ": instagram-post

#let t = (accent: rgb("#6c8b75"), bg: rgb("#2a2c30"), primary: rgb("#d0635c"), secondary: rgb("#804049"), surface: rgb("#2a2c30"), text: rgb("#ffffff"), )

#instagram-post(theme: t)[
  #article-layout(
    t,
    brand: "Presuposicionalismo",
    logo: recolor-svg("assets/Logo.svg", t.text, original: "currentColor"),
    title: "Ciencia: Una celebración y un lamento",
    quote-text: "Cuando David vio las estrellas y el sol, alabó a Dios. ¡Qué nuevas y espectaculares bellezas tenemos ahora a la vista, gracias a los descubrimientos del siglo XX!",
    bg-image: image("assets/bg-ciencia.jpg", width: 100%),
    overlay: recolor-svg("assets/Solid-bg.svg", t.bg, original: "#000000", width: 100%),
    accent: rgb("#2d2d2d"),
    url: "Presuposicionalismo.com",
  )
]
