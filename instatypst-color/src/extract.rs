//! Los colores dominantes de una imagen: k-means en Oklab (con `quantette`), sobre una versión
//! reducida a 512 px, con la parte de la imagen que ocupa cada uno.

use crate::roles::Candidate;
use crate::Rgb;
use quantette::{ImageBuf, Pipeline, QuantizeMethod};
use std::fmt;
use std::path::Path;

/// Lado mayor al que se reduce la imagen antes de agrupar.
pub const MAX_SIDE: u32 = 512;

#[derive(Debug)]
pub enum ExtractError {
    /// No se pudo abrir o decodificar la imagen.
    Image(image::ImageError),
    /// La imagen no dio ningún color.
    Empty,
    /// El número de colores o el tamaño de la imagen no son válidos.
    Size,
}

impl fmt::Display for ExtractError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Image(e) => write!(f, "no se pudo leer la imagen: {e}"),
            Self::Empty => f.write_str("la imagen no tiene colores"),
            Self::Size => f.write_str("número de colores o tamaño de imagen no válidos"),
        }
    }
}

impl std::error::Error for ExtractError {}

impl From<image::ImageError> for ExtractError {
    fn from(e: image::ImageError) -> Self {
        Self::Image(e)
    }
}

/// Los `k` colores dominantes de una imagen ya cargada, de más a menos abundante.
/// Si la imagen es mayor que [`MAX_SIDE`], se reduce antes.
pub fn candidates(img: &image::RgbImage, k: u8) -> Result<Vec<Candidate>, ExtractError> {
    let reducida;
    let img = if img.width().max(img.height()) > MAX_SIDE {
        reducida = image::DynamicImage::ImageRgb8(img.clone())
            .thumbnail(MAX_SIDE, MAX_SIDE)
            .to_rgb8();
        &reducida
    } else {
        img
    };
    let buf = ImageBuf::try_from(img.clone()).map_err(|_| ExtractError::Size)?;
    let tam = u16::from(k).try_into().map_err(|_| ExtractError::Size)?;
    let (paleta, cuentas) = Pipeline::new()
        .palette_size(tam)
        .quantize_method(QuantizeMethod::kmeans())
        .parallel(true)
        .input_image(buf.as_ref())
        .output_oklab_palette_and_counts()
        .ok_or(ExtractError::Empty)?;
    let (paleta, cuentas) = (paleta.into_vec(), cuentas.into_vec());
    let total: u32 = cuentas.iter().sum();
    if total == 0 {
        return Err(ExtractError::Empty);
    }
    let mut v: Vec<Candidate> = paleta
        .iter()
        .zip(&cuentas)
        .map(|(o, n)| Candidate {
            rgb: Rgb::from_oklab(*o),
            share: *n as f32 / total as f32,
        })
        .collect();
    v.sort_by(|a, b| b.share.total_cmp(&a.share));
    Ok(v)
}

/// Lo mismo, desde un archivo. Para no cargar una foto de 96 megapíxeles entera en memoria
/// como RGB, se reduce antes de convertirla.
pub fn candidates_from_path(path: &Path, k: u8) -> Result<Vec<Candidate>, ExtractError> {
    let img = image::open(path)?;
    let img = if img.width().max(img.height()) > MAX_SIDE {
        img.thumbnail(MAX_SIDE, MAX_SIDE)
    } else {
        img
    };
    candidates(&img.to_rgb8(), k)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::contrast::ratio;
    use crate::roles::select;

    /// 60 % navy, 30 % crema, 10 % rosa.
    fn sintetica() -> image::RgbImage {
        image::RgbImage::from_fn(100, 100, |x, _| {
            let c = if x < 60 {
                [10, 49, 94]
            } else if x < 90 {
                [245, 243, 238]
            } else {
                [234, 97, 132]
            };
            image::Rgb(c)
        })
    }

    #[test]
    fn encuentra_los_tres_colores_con_su_parte() {
        let v = candidates(&sintetica(), 4).unwrap();
        let cerca = |hex: &str| {
            let o = Rgb::parse(hex).unwrap().oklab();
            v.iter()
                .find(|c| {
                    let d = c.rgb.oklab();
                    (d.l - o.l).hypot((d.a - o.a).hypot(d.b - o.b)) < 0.03
                })
                .unwrap_or_else(|| panic!("no hay un candidato cerca de {hex}: {v:?}"))
        };
        assert!((cerca("#0a315e").share - 0.6).abs() < 0.05);
        assert!((cerca("#f5f3ee").share - 0.3).abs() < 0.05);
        assert!((cerca("#ea6184").share - 0.1).abs() < 0.05);
        assert!(
            v.windows(2).all(|w| w[0].share >= w[1].share),
            "ordenados de más a menos"
        );
    }

    #[test]
    fn de_la_imagen_a_los_roles() {
        let s = select(&candidates(&sintetica(), 4).unwrap()).unwrap();
        assert_eq!(s.roles.background.hex(), "#0a315e");
        assert!(ratio(s.roles.text, s.roles.background) >= 7.0);
        assert!(s.warnings.is_empty(), "{:?}", s.warnings);
    }

    #[test]
    fn una_imagen_grande_se_reduce_y_da_lo_mismo() {
        let grande = image::RgbImage::from_fn(2000, 2000, |x, _| {
            let c = if x < 1200 {
                [10, 49, 94]
            } else if x < 1800 {
                [245, 243, 238]
            } else {
                [234, 97, 132]
            };
            image::Rgb(c)
        });
        let v = candidates(&grande, 4).unwrap();
        assert!((v[0].share - 0.6).abs() < 0.05, "{v:?}");
    }

    #[test]
    fn k_cero_es_un_error() {
        assert!(matches!(
            candidates(&sintetica(), 0),
            Err(ExtractError::Size)
        ));
    }

    #[test]
    fn un_archivo_que_no_existe_es_un_error_de_imagen() {
        let e =
            candidates_from_path(Path::new("/tmp/instatypst-no-hay-imagen.png"), 4).unwrap_err();
        assert!(matches!(e, ExtractError::Image(_)));
        assert!(e.to_string().starts_with("no se pudo leer la imagen"));
    }
}
