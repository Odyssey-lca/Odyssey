use matrices::MappedVector;
use serde::{Deserialize, Serialize};
use std::hash::Hash;

pub mod cml;
pub mod ef31;
pub mod traci;

use cml::CML;
use ef31::EF31;
use traci::TRACI;

#[derive(PartialEq, std::cmp::Eq, Clone, Serialize, Deserialize, Debug, Hash)]
pub enum ImpactCategory {
    EF31(EF31),
    CML(CML),
    TRACI(TRACI),
}

pub enum ImpactMethod {
    EF31,
    CML,
    TRACI,
}

impl ImpactMethod {
    pub fn get_empty_vector(&self) -> MappedVector<ImpactCategory> {
        match self {
            ImpactMethod::EF31 => EF31::get_empty_vector(),
            ImpactMethod::CML => CML::get_empty_vector(),
            ImpactMethod::TRACI => TRACI::get_empty_vector(),
        }
    }
}
