
// Auto-generado por rrss-cli-rs
#import "lib.typ": *
#import "templates/instagram-post.typ": instagram-post

#let t = (accent: rgb("#0f3460"), bg: rgb("#0f0f0f"), highlight: rgb("#ffd369"), muted: rgb("#a0a0a0"), primary: rgb("#e94560"), secondary: rgb("#533483"), surface: rgb("#1a1a2e"), text: rgb("#f5f5f5"), )

#instagram-post(theme: t)[
  #article-layout(
    t,
    brand: "Presuposicionalismo",
    logo: recolor-svg("assets/Logo.svg", t.text, original: "currentColor"),
    title: "La Creación bajo ataque",
    quote-text: "Cuando los incrédulos dicen que saben que el mundo es eterno o auto-existente, por un lado, están ejerciendo fe en sus presuposiciones naturalistas",
    bg-image: image("assets/bg-ciencia.jpg", width: 100%),
    overlay: recolor-svg("assets/Solid-bg.svg", t.bg, original: "#000000", width: 100%),
    accent: rgb("#aa2837"),
    url: "Presuposicionalismo.com",
  )
]
