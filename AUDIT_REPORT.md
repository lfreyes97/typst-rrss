# 🔍 Auditoría de Código - `typst-rrss`

**Fecha:** 2024
**Versión auditada:** 0.1.0
**Nivel de severidad:** 🟡 MEDIO (5 issues críticos, 8 medios, 12 menores)

---

## 📋 Resumen Ejecutivo

El proyecto `typst-rrss` es una CLI bien estructurada en Rust que automatiza la generación de contenido visual para redes sociales. La arquitectura es sólida, pero hay **vulnerabilidades de seguridad importantes**, **problemas de manejo de errores** y **deuda técnica que debe resolverse antes de producción**.

### Puntuación General
- **Seguridad:** 6/10 ⚠️
- **Rendimiento:** 7/10 ✓
- **Mantenibilidad:** 7/10 ✓
- **Cobertura de errores:** 5/10 ⚠️
- **Calidad general:** 6.5/10

---

## 🔴 CRÍTICO (5 issues)

### 1. **Inyección de Comandos - Riesgo CRITICO**

**Ubicación:** `main.rs` líneas 319-326, 382-394, 456-463

**Problema:**
```rust
// ❌ INSEGURO - Sin validación de entrada
let status = Command::new("typst")
    .arg("compile")
    .arg("--root")
    .arg(&root)
    .arg(format!("--ppi={}", ppi))
    .arg(&target)  // ← Usuario puede controlar esto
    .arg(&output_path)
    .status();
```

Los argumentos `target`, `output_path`, `brand`, `title`, `quote` provienen directamente de:
- Línea de comandos (CLI)
- Archivo TOML sin validar
- Rutas de imágenes del usuario

**Impacto:** Un atacante podría inyectar comandos maliciosos:
```bash
./rrss generate --title '"; rm -rf /; echo "' --output /tmp/bad.typ
```

**Solución:**
```rust
// ✅ SEGURO - Usar array en lugar de strings formateados
let status = Command::new("typst")
    .arg("compile")
    .arg("--root")
    .arg(&root)
    .arg(format!("--ppi={}", ppi))
    .arg(&target)  // Separado como argumento, no interpolado
    .arg(&output_path)
    .status();

// Validar todas las rutas
fn validate_path(path: &str) -> Result<&str> {
    let p = Path::new(path);
    if !p.exists() {
        return Err(anyhow::anyhow!("Path does not exist: {}", path));
    }
    if p.is_absolute() && !p.starts_with("/home") && !p.starts_with("/tmp") {
        return Err(anyhow::anyhow!("Suspicious path: {}", path));
    }
    Ok(path)
}
```

**Severidad:** 🔴 CRÍTICO

---

### 2. **Edition de Cargo.toml Inválida**

**Ubicación:** `rrss-cli-rs/Cargo.toml` línea 3

```toml
edition = "2024"  # ❌ No existe
```

**Problema:** Rust solo soporta editions 2015, 2018, 2021. Esto causará fallo en compilación.

**Solución:**
```toml
edition = "2021"
```

**Severidad:** 🔴 CRÍTICO (impide compilación)

---

### 3. **Unwrap sin Validación - Panics en Producción**

**Ubicaciones múltiples:**

```rust
// images.rs:17-19
let root_path = Path::new(image_path).parent().unwrap();  // ❌ Panic si image_path = "/"
let stem = Path::new(image_path).file_stem().unwrap().to_str().unwrap();  // ❌ Double unwrap
let ext = Path::new(image_path).extension().unwrap().to_str().unwrap();  // ❌ Double unwrap

// main.rs:305
let name = target.file_stem().unwrap().to_str().unwrap();  // ❌ Panic posible

// colors.rs:40
let img = image::open(Path::new(image_path))?;  // ✓ OK, usa ?
let pixels = img.as_raw();  // ❌ Sin validación de tamaño
```

**Impacto:** La aplicación crashea en lugar de mostrar error útil.

