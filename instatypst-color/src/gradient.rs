//! Rampas de n pasos y duotonos, con `colorgrad`. Todo se interpola en Oklab, que es perceptualmente
//! uniforme: una rampa entre dos colores no tiene el «barro» del medio que da el RGB.
//!
//! - [`ramp`]: n colores equiespaciados entre dos o más paradas (los extremos son las paradas).
//! - [`Duotone`]: de un tono oscuro a uno claro, como el `recolor_image` de la versión en Python.
//!   [`Duotone::from_roles`] lo elige a partir de los roles de una pieza; con la feature `image`,
//!   [`apply`] lo aplica a una foto con una intensidad.

use crate::{Rgb, Roles};
use colorgrad::{BlendMode, Color, Gradient, GradientBuilder, LinearGradient};
use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GradientError {
    /// Una rampa necesita al menos dos paradas.
    NotEnoughStops,
    /// `colorgrad` no pudo construir el degradado.
    Build(String),
}

impl fmt::Display for GradientError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NotEnoughStops => f.write_str("una rampa necesita al menos dos colores"),
            Self::Build(e) => write!(f, "no se pudo construir el degradado: {e}"),
        }
    }
}

impl std::error::Error for GradientError {}

fn a_color(c: Rgb) -> Color {
    Color::from_rgba8(c.0[0], c.0[1], c.0[2], 255)
}

fn de_color(c: &Color) -> Rgb {
    let [r, g, b, _] = c.to_rgba8();
    Rgb([r, g, b])
}

fn degradado(paradas: &[Rgb]) -> Result<LinearGradient, GradientError> {
    if paradas.len() < 2 {
        return Err(GradientError::NotEnoughStops);
    }
    let colores: Vec<Color> = paradas.iter().copied().map(a_color).collect();
    GradientBuilder::new()
        .colors(&colores)
        .mode(BlendMode::Oklab)
        .build::<LinearGradient>()
        .map_err(|e| GradientError::Build(e.to_string()))
}

/// `n` colores equiespaciados a lo largo de las paradas. Con `n = 1` es la primera parada, y con
/// `n = 0`, nada. Los extremos de la rampa son exactamente la primera y la última parada.
pub fn ramp(paradas: &[Rgb], n: usize) -> Result<Vec<Rgb>, GradientError> {
    let g = degradado(paradas)?;
    Ok(match n {
        0 => Vec::new(),
        1 => vec![paradas[0]],
        _ => g.colors(n).map(|c| de_color(&c)).collect(),
    })
}

/// Un duotono: las sombras de una imagen van a `dark` y las luces a `light`.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Duotone {
    pub dark: Rgb,
    pub light: Rgb,
}

impl Duotone {
    /// El duotono de una pieza: su fondo y su acento, ordenados por luminosidad (el más oscuro es
    /// `dark`), de modo que sirve igual con un fondo oscuro que con uno claro.
    pub fn from_roles(r: &Roles) -> Self {
        let (a, b) = (r.background, r.accent);
        if crate::contrast::luminance(a) <= crate::contrast::luminance(b) {
            Self { dark: a, light: b }
        } else {
            Self { dark: b, light: a }
        }
    }

    /// La tabla de 256 colores: la entrada `i` es el color de un gris de valor `i`.
    pub fn table(&self) -> [Rgb; 256] {
        let v = ramp(&[self.dark, self.light], 256).expect("dos paradas siempre valen");
        v.try_into()
            .expect("una rampa de 256 pasos tiene 256 colores")
    }
}

