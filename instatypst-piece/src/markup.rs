//! Contenido en sintaxis Typst → bloques del contrato.
//!
//! El autor escribe Typst (títulos, listas, un puñado de funciones de rol) y
//! este módulo lo convierte en la misma lista de `Block` que se puede escribir
//! a mano. Es a la vez el validador de la sintaxis: todo lo que no esté en la
//! lista cerrada de abajo es un error con su línea y columna. Así el contenido
//! nunca ejecuta código arbitrario (`#read`, `#import`, `#let`, `{ … }`…).
//!
//! Sintaxis permitida:
//!
//! | Escribes                                   | Bloque                |
//! | :----------------------------------------- | :-------------------- |
//! | `= texto`                                  | `title`               |
//! | `== texto`                                 | `subtitle`            |
//! | un párrafo                                 | `body`                |
//! | `- a` (seguidos)                           | `list`                |
//! | `+ a` (seguidos)                           | `list` (numerada)     |
//! | `#label[..]` `#conclusion[..]` `#footer[..]` `#signature[..]` | el del mismo nombre |
//! | `#epigraph[..]` / `#epigraph(source: "x")[..]` | `epigraph` (con su fuente) |
//! | `#quote(attribution: "x")[..]`             | `quote` (+ `attribution`) |
//! | `#verse[a \ b]`                            | `verse`               |
//! | `#number(3)` / `#number(3, total: 8)`      | `number`              |
//! | `#date(day: 12, month: "NOV")`             | `date`                |
//! | `#attribution(author:, role:, work:, date:)` | `attribution`       |
//!
//! Dentro de un texto solo hay énfasis (`_x_`, `*x*`), saltos de línea y
//! caracteres escapados.

use typst_syntax::{ast, LinkedNode, SyntaxKind};

use crate::model::{Block, Issue, Item, ListOrder};

/// Funciones de rol con un solo argumento de texto.
const FUNCIONES_TEXTO: &[&str] = &[
    "label",
    "conclusion",
    "footer",
    "signature",
    "epigraph",
    "quote",
    "verse",
];

const FUNCIONES: &[&str] = &[
    "label",
    "conclusion",
    "footer",
    "signature",
    "epigraph",
    "quote",
    "verse",
    "number",
    "date",
    "attribution",
];

