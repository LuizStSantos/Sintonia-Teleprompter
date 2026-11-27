//! # sintonia-roteiro
//!
//! Lógica do roteiro do teleprompter, sem nenhuma dependência de interface:
//!
//! - [`Roteiro`] / [`Segmento`]: blocos de leitura vindos do **Espelho** do
//!   Sintonia, de um **TXT** ou digitados;
//! - [`trechos_destaque`]: texto em destaque entre `<<` e `>>`;
//! - [`decodificar_txt`]: TXT em UTF-8 ou Windows-1252;
//! - [`tempo`]: cronômetro e leitura/formatação de durações.

mod destaque;
mod roteiro;
pub mod tempo;
mod txt;

pub use destaque::trechos_destaque;
pub use roteiro::{EspelhoResumo, Origem, Roteiro, Segmento, Tipo, data_br};
pub use txt::decodificar_txt;
