use core::fmt;

use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Eq, Clone, Copy, Serialize, Deserialize)]
pub struct Dimension {
    pub length: i32,      // meter
    pub mass: i32,        // kilogram
    pub time: i32,        // second
    pub current: i32,     // ampere
    pub temperature: i32, // kelvin
    pub amount: i32,      // mole
}

impl std::ops::Mul for Dimension {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self {
        Self {
            length: self.length + rhs.length,
            mass: self.mass + rhs.mass,
            time: self.time + rhs.time,
            current: self.current + rhs.current,
            temperature: self.temperature + rhs.temperature,
            amount: self.amount + rhs.amount,
        }
    }
}

impl std::ops::Div for Dimension {
    type Output = Self;

    fn div(self, rhs: Self) -> Self {
        Self {
            length: self.length - rhs.length,
            mass: self.mass - rhs.mass,
            time: self.time - rhs.time,
            current: self.current - rhs.current,
            temperature: self.temperature - rhs.temperature,
            amount: self.amount - rhs.amount,
        }
    }
}

impl Dimension {
    pub const fn mul(self, rhs: Self) -> Self {
        Self {
            length: self.length + rhs.length,
            mass: self.mass + rhs.mass,
            time: self.time + rhs.time,
            current: self.current + rhs.current,
            temperature: self.temperature + rhs.temperature,
            amount: self.amount + rhs.amount,
        }
    }

    pub const fn div(self, rhs: Self) -> Self {
        Self {
            length: self.length - rhs.length,
            mass: self.mass - rhs.mass,
            time: self.time - rhs.time,
            current: self.current - rhs.current,
            temperature: self.temperature - rhs.temperature,
            amount: self.amount - rhs.amount,
        }
    }

    pub const fn pow(self, k: i32) -> Self {
        Self {
            length: self.length * k,
            mass: self.mass * k,
            time: self.time * k,
            current: self.current * k,
            temperature: self.temperature * k,
            amount: self.amount * k,
        }
    }
}

impl fmt::Display for Dimension {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{};{};{};{};{};{}",
            self.length,
            self.mass,
            self.time,
            self.current,
            self.temperature,
            self.amount,
        )
    }
}

#[rustfmt::skip]
pub const DIMENSIONLESS: Dimension = Dimension {length: 0, mass: 0, time: 0, current: 0, temperature: 0, amount: 0};
pub const LENGTH: Dimension = Dimension { length: 1, ..DIMENSIONLESS };
pub const MASS: Dimension = Dimension { mass: 1, ..DIMENSIONLESS };
pub const TIME: Dimension = Dimension { time: 1, ..DIMENSIONLESS };
pub const CURRENT: Dimension = Dimension { current: 1, ..DIMENSIONLESS };
pub const TEMPERATURE: Dimension = Dimension { temperature: 1, ..DIMENSIONLESS };
pub const AMOUNT: Dimension = Dimension { amount: 1, ..DIMENSIONLESS };
pub const AREA: Dimension = LENGTH.pow(2);
pub const VOLUME: Dimension = LENGTH.pow(3);
pub const VELOCITY: Dimension = LENGTH.div(TIME);
pub const ACCELERATION: Dimension = MASS.div(TIME.pow(2));
pub const ENERGY: Dimension = MASS.mul(VELOCITY.pow(2));
pub const POWER: Dimension = ENERGY.div(TIME);