/// El inverso de `parse`: la lista de bloques escrita como Typst, de modo que
/// `parse(render(b)) == b`. No todo bloque tiene forma en Typst (el contrato lo dice: la lista con
/// etiquetas por ítem, `contrast`…, solo se escriben como bloques): en ese caso el error nombra el
/// bloque (`@N`, 1-based) y por qué. Pasar de bloques a texto es posible mientras todos quepan.
pub fn render(blocks: &[Block]) -> Result<String, String> {
    let mut partes: Vec<String> = Vec::new();
    let mut i = 0;
    while i < blocks.len() {
        let n = i + 1;
        let b = &blocks[i];
        let no = |why: &str| Err(format!("el bloque {n} (`{}`) no se puede escribir en Typst: {why}", b.kind()));
        let linea = |t: &str| -> Result<(), String> {
            if t.contains('\n') {
                return Err(format!("el bloque {n} (`{}`): el texto tiene un salto de línea", b.kind()));
            }
            Ok(())
        };
        // Un texto de bloque va tal cual (ya es Typst en línea): no puede partirse en párrafos,
        // ni empezar con algo que Typst lea como título o lista, ni desequilibrar los corchetes.
        let texto = |t: &str, entre_corchetes: bool| -> Result<(), String> {
            if t.trim().is_empty() {
                return Err(format!("el bloque {n} (`{}`) tiene el texto vacío", b.kind()));
            }
            if t.contains("\n\n") || t.lines().any(|l| l.trim().is_empty()) {
                return Err(format!("el bloque {n} (`{}`): una línea en blanco lo partiría en dos párrafos", b.kind()));
            }
            if !entre_corchetes {
                let c = t.trim_start().chars().next().unwrap_or(' ');
                if matches!(c, '=' | '-' | '+' | '#' | '/') {
                    return Err(format!("el bloque {n} (`{}`): empieza con `{c}`, que Typst toma por marcado", b.kind()));
                }
            }
            // Una `@` suelta es una referencia de Typst: se escribe `\@usuario`.
            let mut previo = ' ';
            for ch in t.chars() {
                if ch == '@' && previo != '\\' {
                    return Err(format!("el bloque {n} (`{}`): una `@` sin escapar; escríbela `\\@`", b.kind()));
                }
                previo = ch;
            }
            let (mut abre, mut cierra) = (0i32, 0i32);
            for ch in t.chars() {
                abre += (ch == '[') as i32;
                cierra += (ch == ']') as i32;
            }
            if abre != cierra {
                return Err(format!("el bloque {n} (`{}`): los corchetes del texto no están equilibrados", b.kind()));
            }
            Ok(())
        };
        match b {
            Block::Title { text } => {
                linea(text)?;
                texto(text, true)?;
                partes.push(format!("= {}", text.trim()));
            }
            Block::Subtitle { text } => {
                linea(text)?;
                texto(text, true)?;
                partes.push(format!("== {}", text.trim()));
            }
            Block::Body { text } => {
                texto(text, false)?;
                partes.push(text.trim().to_string());
            }
            Block::Label { text } | Block::Conclusion { text } | Block::Footer { text } | Block::Signature { text } => {
                texto(text, true)?;
                partes.push(format!("#{}[{}]", b.kind(), text.trim()));
            }
            Block::Epigraph { text, source } => {
                texto(text, true)?;
                partes.push(match source {
                    Some(s) => format!("#epigraph(source: {})[{}]", cadena(s), text.trim()),
                    None => format!("#epigraph[{}]", text.trim()),
                });
            }
            Block::Quote { text } => {
                texto(text, true)?;
                // Una atribución solo de autor, justo después, es el `attribution:` de la cita.
                let autor = match blocks.get(i + 1) {
                    Some(Block::Attribution { author: Some(a), role: None, work: None, date: None }) => Some(a),
                    _ => None,
                };
                match autor {
                    Some(a) => {
                        partes.push(format!("#quote(attribution: {})[{}]", cadena(a), text.trim()));
                        i += 1;
                    }
                    None => partes.push(format!("#quote[{}]", text.trim())),
                }
            }
            Block::Verse { lines, role } => {
                if role.is_some() {
                    return no("el verso con `role` solo se escribe como bloque");
                }
                if lines.is_empty() || lines.iter().any(|l| l.trim().is_empty() || l.contains('\n') || l.contains('\\')) {
                    return no("las líneas del verso no pueden estar vacías ni llevar saltos");
                }
                for l in lines {
                    texto(l, true)?;
                }
                partes.push(format!("#verse[{}]", lines.iter().map(|l| l.trim()).collect::<Vec<_>>().join(" \\ ")));
            }
            Block::Attribution { author, role, work, date } => {
                let campos: Vec<String> = [("author", author), ("role", role), ("work", work), ("date", date)]
                    .into_iter()
                    .filter_map(|(k, v)| v.as_ref().map(|v| format!("{k}: {}", cadena(v))))
                    .collect();
                partes.push(format!("#attribution({})", campos.join(", ")));
            }
            Block::List { order, items } => {
                use crate::model::ListOrder;
                let marca = match order {
                    ListOrder::None => "-",
                    ListOrder::Numbered => "+",
                    ListOrder::Chronological => return no("el orden `chronological` solo se escribe como bloque"),
                };
                let mut lineas = Vec::new();
                for it in items {
                    match it {
                        Item::Text(t) => {
                            linea(t)?;
                            texto(t, true)?;
                            lineas.push(format!("{marca} {}", t.trim()));
                        }
                        Item::Full(_) => return no("los ítems con etiqueta o rol solo se escriben como bloque"),
                    }
                }
                if lineas.is_empty() {
                    return no("la lista está vacía");
                }
                // Dos listas seguidas del mismo tipo se leerían como una sola.
                if let Some(Block::List { order: o2, .. }) = blocks.get(i + 1) {
                    if o2 == order {
                        return no("la lista que sigue es del mismo tipo y se leerían como una sola");
                    }
                }
                partes.push(lineas.join("\n"));
            }
            Block::Contrast { .. } => return no("`contrast` solo se escribe como bloque"),
            Block::Number { value, total } => partes.push(match total {
                Some(t) => format!("#number({value}, total: {t})"),
                None => format!("#number({value})"),
            }),
            Block::Date { day, month } => partes.push(format!("#date(day: {day}, month: {})", cadena(month))),
        }
        i += 1;
    }
    Ok(partes.join("\n\n"))
}

/// Una cadena de Typst `"…"`.
fn cadena(s: &str) -> String {
    let mut o = String::from("\"");
    for c in s.chars() {
        match c {
            '"' => o.push_str("\\\""),
            '\\' => o.push_str("\\\\"),
            '\n' => o.push_str("\\n"),
            c => o.push(c),
        }
    }
    o.push('"');
    o
}

