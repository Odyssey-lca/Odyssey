#[macro_use]
mod common;

mod conversions {

    // Success
    test_convert_unit_success!(convert_min_to_s, "min", "s", 0, 0, 1, 0, 0, 0, 60.0);
    test_convert_unit_success!(convert_h_to_min, "h", "min", 0, 0, 1, 0, 0, 0, 60.0);
    test_convert_unit_success!(convert_d_to_h, "d", "h", 0, 0, 1, 0, 0, 0, 24.0);
    test_convert_unit_success!(convert_t_to_g, "t", "g", 0, 1, 0, 0, 0, 0, 1000000.0);
    test_convert_unit_success!(convert_ha_to_m2, "ha", "m2", 2, 0, 0, 0, 0, 0, 10000.0);
    test_convert_unit_success!(
        convert_kmperhour_to_mpersecond, 
        "km/h", 
        "m/s", 
        1, 0, -1, 0, 0, 0,
        0.2777777777777778
    );

    test_convert_unit_success!(
        convert_area_mass, 
        "kg/m2", 
        "t/ha", 
        -2, 1, 0, 0, 0, 0,
        10.0                
    );

    // Fail
    test_convert_unit_fail!(fail_m_to_s, "m", "s");
    test_convert_unit_fail!(fail_t_to_l, "t", "l");
    test_convert_unit_fail!(fail_s_to_ha, "s", "ha");
    test_convert_unit_fail!(fail_mcube_to_h, "m3", "h");
    test_convert_unit_fail!(fail_msquare_to_g, "m2", "g");
}