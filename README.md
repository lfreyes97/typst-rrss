# typst-rrss

Sistema modular para generar imágenes de redes sociales usando **Typst** y **Python**.

Diseñado para crear contenido visual coherente (frases, artículos, estadísticas) de forma programática y estilizada.

## 🚀 Inicio Rápido

### Requisitos
- **Typst** (para compilar las imágenes)
- **uv** (para gestionar el entorno de Python)
- **Python 3.12+**

```bash
# Instalar dependencias
uv sync

# Generar imágenes de ejemplo
./rrss build
```

## 🛠️ CLI `rrss`

El script `rrss` es tu centro de comando. Funciona como un wrapper que automatiza la generación de archivos `.typ` y su compilación.

### Comandos Principales

| Comando | Descripción |
| :--- | :--- |
| **`./rrss build`** | Genera todos los posts definidos en `posts.toml`. |
| **`./rrss extract <img.jpg>`** | Analiza una imagen y extrae su paleta de colores. |
| **`./rrss colors "#hex"`** | Genera un esquema de color completo (HSL) a partir de un hex. |
| **`./rrss compile`** | Compila manualmente archivos `.typ` a PNG. |

### Opciones de Build

```bash
# Generar solo un post específico por nombre
./rrss build --only ciencia

# Ver qué se generaría sin ejecutar nada (dry-run)
./rrss build --dry-run
```

---

## 📝 Configuración Declarativa (`posts.toml`)

Define tu contenido en `posts.toml`. Puedes configurar valores por defecto y sobreescribirlos en cada post.

```toml
[defaults]
brand = "Mi Marca"
theme = "dark"      # dark, light, ocean, sunset, forest
layout = "article"  # article, quote, hero

[[post]]
name = "ejemplo"
title = "Título del Post"
image = "assets/fondo.jpg"

# ✨ Características Mágicas ✨
accent = "auto"     # Extrae el mejor color de acento de la imagen
recolor = true      # Aplica un tinte (duotone) con los colores del tema
```

---

## 🎨 Temas y Layouts

### Layouts Disponibles
1.  **`article-layout`**: Estilo "Presuposicionalismo" (título grande, cita, footer).
2.  **`quote-layout`**: Cita centrada, autor y marca de agua.
3.  **`hero-layout`**: Título impactante con subtítulo y tag.

### Paletas (Themes)
Definidos en `lib/theme.typ`:
- `dark` / `light` (Neutros)
- `ocean` (Azules profundos y cian)
- `sunset` (Morados, rojos y naranjas)
- `forest` (Verdes naturales)

---

## 📂 Estructura del Proyecto

- `posts.toml`: Configuración de tus posts.
- `rrss`: Wrapper ejecutable (bash).
- `rrss.py`: Lógica de generación y procesamiento de imagen (Python).
- `lib/`: Componentes reutilizables de Typst (`theme`, `layouts`, `elements`).
- `assets/`: Imágenes de fondo y recursos.
- `output/`: Imágenes generadas (PNG).