/// Resultado: los bloques que se pudieron derivar y las incidencias. Si hay
/// incidencias, los bloques no son fiables.
pub fn parse(src: &str) -> (Vec<Block>, Vec<Issue>) {
    let mut c = Conv {
        src,
        blocks: Vec::new(),
        issues: Vec::new(),
        lista_abierta: false,
        parrafo: None,
    };
    let raiz = typst_syntax::parse(src);
    let nodo = LinkedNode::new(&raiz);

    c.errores_de_sintaxis(&nodo);
    if !c.issues.is_empty() {
        return (Vec::new(), c.issues);
    }

    for hijo in nodo.children() {
        c.top(&hijo);
    }
    c.cerrar_parrafo();
    (c.blocks, c.issues)
}

struct Conv<'a> {
    src: &'a str,
    blocks: Vec<Block>,
    issues: Vec<Issue>,
    /// El último bloque emitido es una lista que el siguiente ítem puede seguir.
    lista_abierta: bool,
    /// Rango en bytes del párrafo en curso.
    parrafo: Option<(usize, usize)>,
}

impl<'a> Conv<'a> {
    fn pos(&self, off: usize) -> String {
        let antes = &self.src[..off.min(self.src.len())];
        let linea = antes.matches('\n').count() + 1;
        let col = antes.rsplit('\n').next().map_or(0, |l| l.chars().count()) + 1;
        format!("@{linea}:{col}")
    }

    fn error(&mut self, off: usize, code: &'static str, message: String) {
        let path = self.pos(off);
        self.issues.push(Issue {
            path,
            code,
            message,
        });
    }

    fn errores_de_sintaxis(&mut self, n: &LinkedNode) {
        if n.kind() == SyntaxKind::Error {
            let msg = n
                .errors_and_warnings()
                .0
                .first()
                .map(|e| e.message.to_string())
                .unwrap_or_else(|| "sintaxis no válida".into());
            self.error(n.offset(), "markup_syntax", msg);
            return;
        }
        for h in n.children() {
            self.errores_de_sintaxis(&h);
        }
    }

    fn prohibido(&mut self, n: &LinkedNode, que: &str) {
        self.error(
            n.offset(),
            "markup_forbidden",
            format!(
                "{que} no está permitido en el contenido. Se admite: texto con _énfasis_ y *negrita*, \
                 `= título`, `== subtítulo`, listas (`-`, `+`) y las funciones {}",
                FUNCIONES.iter().map(|f| format!("#{f}")).collect::<Vec<_>>().join(", ")
            ),
        );
    }

    // ── nivel superior ──────────────────────────────────────

    fn top(&mut self, n: &LinkedNode) {
        match n.kind() {
            SyntaxKind::Space => {}
            SyntaxKind::Parbreak => self.cerrar_parrafo(),
            SyntaxKind::Text
            | SyntaxKind::Linebreak
            | SyntaxKind::Escape
            | SyntaxKind::Shorthand
            | SyntaxKind::SmartQuote
            | SyntaxKind::Strong
            | SyntaxKind::Emph => {
                self.inline(n);
                let r = n.range();
                self.parrafo = Some(match self.parrafo {
                    Some((a, _)) => (a, r.end),
                    None => (r.start, r.end),
                });
            }
            SyntaxKind::Heading => {
                self.cerrar_parrafo();
                self.titulo(n);
            }
            SyntaxKind::ListItem | SyntaxKind::EnumItem => {
                self.cerrar_parrafo();
                self.item(n);
            }
            SyntaxKind::Ref | SyntaxKind::RefMarker => self.inline(n), // explica cómo escapar la arroba
            SyntaxKind::Hash => {} // el `#`; lo que sigue se examina aparte
            SyntaxKind::FuncCall => {
                self.cerrar_parrafo();
                self.llamada(n);
            }
            SyntaxKind::TermItem => self.prohibido(
                n,
                "una lista de términos (`/ término: …`; aún no tiene bloque)",
            ),
            otro => self.prohibido(n, &format!("`{otro:?}`")),
        }
    }

    fn cerrar_parrafo(&mut self) {
        if let Some((a, b)) = self.parrafo.take() {
            let texto = self.src[a..b].trim().to_string();
            if !texto.is_empty() {
                self.blocks.push(Block::Body { text: texto });
                self.lista_abierta = false;
            }
        }
    }

    // ── texto en línea ──────────────────────────────────────

    /// Solo énfasis, saltos de línea y caracteres escapados.
    fn inline(&mut self, n: &LinkedNode) {
        match n.kind() {
            SyntaxKind::Text
            | SyntaxKind::Space
            | SyntaxKind::Linebreak
            | SyntaxKind::Escape
            | SyntaxKind::Shorthand
            | SyntaxKind::SmartQuote
            | SyntaxKind::Star
            | SyntaxKind::Underscore => {}
            SyntaxKind::Strong | SyntaxKind::Emph | SyntaxKind::Markup => {
                for h in n.children() {
                    self.inline(&h);
                }
            }
            SyntaxKind::Ref | SyntaxKind::RefMarker => self.error(
                n.offset(),
                "markup_forbidden",
                "`@…` es una referencia de Typst; para escribir un usuario, escapa la arroba: `\\@usuario`"
                    .into(),
            ),
            otro => self.prohibido(n, &format!("`{otro:?}` dentro de un texto")),
        }
    }

