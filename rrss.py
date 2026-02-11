#!/usr/bin/env python3
"""rrss.py — CLI para generar imágenes de redes sociales con Typst."""

import colorsys
import json
import subprocess
import textwrap
import tomllib
from pathlib import Path

import click
import colorgram
from PIL import Image, ImageEnhance, ImageOps, ImageFilter
from rich.console import Console
from rich.panel import Panel
from rich.table import Table
from rich.color import Color as RichColor
from rich import print as rprint

console = Console()
ROOT = Path(__file__).parent.resolve()

# ─── Plataformas válidas ─────────────────────────────────────────────────────

PLATFORMS = {
    "instagram-post": (1080, 1080),
    "instagram-story": (1080, 1920),
    "facebook-post": (1200, 630),
    "twitter-post": (1600, 900),
    "linkedin-post": (1200, 627),
    "og-image": (1200, 630),
}

LAYOUTS = ["article", "quote", "hero", "stat", "centered-card", "split"]

# ─── Paletas (espejo de lib/theme.typ) ───────────────────────────────────────

THEME_PALETTES = {
    "dark": {
        "bg": "#0f0f0f", "surface": "#1a1a2e", "primary": "#e94560",
        "secondary": "#533483", "accent": "#0f3460", "text": "#f5f5f5",
        "muted": "#a0a0a0", "highlight": "#ffd369",
    },
    "light": {
        "bg": "#fafafa", "surface": "#ffffff", "primary": "#e94560",
        "secondary": "#6c5ce7", "accent": "#00b894", "text": "#1a1a2e",
        "muted": "#6b7280", "highlight": "#fdcb6e",
    },
    "ocean": {
        "bg": "#0a192f", "surface": "#112240", "primary": "#64ffda",
        "secondary": "#8892b0", "accent": "#233554", "text": "#ccd6f6",
        "muted": "#8892b0", "highlight": "#64ffda",
    },
    "sunset": {
        "bg": "#1a1a2e", "surface": "#16213e", "primary": "#ff6b6b",
        "secondary": "#feca57", "accent": "#ff9ff3", "text": "#f5f5f5",
        "muted": "#a0a0a0", "highlight": "#feca57",
    },
    "forest": {
        "bg": "#1b2d1b", "surface": "#2d4a2d", "primary": "#a8e6cf",
        "secondary": "#dcedc1", "accent": "#ffd3b6", "text": "#f0f0f0",
        "muted": "#98b898", "highlight": "#ffaaa5",
    },
}

# ─── Utilidades de color ─────────────────────────────────────────────────────


def hex_to_rgb_tuple(hex_color: str) -> tuple[int, int, int]:
    """Convierte hex a tupla RGB (0-255)."""
    h = hex_color.lstrip("#")
    return (int(h[0:2], 16), int(h[2:4], 16), int(h[4:6], 16))


def hex_to_hsl(hex_color: str) -> tuple[float, float, float]:
    """Convierte un color hex (#RRGGBB) a HSL (h: 0-360, s: 0-1, l: 0-1)."""
    hex_color = hex_color.lstrip("#")
    r, g, b = (int(hex_color[i : i + 2], 16) / 255 for i in (0, 2, 4))
    h, l, s = colorsys.rgb_to_hls(r, g, b)
    return h * 360, s, l


def hsl_to_hex(h: float, s: float, l: float) -> str:
    """Convierte HSL (h: 0-360, s: 0-1, l: 0-1) a hex."""
    r, g, b = colorsys.hls_to_rgb(h / 360, l, s)
    return f"#{int(r * 255):02x}{int(g * 255):02x}{int(b * 255):02x}"


def clamp(val: float, mn: float = 0.0, mx: float = 1.0) -> float:
    return max(mn, min(mx, val))


