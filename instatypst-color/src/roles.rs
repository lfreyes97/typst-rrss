//! Elegir los cinco roles del contrato (fondo, texto, apagado, acento, línea) a partir de
//! una lista de colores candidatos: los de una foto, o los de una paleta cualquiera.
//!
//! Las reglas, en Oklab/Oklch:
//! 1. **Fondo**: el candidato más abundante si es claramente claro (L > 0,75) u oscuro
//!    (L < 0,35); si es dudoso, decide la luminosidad media (oscuro bajo 0,6).
//! 2. **Texto**: el candidato (o `#f5f3ee` / `#141414`) de mayor contraste con el fondo.
//! 3. **Acento**: el de más *chroma* con contraste ≥ 3 sobre el fondo, población ≥ 1 % y
//!    chroma ≥ 0,04. Si ninguno cumple (una foto sin color), el acento es el texto y se avisa.
//! 4. **Apagado** y **línea**: mezcla del fondo y el texto al 55 %.

use crate::contrast::{ratio, MIN_ACCENT};
use crate::{Rgb, Roles, Warning};
use palette::{FromColor, Mix, Oklch};

/// Un color candidato y qué parte de la imagen ocupa (0.0 – 1.0).
#[derive(Clone, Copy, PartialEq, Debug)]
pub struct Candidate {
    pub rgb: Rgb,
    pub share: f32,
}

#[derive(Clone, PartialEq, Debug)]
pub struct Selection {
    pub roles: Roles,
    pub warnings: Vec<Warning>,
}

const CLARO: Rgb = Rgb([0xf5, 0xf3, 0xee]);
const OSCURO: Rgb = Rgb([0x14, 0x14, 0x14]);

const CLARAMENTE_CLARO: f32 = 0.75;
const CLARAMENTE_OSCURO: f32 = 0.35;
const MEDIA_OSCURA: f32 = 0.6;
const CHROMA_MINIMO: f32 = 0.04;
const POBLACION_MINIMA: f32 = 0.01;

fn l(c: Rgb) -> f32 {
    c.oklab().l
}

fn chroma(c: Rgb) -> f32 {
    Oklch::from_color(c.oklab()).chroma
}

fn mas_abundante<'a>(it: impl Iterator<Item = &'a Candidate>) -> Option<&'a Candidate> {
    it.max_by(|a, b| a.share.total_cmp(&b.share))
}

/// Los roles a partir de unos candidatos. `None` si no hay ninguno.
pub fn select(candidatos: &[Candidate]) -> Option<Selection> {
    if candidatos.is_empty() {
        return None;
    }
    let total: f32 = candidatos
        .iter()
        .map(|c| c.share)
        .sum::<f32>()
        .max(f32::EPSILON);
    let media: f32 = candidatos.iter().map(|c| l(c.rgb) * c.share).sum::<f32>() / total;
    let cima = mas_abundante(candidatos.iter())?;

    // 1 · fondo
    let fondo = if l(cima.rgb) > CLARAMENTE_CLARO || l(cima.rgb) < CLARAMENTE_OSCURO {
        cima.rgb
    } else if media < MEDIA_OSCURA {
        mas_abundante(candidatos.iter().filter(|c| l(c.rgb) < 0.45))
            .or_else(|| {
                candidatos
                    .iter()
                    .min_by(|a, b| l(a.rgb).total_cmp(&l(b.rgb)))
            })?
            .rgb
    } else {
        mas_abundante(candidatos.iter().filter(|c| l(c.rgb) > 0.80))
            .or_else(|| {
                candidatos
                    .iter()
                    .max_by(|a, b| l(a.rgb).total_cmp(&l(b.rgb)))
            })?
            .rgb
    };

    // 2 · texto
    let texto = candidatos
        .iter()
        .map(|c| c.rgb)
        .chain([CLARO, OSCURO])
        .max_by(|a, b| ratio(*a, fondo).total_cmp(&ratio(*b, fondo)))?;

    // 3 · acento
    let mut warnings = Vec::new();
    let acento = candidatos
        .iter()
        .filter(|c| {
            c.rgb != fondo
                && c.share >= POBLACION_MINIMA
                && chroma(c.rgb) >= CHROMA_MINIMO
                && ratio(c.rgb, fondo) >= MIN_ACCENT
        })
        .max_by(|a, b| chroma(a.rgb).total_cmp(&chroma(b.rgb)))
        .map(|c| c.rgb)
        .unwrap_or_else(|| {
            warnings.push(Warning {
                code: "no_accent",
                message: "la imagen no tiene un color con acento suficiente: el acento es el texto; elige uno".into(),
            });
            texto
        });

    // 4 · apagado y línea
    let apagado = Rgb::from_oklab(fondo.oklab().mix(texto.oklab(), 0.55));

    Some(Selection {
        roles: Roles {
            background: fondo,
            text: texto,
            muted: apagado,
            accent: acento,
            line: apagado,
        },
        warnings,
    })
}

