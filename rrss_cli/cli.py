import json
import subprocess
import tomllib
from pathlib import Path

import click
from rich.console import Console
from rich.panel import Panel
from rich.table import Table

# Import from package
from .config import PLATFORMS, THEME_PALETTES
from .colors import generate_palette, suggest_accent, extract_from_image, hex_to_hsl, hsl_to_hex, rgb_to_hex
from .images import recolor_image, generate_contours, generate_noise_contours
from .templates import MAIN_TEMPLATE_ARTICLE, MAIN_TEMPLATE_QUOTE, MAIN_TEMPLATE_HERO, MAIN_TEMPLATE_CAROUSEL

console = Console()
# ROOT is assumed to be 1 level up from this file (package dir) -> ..
# But simpler: assume typst project root is current working directory or passed as env.
# For compatibility with original script, we resolve ROOT relative to the entry point or __file__.
# Let's assume ROOT is the parent of the package directory.
ROOT = Path(__file__).parent.parent.resolve()


def suggest_accent_fn(image_path: str) -> str:
    """Wrapper para suggest_accent (evita conflicto de nombres con click)."""
    return suggest_accent(image_path)


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


def _get_size(path: Path | None) -> str:
    if not path or not path.exists():
        return "0K"
    size = path.stat().st_size
    if size < 1024:
        return f"{size}B"
    elif size < 1024 * 1024:
        return f"{size / 1024:.0f}K"
    else:
        return f"{size / (1024 * 1024):.1f}M"


@click.group()
def cli():
    """🎨 typst-rrss — Generador de imágenes para redes sociales."""
    pass


@cli.command()
@click.argument("base_color")
@click.option("--format", "-f", "fmt", type=click.Choice(["table", "json", "typst"]), default="table", help="Formato de salida")
@click.option("--name", "-n", default="custom", help="Nombre de la paleta (para formato typst)")
def colors(base_color: str, fmt: str, name: str):
    """Genera un esquema de colores a partir de un color hex base."""
    base_color = base_color.lstrip("#")
    if len(base_color) != 6:
        console.print("[red]Error:[/red] El color debe tener formato #RRGGBB")
        raise SystemExit(1)

    palette = generate_palette(f"#{base_color}")
    _print_palette(palette, fmt, name, f"#{base_color}")


@cli.command()
@click.argument("image", type=click.Path(exists=True))
@click.option("--count", "-c", default=8, help="Cantidad de colores a extraer")
@click.option("--format", "-f", "fmt", type=click.Choice(["table", "json", "palette"]), default="table", help="Formato de salida")
@click.option("--suggest-accent", is_flag=True, help="Sugerir el mejor color de acento")
@click.option("--name", "-n", default="extracted", help="Nombre de la paleta (para --format palette)")
def extract(image: str, count: int, fmt: str, suggest_accent: bool, name: str):
    """Extrae colores dominantes de una imagen."""
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
        base = extracted[0]["hex"]
        console.print(f"[dim]Color base: {base}[/dim]\n")
        palette = generate_palette(base)
        _print_palette(palette, "table", name, f"desde {Path(image).name}")

    else:
        table = Table(title=f"Colores de [bold]{Path(image).name}[/bold]", show_lines=True)
        table.add_column("#", style="dim")
        table.add_column("Hex")
        table.add_column("RGB")
        table.add_column("%", justify="right")
        table.add_column("██████", justify="center")

        for i, c in enumerate(extracted, 1):
            r, g, b = c["rgb"]
            pct = f"{c['proportion'] * 100:.1f}%"
            table.add_row(str(i), c["hex"], f"({r}, {g}, {b})", pct, f"[on {c['hex']}]      [/]")

        console.print(table)


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
    """Genera main.typ con los datos proporcionados."""
    if auto_accent and image_path:
        accent = suggest_accent(str(ROOT / image_path))
        console.print(f"[cyan]⟩[/cyan] Acento auto-extraído: [on {accent}]  {accent}  [/]")

    accent = accent.lstrip("#")
    
    bg_image_line = "none"
    if layout == "carousel":
        if contour:
            if image_path:
                abs_contour_path = generate_contours(str(ROOT / image_path))
                rel_contour_path = Path(abs_contour_path).relative_to(ROOT)
                bg_image_line = f'image("{rel_contour_path}", width: 100%)'
                console.print(f"[cyan]⟩[/cyan] Contorno extraído de imagen: {rel_contour_path}")
            else:
                abs_contour_path = generate_noise_contours(root_path=ROOT)
                rel_contour_path = Path(abs_contour_path).relative_to(ROOT)
                bg_image_line = f'image("{rel_contour_path}")'
                console.print(f"[cyan]⟩[/cyan] Contorno generativo creado (Perlin): {rel_contour_path}")
        elif image_path:
             bg_image_line = f'image("{image_path}", width: 100%)'

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
        if slides and isinstance(slides, str) and "|" in slides:
             slides_list = [s.strip() for s in slides.split("|")]
             slides_content = "(" + ", ".join([f'"{s}"' for s in slides_list]) + ",)"
        else:
             slides_content = slides
            
        content = MAIN_TEMPLATE_CAROUSEL.format(
            platform=platform,
            theme=theme_name,
            title=title,
            slides_content=slides_content,
            bg_image_line=bg_image_line,
        )

    out_path = ROOT / output_file
    out_path.write_text(content, encoding="utf-8")
    console.print(f"[green]✓[/green] Generado [bold]{output_file}[/bold] ({layout}, {platform})")


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
            if template:
                generated = list(Path(output_dir).glob(f"{name}-*.png"))
                size_str = f"{len(generated)} imgs"
            else:
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
    """Pipeline completo: genera main.typ + compila a PNG."""
    typ_file = f"{output_name}.typ"

    ctx = click.get_current_context()
    ctx.invoke(
        generate,
        brand=brand, title=title, quote=quote, image_path=image_path,
        accent=accent, auto_accent=auto_accent, url=url, platform=platform, layout=layout,
        theme_name=theme_name, author=author, tag=tag, output_file=typ_file,
    )

    ctx.invoke(
        compile,
        files=(typ_file,), ppi=ppi, output_dir="output", compile_all=False,
    )


@cli.command()
@click.argument("config_file", default="posts.toml", type=click.Path(exists=True))
@click.option("--only", "-o", "only_name", default=None, help="Generar solo un post por nombre")
@click.option("--dry-run", is_flag=True, help="Mostrar lo que se generaría sin compilar")
def build(config_file: str, only_name: str | None, dry_run: bool):
    """Genera posts desde un archivo TOML declarativo."""
    config_path = Path(config_file)
    with open(config_path, "rb") as f:
        config = tomllib.load(f)

    defaults = config.get("defaults", {})
    posts = config.get("post", [])

    if not posts:
        console.print("[yellow]⚠[/yellow] No se encontraron [[post]] en el archivo.")
        return

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
        slides = merged.get("slides", [])
        contour = merged.get("contour", False)
        
        if slides:
            slides_content = "(" + ", ".join([f'[_{s}_]' for s in slides]) + ",)"
        else:
            slides_content = "()"

        auto_accent = accent == "auto"
        if auto_accent:
            accent = "#4a3f6b"

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

            ctx.invoke(
                generate,
                brand=brand, title=title, quote=quote, image_path=actual_image,
                accent=accent, auto_accent=auto_accent, url=url, platform=platform,
                layout=layout, theme_name=theme_name, author=author, tag=tag,
                slides=slides_content, contour=contour,
                output_file=typ_file,
            )

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
