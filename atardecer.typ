
// Auto-generado por rrss-cli-rs
#import "lib.typ": *
#import "templates/instagram-post.typ": instagram-post

#let t = (accent: rgb("#ffd3b6"), bg: rgb("#1b2d1b"), highlight: rgb("#ffaaa5"), muted: rgb("#98b898"), primary: rgb("#a8e6cf"), secondary: rgb("#dcedc1"), surface: rgb("#2d4a2d"), text: rgb("#f0f0f0"), )

#instagram-post(theme: t)[
  #article-layout(
    t,
    brand: "Presuposicionalismo",
    title: "La luz en la oscuridad",
    quote-text: "El temor de Jehová es el principio de la sabiduría, y el conocimiento del Santo es la inteligencia.",
    bg-image: image("assets/bg3.jpg", width: 100%),
    accent: rgb("#972c0f"),
    url: "Presuposicionalismo.com",
  )
]
