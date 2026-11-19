pub mod build;
pub mod cli;
pub mod process;
pub mod scaffold;

// El contrato y el puente viven en sus crates; se re-exportan para no mover las rutas.
pub use instatypst_bridge::{check, colors, config, pieza, render, themes, typst};
pub use instatypst_piece::{markup, model};
