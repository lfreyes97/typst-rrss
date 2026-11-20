//! Autocompletado con `typst-ide`.

use std::path::Path;

use typst_layout::PagedDocument;

use crate::{Engine, EngineWorld};

#[derive(Clone, Debug)]
pub struct Completion {
    pub label: String,
    /// Cuerpo con marcadores `${nombre}`; si falta, el propio `label`.
    pub apply: Option<String>,
    pub detail: Option<String>,
}

/// Completa en `cursor` (byte de `text`). Devuelve dónde empieza el tramo que
/// sustituye el elemento elegido y las opciones.
pub fn complete(engine: &Engine, root: &Path, filename: &str, text: &str, cursor: usize, explicit: bool) -> (usize, Vec<Completion>) {
    let mut world = EngineWorld::new(engine, root.to_path_buf(), filename, text.to_string());
    world.refresh_known_files();
    let source = world.main_source();
    let cursor = cursor.min(source.text().len());
    let (from, items) = typst_ide::autocomplete(&world, None::<&PagedDocument>, &source, cursor, explicit).unwrap_or((cursor, Vec::new()));
    let items = items
        .into_iter()
        .map(|c| Completion { label: c.label.to_string(), apply: c.apply.map(|a| a.to_string()), detail: c.detail.map(|d| d.to_string()) })
        .collect();
    (from, items)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn completa_funciones_y_variables() {
        let e = Engine::new(vec![]);
        let dir = std::env::temp_dir();
        let t = "#let numero = 3\n#ali";
        let (from, items) = complete(&e, &dir, "main.typ", t, t.len(), false);
        assert!(items.iter().any(|i| i.label == "align"));
        assert_eq!(&t[from..], "ali");
        let t = "#let numero = 3\n#num";
        assert!(complete(&e, &dir, "main.typ", t, t.len(), false).1.iter().any(|i| i.label == "numero"));
    }
}
