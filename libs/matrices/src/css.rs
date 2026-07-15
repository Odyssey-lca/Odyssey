use serde::{Deserialize, Serialize};

use super::suitesparse::{cs_dis, css_init};

use super::cs::Cs;
use crate::error::Error;

/// Symbolic analysis of a sparse matrix factorization.
///
/// This struct represents the **symbolic phase** of a sparse
/// factorization as performed by SuiteSparse / CSparse.
///
/// It contains **structural information only** (no numerical values),
/// such as permutations and fill-in estimates. The symbolic analysis
/// can be reused to factorize multiple matrices that share the same
/// sparsity pattern.
///
/// This is a partial, Rust-owned representation of the `cs_dis`
/// structure used internally by CSparse.
#[derive(Serialize, Deserialize, Debug)]
pub struct Css {
    /// Column permutation applied before factorization.
    ///
    /// This reordering reduces fill-in and improves numerical stability.
    /// The permutation is of length `n`, where `n` is the number of columns.
    pub q: Vec<i32>,

    /// Estimated number of nonzeros in the `L` factor.
    ///
    /// This value is computed during symbolic analysis and is used
    /// to allocate exact storage for numeric factorization.
    pub lnz: f64,

    /// Estimated number of nonzeros in the `U` factor.
    ///
    /// Like `lnz`, this is a structural estimate independent of
    /// the actual numerical values of the matrix.
    pub unz: f64,
}

impl Css {
    /// Performs symbolic analysis on a sparse matrix.
    ///
    /// This function calls into SuiteSparse / CSparse to compute
    /// permutations and fill-in estimates for the given matrix.
    ///
    /// Returns [Err] if symbolic analysis fails (e.g. due to
    /// allocation failure).
    ///
    /// # Safety
    ///
    /// This function temporarily creates an FFI view of `cs`
    /// and passes raw pointers to the C API. The matrix must not
    /// be modified or reallocated during the call.
    pub fn new(cs: &mut Cs) -> Result<Self, Error> {
        unsafe {
            let css = css_init(&cs.as_ffi());
            if css.is_null() {
                Err(Error::SymbolicAnalysisFailed)
            } else {
                Ok(Css::from_ffi(css, cs.n))
            }
        }
    }

    /// Creates a mutable FFI view of this symbolic analysis.
    ///
    /// Only the fields required by the numeric factorization
    /// routines are populated. All other pointers are set to null.
    ///
    /// # Safety
    ///
    /// The returned `cs_dis` contains raw pointers into `self`
    /// and must not outlive it.
    pub fn as_ffi(&mut self) -> cs_dis {
        cs_dis {
            pinv: std::ptr::null_mut(),
            q: self.q.as_mut_ptr(),
            parent: std::ptr::null_mut(),
            cp: std::ptr::null_mut(),
            leftmost: std::ptr::null_mut(),
            m2: 0i32,
            lnz: self.lnz,
            unz: self.unz,
        }
    }

    /// Takes ownership of a `cs_dis` returned by CSparse and
    /// converts it into a safe Rust [Css].
    ///
    /// # Safety
    ///
    /// - `ffi` must be a valid pointer returned by CSparse
    /// - `ffi->q` must point to an allocation of length `n`
    /// - The memory referenced by `q` must not be freed elsewhere
    ///
    /// After this call, ownership of the permutation vector
    /// is transferred to Rust.
    unsafe fn from_ffi(ffi: *mut cs_dis, n: usize) -> Self {
        unsafe {
            let result = Self {
                q: Vec::from_raw_parts((*ffi).q, n, n),
                lnz: (*ffi).lnz,
                unz: (*ffi).unz,
            };
            libc::free(ffi as *mut libc::c_void);
            result
        }
    }
}
