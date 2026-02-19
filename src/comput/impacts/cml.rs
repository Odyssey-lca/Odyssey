use std::sync::Arc;

use bimap::BiHashMap;
use mapped_sparse_matrix::MappedVector;
use serde::{Deserialize, Serialize};
use strum::IntoEnumIterator;
use strum_macros::EnumIter;

use crate::comput::impacts::ImpactCategory;

#[derive(PartialEq, std::cmp::Eq, Clone, Serialize, Deserialize, Debug, Hash, EnumIter)]
pub enum CML {
    Gwp100,
    ResourcesMetalsMinerals,
    EnergyResourcesNonRenewable,
    OzoneDepletion,
    HumanToxicity,
    EcotoxicityFreshwater,
    EcotoxicityMarine,
    EcotoxicityTerrestrial,
    PhotochemicalOxidantFormation,
    Acidification,
    Eutrophication,
}

impl CML {
    pub fn get_empty_vector() -> MappedVector<ImpactCategory> {
        let mut mapping = BiHashMap::new();
        CML::iter().enumerate().for_each(|(i, c)| {
            let _ = mapping.insert(ImpactCategory::CML(c), i);
        });
        let length = mapping.len();
        MappedVector::new(Arc::new(mapping), vec![0.; length])
    }
}
