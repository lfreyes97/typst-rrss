pub const GENERIC_TEMPLATE: &str = r##"
// Auto-generado por rrss-cli-rs
#import "@local/rrss:0.1.0": *

#let t = {theme}

#show: set-dimensions.with(platform: "{platform}", theme: t)

#{layout}(
  t,
  brand: "{brand}",
  title: "{title}",
  quote-text: "{quote}",
  author: "{author}",
  source: {source},
  url: "{url}",
  bg-image: {bg_image_line},
  overlay: {overlay_line},
  logo: {logo_line},
  accent: rgb("{accent}"),
  tag: {tag_line},
  slides: {slides_content},
)
"##;
