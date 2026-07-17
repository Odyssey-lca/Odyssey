use std::path::Path;

use matrices::{MappedMatrix, MappedMatrixBuilder};
use crate::impacts::{traci::TRACI, ImpactCategory};
use errors::Result;
use super::misc::{get_ecoinvent_mapping_file, get_empty_matrix};

#[rustfmt::skip]
#[derive(Debug, serde::Deserialize)]
pub struct TRACIImpacts {
    #[serde(alias = "elementary_flow_id")]
    pub flow_id: String,

    #[serde(alias = "climate change|global warming potential (GWP100)")]
    pub gwp100: Option<f64>,

    #[serde(alias = "acidification|acidification potential (AP)")]
    pub acidification: Option<f64>,

    #[serde(alias = "eutrophication|eutrophication potential")]
    pub eutrophication: Option<f64>,

    #[serde(alias = "particulate matter formation|particulate matter formation potential (PMFP)")]
    pub particulate_matter: Option<f64>,

    #[serde(alias = "ozone depletion|ozone depletion potential (ODP)")]
    pub ozone_depletion: Option<f64>,

    #[serde(alias = "photochemical oxidant formation|maximum incremental reactivity (MIR)")]
    pub photochemical_oxidant: Option<f64>,

    #[serde(alias = "ecotoxicity: freshwater|ecotoxicity: freshwater")]
    pub ecotoxicity_freshwater: Option<f64>,

    #[serde(alias = "human toxicity: carcinogenic|human toxicity: carcinogenic")]
    pub human_toxicity_carcinogenic: Option<f64>,

    #[serde(alias = "human toxicity: non-carcinogenic|human toxicity: non-carcinogenic")]
    pub human_toxicity_non_carcinogenic: Option<f64>,
}

#[rustfmt::skip]
impl TRACIImpacts {
  
  fn add_triplets(&self, a: &mut MappedMatrixBuilder<ImpactCategory, String>, col: String) {
        // Order is important
        a.add_triplet(ImpactCategory::TRACI(TRACI::Gwp100), col.clone(), self.gwp100.unwrap_or(0.));
        a.add_triplet(ImpactCategory::TRACI(TRACI::Acidification), col.clone(), self.acidification.unwrap_or(0.));
        a.add_triplet(ImpactCategory::TRACI(TRACI::ParticulMatter), col.clone(), self.particulate_matter.unwrap_or(0.));
        a.add_triplet(ImpactCategory::TRACI(TRACI::OzoneDepletion), col.clone(), self.ozone_depletion.unwrap_or(0.));
        a.add_triplet(ImpactCategory::TRACI(TRACI::PhotochemicalOxidant), col.clone(), self.photochemical_oxidant.unwrap_or(0.));
        a.add_triplet(ImpactCategory::TRACI(TRACI::Eutrophication), col.clone(), self.eutrophication.unwrap_or(0.));
        a.add_triplet(ImpactCategory::TRACI(TRACI::EcotoxicityFreshwater), col.clone(), self.ecotoxicity_freshwater.unwrap_or(0.));
        a.add_triplet(ImpactCategory::TRACI(TRACI::HumanToxicityCarcinogenic), col.clone(), self.human_toxicity_carcinogenic.unwrap_or(0.));
        a.add_triplet(ImpactCategory::TRACI(TRACI::HumanToxicityNonCarcinogenic), col.clone(), self.human_toxicity_non_carcinogenic.unwrap_or(0.));
    }
}

pub fn get_traci_matrix(
    version: &str,
    intervention: &MappedMatrix<String, String>,
    cache_path: &Path,
) -> Result<MappedMatrix<ImpactCategory, String>> {
    let mut mat = get_empty_matrix(TRACI::get_empty_vector(), intervention);
    let mut rdr = get_ecoinvent_mapping_file(version, "TRACI v2.1", cache_path)?;
    for result in rdr.deserialize() {
        let record: TRACIImpacts = result?;
        let elementary_id = record.flow_id.clone();
        if intervention.contains_row(&elementary_id) {
            record.add_triplets(&mut mat, elementary_id);
        }
    }
    Ok(mat.build())
}
