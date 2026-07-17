pub mod ecospold2;
pub mod impacts;

use impacts::ImpactCategory;
use ecospold2::Ecoinvent;
use errors::{OdysseyErrors, Result};
use matrices::MappedVector;
use search::InventoryItem;
use std::{ fmt::{self}, path::Path};

pub fn load_database(name: &str, version: &str, cache_path: &Path) -> Result<Box<dyn Database>> {
    match name.to_lowercase().as_str() {
        "ecoinvent" => Ok(Box::new(Ecoinvent::load_from_cache(
            version,
            &cache_path.join(name).join(version),
        )?)),
        _ => Err(OdysseyErrors::MissingDatabase("haha".to_string())),
    }
}

#[derive(Debug)]
pub struct DatabaseInfos {
    pub name: String,
    pub version: String,
}

impl fmt::Display for DatabaseInfos {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}_{}", self.name, self.version)
    }
}

pub trait Database {
    /// Name of the database
    fn name(&self) -> String;

    fn list_candidates(&self) -> Vec<&InventoryItem>;

    fn get_candidate(&self, id: &str) -> Option<&InventoryItem>;

    fn empty_reference_flow(&self) -> MappedVector<String>;
    fn empty_impacts(&self, method: &str) -> MappedVector<ImpactCategory>;

    /// Performs the life cycle assessment of the items specified in the reference flow `f`.
    fn lca(
        &mut self,
        f: &MappedVector<String>,
        method: &str,
    ) -> Result<MappedVector<ImpactCategory>>;
}
