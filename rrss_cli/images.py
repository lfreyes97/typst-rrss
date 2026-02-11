from pathlib import Path
from PIL import Image, ImageOps, ImageFilter
import numpy as np
import matplotlib.pyplot as plt
from scipy.ndimage import gaussian_filter

from .config import THEME_PALETTES
from .colors import hex_to_rgb_tuple

# Assuming ROOT is needed, passing it or handling it via current working dir/relative paths might be better.
# For now, let's accept absolute paths or handle resolution in the caller.

def recolor_image(
    image_path: str,
    theme_name: str,
    output_path: str | None = None,
    intensity: float = 0.7,
) -> str:
    """Aplica un duotone/tinte a una imagen usando los colores del tema."""
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
    """Genera una versión de contornos (blueprint) de la imagen."""
    img = Image.open(image_path).convert("L")  # Grayscale
    
    # 1. Reducir ruido
    img = img.filter(ImageFilter.GaussianBlur(radius=1.5))
    
    # 2. Detectar bordes
    edges = img.filter(ImageFilter.FIND_EDGES) 
    
    # 3. Limpiar bordes débiles
    edges = edges.point(lambda x: 255 if x > 30 else 0)
    
    white = Image.new("L", edges.size, 255)
    mask = edges

    # Componer: Color blanco + Máscara
    result = Image.merge("LA", (white, mask))

    p = Path(image_path)
    output_path = str(p.parent / f"{p.stem}_contour.png")
    result.save(output_path, "PNG")
    return output_path


def generate_noise_contours(root_path: Path, width=3240, height=1350, scale=4, levels=15, seed=None) -> str:
    """Genera contornos suaves usando ruido gaussiano filtrado (simula Perlin)."""
    if seed:
        np.random.seed(seed)
        
    w_low, h_low = width // 10, height // 10
    noise = np.random.randn(h_low, w_low)
    
    sigma = max(w_low, h_low) / scale
    smooth = gaussian_filter(noise, sigma=sigma)
    
    fig_w, fig_h = width / 100, height / 100
    fig = plt.figure(figsize=(fig_w, fig_h), frameon=False)
    ax = plt.Axes(fig, [0., 0., 1., 1.])
    ax.set_axis_off()
    fig.add_axes(ax)
    
    ax.contour(smooth, levels=levels, colors='white', alpha=0.3, linewidths=1.5)
    
    output_path = root_path / "assets" / "generated_contours.svg"
    output_path.parent.mkdir(exist_ok=True)
    
    fig.savefig(str(output_path), format='svg', transparent=True)
    plt.close(fig)
    
    return str(output_path)
