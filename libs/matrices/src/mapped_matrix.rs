use super::{
    cs::Cs,
    suitesparse::{cs_din, csparse_matvec, csparse_solve},
};
use crate::{MappedVector, csn::Csn, css::Css, error::Error, suitesparse::csparse_matmat};
use bimap::{BiHashMap, BiMap};
use serde::{Deserialize, Serialize};
use sprs::{CsMat, TriMat};
use std::{collections::HashMap, hash::Hash, sync::Arc};

/// Builder for [MappedMatrix].
///
/// ## Example
/// ```
/// use matrices::{MappedMatrixBuilder, MappedMatrix};
/// let mut builder = MappedMatrixBuilder::new();
/// builder.add_triplet("a", "c",  1.0);
/// builder.add_triplet("a", "d",  2.0);
/// builder.add_triplet("b", "c", -0.1);
/// builder.add_triplet("b", "d",  3.0);
/// let mm: MappedMatrix<&str, &str> = builder.build();
/// ```
///
#[derive(Debug, Clone, Default)]
pub struct MappedMatrixBuilder<R, C>
where
    R: std::cmp::Eq + Hash,
    C: std::cmp::Eq + Hash,
{
    rows: BiMap<R, usize>,
    cols: BiMap<C, usize>,
    triplets: HashMap<(usize, usize), f64>,
}

impl<R, C> MappedMatrixBuilder<R, C>
where
    R: std::cmp::Eq + Hash + Clone,
    C: std::cmp::Eq + Hash + Clone,
{
    pub fn new() -> Self {
        MappedMatrixBuilder {
            rows: BiMap::new(),
            cols: BiMap::new(),
            triplets: HashMap::new(),
        }
    }

    pub fn copy_rows_into_cols<T>(&mut self, copied: &MappedMatrix<C, T>)
    where
        T: std::cmp::Eq + Hash + Clone,
    {
        self.cols = (*copied.rows).clone();
    }

    pub fn copy_vec_into_rows(&mut self, copied: &MappedVector<R>) {
        self.rows = (*copied.mapping).clone();
    }

    pub fn row(&self, id: &R) -> Option<&usize> {
        self.rows.get_by_left(id)
    }

    pub fn col(&self, id: &C) -> Option<&usize> {
        self.cols.get_by_left(id)
    }

    pub fn add_col(&mut self, id: C) {
        if self.col(&id).is_none() {
            self.cols.insert(id, self.cols.len());
        }
    }

    pub fn add_row(&mut self, id: R) {
        if self.row(&id).is_none() {
            self.rows.insert(id, self.rows.len());
        }
    }

    fn triplets_to_csc(self) -> Cs {
        // Create a COO (triplet) matrix. This is not done before since sizes are required.
        let mut triplet_mat =
            TriMat::<f64>::with_capacity((self.rows.len(), self.cols.len()), self.triplets.len());
        for ((r, c), v) in self.triplets {
            triplet_mat.add_triplet(r, c, v);
        }

        // Convert COO -> CSC format
        let csc_mat: CsMat<f64> = triplet_mat.to_csc();

        let p: Vec<i32> = csc_mat
            .indptr()
            .as_slice()
            .unwrap()
            .iter()
            .map(|&x| x as i32)
            .collect();
        let i: Vec<i32> = csc_mat.indices().iter().map(|&x| x as i32).collect();
        let x: Vec<f64> = csc_mat.data().to_vec();

        let m = csc_mat.rows();
        let n = csc_mat.cols();

        Cs::new(m, n, p, i, x).unwrap()
    }

    /// Number of rows in the mapped matrix.
    pub fn nrows(&self) -> usize {
        self.rows.len()
    }

    /// Number of columns in the mapped matrix.
    pub fn ncols(&self) -> usize {
        self.cols.len()
    }

    pub fn add_triplet(&mut self, row: R, col: C, value: f64) {
        let row_index = if let Some(index) = self.row(&row) {
            *index
        } else {
            self.rows.insert(row, self.rows.len());
            self.rows.len() - 1
        };
        let col_index = if let Some(index) = self.col(&col) {
            *index
        } else {
            self.cols.insert(col, self.cols.len());
            self.cols.len() - 1
        };
        if let Some(a) = self.triplets.get_mut(&(row_index, col_index)) {
            *a += value;
        } else {
            self.triplets.insert((row_index, col_index), value);
        }
    }

    pub fn build(self) -> MappedMatrix<R, C> {
        let cols = self.cols.clone();
        let rows = self.rows.clone();
        let mut cs = self.triplets_to_csc();
        let (css, csn) = match Css::new(&mut cs) {
            Ok(mut css) => {
                let csn = Csn::new(&mut cs, &mut css).ok();
                (Some(css), csn)
            }
            Err(_) => (None, None),
        };
        MappedMatrix {
            rows: Arc::new(rows),
            cols: Arc::new(cols),
            cs,
            css,
            csn,
        }
    }
}

