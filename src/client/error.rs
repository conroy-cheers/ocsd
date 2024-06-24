use std::{error::Error, fmt::Display};

#[derive(Debug, Clone)]
pub struct MappingError {
    msg: String,
}

impl MappingError {
    pub(crate) fn new(msg: impl Into<String>) -> Self {
        Self { msg: msg.into() }
    }
}

impl Display for MappingError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        ("unable to open map: ".to_string() + &self.msg).fmt(f)
    }
}

impl Error for MappingError {}
