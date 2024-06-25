//! Protocol interface for manipulating, decoding, and encoding
//! OCSD structures.

mod data;
pub mod error;
mod ocsd;
mod temperature;

pub use ocsd::*;
pub use temperature::Celsius;