def generate_palette(base_hex: str) -> dict[str, str]:
    """Genera un esquema de 8 colores a partir de un color hex base."""
    h, s, l = hex_to_hsl(base_hex)

    palette = {
        "bg": hsl_to_hex(h, clamp(s * 0.3), clamp(l * 0.12, mx=0.1)),
        "surface": hsl_to_hex(h, clamp(s * 0.35), clamp(l * 0.18, mx=0.15)),
        "primary": hsl_to_hex(h, s, l),
        "secondary": hsl_to_hex((h + 30) % 360, clamp(s * 0.7), clamp(l * 0.55)),
        "accent": hsl_to_hex((h + 180) % 360, clamp(s * 0.5), clamp(l * 0.4)),
        "text": hsl_to_hex(h, clamp(s * 0.1), clamp(0.95)),
        "muted": hsl_to_hex(h, clamp(s * 0.15), clamp(0.6)),
        "highlight": hsl_to_hex((h + 60) % 360, clamp(s * 0.8), clamp(l * 0.7)),
    }
    # strip leading # from all values for consistency
    return {k: v.lstrip("#") for k, v in palette.items()}


def rgb_to_hex(r: int, g: int, b: int) -> str:
    """Convierte RGB (0-255) a hex."""
    return f"#{r:02x}{g:02x}{b:02x}"


def extract_from_image(image_path: str, count: int = 8) -> list[dict]:
    """Extrae los colores dominantes de una imagen usando colorgram.

    Retorna una lista de dicts con 'hex', 'rgb', 'hsl', 'proportion'.
    """
    colors = colorgram.extract(image_path, count)
    results = []
    for c in colors:
        r, g, b = c.rgb.r, c.rgb.g, c.rgb.b
        hex_val = rgb_to_hex(r, g, b)
        h, s, l = hex_to_hsl(hex_val)
        results.append({
            "hex": hex_val,
            "rgb": (r, g, b),
            "hsl": (round(h, 1), round(s, 3), round(l, 3)),
            "proportion": round(c.proportion, 4),
        })
    return results


def suggest_accent(image_path: str) -> str:
    """Sugiere el mejor color de acento a partir de una imagen.

    Busca el color más saturado y con luminosidad media entre los dominantes.
    """
    colors = extract_from_image(image_path, count=12)
    if not colors:
        return "#4a3f6b"

    # Puntuar cada color: preferir saturación alta + luminosidad media
    scored = []
    for c in colors:
        h, s, l = c["hsl"]
        # Evitar colores muy oscuros/claros o muy desaturados
        if l < 0.08 or l > 0.92 or s < 0.05:
            continue
        # Score: saturación alta + luminosidad entre 0.2-0.6
        lum_score = 1.0 - abs(l - 0.4) * 2  # máximo en l=0.4
        score = s * 0.6 + lum_score * 0.3 + c["proportion"] * 0.1
        scored.append((score, c["hex"]))

    if not scored:
        # Fallback: usar el segundo color más dominante (el primero suele ser fondo)
        return colors[min(1, len(colors) - 1)]["hex"]

    scored.sort(reverse=True)
    return scored[0][1]


# ─── Recolor de imagen ──────────────────────────────────────────────────────


def recolor_image(
    image_path: str,
    theme_name: str,
    output_path: str | None = None,
    intensity: float = 0.7,
) -> str:
    """Aplica un duotone/tinte a una imagen usando los colores del tema.

    Convierte a escala de grises y mapea sombras → bg, luces → primary.
    intensity: 0.0 = original, 1.0 = duotone puro.

    Retorna la ruta de la imagen recoloreada.
    """
    palette = THEME_PALETTES.get(theme_name, THEME_PALETTES["dark"])
    dark_color = hex_to_rgb_tuple(palette["bg"])
    light_color = hex_to_rgb_tuple(palette["primary"])

    img = Image.open(image_path).convert("RGB")
    gray = ImageOps.grayscale(img)

    # Crear gradiente de 256 valores para cada canal
    gradient = []
    for i in range(256):
        t = i / 255.0
        r = int(dark_color[0] * (1 - t) + light_color[0] * t)
        g = int(dark_color[1] * (1 - t) + light_color[1] * t)
        b = int(dark_color[2] * (1 - t) + light_color[2] * t)
        gradient.extend([r, g, b])

    # Aplicar LUT (lookup table) al grayscale
    duotone = gray.convert("RGB")
    # Aplicar el gradiente como palette
    lut = gradient  # 256 * 3 = 768 valores
    channels = duotone.split()
    r_ch = channels[0].point(lut[0::3])
    g_ch = channels[0].point(lut[1::3])
    b_ch = channels[0].point(lut[2::3])
    duotone = Image.merge("RGB", (r_ch, g_ch, b_ch))

    # Mezclar original con duotone según intensidad
    result = Image.blend(img, duotone, intensity)

    # Guardar
    if output_path is None:
        p = Path(image_path)
        output_path = str(p.parent / f"{p.stem}_{theme_name}{p.suffix}")

    result.save(output_path, quality=92)
    return output_path


