pub const MAIN_TEMPLATE_CAROUSEL: &str = r##"
// Auto-generado por rrss-cli-rs
#import "lib.typ": *
#import "templates/{platform}.typ": {platform}

#let t = {theme}
#let slides = {slides_content}

#{platform}(theme: t)[
  #carousel-layout(
    t,
    slides: slides,
    title: "{title}",
    bg-image: {bg_image_line},
  )
]
"##;

pub const MAIN_TEMPLATE_ARTICLE: &str = r##"
// Auto-generado por rrss-cli-rs
#import "lib.typ": *
#import "templates/{platform}.typ": {platform}

#let t = {theme}

#{platform}(theme: t)[
  #article-layout(
    t,
    brand: "{brand}",
    {logo_line}
    title: "{title}",
    quote-text: "{quote}",
    {bg_image_line}
    {overlay_line}
    accent: rgb("{accent}"),
    url: "{url}",
  )
]
"##;

pub const MAIN_TEMPLATE_QUOTE: &str = r##"
// Auto-generado por rrss-cli-rs
#import "lib.typ": *
#import "templates/{platform}.typ": {platform}

#let t = {theme}

#{platform}(theme: t)[
  #quote-layout(
    t,
    quote-text: "{quote}",
    author: "{author}",
  )
  #watermark(t, "{brand}")
]
"##;

pub const MAIN_TEMPLATE_HERO: &str = r##"
// Auto-generado por rrss-cli-rs
#import "lib.typ": *
#import "templates/{platform}.typ": {platform}

#let t = {theme}

#{platform}(theme: t)[
  #hero-layout(
    t,
    title: "{title}",
    subtitle: "{quote}",
    {tag_line}
  )
  #watermark(t, "{brand}")
]
"##;
