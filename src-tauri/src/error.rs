use std::io;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum PromptToolError {
    #[error("IO error: {0}")]
    Io(#[from] io::Error),

    #[error("TOML parsing error: {0}")]
    TomlParse(#[from] toml::de::Error),

    #[error("Configuration error: {0}")]
    Config(String),

    #[error("Prompt validation error: {0}")]
    Validation(String),

    #[error("Search error: {0}")]
    Search(String),
}

pub type Result<T> = std::result::Result<T, PromptToolError>;

impl serde::Serialize for PromptToolError {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}
