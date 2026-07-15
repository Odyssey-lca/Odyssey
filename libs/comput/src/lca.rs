use matrices::MappedVector;
use search::InventoryItem;
use super::impacts::ImpactCategory;
use errors::Result;

pub trait Database {
    /// Name of the database
    fn name(&self) -> String;

    fn list_candidates(&self) -> Vec<&InventoryItem>;

    fn get_candidate(&self, id: &str) -> Option<&InventoryItem>;

    fn empty_reference_flow(&self) -> MappedVector<String>;
    fn empty_impacts(&self, method: &str) -> MappedVector<ImpactCategory>;

    /// Performs the life cycle assessment of the items specified in the reference flow `f`.
    fn lca(
        &mut self,
        f: &MappedVector<String>,
        method: &str,
    ) -> Result<MappedVector<ImpactCategory>>;
}
