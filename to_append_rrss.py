
# ─── Generative Contours (Noise) ─────────────────────────────────────────────

def generate_noise_contours(width=3240, height=1350, scale=4, levels=15, seed=None):
    """Genera contornos suaves usando ruido gaussiano filtrado (simula Perlin).
    
    Retorna la ruta del archivo SVG generado.
    """
    if seed:
        np.random.seed(seed)
        
    # Crear grid de ruido reducido
    w_low, h_low = width // 10, height // 10
    noise = np.random.randn(h_low, w_low)
    
    # Suavizar para crear "montañas" (efecto tipo Perlin)
    sigma = max(w_low, h_low) / scale
    smooth = gaussian_filter(noise, sigma=sigma)
    
    # Configurar plot sin bordes ni ejes
    fig_w, fig_h = width / 100, height / 100 # Medida arbitraria para mantener ratio
    fig = plt.figure(figsize=(fig_w, fig_h), frameon=False)
    ax = plt.Axes(fig, [0., 0., 1., 1.])
    ax.set_axis_off()
    fig.add_axes(ax)
    
    # Dibujar contornos
    # levels = número de líneas
    ax.contour(smooth, levels=levels, colors='white', alpha=0.3, linewidths=1.5)
    
    # Guardar SVG
    output_path = ROOT / "assets" / "generated_contours.svg"
    output_path.parent.mkdir(exist_ok=True)
    
    fig.savefig(str(output_path), format='svg', transparent=True)
    plt.close(fig)
    
    return str(output_path)
