use thiserror::Error;
pub type Result<T> = std::result::Result<T, SearchErrors>;

#[derive(Error, Debug)]
pub enum SearchErrors {
    #[error("{0:?}")]
    TantivyError(#[from] tantivy::TantivyError),
    
    #[error("{0:?}")]
    WrongDatabaseName(String),
}
