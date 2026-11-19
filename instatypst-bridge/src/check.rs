//! `check`: validar un documento sin compilar nada, con una salida que otro programa pueda leer.
//!
//! Es la interfaz de motor del CLI: una GUI, un editor de texto o el CI preguntan «¿está bien
//! este documento?» y reciben todas las incidencias —no solo la primera—, cada una con la ruta
//! del campo y la pieza a la que pertenece. No toca el contrato congelado: es la superficie del
//! CLI, con su propia versión (`instatypst-check/v1`).
//!
//! Pasa por las mismas etapas que `build`, sin lanzar `typst`:
//!   1. lectura y validación estructural y semántica del contrato (`model`);
//!   2. el puente a los presets (`pieza`): lo que un estilo aún no sabe construir;
//!   3. el render a Typst de cada pieza, de donde salen los avisos de contraste.
//!
//! Códigos: los 40 del contrato (docs/contrato-v1.md), más `read` (no se pudo leer el archivo),
//! `config` (un posts.toml de siempre no es válido), `build` (el puente o el render fallan) y
//! `contrast_low` (aviso).

use serde::Serialize;
use std::collections::HashSet;
use std::fs;
use std::path::Path;

use crate::config::Config;
use crate::model::{self, Catalog, Syntax};
use crate::{pieza, render};

