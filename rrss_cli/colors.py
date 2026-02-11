import colorsys
import colorgram

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
