//! Renderizar páginas y exportar a PDF, PNG o SVG.

use std::path::{Path, PathBuf};

use typst::utils::Scalar;
use typst_layout::PagedDocument;

/// Formato de salida de `export_pages`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ExportFormat {
    Pdf,
    /// PDF/A-2b.
    PdfA,
    Png,
    Svg,
}

impl ExportFormat {
    pub fn extension(self) -> &'static str {
        match self {
            Self::Pdf | Self::PdfA => "pdf",
            Self::Png => "png",
            Self::Svg => "svg",
        }
    }
}

/// Rasteriza una página a `pixel_per_pt` px/pt (1.0 = 72 ppp).
pub fn render_page(doc: &PagedDocument, page: usize, pixel_per_pt: f32) -> Option<tiny_skia::Pixmap> {
    let p = doc.pages().get(page)?;
    Some(typst_render::render(
        p,
        &typst_render::RenderOptions {
            pixel_per_pt: Scalar::new(pixel_per_pt.clamp(0.05, 16.0) as f64),
            render_bleed: false,
        },
    ))
}

/// Pixmap premultiplicado de tiny-skia → RGBA8 sobre fondo blanco, con
/// `(ancho, alto)`.
pub fn pixmap_to_rgba(pix: &tiny_skia::Pixmap) -> (usize, usize, Vec<u8>) {
    let mut out = Vec::with_capacity(pix.data().len());
    for px in pix.data().chunks_exact(4) {
        // Ya premultiplicado: sobre blanco es `c + 255·(1 − α)`, sin dividir
        // por α (eso lo despremultiplicaba y aclaraba los bordes).
        let white = 255 - px[3];
        out.extend_from_slice(&[
            px[0].saturating_add(white),
            px[1].saturating_add(white),
            px[2].saturating_add(white),
            255,
        ]);
    }
    (pix.width() as usize, pix.height() as usize, out)
}

/// Escribe el PDF entero, o PNG/SVG por página expandiendo `{p}` en
/// `dest_template`. `pages`: 1-based; vacío = todas (ver `parse_pages`).
pub fn export_pages(
    doc: &PagedDocument,
    dest_template: &Path,
    fmt: ExportFormat,
    ppi: Option<u32>,
    pages: &[usize],
) -> Result<(), String> {
    if doc.pages().is_empty() {
        return Err("el documento no tiene páginas".to_string());
    }
    if let Some(&p) = pages.iter().find(|&&p| p > doc.pages().len()) {
        return Err(format!("la página {} no existe: el documento tiene {}", p, doc.pages().len()));
    }
    if matches!(fmt, ExportFormat::Pdf | ExportFormat::PdfA) {
        let mut opts = typst_pdf::PdfOptions::default();
        if !pages.is_empty() {
            // typst no admite un rango con el PDF etiquetado (accesibilidad).
            opts.tagged = false;
            opts.page_ranges = Some(typst::layout::PageRanges::new(
                pages
                    .iter()
                    .filter_map(|&p| std::num::NonZeroUsize::new(p))
                    .map(|n| Some(n)..=Some(n))
                    .collect(),
            ));
        }
        if fmt == ExportFormat::PdfA {
            opts.standards = typst_pdf::PdfStandards::new(&[typst_pdf::PdfStandard::A_2b])
                .map_err(|e| e.message().to_string())?;
        }
        let bytes = typst_pdf::pdf(doc, &opts).map_err(|errors| {
            errors
                .first()
                .map(|e| e.message.to_string())
                .unwrap_or_else(|| "falló la exportación a PDF".to_string())
        })?;
        return std::fs::write(dest_template, bytes)
            .map_err(|e| format!("no se pudo escribir {}: {}", dest_template.display(), e));
    }
    // Los archivos conservan el número de página original (`doc-5.png`).
    let multi = if pages.is_empty() { doc.pages().len() } else { pages.len() } > 1;
    for (i, page) in doc.pages().iter().enumerate() {
        if !pages.is_empty() && !pages.contains(&(i + 1)) {
            continue;
        }
        let bytes: Vec<u8> = match fmt {
            ExportFormat::Svg => typst_svg::svg(page, &typst_svg::SvgOptions::default()).into_bytes(),
            _ => {
                let ppi = ppi.unwrap_or(144) as f64;
                let pix = typst_render::render(
                    page,
                    &typst_render::RenderOptions {
                        pixel_per_pt: Scalar::new(ppi / 72.0),
                        ..Default::default()
                    },
                );
                pix.encode_png().map_err(|e| format!("no se pudo codificar PNG: {}", e))?
            }
        };
        let dest = expand_dest(dest_template, fmt.extension(), multi, i + 1);
        if let Some(dir) = dest.parent() {
            std::fs::create_dir_all(dir).map_err(|e| format!("no se pudo crear {dir:?}: {e}"))?;
        }
        std::fs::write(&dest, bytes)
            .map_err(|e| format!("no se pudo escribir {}: {}", dest.display(), e))?;
    }
    Ok(())
}

