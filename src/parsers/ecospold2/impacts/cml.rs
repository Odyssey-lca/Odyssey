use crate::{
    comput::impacts::{cml::CML, ImpactCategory},
    errors::Result,
    parsers::ecospold2::impacts::misc::{get_ecoinvent_mapping_file, get_empty_matrix},
    utils::matrix::{MappedMatrix, MappedMatrixBuilder},
};

#[rustfmt::skip]
#[derive(Debug, serde::Deserialize)]
pub struct CMLImpacts {
    #[serde(alias = "elementary_flow_id")]
    pub flow_id: String,

    #[serde(alias = "climate change|global warming potential (GWP100)")]
    pub gwp100: Option<f64>,

    #[serde(alias = "material resources: metals/minerals|abiotic depletion potential (ADP): elements (ultimate reserves)")]
    pub resources_metals_minerals: Option<f64>,

    #[serde(alias = "energy resources: non-renewable|abiotic depletion potential (ADP): fossil fuels")]
    pub energy_resources_non_renewable: Option<f64>,

    #[serde(alias = "ozone depletion|ozone layer depletion (ODP steady state)")]
    pub ozone_depletion: Option<f64>,

    #[serde(alias = "human toxicity|human toxicity (HTP inf)")]
    pub human_toxicity: Option<f64>,

    #[serde(alias = "ecotoxicity: freshwater|freshwater aquatic ecotoxicity (FAETP inf)")]
    pub ecotoxicity_freshwater: Option<f64>,

    #[serde(alias = "ecotoxicity: marine|marine aquatic ecotoxicity (MAETP inf)")]
    pub ecotoxicity_marine: Option<f64>,

    #[serde(alias = "ecotoxicity: terrestrial|terrestrial ecotoxicity (TETP inf)")]
    pub ecotoxicity_terrestrial: Option<f64>,

    #[serde(alias = "photochemical oxidant formation|photochemical oxidation (high NOx)")]
    pub photochemical_oxidant_formation: Option<f64>,

    #[serde(alias = "acidification|acidification (incl. fate, average Europe total, A&B)")]
    pub acidification: Option<f64>,

    #[serde(alias = "eutrophication|eutrophication (fate not incl.)")]
    pub eutrophication: Option<f64>,
}

#[rustfmt::skip]
impl CMLImpacts {
  
  fn add_triplets(&self, a: &mut MappedMatrixBuilder<ImpactCategory, String>, col: String) {
        // Order is important
        a.add_triplet(ImpactCategory::CML(CML::Gwp100), col.clone(), self.gwp100.unwrap_or(0.));
        a.add_triplet(ImpactCategory::CML(CML::ResourcesMetalsMinerals), col.clone(), self.resources_metals_minerals.unwrap_or(0.));
        a.add_triplet(ImpactCategory::CML(CML::EnergyResourcesNonRenewable), col.clone(), self.energy_resources_non_renewable.unwrap_or(0.));
        a.add_triplet(ImpactCategory::CML(CML::OzoneDepletion), col.clone(), self.ozone_depletion.unwrap_or(0.));
        a.add_triplet(ImpactCategory::CML(CML::HumanToxicity), col.clone(), self.human_toxicity.unwrap_or(0.));
        a.add_triplet(ImpactCategory::CML(CML::EcotoxicityFreshwater), col.clone(), self.ecotoxicity_freshwater.unwrap_or(0.));
        a.add_triplet(ImpactCategory::CML(CML::EcotoxicityMarine), col.clone(), self.ecotoxicity_marine.unwrap_or(0.));
        a.add_triplet(ImpactCategory::CML(CML::EcotoxicityTerrestrial), col.clone(), self.ecotoxicity_terrestrial.unwrap_or(0.));
        a.add_triplet(ImpactCategory::CML(CML::PhotochemicalOxidantFormation), col.clone(), self.photochemical_oxidant_formation.unwrap_or(0.));
        a.add_triplet(ImpactCategory::CML(CML::Acidification), col.clone(), self.acidification.unwrap_or(0.));
        a.add_triplet(ImpactCategory::CML(CML::Eutrophication), col.clone(), self.eutrophication.unwrap_or(0.));
    }
}

pub fn get_cml_matrix(
    version: &str,
    intervention: &MappedMatrix<String, String>,
) -> Result<MappedMatrix<ImpactCategory, String>> {
    let mut mat = get_empty_matrix(CML::get_empty_vector(), intervention);
    let mut rdr = get_ecoinvent_mapping_file(version, "CML v4.8 2016")?;
    for result in rdr.deserialize() {
        let record: CMLImpacts = result?;
        let elementary_id = record.flow_id.clone();
        if intervention.contains_row(&elementary_id) {
            record.add_triplets(&mut mat, elementary_id);
        }
    }
    Ok(mat.build())
}
