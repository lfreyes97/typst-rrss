pub mod build;
pub mod check;
pub mod cli;
pub mod colors;
pub mod config;
pub mod pieza;
pub mod process;
pub mod render;
pub mod scaffold;
pub mod themes;
pub mod typst;

// El contrato vive en `instatypst-piece`; se re-exporta para no mover las rutas.
pub use instatypst_piece::{markup, model};
