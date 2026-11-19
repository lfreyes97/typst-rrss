use std::path::PathBuf;
use std::sync::{Arc, OnceLock};

use typst_kit::fonts::FontStore;

/// Catálogo de fuentes compartido por todas las compilaciones.
///
/// De las del sistema solo se escanean los metadatos; cada una se lee del
/// disco la primera vez que un documento la usa.
pub struct Engine {
    font_dirs: Vec<PathBuf>,
    fonts: OnceLock<Arc<FontStore>>,
}

impl Engine {
    /// `font_dirs`: carpetas con fuentes propias (p. ej. `Fonts/`); ganan a
    /// las del sistema con el mismo nombre, como `--font-path` en `typst`.
    pub fn new(font_dirs: Vec<PathBuf>) -> Self {
        Self {
            font_dirs,
            fonts: OnceLock::new(),
        }
    }

    pub(crate) fn fonts(&self) -> Arc<FontStore> {
        self.fonts
            .get_or_init(|| {
                let mut store = FontStore::new();
                for dir in &self.font_dirs {
                    store.extend(typst_kit::fonts::scan(dir));
                }
                store.extend(typst_kit::fonts::system());
                // Las de serie (Libertinus, New Computer Modern…) al final,
                // como el CLI de Typst.
                store.extend(typst_kit::fonts::embedded());
                Arc::new(store)
            })
            .clone()
    }

    /// Familias disponibles, ordenadas y sin repetir.
    pub fn font_families(&self) -> Vec<String> {
        let mut v: Vec<String> = self
            .fonts()
            .book()
            .families()
            .map(|(name, _)| name.to_string())
            .collect();
        v.sort();
        v.dedup();
        v
    }
}
