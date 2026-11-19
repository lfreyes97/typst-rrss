//! El puente entre el contrato (`instatypst-piece`) y Typst: resuelve cada pieza
//! (paleta, formato, bloques → parámetros del preset), genera su fuente `.typ` y
//! calcula el informe de `check`. No lanza procesos ni compila: eso es del CLI
//! (`typst compile`) o de la GUI (`instatypst-engine`).

pub mod check;
pub mod colors;
pub mod config;
pub mod pieza;
pub mod render;
pub mod themes;
pub mod typst;

// Las rutas `crate::model` / `crate::markup` siguen valiendo dentro del puente.
pub use instatypst_piece::{markup, model};
