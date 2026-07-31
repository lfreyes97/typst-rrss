# 🚀 typst-rrss

Sistema modular de alto rendimiento para generar imágenes de redes sociales usando **Typst** y una CLI segura escrita en **Rust**.

Diseñado para crear contenido visual coherente (frases, artículos, estadísticas) de forma programática, estilizada y automatizada, garantizando robustez y seguridad en el procesamiento de datos.

---

## 🛡️ Seguridad y Robustez

Tras una auditoría exhaustiva, el sistema ha sido fortalecido con protecciones de grado de producción:

- **Prevención de Inyección de Comandos**: Ejecución segura de binarios externos (`typst`) mediante el paso de argumentos estructurados en lugar de interpolación de strings.
- **Validación de Rutas (Safe Path Validation)**: El sistema detecta y bloquea intentos de *Path Traversal*, restringiendo el acceso únicamente a directorios seguros (`/home`, `/tmp` y el directorio de trabajo).
- **Protección contra Agotamiento de Recursos (DoS)**: 
  - Límites estrictos de PPI (72 - 1200).
  - Validación de dimensiones de imagen (máximo 4K).
  - Control de tamaño de archivos de entrada (máximo 256MB).
- **Sanitización de Entradas**: Validación de longitud máxima y eliminación de bytes nulos en todos los strings de entrada (títulos, citas, marcas).
- **Escapado Automático**: Sanitización de caracteres especiales para el motor de Typst, evitando inyecciones de scripting en las plantillas.
- **Manejo de Errores Moderno**: Eliminación total de `panics` en el flujo principal, utilizando `anyhow` para una propagación de errores limpia y descriptiva.

---

## 🚀 Inicio Rápido

### Requisitos
- **Typst 0.11+** (para compilar las imágenes a PNG/PDF)
- **Rust 1.75+** (para compilar la CLI)

### Instalación

1. **Instalar el Paquete Typst localmente**
   Crea un enlace simbólico de la carpeta `rrss-pkg` hacia el directorio de paquetes locales de Typst:

```bash
mkdir -p ~/.local/share/typst/packages/local/rrss/0.1.0
ln -s $(pwd)/rrss-pkg/* ~/.local/share/typst/packages/local/rrss/0.1.0/
```

2. **Instalar la CLI**
   Compila e instala el binario en tu sistema:

```bash
cargo install --path rrss-cli-rs
```

### Uso Básico

```bash
# Generar todas las imágenes definidas en un posts.toml
rrss-cli-rs build

# Generar solo un post específico
rrss-cli-rs build --only ciencia

# Generar directamente con parámetros (con validaciones de seguridad automáticas)
rrss-cli-rs generate --title "Seguridad Primero" --brand "Typst-RRSS" --layout article --platform instagram-post --output post.typ
```

---

## 🛠️ CLI `rrss-cli-rs`

La CLI automatiza el flujo completo: validación, procesamiento, generación de código Typst y compilación.

### Comandos Principales

| Comando | Descripción |
| :--- | :--- |
| **`build`** | Genera y compila los posts definidos en `posts.toml`. |
| **`generate`** | Genera un archivo `.typ` basado en argumentos. Incluye sanitización. |
| **`full`** | Pipeline completo: genera `.typ` y compila a `.png` en un solo paso. |
| **`extract`** | Analiza una imagen para extraer paletas de colores y acentos. |
| **`colors`** | Genera esquemas de color armónicos desde un hex base. |

### 🪵 Logging y Observabilidad (Tracing)

El sistema utiliza `tracing` para proporcionar visibilidad en tiempo real:
- **Nivel INFO (default)**: Progreso de generación y compilación.
- **Errores Detallados**: Incluye el stderr del motor de Typst si la compilación falla, facilitando el debug.

---

## 📝 Configuración (`posts.toml`)

Control total de contenido y diseño. Ejemplo de configuración:

```toml
[defaults]
brand = "Mi Marca"
layout = "article"
theme = "dark"
platform = "instagram-post"

[[posts]]
name = "seguridad"
title = "Arquitectura Segura"
quote = "La seguridad no es un producto, es un proceso."
image = "assets/tech.jpg"
theme = "auto" # ✨ Modo Inteligente (Thaimeleon)
```

---

## 🎨 Layouts y Templates

Los diseños están definidos en Typst dentro del paquete `rrss-pkg/`.

### Layouts Soportados
- **`article`**: Contenido editorial.
- **`quote`**: Frases y citas destacadas.
- **`hero`**: Títulos de alto impacto.
- **`carousel`**: Secuencias para Instagram.

### Templates de Redes Sociales
- `instagram-post` / `instagram-story`
- `twitter-post` / `linkedin-post`
- `facebook-post` / `og-image`

---

## 📂 Estructura del Proyecto

- **`rrss-cli-rs/`**: CLI en Rust (motor de lógica y seguridad).
- **`rrss-pkg/`**: Paquete oficial de Typst (estilos y templates).
- **`assets/`**: Recursos y ejemplos multimedia.
- **`tests/`**: Suite de validación funcional.

---

## 📄 Licencia

Este proyecto se distribuye bajo la licencia MIT. Consulta el archivo `LICENSE` para más detalles.

