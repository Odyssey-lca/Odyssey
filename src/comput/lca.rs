use crate::{
    comput::impacts::ImpactCategory,
    errors::Result,
    utils::{matrix::MappedVector, search::InventoryItem},
};

pub trait Database {
    /// Name of the database
    fn name(&self) -> String;

    fn list_candidates(&self) -> Vec<&InventoryItem>;

    fn find_candidate(&self, id: &str) -> Option<&InventoryItem>;

    /// Performs the inventory for the items specified in the reference flow `f`.
    fn lci(&mut self, f: &MappedVector<String>) -> Result<MappedVector<String>>;

    /// Performs the impact assessment given the supply vector `s`.
    fn lcia(
        &mut self,
        s: &MappedVector<String>,
        method: &str,
    ) -> Result<MappedVector<ImpactCategory>>;

    fn empty_reference_flow(&self) -> MappedVector<String>;
    fn empty_impacts(&self, method: &str) -> MappedVector<ImpactCategory>;

    /// Performs the life cycle assessment of the items specified in the reference flow `f`.
    /// This function is equivalent to performing `lci` followed by `lcia`.
    fn lca(
        &mut self,
        f: &MappedVector<String>,
        method: &str,
    ) -> Result<MappedVector<ImpactCategory>> {
        let s = self.lci(f)?;
        self.lcia(&s, method)
    }
}
