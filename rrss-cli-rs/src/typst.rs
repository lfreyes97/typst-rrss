//! Emitir datos como literales de Typst, escapados.
//!
//! Todo lo que llega de fuera (flags, `posts.toml`) y acaba dentro de un
//! `.typ` pasa por aquí. Nada se interpola a mano con `format!("\"{}\"", ..)`.

use anyhow::{bail, Result};

/// Literal de string de Typst: `"..."`, con `\`, `"` y los caracteres de
/// control escapados. Un `#`, `$` o `*` dentro de un string no significan nada
/// para Typst, así que no se tocan.
pub fn str_lit(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 2);
    out.push('"');
    for c in s.chars() {
        match c {
            '\\' => out.push_str("\\\\"),
            '"' => out.push_str("\\\""),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if c.is_control() => out.push_str(&format!("\\u{{{:x}}}", c as u32)),
            c => out.push(c),
        }
    }
    out.push('"');
    out
}

/// Contenido (markup) construido desde un string: `eval("...", mode: "markup")`.
/// El texto no puede salirse del string, pero Typst SÍ lo interpreta: `#...`
/// dentro del texto se ejecuta como código. Usar solo si el autor del TOML
/// quiere formato (`*negrita*`); para texto sin formato, `txt_lit`.
///
/// Ojo: un `str` y un contenido (`[...]`) NO son equivalentes dentro de los
/// presets de instatypst. Por ejemplo, `carteles.mosaico` reparte distinto la
/// `etiqueta` si es un `str`. Contenido = `{ md }` / `{ txt }` / `*_md` / `*_txt`.
pub fn markup_lit(s: &str) -> String {
    format!("eval({}, mode: \"markup\", scope: _alcance)", str_lit(s))
}

/// Escapa lo que Typst interpreta en markup, para que el texto salga tal cual:
/// `\ # * _ ` $ @ < > [ ] ~` en cualquier sitio, y los marcadores de línea
/// (`= `, `- `, `+ `, `/ `, `1. `) cuando abren una línea. Los atajos tipográficos
/// (`--`, `...`, comillas inteligentes) se dejan: igual que en un `[...]` escrito a mano.
pub fn escape_markup(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 8);
    for (i, linea) in s.split('\n').enumerate() {
        if i > 0 {
            out.push('\n');
        }
        let resto = linea.trim_start();
        let sangria = &linea[..linea.len() - resto.len()];
        out.push_str(sangria);

        let marcador = resto
            .strip_prefix(['=', '-', '+', '/'])
            .is_some_and(|r| r.is_empty() || r.starts_with(' '));
        let num = resto
            .split_once(". ")
            .is_some_and(|(n, _)| !n.is_empty() && n.chars().all(|c| c.is_ascii_digit()));
        for (j, c) in resto.char_indices() {
            let al_inicio = j == 0;
            let especial = "\\#*_`$@<>[]~".contains(c)
                || (al_inicio && marcador)
                || (num && c == '.' && resto[..j].chars().all(|d| d.is_ascii_digit()) && j > 0);
            if especial {
                out.push('\\');
            }
            out.push(c);
        }
    }
    out
}

/// Contenido de texto plano: lo que escribirías entre `[` y `]`, sin que nada se
/// interprete. Misma estructura que un `[...]` a mano (secuencia de texto y espacios).
pub fn txt_lit(s: &str) -> String {
    markup_lit(&escape_markup(s))
}

/// `true` si `k` sirve como identificador de Typst (nombre de argumento).
pub fn is_ident(k: &str) -> bool {
    let mut it = k.chars();
    matches!(it.next(), Some(c) if c.is_alphabetic() || c == '_')
        && it.all(|c| c.is_alphanumeric() || c == '_' || c == '-')
}

/// Valor TOML como literal de Typst. Los strings siempre salen como `str`.
/// Las tablas pasan a diccionarios y los arrays a arrays; las claves deben ser
/// identificadores válidos. Fechas y horas de TOML no tienen equivalente: error.
pub fn value_lit(v: &toml::Value) -> Result<String> {
    Ok(match v {
        toml::Value::String(s) => str_lit(s),
        toml::Value::Integer(i) => i.to_string(),
        toml::Value::Float(f) if f.is_finite() => format!("{f:?}"),
        toml::Value::Float(f) => bail!("número no finito: {f}"),
        toml::Value::Boolean(b) => b.to_string(),
        toml::Value::Array(a) => {
            let items = a.iter().map(value_lit).collect::<Result<Vec<_>>>()?;
            match items.len() {
                1 => format!("({},)", items[0]),
                _ => format!("({})", items.join(", ")),
            }
        }
        toml::Value::Table(t) => {
            if let Some(lit) = tagged_lit(t)? {
                return Ok(lit);
            }
            if t.is_empty() {
                return Ok("(:)".into());
            }
            let mut parts = Vec::new();
            for (k, v) in t {
                if !is_ident(k) {
                    bail!("clave no válida como identificador de Typst: {k:?}");
                }
                parts.push(format!("{k}: {}", value_lit(v)?));
            }
            format!("({})", parts.join(", "))
        }
        toml::Value::Datetime(_) => bail!("las fechas de TOML no se pueden pasar a Typst"),
    })
}

