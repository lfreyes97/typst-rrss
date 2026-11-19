//! El catálogo de esquemas del motor: los cinco propios y once de base16. Son datos.
//!
//! Cada uno se reduce a los roles del contrato: `background`, `text`, `muted`, `accent`
//! (el «primary» de la CLI de siempre) y `line` (igual que `muted`). De las ocho claves que
//! tenían (`surface`, `secondary`, `highlight`…) ninguna se usaba.

use crate::{Rgb, Roles};

/// (nombre, fondo, texto, apagado, acento)
const TABLA: [(&str, &str, &str, &str, &str); 16] = [
    ("dark", "#0f0f0f", "#f5f5f5", "#a0a0a0", "#e94560"),
    ("light", "#fafafa", "#1a1a2e", "#6b7280", "#e94560"),
    ("ocean", "#0a192f", "#ccd6f6", "#8892b0", "#64ffda"),
    ("sunset", "#1a1a2e", "#f5f5f5", "#a0a0a0", "#ff6b6b"),
    ("forest", "#1b2d1b", "#f0f0f0", "#98b898", "#a8e6cf"),
    ("nord", "#2e3440", "#e5e9f0", "#4c566a", "#bf616a"),
    ("gruvbox-dark", "#282828", "#d5c4a1", "#665c54", "#fb4934"),
    (
        "catppuccin-mocha",
        "#1e1e2e",
        "#cdd6f4",
        "#45475a",
        "#f38ba8",
    ),
    ("tokyo-night", "#1a1b26", "#cfc9c2", "#414868", "#f7768e"),
    ("dracula", "#282a36", "#f8f8f2", "#6272a4", "#ff5555"),
    ("one-dark", "#282c34", "#abb2bf", "#545862", "#e06c75"),
    ("rose-pine", "#191724", "#e0def4", "#6e6a86", "#eb6f92"),
    (
        "catppuccin-latte",
        "#eff1f5",
        "#4c4f69",
        "#bcc0cc",
        "#d20f39",
    ),
    ("gruvbox-light", "#fbf1c7", "#3c3836", "#bdae93", "#9d0006"),
    ("solarized-dark", "#002b36", "#93a1a1", "#657b83", "#dc322f"),
    (
        "solarized-light",
        "#fdf6e3",
        "#586e75",
        "#839496",
        "#dc322f",
    ),
];

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Scheme {
    pub name: &'static str,
    pub roles: Roles,
}

fn de_tabla((name, bg, text, muted, accent): (&'static str, &str, &str, &str, &str)) -> Scheme {
    let c = |h: &str| Rgb::parse(h).expect("la tabla solo lleva colores válidos");
    let muted = c(muted);
    Scheme {
        name,
        roles: Roles {
            background: c(bg),
            text: c(text),
            muted,
            accent: c(accent),
            line: muted,
        },
    }
}

/// Todos, en el orden del catálogo.
pub fn all() -> Vec<Scheme> {
    TABLA.iter().copied().map(de_tabla).collect()
}

pub fn get(name: &str) -> Option<Scheme> {
    TABLA.iter().copied().find(|t| t.0 == name).map(de_tabla)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hay_dieciseis_con_nombres_unicos() {
        let todos = all();
        assert_eq!(todos.len(), 16);
        let mut nombres: Vec<_> = todos.iter().map(|s| s.name).collect();
        nombres.sort_unstable();
        nombres.dedup();
        assert_eq!(nombres.len(), 16);
    }

    #[test]
    fn get_encuentra_por_nombre() {
        let nord = get("nord").unwrap();
        assert_eq!(nord.roles.background.hex(), "#2e3440");
        assert_eq!(nord.roles.line, nord.roles.muted);
        assert!(get("no-existe").is_none());
    }
}
