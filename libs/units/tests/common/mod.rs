#[macro_export]
macro_rules! test_parse_unit_success {
    ($name:ident, $input:expr, $length:expr, $mass:expr, $time:expr, $current:expr, $temperature:expr, $amount:expr, $scale:expr) => {
        #[test]
        fn $name() {
            let result = units::parser::parse_unit($input).unwrap();
            assert_eq!(result.dimension.length, $length);
            assert_eq!(result.dimension.mass, $mass);
            assert_eq!(result.dimension.time, $time);
            assert_eq!(result.dimension.current, $current);
            assert_eq!(result.dimension.temperature, $temperature);
            assert_eq!(result.dimension.amount, $amount);
            assert_eq!(result.scale_to_si, $scale);
        }
    };
}

#[macro_export]
macro_rules! test_parse_unit_fail {
    ($name:ident, $input:expr) => {
        #[test]
        fn $name() {
            let result = units::parser::parse_unit($input);
            assert!(result.is_none());
        }
    };
}

#[macro_export]
macro_rules! test_convert_unit_success {
    ($name:ident, $unit1:expr, $unit2:expr, $length:expr, $mass:expr, $time:expr, $current:expr, $temperature:expr, $amount:expr, $scale:expr) => {
        #[test]
        fn $name() {
            let u1 = units::parser::parse_unit($unit1).unwrap();
            let u2 = units::parser::parse_unit($unit2).unwrap();
            let result = (u1.convert(&u2)).expect("Impossible conversion");

            assert_eq!(result, $scale);
        }
    };
}

#[macro_export]
macro_rules! test_convert_unit_fail {
    ($name:ident, $unit1:expr, $unit2:expr) => {
        #[test]
        fn $name() {
            let u1 = units::parser::parse_unit($unit1).unwrap();
            let u2 = units::parser::parse_unit($unit2).unwrap();
            let result = u1.convert(&u2);

            assert!(result.is_none());
        }
    };
}
