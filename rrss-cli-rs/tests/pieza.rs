//! El puente pieza → Config → .typ: que construya, que falle diciendo por qué y,
//! sobre todo, que dé la misma imagen que el formato de siempre.

use rrss_cli_rs::config::Config;
use rrss_cli_rs::model::{self, Catalog, Syntax};
use rrss_cli_rs::{pieza, process, render};
use std::fs;
use std::path::{Path, PathBuf};

fn fixture(rel: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/contrato")
        .join(rel)
}

/// Una pieza en texto → Config, sin tocar el disco.
fn construir(texto: &str) -> anyhow::Result<Config> {
    let cat = Catalog::embedded();
    let doc = model::load_checked(texto, Syntax::Toml, &cat)
        .map_err(|i| anyhow::anyhow!("{}", pieza::formatear(&i)))?;
    pieza::a_config(&doc, &cat)
}

fn error_de(texto: &str) -> String {
    format!("{:#}", construir(texto).expect_err("debería fallar"))
}

// ── Detección ───────────────────────────────────────────────

#[test]
fn distingue_piezas_del_posts_toml_de_siempre() {
    let legacy = "[[post]]\nname='a'\npreset='liso'\npos=['x']";
    let nuevo = "[[post]]\nname='a'\nstyle='liso'\ncontent='x'";
    let con_schema = "schema='instatypst-piece/v1'";
    assert!(!pieza::es_de_piezas(legacy, Syntax::Toml));
    assert!(pieza::es_de_piezas(nuevo, Syntax::Toml));
    assert!(pieza::es_de_piezas(con_schema, Syntax::Toml));
    assert!(!pieza::es_de_piezas("esto no es toml [[", Syntax::Toml));
}

// ── Construir ───────────────────────────────────────────────

#[test]
fn los_fixtures_construibles_se_renderizan() {
    for archivo in ["build/paridad.toml", "build/nuevos.toml"] {
        let cfg = pieza::cargar(&fixture(archivo)).unwrap_or_else(|e| panic!("{archivo}: {e:#}"));
        let posts = cfg.resolve_all().unwrap_or_else(|e| panic!("{archivo}: {e:#}"));
        assert!(!posts.is_empty());
        for p in &posts {
            let r = render::render(p, &cfg).unwrap_or_else(|e| panic!("{}: {e:#}", p.name));
            assert!(
                r.typ.contains(&format!("#{}(", p.preset)),
                "{}: el .typ no llama a {}\n{}",
                p.name,
                p.preset,
                r.typ
            );
        }
    }
}

