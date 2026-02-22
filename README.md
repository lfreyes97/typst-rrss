# typst-rrss

Sistema modular para generar imágenes de redes sociales usando **Typst** y una CLI de alto rendimiento en **Rust**.

Diseñado para crear contenido visual coherente (frases, artículos, estadísticas) de forma programática, estilizada y automatizada.

## 🚀 Inicio Rápido

### Requisitos
- **Typst** (para compilar las imágenes a PNG/PDF)
- **Rust** (para compilar la CLI)

### Instalación

El sistema se divide en dos partes: el paquete de diseño de Typst y el CLI de Rust.

1. **Instalar el Paquete Typst localmente**
   Crea un enlace simbólico de la carpeta `rrss-pkg` hacia donde Typst busca paquetes locales:

```bash
mkdir -p ~/.local/share/typst/packages/local/rrss/0.1.0
ln -s $(pwd)/rrss-pkg/* ~/.local/share/typst/packages/local/rrss/0.1.0/
```

2. **Instalar la CLI de manera Global**
   Usa Cargo para instalar el binario en tu sistema, lo que te permitirá ejecutar `rrss` desde cualquier carpeta:

```bash
cargo install --path rrss-cli-rs
```

### Uso Básico

Una vez instalada, puedes usar la CLI desde cualquier carpeta para generar posts basados en un archivo TOML o argumentos directos.

```bash
# Generar todas las imágenes definidas en un posts.toml local
rrss-cli-rs build

# Generar solo un post específico desde el TOML
rrss-cli-rs build --only ciencia

# Generar directamente con parámetros de consola
rrss-cli-rs generate --title "Hola Mundo" --brand "Mi Marca" --layout article --platform instagram-post --output post.typ
typst compile post.typ post.png
```

---

## 🛠️ CLI `rrss-cli-rs`

El ejecutable `rrss-cli-rs` automatiza todo el flujo: parseo de configuración, procesamiento de imágenes, generación de código Typst y compilación usando el paquete global `@local/rrss:0.1.0`.

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

Control total de tu contenido y diseño en un solo archivo. Puedes tener este archivo en cualquier carpeta de tu ordenador.

### Uso con `.toml`

Simplemente crea un archivo `posts.toml` en tu carpeta actual:

```toml
[defaults]
brand = "Mi Marca"
layout = "article"
theme = "dark"
platform = "instagram-post"

# Post 1: Usa el tema por defecto ("dark")
[[posts]]
name = "oceano"
title = "La Roca de los Siglos"
quote = "Una cita inspiradora aquí..."
image = "assets/mar.jpg"
overlay = "assets/Solid-bg.svg"

# Post 2: Extrae colores automáticamente de la imagen
[[posts]]
name = "naturaleza"
title = "Creación"
image = "assets/bosque.jpg"
theme = "auto"      # ✨ MODO AUTOMÁTICO
```

Luego, simplemente ejecuta:
```bash
rrss-cli-rs build
```

---

## 🎨 Layouts y Templates

Los diseños están definidos en Typst dentro del paquete `rrss-pkg/` y pueden ser usados independientemente de la CLI usando:
```typst
#import "@local/rrss:0.1.0": *
```

### Layouts Soportados
- **`article`**: Para contenido tipo blog o ensayo.
- **`quote`**: Para citas destacadas.
- **`hero`**: Para títulos grandes o portadas.
- **`carousel`**: Para secuencias de imágenes (Instagram Carousels).

### Templates de Redes Sociales
Listos para usar con dimensiones específicas:
- `instagram-post` (1080x1080)
- `instagram-story` (1080x1920)
- `twitter-post` (1600x900)
- `linkedin-post` (1200x627)
- `facebook-post` (1200x630)
- `og-image` (1200x630)

---

## 📂 Estructura del Proyecto

- **`posts.toml`**: Ejemplo de configuración.
- **`rrss-cli-rs/`**: Código fuente de la CLI global en Rust.
- **`rrss-pkg/`**: Paquete oficial de Typst que contiene temas, layouts y elementos de diseño.
  - `lib/`: Librería Typst modular.
  - `templates/`: Plantillas para cada plataforma.
- **`content/`**: Ejemplos independientes construidos en Typst puro.
- **`assets/`**: Imágenes y recursos de ejemplo.
