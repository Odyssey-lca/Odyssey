use once_cell::sync::Lazy;
use std::path::PathBuf;

pub static ODYSSEY_PATH: Lazy<PathBuf> = Lazy::new(|| dirs::data_dir().unwrap().join("Odyssey"));
pub static SEARCH_PATH: Lazy<PathBuf> = Lazy::new(|| ODYSSEY_PATH.join("search"));
pub static DATABASES_PATH: Lazy<PathBuf> = Lazy::new(|| ODYSSEY_PATH.join("databases"));
pub static DATABASES_FILE: Lazy<PathBuf> = Lazy::new(|| DATABASES_PATH.join("databases.json"));
