use git2::Repository;
use matrices::{MappedMatrix, MappedVector};
use serde::{Deserialize, Serialize};

use std::fs::{self, File};
use std::io::{BufReader, BufWriter};
use std::{collections::HashMap, path::Path};

use comput::impacts::ImpactCategory;
use comput::lca::Database;
use errors::Result;
use search::InventoryItem;

mod build;
mod impacts;
mod parse;

use build::{build_matrices, build_search_candidates};
use impacts::get_impact_matrices;
use parse::parse_ecospold2;

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

    fn load_from_files(version: &str, path: &Path, cache_path: &Path) -> Result<Self> {
        let mut processes = parse_ecospold2(path)?;
        let candidates = build_search_candidates(&mut processes, version);
        let (technology, intervention) = build_matrices(processes)?;

        let classifications = get_impact_matrices(
            version,
            vec!["ef31", "cml", "traci"],
            &intervention,
            cache_path,
        )?;
        Ok(Ecoinvent {
            version: version.to_string(),
            technology,
            intervention,
            classifications,
            candidates,
        })
    }

    pub fn load(version: &str, path: &Path, cache_path: &Path) -> Result<impl Database + use<>> {
        let cache_path = cache_path.join("Ecoinvent");
        if fs::exists(cache_path.join(version))? {
            let file = File::open(cache_path.join(version))?;
            let reader = BufReader::new(file);
            let data = bincode::deserialize_from(reader)?;
            return Ok(data);
        }
        upload_lcia_files(&cache_path)?;
        let res = Self::load_from_files(version, path, &cache_path)?;
        res.cache(&cache_path.join(version))?;
        Ok(res)
    }

    pub fn load_from_cache(version: &str, path: &Path) -> Result<impl Database + use<>> {
        if fs::exists(path)? {
            let file = File::open(path)?;
            let reader = BufReader::new(file);
            let data: Ecoinvent = bincode::deserialize_from(reader)?;
            Ok(data)
        } else {
            Err(errors::OdysseyErrors::NoCache(format!(
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
        let s = self.technology.solve(f)?;
        let g = self.intervention.dot(&s);
        let ef = self.classifications.get_mut(method).unwrap();
        let h = ef.dot(&g);
        Ok(h)
    }
}

fn upload_lcia_files(cache_path: &Path) -> Result<()> {
    let path = cache_path.join("ecoinvent_lcia");
    if !fs::exists(&path)? {
        Repository::clone("https://github.com/ecoinvent/lcia.git", path)?;
    }
    Ok(())
}
