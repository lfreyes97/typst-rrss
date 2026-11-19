//! El contrato por ejemplo: los fixtures válidos deben pasar y dar el mismo
//! documento en las tres sintaxis; los inválidos deben fallar con el código
//! que declaran en su primera línea (`# expect: <code>`).

use instatypst_cli::model::{self, Catalog, Slot, Syntax};
use std::fs;
use std::path::PathBuf;

fn dir(sub: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/contrato")
        .join(sub)
}

fn archivos(sub: &str) -> Vec<PathBuf> {
    let mut v: Vec<_> = fs::read_dir(dir(sub))
        .unwrap()
        .map(|e| e.unwrap().path())
        .filter(|p| p.extension().is_some_and(|e| e == "toml"))
        .collect();
    v.sort();
    v
}

#[test]
fn los_validos_pasan() {
    for p in archivos("valid") {
        let texto = fs::read_to_string(&p).unwrap();
        if let Err(issues) = model::load_checked(&texto, Syntax::Toml, &Catalog::embedded()) {
            panic!("{} debería ser válido: {issues:#?}", p.display());
        }
    }
}

#[test]
fn las_tres_sintaxis_dan_el_mismo_documento() {
    for p in archivos("valid") {
        let texto = fs::read_to_string(&p).unwrap();
        let doc = model::parse(&texto, Syntax::Toml).unwrap();
        for s in [Syntax::Toml, Syntax::Json, Syntax::Yaml] {
            let escrito = model::write(&doc, s)
                .unwrap_or_else(|e| panic!("{}: escribir {s:?}: {e}", p.display()));
            let de_vuelta = model::parse(&escrito, s)
                .unwrap_or_else(|e| panic!("{}: leer {s:?}: {e}\n{escrito}", p.display()));
            assert_eq!(doc, de_vuelta, "{} difiere vía {s:?}", p.display());
        }
    }
}

#[test]
fn los_invalidos_fallan_con_su_codigo() {
    for p in archivos("invalid") {
        let texto = fs::read_to_string(&p).unwrap();
        let esperado = texto
            .lines()
            .next()
            .and_then(|l| l.strip_prefix("# expect: "))
            .unwrap_or_else(|| panic!("{} no declara `# expect:`", p.display()))
            .trim()
            .to_string();
        match model::load_checked(&texto, Syntax::Toml, &Catalog::embedded()) {
            Ok(_) => panic!("{} debería fallar con {esperado}", p.display()),
            Err(issues) => assert!(
                issues.iter().any(|i| i.code == esperado),
                "{}: esperaba {esperado}, salió {issues:#?}",
                p.display()
            ),
        }
    }
}

#[test]
fn cada_incidencia_lleva_ruta() {
    let texto = fs::read_to_string(dir("invalid").join("two-titles.toml")).unwrap();
    let issues = model::load(&texto, Syntax::Toml).unwrap_err();
    assert_eq!(issues[0].path, "post[0].content[1]");
}

/// El error es la interfaz del contrato: tiene que nombrar la causa.
#[test]
fn los_errores_de_lectura_nombran_la_causa() {
    for (archivo, debe_decir) in [
        ("palette-old-role", "tinta"),
        ("block-unknown", "remate"),
        ("bad-format", "feed-4x5"),
        ("unknown-field", "colour"),
    ] {
        let texto = fs::read_to_string(dir("invalid").join(format!("{archivo}.toml"))).unwrap();
        let issues = model::load(&texto, Syntax::Toml).unwrap_err();
        assert!(
            issues[0].message.contains(debe_decir),
            "{archivo}: el mensaje no nombra {debe_decir:?}: {}",
            issues[0].message
        );
    }
}

