// Auto-generado por rrss-cli-rs
#import "@local/rrss:0.1.0": *

#let t = (accent: rgb("#3e7daa"), bg: rgb("#101417"), highlight: rgb("#004c6c"), muted: rgb("#8b9198"), primary: rgb("#92cef6"), secondary: rgb("#b6c9d8"), surface: rgb("#101417"), text: rgb("#dfe3e7"), )

#show: set-dimensions.with(platform: "instagram-post", theme: t)

#article(
    t,
    brand: "Presuposicionalismo",
    logo: recolor-svg("assets/Logo.svg", t.text, original: "currentColor"),
    title: "Diseño Dinámico: Material You",
    quote-text: "Tus colores se adaptan automáticamente a la imagen de fondo con el motor de Material Design 3.",
    bg-image: image("assets/bg2.jpg", width: 100%),
    overlay: recolor-svg("assets/Solid-bg.svg", t.bg, width: 100%),
    url: "Presuposicionalismo.com",
)
