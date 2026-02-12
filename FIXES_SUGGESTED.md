# 🔧 Fixes Sugeridos - Código Corregido

Este documento contiene ejemplos de código corregido para los 5 issues críticos identificados en la auditoría.

---

## 🔴 FIX #1: Inyección de Comandos

### ANTES (Inseguro)
```rust
// main.rs - línea ~320
let status = Command::new("typst")
    .arg("compile")
    .arg("--root")
    .arg(&root)
    .arg(format!("--ppi={}", ppi))  // ❌ Vulnerable a inyección
    .arg(&target)
    .arg(&output_path)
    .status();
```

### DESPUÉS (Seguro)
```rust
// main.rs - crear nueva función helper
fn compile_typst(
    input: &Path,
    output: &Path,
    ppi: u32,
    root: &Path,
) -> Result<bool> {
    // Validar entrada
    validate_path(input)?;
    validate_path(output)?;
    
    if ppi < 72 || ppi > 1200 {
        return Err(anyhow!("PPI must be between 72 and 1200, got {}", ppi));
    }
    
    // Ejecutar con argumentos separados (no interpolación)
    let output = Command::new("typst")
        .arg("compile")
        .arg("--root")
        .arg(root)
        .arg("--ppi")           // ✅ Argumento separado
        .arg(ppi.to_string())   // ✅ Valor separado
        .arg(input)
        .arg(output)
        .output()  // Capturar output completo
        .context("Failed to execute typst")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        eprintln!("Typst error:\n{}", stderr);
        return Err(anyhow!("Typst compilation failed"));
    }

    Ok(true)
}

fn validate_path(path: &Path) -> Result<()> {
    // No permitir rutas absolutas peligrosas
    if path.is_absolute() {
        if !path.to_string_lossy().starts_with("/tmp") 
            && !path.to_string_lossy().starts_with("/home")
            && !path.to_string_lossy().starts_with(&std::env::current_dir()?.to_string_lossy().to_string())
        {
            return Err(anyhow!("Path traversal attempt detected: {}", path.display()));
        }
    }
    Ok(())
}

// Luego en main.rs, reemplazar todos los Command::new("typst"):
// Antes:
match status {
    Ok(s) if s.success() => println!("✓"),
    _ => println!("✗ Error"),
}

// Después:
match compile_typst(&target, &output_path, ppi, &root) {
    Ok(_) => println!("✓ Compiled: {}", output_path.display()),
    Err(e) => {
        eprintln!("✗ Compilation error: {}", e);
        return Err(e);  // Propagar error en lugar de ignorar
    }
}
```

---

## 🔴 FIX #2: Edition = "2024" (Cargo.toml)

### ANTES (Inválido)
```toml
[package]
name = "rrss-cli-rs"
version = "0.1.0"
edition = "2024"  # ❌ NO EXISTE
```

### DESPUÉS (Válido)
```toml
[package]
name = "rrss-cli-rs"
version = "0.1.0"
edition = "2021"  # ✅ Última edición estable

[dependencies]
anyhow = "1.0.101"
clap = { version = "4.5.58", features = ["derive"] }
color-thief = "0.2.2"
image = "0.25.9"
noise = "0.9.0"
palette = { version = "0.7.6", features = ["std"] }
rand = "0.10.0"
rayon = "1.11.0"
serde = { version = "1.0.228", features = ["derive"] }
serde_json = "1.0.149"
toml = "1.0.0"
walkdir = "2.5.0"
# Agregar para logging
tracing = "0.1"
tracing-subscriber = "0.3"
```

---

## 🔴 FIX #3: Unwraps sin Validación

### ANTES (Panics)
```rust
// images.rs - línea ~15-19
pub fn recolor_image(
    image_path: &str,
    theme_name: &str,
    theme: &HashMap<String, String>,
    output_path: Option<&str>,
    intensity: f32,
) -> anyhow::Result<String> {
    let img = image::open(image_path)?;
    let (width, height) = img.dimensions();
    let root_path = Path::new(image_path).parent().unwrap();  // ❌ Panic si sin padre
    let stem = Path::new(image_path).file_stem().unwrap().to_str().unwrap();  // ❌ Double unwrap
    let ext = Path::new(image_path).extension().unwrap().to_str().unwrap();  // ❌ Double unwrap
    
    // ... resto del código
}
```

