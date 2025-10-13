use grust::*;
grust! {
    #![dump = "grust/out/maths_fun.rs"]
    // Functions of maths::trigo module
    // `cos` test
    use function std::maths::trigo::cos(x: float) -> float;
    function that_use_cos(theta: float) -> float {
        return cos(theta);
    }

    // `sin` test
    use function std::maths::trigo::sin(x: float) -> float;
    function that_use_sin(theta: float) -> float {
        return sin(theta);
    }

    // `tan` test
    use function std::maths::trigo::tan(x: float) -> float;
    function that_use_tan(theta: float) -> float {
        return tan(theta);
    }

    // `acos` test
    use function std::maths::trigo::acos(x: float) -> float;
    function that_use_acos(theta: float) -> float {
        return acos(theta);
    }

    // `asin` test
    use function std::maths::trigo::asin(x: float) -> float;
    function that_use_asin(theta: float) -> float {
        return asin(theta);
    }

    // `atan` test
    use function std::maths::trigo::atan(x: float) -> float;
    function that_use_atan(theta: float) -> float {
        return atan(theta);
    }

    // `atan2` test
    use function std::maths::trigo::atan2(y: float, x: float) -> float;
    function that_use_atan2(y: float, x: float) -> float {
        return atan2(y,x);
    }

    // `cosh` test
    use function std::maths::trigo::cosh(x: float) -> float;
    function that_use_cosh(theta: float) -> float {
        return cosh(theta);
    }

    // `sinh` test
    use function std::maths::trigo::sinh(x: float) -> float;
    function that_use_sinh(theta: float) -> float {
        return sinh(theta);
    }

    // `tanh` test
    use function std::maths::trigo::tanh(x: float) -> float;
    function that_use_tanh(theta: float) -> float {
        return tanh(theta);
    }

    // `acosh` test
    use function std::maths::trigo::acosh(x: float) -> float;
    function that_use_acosh(theta: float) -> float {
        return acosh(theta);
    }

    // `asinh` test
    use function std::maths::trigo::asinh(x: float) -> float;
    function that_use_asinh(theta: float) -> float {
        return asinh(theta);
    }

    // `atanh` test
    use function std::maths::trigo::atanh(x: float) -> float;
    function that_use_atanh(theta: float) -> float {
        return atanh(theta);
    }

    // Functions of maths::round module
    // `round` test
    use function std::maths::round::round(x: float) -> int;
    function that_use_round(x: float) -> int {
        return round(x);
    }

    // `floor` test
    use function std::maths::round::floor(x: float) -> int;
    function that_use_floor(x: float) -> int {
        return floor(x);
    }

    // `ceil` test
    use function std::maths::round::ceil(x: float) -> int;
    function that_use_ceil(x: float) -> int {
        return ceil(x);
    }

    // Functions of maths::exponential module
    // `exp` test
    use function std::maths::exponential::exp(x: float) -> float;
    function that_use_exp(x: float) -> float {
        return exp(x);
    }

    // `ln` test
    use function std::maths::exponential::ln(x: float) -> float;
    function that_use_ln(x: float) -> float {
        return ln(x);
    }

    // `log` test
    use function std::maths::exponential::log(x: float, n: float) -> float;
    function that_use_log(x: float, n: float) -> float {
        return log(x, n);
    }

    // `log10` test
    use function std::maths::exponential::log10(x: float) -> float;
    function that_use_log10(x: float) -> float {
        return log10(x);
    }

    // `log2` test
    use function std::maths::exponential::log2(x: float) -> float;
    function that_use_log2(x: float) -> float {
        return log2(x);
    }

    // Functions of maths::usuals module
    // `min` test
    use function std::maths::usuals::min(x: float, y: float) -> float;
    function that_use_min(x: float, y: float) -> float {
        return min(x, y);
    }

    // `min_i` test
    use function std::maths::usuals::min_i(x: int, y: int) -> int;
    function that_use_min_i(x: int, y: int) -> int {
        return min_i(x, y);
    }

    // `max` test
    use function std::maths::usuals::max(x: float, y: float) -> float;
    function that_use_max(x: float, y: float) -> float {
        return max(x, y);
    }

    // `max_i` test
    use function std::maths::usuals::max_i(x: int, y: int) -> int;
    function that_use_max_i(x: int, y: int) -> int {
        return max_i(x, y);
    }

    // `abs` test
    use function std::maths::usuals::abs(x: float) -> float;
    function that_use_abs(x: float) -> float {
        return abs(x);
    }

    // `abs_i` test
    use function std::maths::usuals::abs_i(x: int) -> int;
    function that_use_abs_i(x: int) -> int {
        return abs_i(x);
    }

    // `pow` test
    use function std::maths::usuals::pow(x: float, y: float) -> float;
    function that_use_pow(x: float, n: float) -> float {
        return pow(x, n);
    }

    // `pow_i` test
    use function std::maths::usuals::pow_i(x: int, n: int) -> int;
    function that_use_pow_i(x: int, n: int) -> int {
        return pow_i(x, n);
    }

    // `sqrt` test
    use function std::maths::usuals::sqrt(x: float) -> float;
    function that_use_sqrt(x: float) -> float {
        return sqrt(x);
    }

    // `sqrt_i` test
    use function std::maths::usuals::sqrt_i(x: int) -> float;
    function that_use_sqrt_i(x: int) -> float {
        return sqrt_i(x);
    }

}