def generate_contours(image_path: str) -> str:
    """Genera una versión de contornos (blueprint) de la imagen.

    Usa filtro FIND_EDGES, invierte colores y hace transparente el fondo negro.
    Aplica Blur y Threshold para limpiar el ruido y dejar trazos más definidos.
    Retorna la ruta de la imagen procesada.
    """
    img = Image.open(image_path).convert("L")  # Grayscale
    
    # 1. Reducir ruido (detalles finos)
    img = img.filter(ImageFilter.GaussianBlur(radius=1.5))
    
    # 2. Detectar bordes
    edges = img.filter(ImageFilter.FIND_EDGES) 
    
    # 3. Limpiar bordes débiles (Threshold)
    # Todo lo menor a 30 se vuelve negro (0), lo mayor se vuelve blanco (255)
    # Ajustar el umbral según se necesite. Valor alto = menos líneas.
    edges = edges.point(lambda x: 255 if x > 30 else 0)
    
    # Invertir: bordes negros -> blancos, fondo blanco -> negro (para la máscara visual)
    # Pero para la máscara alpha queremos: bordes=blanco(opaco), fondo=negro(transparente)
    
    # Truco:
    # La imagen 'edges' actual tiene bordes BLANCOS y fondo NEGRO.
    # Queremos pintar de blanco donde haya bordes.
    
    white = Image.new("L", edges.size, 255)
    mask = edges  # Bordes blancos (opacos), fondo negro (transparente)

    # Componer: Color blanco + Máscara
    result = Image.merge("LA", (white, mask))

    p = Path(image_path)
    output_path = str(p.parent / f"{p.stem}_contour.png")
    result.save(output_path, "PNG")
    return output_path


# ─── Templates Typst ─────────────────────────────────────────────────────────

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
      bg-contour: {bg_contour_line},
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


# ─── CLI ─────────────────────────────────────────────────────────────────────


@click.group()
def cli():
    """🎨 typst-rrss — Generador de imágenes para redes sociales."""
    pass


# ─── Subcomando: colors ──────────────────────────────────────────────────────


@cli.command()
@click.argument("base_color")
@click.option("--format", "-f", "fmt", type=click.Choice(["table", "json", "typst"]), default="table", help="Formato de salida")
@click.option("--name", "-n", default="custom", help="Nombre de la paleta (para formato typst)")
def colors(base_color: str, fmt: str, name: str):
    """Genera un esquema de colores a partir de un color hex base.

    Ejemplo: rrss.py colors "#e94560"
    """
    base_color = base_color.lstrip("#")
    if len(base_color) != 6:
        console.print("[red]Error:[/red] El color debe tener formato #RRGGBB")
        raise SystemExit(1)

    palette = generate_palette(f"#{base_color}")
    _print_palette(palette, fmt, name, f"#{base_color}")


def _print_palette(palette: dict, fmt: str, name: str, title: str):
    """Imprime una paleta en el formato solicitado."""
    if fmt == "json":
        click.echo(json.dumps({k: f"#{v}" for k, v in palette.items()}, indent=2))

    elif fmt == "typst":
        lines = [f"  {name}: ("]
        for key, val in palette.items():
            lines.append(f'    {key}: rgb("#{val}"),')
        lines.append("  ),")
        click.echo("\n".join(lines))

    else:  # table
        table = Table(title=f"Esquema de color: [bold]{title}[/bold]", show_lines=True)
        table.add_column("Rol", style="bold")
        table.add_column("Hex")
        table.add_column("██████", justify="center")

        for key, val in palette.items():
            table.add_row(key, f"#{val}", f"[on #{val}]      [/]")

        console.print(table)


# ─── Subcomando: extract ─────────────────────────────────────────────────────


