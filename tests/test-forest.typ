// Auto-generado por rrss-cli-rs
#import "@local/rrss:0.1.0": *

#let t = (accent: rgb("#15292f"), bg: rgb("#0d0908"), highlight: rgb("#506116"), muted: rgb("#a5938d"), primary: rgb("#983112"), secondary: rgb("#493b15"), surface: rgb("#140d0b"), text: rgb("#f3f2f1"), )

#instagram-post(theme: t)[
  #quote-layout(
    t,
    brand: "RRSS Test",
    title: "Nature Extracted Colors",
    quote-text: "Nature never did betray the heart that loved her.",
    author: "William Wordsworth",
    bg-image: image("assets/bg3.jpg", width: 100%),
    overlay: recolor-svg("assets/Solid-bg.svg", t.bg, width: 100%),
    url: "Presuposicionalismo.com",
  )
]
