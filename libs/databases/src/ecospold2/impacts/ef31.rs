use std::path::Path;

use matrices::{MappedMatrix, MappedMatrixBuilder};
use comput::impacts::{ef31::EF31, ImpactCategory};
use errors::Result;
use super::misc::{get_ecoinvent_mapping_file, get_empty_matrix};

#[rustfmt::skip]
#[derive(Debug, serde::Deserialize)]
pub struct EF31Impacts {
    #[serde(alias = "elementary_flow_id")]
    pub flow_id: String,

    #[serde(alias = "climate change|global warming potential (GWP100)")]
    pub gwp100: Option<f64>,

    #[serde(alias = "acidification|accumulated exceedance (AE)")]
    pub acidification_ae: Option<f64>,

    #[serde(alias = "climate change: biogenic|global warming potential (GWP100)")]
    pub biogenic_gwp100: Option<f64>,

    #[serde(alias = "climate change: fossil|global warming potential (GWP100)")]
    pub fossil_gwp100: Option<f64>,
    
    #[serde(alias = "climate change: land use and land use change|global warming potential (GWP100)")]
    pub climate_change_land_use: Option<f64>,
    
    #[serde(alias = "particulate matter formation|impact on human health")]
    pub particul_matter: Option<f64>,
    
    #[serde(alias = "ecotoxicity: freshwater|comparative toxic unit for ecosystems (CTUe)")]
    pub ecotoxicity_freshwater: Option<f64>,

    #[serde(alias = "ecotoxicity: freshwater, inorganics|comparative toxic unit for ecosystems (CTUe)")]
    pub ecotoxicity_freshwater_inorganics: Option<f64>,

    #[serde(alias = "ecotoxicity: freshwater, organics|comparative toxic unit for ecosystems (CTUe)")]
    pub ecotoxicity_freshwater_organics: Option<f64>,
    
    #[serde(alias = "eutrophication: marine|fraction of nutrients reaching marine end compartment (N)")]
    pub eutrophication_marine: Option<f64>,

    #[serde(alias = "eutrophication: freshwater|fraction of nutrients reaching freshwater end compartment (P)")]
    pub eutrophication_freshwater: Option<f64>,

    #[serde(alias = "eutrophication: terrestrial|accumulated exceedance (AE)")]
    pub eutrophication_terrestrial: Option<f64>,

    #[serde(alias = "human toxicity: carcinogenic|comparative toxic unit for human (CTUh)")]
    pub human_toxicity_carcinogenic: Option<f64>,

    #[serde(alias = "human toxicity: carcinogenic, inorganics|comparative toxic unit for human (CTUh)")]
    pub human_toxicity_carcinogenic_inorganics: Option<f64>,

    #[serde(alias = "human toxicity: carcinogenic, organics|comparative toxic unit for human (CTUh)")]
    pub human_toxicity_carcinogenic_organics: Option<f64>,

    #[serde(alias = "human toxicity: non-carcinogenic|comparative toxic unit for human (CTUh)")]
    pub human_toxicity_non_cacrinogenic: Option<f64>,
    
    #[serde(alias = "human toxicity: non-carcinogenic, inorganics|comparative toxic unit for human (CTUh)")]
    pub human_toxicity_non_cacinogenic_inorganics: Option<f64>,
    
    #[serde(alias = "human toxicity: non-carcinogenic, organics|comparative toxic unit for human (CTUh)")]
    pub human_toxicity_non_cacinogenic_organics: Option<f64>,
    
    #[serde(alias = "ionising radiation: human health|human exposure efficiency relative to u235")]
    pub ionising_radiation: Option<f64>,

    #[serde(alias = "land use|soil quality index")]
    pub land_use: Option<f64>,

    #[serde(alias= "ozone depletion|ozone depletion potential (ODP)")]
    pub ozone_depletion: Option<f64>,
    
    #[serde(alias = "photochemical oxidant formation: human health|tropospheric ozone concentration increase")]
    pub photochemical_oxidant: Option<f64>,
    
    #[serde(alias = "energy resources: non-renewable|abiotic depletion potential (ADP): fossil fuels")]
    pub energy_resources_non_renewable: Option<f64>,
    
    #[serde(alias = "material resources: metals/minerals|abiotic depletion potential (ADP): elements (ultimate reserves)")]
    pub resources_metals_minerals: Option<f64>,
    
    #[serde(alias = "water use|user deprivation potential (deprivation-weighted water consumption)")]
    pub water_use: Option<f64>,
}

#[rustfmt::skip]
impl EF31Impacts {