@cli.command()
@click.argument("image", type=click.Path(exists=True))
@click.option("--count", "-c", default=8, help="Cantidad de colores a extraer")
@click.option("--format", "-f", "fmt", type=click.Choice(["table", "json", "palette"]), default="table", help="Formato de salida")
@click.option("--suggest-accent", is_flag=True, help="Sugerir el mejor color de acento")
@click.option("--name", "-n", default="extracted", help="Nombre de la paleta (para --format palette)")
def extract(image: str, count: int, fmt: str, suggest_accent: bool, name: str):
    """Extrae colores dominantes de una imagen.

    Ejemplo: rrss.py extract assets/bg-ciencia.jpg
    """
    console.print(f"[cyan]⟩[/cyan] Analizando [bold]{image}[/bold]...")
    extracted = extract_from_image(image, count)

    if suggest_accent:
        accent = suggest_accent_fn(image)
        console.print(f"\n[bold]Acento sugerido:[/bold] [on {accent}]  {accent}  [/]")
        console.print(f"\nUsa: [dim]--accent \"{accent}\"[/dim]")
        return

    if fmt == "json":
        click.echo(json.dumps([
            {"hex": c["hex"], "rgb": list(c["rgb"]), "proportion": c["proportion"]}
            for c in extracted
        ], indent=2))

    elif fmt == "palette":
        # Usar el color más dominante para generar una paleta completa
        base = extracted[0]["hex"]
        console.print(f"[dim]Color base: {base}[/dim]\n")
        palette = generate_palette(base)
        _print_palette(palette, "table", name, f"desde {Path(image).name}")

    else:  # table
        table = Table(title=f"Colores de [bold]{Path(image).name}[/bold]", show_lines=True)
        table.add_column("#", style="dim")
        table.add_column("Hex")
        table.add_column("RGB")
        table.add_column("%", justify="right")
        table.add_column("██████", justify="center")

        for i, c in enumerate(extracted, 1):
            r, g, b = c["rgb"]
            pct = f"{c['proportion'] * 100:.1f}%"
            table.add_row(
                str(i),
                c["hex"],
                f"({r}, {g}, {b})",
                pct,
                f"[on {c['hex']}]      [/]",
            )

        console.print(table)


def suggest_accent_fn(image_path: str) -> str:
    """Wrapper para suggest_accent (evita conflicto de nombres con click)."""
    return suggest_accent(image_path)


# ─── Subcomando: generate ────────────────────────────────────────────────────


