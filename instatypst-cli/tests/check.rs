//! `instatypst check`: validar sin compilar, con una salida que otro programa lea.

use instatypst_cli::check::{check_file, Severity, SCHEMA};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

fn fixture(rel: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/contrato").join(rel)
}

/// Un documento escrito a un archivo temporal, para pasárselo a `check`.
fn escribir(nombre: &str, ext: &str, texto: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("instatypst-check-{}", std::process::id()));
    fs::create_dir_all(&dir).unwrap();
    let p = dir.join(format!("{nombre}.{ext}"));
    fs::write(&p, texto).unwrap();
    p
}

fn codigos(path: &Path) -> Vec<String> {
    check_file(path).findings.iter().map(|f| f.code.clone()).collect()
}

const VARIOS: &str = r##"schema = "instatypst-piece/v1"
[[post]]
name = "uno"
style = "suizo"
[[post.content]]
type = "title"
text = "Sin conclusión"
[[post]]
name = "dos"
style = "liso"
palette = { background = "#12345", text = "#ffffff" }
content = "x"
[[post]]
name = "tres"
style = "capitulo-orlado"
[[post.content]]
type = "number"
value = 1
[[post.content]]
type = "title"
text = "x"
"##;

#[test]
fn un_documento_correcto_no_tiene_incidencias() {
    let r = check_file(&fixture("build/nuevos.toml"));
    assert!(r.ok && r.findings.is_empty(), "{:#?}", r.findings);
    assert_eq!((r.kind, r.pieces), ("pieces", 4));
}

#[test]
fn junta_los_errores_de_varias_piezas_en_una_sola_pasada() {
    let r = check_file(&escribir("varios", "toml", VARIOS));
    assert!(!r.ok);
    let por_pieza = |nombre: &str| -> Vec<&str> {
        r.findings.iter().filter(|f| f.piece.as_deref() == Some(nombre)).map(|f| f.code.as_str()).collect()
    };
    // dos errores del contrato y uno del puente, en tres piezas distintas
    assert_eq!(por_pieza("uno"), ["block_missing"]);
    assert_eq!(por_pieza("dos"), ["color_invalid"]);
    assert_eq!(por_pieza("tres"), ["build"]);
    assert_eq!(r.errors(), 3);
}

#[test]
fn cada_incidencia_lleva_la_ruta_del_campo() {
    let r = check_file(&escribir("rutas", "toml", VARIOS));
    let dos = r.findings.iter().find(|f| f.code == "color_invalid").unwrap();
    assert_eq!(dos.path, "post[1].palette.background");
    let tres = r.findings.iter().find(|f| f.code == "build").unwrap();
    assert_eq!(tres.path, "post[2]");
}

#[test]
fn un_error_del_documento_entero_detiene_el_puente() {
    // schema_version afecta a todo: no tiene sentido pedirle nada al puente
    let doc = VARIOS.replace("instatypst-piece/v1", "instatypst-piece/v9");
    assert_eq!(codigos(&escribir("global", "toml", &doc)).iter().filter(|c| *c == "build").count(), 0);
    assert!(codigos(&escribir("global2", "toml", &doc)).contains(&"schema_version".to_string()));
}

#[test]
fn los_avisos_no_cuentan_como_error() {
    let doc = "schema = \"instatypst-piece/v1\"\n[[post]]\nname = \"baja\"\nstyle = \"liso\"\npalette = { background = \"#777777\", text = \"#7a7a7a\" }\ncontent = \"x\"\n";
    let r = check_file(&escribir("aviso", "toml", doc));
    assert!(r.ok, "un aviso de contraste no es un error: {:#?}", r.findings);
    assert!(r.warnings() >= 1 && r.findings.iter().all(|f| f.severity == Severity::Warning && f.code == "contrast_low"));
    assert_eq!(r.findings[0].piece.as_deref(), Some("baja"));
}

