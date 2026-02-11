import textwrap

MAIN_TEMPLATE_CAROUSEL = textwrap.dedent("""\
    // Auto-generado por rrss.py
    #import "lib.typ": *

    #let platform = "{platform}"
    #let t = theme("{theme}")
    #let dims = platforms.at(platform)

    #set page(
      width: dims.width,
      height: dims.height,
      margin: 0pt,
      fill: t.bg,
    )

    #set text(font: fonts.body.first(), fill: t.text)
    
    // Slides
    #let slides = {slides_content}

    #carousel-layout(
      t,
      slides: slides,
      title: "{title}",
      bg-image: {bg_image_line},
    )
""")

MAIN_TEMPLATE_ARTICLE = textwrap.dedent("""\
    // Auto-generado por rrss.py
    #import "lib.typ": *

    #let platform = "{platform}"
    #let t = theme("{theme}")
    #let dims = platforms.at(platform)

    #set page(
      width: dims.width,
      height: dims.height,
      margin: 0pt,
      fill: rgb("#1a1a1a"),
    )

    #set text(font: fonts.body.first(), fill: t.text)

    #article-layout(
      brand: "{brand}",
      title: "{title}",
      quote-text: "{quote}",
      {bg_image_line}
      accent: rgb("{accent}"),
      url: "{url}",
    )
""")

MAIN_TEMPLATE_QUOTE = textwrap.dedent("""\
    // Auto-generado por rrss.py
    #import "lib.typ": *

    #let platform = "{platform}"
    #let t = theme("{theme}")
    #let dims = platforms.at(platform)

    #set page(
      width: dims.width,
      height: dims.height,
      margin: 0pt,
      fill: t.bg,
    )

    #set text(font: fonts.body.first(), fill: t.text)

    #quote-layout(
      t,
      quote-text: "{quote}",
      author: "{author}",
    )

    #watermark(t, "{brand}")
""")

MAIN_TEMPLATE_HERO = textwrap.dedent("""\
    // Auto-generado por rrss.py
    #import "lib.typ": *

    #let platform = "{platform}"
    #let t = theme("{theme}")
    #let dims = platforms.at(platform)

    #set page(
      width: dims.width,
      height: dims.height,
      margin: 0pt,
      fill: t.bg,
    )

    #set text(font: fonts.body.first(), fill: t.text)

    #hero-layout(
      t,
      title: "{title}",
      subtitle: "{quote}",
      {tag_line}
    )

    #watermark(t, "{brand}")
""")
