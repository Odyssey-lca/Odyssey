use mapped_sparse_matrix::MappedVector;
use serde::{Deserialize, Serialize};
use std::hash::Hash;

pub mod cml;
pub mod ef31;
pub mod traci;

pub use ef31::EF31;

use crate::comput::impacts::{cml::CML, traci::TRACI};

#[derive(PartialEq, std::cmp::Eq, Clone, Serialize, Deserialize, Debug, Hash)]
pub enum ImpactCategory {
    EF31(EF31),
    CML(CML),
    TRACI(TRACI),
}

impl ImpactCategory {
    pub fn get_empty_vector(method: &str) -> MappedVector<ImpactCategory> {
        match method {
            "ef31" => EF31::get_empty_vector(),
            "cml" => CML::get_empty_vector(),
            "traci" => TRACI::get_empty_vector(),
            a => panic!("Unknown Method: {}", a),
        }
    }
}
