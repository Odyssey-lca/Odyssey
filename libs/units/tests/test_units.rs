#[macro_use]
mod common;

mod base_unit {

    test_parse_unit_success!(unite_meter, "m", 1, 0, 0, 0, 0, 0, 1.0);
    test_parse_unit_success!(unite_gramm, "g", 0, 1, 0, 0, 0, 0, 0.001);
    test_parse_unit_success!(unite_ton, "t", 0, 1, 0, 0, 0, 0, 1000.0);
    test_parse_unit_success!(unite_area_m2, "m2", 2, 0, 0, 0, 0, 0, 1.0);
    test_parse_unit_success!(unite_volume_m3, "m3", 3, 0, 0, 0, 0, 0, 1.0);
    test_parse_unit_success!(unite_volume_sm3, "Sm3", 3, 0, 0, 0, 0, 0, 1.0);
    test_parse_unit_success!(unite_volume_liter_lowercase, "l", 3, 0, 0, 0, 0, 0, 0.001);
    test_parse_unit_success!(unite_volume_liter_uppercase, "L", 3, 0, 0, 0, 0, 0, 0.001);
    test_parse_unit_success!(unite_energy_joule, "J", 2, 1, -2, 0, 0, 0, 1.0);
    test_parse_unit_success!(unite_energy_wh, "Wh", 2, 1, -2, 0, 0, 0, 3600.0);
}

mod additional_units {

    test_parse_unit_success!(unite_time_second, "s", 0, 0, 1, 0, 0, 0, 1.0);
    test_parse_unit_success!(unite_time_minute, "min", 0, 0, 1, 0, 0, 0, 60.0);
    test_parse_unit_success!(unite_time_hour_short, "h", 0, 0, 1, 0, 0, 0, 3600.0);
    test_parse_unit_success!(unite_time_hour_long, "hour", 0, 0, 1, 0, 0, 0, 3600.0);
    test_parse_unit_success!(unite_time_day_short, "d", 0, 0, 1, 0, 0, 0, 86400.0);
    test_parse_unit_success!(unite_time_day_long, "day", 0, 0, 1, 0, 0, 0, 86400.0);
    test_parse_unit_success!(unite_time_year_short, "y", 0, 0, 1, 0, 0, 0, 31556952.0);
    test_parse_unit_success!(unite_time_year_long, "year", 0, 0, 1, 0, 0, 0, 31556952.0);
    test_parse_unit_success!(unite_time_year_alt, "yr", 0, 0, 1, 0, 0, 0, 31556952.0);
    test_parse_unit_success!(unite_area_hectare, "ha", 2, 0, 0, 0, 0, 0, 10000.0);
}

mod prefixes {

    test_parse_unit_success!(prefixe_nano, "nm", 1, 0, 0, 0, 0, 0, 0.000000001);
    test_parse_unit_success!(prefixe_micro, "µm", 1, 0, 0, 0, 0, 0, 0.000001);
    test_parse_unit_success!(prefixe_milli, "mm", 1, 0, 0, 0, 0, 0, 0.001);
    test_parse_unit_success!(prefixe_centi, "cm", 1, 0, 0, 0, 0, 0, 0.01);
    test_parse_unit_success!(prefixe_deci, "dm", 1, 0, 0, 0, 0, 0, 0.1);
    test_parse_unit_success!(prefixe_deca, "dam", 1, 0, 0, 0, 0, 0, 10.0);
    test_parse_unit_success!(prefixe_hecto, "hm", 1, 0, 0, 0, 0, 0, 100.0);
    test_parse_unit_success!(prefixe_kilo, "km", 1, 0, 0, 0, 0, 0, 1000.0);
    test_parse_unit_success!(prefixe_mega, "Mm", 1, 0, 0, 0, 0, 0, 1000000.0);
    test_parse_unit_success!(prefixe_giga, "Gm", 1, 0, 0, 0, 0, 0, 1000000000.0);
}