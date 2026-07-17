#[macro_use]
mod common;

mod operators {

    // Success
    test_parse_unit_success!(op_mul_star, "m*s", 1, 0, 1, 0, 0, 0, 1.0);
    test_parse_unit_success!(op_mul_point, "m.s", 1, 0, 1, 0, 0, 0, 1.0);
    test_parse_unit_success!(op_div, "m/s", 1, 0, -1, 0, 0, 0, 1.0);
    test_parse_unit_success!(op_simple_exp, "m^3", 3, 0, 0, 0, 0, 0, 1.0);
    test_parse_unit_success!(op_exp_with_prefixe, "km^2", 2, 0, 0, 0, 0, 0, 1000000.0);
    test_parse_unit_success!(op_parenthesis, "m/(s*g)", 1, -1, -1, 0, 0, 0, 1000.0);
    test_parse_unit_success!(op_two_div, "m/s/t", 1, -1, -1, 0, 0, 0, 0.001);

    // Fail
    test_parse_unit_fail!(fail_no_exp, "m^");
    test_parse_unit_fail!(fail_missing_atom2, "m/");
    test_parse_unit_fail!(fail_two_div, "m//s");
    test_parse_unit_fail!(fail_missing_atom1, "/s");
    test_parse_unit_fail!(fail_wrong_op, "m+s");
    test_parse_unit_fail!(erreur_no_close_parenthesis, "m/(s*g");
}