const PI: f64 = ::std::f64::consts::PI;

#[test]
fn should_compute_the_cos_for_pi() {
    let input = PI;
    let test = that_use_cos(input);
    let correct = input.cos();
    assert_eq!(test, correct);
}

#[test]
fn should_compute_the_sin_for_pi() {
    let input = PI;
    let test = that_use_sin(input);
    let correct = input.sin();
    assert_eq!(test, correct);
}

#[test]
fn should_compute_the_tan_for_1() {
    let input = 1.0;
    let test = that_use_tan(input);
    let correct = input.tan();
    assert_eq!(test, correct);
}

#[test]
fn should_compute_the_acos_for_one_half() {
    let input = 0.5;
    let test = that_use_acos(input);
    let correct = input.acos();
    assert_eq!(test, correct);
}

#[test]
fn should_compute_the_asin_for_one_half() {
    let input = 0.5;
    let test = that_use_asin(input);
    let correct = input.asin();
    assert_eq!(test, correct);
}

#[test]
fn should_compute_the_atan_for_1() {
    let input = 1.0;
    let test = that_use_atan(input);
    let correct = input.atan();
    assert_eq!(test, correct);
}

#[test]
fn should_compute_the_atan2_for_45_deg_angle() {
    let x = 3.0_f64;
    let y = -3.0_f64;
    let test = that_use_atan2(y, x);
    let correct = y.atan2(x);
    assert_eq!(test, correct);
}

#[test]
fn should_compute_the_cosh_for_pi() {
    let input = PI;
    let test = that_use_cosh(input);
    let correct = input.cosh();
    assert_eq!(test, correct);
}

#[test]
fn should_compute_the_sinh_for_pi() {
    let input = PI;
    let test = that_use_sinh(input);
    let correct = input.sinh();
    assert_eq!(test, correct);
}

#[test]
fn should_compute_the_tanh_for_one_half() {
    let input = 0.5;
    let test = that_use_tanh(input);
    let correct = input.tanh();
    assert_eq!(test, correct);
}

#[test]
fn should_compute_the_acosh_for_pi() {
    let input = PI;
    let test = that_use_acosh(input);
    let correct = input.acosh();
    assert_eq!(test, correct);
}

#[test]
fn should_compute_the_asinh_for_pi() {
    let input = PI;
    let test = that_use_asinh(input);
    let correct = input.asinh();
    assert_eq!(test, correct);
}

#[test]
fn should_compute_the_atanh_for_one_half() {
    let input = 0.5;
    let test = that_use_atanh(input);
    let correct = input.atanh();
    assert_eq!(test, correct);
}

#[test]
fn should_compute_round_value_for_minus_3_dot_4() {
    let input = -3.4;
    let test = that_use_round(input);
    let correct = input.round() as i64;
    assert_eq!(test, correct);
}

#[test]
fn should_compute_round_value_for_4_dot_5() {
    let input = 4.5;
    let test = that_use_round(input);
    let correct = input.round() as i64;
    assert_eq!(test, correct);
}

#[test]
fn should_compute_floor_value_for_minus_3_dot_4() {
    let input = -3.4;
    let test = that_use_floor(input);
    let correct = input.floor() as i64;
    assert_eq!(test, correct);
}

#[test]
fn should_compute_floor_value_for_4_dot_5() {
    let input = 4.5;
    let test = that_use_floor(input);
    let correct = input.floor() as i64;
    assert_eq!(test, correct);
}

#[test]
fn should_compute_ceil_value_for_minus_3_dot_4() {
    let input = -3.4;
    let test = that_use_ceil(input);
    let correct = input.ceil() as i64;
    assert_eq!(test, correct);
}

