use std::fmt;

use serde::{Deserialize, Serialize};

use crate::dimension::*;

include!(concat!(env!("OUT_DIR"), "/codegen.rs"));

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct Unit {
    pub dimension: Dimension,
    pub scale_to_si: f64,
    pub substance: Option<String>,
}

impl Unit {
    /// Converts the current unit into another target unit, if their physical dimensions match.
    ///
    /// # Examples
    /// 
    /// ```
    /// use units::parser::parse_unit;
    /// 
    /// let hour = parse_unit("h").unwrap();
    /// let second = parse_unit("s").unwrap();
    /// 
    /// // Successful conversion (same dimensions)
    /// if let Some(scale) = hour.convert(&second) {
    ///     assert_eq!(scale, 3600.0);
    /// }
    /// 
    /// // Failed conversion (different dimensions: time to length)
    /// let meter = parse_unit("m").unwrap();
    /// assert!(hour.convert(&meter).is_none());
    /// ```
    pub fn convert(&self, to: &Unit) -> Option<f64> {
        if self.dimension == to.dimension && self.substance == to.substance {
            let scale = self.scale_to_si / to.scale_to_si;
            Some(scale)
        } else {
            None
        }
    }

    pub fn pow(&self, exp: i32) -> Unit {
        let scale_to_si = self.scale_to_si.powi(exp);
        let dimension = self.dimension.pow(exp);
        let substance = self.substance.clone();
        Unit { dimension, scale_to_si, substance }
    }

    pub fn format_without_scale(&self) -> String {
      format!("Dimension : {}, Substance : {:?}", self.dimension, self.substance)
    }
}

impl std::ops::Mul for Unit {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self {

        let substance = match (self.substance, rhs.substance) {
            (Some(s_lhs), Some(s_rhs)) => Some(format!("{}*{}", s_lhs, s_rhs)),
            (Some(s_lhs), None) => Some(s_lhs),
            (None, Some(s_rhs)) => Some(s_rhs),
            (None, None) => None,
        };

        Self {
            scale_to_si: self.scale_to_si * rhs.scale_to_si,
            dimension: self.dimension * rhs.dimension,
            substance,
        }
    }
}

impl std::ops::Div for Unit {
    type Output = Self;

    fn div(self, rhs: Self) -> Self {

        let substance = match (self.substance, rhs.substance) {
            (Some(s_lhs), Some(s_rhs)) => Some(format!("{}/{}", s_lhs, s_rhs)),
            (Some(s_lhs), None) => Some(s_lhs),
            (None, Some(s_rhs)) => Some(format!("1/{}", s_rhs)),
            (None, None) => None,
        };

        Self {
            scale_to_si: self.scale_to_si / rhs.scale_to_si,
            dimension: self.dimension / rhs.dimension,
            substance,
        }
    }
}

impl fmt::Display for Unit {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "Dimension : {}, Scale_to_si : {}, Substance : {:?}",
            self.dimension,
            self.scale_to_si,
            self.substance,
        )
    }
}
