//! Typst embebido para la GUI (y, más adelante, para el CLI).
//!
//! Sin interfaz gráfica ni traducciones: devuelve datos (`PagedDocument`,
//! pixmaps, diagnósticos, mapa posición ↔ fuente). La versión de Typst va
//! fijada con la de `typst-syntax` de `instatypst-piece`.
//!
//! Parte del motor de Grammata (mismo autor), sin lo que dependía de egui.

mod diagnostics;
mod export;
mod fonts;
mod geometry;
mod sync;
mod world;

pub use diagnostics::{Diagnostic, Severity, count};
pub use export::{ExportFormat, export_pages, parse_pages, pixmap_to_rgba, render_page};
pub use fonts::Engine;
pub use geometry::{page_boxes, BoxInfo, BoxKind};
pub use sync::{SyncEntry, line_for_pos, page_for_line};
pub use typst_layout::PagedDocument;
pub use world::EngineWorld;

use std::path::Path;
use std::time::Instant;

/// Resultado de compilar un documento.
pub struct Compiled {
    /// `None` si hubo errores.
    pub doc: Option<PagedDocument>,
    /// Errores y avisos, ya con línea y columna.
    pub diags: Vec<Diagnostic>,
    pub ms: u128,
    /// Texto colocado con su página y posición (base de «clic → código»).
    pub sync: Vec<SyncEntry>,
}

impl Engine {
    /// Compila `text` como documento principal (`filename`, relativo a `root`).
    pub fn compile(&self, text: &str, root: &Path, filename: &str) -> Compiled {
        let t0 = Instant::now();
        let world = EngineWorld::new(self, root.to_path_buf(), filename, text.to_string());
        let warned = typst::compile::<PagedDocument>(&world);
        let mut diags = diagnostics::map(&world, world.main_id(), &warned.warnings);
        let (doc, sync) = match warned.output {
            Ok(doc) => {
                let sync = sync::extract(&world, &doc);
                (Some(doc), sync)
            }
            Err(errors) => {
                diags.extend(diagnostics::map(&world, world.main_id(), &errors));
                (None, Vec::new())
            }
        };
        // Como `typst watch`: sin esto la caché crece con cada compilación.
        comemo::evict(10);
        Compiled {
            doc,
            diags,
            ms: t0.elapsed().as_millis(),
            sync,
        }
    }
}