/// 2D matrix with values maped to each rows and columns.
#[derive(Serialize, Deserialize, Debug)]
pub struct MappedMatrix<R, C>
where
    R: std::cmp::Eq + Hash + Clone,
    C: std::cmp::Eq + Hash + Clone,
{
    rows: Arc<BiHashMap<R, usize>>,
    cols: Arc<BiHashMap<C, usize>>,
    cs: Cs,
    css: Option<Css>,
    csn: Option<Csn>,
}

impl<R, C> MappedMatrix<R, C>
where
    R: std::cmp::Eq + Hash + Clone,
    C: std::cmp::Eq + Hash + Clone,
{
    pub fn new(
        rows: Arc<BiHashMap<R, usize>>,
        cols: Arc<BiHashMap<C, usize>>,
        cs: Cs,
        css: Option<Css>,
        csn: Option<Csn>,
    ) -> Self {
        Self {
            rows,
            cols,
            cs,
            css,
            csn,
        }
    }

    /// Returns the index of the row corresponding to `id`.
    /// This returns [None] if `id` has no corresponding row.
    pub fn row(&self, id: &R) -> Option<&usize> {
        self.rows.get_by_left(id)
    }

    /// Returns the index of the column corresponding to `id`.
    /// This returns [None] if `id` has no corresponding column.
    pub fn col(&self, id: &C) -> Option<&usize> {
        self.cols.get_by_left(id)
    }

    /// Returns the value mapped to the row `index`.
    /// This returns [None] if `index` is out of range, or
    /// if the row was not mapped to any value.
    pub fn irow(&self, index: &usize) -> Option<&R> {
        self.rows.get_by_right(index)
    }

    /// Returns the value mapped to the column `index`.
    /// This returns [None] if `index` is out of range, or
    /// if the column was not mapped to any value.
    pub fn icol(&self, index: &usize) -> Option<&C> {
        self.cols.get_by_right(index)
    }

    /// Number of rows in the mapped matrix.
    pub fn nrows(&self) -> usize {
        self.rows.len()
    }

    /// Number of columns in the mapped matrix.
    pub fn ncols(&self) -> usize {
        self.cols.len()
    }

    /// Copy of the columns matrix in a zero valued vector.
    pub fn zeros_like_cols(&self) -> MappedVector<C> {
        MappedVector::new(self.cols.clone(), vec![0.0; self.ncols()])
    }

    /// Copy of the rows matrix in a zero valued vector.
    pub fn zeros_like_rows(&self) -> MappedVector<R> {
        MappedVector::new(self.rows.clone(), vec![0.0; self.nrows()])
    }

    /// Test weither any row was mapped to `id`.
    pub fn contains_row(&self, id: &R) -> bool {
        self.rows.contains_left(id)
    }

    /// Test weither any column was mapped to `id`.
    pub fn contains_col(&self, id: &C) -> bool {
        self.cols.contains_left(id)
    }

    /// Solve the system `Ax = b`, where `A` is a `MappedMatrix` and `b` a known `Vec<f64>`.
    /// The returned value is `x`.
    ///
    /// # Example
    /// ```
    /// # use matrices::{MM, MV, MappedMatrixBuilder, MappedMatrix};
    /// let mut A = MM!["a" => { "c" =>  1.0, "d" => 2.0 },
    ///                 "b" => { "c" => -0.1, "d" => 3.0 }];
    /// let b = MV!["a" => 10.0, "b" => 5.0];
    /// let x = A.solve(&b).unwrap();
    /// assert!(x == MV!["c" => 6.25, "d" => 1.875]);
    /// ```
    pub fn solve(&mut self, rhs: &MappedVector<R>) -> Result<MappedVector<C>, Error> {
        if self.css.is_none() || self.csn.is_none() {
            return Err(Error::NotInvertible);
        }
        if rhs.values.len() != self.rows.len() {
            return Err(Error::DimensionMismatch {
                expected: self.rows.len(),
                actual: rhs.values.len(),
            });
        }
        let mut res = vec![0f64; self.cols.len()];

        let css = self.css.as_mut().unwrap();
        let css = css.as_ffi();

        let csn = self.csn.as_mut().unwrap();
        let mut l = csn.l.as_ffi();
        let mut u = csn.u.as_ffi();
        let csn = cs_din {
            L: &mut l as *mut _,
            U: &mut u as *mut _,
            pinv: csn.pinv.as_mut_ptr(),
            B: std::ptr::null_mut(),
        };

        unsafe {
            csparse_solve(
                &css,
                &csn,
                self.cs.n as i32,
                rhs.values.as_ptr(),
                res.as_mut_ptr(),
            );
        }

        Ok(MappedVector::new(self.cols.clone(), res))
    }

    /// Multiplies a `MappedMatrix` with a `Vec<f64>`.
    ///
    /// # Example
    /// ```
    /// # use matrices::{MM, MV, MappedMatrixBuilder, MappedMatrix, MappedVector};
    /// # use std::ops::Add;
    /// let A = MM!["a" => { "c" =>  1.0, "d" => 2.0 },
    ///             "b" => { "c" => -0.1, "d" => 3.0 }];
    /// let b = MV!["c" => 6.25, "d" => 1.875];
    /// let x = A.dot(&b);
    /// assert!(x == MV!["a" => 10.0, "b" => 5.]);
    /// ```
    pub fn dot(&self, rhs: &MappedVector<C>) -> MappedVector<R> {
        let mut res = vec![0f64; self.rows.len()];
        unsafe {
            csparse_matvec(&self.cs.as_ffi_const(), rhs.values.as_ptr(), res.as_mut_ptr());
        }
        MappedVector::new(self.rows.clone(), res)
    }

    /// Multiplies two `MappedMatrix`.
    /// The resulting matrix can't be used to solve systems
    pub fn quick_mat_mul<R2, C2>(
        &self,
        rhs: &MappedMatrix<R2, C2>,
    ) -> Result<MappedMatrix<R, C2>, Error>
    where
        R2: std::cmp::Eq + Hash + Clone,
        C2: std::cmp::Eq + Hash + Clone,
    {
        let cs = unsafe {
            let ptr = csparse_matmat(&self.cs.as_ffi_const(), &rhs.cs.as_ffi_const());
            if ptr.is_null() {
                return Err(Error::MatrixMulFailed);
            }
            Cs::from_ffi(ptr)
        };
        Ok(MappedMatrix {
            rows: self.rows.clone(),
            cols: rhs.cols.clone(),
            cs,
            css: None,
            csn: None,
        })
    }
}

