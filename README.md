# typst-rrss

Sistema modular para generar imágenes de redes sociales usando **Typst** y una CLI de alto rendimiento en **Rust**.

Diseñado para crear contenido visual coherente (frases, artículos, estadísticas) de forma programática, estilizada y automatizada.

## 🚀 Inicio Rápido

### Requisitos
- **Typst** (para compilar las imágenes a PNG/PDF)
- **Rust** (para compilar la CLI)

### Instalación

```bash
# Compilar la CLI
cd rrss-cli-rs
cargo build --release
cd ..

# (Opcional) Crear un alias o symlink
ln -s rrss-cli-rs/target/release/rrss-cli-rs rrss
```

### Uso Básico

```bash
# Generar todas las imágenes definidas en posts.toml
./rrss build

# Generar solo un post específico
./rrss build --only ciencia

# Extraer paleta de colores de una imagen
./rrss extract assets/mi-imagen.jpg
```

---

## 🛠️ CLI `rrss-cli-rs`

El ejecutable `rrss` (o `rrss-cli-rs`) automatiza todo el flujo: parseo de configuración, procesamiento de imágenes, generación de código Typst y compilación.

### Comandos Principales

| Comando | Descripción |
| :--- | :--- |
| **`build`** | Genera y compila los posts definidos en `posts.toml`. Opciones: `--only <name>`, `--dry-run`. |
| **`generate`** | Genera un *único* archivo `.typ` basado en argumentos de línea de comandos. |
| **`full`** | Pipeline completo para un solo post (generar `.typ` + compilar a `.png`). |
| **`extract`** | Analiza una imagen y extrae una paleta de colores dominante y sugerencias de acento. |
| **`compile`** | Utilería para compilar manualmente archivos `.typ` en lote. |
| **`colors`** | Genera esquemas de color armónicos a partir de un color base hexadecimal. |

---

## 📝 Configuración (`posts.toml`)

Control total de tu contenido y diseño en un solo archivo.

### Definición de Temas (`[themes]`)

Puedes definir tus propias paletas de colores. El sistema soporta dos modos de coloreado:

1.  **Modo Recolor (Predefinido)**: Usa una paleta fija y tiñe la imagen de fondo.
2.  **Modo Extracción (Auto)**: Extrae colores de la imagen automáticamente.

```toml
# Definir un tema personalizado
[themes.ocean]
bg = "#0a192f"
text = "#ccd6f6"
primary = "#64ffda"
secondary = "#8892b0"
accent = "#112240"
surface = "#112240"

# Valores por defecto para todos los posts
[defaults]
brand = "Mi Marca"
layout = "article"
theme = "dark"

# Post 1: Usa el tema "ocean" y recolorea la imagen
[[post]]
name = "oceano"
title = "La Roca de los Siglos"
image = "assets/mar.jpg"
theme = "ocean"
recolor = true      # Aplica duotone con los colores del tema
recolor_intensity = 0.7

# Post 2: Extrae colores automáticamente de la imagen
[[post]]
name = "naturaleza"
title = "Creación"
image = "assets/bosque.jpg"
theme = "auto"      # ✨ MODO AUTOMÁTICO
```

---

## 🎨 Layouts y Templates

Los diseños están definidos en Typst (`lib/layouts/*.typ` y `templates/*.typ`) y reciben dinámicamente la paleta de colores desde Rust.

### Layouts Soportados
- **`article`**: Para contenido tipo blog o ensayo.
- **`quote`**: Para citas destacadas.
- **`hero`**: Para títulos grandes o portadas.
- **`carousel`**: Para secuencias de imágenes (Instagram Carousels).

### Templates de Redes Sociales
Archivos en `templates/` listos para usar con dimensiones específicas:
- `instagram-post` (1080x1080)
- `instagram-story` (1080x1920)
- `twitter-post` (1600x900)
- `linkedin-post` (1200x627)
- `facebook-post` (1200x630)
- `og-image` (1200x630)

---

## 📂 Estructura del Proyecto

- **`posts.toml`**: Archivo maestro de configuración.
- **`rrss-cli-rs/`**: Código fuente de la CLI en Rust.
- **`lib/`**: Librería Typst modular (`layouts`, `elements`).
- **`templates/`**: Plantillas `.typ` para cada plataforma.
- **`assets/`**: Imágenes y recursos.
- **`output/`**: Resultados generados.