    /// El texto (con su marcado en línea) de un nodo con cuerpo `Markup`.
    fn cuerpo(&mut self, n: &LinkedNode) -> Option<String> {
        let markup = n.children().find(|h| h.kind() == SyntaxKind::Markup)?;
        self.inline(&markup);
        let r = markup.range();
        Some(self.src[r].trim().to_string())
    }

    // ── títulos y listas ────────────────────────────────────

    fn titulo(&mut self, n: &LinkedNode) {
        let Some(h) = n.cast::<ast::Heading>() else {
            return;
        };
        let nivel = h.depth().get();
        let Some(texto) = self.cuerpo(n) else { return };
        self.lista_abierta = false;
        match nivel {
            1 => self.blocks.push(Block::Title { text: texto }),
            2 => self.blocks.push(Block::Subtitle { text: texto }),
            _ => self.error(
                n.offset(),
                "heading_level",
                format!("título de nivel {nivel}: solo `=` (title) y `==` (subtitle)"),
            ),
        }
    }

    fn item(&mut self, n: &LinkedNode) {
        let orden = if n.kind() == SyntaxKind::EnumItem {
            ListOrder::Numbered
        } else {
            ListOrder::None
        };
        let Some(texto) = self.cuerpo(n) else { return };
        if self.lista_abierta {
            if let Some(Block::List { order, items }) = self.blocks.last_mut() {
                if *order == orden {
                    items.push(Item::Text(texto));
                    return;
                }
            }
        }
        self.blocks.push(Block::List {
            order: orden,
            items: vec![Item::Text(texto)],
        });
        self.lista_abierta = true;
    }

    // ── funciones de rol ────────────────────────────────────

    fn llamada(&mut self, n: &LinkedNode) {
        self.lista_abierta = false;
        let n = &n.clone();
        let Some(f) = n.cast::<ast::FuncCall>() else {
            return;
        };
        let nombre = match f.callee() {
            ast::Expr::Ident(i) => i.get().to_string(),
            _ => {
                self.prohibido(n, "esta llamada");
                return;
            }
        };
        if !FUNCIONES.contains(&nombre.as_str()) {
            self.error(
                n.offset().saturating_sub(1),
                "markup_function",
                format!(
                    "#{nombre} no está permitida; las funciones válidas: {}",
                    FUNCIONES
                        .iter()
                        .map(|f| format!("#{f}"))
                        .collect::<Vec<_>>()
                        .join(", ")
                ),
            );
            return;
        }

        let Some(args) = n.children().find(|h| h.kind() == SyntaxKind::Args) else {
            self.error(
                n.offset().saturating_sub(1),
                "markup_args",
                format!("#{nombre} necesita argumentos"),
            );
            return;
        };

        // Posicionales y con nombre, tal como aparecen.
        let mut posicionales: Vec<LinkedNode> = Vec::new();
        let mut con_nombre: Vec<(String, LinkedNode)> = Vec::new();
        for h in args.children() {
            match h.kind() {
                SyntaxKind::Named => {
                    let clave = h
                        .children()
                        .find(|x| x.kind() == SyntaxKind::Ident)
                        .map(|x| x.get().leaf_text().to_string())
                        .unwrap_or_default();
                    if let Some(v) = h.children().next_back() {
                        con_nombre.push((clave, v));
                    }
                }
                SyntaxKind::Str | SyntaxKind::Int | SyntaxKind::ContentBlock => {
                    posicionales.push(h)
                }
                SyntaxKind::LeftParen
                | SyntaxKind::RightParen
                | SyntaxKind::Comma
                | SyntaxKind::Space => {}
                otro => self.prohibido(&h, &format!("`{otro:?}` como argumento de #{nombre}")),
            }
        }

        match nombre.as_str() {
            "number" => self.numero(n, &posicionales, &con_nombre),
            "date" => self.fecha(n, &posicionales, &con_nombre),
            "attribution" => self.atribucion(n, &posicionales, &con_nombre),
            otro if FUNCIONES_TEXTO.contains(&otro) => {
                self.rol_de_texto(n, otro, &posicionales, &con_nombre)
            }
            _ => {}
        }
    }