#[macro_export]
macro_rules! MM {
    (
        $(
            $row_label:expr => {
                $(
                    $col_label:expr => $val:expr
                ),* $(,)?
            }
        ),* $(,)?
    ) => {{
        let mut builder = $crate::MappedMatrixBuilder::new();
        $(
            $(
                builder.add_triplet($row_label, $col_label, $val);
            )*
        )*
        builder.build()
    }};
}

#[rustfmt::skip]
#[cfg(test)]
mod tests {

    use super::*;
    use crate::MV;

    ///
    ///     ----------------- A ----------------       -- B --
    ///     |  1.0    0.0    0.0    0.0    0.0 |      | 50.0 |
    ///     | -0.02   1.0    0.0    0.0    0.0 |      |  0.0 |
    ///     | -0.2    0.0    1.0    0.0    0.0 |      |  0.0 |
    ///     | -0.2    0.0    1.0    0.0    0.0 |      |  0.0 |
    ///     | -0.2    0.0    1.0    0.0    0.0 |      |  0.0 |
    ///
    /// Compute x where Ax = b
    #[test]
    fn test_solve() {

      let mut a = MappedMatrixBuilder::new();
      a.add_triplet("0", "0", 1.0);
      a.add_triplet("1", "0", -0.02);
      a.add_triplet("2", "0", -0.2);
      a.add_triplet("3", "0", -0.1);
      a.add_triplet("1", "1", 1.0);
      a.add_triplet("3", "1", -1.0);
      a.add_triplet("4", "1", -10.0);
      a.add_triplet("2", "2", 1.0);
      a.add_triplet("3", "2", -1.0);
      a.add_triplet("4", "2", -2.0);
      a.add_triplet("3", "3", 1.0);
      a.add_triplet("4", "3", -1.0);
      a.add_triplet("3", "4", -0.1);
      a.add_triplet("4", "4", 1.);
      let mut a = a.build();

      let mut b = a.zeros_like_rows();
      b.set("0", 50.);
      b.set("1", 0.);
      b.set("2", 0.);
      b.set("3", 0.);
      b.set("4", 0.);
      let x = a.solve(&b).unwrap();
      assert!(x.values.iter().zip(vec![50.0, 1.0, 10.0, 21.11111, 51.11111]).all(|(a,b)| (a-b).abs() < 1e-5));
    }