#[test]
fn las_tres_sintaxis_construyen_lo_mismo() {
    let original = fs::read_to_string(fixture("build/nuevos.toml")).unwrap();
    let doc = model::parse(&original, Syntax::Toml).unwrap();
    let dir = std::env::temp_dir().join(format!("rrss-sintaxis-{}", std::process::id()));
    fs::create_dir_all(&dir).unwrap();

    let typ_de = |ruta: &Path| -> Vec<String> {
        let cfg = pieza::cargar(ruta).unwrap();
        cfg.resolve_all()
            .unwrap()
            .iter()
            .map(|p| render::render(p, &cfg).unwrap().typ)
            .collect()
    };
    let base = typ_de(&fixture("build/nuevos.toml"));
    for (ext, s) in [("json", Syntax::Json), ("yaml", Syntax::Yaml), ("toml", Syntax::Toml)] {
        let ruta = dir.join(format!("p.{ext}"));
        fs::write(&ruta, model::write(&doc, s).unwrap()).unwrap();
        assert_eq!(base, typ_de(&ruta), "el .typ difiere si el documento es {ext}");
    }
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn un_posts_toml_de_siempre_sigue_cargando() {
    let ruta = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../examples/presets.toml");
    let cfg = pieza::cargar(&ruta).unwrap();
    assert!(cfg.resolve_all().unwrap().len() >= 8);
}

// ── Lo que el puente no traduce falla diciéndolo ────────────

const BASE: &str = "[[post]]\nname='a'\n";

#[test]
fn un_estilo_sin_entradas_lo_dice() {
    let e = error_de(&format!("{BASE}style='capitulo-orlado'\n[[post.content]]\ntype='number'\nvalue=1\n[[post.content]]\ntype='title'\ntext='x'"));
    assert!(e.contains("aún no se puede construir"), "{e}");
}

#[test]
fn un_rol_de_paleta_que_no_pasa_lo_dice() {
    let e = error_de(&format!("{BASE}style='liso'\npalette={{ background='#000000', good='#00ff00' }}\ncontent='x'"));
    assert!(e.contains("good"), "{e}");
}

#[test]
fn un_fondo_sin_traduccion_lo_dice() {
    let e = error_de(&format!("{BASE}style='liso'\nbackground={{ type='grid' }}\ncontent='x'"));
    assert!(e.contains("fondo"), "{e}");
}

#[test]
fn un_campo_sin_destino_en_el_preset_lo_dice() {
    // liso no tiene dónde poner la fecha de la atribución
    let e = error_de(&format!("{BASE}style='liso'\n[[post.content]]\ntype='body'\ntext='x'\n[[post.content]]\ntype='attribution'\nauthor='A'\ndate='1900'"));
    assert!(e.contains("attribution.date"), "{e}");
}

#[test]
fn la_tipografia_se_traduce_o_se_rechaza() {
    let ok = construir(&format!("{BASE}style='liso'\ntypography={{ body={{ family='Jost', weight=500 }} }}\ncontent='x'")).unwrap();
    assert_eq!(ok.posts[0].args["fuente"].as_str(), Some("Jost"));
    assert_eq!(ok.posts[0].args["peso"].as_integer(), Some(500));
    let e = error_de(&format!("{BASE}style='liso'\ntypography={{ heading={{ family='Jost' }} }}\ncontent='x'"));
    assert!(e.contains("heading"), "{e}");
}

#[test]
fn las_paletas_del_documento_no_borran_las_de_fabrica() {
    let cfg = construir(&format!("[palettes.marca]\nbackground='#102030'\ntext='#f0f0f0'\n{BASE}style='suizo'\npalette='marca'\ncontent='''\n= T\n#conclusion[C]\n'''")).unwrap();
    let t = cfg.themes.unwrap();
    assert!(t.contains_key("marca") && t.contains_key("ocean") && t.contains_key("nord"));
}

// ── Paridad: la misma imagen que el formato de siempre ──────

fn entorno_para_compilar() -> Option<(PathBuf, Vec<PathBuf>)> {
    std::process::Command::new("typst").arg("--version").output().ok()?;
    let home = std::env::var("HOME").ok()?;
    if !Path::new(&home).join(".local/share/typst/packages/local/instatypst").exists() {
        return None;
    }
    let fuentes = PathBuf::from("/home/home/Documents/Instatypst/Fonts");
    fuentes.is_dir().then(|| (std::env::temp_dir(), vec![fuentes]))
}

fn imagen(cfg: &Config, nombre: &str, dir: &Path, etiqueta: &str, fuentes: &[PathBuf]) -> Vec<u8> {
    let posts = cfg.resolve_all().unwrap();
    let p = posts.iter().find(|p| p.name == nombre).unwrap_or_else(|| panic!("no hay {nombre}"));
    let typ = dir.join(format!("{etiqueta}.typ"));
    let png = dir.join(format!("{etiqueta}.png"));
    fs::write(&typ, render::render(p, cfg).unwrap().typ).unwrap();
    process::compile_typst_with(&typ, &png, 72, dir, fuentes).unwrap_or_else(|e| panic!("{nombre}: {e:#}"));
    fs::read(&png).unwrap()
}

/// Cuatro estilos, escritos como piezas y como posts de siempre: la imagen es la misma.
#[test]
fn una_pieza_da_la_misma_imagen_que_el_post_de_siempre() {
    let Some((tmp, fuentes)) = entorno_para_compilar() else {
        eprintln!("paridad omitida: falta typst, el paquete instatypst o las fuentes");
        return;
    };
    let dir = tmp.join(format!("rrss-paridad-{}", std::process::id()));
    fs::create_dir_all(&dir).unwrap();

    let viejo = Config::load(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../examples/presets.toml")).unwrap();
    let nuevo = pieza::cargar(&fixture("build/paridad.toml")).unwrap();
    for n in ["presets/liso", "presets/suizo", "presets/epigrafe", "presets/esquela"] {
        let seguro = n.replace('/', "-");
        let a = imagen(&viejo, n, &dir, &format!("viejo-{seguro}"), &fuentes);
        let b = imagen(&nuevo, n, &dir, &format!("nuevo-{seguro}"), &fuentes);
        assert!(a == b, "{n}: la pieza nueva no da la misma imagen que el post de siempre");
    }
    let _ = fs::remove_dir_all(&dir);
}

// ── Marca ───────────────────────────────────────────────────

fn marca_de(cfg: &Config, i: usize) -> Option<String> {
    cfg.posts[i]
        .args
        .get("marca")
        .and_then(|v| v.as_table())
        .and_then(|t| t.get("typst"))
        .and_then(|v| v.as_str())
        .map(String::from)
}

fn liso_con(brand: &str, extra: &str) -> String {
    format!("{brand}\n[[post]]\nname='a'\nstyle='liso'\n{extra}content='x'\n")
}

#[test]
fn la_marca_de_texto_se_traduce_a_una_funcion() {
    let cfg = construir(&liso_con("[brand]\nlogo={ text='@iglesiaraah' }", "")).unwrap();
    assert_eq!(marca_de(&cfg, 0).as_deref(), Some(r#"marca.texto("@iglesiaraah")"#));
}

#[test]
fn la_marca_de_imagen_distingue_svg_de_mapa_de_bits() {
    let svg = construir(&liso_con("[brand]\nlogo={ image='assets/Logo.svg' }", "")).unwrap();
    assert_eq!(marca_de(&svg, 0).as_deref(), Some(r#"marca.svg(read("/assets/Logo.svg"))"#));
    let png = construir(&liso_con("[brand]\nlogo={ image='assets/logo.png' }\nsize=0.05", "")).unwrap();
    assert_eq!(
        marca_de(&png, 0).as_deref(),
        Some(r#"marca.imagen(read("/assets/logo.png", encoding: none), proporcion: 0.05)"#)
    );
}

#[test]
fn la_marca_de_typst_importa_tu_funcion() {
    let cfg = construir(&liso_con("[brand]\nlogo={ typst='logo.typ', fn='logo' }", "")).unwrap();
    assert_eq!(marca_de(&cfg, 0).as_deref(), Some(r#"{ import "/logo.typ": logo; logo }"#));
}

#[test]
fn el_texto_de_la_marca_no_se_sale_del_string() {
    let cfg = construir(&liso_con("[brand]\nlogo={ text='a\"); #read(\"x\"); (\"b' }", "")).unwrap();
    let e = marca_de(&cfg, 0).unwrap();
    // las comillas del texto van escapadas: sigue siendo un solo literal
    assert!(e.starts_with(r#"marca.texto("a\")"#), "{e}");
}

#[test]
fn la_pieza_manda_sobre_la_serie_y_false_la_quita() {
    let doc = "[brand]\nlogo={ text='SERIE' }\n\
               [[post]]\nname='a'\nstyle='liso'\ncontent='x'\n\
               [[post]]\nname='b'\nstyle='liso'\nbrand=false\ncontent='x'\n\
               [[post]]\nname='c'\nstyle='liso'\nbrand={ logo={ text='PROPIA' } }\ncontent='x'\n";
    let cfg = construir(doc).unwrap();
    assert!(marca_de(&cfg, 0).unwrap().contains("SERIE"));
    assert!(marca_de(&cfg, 1).is_none(), "brand = false la quita");
    assert!(marca_de(&cfg, 2).unwrap().contains("PROPIA"));
}

#[test]
fn una_firma_de_texto_en_la_pieza_manda_sobre_la_marca() {
    let doc = "[brand]\nlogo={ text='SERIE' }\n[[post]]\nname='a'\nstyle='liso'\ncontent='''\nCuerpo.\n#signature[\\@otro]\n'''\n";
    let cfg = construir(doc).unwrap();
    let marca = &cfg.posts[0].args["marca"];
    assert!(marca.as_table().unwrap().contains_key("md"), "la firma de texto es contenido: {marca:?}");
}

#[test]
fn un_estilo_que_no_acepta_funcion_lo_dice() {
    let e = error_de("[brand]\nlogo={ text='X' }\n[[post]]\nname='a'\nstyle='epigrafe'\ncontent='Cita.'");
    assert!(e.contains("aún no acepta una marca"), "{e}");
    // y se resuelve quitándola en esa pieza
    construir("[brand]\nlogo={ text='X' }\n[[post]]\nname='a'\nstyle='epigrafe'\nbrand=false\ncontent='Cita.'").unwrap();
}

#[test]
fn la_marca_llega_al_typ_generado() {
    let cfg = construir(&liso_con("[brand]\nlogo={ image='assets/Logo.svg' }", "")).unwrap();
    let p = &cfg.resolve_all().unwrap()[0];
    let typ = render::render(p, &cfg).unwrap().typ;
    assert!(typ.contains(r#"marca: (marca.svg(read("/assets/Logo.svg")))"#), "{typ}");
}

/// Compilación real: la marca (SVG recoloreado o función propia) cambia la imagen.
#[test]
fn la_marca_se_compila_de_verdad() {
    let Some((tmp, fuentes)) = entorno_para_compilar() else {
        eprintln!("marca omitida: falta typst, el paquete instatypst o las fuentes");
        return;
    };
    let dir = tmp.join(format!("rrss-marca-{}", std::process::id()));
    fs::create_dir_all(dir.join("assets")).unwrap();
    fs::copy(fixture("build/marca/logo-demo.svg"), dir.join("assets/logo-demo.svg")).unwrap();
    fs::copy(fixture("build/marca/logo.typ"), dir.join("logo.typ")).unwrap();

    let de = |brand: &str, nombre: &str| -> Vec<u8> {
        let ruta = dir.join(format!("{nombre}.toml"));
        fs::write(
            &ruta,
            format!("schema='instatypst-piece/v1'\n{brand}\n[[post]]\nname='{nombre}'\nstyle='liso'\npalette={{ background='#14213d', text='#ffffff' }}\ncontent='Todo es gracia.'\n"),
        )
        .unwrap();
        let cfg = pieza::cargar(&ruta).unwrap();
        imagen(&cfg, nombre, &dir, nombre, &fuentes)
    };
    let sin = de("", "sin");
    let svg = de("[brand]\nlogo={ image='assets/logo-demo.svg' }", "svg");
    let func = de("[brand]\nlogo={ typst='logo.typ', fn='logo' }", "func");
    let texto = de("[brand]\nlogo={ text='@iglesiaraah' }", "texto");
    for (n, b) in [("svg", &svg), ("func", &func), ("texto", &texto)] {
        assert!(b != &sin, "la marca {n} no cambió la imagen");
    }
    assert!(svg != func && svg != texto && func != texto, "las tres marcas deberían verse distintas");
    let _ = fs::remove_dir_all(&dir);
}

/// Un caso real: la lámina 01 de la campaña Warfield, hecha a mano con su `logo(color, lienzo)`,
/// y la misma como pieza con `brand.typst`. Se omite si no está la campaña.
#[test]
fn la_marca_de_la_campana_da_la_lamina_01_de_siempre() {
    let campana = PathBuf::from("/home/home/Documents/Instatypst");
    let Some((tmp, fuentes)) = entorno_para_compilar() else {
        eprintln!("omitida: falta typst, el paquete o las fuentes");
        return;
    };
    if !campana.join("trabajos/campana/_comun.typ").exists() {
        eprintln!("omitida: no está trabajos/campana");
        return;
    }
    let dir = tmp.join(format!("rrss-campana-{}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(dir.join("trabajos/campana")).unwrap();
    let copiar = |de: &Path, a: &Path| {
        fs::create_dir_all(a).unwrap();
        for e in walk(de) {
            let rel = e.strip_prefix(de).unwrap();
            let destino = a.join(rel);
            fs::create_dir_all(destino.parent().unwrap()).unwrap();
            fs::copy(&e, &destino).unwrap();
        }
    };
    copiar(&campana.join("src"), &dir.join("src"));
    for f in ["_comun.typ", "logo.typ", "01-calvinismo-pureza.typ"] {
        fs::copy(campana.join("trabajos/campana").join(f), dir.join("trabajos/campana").join(f)).unwrap();
    }
    fs::copy(campana.join("typst.toml"), dir.join("typst.toml")).unwrap();

    // a mano
    let original = dir.join("original.png");
    process::compile_typst_with(&dir.join("trabajos/campana/01-calvinismo-pureza.typ"), &original, 72, &dir, &fuentes).unwrap();

    // como pieza, con la marca de la campaña
    let ruta = dir.join("pieza.toml");
    fs::write(&ruta, "schema='instatypst-piece/v1'\n[brand]\nlogo={ typst='trabajos/campana/_comun.typ', fn='logo' }\n\
        [[post]]\nname='01'\nstyle='suizo'\nformat='portrait'\ncontent='''\n#label[DEL ENSAYO]\n= El calvinismo es simplemente la religión en su\n== ¿Qué es el calvinismo?\n#conclusion[PUREZA.]\n#attribution(author: \"B. B. WARFIELD\", date: \"1851 – 1921\")\n'''\n").unwrap();
    let cfg = pieza::cargar(&ruta).unwrap();
    let nuevo = imagen(&cfg, "01", &dir, "pieza", &fuentes);
    assert!(fs::read(&original).unwrap() == nuevo, "la lámina 01 como pieza no es idéntica a la hecha a mano");
    let _ = fs::remove_dir_all(&dir);
}

fn walk(d: &Path) -> Vec<PathBuf> {
    let mut v = Vec::new();
    for e in fs::read_dir(d).unwrap().flatten() {
        let p = e.path();
        if p.is_dir() {
            v.extend(walk(&p));
        } else {
            v.push(p);
        }
    }
    v
}

// ── Epígrafe ────────────────────────────────────────────────

#[test]
fn en_liso_el_epigrafe_va_aparte_del_cuerpo() {
    let doc = "[[post]]\nname='a'\nstyle='liso'\ncontent='''\n#epigraph(source: \"Efesios 2:8\")[«Por gracia sois salvos.»]\nTodo es gracia.\n'''";
    let cfg = construir(doc).unwrap();
    let a = &cfg.posts[0].args;
    assert!(a.contains_key("epigrafe"), "{a:?}");
    assert!(a.contains_key("epigrafe-fuente"), "{a:?}");
    // el cuerpo es lo otro: no se mezcló con el epígrafe
    let texto = cfg.posts[0].pos[0].as_table().unwrap()["md"].as_str().unwrap();
    assert_eq!(texto, "Todo es gracia.");
}

#[test]
fn sin_entrada_propia_el_epigrafe_se_dibuja_como_cuerpo() {
    // esquela no tiene `epigraph`: cae en `body` (su `duelo`); sin fuente, pasa
    let ok = construir("[[post]]\nname='a'\nstyle='esquela'\ncontent='''\n= Expiación\n#epigraph[«Una cita.»]\n'''").unwrap();
    assert!(ok.posts[0].args.contains_key("duelo"));
    // pero una fuente no tiene dónde ir, y lo dice
    let e = error_de("[[post]]\nname='a'\nstyle='esquela'\ncontent='''\n= Expiación\n#epigraph(source: \"X\")[«Una cita.»]\n'''");
    assert!(e.contains("fuente de un epígrafe"), "{e}");
}

#[test]
fn liso_exige_cuerpo_aunque_haya_epigrafe() {
    let e = error_de("[[post]]\nname='a'\nstyle='liso'\ncontent='''\n#epigraph[«Sola.»]\n'''");
    assert!(e.contains("body") || e.contains("necesita"), "{e}");
}

#[test]
fn liso_con_epigrafe_se_compila_y_cambia_la_imagen() {
    let Some((tmp, fuentes)) = entorno_para_compilar() else {
        eprintln!("omitida: falta typst, el paquete o las fuentes");
        return;
    };
    let dir = tmp.join(format!("rrss-epigrafe-{}", std::process::id()));
    fs::create_dir_all(&dir).unwrap();
    let de = |contenido: &str, nombre: &str| -> Vec<u8> {
        let ruta = dir.join(format!("{nombre}.toml"));
        fs::write(&ruta, format!("schema='instatypst-piece/v1'\n[[post]]\nname='{nombre}'\nstyle='liso'\npalette={{ background='#14213d', text='#ffffff' }}\ncontent='''\n{contenido}\n'''\n")).unwrap();
        imagen(&pieza::cargar(&ruta).unwrap(), nombre, &dir, nombre, &fuentes)
    };
    let sin = de("Todo es gracia.", "sin");
    let con = de("#epigraph(source: \"Efesios 2:8\")[«Por gracia sois salvos.»]\nTodo es gracia.", "con");
    assert!(sin != con, "el epígrafe no cambió la imagen");
    let _ = fs::remove_dir_all(&dir);
}

// ── Estilo propio ───────────────────────────────────────────

#[test]
fn un_estilo_propio_recibe_la_pieza_resuelta() {
    let cfg = pieza::cargar(&fixture("valid/estilo-propio.toml")).unwrap();
    let posts = cfg.resolve_all().unwrap();
    assert_eq!(posts.len(), 2);
    let e = posts[0].estilo_propio.as_ref().expect("estilo propio");
    assert_eq!((e.archivo.as_str(), e.funcion.as_str()), ("layouts/mi-estilo.typ", "mi-estilo"));
    // la posición en la serie la cuenta el documento
    assert_eq!(e.pieza["index"].as_integer(), Some(1));
    assert_eq!(e.pieza["total"].as_integer(), Some(2));
    assert_eq!(posts[1].estilo_propio.as_ref().unwrap().pieza["index"].as_integer(), Some(2));

    let typ = render::render(&posts[0], &cfg).unwrap().typ;
    assert!(typ.contains(r#"#import "/layouts/mi-estilo.typ": mi-estilo"#), "{typ}");
    assert!(typ.contains("#mi-estilo("), "{typ}");
    assert!(typ.contains("format: (1080, 1350)"), "{typ}");
    assert!(typ.contains("background: rgb(\"#691322\")"), "{typ}");
    assert!(typ.contains("tema: \"granates\""), "las opciones viajan tal cual: {typ}");
}

#[test]
fn un_estilo_propio_no_lee_un_tema_de_instatypst() {
    let cfg = construir("[[post]]\nname='a'\nstyle={ file='x.typ', fn='f' }\npalette='suizo'\ncontent='x'").unwrap();
    let p = &cfg.resolve_all().unwrap()[0];
    let e = format!("{:#}", render::render(p, &cfg).err().expect("debería fallar"));
    assert!(e.contains("tema de Instatypst"), "{e}");
}

#[test]
fn los_bloques_llegan_con_su_tipo_real() {
    let cfg = construir("[[post]]\nname='a'\nstyle={ file='x.typ', fn='f' }\ncontent='''\n#verse[uno \\ dos]\n#epigraph(source: \"S\")[E]\n#number(2, total: 5)\n'''").unwrap();
    let p = &cfg.resolve_all().unwrap()[0];
    let typ = render::render(p, &cfg).unwrap().typ;
    // sin parentesco: un verso sigue siendo verse, y un epígrafe, epigraph
    assert!(typ.contains(r#"type: "verse""#) && typ.contains(r#"type: "epigraph""#), "{typ}");
    assert!(typ.contains("value: 2") && typ.contains("total: 5"), "{typ}");
}

#[test]
fn la_marca_llega_al_estilo_propio_como_funcion() {
    let cfg = construir("[brand]\nlogo={ text='@x' }\n[[post]]\nname='a'\nstyle={ file='x.typ', fn='f' }\ncontent='x'").unwrap();
    let typ = render::render(&cfg.resolve_all().unwrap()[0], &cfg).unwrap().typ;
    assert!(typ.contains(r#"brand: (marca.texto("@x"))"#), "{typ}");
}

#[test]
fn el_estilo_propio_se_compila_de_verdad() {
    let Some((tmp, fuentes)) = entorno_para_compilar() else {
        eprintln!("omitida: falta typst, el paquete o las fuentes");
        return;
    };
    let dir = tmp.join(format!("rrss-propio-{}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(dir.join("layouts")).unwrap();
    fs::copy(fixture("build/estilo/mi-estilo.typ"), dir.join("layouts/mi-estilo.typ")).unwrap();
    fs::copy(fixture("valid/estilo-propio.toml"), dir.join("piezas.toml")).unwrap();
    let cfg = pieza::cargar(&dir.join("piezas.toml")).unwrap();
    let a = imagen(&cfg, "propio/1", &dir, "uno", &fuentes);
    let b = imagen(&cfg, "propio/2", &dir, "dos", &fuentes);
    assert!(!a.is_empty() && a != b, "las dos piezas deberían verse distintas");
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn las_claves_opcionales_de_la_pieza_siempre_existen() {
    let cfg = construir("[[post]]\nname='a'\nstyle={ file='x.typ', fn='f' }\ncontent='x'").unwrap();
    let typ = render::render(&cfg.resolve_all().unwrap()[0], &cfg).unwrap().typ;
    for clave in ["format: none", "palette: none", "brand: none", "variant: none", "background: none", "typography: none"] {
        assert!(typ.contains(clave), "falta `{clave}`: {typ}");
    }
}

/// Un caso real: el carrusel «La disociación» (Post/disociacion.typ), hecho a mano, y el mismo
/// como 9 piezas con un estilo propio que conserva su diseño línea por línea. Cada lámina sale
/// idéntica. Se omite si no está el original.
#[test]
fn la_disociacion_como_piezas_da_las_laminas_de_siempre() {
    let Some((tmp, fuentes)) = entorno_para_compilar() else {
        eprintln!("omitida: falta typst, el paquete o las fuentes");
        return;
    };
    let original = PathBuf::from("/home/home/Proyectos/Post");
    if !original.join("disociacion.typ").exists() {
        eprintln!("omitida: no está Post/disociacion.typ");
        return;
    }
    let dir = tmp.join(format!("rrss-disociacion-{}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(dir.join("layouts")).unwrap();
    let real = fixture("real/disociacion");
    fs::copy(real.join("layouts/disociacion.typ"), dir.join("layouts/disociacion.typ")).unwrap();
    fs::copy(real.join("disociacion.toml"), dir.join("disociacion.toml")).unwrap();

    // a mano: las 9 láminas del original
    let viejas = dir.join("viejo");
    fs::create_dir_all(&viejas).unwrap();
    process::compile_typst_with(&original.join("disociacion.typ"), &viejas.join("{0p}.png"), 72, &original, &fuentes).unwrap();

    // como piezas
    let cfg = pieza::cargar(&dir.join("disociacion.toml")).unwrap();
    for n in 1..=9 {
        let nuevo = imagen(&cfg, &format!("dis/0{n}"), &dir, &format!("n{n}"), &fuentes);
        let viejo = fs::read(viejas.join(format!("{n}.png"))).unwrap();
        assert!(viejo == nuevo, "la lámina {n} no es idéntica a la hecha a mano");
    }
    let _ = fs::remove_dir_all(&dir);
}

// ── El rótulo (`label`) ─────────────────────────────────────

#[test]
fn el_rotulo_puede_repetirse_en_un_estilo_propio() {
    // marca quiebres dentro del flujo: varios rótulos en una misma pieza
    let cfg = construir("[[post]]\nname='a'\nstyle={ file='x.typ', fn='f' }\ncontent='''\n#label[Uno]\n= T\n#label[Dos]\nCuerpo.\n#label[Tres]\n'''").unwrap();
    let typ = render::render(&cfg.resolve_all().unwrap()[0], &cfg).unwrap().typ;
    assert_eq!(typ.matches(r#"type: "label""#).count(), 3, "{typ}");
}

#[test]
fn un_estilo_con_un_solo_lugar_para_el_rotulo_no_admite_dos() {
    let e = error_de("[[post]]\nname='a'\nstyle='suizo'\ncontent='''\n#label[Uno]\n#label[Dos]\n= T\n#conclusion[C]\n'''");
    assert!(e.contains("solo tiene un lugar para `label`"), "{e}");
    // y con uno solo, sin problema
    construir("[[post]]\nname='a'\nstyle='suizo'\ncontent='''\n#label[Uno]\n= T\n#conclusion[C]\n'''").unwrap();
}