    fn rol_de_texto(
        &mut self,
        n: &LinkedNode,
        nombre: &str,
        pos: &[LinkedNode],
        nom: &[(String, LinkedNode)],
    ) {
        let [arg] = pos else {
            self.error(
                n.offset().saturating_sub(1),
                "markup_args",
                format!("#{nombre} lleva un solo texto: `#{nombre}[…]`"),
            );
            return;
        };
        let atribucion = nom.iter().find(|(k, _)| k == "attribution");
        for (k, v) in nom {
            // `quote` lleva su atribución; `epigraph`, su fuente (de quién es el epígrafe)
            let permitido = (nombre == "quote" && k == "attribution")
                || (nombre == "epigraph" && k == "source");
            if !permitido {
                self.error(
                    v.offset(),
                    "markup_args",
                    format!("#{nombre} no tiene el argumento `{k}`"),
                );
            }
        }

        if nombre == "verse" {
            let Some(lineas) = self.lineas(arg) else {
                return;
            };
            self.blocks.push(Block::Verse {
                lines: lineas,
                role: None,
            });
            return;
        }
        let Some(texto) = self.texto_de(arg) else {
            return;
        };
        let fuente = nom
            .iter()
            .find(|(k, _)| k == "source")
            .and_then(|(_, v)| self.texto_de(v));
        let bloque = match nombre {
            "label" => Block::Label { text: texto },
            "conclusion" => Block::Conclusion { text: texto },
            "footer" => Block::Footer { text: texto },
            "signature" => Block::Signature { text: texto },
            "epigraph" => Block::Epigraph {
                text: texto,
                source: fuente,
            },
            _ => Block::Quote { text: texto },
        };
        self.blocks.push(bloque);
        if let Some((_, v)) = atribucion {
            if let Some(autor) = self.texto_de(v) {
                self.blocks.push(Block::Attribution {
                    author: Some(autor),
                    role: None,
                    work: None,
                    date: None,
                });
            }
        }
    }

    fn numero(&mut self, n: &LinkedNode, pos: &[LinkedNode], nom: &[(String, LinkedNode)]) {
        let inicio = n.offset().saturating_sub(1);
        if pos.len() != 1 || pos[0].kind() != SyntaxKind::Int {
            self.error(
                inicio,
                "markup_args",
                "#number lleva un entero: `#number(3)` o `#number(3, total: 8)`".into(),
            );
            return;
        }
        let Ok(value) = pos[0].get().leaf_text().parse::<u32>() else {
            self.error(
                pos[0].offset(),
                "markup_args",
                "#number necesita un entero no negativo".into(),
            );
            return;
        };
        let mut total = None;
        for (k, v) in nom {
            if k != "total" {
                self.error(
                    v.offset(),
                    "markup_args",
                    format!("#number no tiene el argumento `{k}`; usa total"),
                );
            } else if v.kind() == SyntaxKind::Int {
                total = v.get().leaf_text().parse::<u32>().ok();
            } else {
                self.error(v.offset(), "markup_args", "`total` es un entero".into());
            }
        }
        self.blocks.push(Block::Number { value, total });
    }

    fn fecha(&mut self, n: &LinkedNode, pos: &[LinkedNode], nom: &[(String, LinkedNode)]) {
        let inicio = n.offset().saturating_sub(1);
        if !pos.is_empty() {
            self.error(
                inicio,
                "markup_args",
                "#date solo lleva argumentos con nombre: day, month".into(),
            );
            return;
        }
        let (mut day, mut month) = (None, None);
        for (k, v) in nom {
            match k.as_str() {
                "day" if v.kind() == SyntaxKind::Int => {
                    day = v.get().leaf_text().parse::<u8>().ok()
                }
                "day" => self.error(
                    v.offset(),
                    "markup_args",
                    "`day` es un entero de 1 a 31".into(),
                ),
                "month" => month = self.texto_de(v),
                otro => self.error(
                    v.offset(),
                    "markup_args",
                    format!("#date no tiene el argumento `{otro}`; usa day y month"),
                ),
            }
        }
        match (day, month) {
            (Some(day), Some(month)) => self.blocks.push(Block::Date { day, month }),
            _ => self.error(
                inicio,
                "markup_args",
                "#date necesita day y month: `#date(day: 12, month: \"NOV\")`".into(),
            ),
        }
    }

    fn atribucion(&mut self, n: &LinkedNode, pos: &[LinkedNode], nom: &[(String, LinkedNode)]) {
        if !pos.is_empty() {
            self.error(
                n.offset().saturating_sub(1),
                "markup_args",
                "#attribution solo lleva argumentos con nombre: author, role, work, date".into(),
            );
            return;
        }
        let (mut author, mut role, mut work, mut date) = (None, None, None, None);
        for (k, v) in nom {
            let Some(t) = self.texto_de(v) else { continue };
            match k.as_str() {
                "author" => author = Some(t),
                "role" => role = Some(t),
                "work" => work = Some(t),
                "date" => date = Some(t),
                otro => self.error(
                    v.offset(),
                    "markup_args",
                    format!("#attribution no tiene el argumento `{otro}`; usa author, role, work o date"),
                ),
            }
        }
        if author.is_none() && work.is_none() && date.is_none() {
            self.error(
                n.offset().saturating_sub(1),
                "markup_args",
                "#attribution necesita author, work o date".into(),
            );
            return;
        }
        self.blocks.push(Block::Attribution {
            author,
            role,
            work,
            date,
        });
    }