/// Argumentos con nombre: `clave: valor, ...`, en el orden de la tabla.
pub fn named_args(t: &toml::Table) -> Result<String> {
    let mut parts = Vec::new();
    for (k, v) in t {
        if let toml::Value::String(s) = v {
            let sufijo = [("_md", markup_lit as fn(&str) -> String), ("_txt", txt_lit)];
            if let Some((base, f)) = sufijo
                .iter()
                .find_map(|(suf, f)| k.strip_suffix(suf).map(|b| (b, f)))
            {
                if !is_ident(base) {
                    bail!("argumento no válido como identificador de Typst: {base:?}");
                }
                parts.push(format!("{base}: {}", f(s)));
                continue;
            }
        }
        if !is_ident(k) {
            bail!("argumento no válido como identificador de Typst: {k:?}");
        }
        parts.push(format!("{k}: {}", value_lit(v)?));
    }
    Ok(parts.join(", "))
}

/// Ruta de un recurso para Typst: relativa a la raíz del proyecto (`/assets/x.jpg`),
/// así no depende de dónde se escriba el `.typ`. Rechaza `..` y bytes nulos.
pub fn asset_path(p: &str) -> Result<String> {
    if p.is_empty() || p.contains('\0') {
        bail!("ruta de recurso vacía o con bytes nulos");
    }
    if p.split('/').any(|c| c == "..") {
        bail!("la ruta {p:?} sale de la raíz del proyecto (`..`)");
    }
    Ok(format!("/{}", p.trim_start_matches('/')))
}

/// Longitud de Typst: número con unidad (`96pt`, `0.12em`, `50%`).
fn es_longitud(s: &str) -> bool {
    let n = s.trim_start_matches('-');
    let unidad = ["pt", "mm", "cm", "in", "em", "fr", "%"]
        .iter()
        .find(|u| n.ends_with(**u));
    let Some(u) = unidad else { return false };
    let num = &n[..n.len() - u.len()];
    !num.is_empty()
        && num.chars().all(|c| c.is_ascii_digit() || c == '.')
        && num.matches('.').count() <= 1
        && !num.starts_with('.')
        && !num.ends_with('.')
}

/// Palabra clave o alineación de Typst: `center`, `auto`, `none`, `top + left`.
fn es_clave(s: &str) -> bool {
    !s.is_empty() && s.split(" + ").all(is_ident)
}