/// Páginas de una especificación como `1-3, 5, 8-` (1-based; `-4` = hasta la 4,
/// `8-` = desde la 8). Devuelve las páginas ordenadas y sin repetir; vacía
/// (o en blanco) significa todas. Con `total` > 0 se comprueba el límite.
pub fn parse_pages(spec: &str, total: usize) -> Result<Vec<usize>, String> {
    let spec = spec.trim();
    if spec.is_empty() {
        return Ok(Vec::new());
    }
    let num = |t: &str| -> Result<usize, String> {
        match t.trim().parse::<usize>() {
            Ok(0) => Err("las páginas empiezan en 1".to_string()),
            Ok(n) => Ok(n),
            Err(_) => Err(format!("«{}» no es un número de página", t.trim())),
        }
    };
    let mut out = Vec::new();
    for part in spec
        .split([',', ';'])
        .map(str::trim)
        .filter(|p| !p.is_empty())
    {
        let (a, b) = match part.split_once('-') {
            None => {
                let n = num(part)?;
                (n, n)
            }
            Some((a, b)) => {
                let a = if a.trim().is_empty() { 1 } else { num(a)? };
                let b = if b.trim().is_empty() {
                    if total == 0 {
                        return Err("«8-» necesita saber cuántas páginas hay".to_string());
                    }
                    total
                } else {
                    num(b)?
                };
                (a, b)
            }
        };
        if a > b {
            return Err(format!("el rango {}-{} está al revés", a, b));
        }
        if total > 0 && b > total {
            return Err(format!("la página {} no existe: el documento tiene {}", b, total));
        }
        out.extend(a..=b);
    }
    out.sort_unstable();
    out.dedup();
    Ok(out)
}

fn expand_dest(template: &Path, fmt: &str, multi: bool, page: usize) -> PathBuf {
    let s = template.to_string_lossy();
    if s.contains("{p}") {
        return PathBuf::from(s.replace("{p}", &page.to_string()));
    }
    if multi {
        let stem = template
            .file_stem()
            .map(|s| s.to_string_lossy().into_owned())
            .unwrap_or_default();
        return template.with_file_name(format!("{stem}-{page}.{fmt}"));
    }
    template.to_path_buf()
}


#[cfg(test)]
mod tests {
    use super::*;
    use crate::Engine;

    const DOC: &str = "#set page(width: 100pt, height: 60pt, margin: 4pt, fill: rgb(\"#102030\"))\n#set text(fill: white)\nHola\n#pagebreak()\nDos\n";

    fn compile() -> crate::Compiled {
        Engine::new(vec![]).compile(DOC, &std::env::temp_dir(), "t.typ")
    }

    #[test]
    fn rangos_de_paginas() {
        assert_eq!(parse_pages("", 0).unwrap(), Vec::<usize>::new());
        assert_eq!(parse_pages("1-3, 5, 8-", 9).unwrap(), vec![1, 2, 3, 5, 8, 9]);
        assert!(parse_pages("0", 0).is_err());
        assert!(parse_pages("3-1", 0).is_err());
        assert!(parse_pages("4", 3).is_err());
    }

    #[test]
    fn compila_y_renderiza_con_el_color_de_fondo() {
        let c = compile();
        let doc = c.doc.expect("debía compilar");
        assert_eq!(doc.pages().len(), 2);
        let pix = render_page(&doc, 0, 2.0).unwrap();
        let (w, h, rgba) = pixmap_to_rgba(&pix);
        assert_eq!((w, h), (200, 120));
        // esquina: fondo #102030, opaco
        assert_eq!(&rgba[0..4], &[0x10, 0x20, 0x30, 255]);
        assert!(render_page(&doc, 2, 1.0).is_none());
    }

    #[test]
    fn un_error_trae_linea_y_columna() {
        let c = Engine::new(vec![]).compile("a\n#let x = (\n", &std::env::temp_dir(), "t.typ");
        assert!(c.doc.is_none());
        let d = c.diags.iter().find(|d| d.severity == crate::Severity::Error).unwrap();
        assert!(d.line.is_some() && d.col.is_some(), "{d:?}");
    }

    #[test]
    fn el_clic_en_la_pagina_lleva_a_su_linea() {
        let c = compile();
        assert!(!c.sync.is_empty());
        // «Dos» está en la línea 5 y en la página 2 (índice 1)
        let (page, y) = crate::page_for_line(&c.sync, None, 5).unwrap();
        assert_eq!(page, 1);
        let e = crate::line_for_pos(&c.sync, page, 5.0, y).unwrap();
        assert_eq!(e.line, 5);
    }

    #[test]
    fn exporta_png_por_pagina_y_pdf() {
        let c = compile();
        let doc = c.doc.unwrap();
        let dir = std::env::temp_dir().join(format!("insta-engine-{}", std::process::id()));
        export_pages(&doc, &dir.join("l.png"), ExportFormat::Png, Some(72), &[]).unwrap();
        assert!(dir.join("l-1.png").exists() && dir.join("l-2.png").exists());
        export_pages(&doc, &dir.join("l.pdf"), ExportFormat::Pdf, None, &[2]).unwrap();
        assert!(std::fs::read(dir.join("l.pdf")).unwrap().starts_with(b"%PDF"));
        let _ = std::fs::remove_dir_all(dir);
    }
}
