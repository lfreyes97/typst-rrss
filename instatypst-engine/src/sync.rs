//! Mapa posición ↔ fuente: de un clic en la página a la línea del código y al revés.

use std::collections::HashMap;

use typst::layout::{Frame, FrameItem, Point, Transform};
use typst::syntax::{FileId, LinkedNode, Span};
use typst::text::TextItem;
use typst_layout::PagedDocument;

use crate::EngineWorld;

/// Un tramo de texto colocado: una línea fuente en un renglón de una página.
/// De aquí salen el scroll de la preview (forward) y el salto al código
/// desde un clic (inverso).
#[derive(Clone, Debug)]
pub struct SyncEntry {
    /// `None` = documento actual; `Some` = incluido (relativo a la raíz).
    pub file: Option<String>,
    pub line: usize,
    pub col: usize,
    pub page: usize,
    /// Inicio del tramo en puntos dentro de la página (coords de frame).
    pub x: f32,
    /// Línea base del renglón, en puntos.
    pub y: f32,
}

/// Cada tramo de texto de cada página con su línea fuente, recorriendo los
/// frames (con sus transformaciones) como hace `typst-ide`. Los tramos
/// contiguos de una misma línea fuente en un mismo renglón se funden.
pub(crate) fn extract(world: &EngineWorld, doc: &PagedDocument) -> Vec<SyncEntry> {
    let mut out: Vec<SyncEntry> = Vec::new();
    // Span → offset del fichero principal en una pasada: `World::range` busca
    // desde la raíz en cada llamada (~30 µs; miles de spans por documento).
    let mut offsets = HashMap::new();
    let main = world.main_id();
    index_spans(LinkedNode::new(world.main.root()), &mut offsets);
    // El mismo nodo de texto produce muchos tramos (y cabeceras repetidas).
    let mut located: HashMap<Span, Option<(Option<String>, usize, usize)>> = HashMap::new();
    for (page, p) in doc.pages().iter().enumerate() {
        walk_text(&p.frame, Transform::identity(), &mut |pos, text| {
            let Some(span) = text
                .glyphs
                .iter()
                .map(|g| g.span.0)
                .find(|s| !s.is_detached())
            else {
                return;
            };
            let loc = located
                .entry(span)
                .or_insert_with(|| match offsets.get(&span) {
                    Some(&b) => {
                        let (l, c) = world.main.lines().byte_to_line_column(b)?;
                        Some((None, l + 1, c + 1))
                    }
                    None => {
                        let (file, line, col) = locate_span(world, main, span);
                        Some((file, line?, col?))
                    }
                });
            let Some((file, line, col)) = loc.clone() else {
                return;
            };
            let (x, y) = (pos.x.to_pt() as f32, pos.y.to_pt() as f32);
            if let Some(last) = out.last_mut()
                && last.page == page
                && last.line == line
                && last.file == file
                && (last.y - y).abs() < 0.5
            {
                last.x = last.x.min(x);
                return;
            }
            out.push(SyncEntry {
                file,
                line,
                col,
                page,
                x,
                y,
            });
        });
    }
    out
}

fn index_spans(node: LinkedNode, out: &mut HashMap<Span, usize>) {
    out.insert(node.span(), node.offset());
    for child in node.children() {
        index_spans(child, out);
    }
}

/// Visita los `TextItem` del frame con su posición en coords de página.
fn walk_text<F: FnMut(Point, &TextItem)>(frame: &Frame, ts: Transform, f: &mut F) {
    for (pos, item) in frame.items() {
        match item {
            FrameItem::Group(g) => {
                let ts = ts
                    .pre_concat(Transform::translate(pos.x, pos.y))
                    .pre_concat(g.transform);
                walk_text(&g.frame, ts, f);
            }
            FrameItem::Text(t) => f(pos.transform(ts), t),
            _ => {}
        }
    }
}

/// Línea → (página, y): la primera aparición de esa línea; si no produce
/// texto (código, línea vacía), la de la línea anterior más cercana que sí;
/// si no hay, la siguiente. No supone orden: las cabeceras repetidas de cada
/// página apuntan a líneas del principio.
pub fn page_for_line(sync: &[SyncEntry], file: Option<&str>, line: usize) -> Option<(usize, f32)> {
    let mut before: Option<&SyncEntry> = None;
    let mut after: Option<&SyncEntry> = None;
    for e in sync.iter().filter(|e| e.file.as_deref() == file) {
        if e.line == line {
            return Some((e.page, e.y));
        }
        if e.line < line && before.is_none_or(|b| e.line > b.line) {
            before = Some(e);
        }
        if e.line > line && after.is_none_or(|b| e.line < b.line) {
            after = Some(e);
        }
    }
    before.or(after).map(|e| (e.page, e.y))
}

/// (Página, x, y) → tramo bajo el punto: el renglón cuya línea base es la
/// primera por debajo de `y` (o el último de la página), y en él el tramo
/// que empieza más a la derecha sin pasar de `x`. Sin texto en la página,
/// el último tramo de las anteriores.
pub fn line_for_pos(sync: &[SyncEntry], page: usize, x: f32, y: f32) -> Option<&SyncEntry> {
    let on_page = || sync.iter().filter(move |e| e.page == page);
    // Tolerancia: un clic sobre los descendentes queda bajo la línea base.
    let row_y = on_page()
        .map(|e| e.y)
        .filter(|&ey| ey >= y - 3.0)
        .min_by(f32::total_cmp)
        .or_else(|| on_page().map(|e| e.y).max_by(f32::total_cmp));
    let Some(row_y) = row_y else {
        return sync.iter().rfind(|e| e.page < page);
    };
    let row = || on_page().filter(move |e| (e.y - row_y).abs() < 1.0);
    row()
        .filter(|e| e.x <= x + 1.0)
        .max_by(|a, b| a.x.total_cmp(&b.x))
        .or_else(|| row().min_by(|a, b| a.x.total_cmp(&b.x)))
}
fn locate_span(
    world: &EngineWorld,
    main: FileId,
    span: Span,
) -> (Option<String>, Option<usize>, Option<usize>) {
    crate::diagnostics::locate_span(world, main, span)
}