@cli.command()
@click.option("--brand", "-b", default="Presuposicionalismo", help="Nombre de la marca")
@click.option("--title", "-t", required=True, help="Título del post")
@click.option("--quote", "-q", required=True, help="Cita o extracto")
@click.option("--image", "-i", "image_path", default=None, help="Ruta a imagen de fondo")
@click.option("--accent", "-a", default="#4a3f6b", help="Color de acento hex")
@click.option("--auto-accent", is_flag=True, help="Extraer acento automáticamente de la imagen")
@click.option("--url", "-u", default="Presuposicionalismo.com", help="URL del footer")
@click.option("--platform", "-p", type=click.Choice(list(PLATFORMS.keys())), default="instagram-post", help="Plataforma")
@click.option("--layout", "-l", type=click.Choice(["article", "quote", "hero"]), default="article", help="Layout a usar")
@click.option("--theme", "theme_name", type=click.Choice(["dark", "light", "ocean", "sunset", "forest"]), default="dark", help="Tema de color")
@click.option("--author", default="", help="Autor (para layout quote)")
@click.option("--tag", default=None, help="Tag/etiqueta (para layout hero)")
@click.option("--slides", default=None, help="Lista de slides (separados por |) para layout carousel")
@click.option("--contour", is_flag=True, help="Generar contorno para carousel")
@click.option("--output", "-o", "output_file", default="main.typ", help="Archivo de salida")
def generate(brand, title, quote, image_path, accent, auto_accent, url, platform, layout, theme_name, author, tag, slides, contour, output_file):
    """Genera main.typ con los datos proporcionados.

    Ejemplo: rrss.py generate --title "Mi título" --quote "Mi cita"
    """
    # Auto-accent: extraer color de la imagen
    if auto_accent and image_path:
        accent = suggest_accent(str(ROOT / image_path))
        console.print(f"[cyan]⟩[/cyan] Acento auto-extraído: [on {accent}]  {accent}  [/]")

    accent = accent.lstrip("#")
    
    # Procesar contour si es solicitado
    bg_contour_line = "none"
    if contour and image_path and layout == "carousel":
        abs_contour_path = generate_contours(str(ROOT / image_path))
        # Convertir a relativa para Typst
        rel_contour_path = Path(abs_contour_path).relative_to(ROOT)
        bg_contour_line = f'image("{rel_contour_path}", width: 100%)'
        console.print(f"[cyan]⟩[/cyan] Contorno generado: {rel_contour_path}")

    if layout == "article":
        bg_line = f'bg-image: image("{image_path}", width: 100%),' if image_path else "// sin imagen de fondo"
        content = MAIN_TEMPLATE_ARTICLE.format(
            platform=platform,
            theme=theme_name,
            brand=brand,
            title=title,
            quote=quote,
            bg_image_line=bg_line,
            accent=f"#{accent}",
            url=url,
        )
    elif layout == "quote":
        content = MAIN_TEMPLATE_QUOTE.format(
            platform=platform,
            theme=theme_name,
            brand=brand,
            title=title,
            quote=quote,
            author=author or brand,
        )
    elif layout == "hero":
        tag_line = f'tag: "{tag}",' if tag else "// sin tag"
        content = MAIN_TEMPLATE_HERO.format(
            platform=platform,
            theme=theme_name,
            brand=brand,
            title=title,
            quote=quote,
            tag_line=tag_line,
        )
    elif layout == "carousel":
        # slides puede venir como string separado por pipes desde CLI
        # o se asume ya formateado si viene desde build
        if slides and isinstance(slides, str) and "|" in slides:
             slides_list = [s.strip() for s in slides.split("|")]
             slides_content = "(" + ", ".join([f'"{s}"' for s in slides_list]) + ",)"
        else:
             slides_content = slides # Asumimos string Typst válido o vacío
            
        content = MAIN_TEMPLATE_CAROUSEL.format(
            platform=platform,
            theme=theme_name,
            title=title,
            slides_content=slides_content,
            bg_contour_line=bg_contour_line,
        )

    out_path = ROOT / output_file
    out_path.write_text(content, encoding="utf-8")
    console.print(f"[green]✓[/green] Generado [bold]{output_file}[/bold] ({layout}, {platform})")


# ─── Subcomando: compile ─────────────────────────────────────────────────────

def _get_size(path: Path | None) -> str:
    """Helper para obtener el tamaño de un archivo en formato legible."""
    if not path or not path.exists():
        return "0K"
    size = path.stat().st_size
    if size < 1024:
        return f"{size}B"
    elif size < 1024 * 1024:
        return f"{size / 1024:.0f}K"
    else:
        return f"{size / (1024 * 1024):.1f}M"