  fn add_triplets(&self, a: &mut MappedMatrixBuilder<ImpactCategory, String>, col: String) {
        // Order is important
        a.add_triplet(ImpactCategory::EF31(EF31::Gwp100), col.clone(), self.gwp100.unwrap_or(0.));
        a.add_triplet(ImpactCategory::EF31(EF31::Acidification), col.clone(), self.acidification_ae.unwrap_or(0.));
        a.add_triplet(ImpactCategory::EF31(EF31::BiogenicGwp100), col.clone(), self.biogenic_gwp100.unwrap_or(0.));
        a.add_triplet(ImpactCategory::EF31(EF31::FossilGwp100), col.clone(), self.fossil_gwp100.unwrap_or(0.));
        a.add_triplet(ImpactCategory::EF31(EF31::ClimateChangeLandUse), col.clone(), self.climate_change_land_use.unwrap_or(0.));
        a.add_triplet(ImpactCategory::EF31(EF31::OzoneDepletion), col.clone(), self.ozone_depletion.unwrap_or(0.));
        a.add_triplet(ImpactCategory::EF31(EF31::ParticulMatter), col.clone(), self.particul_matter.unwrap_or(0.));
        a.add_triplet(ImpactCategory::EF31(EF31::EcotoxicityFreshwater), col.clone(), self.ecotoxicity_freshwater.unwrap_or(0.));
        a.add_triplet(ImpactCategory::EF31(EF31::EcotoxicityFreshwaterInorganics), col.clone(), self.ecotoxicity_freshwater_inorganics.unwrap_or(0.));
        a.add_triplet(ImpactCategory::EF31(EF31::EcotoxicityFreshwaterOrganics), col.clone(), self.ecotoxicity_freshwater_organics.unwrap_or(0.));
        a.add_triplet(ImpactCategory::EF31(EF31::EutrophicationMarine), col.clone(), self.eutrophication_marine.unwrap_or(0.));
        a.add_triplet(ImpactCategory::EF31(EF31::EutrophicationFreshwater), col.clone(), self.eutrophication_freshwater.unwrap_or(0.));
        a.add_triplet(ImpactCategory::EF31(EF31::EutrophicationTerrestrial), col.clone(), self.eutrophication_terrestrial.unwrap_or(0.));
        a.add_triplet(ImpactCategory::EF31(EF31::HumanToxicityCarcinogenic), col.clone(), self.human_toxicity_carcinogenic.unwrap_or(0.));
        a.add_triplet(ImpactCategory::EF31(EF31::HumanToxicityCarcinogenicInorganics), col.clone(), self.human_toxicity_carcinogenic_inorganics.unwrap_or(0.));
        a.add_triplet(ImpactCategory::EF31(EF31::HumanToxicityCarcinogenicOrganics), col.clone(), self.human_toxicity_carcinogenic_organics.unwrap_or(0.));
        a.add_triplet(ImpactCategory::EF31(EF31::HumanToxicityNonCacrinogenic), col.clone(), self.human_toxicity_non_cacrinogenic.unwrap_or(0.));
        a.add_triplet(ImpactCategory::EF31(EF31::HumanToxicityNonCacinogenicInorganics), col.clone(), self.human_toxicity_non_cacinogenic_inorganics.unwrap_or(0.));
        a.add_triplet(ImpactCategory::EF31(EF31::HumanToxicityNonCacinogenicOrganics), col.clone(), self.human_toxicity_non_cacinogenic_organics.unwrap_or(0.));
        a.add_triplet(ImpactCategory::EF31(EF31::IonisingRadiation), col.clone(), self.ionising_radiation.unwrap_or(0.));
        a.add_triplet(ImpactCategory::EF31(EF31::LandUse), col.clone(), self.land_use.unwrap_or(0.));
        a.add_triplet(ImpactCategory::EF31(EF31::PhotochemicalOxidant), col.clone(), self.photochemical_oxidant.unwrap_or(0.));
        a.add_triplet(ImpactCategory::EF31(EF31::EnergyResourcesNonRenewable), col.clone(), self.energy_resources_non_renewable.unwrap_or(0.));
        a.add_triplet(ImpactCategory::EF31(EF31::ResourcesMetalsMinerals), col.clone(), self.resources_metals_minerals.unwrap_or(0.));
        a.add_triplet(ImpactCategory::EF31(EF31::WaterUse), col.clone(), self.water_use.unwrap_or(0.));
    }
}

pub fn get_ef31_matrix(
    version: &str,
    intervention: &MappedMatrix<String, String>,
    cache_path: &Path,
) -> Result<MappedMatrix<ImpactCategory, String>> {
    let mut mat = get_empty_matrix(EF31::get_empty_vector(), intervention);
    let mut rdr = get_ecoinvent_mapping_file(version, "EF v3.1", cache_path)?;
    for result in rdr.deserialize() {
        let record: EF31Impacts = result?;
        let elementary_id = record.flow_id.clone();
        if intervention.contains_row(&elementary_id) {
            record.add_triplets(&mut mat, elementary_id);
        }
    }
    Ok(mat.build())
}