### DESPUÉS (Seguro)
```rust
// images.rs
pub fn recolor_image(
    image_path: &str,
    theme_name: &str,
    theme: &HashMap<String, String>,
    output_path: Option<&str>,
    intensity: f32,
) -> anyhow::Result<String> {
    // Validar tamaño
    let metadata = std::fs::metadata(image_path)?;
    const MAX_SIZE_MB: u64 = 256;
    if metadata.len() > MAX_SIZE_MB * 1024 * 1024 {
        return Err(anyhow!(
            "Image too large: {} MB (max: {} MB)",
            metadata.len() / (1024 * 1024),
            MAX_SIZE_MB
        ));
    }

    let img = image::open(image_path)?;
    let (width, height) = img.dimensions();
    
    const MAX_DIMENSION: u32 = 4096;
    if width > MAX_DIMENSION || height > MAX_DIMENSION {
        return Err(anyhow!(
            "Image dimensions too large: {}x{} (max: {}x{})",
            width, height, MAX_DIMENSION, MAX_DIMENSION
        ));
    }

    // ✅ Usar context() en lugar de unwrap()
    let root_path = Path::new(image_path)
        .parent()
        .ok_or_else(|| anyhow!("Invalid image path: no parent directory"))?;

    let stem = Path::new(image_path)
        .file_stem()
        .and_then(|s| s.to_str())
        .ok_or_else(|| anyhow!("Invalid image filename"))?;

    let ext = Path::new(image_path)
        .extension()
        .and_then(|s| s.to_str())
        .ok_or_else(|| anyhow!("Invalid image extension"))?;

    let default_output = root_path.join(format!("{}_{}.{}", stem, theme_name, ext));
    let out_path = output_path.map(PathBuf::from).unwrap_or(default_output);
    
    let is_jpeg = out_path
        .extension()
        .map_or(false, |e| {
            let s = e.to_string_lossy().to_lowercase();
            s == "jpg" || s == "jpeg"
        });

    let bg_hex = theme
        .get("bg")
        .ok_or_else(|| anyhow!("Theme missing 'bg' color"))?;
    let primary_hex = theme
        .get("primary")
        .ok_or_else(|| anyhow!("Theme missing 'primary' color"))?;

    let bg_color = Srgb::from_str(bg_hex)
        .map_err(|_| anyhow!("Invalid bg color: {}", bg_hex))?
        .into_linear();
    let primary_color = Srgb::from_str(primary_hex)
        .map_err(|_| anyhow!("Invalid primary color: {}", primary_hex))?
        .into_linear();

    let mut buffer = ImageBuffer::new(width, height);

    for (x, y, pixel) in img.pixels() {
        let (r, g, b, a) = (pixel[0], pixel[1], pixel[2], pixel[3]);

        if a == 0 && !is_jpeg {
            buffer.put_pixel(x, y, Rgba([0, 0, 0, 0]));
            continue;
        }

        let original = Srgb::new(r as f32 / 255.0, g as f32 / 255.0, b as f32 / 255.0).into_linear();
        let gray = original.red * 0.2126 + original.green * 0.7152 + original.blue * 0.0722;
        let duotone = bg_color.mix(primary_color, gray);
        let blended = original.mix(duotone, intensity);
        let final_srgb: Srgb = Srgb::from_linear(blended);

        buffer.put_pixel(x, y, Rgba([
            (final_srgb.red * 255.0).round() as u8,
            (final_srgb.green * 255.0).round() as u8,
            (final_srgb.blue * 255.0).round() as u8,
            a
        ]));
    }

    if is_jpeg {
        let dynamic = image::DynamicImage::ImageRgba8(buffer);
        let rgb_img = dynamic.to_rgb8();
        rgb_img.save(&out_path)?;
    } else {
        buffer.save(&out_path)?;
    }

    Ok(out_path.to_string_lossy().to_string())
}
```

---

## 🔴 FIX #4: Validación de Entrada Débil

### ANTES (Sin Validar)
```rust
// colors.rs - línea ~2
pub fn hex_to_rgb_tuple(hex_color: &str) -> (u8, u8, u8) {
    let hex_color = hex_color.trim_start_matches('#');
    let r = u8::from_str_radix(&hex_color[0..2], 16).unwrap_or(0);  // ❌ Panic si len < 2
    let g = u8::from_str_radix(&hex_color[2..4], 16).unwrap_or(0);
    let b = u8::from_str_radix(&hex_color[4..6], 16).unwrap_or(0);
    (r, g, b)
}
```