@cli.command()
@click.argument("files", nargs=-1)
@click.option("--ppi", default=144, type=int, help="Pixeles por pulgada (default 144)")
@click.option("--output-dir", "-d", default="output", help="Directorio de salida")
@click.option("--all", "-a", "compile_all", is_flag=True, help="Compilar todos los .typ en content/")
@click.option("--template", default=None, help="Patrón de numeración (ej: {0p}) para multipágina")
def compile(files, ppi, output_dir, compile_all, template):
    """Compila archivos .typ a PNG."""
    if compile_all:
        files = sorted((ROOT / "content").glob("*.typ"))
    elif files:
        files = [ROOT / f for f in files]
    else:
        files = [ROOT / "main.typ"]
    
    if not files:
        console.print("[yellow]⚠[/yellow] No se especificaron archivos para compilar.")
        return

    Path(output_dir).mkdir(exist_ok=True)
    
    console.print(Panel(
        f"[bold]PPI:[/bold] {ppi}  [bold]Archivos:[/bold] {len(files)}  [bold]Salida:[/bold] {output_dir}/",
        title="[bold cyan]typst-rrss[/bold cyan]",
        border_style="cyan",
    ))

    success = 0
    failed = 0

    for target in files:
        name = target.stem
        
        if template:
            # Si hay template, usamos el patrón (ej: {0p}) en el nombre
            out_filename = f"{name}-{template}.png"
        else:
            out_filename = f"{name}.png"
            
        output = Path(output_dir) / out_filename
        
        console.print(f"  [blue]⟩[/blue] {name}...", end=" ")

        result = subprocess.run(
            ["typst", "compile", "--root", str(ROOT), f"--ppi={ppi}", str(target), str(output)],
            capture_output=True,
            text=True,
        )

        if result.returncode == 0:
            # En multipágina, typst genera archivo_1.png, archivo_2.png, etc. si el patrón es {n}
            # Ojo: si usamos template={0p}, typst genera carrusel-test-01.png, carrusel-test-02.png
            # str(output) ya contiene el patrón.
            
            # Para mostrar tamaño, intentamos adivinar el primer archivo generado
            # Si el patrón tiene {p}, el primer archivo seria ...1.png
            if template:
                # Comprobar si se generó algún archivo que coincida
                generated = list(Path(output_dir).glob(f"{name}-*.png"))
                size_str = f"{len(generated)} imgs"
            else:
                size = output.stat().st_size if output.exists() else 0
                size_str = _get_size(output)
            
            console.print(f"[green]✓[/green] ({size_str})")
            success += 1
        else:
            console.print("[red]✗ Error[/red]")
            for line in result.stderr.strip().split("\n")[:5]:
                console.print(f"    {line}", style="dim")
            failed += 1

    console.print()
    console.print(f"[bold]Resultado:[/bold] [green]{success} generadas[/green]", end="")
    if failed:
        console.print(f" · [red]{failed} con errores[/red]")
    else:
        console.print()


# ─── Subcomando: full ────────────────────────────────────────────────────────


@cli.command()
@click.option("--brand", "-b", default="Presuposicionalismo")
@click.option("--title", "-t", required=True, help="Título del post")
@click.option("--quote", "-q", required=True, help="Cita o extracto")
@click.option("--image", "-i", "image_path", default=None, help="Ruta a imagen de fondo")
@click.option("--accent", "-a", default="#4a3f6b", help="Color de acento hex")
@click.option("--auto-accent", is_flag=True, help="Extraer acento de la imagen")
@click.option("--url", "-u", default="Presuposicionalismo.com")
@click.option("--platform", "-p", type=click.Choice(list(PLATFORMS.keys())), default="instagram-post")
@click.option("--layout", "-l", type=click.Choice(["article", "quote", "hero"]), default="article")
@click.option("--theme", "theme_name", type=click.Choice(["dark", "light", "ocean", "sunset", "forest"]), default="dark")
@click.option("--author", default="")
@click.option("--tag", default=None)
@click.option("--ppi", default=144, type=int)
@click.option("--output-name", "-o", default="main", help="Nombre del archivo de salida (sin extensión)")
def full(brand, title, quote, image_path, accent, auto_accent, url, platform, layout, theme_name, author, tag, ppi, output_name):
    """Pipeline completo: genera main.typ + compila a PNG.

    Ejemplo: rrss.py full --title "Mi título" --quote "Mi cita" --accent "#e94560"
    """
    typ_file = f"{output_name}.typ"

    # Paso 1: Generate
    ctx = click.get_current_context()
    ctx.invoke(
        generate,
        brand=brand, title=title, quote=quote, image_path=image_path,
        accent=accent, auto_accent=auto_accent, url=url, platform=platform, layout=layout,
        theme_name=theme_name, author=author, tag=tag, output_file=typ_file,
    )

    # Paso 2: Compile
    ctx.invoke(
        compile,
        files=(typ_file,), ppi=ppi, output_dir="output", compile_all=False,
    )


# ─── Subcomando: build ───────────────────────────────────────────────────────


