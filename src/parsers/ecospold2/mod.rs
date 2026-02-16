use git2::Repository;
use serde::{Deserialize, Serialize};

use std::fs::{self, File};
use std::io::{BufReader, BufWriter};
use std::{collections::HashMap, path::Path};

use crate::comput::impacts::ImpactCategory;
use crate::comput::lca::Database;
use crate::errors::Result;
use crate::parsers::ecospold2::build::{build_candidates, build_matrices};
use crate::parsers::ecospold2::impacts::get_impact_matrices;
use crate::parsers::ecospold2::parse::parse_ecospold2;
use crate::utils::constants::DATABASES_PATH;
use crate::utils::matrix::{MappedMatrix, MappedVector};
use crate::utils::search::InventoryItem;

mod build;
mod impacts;
mod parse;

#[derive(Serialize, Deserialize, Debug)]
pub struct Ecoinvent {
    version: String,
    technology: MappedMatrix<String, String>,
    intervention: MappedMatrix<String, String>,
    classifications: HashMap<String, MappedMatrix<ImpactCategory, String>>,
    candidates: HashMap<String, InventoryItem>,
}

impl Ecoinvent {
    /// Save the database data in a cache at the specified `path`.
    fn cache(&self, cache: &Path) -> Result<()> {
        let file = File::create(cache)?;
        let writer = BufWriter::new(file);
        bincode::serialize_into(writer, self).expect("Failed to serialize");
        Ok(())
    }

    fn load_from_files(version: &str, path: &Path) -> Result<Self> {
        let mut processes = parse_ecospold2(path)?;
        let candidates = build_candidates(&mut processes, version);
        let (technology, intervention) = build_matrices(processes)?;
        upload_lcia_files()?;
        let classifications =
            get_impact_matrices(version, vec!["ef31", "cml", "traci"], &intervention)?;
        Ok(Ecoinvent {
            version: version.to_string(),
            technology,
            intervention,
            classifications,
            candidates,
        })
    }

    pub fn load(version: &str, path: &Path, cache: Option<&Path>) -> Result<impl Database> {
        if let Some(cache) = cache {
            if fs::exists(cache)? {
                let file = File::open(cache)?;
                let reader = BufReader::new(file);
                let data = bincode::deserialize_from(reader)?;
                return Ok(data);
            }
        }
        let res = Self::load_from_files(version, path)?;
        if let Some(cache) = cache {
            res.cache(cache)?;
        }
        Ok(res)
    }

    pub fn load_from_cache(version: &str, path: &Path) -> Result<impl Database> {
        if fs::exists(path)? {
            let file = File::open(path)?;
            let reader = BufReader::new(file);
            let data: Ecoinvent = bincode::deserialize_from(reader)?;
            Ok(data)
        } else {
            Err(crate::errors::OdysseyErrors::NoCache(format!(
                "Ecoinvent {} was not previously loaded",
                version
            )))
        }
    }
}
impl Database for Ecoinvent {
    fn name(&self) -> String {
        format!("ecoinvent_{}", self.version)
    }

    fn empty_reference_flow(&self) -> MappedVector<String> {
        self.technology.zeros_like_cols()
    }

    fn empty_impacts(&self, method: &str) -> MappedVector<ImpactCategory> {
        self.classifications.get(method).unwrap().zeros_like_rows()
    }

    fn list_candidates(&self) -> Vec<&InventoryItem> {
        self.candidates.values().collect()
    }

    fn get_candidate(&self, id: &str) -> Option<&InventoryItem> {
        self.candidates.get(id)
    }

    fn lca(
        &mut self,
        f: &MappedVector<String>,
        method: &str,
    ) -> Result<MappedVector<ImpactCategory>> {
        let s = self.technology.solve(f);
        let g = self.intervention.dot(&s);
        let ef = self.classifications.get_mut(method).unwrap();
        let h = ef.dot(&g);
        Ok(h)
    }
}

fn upload_lcia_files() -> Result<()> {
    let path = DATABASES_PATH.join("ecoinvent_lcia");
    if !fs::exists(&path)? {
        Repository::clone("https://github.com/ecoinvent/lcia.git", path)?;
    }
    Ok(())
}