/// Tablas de un solo campo con significado propio: `{ md = "…" }` (formato),
/// `{ txt = "…" }` (texto plano como contenido),
/// `{ color = "#RRGGBB" }`, `{ image = "ruta" }` (imagen de fondo a sangre),
/// `{ len = "96pt" }` (longitud), `{ kw = "center" }` (palabra clave: `auto`,
/// `none`, alineaciones o el nombre de una variable de `defs`) y
/// `{ typst = "3pt + rgb(\"#fff\")" }` (expresión tal cual: no se escapa nada,
/// úsese solo con TOML propio). TOML no tiene unidades ni identificadores sueltos.
fn tagged_lit(t: &toml::Table) -> Result<Option<String>> {
    if t.len() != 1 {
        return Ok(None);
    }
    let (k, v) = t.iter().next().expect("len == 1");
    let s = match (k.as_str(), v) {
        ("md" | "txt" | "color" | "image" | "len" | "kw" | "typst", toml::Value::String(s)) => s,
        ("md" | "txt" | "color" | "image" | "len" | "kw" | "typst", _) => {
            bail!("`{k}` espera un string")
        }
        _ => return Ok(None),
    };
    Ok(Some(match k.as_str() {
        "md" => markup_lit(s),
        "txt" => txt_lit(s),
        "len" => {
            if !es_longitud(s) {
                bail!("longitud no válida: {s:?} (ej. \"96pt\", \"0.12em\", \"50%\")");
            }
            s.clone()
        }
        "typst" => {
            if s.trim().is_empty() {
                bail!("`typst` está vacío");
            }
            format!("({s})")
        }
        "kw" => {
            if !es_clave(s) {
                bail!("palabra clave no válida: {s:?} (ej. \"center\", \"auto\", \"top + left\")");
            }
            s.clone()
        }
        "color" => {
            let h = s.strip_prefix('#').unwrap_or("");
            if !(h.len() == 6 || h.len() == 8) || !h.chars().all(|c| c.is_ascii_hexdigit()) {
                bail!("color no válido: {s:?} (usa #RRGGBB)");
            }
            format!("rgb({})", str_lit(s))
        }
        _ => format!(
            "image({}, width: 100%, height: 100%, fit: \"cover\")",
            str_lit(&asset_path(s)?)
        ),
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn escapa_barra_y_comillas() {
        assert_eq!(str_lit(r"Ruta C:\"), r#""Ruta C:\\""#);
        assert_eq!(str_lit(r#"Tom "T""#), r#""Tom \"T\"""#);
        assert_eq!(str_lit(r#"a\" "#), r#""a\\\" ""#);
    }

    #[test]
    fn escapa_control_y_deja_unicode() {
        assert_eq!(str_lit("a\nb\tc"), r#""a\nb\tc""#);
        assert_eq!(str_lit("\u{0}"), r#""\u{0}""#);
        assert_eq!(str_lit("¿Qué? — ñ #x $y"), "\"¿Qué? — ñ #x $y\"");
    }

    #[test]
    fn markup_no_se_sale_del_string() {
        let m = markup_lit(r#"x"] #panic("y")"#);
        assert_eq!(
            m,
            r#"eval("x\"] #panic(\"y\")", mode: "markup", scope: _alcance)"#
        );
    }

    #[test]
    fn identificadores() {
        assert!(is_ident("font-body") && is_ident("_x") && is_ident("frase"));
        assert!(!is_ident("") && !is_ident("1a") && !is_ident("a b") && !is_ident("a:b"));
    }

    #[test]
    fn valores_toml() {
        let t: toml::Table = toml::from_str(
            r#"
            frase = "hola \"x\""
            n = 3
            r = 1.0
            ok = true
            lista = ["a"]
            vacia = []
            [caja]
            ancho = 2
            "#,
        )
        .unwrap();
        let s = named_args(&t).unwrap();
        assert!(s.contains(r#"frase: "hola \"x\"""#));
        assert!(s.contains("n: 3") && s.contains("r: 1.0") && s.contains("ok: true"));
        assert!(s.contains(r#"lista: ("a",)"#) && s.contains("vacia: ()"));
        assert!(s.contains("caja: (ancho: 2)"));
    }

    #[test]
    fn rechaza_claves_y_fechas_invalidas() {
        let t: toml::Table = toml::from_str(r#""a b" = 1"#).unwrap();
        assert!(named_args(&t).is_err());
        let t: toml::Table = toml::from_str("d = 2026-11-11").unwrap();
        assert!(named_args(&t).is_err());
    }

    #[test]
    fn sufijo_md_y_tablas_etiquetadas() {
        let t: toml::Table = toml::from_str(
            r##"
            frase_md = "*hola*"
            fondo = { color = "#691322" }
            foto = { image = "assets/a.jpg" }
            pie = { md = "x" }
            "##,
        )
        .unwrap();
        let s = named_args(&t).unwrap();
        assert!(s.contains(r#"frase: eval("*hola*", mode: "markup", scope: _alcance)"#));
        assert!(s.contains(r##"fondo: rgb("#691322")"##));
        assert!(
            s.contains(r#"foto: image("/assets/a.jpg", width: 100%, height: 100%, fit: "cover")"#)
        );
        assert!(s.contains(r#"pie: eval("x", mode: "markup", scope: _alcance)"#));
    }

    #[test]
    fn color_y_ruta_invalidos() {
        let t: toml::Table = toml::from_str(r#"c = { color = "rojo" }"#).unwrap();
        assert!(named_args(&t).is_err());
        let t: toml::Table = toml::from_str(r#"i = { image = "../x.png" }"#).unwrap();
        assert!(named_args(&t).is_err());
        assert_eq!(asset_path("/a/b.png").unwrap(), "/a/b.png");
    }

    #[test]
    fn longitudes_y_claves() {
        let t: toml::Table = toml::from_str(
            r#"
            margen = { len = "96pt" }
            tracking = { len = "0.12em" }
            neg = { len = "-3pt" }
            alinear = { kw = "center" }
            ancla = { kw = "top + left" }
            sin = { kw = "none" }
            "#,
        )
        .unwrap();
        let s = named_args(&t).unwrap();
        assert!(
            s.contains("margen: 96pt") && s.contains("tracking: 0.12em") && s.contains("neg: -3pt")
        );
        assert!(
            s.contains("alinear: center")
                && s.contains("ancla: top + left")
                && s.contains("sin: none")
        );
    }

    #[test]
    fn longitudes_y_claves_invalidas() {
        for bad in ["96", "pt", "1.2.3pt", ".5pt", "5pt)", "9 pt", "rgb(1)"] {
            let t: toml::Table = toml::from_str(&format!("x = {{ len = {bad:?} }}")).unwrap();
            assert!(named_args(&t).is_err(), "{bad:?}");
        }
        for bad in ["", "a b", "read(\"x\")", "a +", "1a"] {
            let t: toml::Table = toml::from_str(&format!("x = {{ kw = {bad:?} }}")).unwrap();
            assert!(named_args(&t).is_err(), "{bad:?}");
        }
    }

    #[test]
    fn expresion_cruda() {
        let t: toml::Table = toml::from_str(r##"c = { typst = '3pt + rgb("#fff")' }"##).unwrap();
        assert_eq!(named_args(&t).unwrap(), r##"c: (3pt + rgb("#fff"))"##);
        let t: toml::Table = toml::from_str("c = { typst = ' ' }").unwrap();
        assert!(named_args(&t).is_err());
    }

    /// `txt_lit` debe devolver el texto original al evaluarlo en Typst. Usa el
    /// `typst` del sistema; si no está instalado, no hay nada que comprobar.
    #[test]
    fn txt_lit_sale_tal_cual_en_typst_real() {
        if std::process::Command::new("typst")
            .arg("--version")
            .output()
            .is_err()
        {
            return;
        }
        let casos = [
            "a # b",
            "*x* _y_ ```z```",
            "$x$ y",
            "@ref <lab> y [a] ~ z",
            "\\ barra \\\\ doble",
            "#panic(1) y #read(x) y #{1+1}",
            "= no es título",
            "- no es lista",
            "+ no es enumeración",
            "/ no: término",
            "1. no es numeración",
            "12. tampoco",
            "texto normal, con ñ, acentos y ¿qué?",
            "palabras-con-guion y a - b",
        ];
        let mut doc = String::from(
            "#let flat(c) = {\n  if c.has(\"text\") { c.text }\n  else if c.func() == [ ].func() { \" \" }\n  \
             else if c.has(\"children\") { c.children.map(flat).join() }\n  else if c.has(\"body\") { flat(c.body) }\n  \
             else { \"<\" + repr(c.func()) + \">\" }\n}\n#let _alcance = (:)\n",
        );
        for c in casos {
            doc.push_str(&format!(
                "#assert.eq(flat({}), {}, message: {})\n",
                txt_lit(c),
                str_lit(c),
                str_lit(c)
            ));
        }
        let dir = std::env::temp_dir().join(format!("rrss-txt-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let typ = dir.join("t.typ");
        std::fs::write(&typ, doc).unwrap();
        let out = std::process::Command::new("typst")
            .args(["compile", "--root"])
            .arg(&dir)
            .arg(&typ)
            .arg(dir.join("t.pdf"))
            .output()
            .unwrap();
        let _ = std::fs::remove_dir_all(&dir);
        assert!(
            out.status.success(),
            "{}",
            String::from_utf8_lossy(&out.stderr)
        );
    }

    #[test]
    fn escape_de_marcadores_solo_al_inicio_de_linea() {
        assert_eq!(escape_markup("a - b"), "a - b");
        assert_eq!(escape_markup("- b"), "\\- b");
        assert_eq!(escape_markup("x\n= h"), "x\n\\= h");
        assert_eq!(escape_markup("3. v"), "3\\. v");
        assert_eq!(escape_markup("3.5 v"), "3.5 v");
        assert_eq!(escape_markup("#a *b* _c_"), "\\#a \\*b\\* \\_c\\_");
    }

    #[test]
    fn sufijo_y_etiqueta_txt() {
        let t: toml::Table = toml::from_str(
            r#"
            etiqueta_txt = "A # B"
            pie = { txt = "x" }
            "#,
        )
        .unwrap();
        let s = named_args(&t).unwrap();
        assert!(
            s.contains(r##"etiqueta: eval("A \\# B", mode: "markup", scope: _alcance)"##),
            "{s}"
        );
        assert!(
            s.contains(r#"pie: eval("x", mode: "markup", scope: _alcance)"#),
            "{s}"
        );
    }
}