**Solución:**
```rust
// ✅ MEJOR
fn extract_from_image(image_path: &str, count: u8) -> Result<Vec<color_thief::Color>> {
    let img = image::open(Path::new(image_path))
        .context("Failed to open image file")?;
    
    let img = img.to_rgba8();
    
    if img.width() * img.height() == 0 {
        return Err(anyhow::anyhow!("Image is empty"));
    }
    
    let pixels = img.as_raw();
    let palette = color_thief::get_palette(pixels, color_thief::ColorFormat::Rgba, 10, count)
        .context("Failed to extract color palette")?;
    Ok(palette)
}

// En images.rs
let root_path = Path::new(image_path).parent()
    .ok_or_else(|| anyhow::anyhow!("Invalid image path: {}", image_path))?;

let stem = Path::new(image_path).file_stem()
    .and_then(|s| s.to_str())
    .ok_or_else(|| anyhow::anyhow!("Invalid file stem: {}", image_path))?;
```

**Severidad:** 🔴 CRÍTICO

---

### 4. **Falta de Validación de Entrada - Potencial DoS**

**Ubicación:** `main.rs` Build command, función `resolve_theme`

```rust
// ❌ Sin límites ni validación
let count: u8;  // Rango 0-255
let ppi: u32;   // Rango 0-4.294B - podría causar DoS compilando imágenes huge

// Tampoco se valida:
// - Longitud de strings (title, quote, brand)
// - Tamaño de arrays (slides)
// - Valores de intensidad de recolor
```

**Impacto:**
```bash
./rrss extract myimage.jpg --count 255  # Extrae 255 colores, muy lento
./rrss compile main.typ --ppi 99999     # Intenta compilar a 99999 PPI
./rrss generate --title "$(cat /etc/passwd)" # String injection en Typst
```

**Solución:**
```rust
#[derive(Parser)]
enum Commands {
    Extract {
        image: PathBuf,
        
        #[arg(short, long, default_value_t = 8, value_parser = 1..=16)]
        count: u8,  // ✅ Limitado a 1-16
        
        format: ExtractFormat,
        suggest_accent: bool,
        name: String,
    },
    
    Compile {
        files: Vec<PathBuf>,
        
        #[arg(long, default_value_t = 144, value_parser = 72..=1200)]
        ppi: u32,  // ✅ Limitado a 72-1200 PPI
        
        output_dir: String,
        all: bool,
        template: Option<String>,
    },
}

// Validar strings
fn validate_string_input(s: &str, max_len: usize, field: &str) -> Result<()> {
    if s.len() > max_len {
        return Err(anyhow::anyhow!("{} exceeds max length of {}", field, max_len));
    }
    if s.contains('\0') {
        return Err(anyhow::anyhow!("{} contains null byte", field));
    }
    Ok(())
}

// En Build command:
validate_string_input(&title, 200, "title")?;
validate_string_input(&quote, 1000, "quote")?;
validate_string_input(&brand, 100, "brand")?;
```

**Severidad:** 🔴 CRÍTICO

---

### 5. **Gestión de Errores Inconsistente - Silenciamiento de Errores**

**Ubicación:** `main.rs` líneas 325-329, 463-467

```rust
// ❌ Ignora errores silenciosamente
match status {
    Ok(s) if s.success() => println!("✓"),
    _ => println!("✗ Error"),  // No muestra qué salió mal
}

// ❌ En Build command - continúa sin verificar resultado
let status = Command::new("typst")
    .arg("compile")
    ...
    .status();  // El resultado no se usa, ¡se ignora!

match status {
    Ok(s) if s.success() => println!("✓ Compiled"),
    _ => println!("✗ Compilation failed"),
}
```

**Impacto:** Errores silenciosos, difícil debugging, imágenes no generadas pasan desapercibidas.

