use std::io;
use thiserror::Error;
pub type Result<T> = std::result::Result<T, OdysseyErrors>;

#[derive(Error, Debug)]
pub enum OdysseyErrors {
    #[error("{0:?}")]
    IoError(#[from] io::Error),
    #[error("{0:?}")]
    GitError(#[from] git2::Error),
    #[error("{0:?}")]
    CsvError(#[from] csv::Error),
    #[error("{0:?}")]
    XmlError(#[from] quick_xml::Error),
    #[error("{0:?}")]
    XmlDeError(#[from] quick_xml::de::DeError),
    #[error("{0:?}")]
    JsonError(#[from] serde_json::Error),
    #[error("{0:?}")]
    BinCacheError(#[from] bincode::Error),
    #[error("{0:?}")]
    UuidError(#[from] uuid::Error),
    #[error("{0:?}")]
    TantivyError(#[from] tantivy::TantivyError),
    #[error("{0:?}")]
    MissingId(String),
    #[error("{0:?}")]
    MissingDatabase(String),
    #[error("{0:?}")]
    NoCache(String),
}
