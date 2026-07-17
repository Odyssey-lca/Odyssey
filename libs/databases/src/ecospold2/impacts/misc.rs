use errors::Result;
use crate::impacts::ImpactCategory;
use csv::Reader;
use matrices::{MappedMatrix, MappedMatrixBuilder, MappedVector};
use std::{fs::File, path::Path};

pub fn get_empty_matrix(
    empty_vector: MappedVector<ImpactCategory>,
    intervention: &MappedMatrix<String, String>,
) -> MappedMatrixBuilder<ImpactCategory, String> {
    let mut mat = MappedMatrixBuilder::new();
    mat.copy_rows_into_cols(intervention);
    mat.copy_vec_into_rows(&empty_vector);
    mat
}

pub fn get_ecoinvent_mapping_file(version: &str, method: &str, cache_path: &Path) -> Result<Reader<File>> {
    let file = File::open(
        cache_path
            .join("ecoinvent_lcia")
            .join(format!("{}/methods_mapped", version))
            .join(format!("{}_mapped_{}.csv", method, version)),
    )?;
    Ok(csv::Reader::from_reader(file))
}