**Solución:**
```rust
// ✅ MEJOR
let output = Command::new("typst")
    .arg("compile")
    .arg("--root")
    .arg(&root)
    .arg(format!("--ppi={}", ppi))
    .arg(&target)
    .arg(&output_path)
    .output()  // Captura stdout/stderr
    .context("Failed to execute typst")?;

if !output.status.success() {
    let stderr = String::from_utf8_lossy(&output.stderr);
    eprintln!("Typst compilation failed:\n{}", stderr);
    return Err(anyhow::anyhow!("Compilation failed for: {}", target.display()));
}
println!("✓ Compiled: {}", output_path.display());
```

**Severidad:** 🔴 CRÍTICO

---

## 🟠 MEDIO (8 issues)

### 6. **Duplicación Masiva de Código**

**Ubicación:** `main.rs` Commands::Generate vs Commands::Full

El comando `Full` duplica casi toda la lógica de `Generate`. Esto viola DRY (Don't Repeat Yourself).

```rust
// ❌ DUPLICADO - Mismo código en dos comandos
let mut final_accent = accent.clone();
if *auto_accent {
    if let Some(img_path) = image {
        final_accent = colors::suggest_accent(img_path);
        println!("Auto-extracted accent: {}", final_accent);
    }
}
let theme_map = resolve_theme(&theme, image.as_deref(), None);
let content = generate_typst_content(...)?;
fs::write(output, content)?;
```

**Solución:**
```rust
struct GenerateParams {
    brand: String,
    title: String,
    quote: String,
    image: Option<String>,
    logo: Option<String>,
    overlay: Option<String>,
    accent: String,
    auto_accent: bool,
    url: String,
    platform: String,
    layout: String,
    theme: String,
    author: String,
    tag: Option<String>,
    ppi: u32,
}

fn do_generate(params: &GenerateParams, cfg: Option<&config::Config>) -> Result<String> {
    let mut final_accent = params.accent.clone();
    if params.auto_accent {
        if let Some(img_path) = &params.image {
            final_accent = colors::suggest_accent(img_path);
        }
    }
    let theme_map = resolve_theme(&params.theme, params.image.as_deref(), cfg);
    generate_typst_content(...)
}

// Reutilizable en Generate, Full, y Build
```

**Severidad:** 🟠 MEDIO

---

### 7. **Falta de Logs Estructurados**

**Ubicación:** Todo `main.rs`

```rust
// ❌ Logs ad-hoc sin contexto
println!("Analyzing {:?}...", image);
println!("✓ Generated {} ({}, {})", output, layout, platform);
println!("    🎨 Recoloring with theme {} (intensity: {})", theme, recolor_intensity);
```

Sin logging estructurado es imposible:
- Debuggear en producción
- Automatizar pipelines CI/CD
- Medir rendimiento
- Auditar eventos

**Solución:**
```rust
// Agregar `tracing` o `log` a Cargo.toml
[dependencies]
log = "0.4"
env_logger = "0.11"

// O mejor, usar tracing:
[dependencies]
tracing = "0.1"
tracing-subscriber = "0.3"

// En main.rs
fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_max_level(Level::INFO)
        .init();
    
    let cli = Cli::parse();
    
    match &cli.command {
        Commands::Build { config_file, only, dry_run } => {
            tracing::info!("Starting build from: {}", config_file.display());
            
            for post in posts {
                tracing::debug!("Processing post: {}", name);
                if recolor {
                    tracing::info!(theme = %theme, intensity = %recolor_intensity, "Recoloring image");
                }
            }
        }
    }
}
```

**Severidad:** 🟠 MEDIO

---

### 8. **Gestión de Memoria en Imágenes Grandes**

**Ubicación:** `images.rs:recolor_image`

```rust
// ❌ Sin streaming, todo en memoria
let img = image::open(image_path)?;
let (width, height) = img.dimensions();
let mut buffer = ImageBuffer::new(width, height);  // Aloca width*height*4 bytes

// Para imagen 4K (4096x4096): ~67MB
// Para imagen 8K (8192x8192): ~268MB
// Sin límite = DoS potencial
```

**Impacto:** Procesador de imágenes extremadamente grande causa OOM.

**Solución:**
```rust
const MAX_IMAGE_DIMENSION: u32 = 4096;  // 4K
const MAX_IMAGE_SIZE_MB: u64 = 256;

pub fn recolor_image(
    image_path: &str,
    theme_name: &str,
    theme: &HashMap<String, String>,
    output_path: Option<&str>,
    intensity: f32,
) -> anyhow::Result<String> {
    let metadata = std::fs::metadata(image_path)?;
    if metadata.len() > MAX_IMAGE_SIZE_MB * 1024 * 1024 {
        return Err(anyhow::anyhow!(
            "Image file too large: {} MB (max: {} MB)",
            metadata.len() / (1024 * 1024),
            MAX_IMAGE_SIZE_MB
        ));
    }
    
    let img = image::open(image_path)?;
    let (width, height) = img.dimensions();
    
    if width > MAX_IMAGE_DIMENSION || height > MAX_IMAGE_DIMENSION {
        return Err(anyhow::anyhow!(
            "Image dimensions too large: {}x{} (max: {}x{})",
            width, height, MAX_IMAGE_DIMENSION, MAX_IMAGE_DIMENSION
        ));
    }
    
    // ... resto del código
}
```

**Severidad:** 🟠 MEDIO

---

### 9. **Thread Safety - Posible Race Condition**

**Ubicación:** `main.rs:Build` línea 428-470

```rust
// ❌ Sin sincronización, múltiples posts escriben en output/
for (i, post) in posts.iter().enumerate() {
    let typ_file = format!("{}.typ", name);  // ✓ Único
    let output_path = Path::new(output_dir).join(&out_filename);  // ✓ Único
    
    // Pero si dos compilaciones usan el mismo nombre...
    // ¿Y si el usuario corre ./rrss build dos veces simultáneamente?
}
```

Si se ejecutan dos builds concurrentes, pueden escribir el mismo archivo simultáneamente.

**Solución:**
```rust
use std::sync::Mutex;
use std::sync::Arc;

// Para builds concurrentes, usar un lock
let output_lock = Arc::new(Mutex::new(()));

for post in posts {
    let lock = Arc::clone(&output_lock);
    // O mejor, usar rayon para paralelizar:
    use rayon::prelude::*;
    
    posts.par_iter().try_for_each(|post| -> Result<()> {
        // Generar .typ
        let typ_file = format!("{}.typ", name);
        // Compilar
        // Rayon maneja la sincronización
        Ok(())
    })?;
}
```

**Severidad:** 🟠 MEDIO

---

### 10. **Validación Insuficiente de Archivos TOML**

**Ubicación:** `config.rs:Config::load`

```rust
// ❌ Sin validar estructura
pub fn load<P: AsRef<Path>>(path: P) -> Result<Self> {
    let content = fs::read_to_string(path)
        .context("Failed to read config file")?;
    let config: Config = toml::from_str(&content)
        .context("Failed to parse config file")?;
    Ok(config)
}

// No valida:
// - Valores requeridos ausentes
// - Tipos incorrectos
// - Rangos válidos
```

**Solución:**
```rust
impl Config {
    pub fn validate(&self) -> Result<()> {
        // Validar defaults
        if let Some(brand) = self.defaults.get("brand") {
            if !brand.is_str() {
                return Err(anyhow::anyhow!("defaults.brand must be a string"));
            }
        }
        
        // Validar posts
        if let Some(posts) = &self.posts {
            for (i, post) in posts.iter().enumerate() {
                if post.name.is_empty() {
                    return Err(anyhow::anyhow!("post[{}].name is required and cannot be empty", i));
                }
                
                if let Some(ppi) = post.ppi {
                    if ppi < 72 || ppi > 1200 {
                        return Err(anyhow::anyhow!("post[{}].ppi must be between 72 and 1200", i));
                    }
                }
                
                if let Some(intensity) = post.recolor_intensity {
                    if !(0.0..=1.0).contains(&intensity) {
                        return Err(anyhow::anyhow!("post[{}].recolor_intensity must be 0.0-1.0", i));
                    }
                }
            }
        }
        
        Ok(())
    }
}

// En main.rs
let cfg = config::Config::load(config_file)?;
cfg.validate()?;  // Validar después de cargar
```

**Severidad:** 🟠 MEDIO

---

### 11. **Color Hex Validation**

**Ubicación:** `colors.rs:hex_to_rgb_tuple`, `images.rs:recolor_image`

```rust
// ❌ Sin validar formato
pub fn hex_to_rgb_tuple(hex_color: &str) -> (u8, u8, u8) {
    let hex_color = hex_color.trim_start_matches('#');
    let r = u8::from_str_radix(&hex_color[0..2], 16).unwrap_or(0);  // ❌ unwrap_or silencia error
    let g = u8::from_str_radix(&hex_color[2..4], 16).unwrap_or(0);
    let b = u8::from_str_radix(&hex_color[4..6], 16).unwrap_or(0);
    (r, g, b)
}

// Permite: "#ZZZZZZ", "12345", "gg0000" → todo retorna negro (#000000)
```

**Solución:**
```rust
pub fn hex_to_rgb_tuple(hex_color: &str) -> Result<(u8, u8, u8)> {
    let hex_color = hex_color.trim_start_matches('#');
    
    if hex_color.len() != 6 {
        return Err(anyhow::anyhow!(
            "Invalid hex color: expected 6 characters, got {}",
            hex_color.len()
        ));
    }
    
    let r = u8::from_str_radix(&hex_color[0..2], 16)
        .context("Invalid hex color (R component)")?;
    let g = u8::from_str_radix(&hex_color[2..4], 16)
        .context("Invalid hex color (G component)")?;
    let b = u8::from_str_radix(&hex_color[4..6], 16)
        .context("Invalid hex color (B component)")?;
    
    Ok((r, g, b))
}

// Validar en accent
if !accent.starts_with('#') || accent.len() != 7 {
    return Err(anyhow::anyhow!("Accent must be valid hex like #RRGGBB, got: {}", accent));
}
```

**Severidad:** 🟠 MEDIO

---

### 12. **Falta de Rate Limiting en Extract**

**Ubicación:** `main.rs:Commands::Extract`

```rust
// ❌ Sin límite de tamaño de imagen a analizar
let extracted = colors::extract_from_image(image.to_str().unwrap(), *count)?;
```

Un atacante podría:
```bash
# Atacar con imagen 50GB
./rrss extract /mnt/huge_image.bin --count 255
```

**Solución:** Usar misma estrategia que `recolor_image` (punto 8).

**Severidad:** 🟠 MEDIO

---

## 🟡 MENOR (12 issues)

### 13. **Uso de `unwrap()` evitable**

**Ubicación:** Multiple
- `main.rs:316` - `target.file_stem().unwrap().to_str().unwrap()`
- `colors.rs:24-26` - `u8::from_str_radix(...).unwrap_or(0)` (issue 11)

### 14. **PPI por defecto inconsistente**

```rust
// main.rs:121: Full command
#[arg(long, default_value_t = 144)]
ppi: u32,

// main.rs:104: Compile command
#[arg(long, default_value_t = 144)]
ppi: u32,

// posts.toml defaults
ppi = 144

// Pero en Build command se lee de defaults sin validar rango
```

### 15. **Falta de Documentación de Seguridad**

No existe:
- `SECURITY.md`
- Guía de prácticas seguras
- Validación de entrada documentada
- Límites de recurso documentados

### 16. **Error Handling Asimétrico**

En algunos lugares usa `?`, en otros `match`, en otros `unwrap_or`. Inconsistente.

### 17. **Sin Pruebas Unitarias**

No hay carpeta `tests/`, sin test coverage.

### 18. **Contour Generation Incompleta**

```rust
// images.rs:83-94
pub fn generate_contours(image_path: &str) -> anyhow::Result<String> {
    // Placeholder implementation
    let img = image::open(image_path)?;
    let gray = img.grayscale();
    // Saving grayscale as "contour" for now. A real implementation would need edge detection kernel.
}

// ❌ TODO sin implementar
pub fn generate_noise_contours(_root_path: &Path) -> anyhow::Result<String> {
     Err(anyhow::anyhow!("Noise contour generation not yet implemented"))
}
```

### 19. **SVG Recoloring Inseguro**

```rust
// main.rs:502
if l.to_lowercase().ends_with(".svg") {
    format!("logo: recolor-svg(\"{}\", t.text, original: \"currentColor\"),", l)
}

// ❌ Sin validar que el SVG existe o es válido
// Typst podría fallar silenciosamente
```

### 20. **Slide Parsing Frágil**

```rust
// main.rs:426-434
let slides_str: Option<String> = if let Some(s) = &post.slides {
    Some(s.join("|"))
} else if let Some(val) = defaults.get("slides") {
    if let Some(arr) = val.as_array() {
        Some(arr.iter().filter_map(|v| v.as_str()).collect::<Vec<_>>().join("|"))
    } else {
        None
    }
} else {
    None
};

// ❌ Si un slide contiene "|", se rompe el parsing
// Mejor usar JSON o escaping
```

### 21. **No Hay Soporte para Rutas Relativas Seguras**

```rust
// ❌ Ruta relativa potencialmente peligrosa
let img = format!("../../../etc/passwd");
```

### 22. **Typo/Error: Cargo.toml Edition**

Ya listado en #2, pero merece mención.

### 23. **Sin Respeto por .gitignore**

El archivo `.gitignore` probablemente lista `output/`, pero si una compilación falla, puede quedar basura.

### 24. **Falta de Progress Indication para Builds Largos**

Para múltiples posts, no hay indicador de progreso (1/20, 2/20...).

---

## ✅ Aspectos Positivos

1. ✓ Uso de `anyhow` para error handling
2. ✓ Estructura modular (colors, templates, images, config)
3. ✓ CLI bien documentada con `clap`
4. ✓ Soporte para múltiples formatos de salida
5. ✓ Tema automático basado en imágenes (inteligente)
6. ✓ Pipeline visual descriptivo (símbolos emoji)
7. ✓ Configuración centralizada en TOML

---

## 🔧 Recomendaciones Prioritarias

### Semana 1 (BLOQUERS)
1. Fijar `edition = "2021"` en Cargo.toml
2. Implementar validación de strings y límites PPI
3. Cambiar `Command` a usar arrays, no strings interpolados
4. Reemplazar todos los `unwrap()` con `.context()?`

### Semana 2
5. Agregar logging estructurado con `tracing`
6. Implementar validación de TOML en `Config::validate()`
7. Extraer lógica común en función `do_generate()`
8. Agregar límites de tamaño de imagen

### Semana 3
9. Escribir pruebas unitarias (mínimo 50% coverage)
10. Agregar `SECURITY.md` con guía de prácticas
11. Completar `generate_contours()` y `generate_noise_contours()`
12. Documentar validación de entrada

---

## 📊 Matriz de Riesgo

```
┌─────────────────────────────────────────────────────┐
│   IMPACTO vs PROBABILIDAD                           │
├─────────────────────────────────────────────────────┤
│  CRÍTICO   │ #1 (inyección) #2 (edition)           │
│  ALTO      │ #3 (unwraps)  #5 (errores silenc.)    │
│  MEDIO     │ #4 (validación)  #8 (DoS mem)         │
│  BAJO      │ #13-24 (menores)                      │
└─────────────────────────────────────────────────────┘
```

---

## 📝 Conclusión

El proyecto tiene **buena arquitectura pero necesita hardening de seguridad antes de usar en producción**. Los issues críticos (#1-5) deben resolverse inmediatamente. Los medios (#6-12) dentro de 2-3 semanas.

**Recomendación:** 🟡 **NO USAR EN PRODUCCIÓN** hasta que los 5 críticos sean solucionados.

**Revisión sugerida:** Después de implementar recomendaciones prioritarias.