@cli.command()
@click.argument("config_file", default="posts.toml", type=click.Path(exists=True))
@click.option("--only", "-o", "only_name", default=None, help="Generar solo un post por nombre")
@click.option("--dry-run", is_flag=True, help="Mostrar lo que se generaría sin compilar")
def build(config_file: str, only_name: str | None, dry_run: bool):
    """Genera posts desde un archivo TOML declarativo.

    \b
    Ejemplo:
      ./rrss build                    # procesa posts.toml
      ./rrss build mis-posts.toml     # archivo personalizado
      ./rrss build --only ciencia     # solo un post
      ./rrss build --dry-run          # previsualizar sin compilar
    """
    config_path = Path(config_file)
    with open(config_path, "rb") as f:
        config = tomllib.load(f)

    defaults = config.get("defaults", {})
    posts = config.get("post", [])

    if not posts:
        console.print("[yellow]⚠[/yellow] No se encontraron [[post]] en el archivo.")
        return

    # Filtrar si se pidió uno específico
    if only_name:
        posts = [p for p in posts if p.get("name") == only_name]
        if not posts:
            console.print(f"[red]Error:[/red] No se encontró post con nombre [bold]{only_name}[/bold]")
            raise SystemExit(1)

    console.print(Panel(
        f"[bold]Archivo:[/bold] {config_file}  [bold]Posts:[/bold] {len(posts)}",
        title="[bold cyan]rrss build[/bold cyan]",
        border_style="cyan",
    ))

    ctx = click.get_current_context()
    success = 0
    failed = 0

    for post in posts:
        name = post.get("name", f"post_{posts.index(post)}")

        # Merge: defaults ← post (post gana)
        merged = {**defaults, **post}

        title = merged.get("title", "")
        quote = merged.get("quote", "")
        brand = merged.get("brand", "Presuposicionalismo")
        url = merged.get("url", "Presuposicionalismo.com")
        platform = merged.get("platform", "instagram-post")
        layout = merged.get("layout", "article")
        theme_name = merged.get("theme", "dark")
        image_path = merged.get("image", None)
        accent = merged.get("accent", "#4a3f6b")
        author = merged.get("author", "")
        tag = merged.get("tag", None)
        ppi = merged.get("ppi", 144)
        
        # Carousel slides
        slides = merged.get("slides", [])
        contour = merged.get("contour", False)
        
        # Formatear slides para Typst
        if slides:
            slides_content = "(" + ", ".join([f'[_{s}_]' for s in slides]) + ",)"
        else:
            slides_content = "()"

        # Auto-accent
        auto_accent = accent == "auto"
        if auto_accent:
            accent = "#4a3f6b"  # default, se sobreescribe en generate

        console.print(f"\n  [blue]⟩[/blue] [bold]{name}[/bold] — {title}")

        if dry_run:
            recolor = merged.get("recolor", False)
            console.print(f"    layout={layout} platform={platform} accent={accent if not auto_accent else 'auto'}{' recolor='+theme_name if recolor else ''}")
            if image_path:
                console.print(f"    image={image_path}")
            continue

        try:
            typ_file = f"{name}.typ"
            actual_image = image_path

            # Recolor: aplicar tinte del tema a la imagen
            recolor = merged.get("recolor", False)
            if recolor and image_path:
                recolor_intensity = float(merged.get("recolor_intensity", 0.7))
                recolored_path = str(ROOT / "assets" / f"{Path(image_path).stem}_{theme_name}.jpg")
                console.print(f"    [magenta]🎨[/magenta] Recoloreando con tema [bold]{theme_name}[/bold] (intensidad: {recolor_intensity})")
                recolor_image(
                    str(ROOT / image_path),
                    theme_name,
                    output_path=recolored_path,
                    intensity=recolor_intensity,
                )
                actual_image = f"assets/{Path(image_path).stem}_{theme_name}.jpg"

            # Generate
            ctx.invoke(
                generate,
                brand=brand, title=title, quote=quote, image_path=actual_image,
                accent=accent, auto_accent=auto_accent, url=url, platform=platform,
                layout=layout, theme_name=theme_name, author=author, tag=tag,
                slides=slides_content, contour=contour,
                output_file=typ_file,
            )

            # Compile
            template_pattern = "{0p}" if layout == "carousel" else None
            ctx.invoke(
                compile,
                files=(typ_file,), ppi=ppi, output_dir="output", compile_all=False,
                template=template_pattern,
            )
            success += 1
        except Exception as e:
            console.print(f"    [red]✗ Error: {e}[/red]")
            failed += 1

    if not dry_run:
        console.print(f"\n[bold]Total:[/bold] [green]{success} generados[/green]", end="")
        if failed:
            console.print(f" · [red]{failed} con errores[/red]")
        else:
            console.print()


if __name__ == "__main__":
    cli()
