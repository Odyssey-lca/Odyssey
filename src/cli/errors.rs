use std::path::PathBuf;

pub type Result<T> = std::result::Result<T, CliError>;

#[derive(thiserror::Error, Debug)]
pub enum CliError {
    #[error("{0:?}")] OdysseyError(#[from] odyssey::errors::OdysseyErrors),
    #[error("{0:?}")] IoError(#[from] std::io::Error),
    #[error("{0:?}")] TantivyError(#[from] tantivy::TantivyError),
    #[error("CliError: -->  {path}:{line}")] GlobalCliError { path: PathBuf, line: usize },
}