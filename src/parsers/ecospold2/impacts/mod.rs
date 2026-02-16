use std::collections::HashMap;

use crate::{
    comput::impacts::ImpactCategory,
    errors::Result,
    parsers::ecospold2::impacts::{
        cml::get_cml_matrix, ef31::get_ef31_matrix, traci::get_traci_matrix,
    },
    utils::matrix::MappedMatrix,
};

pub mod cml;
pub mod ef31;
mod misc;
pub mod traci;

pub fn get_impact_matrices(
    version: &str,
    methods: Vec<&str>,
    intervention: &MappedMatrix<String, String>,
) -> Result<HashMap<String, MappedMatrix<ImpactCategory, String>>> {
    let mut res = HashMap::new();
    for method in methods {
        let mat = match method {
            "ef31" => get_ef31_matrix(version, intervention)?,
            "cml" => get_cml_matrix(version, intervention)?,
            "traci" => get_traci_matrix(version, intervention)?,
            _ => panic!("Unsuported method"),
        };
        res.insert(method.to_string(), mat);
    }
    Ok(res)
}