    /// Una cadena `"x"` o un contenido `[x]`, como texto.
    fn texto_de(&mut self, n: &LinkedNode) -> Option<String> {
        match n.kind() {
            SyntaxKind::Str => n.cast::<ast::Str>().map(|s| s.get().to_string()),
            SyntaxKind::ContentBlock => self.cuerpo(n),
            otro => {
                self.prohibido(n, &format!("`{otro:?}` como texto"));
                None
            }
        }
    }

    /// Las líneas de un verso: se cortan en `\` y en cada salto de línea.
    fn lineas(&mut self, n: &LinkedNode) -> Option<Vec<String>> {
        if n.kind() != SyntaxKind::ContentBlock {
            self.prohibido(n, "un verso que no es contenido `[…]`");
            return None;
        }
        let markup = n.children().find(|h| h.kind() == SyntaxKind::Markup)?;
        self.inline(&markup);
        let mut lineas = Vec::new();
        let mut actual: Option<(usize, usize)> = None;
        let cortar = |actual: &mut Option<(usize, usize)>, src: &str, v: &mut Vec<String>| {
            if let Some((a, b)) = actual.take() {
                let t = src[a..b].trim();
                if !t.is_empty() {
                    v.push(t.to_string());
                }
            }
        };
        for h in markup.children() {
            let r = h.range();
            match h.kind() {
                SyntaxKind::Linebreak | SyntaxKind::Parbreak => {
                    cortar(&mut actual, self.src, &mut lineas)
                }
                SyntaxKind::Space if self.src[r.clone()].contains('\n') => {
                    cortar(&mut actual, self.src, &mut lineas)
                }
                SyntaxKind::Space => {}
                _ => actual = Some(actual.map_or((r.start, r.end), |(a, _)| (a, r.end))),
            }
        }
        cortar(&mut actual, self.src, &mut lineas);
        if lineas.is_empty() {
            self.error(n.offset(), "markup_args", "#verse sin líneas".into());
            return None;
        }
        Some(lineas)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ok(src: &str) -> Vec<Block> {
        let (b, i) = parse(src);
        assert!(i.is_empty(), "{src:?} → {i:#?}");
        b
    }

    fn falla(src: &str) -> Vec<Issue> {
        let (_, i) = parse(src);
        assert!(!i.is_empty(), "{src:?} debería fallar");
        i
    }

    #[test]
    fn titulo_subtitulo_y_parrafo() {
        let b = ok("= Uno\n== Dos\nTexto con _énfasis_.\n");
        assert_eq!(
            b,
            vec![
                Block::Title { text: "Uno".into() },
                Block::Subtitle { text: "Dos".into() },
                Block::Body {
                    text: "Texto con _énfasis_.".into()
                },
            ]
        );
    }

    #[test]
    fn dos_parrafos() {
        let b = ok("Uno.\n\nDos.");
        assert_eq!(b.len(), 2);
    }

    #[test]
    fn funciones_de_rol() {
        let b = ok("#label[ETIQUETA]\n#conclusion[PUREZA.]\n#footer[pie]\n#signature[\\@marca]");
        let tipos: Vec<_> = b.iter().map(|x| format!("{x:?}")).collect();
        assert!(tipos[0].starts_with("Label"), "{tipos:?}");
        assert!(tipos[1].starts_with("Conclusion"));
        assert!(tipos[2].starts_with("Footer"));
        assert!(tipos[3].starts_with("Signature"));
    }

    #[test]
    fn atribucion_y_numero() {
        let b = ok("#attribution(author: \"B. B. Warfield\", date: \"1851\")\n#number(3)");
        assert_eq!(
            b[0],
            Block::Attribution {
                author: Some("B. B. Warfield".into()),
                role: None,
                work: None,
                date: Some("1851".into())
            }
        );
        assert_eq!(
            b[1],
            Block::Number {
                value: 3,
                total: None
            }
        );
        let c = ok("#number(3, total: 8)\n#date(day: 12, month: \"NOV\")\n#attribution(author: \"Ana\", role: \"Pastora\")");
        assert_eq!(
            c[0],
            Block::Number {
                value: 3,
                total: Some(8)
            }
        );
        assert_eq!(
            c[1],
            Block::Date {
                day: 12,
                month: "NOV".into()
            }
        );
        assert!(matches!(&c[2], Block::Attribution { role: Some(r), .. } if r == "Pastora"));
    }

    #[test]
    fn cita_con_atribucion_da_dos_bloques() {
        let b = ok("#quote(attribution: [Warfield])[Todo es gracia.]");
        assert_eq!(
            b[0],
            Block::Quote {
                text: "Todo es gracia.".into()
            }
        );
        assert!(matches!(&b[1], Block::Attribution { author: Some(a), .. } if a == "Warfield"));
    }

    #[test]
    fn verso_se_corta_en_lineas() {
        let b = ok("#verse[Todo es gracia, \\ todo es don,\ntodo es Dios.]");
        assert_eq!(
            b[0],
            Block::Verse {
                lines: vec![
                    "Todo es gracia,".into(),
                    "todo es don,".into(),
                    "todo es Dios.".into()
                ],
                role: None
            }
        );
    }

    #[test]
    fn listas_seguidas_se_agrupan_y_las_distintas_no() {
        let b = ok("- a\n- b\n+ uno\n+ dos\n");
        assert_eq!(b.len(), 2);
        assert!(matches!(&b[0], Block::List { order: ListOrder::None, items } if items.len() == 2));
        assert!(
            matches!(&b[1], Block::List { order: ListOrder::Numbered, items } if items.len() == 2)
        );
    }

    // ── lo que debe rechazar ────────────────────────────────

    #[test]
    fn la_arroba_sin_escapar_explica_como_escribirla() {
        let i = falla("hola @usuario");
        assert!(i[0].message.contains("\\@usuario"), "{}", i[0].message);
        ok("hola \\@usuario");
    }

    #[test]
    fn rechaza_leer_archivos() {
        let i = falla("#read(\"/etc/passwd\")");
        assert_eq!(i[0].code, "markup_function");
        assert!(i[0].message.contains("read"));
    }

    #[test]
    fn rechaza_codigo() {
        for src in [
            "#let x = 1",
            "#import \"a.typ\": *",
            "#{ 1 + 1 }",
            "#set text(red)",
            "#show: it => it",
            "#eval(\"1\")",
            "#label[#read(\"x\")]",
            "#quote(attribution: read(\"x\"))[a]",
        ] {
            falla(src);
        }
    }

    #[test]
    fn rechaza_lo_que_no_es_texto() {
        for src in [
            "$x^2$",
            "`codigo`",
            "https://x.org",
            "@ref",
            "<etiqueta>",
            "/ Ley: x",
        ] {
            falla(src);
        }
    }

    #[test]
    fn rechaza_titulos_profundos_y_argumentos_de_mas() {
        assert_eq!(falla("=== tres")[0].code, "heading_level");
        assert_eq!(falla("#label[a][b]")[0].code, "markup_args");
        assert_eq!(falla("#number(\"3\")")[0].code, "markup_args");
        assert_eq!(falla("#attribution(\"x\")")[0].code, "markup_args");
        assert_eq!(falla("#attribution(autor: \"x\")")[0].code, "markup_args");
    }

    #[test]
    fn la_incidencia_lleva_linea_y_columna() {
        let i = falla("= Bien\n\n#read(\"x\")");
        assert_eq!(i[0].path, "@3:1");
    }

    #[test]
    fn los_errores_de_sintaxis_se_reportan() {
        assert_eq!(falla("#label[sin cerrar")[0].code, "markup_syntax");
    }
}

#[cfg(test)]
mod render_tests {
    use super::*;
    use crate::model::{Content, ItemFull, Role, Side, Syntax};

