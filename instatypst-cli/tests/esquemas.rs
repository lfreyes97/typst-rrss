//! Mientras el CLI siga usando `themes.rs`, su tabla y la de `instatypst-color` deben coincidir:
//! el crate nuevo es la misma información, ya reducida a los roles del contrato.

use instatypst_cli::themes::predefined::get_theme_palettes;
use instatypst_color::schemes;

#[test]
fn los_esquemas_del_crate_de_color_son_los_de_themes_rs() {
    let viejos = get_theme_palettes(None);
    let nuevos = schemes::all();
    assert_eq!(viejos.len(), nuevos.len(), "mismo número de esquemas");
    for s in nuevos {
        let v = viejos
            .get(s.name)
            .unwrap_or_else(|| panic!("`{}` no está en themes.rs", s.name));
        let igual = |clave: &str, rol: instatypst_color::Rgb| {
            assert_eq!(
                v[clave].to_ascii_lowercase(),
                rol.hex(),
                "{}: `{clave}` difiere",
                s.name
            );
        };
        igual("bg", s.roles.background);
        igual("text", s.roles.text);
        igual("muted", s.roles.muted);
        igual("primary", s.roles.accent); // el «primary» de la CLI de siempre es el acento
        assert_eq!(
            s.roles.line, s.roles.muted,
            "{}: la línea es el apagado",
            s.name
        );
    }
}
