pub mod ecospold2;

use std::path::Path;

use comput::lca::Database;
use errors::{OdysseyErrors, Result};
use ecospold2::Ecoinvent;

pub fn load_database(name: &str, version: &str, cache_path: &Path) -> Result<Box<dyn Database>> {
    match name.to_lowercase().as_str() {
        "ecoinvent" => Ok(Box::new(Ecoinvent::load_from_cache(
            version,
            &cache_path.join(name).join(version),
        )?)),
        _ => Err(OdysseyErrors::MissingDatabase("haha".to_string())),
    }
}