    fn ida_y_vuelta(b: Vec<Block>) -> String {
        let t = render(&b).unwrap_or_else(|e| panic!("{e}"));
        let (vuelta, issues) = parse(&t);
        assert!(issues.is_empty(), "{issues:?}\n{t}");
        assert_eq!(vuelta, b, "no vuelve igual:\n{t}");
        t
    }

    #[test]
    fn cada_bloque_con_forma_en_typst_vuelve_igual() {
        let t = ida_y_vuelta(vec![
            Block::Title { text: "La regla de los *90* minutos".into() },
            Block::Subtitle { text: "Cómo organizo _mi día_".into() },
            Block::Label { text: "ENFOQUE".into() },
            Block::Body { text: "Un párrafo\ncon dos líneas y \\@usuario".into() },
            Block::Epigraph { text: "Una cita".into(), source: Some("Alguien \"X\"".into()) },
            Block::Epigraph { text: "Sin fuente".into(), source: None },
            Block::Quote { text: "La cita".into() },
            Block::Attribution { author: Some("Autora".into()), role: None, work: None, date: None },
            Block::Verse { lines: vec!["uno".into(), "dos".into(), "tres".into()], role: None },
            Block::Attribution { author: Some("Yo".into()), role: Some("pastor".into()), work: Some("Obra".into()), date: Some("2026".into()) },
            Block::List { order: ListOrder::None, items: vec![Item::Text("a".into()), Item::Text("b".into())] },
            Block::List { order: ListOrder::Numbered, items: vec![Item::Text("uno".into())] },
            Block::Conclusion { text: "Fin".into() },
            Block::Number { value: 3, total: Some(8) },
            Block::Number { value: 1, total: None },
            Block::Date { day: 12, month: "NOV".into() },
            Block::Footer { text: "pie".into() },
            Block::Signature { text: "\\@yo".into() },
        ]);
        assert!(t.contains("#quote(attribution: \"Autora\")[La cita]"), "{t}");
    }