### DESPUÉS (Validado)
```rust
// colors.rs
use anyhow::{Result, Context};

pub fn hex_to_rgb_tuple(hex_color: &str) -> Result<(u8, u8, u8)> {
    let hex_color = hex_color.trim_start_matches('#');

    if hex_color.len() != 6 {
        return Err(anyhow!(
            "Invalid hex color: expected 6 characters, got '{}' (length: {})",
            hex_color,
            hex_color.len()
        ));
    }

    if !hex_color.chars().all(|c| c.is_ascii_hexdigit()) {
        return Err(anyhow!("Invalid hex color: contains non-hexadecimal characters: {}", hex_color));
    }

    let r = u8::from_str_radix(&hex_color[0..2], 16)
        .context("Invalid red component")?;
    let g = u8::from_str_radix(&hex_color[2..4], 16)
        .context("Invalid green component")?;
    let b = u8::from_str_radix(&hex_color[4..6], 16)
        .context("Invalid blue component")?;

    Ok((r, g, b))
}

pub fn hex_to_hsl(hex_color: &str) -> Result<(f32, f32, f32)> {
    let (r, g, b) = hex_to_rgb_tuple(hex_color)?;  // ✅ Ahora con Result
    let srgb = Srgb::new(r as f32 / 255.0, g as f32 / 255.0, b as f32 / 255.0);
    let hsl = Hsl::from_color(srgb);
    Ok((hsl.hue.into_degrees(), hsl.saturation, hsl.lightness))
}

// En main.rs, agregar validación de entrada
fn validate_string_input(s: &str, max_len: usize, field_name: &str) -> Result<()> {
    if s.is_empty() {
        return Err(anyhow!("{} cannot be empty", field_name));
    }

    if s.len() > max_len {
        return Err(anyhow!(
            "{} exceeds maximum length of {} characters",
            field_name,
            max_len
        ));
    }

    if s.contains('\0') {
        return Err(anyhow!("{} contains null bytes", field_name));
    }

    // Evitar inyección de caracteres especiales de Typst
    if s.contains('"') || s.contains('{') || s.contains('}') {
        // No rechazarlo, pero escaparlo más adelante
    }

    Ok(())
}

// Usar en Generate command:
validate_string_input(&title, 200, "title")?;
validate_string_input(&quote, 1000, "quote")?;
validate_string_input(&brand, 100, "brand")?;
```

---

## 🔴 FIX #5: Gestión de Errores Inconsistente

### ANTES (Silencia Errores)
```rust
// main.rs - línea ~325-329
match status {
    Ok(s) if s.success() => println!("✓"),
    _ => println!("✗ Error"),  // ❌ No muestra qué pasó
}

// En Build command - línea ~463-467
let status = Command::new("typst")
    // ...
    .status();  // ❌ Retorno ignorado, no se propaga

match status {
    Ok(s) if s.success() => println!("✓ Compiled"),
    _ => println!("✗ Compilation failed"),  // ❌ Sin detalles
}
```

### DESPUÉS (Maneja Errores Correctamente)
```rust
// Crear nuevo módulo: src/process.rs
use std::process::Command;
use anyhow::{Result, Context};
use std::path::Path;

pub fn compile_typst_verbose(
    input: &Path,
    output: &Path,
    ppi: u32,
    root: &Path,
) -> Result<()> {
    eprintln!("  Compiling: {}", input.display());

    let mut cmd = Command::new("typst");
    cmd.arg("compile")
        .arg("--root")
        .arg(root)
        .arg("--ppi")
        .arg(ppi.to_string())
        .arg(input)
        .arg(output);

    eprintln!("  Command: {:?}", cmd);  // Debug info

    let output = cmd.output()
        .context("Failed to execute typst binary. Is it installed?")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);

        return Err(anyhow!(
            "Typst compilation failed:\n--- STDERR ---\n{}\n--- STDOUT ---\n{}",
            stderr,
            stdout
        ));
    }

    eprintln!("  ✓ Success: {}", output.display());
    Ok(())
}

// En main.rs, reemplazar el bloque de compilación:
// ANTES:
match status {
    Ok(s) if s.success() => println!("✓"),
    _ => println!("✗ Error"),
}

// DESPUÉS:
match process::compile_typst_verbose(&target, &output_path, ppi, &root) {
    Ok(_) => println!("    ✓ Compiled to: {}", output_path.display()),
    Err(e) => {
        eprintln!("    ✗ Failed: {}", e);
        // En Build, continuar con próximo post; en Full, propagar error
        if is_build_mode {
            continue;  // Siguiente post
        } else {
            return Err(e);  // Propagar
        }
    }
}

// Actualizar Cargo.toml para agregar logging:
[dependencies]
tracing = "0.1"
tracing-subscriber = "0.3"

// En main.rs setup:
fn setup_logging() {
    use tracing_subscriber::fmt;
    fmt()
        .with_max_level(tracing::Level::INFO)
        .init();
}

// En main():
fn main() -> Result<()> {
    setup_logging();
    tracing::info!("Starting rrss-cli");

    let cli = Cli::parse();
    let root = constants::project_root();

    match &cli.command {
        Commands::Build { config_file, only, dry_run } => {
            tracing::info!("Building from: {}", config_file.display());
            // ...
            for post in posts {
                tracing::debug!("Processing post: {}", name);
                // ...
            }
        }
        // ... etc
    }

    Ok(())
}
```