#[test]
fn un_archivo_que_no_existe_es_una_incidencia() {
    let r = check_file(Path::new("/tmp/instatypst-no-existe-jamas.toml"));
    assert!(!r.ok);
    assert_eq!(r.findings[0].code, "read");
}

#[test]
fn un_documento_que_no_se_lee_da_parse() {
    let r = check_file(&escribir("roto", "toml", "schema = \"instatypst-piece/v1\"\n[[post]]\nnombre = \"a\"\n"));
    assert_eq!(r.findings[0].code, "parse");
    assert!(r.findings[0].message.contains("nombre"), "el mensaje debe nombrar el campo: {}", r.findings[0].message);
}

#[test]
fn un_posts_toml_de_siempre_tambien_se_valida() {
    let legacy = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../examples/presets.toml");
    let r = check_file(&legacy);
    assert_eq!(r.kind, "posts");
    assert!(r.ok, "{:#?}", r.errors());
    assert!(r.pieces >= 8);
}

#[test]
fn las_tres_sintaxis_dan_el_mismo_informe() {
    let doc = instatypst_cli::model::parse(VARIOS, instatypst_cli::model::Syntax::Toml).unwrap();
    use instatypst_cli::model::{write, Syntax};
    let a = codigos(&escribir("s", "toml", &write(&doc, Syntax::Toml).unwrap()));
    let b = codigos(&escribir("s", "json", &write(&doc, Syntax::Json).unwrap()));
    let c = codigos(&escribir("s", "yaml", &write(&doc, Syntax::Yaml).unwrap()));
    assert_eq!(a, b);
    assert_eq!(b, c);
}

#[test]
fn el_json_tiene_la_forma_documentada() {
    let r = check_file(&escribir("forma", "toml", VARIOS));
    let v: serde_json::Value = serde_json::from_str(&r.to_json()).expect("JSON válido");
    assert_eq!(v["schema"], SCHEMA);
    assert_eq!(v["ok"], false);
    assert_eq!(v["pieces"], 3);
    let f = &v["findings"][0];
    for k in ["severity", "code", "path", "piece", "message"] {
        assert!(f.get(k).is_some(), "falta `{k}` en {f}");
    }
}

#[test]
fn el_texto_resume_y_marca_cada_incidencia() {
    let t = check_file(&escribir("texto", "toml", VARIOS)).to_text();
    assert!(t.contains("✗ post[1].palette.background · pieza \"dos\" [color_invalid]"), "{t}");
    assert!(t.trim_end().ends_with("3 errores"), "{t}");
}

// ── El binario de verdad: código de salida y qué hay en stdout ──

fn instatypst(args: &[&str]) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_instatypst")).args(args).output().expect("se ejecuta")
}

#[test]
fn el_binario_sale_con_0_si_todo_esta_bien_y_con_1_si_no() {
    let bien = fixture("build/nuevos.toml");
    let o = instatypst(&["check", bien.to_str().unwrap(), "--json"]);
    assert_eq!(o.status.code(), Some(0));
    let v: serde_json::Value = serde_json::from_slice(&o.stdout).expect("stdout es solo JSON");
    assert_eq!(v["ok"], true);

    let mal = escribir("binario", "toml", VARIOS);
    let o = instatypst(&["check", mal.to_str().unwrap(), "--json"]);
    assert_eq!(o.status.code(), Some(1), "con errores, sale con 1");
    let v: serde_json::Value = serde_json::from_slice(&o.stdout).expect("stdout sigue siendo solo JSON");
    assert_eq!(v["ok"], false);
}

#[test]
fn el_binario_no_mezcla_trazas_con_el_informe() {
    let o = instatypst(&["check", fixture("build/nuevos.toml").to_str().unwrap(), "--json"]);
    // todo lo que no es el informe va a stderr; stdout se puede leer con un parser
    assert!(serde_json::from_slice::<serde_json::Value>(&o.stdout).is_ok());
}