/// Los roles a partir de una paleta cualquiera (una lista de colores, sin población):
/// todos pesan lo mismo.
pub fn from_palette(colores: &[Rgb]) -> Option<Selection> {
    let share = 1.0 / colores.len().max(1) as f32;
    let c: Vec<Candidate> = colores
        .iter()
        .map(|&rgb| Candidate { rgb, share })
        .collect();
    select(&c)
}

/// Los roles de una imagen: extrae sus colores dominantes y los pasa por [`select`].
#[cfg(feature = "image")]
pub fn from_image(path: &std::path::Path) -> Result<Selection, crate::extract::ExtractError> {
    let c = crate::extract::candidates_from_path(path, 8)?;
    select(&c).ok_or(crate::extract::ExtractError::Empty)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::contrast::ratio;

    fn c(hex: &str, share: f32) -> Candidate {
        Candidate {
            rgb: Rgb::parse(hex).unwrap(),
            share,
        }
    }

    /// Una foto oscura con un color vivo: navy dominante y un rosa (la del coche).
    #[test]
    fn foto_oscura_con_un_acento_vivo() {
        let s = select(&[
            c("#0a315e", 0.75),
            c("#283e70", 0.06),
            c("#ea6184", 0.05),
            c("#dcd4cc", 0.02),
        ])
        .unwrap();
        assert_eq!(s.roles.background.hex(), "#0a315e");
        assert_eq!(s.roles.accent.hex(), "#ea6184");
        assert!(ratio(s.roles.text, s.roles.background) >= 7.0);
        assert!(s.warnings.is_empty());
    }

    /// El arreglo de la prueba: el color más abundante es un crema, y manda aunque haya
    /// oscuros y la media sea baja. (La foto de la duna naranja.)
    #[test]
    fn el_fondo_es_el_color_mas_abundante_si_es_claramente_claro() {
        let s = select(&[
            c("#e1d5a5", 0.30),
            c("#250802", 0.17),
            c("#93270d", 0.16),
            c("#791806", 0.10),
            c("#bb5825", 0.09),
        ])
        .unwrap();
        assert_eq!(
            s.roles.background.hex(),
            "#e1d5a5",
            "el crema es lo más abundante"
        );
        assert!(l(s.roles.text) < 0.4, "texto oscuro sobre fondo claro");
        assert!(
            chroma(s.roles.accent) > 0.1,
            "el acento es uno de los naranjas"
        );
    }

    /// El otro arreglo: una foto sin color no puede dar un acento ilegible.
    #[test]
    fn una_foto_sin_color_no_da_un_acento_ilegible() {
        let s = select(&[
            c("#c8cdd1", 0.5),
            c("#c3c8cc", 0.3),
            c("#bec3c7", 0.15),
            c("#9ca3ab", 0.05),
        ])
        .unwrap();
        assert_eq!(
            s.roles.accent, s.roles.text,
            "sin acento propio, el acento es el texto"
        );
        assert!(ratio(s.roles.accent, s.roles.background) >= 7.0);
        assert_eq!(s.warnings.len(), 1);
        assert_eq!(s.warnings[0].code, "no_accent");
    }

    /// El acento tiene que leerse sobre el fondo: un rojo más vivo pero casi del mismo
    /// tono que el fondo no sirve, aunque tenga más chroma que el rosa claro.
    #[test]
    fn el_acento_exige_contraste_con_el_fondo() {
        let (rojo, rosa) = (c("#8a0030", 0.10), c("#f08fb0", 0.10));
        assert!(
            chroma(rojo.rgb) > chroma(rosa.rgb),
            "la premisa: el rojo es el más vivo"
        );
        let s = select(&[c("#0a315e", 0.80), rojo, rosa]).unwrap();
        assert!(
            ratio(rojo.rgb, s.roles.background) < 3.0,
            "la premisa: el rojo no se lee sobre el fondo"
        );
        assert_eq!(
            s.roles.accent, rosa.rgb,
            "se elige el que se lee, no el más vivo"
        );
        assert!(s.warnings.is_empty());
    }

    #[test]
    fn el_apagado_queda_entre_el_fondo_y_el_texto() {
        let s = select(&[c("#0a315e", 0.8), c("#ea6184", 0.2)]).unwrap();
        let r = s.roles;
        assert_eq!(r.muted, r.line);
        let (lf, lt, lm) = (l(r.background), l(r.text), l(r.muted));
        assert!(lm > lf.min(lt) && lm < lf.max(lt), "{lf} < {lm} < {lt}");
    }

    #[test]
    fn sin_candidatos_no_hay_seleccion() {
        assert!(select(&[]).is_none());
        assert!(from_palette(&[]).is_none());
    }

    #[test]
    fn una_paleta_cualquiera_tambien_da_roles() {
        let p: Vec<Rgb> = ["#102030", "#e94560", "#f0e6d2", "#406080"]
            .iter()
            .map(|h| Rgb::parse(h).unwrap())
            .collect();
        let s = from_palette(&p).unwrap();
        assert!(ratio(s.roles.text, s.roles.background) >= 4.5);
    }

    #[test]
    fn es_determinista() {
        let v = [c("#0a315e", 0.6), c("#ea6184", 0.2), c("#815a93", 0.2)];
        assert_eq!(select(&v), select(&v));
    }
}
