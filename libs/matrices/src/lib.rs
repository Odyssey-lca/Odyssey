mod cs;
mod csn;
mod css;
mod error;
mod mapped_matrix;
mod mapped_vector;

#[allow(warnings, clippy::all)]
mod suitesparse {
  include!(concat!(env!("OUT_DIR"), "/suitesparse.rs"));
}

pub use self::error::Error;
pub use self::mapped_matrix::{MappedMatrix, MappedMatrixBuilder};
pub use self::mapped_vector::MappedVector;