#[test]
fn should_compute_ceil_value_for_4_dot_5() {
    let input = 4.5;
    let test = that_use_ceil(input);
    let correct = input.ceil() as i64;
    assert_eq!(test, correct);
}

#[test]
fn should_compute_exp_value_for_1() {
    let input = 1.0;
    let test = that_use_exp(input);
    let correct = input.exp();
    assert_eq!(test, correct);
}

#[test]
fn should_compute_ln_value_for_2() {
    let input = 2.0;
    let test = that_use_ln(input);
    let correct = input.ln();
    assert_eq!(test, correct);
}

#[test]
fn should_compute_log_base_3_value_for_2() {
    let input = 2.0;
    let base = 4.0;
    let test = that_use_log(input, base);
    let correct = input.log(base);
    assert_eq!(test, correct);
}

#[test]
fn should_compute_log10_value_of_2() {
    let input = 2.0;
    let test = that_use_log10(input);
    let correct = input.log10();
    assert_eq!(test, correct);
}

#[test]
fn should_compute_log2_value_of_2() {
    let input = 2.0;
    let test = that_use_log2(input);
    let correct = input.log2();
    assert_eq!(test, correct);
}

#[test]
fn should_compute_min_value_of_3_dot_5_and_1_dot_6() {
    let x = 3.5;
    let y = 1.6;
    let test = that_use_min(x, y);
    let correct = x.min(y);
    assert_eq!(test, correct);
}

#[test]
fn should_compute_min_value_of_3_and_1() {
    let x = 3;
    let y = 1;
    let test = that_use_min_i(x, y);
    let correct = x.min(y);
    assert_eq!(test, correct);
}

#[test]
fn should_compute_max_value_of_3_dot_5_and_1_dot_6() {
    let x = 3.5;
    let y = 1.6;
    let test = that_use_max(x, y);
    let correct = x.max(y);
    assert_eq!(test, correct);
}

#[test]
fn should_compute_abs_value_of_minus_3_dot_1() {
    let input = -3.1;
    let test = that_use_abs(input);
    let correct = input.abs();
    assert_eq!(test, correct);
}

#[test]
fn should_compute_abs_value_of_4_dot_3() {
    let input = 4.3;
    let test = that_use_abs(input);
    let correct = input.abs();
    assert_eq!(test, correct);
}

#[test]
fn should_compute_abs_value_of_minus_3() {
    let input = -3;
    let test = that_use_abs_i(input);
    let correct = input.abs();
    assert_eq!(test, correct);
}

#[test]
fn should_compute_abs_value_of_4() {
    let input = 4;
    let test = that_use_abs_i(input);
    let correct = input.abs();
    assert_eq!(test, correct);
}

#[test]
fn should_compute_pow_value_of_3_dot_5_to_the_1_dot_6() {
    let x = 3.5;
    let y = 1.6;
    let test = that_use_pow(x, y);
    let correct = x.powf(y);
    assert_eq!(test, correct);
}

#[test]
fn should_compute_pow_value_of_3_to_the_2() {
    let x = 3;
    let y = 2;
    let test = that_use_pow_i(x, y);
    let y = y as u32;
    let correct = x.pow(y);
    assert_eq!(test, correct);
}

#[test]
#[should_panic]
fn should_compute_sqrt_value_of_minus_3_dot_1() {
    let input = -3.1;
    that_use_sqrt(input);
}

#[test]
fn should_compute_sqrt_value_of_minus_0() {
    let input = -0.0;
    let test = that_use_sqrt(input);
    let correct = input.sqrt();
    assert_eq!(test, correct);
}

#[test]
fn should_compute_sqrt_value_of_4_dot_3() {
    let input = 4.3;
    let test = that_use_sqrt(input);
    let input = input as f64;
    let correct = input.sqrt();
    assert_eq!(test, correct);
}

#[test]
#[should_panic]
fn should_compute_sqrt_value_of_minus_3() {
    let input = -3;
    that_use_sqrt_i(input);
}

#[test]
fn should_compute_sqrt_int_value_of_minus_0() {
    let input = -0;
    let test = that_use_sqrt_i(input);
    let input = -0 as f64;
    let correct = input.sqrt();
    assert_eq!(test, correct);
}

#[test]
fn should_compute_sqrt_value_of_4() {
    let input = 4;
    let test = that_use_sqrt_i(input);
    let input = input as f64;
    let correct = input.sqrt();
    assert_eq!(test, correct);
}
