#import "layouts.typ": *
#import "theme.typ": *
#import "../templates/instagram-post.typ": instagram-post
#import "../templates/instagram-carousel.typ": instagram-carousel
#import "../templates/instagram-story.typ": instagram-story
#import "../templates/linkedin-post.typ": linkedin-post
#import "../templates/twitter-post.typ": twitter-post
#import "../templates/facebook-post.typ": facebook-post

#let render-post(config) = {
  let theme = config.at("theme", default: theme("dark"))
  let platform = config.at("platform", default: "instagram-post")
  let layout = config.at("layout", default: "article")

  // Select Platform Wrapper
  let wrapper = if platform == "instagram-post" { instagram-post } else if platform == "instagram-carousel" {
    instagram-carousel
  } else if platform == "instagram-story" { instagram-story } else if platform == "linkedin-post" {
    linkedin-post
  } else if platform == "twitter-post" { twitter-post } else if platform == "facebook-post" { facebook-post } else {
    instagram-post
  } // Default fallback

  wrapper(theme: theme)[
    #if layout == "article" {
      article-layout(theme, ..config)
    } else if layout == "quote" {
      quote-layout(theme, ..config)
    } else if layout == "hero" {
      hero-layout(theme, ..config)
    } else if layout == "carousel" {
      carousel-layout(theme, ..config)
    } else {
      [Error: Unknown layout '#layout']
    }
  ]
}