/// El catálogo vendorizado es coherente consigo mismo.
#[test]
fn el_catalogo_es_coherente() {
    let cat = Catalog::embedded();
    assert_eq!(cat.styles.len(), 23, "los 23 presets de página");
    for (nombre, sig) in &cat.styles {
        let mut usados: Vec<&str> = Vec::new();
        for r in &sig.bloques.requeridos {
            match r {
                Slot::One(b) => usados.push(b),
                Slot::AnyOf(v) => usados.extend(v.iter().map(String::as_str)),
            }
        }
        usados.extend(sig.bloques.opcionales.iter().map(String::as_str));
        for b in usados {
            assert!(cat.blocks.iter().any(|c| c == b), "{nombre}: bloque fuera del catálogo: {b}");
        }
        assert!(!sig.options.is_empty(), "{nombre}: sin options derivadas");
    }
}

/// Todo bloque del modelo debe poder aparecer en el catálogo, y al revés.
#[test]
fn los_bloques_del_modelo_son_los_del_catalogo() {
    let cat = Catalog::embedded();
    let del_modelo = [
        "title", "subtitle", "label", "body", "quote", "epigraph", "verse", "conclusion",
        "attribution", "list", "contrast", "signature", "footer", "number", "date",
    ];
    let mut a: Vec<&str> = cat.blocks.iter().map(String::as_str).collect();
    let mut b = del_modelo.to_vec();
    a.sort();
    b.sort();
    assert_eq!(a, b, "el modelo de Rust y firmas.json deben coincidir");
}

#[test]
fn el_mensaje_de_estilo_desconocido_lista_los_validos() {
    let texto = fs::read_to_string(dir("invalid").join("style-unknown.toml")).unwrap();
    let issues = model::load_checked(&texto, Syntax::Toml, &Catalog::embedded()).unwrap_err();
    assert!(issues[0].message.contains("suizo"), "{}", issues[0].message);
}

#[test]
fn el_verso_hereda_de_cita_y_cuerpo() {
    let cat = Catalog::embedded();
    assert_eq!(cat.cadena("verse"), ["verse", "quote", "body"]);
    assert_eq!(cat.cadena("epigraph"), ["epigraph", "quote", "body"]);
    assert_eq!(cat.cadena("title"), ["title"]);
}

/// Un verso satisface un requisito de `body` (liso lo pide) aunque no lo sea.
#[test]
fn un_verso_satisface_el_requisito_de_cuerpo() {
    let t = "[[post]]\nname='a'\nstyle='liso'\n[[post.content]]\ntype='verse'\nlines=['a','b']";
    assert!(model::load_checked(t, Syntax::Toml, &Catalog::embedded()).is_ok());
}

/// Escrito en Typst o como lista de bloques, el contenido da el mismo resultado.
#[test]
fn typst_y_bloques_son_equivalentes() {
    let typst = "[[post]]\nname='a'\nstyle='suizo'\ncontent = \"\"\"\n#label[DEL ENSAYO]\n= Título\n#conclusion[PUREZA.]\n\"\"\"";
    let bloques = "[[post]]\nname='a'\nstyle='suizo'\n[[post.content]]\ntype='label'\ntext='DEL ENSAYO'\n[[post.content]]\ntype='title'\ntext='Título'\n[[post.content]]\ntype='conclusion'\ntext='PUREZA.'";
    let cat = Catalog::embedded();
    let a = model::load_checked(typst, Syntax::Toml, &cat).unwrap();
    let b = model::load_checked(bloques, Syntax::Toml, &cat).unwrap();
    let (derivados, _) = match &a.posts[0].content {
        Some(instatypst_cli::model::Content::Markup(m)) => instatypst_cli::markup::parse(m),
        _ => panic!("esperaba contenido Typst"),
    };
    match &b.posts[0].content {
        Some(instatypst_cli::model::Content::Blocks(bs)) => assert_eq!(&derivados, bs),
        _ => panic!("esperaba bloques"),
    }
}

#[test]
fn los_errores_de_typst_llevan_linea_y_columna() {
    let t = fs::read_to_string(dir("invalid").join("markup-read.toml")).unwrap();
    let issues = model::load_checked(&t, Syntax::Toml, &Catalog::embedded()).unwrap_err();
    assert!(issues[0].path.starts_with("post[0].content@"), "{}", issues[0].path);
}