---

## 📋 Checklist de Aplicación

- [ ] Actualizar `Cargo.toml`: cambiar edition a "2021" + agregar tracing
- [ ] Crear `src/process.rs` con función `compile_typst_verbose()`
- [ ] Reemplazar todo `Command::new("typst")` con llamadas a nueva función
- [ ] Actualizar `colors.rs`: cambiar `hex_to_rgb_tuple()` a retornar `Result`
- [ ] Agregar `validate_string_input()` en `main.rs`
- [ ] Actualizar `images.rs` con validación de tamaño
- [ ] Cambiar todos los `println!` a `tracing::info!()` o `eprintln!()` para errores
- [ ] Probar compilación: `cargo build --release`
- [ ] Probar con entrada malformada: `./rrss colors "#1"` (debe dar error, no panic)
- [ ] Probar con imagen grande: crear imagen 5GB (debe rechazarse)

---

## 🧪 Pruebas Sugeridas

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hex_to_rgb_valid() {
        assert_eq!(hex_to_rgb_tuple("#FF0000").unwrap(), (255, 0, 0));
        assert_eq!(hex_to_rgb_tuple("#00FF00").unwrap(), (0, 255, 0));
        assert_eq!(hex_to_rgb_tuple("#0000FF").unwrap(), (0, 0, 255));
    }

    #[test]
    fn test_hex_to_rgb_invalid_length() {
        assert!(hex_to_rgb_tuple("#1").is_err());
        assert!(hex_to_rgb_tuple("#12345").is_err());
        assert!(hex_to_rgb_tuple("#12345678").is_err());
    }

    #[test]
    fn test_hex_to_rgb_invalid_chars() {
        assert!(hex_to_rgb_tuple("#GGGGGG").is_err());
        assert!(hex_to_rgb_tuple("#!@#$%^").is_err());
    }

    #[test]
    fn test_validate_string_input_valid() {
        assert!(validate_string_input("hello", 100, "test").is_ok());
        assert!(validate_string_input("a very long string", 100, "test").is_ok());
    }

    #[test]
    fn test_validate_string_input_too_long() {
        let long_string = "x".repeat(101);
        assert!(validate_string_input(&long_string, 100, "test").is_err());
    }

    #[test]
    fn test_validate_string_input_null_byte() {
        assert!(validate_string_input("hello\0world", 100, "test").is_err());
    }

    #[test]
    fn test_path_traversal_blocked() {
        assert!(validate_path(Path::new("../../etc/passwd")).is_err());
        assert!(validate_path(Path::new("/etc/passwd")).is_err());
    }
}
```

---

## 📞 Preguntas Comunes

**P: ¿Por qué cambiar todos los `format!("--ppi={}", ppi)` a argumentos separados?**
R: Porque `format!()` interpola strings, permitiendo inyección. Con argumentos separados, cada uno se pasa como token opaco.

**P: ¿Por qué validar tamaño de imagen?**
R: Para prevenir DoS. Alguien podría enviar imagen 1TB que aloca toda la memoria.

**P: ¿Rayon no debería usarse para paralelización?**
R: Sí, pero primero resolver seguridad. Después, envolver loop en `posts.par_iter()`.

**P: ¿Y la inyección en templates Typst?**
R: Escapar caracteres especiales: `"` → `\"`, `{` → needs careful handling.

---

## 🚀 Próximos Pasos

1. Aplicar estos 5 fixes
2. `cargo build --release` (debe compilar sin errores)
3. Correr tests: `cargo test`
4. Probar manualmente con entrada maliciosa
5. Review de seguridad adicional
6. Desplegar a staging/producción