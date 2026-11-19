//! `instatypst-color`: los colores de Instatypst que el scaffold de Typst no puede dar.
//!
//! Qué hay aquí y qué no:
//! - **Sí**: extraer los colores de una imagen (`extract`), elegir con ellos los cinco roles
//!   del contrato (`roles`), medir contraste (`contrast`) y un catálogo pequeño de esquemas
//!   (`schemes`).
//! - Con la feature `gradient`: rampas de n pasos y duotonos (`gradient`), interpolados en Oklab.
//! - **No**: paletas decorativas ni tokens (eso es del scaffold, Instatypst); ni catálogos de
//!   esquemas de muestras (eso, si hace falta, es de la GUI).
//!
//! El crate no conoce el contrato: devuelve [`Roles`], y quien lo usa (el CLI, la GUI) hace
//! el pegamento hacia su `palette`. Así `color` y `piece` no dependen entre sí.

pub mod base;
pub mod contrast;
#[cfg(feature = "image")]
pub mod extract;
#[cfg(feature = "gradient")]
pub mod gradient;
pub mod hex;
#[cfg(feature = "material")]
pub mod material;
pub mod roles;
pub mod schemes;

use palette::{FromColor, Oklab, Srgb};
use std::fmt;

/// Un color sRGB de 8 bits por canal.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct Rgb(pub [u8; 3]);

impl Rgb {
    /// `#rrggbb`, en minúsculas.
    pub fn hex(self) -> String {
        hex::format(self)
    }

    pub fn parse(s: &str) -> Result<Self, hex::HexError> {
        hex::parse(s)
    }

    pub(crate) fn oklab(self) -> Oklab {
        let [r, g, b] = self.0;
        Oklab::from_color(Srgb::new(r, g, b).into_format::<f32>())
    }

    pub(crate) fn from_oklab(o: Oklab) -> Self {
        let s: Srgb<u8> = Srgb::from_color(o).into_format();
        Rgb([s.red, s.green, s.blue])
    }
}

impl fmt::Display for Rgb {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.hex())
    }
}

/// Los cinco roles del contrato (`instatypst-piece/v1`).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Roles {
    pub background: Rgb,
    pub text: Rgb,
    pub muted: Rgb,
    pub accent: Rgb,
    pub line: Rgb,
}

/// Algo que quien edita debería saber, sin que sea un error.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Warning {
    /// `contrast_low`, `no_accent`…
    pub code: &'static str,
    pub message: String,
}
