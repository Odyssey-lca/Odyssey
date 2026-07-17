use std::fs::File;
use std::io::BufReader;
use std::{collections::HashMap, fs, path::Path, str::FromStr};

use serde::Deserialize;
use std::str;
use uuid::Uuid;

use rayon::prelude::*;

use errors::Result;

#[derive(Debug, Deserialize)]
pub struct Activity {
    #[serde(rename = "activityName")]
    pub activity_name: String,
}

#[derive(Debug, Deserialize)]
pub struct ActivityDescription {
    pub activity: Activity,
    pub geography: Geograpy,
}

#[derive(Debug, Deserialize)]
pub struct Geograpy {
    pub shortname: String,
}

#[derive(Debug, Deserialize)]
pub struct IntermediateExchange {
    #[serde(rename = "@activityLinkId")]
    pub process_id: Option<Uuid>,

    #[serde(rename = "@intermediateExchangeId")]
    pub product_id: Uuid,

    #[serde(rename = "@amount")]
    pub amount: f64,

    pub name: String,

    #[serde(rename = "unitName")]
    pub unit: String,

    #[serde(rename = "inputGroup")]
    pub input: Option<u16>,
}

#[derive(Debug, Deserialize)]
pub struct ElementaryExchange {
    #[serde(rename = "@elementaryExchangeId")]
    pub product_id: Uuid,

    #[serde(rename = "@amount")]
    pub amount: f64,
}

#[derive(Debug, Deserialize)]
pub struct Flows {
    #[serde(rename = "intermediateExchange")]
    pub intermediates: Vec<IntermediateExchange>,

    #[serde(rename = "elementaryExchange")]
    pub elementaries: Option<Vec<ElementaryExchange>>,
}

#[derive(Debug, Deserialize)]
pub struct ChildActivityDataset {
    #[serde(rename = "activityDescription")]
    pub activity_description: ActivityDescription,

    #[serde(rename = "flowData")]
    pub flows: Flows,
}

#[derive(Debug, Deserialize)]
pub struct EcoSpold {
    #[serde(rename = "childActivityDataset", alias = "activityDataset")]
    pub activity: ChildActivityDataset,
}

pub fn ids_from_str(ids: &str) -> Result<(Uuid, Uuid)> {
    let ids: Vec<&str> = ids.split("_").collect();
    let process_id = Uuid::from_str(ids.first().unwrap())?;
    let product_id = Uuid::from_str(ids.get(1).unwrap())?;
    Ok((process_id, product_id))
}


pub fn parse_ecospold2(folder: &Path) -> Result<HashMap<String, EcoSpold>> {
    let activities_paths = fs::read_dir(folder.join("datasets"))?;
    let res: HashMap<String, EcoSpold> = activities_paths
        .par_bridge()
        .map(|activity_path| {
            let path = activity_path.unwrap().path();
            let (process_id, product_id) =
                ids_from_str(path.file_stem().unwrap().to_str().unwrap()).unwrap();
            let f = File::open(path.clone()).unwrap();
            let reader = BufReader::new(f);
            let process: EcoSpold = match quick_xml::de::from_reader(reader) {
                Ok(data) => data,
                Err(e) => panic!("Error {:?} {:?}", e, path),
            };
            (format!("{}_{}", process_id, product_id), process)
        })
        .collect();
    Ok(res)
}