    #[test]
    fn lo_que_no_cabe_en_typst_dice_cual_bloque_y_por_que() {
        let item = Item::Full(ItemFull { text: "x".into(), label: Some("l".into()), role: Some(Role::Affirm) });
        for (b, frag) in [
            (vec![Block::Body { text: "ok".into() }, Block::Contrast { left: Side { label: None, title: None, items: vec![Item::Text("a".into())] }, right: Side { label: None, title: None, items: vec![] } }], "bloque 2"),
            (vec![Block::List { order: ListOrder::None, items: vec![item] }], "etiqueta o rol"),
            (vec![Block::List { order: ListOrder::Chronological, items: vec![Item::Text("a".into())] }], "chronological"),
            (vec![Block::Verse { lines: vec!["a".into()], role: Some("r".into()) }], "role"),
            (vec![Block::Body { text: "uno\n\ndos".into() }], "línea en blanco"),
            (vec![Block::Body { text: "= no soy título".into() }], "empieza con `=`"),
            (vec![Block::Label { text: "abre [ y no cierra".into() }], "corchetes"),
            (vec![Block::Title { text: "dos\nlíneas".into() }], "salto de línea"),
            (vec![Block::Signature { text: "@yo".into() }], "sin escapar"),
            (vec![Block::List { order: ListOrder::None, items: vec![Item::Text("a".into())] }, Block::List { order: ListOrder::None, items: vec![Item::Text("b".into())] }], "una sola"),
        ] {
            let e = render(&b).unwrap_err();
            assert!(e.contains(frag), "{e:?} no menciona {frag:?}");
        }
    }

    /// Sobre las piezas reales del contrato: lo que se escribe en Typst vuelve igual, y lo que no
    /// cabe falla con mensaje. Y partiendo de Typst, bloques → Typst → bloques es estable.
    #[test]
    fn las_piezas_de_las_fixtures_vuelven_igual_o_dicen_por_que_no() {
        let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../instatypst-cli/tests/fixtures/contrato");
        let (mut ok, mut no_cabe, mut desde_typst) = (0, 0, 0);
        for sub in ["valid", "real", "build"] {
            let Ok(rd) = std::fs::read_dir(dir.join(sub)) else { continue };
            for e in rd.flatten() {
                let ruta = e.path();
                if ruta.extension().is_none_or(|x| x != "toml") {
                    continue;
                }
                let Ok(doc) = crate::model::parse(&std::fs::read_to_string(&ruta).unwrap(), Syntax::Toml) else { continue };
                for post in &doc.posts {
                    match &post.content {
                        Some(Content::Blocks(b)) if !b.is_empty() => match render(b) {
                            Ok(t) => {
                                let (v, issues) = parse(&t);
                                assert!(issues.is_empty(), "{}: {issues:?}\n{t}", ruta.display());
                                assert_eq!(&v, b, "{}: no vuelve igual\n{t}", ruta.display());
                                ok += 1;
                            }
                            Err(m) => {
                                assert!(m.contains("bloque"), "{m}");
                                no_cabe += 1;
                            }
                        },
                        Some(Content::Markup(m)) => {
                            let (b, issues) = parse(m);
                            if !issues.is_empty() {
                                continue;
                            }
                            let t = render(&b).unwrap_or_else(|e| panic!("{}: {e}", ruta.display()));
                            assert_eq!(parse(&t).0, b, "{}: Typst → bloques → Typst → bloques no es estable", ruta.display());
                            desde_typst += 1;
                        }
                        _ => {}
                    }
                }
            }
        }
        assert!(ok + desde_typst >= 3, "pocas piezas comprobadas: {ok} bloques, {desde_typst} desde Typst");
        eprintln!("render: {ok} piezas de bloques vuelven igual, {no_cabe} no caben en Typst, {desde_typst} partiendo de Typst");
    }
}
