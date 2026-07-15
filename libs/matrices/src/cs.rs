use serde::{Deserialize, Serialize};

use super::suitesparse::cs_di;
use crate::error::Error;

/// Sparse immutable matrix in Compressed Sparse Column (CSC) format.
///
/// This is a safe Rust wrapper around the SuiteSparse / CXSparse `cs_di`
/// structure. It owns its data and can be serialized/deserialized.
#[derive(Serialize, Deserialize, Debug)]
pub struct Cs {
    /// Maximum number of nonzero entries (allocated capacity)
    pub nzmax: usize,

    /// Number of rows
    pub m: usize,

    /// Number of columns
    pub n: usize,

    /// Column pointers (length `n + 1`)
    /// `p[j]..p[j+1]` gives the range of nonzeros in column `j`
    pub p: Vec<i32>,

    /// Row indices for each nonzero (length `nzmax`)
    pub i: Vec<i32>,

    /// Numerical values for each nonzero (length `nzmax`)
    pub x: Vec<f64>,
}

impl Cs {
    /// Creates a new sparse matrix in CSC format.
    /// Only basics checks are performed. The caller is responsible
    /// for providing a valid CSC representation.
    pub fn new(m: usize, n: usize, p: Vec<i32>, i: Vec<i32>, x: Vec<f64>) -> Result<Self, Error> {
        if p.len() != n + 1 {
            return Err(Error::InvalidInput {
                details: format!("p has length {}, expected {}", p.len(), n + 1),
            });
        }
        if i.len() != x.len() {
            return Err(Error::InvalidInput {
                details: format!("i has length {}, expected {}", i.len(), x.len()),
            });
        }

        Ok(Cs {
            nzmax: x.len(),
            m,
            n,
            p,
            i,
            x,
        })
    }

    /// Creates a mutable FFI view of this matrix as a `cs_di`.
    ///
    /// The returned struct contains **raw pointers** into the internal
    /// vectors owned by `self`. No allocation or copying is performed.
    ///
    /// # Safety
    ///
    /// - The returned `cs_di` must not outlive `self`
    /// - The vectors `p`, `i`, and `x` must not be reallocated while the
    ///   FFI struct is in use
    /// - The matrix is assumed to be in CSC form (`nz = -1`)
    pub fn as_ffi(&mut self) -> cs_di {
        cs_di {
            m: self.m as i32,
            n: self.n as i32,
            nz: -1i32,
            p: self.p.as_mut_ptr(),
            i: self.i.as_mut_ptr(),
            x: self.x.as_mut_ptr(),
            nzmax: self.nzmax as i32,
        }
    }

    /// Creates a const FFI view of this matrix as a `cs_di`.
    ///
    /// The caller must ensure the C function receiving this view
    /// does not mutate through the pointers.
    ///
    /// # Safety
    ///
    /// - The returned `cs_di` must not outlive `self`
    /// - The vectors `p`, `i`, and `x` must not be reallocated while the
    ///   FFI struct is in use
    /// - The matrix is assumed to be in CSC form (`nz = -1`)
    pub fn as_ffi_const(&self) -> cs_di {
        cs_di {
            m: self.m as i32,
            n: self.n as i32,
            nz: -1i32,
            p: self.p.as_ptr() as *mut i32,
            i: self.i.as_ptr() as *mut i32,
            x: self.x.as_ptr() as *mut f64,
            nzmax: self.nzmax as i32,
        }
    }

    /// Takes ownership of a `cs_di` allocated by SuiteSparse / CSparse
    /// and converts it into a safe Rust `Cs`.
    ///
    /// This function assumes that the memory referenced by `ffi`
    /// was allocated using the C allocator and is exclusively owned
    /// by the caller.
    ///
    /// # Safety
    ///
    /// - `ffi` must be a valid, non-null pointer
    /// - `ffi->p`, `ffi->i`, and `ffi->x` must point to valid allocations
    /// - The memory must not be freed elsewhere after this call
    /// - After calling this function, the Rust `Cs` takes ownership
    ///   of the underlying buffers
    ///
    /// Violating these conditions results in undefined behavior.
    pub unsafe fn from_ffi(ffi: *mut cs_di) -> Self {
        unsafe {
            Self {
                m: (*ffi).m as usize,
                n: (*ffi).n as usize,
                p: Vec::from_raw_parts((*ffi).p, (*ffi).n as usize + 1, (*ffi).n as usize + 1),
                i: Vec::from_raw_parts((*ffi).i, (*ffi).nzmax as usize, (*ffi).nzmax as usize),
                x: Vec::from_raw_parts((*ffi).x, (*ffi).nzmax as usize, (*ffi).nzmax as usize),
                nzmax: (*ffi).nzmax as usize,
            }
        }
    }
}
