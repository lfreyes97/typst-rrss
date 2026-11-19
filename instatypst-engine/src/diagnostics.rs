//! Diagnósticos del compilador embebido.
//!
//! `map` convierte `SourceDiagnostic` (con spans) a nuestro `Diagnostic`:
//! fichero relativo a la raíz (el main como `None`) y línea/columna 1-based.

use crate::EngineWorld;
use typst::World as _;
use typst::WorldExt as _;
use typst::diag::{Severity as TSeverity, SourceDiagnostic};
use typst::syntax::{FileId, Span, VirtualRoot};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Severity {
    Error,
    Warning,
}

#[derive(Clone, Debug)]
pub struct Diagnostic {
    pub severity: Severity,
    /// `None` = el propio documento; `Some` = otro archivo
    /// (relativo a la raíz de compilación).
    pub file: Option<String>,
    pub line: Option<usize>,
    pub col: Option<usize>,
    pub message: String,
    pub hints: Vec<String>,
}

impl Diagnostic {
    pub fn location(&self) -> String {
        let file = self.file.as_deref().unwrap_or("");
        match (self.line, self.col) {
            (Some(l), Some(c)) if file.is_empty() => format!("{l}:{c}"),
            (Some(l), Some(c)) => format!("{file}:{l}:{c}"),
            _ => file.to_string(),
        }
    }
}

pub fn count(diags: &[Diagnostic]) -> (usize, usize) {
    let e = diags
        .iter()
        .filter(|d| d.severity == Severity::Error)
        .count();
    (e, diags.len() - e)
}

pub(crate) fn map(world: &EngineWorld, main: FileId, src: &[SourceDiagnostic]) -> Vec<Diagnostic> {
    src.iter()
        .map(|d| {
            let severity = match d.severity {
                TSeverity::Error => Severity::Error,
                TSeverity::Warning => Severity::Warning,
            };
            let (file, line, col) = match d.span.id() {
                Some(id) => {
                    let rel = display_path(main, id);
                    let lc = world
                        .range(d.span)
                        .and_then(|r| world.source(id).ok().map(|s| (s, r.start)))
                        .and_then(|(s, b)| {
                            let b = b.min(s.text().len());
                            s.lines().byte_to_line_column(b)
                        })
                        .map(|(l, c)| (l + 1, c + 1));
                    (rel, lc.map(|p| p.0), lc.map(|p| p.1))
                }
                None => (None, None, None),
            };
            Diagnostic {
                severity,
                file,
                line,
                col,
                message: d.message.to_string(),
                hints: d.hints.iter().map(|h| h.v.to_string()).collect(),
            }
        })
        .collect()
}

/// Ubica un `Span` del compilador: (fichero, línea, columna 1-based).
/// Comparte lógica con `map` (que parte de `DiagSpan`).
pub(crate) fn locate_span(
    world: &EngineWorld,
    main: FileId,
    span: Span,
) -> (Option<String>, Option<usize>, Option<usize>) {
    let Some(id) = span.id() else {
        return (None, None, None);
    };
    let rel = display_path(main, id);
    let lc = world
        .range(span)
        .and_then(|r| world.source(id).ok().map(|s| (s, r.start)))
        .and_then(|(s, b)| {
            let b = b.min(s.text().len());
            s.lines().byte_to_line_column(b)
        })
        .map(|(l, c)| (l + 1, c + 1));
    (rel, lc.map(|p| p.0), lc.map(|p| p.1))
}
fn display_path(main: FileId, id: FileId) -> Option<String> {
    if id == main {
        return None;
    }
    let rel = id.vpath().get_without_slash().to_string();
    match id.root() {
        VirtualRoot::Project => Some(rel),
        VirtualRoot::Package(spec) => Some(format!("{spec}:{rel}")),
    }
}