/// Aplica un duotono a una imagen: la lleva a grises (luma de ITU-R 601, como hacía PIL), cambia
/// cada gris por su color de la tabla y mezcla con la original según `intensity` (0 = la original,
/// 1 = el duotono puro).
#[cfg(feature = "image")]
pub fn apply(img: &image::RgbImage, d: &Duotone, intensity: f32) -> image::RgbImage {
    let tabla = d.table();
    let i = intensity.clamp(0.0, 1.0);
    let mezcla = |a: u8, b: u8| (a as f32 * (1.0 - i) + b as f32 * i).round() as u8;
    image::RgbImage::from_fn(img.width(), img.height(), |x, y| {
        let p = img.get_pixel(x, y).0;
        let gris = ((p[0] as u32 * 299 + p[1] as u32 * 587 + p[2] as u32 * 114) / 1000) as usize;
        let t = tabla[gris.min(255)].0;
        image::Rgb([mezcla(p[0], t[0]), mezcla(p[1], t[1]), mezcla(p[2], t[2])])
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::contrast::luminance;

    fn rgb(h: &str) -> Rgb {
        Rgb::parse(h).unwrap()
    }

    #[test]
    fn los_extremos_son_las_paradas() {
        let (a, b) = (rgb("#0a315e"), rgb("#ea6184"));
        let r = ramp(&[a, b], 7).unwrap();
        assert_eq!(r.len(), 7);
        assert_eq!((r[0], r[6]), (a, b));
    }

    #[test]
    fn la_rampa_de_oscuro_a_claro_sube_en_luminosidad() {
        let r = ramp(&[rgb("#101820"), rgb("#f5f3ee")], 9).unwrap();
        assert!(
            r.windows(2).all(|w| luminance(w[0]) <= luminance(w[1])),
            "{r:?}"
        );
    }

    #[test]
    fn con_varias_paradas_pasa_por_la_del_medio() {
        let medio = rgb("#e94560");
        let r = ramp(&[rgb("#0a315e"), medio, rgb("#f5f3ee")], 5).unwrap();
        // 5 pasos sobre 3 paradas: el paso 2 cae sobre la parada central
        let (d, m) = (r[2].oklab(), medio.oklab());
        assert!(
            (d.l - m.l).hypot((d.a - m.a).hypot(d.b - m.b)) < 0.02,
            "{r:?}"
        );
    }

    #[test]
    fn los_casos_limite() {
        let p = [rgb("#000000"), rgb("#ffffff")];
        assert!(ramp(&p, 0).unwrap().is_empty());
        assert_eq!(ramp(&p, 1).unwrap(), vec![rgb("#000000")]);
        assert_eq!(ramp(&p, 2).unwrap(), vec![rgb("#000000"), rgb("#ffffff")]);
        assert_eq!(
            ramp(&[rgb("#000000")], 4),
            Err(GradientError::NotEnoughStops)
        );
        assert_eq!(ramp(&[], 4), Err(GradientError::NotEnoughStops));
    }

    #[test]
    fn interpolar_en_oklab_no_da_el_barro_del_rgb() {
        // Entre azul y amarillo, el RGB pasa por un gris apagado; en Oklab el punto medio conserva
        // más chroma. Es lo que justifica elegir Oklab.
        let r = ramp(&[rgb("#0000ff"), rgb("#ffff00")], 3).unwrap();
        let chroma = |c: Rgb| palette::Oklch::from_color(c.oklab()).chroma;
        use palette::FromColor;
        let rgb_medio = Rgb([127, 127, 128]); // lo que daría mezclar en RGB
        assert!(chroma(r[1]) > chroma(rgb_medio), "{:?}", r[1]);
    }

    #[test]
    fn la_tabla_va_de_las_sombras_a_las_luces() {
        let d = Duotone {
            dark: rgb("#101820"),
            light: rgb("#f5f3ee"),
        };
        let t = d.table();
        assert_eq!((t[0], t[255]), (d.dark, d.light));
        assert!(t
            .windows(2)
            .all(|w| luminance(w[0]) <= luminance(w[1]) + 1e-9));
    }

    fn roles(fondo: &str, acento: &str) -> Roles {
        Roles {
            background: rgb(fondo),
            text: rgb("#ffffff"),
            muted: rgb("#888888"),
            accent: rgb(acento),
            line: rgb("#888888"),
        }
    }

    #[test]
    fn el_duotono_de_una_pieza_ordena_por_luminosidad() {
        // fondo oscuro, acento claro
        let a = Duotone::from_roles(&roles("#0a315e", "#ea6184"));
        assert_eq!((a.dark, a.light), (rgb("#0a315e"), rgb("#ea6184")));
        // fondo claro, acento más oscuro: se invierten, para que `dark` siga siendo el oscuro
        let b = Duotone::from_roles(&roles("#f3ece2", "#c8442b"));
        assert_eq!((b.dark, b.light), (rgb("#c8442b"), rgb("#f3ece2")));
    }

    #[cfg(feature = "image")]
    mod con_imagen {
        use super::*;

        fn uniforme(v: u8) -> image::RgbImage {
            image::RgbImage::from_pixel(4, 4, image::Rgb([v, v, v]))
        }

        #[test]
        fn negro_va_a_dark_y_blanco_a_light() {
            let d = Duotone {
                dark: rgb("#101820"),
                light: rgb("#f5f3ee"),
            };
            assert_eq!(apply(&uniforme(0), &d, 1.0).get_pixel(0, 0).0, d.dark.0);
            assert_eq!(apply(&uniforme(255), &d, 1.0).get_pixel(0, 0).0, d.light.0);
        }

        #[test]
        fn intensidad_cero_deja_la_original() {
            let d = Duotone {
                dark: rgb("#ff0000"),
                light: rgb("#00ff00"),
            };
            let img =
                image::RgbImage::from_fn(3, 3, |x, y| image::Rgb([x as u8 * 80, y as u8 * 80, 40]));
            assert_eq!(apply(&img, &d, 0.0), img);
        }

        #[test]
        fn la_mitad_mezcla_a_medias() {
            let d = Duotone {
                dark: rgb("#ff0000"),
                light: rgb("#ff0000"),
            };
            // el duotono es rojo puro: a la mitad, un gris de 100 queda a medio camino del rojo
            let p = apply(&uniforme(100), &d, 0.5).get_pixel(0, 0).0;
            assert_eq!(p, [178, 50, 50]); // (100+255)/2, (100+0)/2, (100+0)/2, redondeado
        }

        #[test]
        fn una_intensidad_fuera_de_rango_se_acota() {
            let d = Duotone {
                dark: rgb("#101820"),
                light: rgb("#f5f3ee"),
            };
            assert_eq!(apply(&uniforme(0), &d, 7.0), apply(&uniforme(0), &d, 1.0));
            assert_eq!(apply(&uniforme(90), &d, -3.0), uniforme(90));
        }
    }
}
