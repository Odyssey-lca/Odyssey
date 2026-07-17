use bimap::BiHashMap;
use std::ops::{Add, AddAssign};
use std::{hash::Hash, sync::Arc};

use crate::MappedMatrix;
use crate::cs::Cs;

/// 2D matrix with values maped to each rows and columns.
#[derive(Debug, Clone)]
pub struct MappedVector<T>
where
    T: std::cmp::Eq + Hash + Clone,
{
    pub mapping: Arc<BiHashMap<T, usize>>,
    pub values: Vec<f64>,
}

impl<T> MappedVector<T>
where
    T: std::cmp::Eq + Hash + Clone,
{
    pub fn empty() -> Self {
        Self {
            mapping: Arc::new(BiHashMap::new()),
            values: vec![],
        }
    }

    pub fn new(mapping: Arc<BiHashMap<T, usize>>, values: Vec<f64>) -> Self {
        Self { mapping, values }
    }

    /// Modify the value associated with the given label.
    /// If a value already existed, it is overwritten and returned.
    /// TODO: Should return a result
    pub fn set(&mut self, label: T, value: f64) -> Option<f64> {
        if self.contains(&label) {
            let index = self.mapping.get_by_left(&label).unwrap();
            let res = self.values[*index];
            self.values[*index] = value;
            Some(res)
        } else {
            None
        }
    }

    /// Add `value` to the existing value associated with the given label.
    /// Returns the old value, or [None] if the label doesn't exist.
    pub fn add_to(&mut self, label: T, value: f64) -> Option<f64> {
        if self.contains(&label) {
            let index = self.mapping.get_by_left(&label).unwrap();
            let res = self.values[*index];
            self.values[*index] += value;
            Some(res)
        } else {
            None
        }
    }

    /// Returns the index of the row corresponding to `id`.
    /// This may fail if `id` has no corresponding row.
    pub fn row(&self, id: &T) -> Option<&usize> {
        self.mapping.get_by_left(id)
    }

    /// Returns the value mapped to the row `index`.
    /// This may fail if `index` is out of range, or
    /// rare cases, if the row was not mapped to any value
    /// (although this should not happen).
    pub fn irow(&self, index: &usize) -> Option<&T> {
        self.mapping.get_by_right(index)
    }

    /// Number of elements in the mapped matrix.
    pub fn nrows(&self) -> usize {
        self.mapping.len()
    }

    /// Test weither any row was mapped to `id`.
    pub fn contains(&self, id: &T) -> bool {
        self.mapping.contains_left(id)
    }

    pub fn diag(&self) -> MappedMatrix<T, T> {
        let n = self.nrows();
        let cs = Cs::new(
            n,
            n,
            (0i32..=n as i32).collect(),
            (0i32..n as i32).collect(),
            self.values.clone(),
        )
        .unwrap();
        MappedMatrix::new(self.mapping.clone(), self.mapping.clone(), cs, None, None)
    }
}

#[macro_export]
macro_rules! MV {
    (
      $(
          $label:expr => $val:expr
      ),* $(,)?
    ) => {{
        let mut values = vec![];
        let mut mapping = ::bimap::BiMap::new();
        $(
            mapping.insert($label, values.len());
            values.push($val);
        )*
        $crate::MappedVector::new(std::sync::Arc::new(mapping), values)
    }};
}

impl<T> Add for MappedVector<T>
where
    T: std::cmp::Eq + Hash + Clone,
{
    type Output = Self;

    fn add(self, other: Self) -> Self {
        let values = self
            .values
            .iter()
            .zip(other.values)
            .map(|(v1, v2)| v1 + v2)
            .collect();
        // TODO: Verify that mappings are the same (DEBUG only)
        Self {
            mapping: self.mapping.clone(),
            values,
        }
    }
}

impl<T> AddAssign for MappedVector<T>
where
    T: std::cmp::Eq + Hash + Clone,
{
    fn add_assign(&mut self, rhs: Self) {
        for (l, r) in self.values.iter_mut().zip(rhs.values) {
            *l += r;
        }
    }
}

impl<T> PartialEq for MappedVector<T>
where
    T: std::cmp::Eq + Hash + Clone,
{
    fn eq(&self, other: &Self) -> bool {
        let eq_values = self.values == other.values;
        let eq_mappings = *self.mapping == *other.mapping;
        eq_values && eq_mappings
    }
}

#[cfg(test)]
mod tests {
    use crate::MV;

    #[test]
    fn test_set() {
        let mut v = MV!["a" => 0.0, "b" => 0.0];
        v.set("a", 5.0);
        v.set("b", 3.0);
        assert_eq!(v.values, vec![5.0, 3.0]);
    }

    #[test]
    fn test_set_overwrite() {
        let mut v = MV!["a" => 0.0];
        v.set("a", 5.0);
        v.set("a", 3.0);
        assert_eq!(v.values, vec![3.0]);
    }

    #[test]
    fn test_set_unknown() {
        let mut v = MV!["a" => 1.0];
        assert!(v.set("b", 2.0).is_none());
        assert_eq!(v.values, vec![1.0]);
    }

    #[test]
    fn test_add_to() {
        let mut v = MV!["a" => 2.0, "b" => 3.0];
        let old = v.add_to("a", 5.0);
        assert_eq!(old, Some(2.0));
        assert_eq!(v.values, vec![7.0, 3.0]);
    }

    #[test]
    fn test_add_to_unknown() {
        let mut v = MV!["a" => 1.0];
        assert!(v.add_to("b", 5.0).is_none());
        assert_eq!(v.values, vec![1.0]);
    }

    #[test]
    fn test_add() {
        let a = MV!["x" => 1.0, "y" => 2.0];
        let b = MV!["x" => 3.0, "y" => 4.0];
        let c = a + b;
        assert_eq!(c.values, vec![4.0, 6.0]);
    }

    #[test]
    fn test_add_assign() {
        let mut a = MV!["x" => 1.0, "y" => 2.0];
        let b = MV!["x" => 3.0, "y" => 4.0];
        a += b;
        assert_eq!(a.values, vec![4.0, 6.0]);
    }

    #[test]
    fn test_partial_eq() {
        let a = MV!["x" => 1.0, "y" => 2.0];
        let b = MV!["x" => 1.0, "y" => 2.0];
        assert_eq!(a, b);
    }

    #[test]
    fn test_partial_eq_ne() {
        let a = MV!["x" => 1.0, "y" => 2.0];
        let b = MV!["x" => 1.0, "y" => 99.0];
        assert_ne!(a, b);
    }

    #[test]
    fn test_diag() {
        let v = MV!["a" => 2.0, "b" => 3.0];
        let m = v.diag();
        assert_eq!(m.nrows(), 2);
        assert_eq!(m.ncols(), 2);

        let col_a = MV!["a" => 1.0, "b" => 0.0];
        let col_b = MV!["a" => 0.0, "b" => 1.0];
        let res_a = m.dot(&col_a);
        let res_b = m.dot(&col_b);
        assert_eq!(res_a.values, vec![2.0, 0.0]);
        assert_eq!(res_b.values, vec![0.0, 3.0]);
    }
}
