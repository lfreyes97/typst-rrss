use anyhow::{anyhow, Result};

/// `#RRGGBB` (con o sin `#`) a sus tres canales.
pub fn hex_to_rgb_tuple(hex_color: &str) -> Result<(u8, u8, u8)> {
    let hex = hex_color.trim_start_matches('#');
    if hex.len() != 6 || !hex.bytes().all(|b| b.is_ascii_hexdigit()) {
        return Err(anyhow!(
            "color no válido: {hex_color:?} (se espera #RRGGBB)"
        ));
    }
    let canal = |i: usize| u8::from_str_radix(&hex[i..i + 2], 16).unwrap_or(0);
    Ok((canal(0), canal(2), canal(4)))
}

/// `true` si `k` sirve como identificador de Typst (nombre de argumento).
pub fn is_ident(k: &str) -> bool {
    let mut it = k.chars();
    matches!(it.next(), Some(c) if c.is_alphabetic() || c == '_')
        && it.all(|c| c.is_alphanumeric() || c == '_' || c == '-')
}
