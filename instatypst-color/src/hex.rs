//! `#RRGGBB` ↔ [`Rgb`].

use crate::Rgb;
use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HexError(pub String);

impl fmt::Display for HexError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "color no válido: {:?} (se espera #RRGGBB)", self.0)
    }
}

impl std::error::Error for HexError {}

/// Acepta `#rrggbb` o `rrggbb`, con cualquier combinación de mayúsculas.
pub fn parse(s: &str) -> Result<Rgb, HexError> {
    let h = s.strip_prefix('#').unwrap_or(s);
    if h.len() != 6 || !h.bytes().all(|b| b.is_ascii_hexdigit()) {
        return Err(HexError(s.to_string()));
    }
    let canal = |i: usize| u8::from_str_radix(&h[i..i + 2], 16).expect("ya validado");
    Ok(Rgb([canal(0), canal(2), canal(4)]))
}

pub fn format(c: Rgb) -> String {
    format!("#{:02x}{:02x}{:02x}", c.0[0], c.0[1], c.0[2])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ida_y_vuelta() {
        for s in ["#000000", "#ffffff", "#691322", "#0a315e"] {
            assert_eq!(format(parse(s).unwrap()), s);
        }
    }

    #[test]
    fn acepta_mayusculas_y_sin_almohadilla() {
        assert_eq!(parse("#E94560").unwrap(), Rgb([0xe9, 0x45, 0x60]));
        assert_eq!(parse("e94560").unwrap(), Rgb([0xe9, 0x45, 0x60]));
    }

    #[test]
    fn rechaza_lo_que_no_es_un_color() {
        for s in [
            "", "#12345", "#1234567", "rojo", "#gggggg", "##123456", "#12 456",
        ] {
            assert!(parse(s).is_err(), "{s:?} debería fallar");
        }
        assert!(parse("rojo").unwrap_err().to_string().contains("rojo"));
    }
}
