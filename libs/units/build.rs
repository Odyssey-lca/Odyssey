use std::env;
use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::Path;

struct Unit<'a> {
    symbol: &'a str,
    dimension: &'a str,
    scale_to_si: f64,
    pow: i32,
}

struct Prefix<'a> {
    symbol: &'a str,
    scale: f64,
}

#[rustfmt::skip]
const UNITS: [Unit; 12] = [
  Unit {symbol: "m",  dimension: "LENGTH", scale_to_si: 1.,    pow: 1}, // Meter
  Unit {symbol: "g",  dimension: "MASS",   scale_to_si: 1e-3,  pow: 1}, // Gramm
  Unit {symbol: "t",  dimension: "MASS",   scale_to_si: 1e3,   pow: 1}, // Ton
  Unit {symbol: "m2", dimension: "AREA",   scale_to_si: 1.,    pow: 2}, // Meter square
  Unit {symbol: "m3", dimension: "VOLUME", scale_to_si: 1.,    pow: 3}, // Cube meter
  Unit {symbol: "Sm3", dimension: "VOLUME", scale_to_si: 1.,   pow: 3}, // Cube meter
  Unit {symbol: "l",  dimension: "VOLUME", scale_to_si: 1e-3,  pow: 1}, // Liter
  Unit {symbol: "L",  dimension: "VOLUME", scale_to_si: 1e-3,  pow: 1}, // Liter
  Unit {symbol: "J",  dimension: "ENERGY", scale_to_si: 1.,    pow: 1}, // Joule
  Unit {symbol: "Wh", dimension: "ENERGY", scale_to_si: 3600., pow: 1}, // Watt hour
  Unit {symbol: "W",  dimension: "POWER",  scale_to_si: 1.,    pow: 1 }, // Watt
  Unit {symbol: "cal", dimension: "ENERGY", scale_to_si: 4.184,pow: 1 }, // Calorie
];

#[rustfmt::skip]
const ADDITIONAL_UNITS: [Unit; 13] = [
  Unit {symbol: "s",    dimension: "TIME", scale_to_si: 1.,          pow: 1},
  Unit {symbol: "min",  dimension: "TIME", scale_to_si: 60.,         pow: 1},
  Unit {symbol: "h",    dimension: "TIME", scale_to_si: 3_600.,      pow: 1},
  Unit {symbol: "hour", dimension: "TIME", scale_to_si: 3_600.,      pow: 1},
  Unit {symbol: "d",    dimension: "TIME", scale_to_si: 86_400.,     pow: 1},
  Unit {symbol: "day",  dimension: "TIME", scale_to_si: 86_400.,     pow: 1},
  Unit {symbol: "month",dimension: "TIME", scale_to_si: 2_629_800.,  pow: 1 },
  Unit {symbol: "y",    dimension: "TIME", scale_to_si: 31_556_952., pow: 1},
  Unit {symbol: "year", dimension: "TIME", scale_to_si: 31_556_952., pow: 1},
  Unit {symbol: "yr",   dimension: "TIME", scale_to_si: 31_556_952., pow: 1},
  Unit {symbol: "ha",   dimension: "AREA", scale_to_si: 10_000.,     pow: 2},
  Unit {symbol: "hp",   dimension: "POWER", scale_to_si: 745.699872, pow: 1 }, // Horsepower imperial
  Unit {symbol: "K",    dimension: "TEMPERATURE", scale_to_si: 1.,   pow: 1}, // Kelvin
];

#[rustfmt::skip]
const PREFIXES: [Prefix; 10] = [
  Prefix {symbol: "n",  scale: 1e-9},
  Prefix {symbol: "µ",  scale: 1e-6},
  Prefix {symbol: "m",  scale: 1e-3},
  Prefix {symbol: "c",  scale: 1e-2},
  Prefix {symbol: "d",  scale: 1e-1},
  Prefix {symbol: "da", scale: 1e1},
  Prefix {symbol: "h",  scale: 1e2},
  Prefix {symbol: "k",  scale: 1e3},
  Prefix {symbol: "M",  scale: 1e6},
  Prefix {symbol: "G",  scale: 1e9},
];

fn main() {
    let path = Path::new(&env::var("OUT_DIR").unwrap()).join("codegen.rs");
    let mut file = BufWriter::new(File::create(&path).unwrap());
    let mut unit_map = phf_codegen::Map::new();
    for unit in UNITS {
        unit_map.entry(
            unit.symbol.to_string(),
            format!(
                "Unit {{ dimension: {}, scale_to_si: {:e}, substance: None }}",
                unit.dimension, unit.scale_to_si
            ),
        );
        for pre in PREFIXES {
            let key = format!("{}{}", pre.symbol, unit.symbol);
            let val = format!(
                "Unit {{ dimension: {}, scale_to_si: {:e}, substance: None }}",
                unit.dimension,
                unit.scale_to_si * (pre.scale.powi(unit.pow))
            );
            unit_map.entry(key, val);
        }
    }

    for unit in ADDITIONAL_UNITS {
        unit_map.entry(
            unit.symbol.to_string(),
            format!(
                "Unit {{ dimension: {}, scale_to_si: {:e}, substance: None }}",
                unit.dimension, unit.scale_to_si
            ),
        );
    }

    writeln!(
        &mut file,
        "pub static UNITS: phf::Map<&'static str, Unit> = {};",
        unit_map.build()
    )
    .unwrap();
}
