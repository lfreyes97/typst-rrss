pub const GENERIC_TEMPLATE: &str = r##"
// Auto-generado por rrss-cli-rs
#import "lib.typ": *
#import "lib/dispatcher.typ": render-post

#let config = (
    brand: "{brand}",
    title: "{title}",
    quote: "{quote}",
    author: "{author}",
    source: {source},
    url: "{url}",
    platform: "{platform}",
    layout: "{layout}",
    theme: {theme},
    bg-image: {bg_image_line},
    overlay: {overlay_line},
    logo: {logo_line},
    accent: rgb("{accent}"),
    tag: {tag_line},
    slides: {slides_content},
    contour: {contour},
)

#render-post(config)
"##;
