use anyhow::{anyhow, Context, Result};
use std::path::Path;
use std::process::Command;
use tracing::{error, info};

/// Safely compile a Typst file using the system's typst binary.
/// This function prevents command injection by using separate arguments
/// instead of string interpolation and validates the input paths.
pub fn compile_typst(input: &Path, output: &Path, ppi: u32, root: &Path) -> Result<()> {
    compile_typst_with(input, output, ppi, root, &[])
}

/// Como `compile_typst`, con carpetas de fuentes extra (`typst --font-path`).
pub fn compile_typst_with(
    input: &Path,
    output: &Path,
    ppi: u32,
    root: &Path,
    font_paths: &[std::path::PathBuf],
) -> Result<()> {
    // 1. Path validation to prevent traversal outside allowed directories
    validate_path(input)?;
    validate_path(output)?;

    // 2. PPI validation to prevent Resource Exhaustion (DoS)
    if !(72..=1200).contains(&ppi) {
        return Err(anyhow!(
            "PPI value must be between 72 and 1200, got {}. Prevents DoS.",
            ppi
        ));
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
        .context("Failed to execute typst binary. Is it installed and in your PATH?")?;

    if !output_result.status.success() {
        let stderr = String::from_utf8_lossy(&output_result.stderr);
        error!(
            "Typst compilation failed for {}:\n{}",
            input.display(),
            stderr
        );
        if stderr.contains("multiple images") {
            return Err(anyhow!(
                "la pieza ocupa más de una página (el texto se desbordó): baja el tamaño o los márgenes"
            ));
        }
        return Err(anyhow!("Typst compilation failed: {}", stderr.trim()));
    }

    Ok(())
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
                    "Path traversal attempt detected or suspicious path: {}",
                    path.display()
                ));
            }
        }
    }

    // Check for null bytes which can be used in some injection attacks
    if path.to_string_lossy().contains('\0') {
        return Err(anyhow!("Path contains null bytes: {}", path.display()));
    }

    Ok(())
}