pub const SCHEMA: &str = "instatypst-check/v1";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Severity {
    Error,
    Warning,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Finding {
    pub severity: Severity,
    pub code: String,
    /// El campo, p. ej. `post[2].content[1]`; vacío si es del documento entero.
    pub path: String,
    /// La pieza a la que pertenece, si se sabe.
    pub piece: Option<String>,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Report {
    pub schema: &'static str,
    pub file: String,
    /// `"pieces"` (un documento de piezas) o `"posts"` (un posts.toml de siempre).
    pub kind: &'static str,
    /// Sin errores (los avisos no cuentan).
    pub ok: bool,
    /// Cuántas piezas declara el documento.
    pub pieces: usize,
    pub findings: Vec<Finding>,
}

impl Report {
    fn new(file: &Path) -> Self {
        Self {
            schema: SCHEMA,
            file: file.display().to_string(),
            kind: "pieces",
            ok: true,
            pieces: 0,
            findings: Vec::new(),
        }
    }

    fn error(&mut self, code: &str, path: &str, piece: Option<String>, message: String) {
        self.findings.push(Finding {
            severity: Severity::Error,
            code: code.into(),
            path: path.into(),
            piece,
            message,
        });
    }

    fn warning(&mut self, code: &str, piece: &str, message: String) {
        self.findings.push(Finding {
            severity: Severity::Warning,
            code: code.into(),
            path: String::new(),
            piece: Some(piece.into()),
            message,
        });
    }

    fn cerrar(mut self) -> Self {
        self.ok = !self.findings.iter().any(|f| f.severity == Severity::Error);
        self
    }

    pub fn errors(&self) -> usize {
        self.findings
            .iter()
            .filter(|f| f.severity == Severity::Error)
            .count()
    }

    pub fn warnings(&self) -> usize {
        self.findings
            .iter()
            .filter(|f| f.severity == Severity::Warning)
            .count()
    }

    pub fn to_json(&self) -> String {
        serde_json::to_string_pretty(self).expect("el informe se serializa") + "\n"
    }

    /// Para una persona: una línea por incidencia y un resumen.
    pub fn to_text(&self) -> String {
        let mut s = String::new();
        for f in &self.findings {
            let marca = if f.severity == Severity::Error {
                "✗"
            } else {
                "⚠"
            };
            let donde = match (&f.piece, f.path.is_empty()) {
                (Some(p), true) => format!("pieza {p:?}"),
                (Some(p), false) => format!("{} · pieza {p:?}", f.path),
                (None, true) => "documento".to_string(),
                (None, false) => f.path.clone(),
            };
            s.push_str(&format!("{marca} {donde} [{}]: {}\n", f.code, f.message));
        }
        let resumen = if self.ok {
            format!("✓ {}: {} piezas, sin errores", self.file, self.pieces)
        } else {
            format!("✗ {}: {} errores", self.file, self.errors())
        };
        let avisos = match self.warnings() {
            0 => String::new(),
            n => format!(", {n} avisos"),
        };
        s.push_str(&format!("{resumen}{avisos}\n"));
        s
    }
}

/// «post[3].content[0]» → 3
fn indice_de(path: &str) -> Option<usize> {
    let resto = path.strip_prefix("post[")?;
    resto[..resto.find(']')?].parse().ok()
}

fn clasificar_aviso(a: &str) -> &'static str {
    if a.starts_with("contraste bajo") {
        "contrast_low"
    } else {
        "warning"
    }
}

/// Valida un archivo. Nunca falla: un error de lectura es una incidencia más.
pub fn check_file(path: &Path) -> Report {
    let mut r = Report::new(path);
    let texto = match fs::read_to_string(path) {
        Ok(t) => t,
        Err(e) => {
            r.error(
                "read",
                "",
                None,
                format!("no se pudo leer {}: {e}", path.display()),
            );
            return r.cerrar();
        }
    };
    let sintaxis = path
        .to_str()
        .and_then(Syntax::from_extension)
        .unwrap_or(Syntax::Toml);

    let cfg = if pieza::es_de_piezas(&texto, sintaxis) {
        match check_piezas(&texto, sintaxis, &mut r) {
            Some(c) => c,
            None => return r.cerrar(),
        }
    } else {
        r.kind = "posts";
        match Config::load(path) {
            Ok(c) => c,
            Err(e) => {
                r.error("config", "", None, format!("{e:#}"));
                return r.cerrar();
            }
        }
    };
    if r.kind == "posts" {
        r.pieces = cfg.posts.len();
    }
    render_todas(&cfg, &mut r);
    r.cerrar()
}

/// Las etapas del contrato y del puente para un documento de piezas. Devuelve el `Config` con las
/// piezas que se pudieron construir, o `None` si el documento ni siquiera es legible.
fn check_piezas(texto: &str, sintaxis: Syntax, r: &mut Report) -> Option<Config> {
    let doc = match model::parse(texto, sintaxis) {
        Ok(d) => d,
        Err(e) => {
            r.error("parse", "", None, format!("{e:#}"));
            return None;
        }
    };
    check_doc(&doc, r)
}

/// El informe de un documento que ya está en memoria (la GUI), sin tocar el disco: las mismas
/// etapas que `check_file` — contrato, firmas y puente, con los avisos de contraste del render.
/// `nombre` es lo que aparece como `file` en el informe.
pub fn check_document(doc: &model::Document, nombre: &str) -> Report {
    let mut r = Report::new(Path::new(nombre));
    if let Some(cfg) = check_doc(doc, &mut r) {
        render_todas(&cfg, &mut r);
    }
    r.cerrar()
}

fn check_doc(doc: &model::Document, r: &mut Report) -> Option<Config> {
    r.pieces = doc.posts.len();
    let nombre_de = |path: &str| {
        indice_de(path)
            .and_then(|i| doc.posts.get(i))
            .and_then(|p| p.name.clone())
    };

    let cat = Catalog::embedded();
    let mut malas: HashSet<usize> = HashSet::new();
    let mut del_documento = false;
    for i in model::validate(doc)
        .into_iter()
        .chain(model::validate_semantic(doc, &cat))
    {
        match indice_de(&i.path) {
            Some(n) => {
                malas.insert(n);
            }
            None => del_documento = true,
        }
        r.error(i.code, &i.path, nombre_de(&i.path), i.message);
    }
    // El puente da por buena una pieza validada. Un error del documento entero (schema, palettes,
    // defaults…) puede afectar a todas, así que ahí se detiene; uno de una pieza no esconde las otras.
    if del_documento {
        return None;
    }

    let (cfg, fallos) = pieza::a_config_omitiendo(doc, &cat, &malas);
    for f in fallos {
        r.error(
            "build",
            &format!("post[{}]", f.indice),
            Some(f.nombre),
            format!("{:#}", f.error),
        );
    }
    Some(cfg)
}

/// Lo que haría `build` antes de lanzar `typst`: resolver cada pieza y renderizarla a Typst.
/// De ahí salen los errores del render y los avisos de contraste.
fn render_todas(cfg: &Config, r: &mut Report) {
    for raw in &cfg.posts {
        let post = match cfg.resolve(raw) {
            Ok(p) => p,
            Err(e) => {
                r.error("build", "", Some(raw.name.clone()), format!("{e:#}"));
                continue;
            }
        };
        match render::render(&post, cfg) {
            Ok(rendered) => {
                for a in rendered.avisos {
                    r.warning(clasificar_aviso(&a), &post.name, a);
                }
            }
            Err(e) => r.error("build", "", Some(post.name.clone()), format!("{e:#}")),
        }
    }
}

#[cfg(test)]
mod tests_documento {
    use super::*;

    /// Desde memoria da el mismo informe que desde el archivo, con los avisos de contraste incluidos.
    #[test]
    fn check_document_da_el_mismo_informe_que_check_file() {
        let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../instatypst-cli/tests/fixtures/contrato");
        let mut vistos = 0;
        for sub in ["valid", "invalid"] {
            for e in std::fs::read_dir(dir.join(sub)).unwrap().flatten() {
                let ruta = e.path();
                if ruta.extension().is_none_or(|x| x != "toml") {
                    continue;
                }
                let texto = std::fs::read_to_string(&ruta).unwrap();
                let Ok(doc) = model::parse(&texto, Syntax::Toml) else { continue };
                if !pieza::es_de_piezas(&texto, Syntax::Toml) {
                    continue;
                }
                let a = check_file(&ruta);
                let b = check_document(&doc, &ruta.display().to_string());
                assert_eq!(a.to_json(), b.to_json(), "{}", ruta.display());
                vistos += 1;
            }
        }
        assert!(vistos >= 10, "solo {vistos} fixtures comparadas");
    }
}
