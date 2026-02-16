pub mod ecospold2;

use crate::{
    comput::lca::Database,
    errors::{OdysseyErrors, Result},
    parsers::ecospold2::Ecoinvent,
    utils::constants::DATABASES_PATH,
};

pub fn load_database(name: &str, version: &str) -> Result<Box<dyn Database>> {
    let database_name = format!("{}_{}", name, version);
    match name.to_lowercase().as_str() {
        "ecoinvent" => Ok(Box::new(Ecoinvent::load_from_cache(
            version,
            &DATABASES_PATH.join(database_name),
        )?)),
        _ => Err(OdysseyErrors::MissingDatabase("haha".to_string())),
    }
}
