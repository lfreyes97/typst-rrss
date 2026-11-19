//! La geometría de una página, sacada del árbol de marcos del documento compilado: una caja por
//! cada texto, forma e imagen, en puntos y con el origen en la esquina superior izquierda.
//! Es lo que usa el modo debug (cajas, medidas, inspector).

use typst::layout::{Frame, FrameItem, Point, Transform};
use typst::visualize::{Geometry, Paint};
use typst_layout::PagedDocument;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BoxKind {
    Text,
    Shape,
    Image,
}

#[derive(Clone, Debug)]
pub struct BoxInfo {
    pub kind: BoxKind,
    /// `[x, y, ancho, alto]` en puntos.
    pub rect: [f32; 4],
    /// El texto, si es una caja de texto.
    pub text: Option<String>,
    pub font: Option<String>,
    /// Tamaño del cuerpo en puntos.
    pub size: Option<f32>,
    /// Color de relleno `[r, g, b, a]`, si es uno sólido.
    pub fill: Option<[u8; 4]>,
}

impl BoxInfo {
    pub fn area(&self) -> f32 {
        self.rect[2] * self.rect[3]
    }

    pub fn contains(&self, x: f32, y: f32) -> bool {
        x >= self.rect[0] && x <= self.rect[0] + self.rect[2] && y >= self.rect[1] && y <= self.rect[1] + self.rect[3]
    }
}

/// Las cajas de la página `page` (0-based), en orden de dibujo (las últimas quedan encima), y el
/// color de fondo de la página si es sólido.
pub fn page_boxes(doc: &PagedDocument, page: usize) -> Option<(Vec<BoxInfo>, Option<[u8; 3]>)> {
    let p = doc.pages().get(page)?;
    let mut out = Vec::new();
    walk(&p.frame, Transform::identity(), &mut out);
    let bg = match p.fill_or_white() {
        Some(Paint::Solid(c)) => {
            let [r, g, b, _] = c.to_vec4_u8();
            Some([r, g, b])
        }
        _ => None,
    };
    Some((out, bg))
}

fn bounds(ts: Transform, x: f64, y: f64, w: f64, h: f64) -> [f32; 4] {
    // Con una rotación la caja deja de ser recta: se toma la envolvente de las cuatro esquinas.
    let pts = [(x, y), (x + w, y), (x, y + h), (x + w, y + h)].map(|(px, py)| {
        let p = Point::new(typst::layout::Abs::pt(px), typst::layout::Abs::pt(py)).transform(ts);
        (p.x.to_pt(), p.y.to_pt())
    });
    let (x0, x1) = pts.iter().fold((f64::MAX, f64::MIN), |a, p| (a.0.min(p.0), a.1.max(p.0)));
    let (y0, y1) = pts.iter().fold((f64::MAX, f64::MIN), |a, p| (a.0.min(p.1), a.1.max(p.1)));
    [x0 as f32, y0 as f32, (x1 - x0) as f32, (y1 - y0) as f32]
}

fn solid(p: &Paint) -> Option<[u8; 4]> {
    match p {
        Paint::Solid(c) => Some(c.to_vec4_u8()),
        _ => None,
    }
}

fn walk(frame: &Frame, ts: Transform, out: &mut Vec<BoxInfo>) {
    for (pos, item) in frame.items() {
        match item {
            FrameItem::Group(g) => {
                let ts = ts.pre_concat(Transform::translate(pos.x, pos.y)).pre_concat(g.transform);
                walk(&g.frame, ts, out);
            }
            FrameItem::Text(t) => {
                let size = t.size;
                let width: f64 = t.glyphs.iter().map(|g| g.x_advance.at(size).to_pt()).sum();
                let m = t.font.metrics();
                let (asc, desc) = (m.ascender.at(size).to_pt(), m.descender.at(size).to_pt());
                let (x, y) = (pos.x.to_pt(), pos.y.to_pt());
                out.push(BoxInfo {
                    kind: BoxKind::Text,
                    rect: bounds(ts, x, y - asc, width, asc - desc),
                    text: Some(t.text.to_string()),
                    font: Some(t.font.info().family.clone()),
                    size: Some(size.to_pt() as f32),
                    fill: solid(&t.fill),
                });
            }
            FrameItem::Shape(s, _) => {
                // Las líneas no son una caja (no tienen área): se dejan fuera.
                if matches!(s.geometry, Geometry::Line(_)) {
                    continue;
                }
                let bb = s.bbox(false);
                let (w, h) = ((bb.max.x - bb.min.x).to_pt(), (bb.max.y - bb.min.y).to_pt());
                out.push(BoxInfo {
                    kind: BoxKind::Shape,
                    rect: bounds(ts, (pos.x + bb.min.x).to_pt(), (pos.y + bb.min.y).to_pt(), w, h),
                    text: None,
                    font: None,
                    size: None,
                    fill: s.fill.as_ref().and_then(solid),
                });
            }
            FrameItem::Image(_, size, _) => out.push(BoxInfo {
                kind: BoxKind::Image,
                rect: bounds(ts, pos.x.to_pt(), pos.y.to_pt(), size.x.to_pt(), size.y.to_pt()),
                text: None,
                font: None,
                size: None,
                fill: None,
            }),
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Engine;

    const DOC: &str = "#set page(width: 200pt, height: 100pt, margin: 10pt, fill: rgb(\"#102030\"))\n#set text(size: 20pt, fill: white)\n#place(top + left, dx: 40pt, dy: 30pt, rect(width: 50pt, height: 20pt, fill: red))\nHola\n";

    #[test]
    fn da_cajas_de_texto_forma_y_el_fondo_de_la_pagina() {
        let c = Engine::new(vec![]).compile(DOC, &std::env::temp_dir(), "g.typ");
        let (boxes, bg) = page_boxes(c.doc.as_ref().expect("compila"), 0).unwrap();
        assert_eq!(bg, Some([0x10, 0x20, 0x30]));
        let t = boxes.iter().find(|b| b.kind == BoxKind::Text).expect("texto");
        assert_eq!(t.text.as_deref(), Some("Hola"));
        assert_eq!(t.size, Some(20.0));
        assert_eq!(t.fill, Some([255, 255, 255, 255]));
        // el texto empieza en el margen (10 pt) y tiene un ancho razonable
        assert!((t.rect[0] - 10.0).abs() < 0.5, "{:?}", t.rect);
        assert!(t.rect[2] > 20.0 && t.rect[2] < 80.0, "{:?}", t.rect);
        let r = boxes.iter().find(|b| b.kind == BoxKind::Shape).expect("rect");
        assert_eq!(r.fill, Some([255, 65, 54, 255]), "el `red` de Typst es #ff4136");
        // `place` mide desde el margen (10 pt): dx 40 → x 50, dy 30 → y 40
        assert!((r.rect[0] - 50.0).abs() < 0.5 && (r.rect[1] - 40.0).abs() < 0.5, "{:?}", r.rect);
        assert!((r.rect[2] - 50.0).abs() < 0.5 && (r.rect[3] - 20.0).abs() < 0.5, "{:?}", r.rect);
        assert!(r.contains(60.0, 50.0) && !r.contains(5.0, 5.0));
        assert!(page_boxes(c.doc.as_ref().unwrap(), 3).is_none());
    }
}