    ///  ---------------- A ----------------       -- b --
    ///  |  0.0   10.0    0.0    0.5    0.0 |      | 50.0 |
    ///  |  0.0   -1.5    0.0    0.0    0.0 |      |  1.0 |
    ///  |  0.0    0.0    1.0    0.0   -1.2 |      | 10.0 |
    ///                                            | 21.1 |
    ///                                            | 51.1 |
    ///
    /// Compute the product Ab
    #[test]
    fn test_mul() {

      let mut builder = MappedMatrixBuilder::new();
      builder.add_col("0");
      builder.add_row("0");
      builder.add_triplet("0", "1", 10.0);
      builder.add_triplet("1", "1", -1.5);
      builder.add_col("2");
      builder.add_triplet("0", "3", 0.5);
      builder.add_triplet("2", "4", -1.2);
      let b = builder.build();
      let mut rhs = b.zeros_like_cols();
      rhs.set("0", 50.);
      rhs.set("1", 1.);
      rhs.set("2", 10.);
      rhs.set("3", 21.111);
      rhs.set("4", 51.111);
      let res = &mut b.dot(&rhs);
      assert!(res.values.iter().zip(vec![20.5555, -1.5, -61.3332]).all(|(a,b)| (a-b).abs() < 1e-5));
    }

    /// A = [[1, 2], [3, 4]]  rows: a,b  cols: c,d
    /// B = [[5, 6], [7, 8]]  rows: c,d  cols: e,f
    /// A * B = [[19, 22], [43, 50]]
    #[test]
    fn test_mat_mul() {
        let a = MM!["a" => { "c" => 1.0, "d" => 2.0 },
                    "b" => { "c" => 3.0, "d" => 4.0 }];
        let b = MM!["c" => { "e" => 5.0, "f" => 6.0 },
                    "d" => { "e" => 7.0, "f" => 8.0 }];
        let c = a.quick_mat_mul(&b).unwrap();

        assert_eq!(c.nrows(), 2);
        assert_eq!(c.ncols(), 2);

        let col_e = MV!["e" => 1.0, "f" => 0.0];
        let col_f = MV!["e" => 0.0, "f" => 1.0];
        let res_e = c.dot(&col_e);
        let res_f = c.dot(&col_f);
        let ia = res_e.row(&"a").unwrap();
        let ib = res_e.row(&"b").unwrap();
        assert!((res_e.values[*ia] - 19.0).abs() < 1e-10);
        assert!((res_e.values[*ib] - 43.0).abs() < 1e-10);
        let ia = res_f.row(&"a").unwrap();
        let ib = res_f.row(&"b").unwrap();
        assert!((res_f.values[*ia] - 22.0).abs() < 1e-10);
        assert!((res_f.values[*ib] - 50.0).abs() < 1e-10);
    }

    #[test]
    fn test_serialization() {
        let a = MM!["x" => { "y" => 1.0, "z" => 2.0 },
                    "w" => { "z" => 3.0 }];
        let json = serde_json::to_string(&a).unwrap();
        let a_back: MappedMatrix<&str, &str> = serde_json::from_str(&json).unwrap();

        assert_eq!(a.nrows(), a_back.nrows());
        assert_eq!(a.ncols(), a_back.ncols());
        // Check values via dot product with unit vectors
        let col_y = MV!["y" => 1.0, "z" => 0.0];
        let col_z = MV!["y" => 0.0, "z" => 1.0];
        let res_y = a.dot(&col_y);
        let res_z = a.dot(&col_z);
        let back_y = a_back.dot(&col_y);
        let back_z = a_back.dot(&col_z);
        assert_eq!(res_y.values, back_y.values);
        assert_eq!(res_z.values, back_z.values);
    }
}
