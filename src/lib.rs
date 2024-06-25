#![deny(missing_docs)]
#![doc = include_str!("../README.md")]

//! ## Feature flags
#![doc = document_features::document_features!(feature_label = r#"<span class="stab portability"><code>{feature}</code></span>"#)]

#[cfg(feature = "devmem")]
pub mod client;
pub mod protocol;

pub use protocol::*;
