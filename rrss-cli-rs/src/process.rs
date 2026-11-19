use anyhow::{anyhow, Context, Result};
use std::path::Path;
use std::process::Command;
use tracing::info;

/// Safely compile a Typst file using the system's typst binary.
/// This function prevents command injection by using separate arguments
/// instead of string interpolation and validates the input paths.
///
/// Si compila, devuelve los avisos de Typst (su stderr, vacío si no hubo).
pub fn compile_typst(input: &Path, output: &Path, ppi: u32, root: &Path) -> Result<String> {
    compile_typst_with(input, output, ppi, root, &[])
}

/// Como `compile_typst`, con carpetas de fuentes extra (`typst --font-path`).
pub fn compile_typst_with(
    input: &Path,
    output: &Path,
    ppi: u32,
    root: &Path,
    font_paths: &[std::path::PathBuf],
) -> Result<String> {
    // 1. Path validation to prevent traversal outside allowed directories
    validate_path(input)?;
    validate_path(output)?;

    // 2. PPI validation to prevent Resource Exhaustion (DoS)
    if !(72..=1200).contains(&ppi) {
        return Err(anyhow!("ppi debe estar entre 72 y 1200, no {ppi}"));
    }

    info!(
        "Compiling {} -> {} (PPI: {})",
        input.display(),
        output.display(),
        ppi
    );

    // 3. Command execution using separate arguments to prevent Command Injection
    let mut cmd = Command::new("typst");
    cmd.arg("compile").arg("--root").arg(root);
    for f in font_paths {
        cmd.arg("--font-path").arg(f);
    }
    let output_result = cmd
        .arg("--ppi")
        .arg(ppi.to_string())
        .arg(input)
        .arg(output)
        .output()
        .context("No se pudo ejecutar `typst`. ¿Está instalado y en el PATH?")?;

    let stderr = String::from_utf8_lossy(&output_result.stderr);
    if !output_result.status.success() {
        if stderr.contains("multiple images") {
            return Err(anyhow!(
                "la pieza ocupa más de una página (el texto se desbordó): baja el tamaño o los márgenes"
            ));
        }
        return Err(anyhow!("Typst no pudo compilar: {}", stderr.trim()));
    }

    Ok(stderr.trim().to_string())
}

/// Los avisos de Typst, sangrados bajo la pieza a la que pertenecen.
pub fn mostrar_avisos(avisos: &str, sangria: &str) {
    for (i, l) in avisos.lines().enumerate() {
        let marca = if i == 0 { "⚠ " } else { "  " };
        println!("{sangria}{marca}{l}");
    }
}

/// Validates that a path is safe to use.
/// Prevents access to sensitive system files through path traversal.
pub fn validate_path(path: &Path) -> Result<()> {
    // If the path is absolute, it must stay within /home or /tmp or current_dir
    if path.is_absolute() {
        let path_str = path.to_string_lossy();
        if !path_str.starts_with("/home") && !path_str.starts_with("/tmp") {
            let current_dir =
                std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."));
            if !path_str.starts_with(current_dir.to_string_lossy().as_ref()) {
                return Err(anyhow!(
                    "ruta no permitida (fuera de /home, /tmp y la carpeta actual): {}",
                    path.display()
                ));
            }
        }
    }

    // Check for null bytes which can be used in some injection attacks
    if path.to_string_lossy().contains('\0') {
        return Err(anyhow!("la ruta contiene bytes nulos: {}", path.display()));
    }

    Ok(())
}
