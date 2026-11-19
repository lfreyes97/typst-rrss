//! El contrato `instatypst-piece/v1`, CONGELADO.
//!
//! Congelar no es prohibir cambiar: es que cambiarlo sea una decisión y no un accidente.
//! Estas pruebas fallan si cambia la forma del contrato (un campo, un bloque, un valor
//! permitido, su serialización). Un cambio compatible se hace en una versión nueva
//! (`instatypst-piece/v2`) y se actualiza el golden a propósito:
//!
//!     UPDATE_GOLDEN=1 cargo test --test congelado
//!
//! Ver docs/contrato-v1.md.

use rrss_cli_rs::model::{self, Catalog, Syntax, SCHEMA};
use std::fs;
use std::path::PathBuf;

fn dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/contrato/congelado")
}

fn todo() -> model::Document {
    let texto = fs::read_to_string(dir().join("todo.toml")).unwrap();
    model::parse(&texto, Syntax::Toml).unwrap_or_else(|e| panic!("todo.toml no se lee: {e:#}"))
}

#[test]
fn el_identificador_del_contrato_es_v1() {
    assert_eq!(SCHEMA, "instatypst-piece/v1");
}

#[test]
fn el_documento_que_lo_usa_todo_es_valido() {
    let issues = model::validate(&todo());
    assert!(issues.is_empty(), "todo.toml dejó de ser válido: {issues:#?}");
}

/// La serialización es la forma del contrato: nombres de campos, enums, anidación.
#[test]
fn la_forma_serializada_no_cambia() {
    let json = model::write(&todo(), Syntax::Json).unwrap() + "\n";
    let golden = dir().join("piece-v1.golden.json");
    if std::env::var_os("UPDATE_GOLDEN").is_some() {
        fs::write(&golden, &json).unwrap();
        eprintln!("golden actualizado: {}", golden.display());
        return;
    }
    let esperado = fs::read_to_string(&golden).expect("falta piece-v1.golden.json (UPDATE_GOLDEN=1)");
    assert!(
        json == esperado,
        "la forma del contrato v1 cambió. Si es a propósito, es otra versión: \
         actualiza el golden con UPDATE_GOLDEN=1 y documéntalo en docs/contrato-v1.md"
    );
}

/// Compatibilidad hacia atrás: lo que v1 escribió hoy tiene que seguir leyéndose.
#[test]
fn el_golden_se_sigue_leyendo_y_da_lo_mismo() {
    let golden = fs::read_to_string(dir().join("piece-v1.golden.json")).unwrap();
    let de_json = model::parse(&golden, Syntax::Json).expect("el golden de v1 ya no se lee");
    assert_eq!(de_json, todo(), "leer el golden da otro documento que el TOML original");
}

#[test]
fn el_documento_da_lo_mismo_en_las_tres_sintaxis() {
    let doc = todo();
    for s in [Syntax::Toml, Syntax::Json, Syntax::Yaml] {
        let escrito = model::write(&doc, s).unwrap();
        assert_eq!(doc, model::parse(&escrito, s).unwrap(), "difiere vía {s:?}");
    }
}

/// Los bloques de contenido y su parentesco son parte del contrato (los estilos se
/// escriben contra ellos). Añadir un estilo no los toca; añadir un bloque sí.
#[test]
fn los_bloques_y_su_parentesco_estan_congelados() {
    let cat = Catalog::embedded();
    let mut bloques: Vec<&str> = cat.blocks.iter().map(String::as_str).collect();
    bloques.sort();
    assert_eq!(
        bloques,
        [
            "attribution", "body", "conclusion", "contrast", "date", "epigraph", "footer", "label",
            "list", "number", "quote", "signature", "subtitle", "title", "verse"
        ],
        "el catálogo de bloques de v1 cambió"
    );
    let parentesco: Vec<(&str, &str)> = cat
        .block_parents
        .iter()
        .map(|(a, b)| (a.as_str(), b.as_str()))
        .collect();
    assert_eq!(
        parentesco,
        [("epigraph", "quote"), ("quote", "body"), ("verse", "quote")],
        "el parentesco entre bloques de v1 cambió"
    );
}

/// Los códigos de error son parte del contrato (los consumidores los usan para señalar el
/// campo). Los que emite el código y los que documenta `docs/contrato-v1.md` son los mismos.
#[test]
fn los_codigos_de_error_del_documento_son_los_del_codigo() {
    use std::collections::BTreeSet;
    // un código es la cadena que sigue a la ruta en `issue(ruta, "código", …)` / `self.error(pos, "código", …)`
    let mut del_codigo: BTreeSet<String> = BTreeSet::new();
    for f in ["src/model.rs", "src/markup.rs"] {
        let s = fs::read_to_string(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(f)).unwrap();
        let mut resto = s.as_str();
        while let Some(i) = resto.find("\"") {
            resto = &resto[i + 1..];
            let Some(j) = resto.find('"') else { break };
            let lit = &resto[..j];
            resto = &resto[j + 1..];
            // los códigos son minúsculas con guion bajo, y van tras `issue(`/`error(` en algún punto previo
            let es_codigo = lit.len() > 3
                && lit.chars().all(|c| c.is_ascii_lowercase() || c == '_')
                && (lit.contains('_') || lit == "parse");
            if es_codigo {
                del_codigo.insert(lit.to_string());
            }
        }
    }
    let doc = fs::read_to_string(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../docs/contrato-v1.md")).unwrap();
    let tabla = &doc[doc.find("**Códigos de error**").unwrap()..doc.find("## Estilo propio").unwrap()];
    let del_doc: BTreeSet<String> = tabla
        .split('`')
        .skip(1)
        .step_by(2)
        .filter(|s| !s.is_empty() && s.chars().all(|c| c.is_ascii_lowercase() || c == '_'))
        .map(String::from)
        .collect();
    assert_eq!(del_doc.len(), 40, "el documento declara 40 códigos");
    // todo código documentado existe en el código (lo contrario no se exige: el código tiene otras cadenas)
    let huerfanos: Vec<_> = del_doc.difference(&del_codigo).collect();
    assert!(huerfanos.is_empty(), "documentados pero no emitidos: {huerfanos:?}");
}

/// Y al revés, en lo que se puede comprobar: todo código que esperan los fixtures inválidos
/// está en el documento. (Un código nuevo sin fixture ni documentar no lo detecta esta prueba.)
#[test]
fn los_codigos_que_esperan_los_fixtures_estan_documentados() {
    let base = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let doc = fs::read_to_string(base.join("../docs/contrato-v1.md")).unwrap();
    let mut sin_documentar = Vec::new();
    for e in fs::read_dir(base.join("tests/fixtures/contrato/invalid")).unwrap().flatten() {
        let t = fs::read_to_string(e.path()).unwrap();
        let codigo = t.lines().next().and_then(|l| l.strip_prefix("# expect: ")).unwrap().trim().to_string();
        if !doc.contains(&format!("`{codigo}`")) {
            sin_documentar.push((e.file_name(), codigo));
        }
    }
    assert!(sin_documentar.is_empty(), "códigos esperados por fixtures y no documentados: {sin_documentar:?}");
}
