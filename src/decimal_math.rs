use fraction::{generic::GenericInteger, BigDecimal, BigUint};

pub fn decimal_log10_ceil(val: &BigDecimal) -> i32 {
    if val.is_sign_negative() {
        panic!("It is not allowed to take the logarithm of a negative number")
    }

    let dec1 = BigDecimal::from(1);
    let dec01 = BigDecimal::from(0.1);

    let mut dec = val.clone();

    let mut digits = 0;
    loop {
        if dec > dec1 {
            digits += 1;
            dec /= 10;
            continue;
        } else if dec <= dec01 {
            digits -= 1;
            dec *= 10;
            continue;
        }

        break;
    }

    digits
}

pub fn decimal_exp10(exp: i32) -> BigDecimal {
    let dec10 = BigDecimal::from(10);
    let dec1 = BigDecimal::from(1);

    let exp_sign = exp.signum();
    let exp_abs = exp.abs() as u32;

    let dec = match exp_sign {
        1 => {
            let res = BigUint::_10().pow(exp_abs);
            BigDecimal::from(res)
        }
        -1 => {
            let mut dec = dec1;

            for _ in 0..exp_abs {
                dec /= &dec10;
            }

            dec
        }
        _ => dec1,
    };

    dec
}

/// Prefere to use over `BigDecimal::from(f64)` since its implementation is prone to blocking execution when used for very small numbers
pub fn decimal_from_f64(value: f64) -> BigDecimal {
    let (f64_norm, f64_exp) = normalize_f64(value);

    let dec_norm = BigDecimal::from(f64_norm);
    let dec = dec_norm * decimal_exp2(f64_exp);

    dec
}

pub fn decimal_exp2(exp: i32) -> BigDecimal {
    let dec2 = BigDecimal::from(2);
    let dec1 = BigDecimal::from(1);
    let bigu1 = BigUint::_1();

    let exp_sign = exp.signum();
    let exp_abs = exp.abs() as u32;

    let dec = match exp_sign {
        1 => {
            let res = bigu1 << exp_abs;
            BigDecimal::from(res)
        }
        -1 => {
            let mut dec = dec1;

            for _ in 0..exp_abs {
                dec /= &dec2;
            }

            dec
        }
        _ => dec1,
    };

    dec
}

fn normalize_f64(value: f64) -> (f64, i32) {
    let bits = value.to_bits();

    let raw_exponent = ((bits >> 52) & 0x7FF) as i32;

    if raw_exponent == (0b11111111111) {
        panic!("Error trying to normalize a NaN or Infinite value");
    }

    let normalized_bits = (bits & !(0b11111111111 << 52)) | (0b01111111111 << 52);
    let normalized_value = f64::from_bits(normalized_bits);

    let exponent = raw_exponent - 1023;

    if raw_exponent == 0 {
        return (normalized_value - 1.0, -1022);
    } else {
        return (normalized_value, exponent);
    }
}

pub fn decimal_format_scientific(dec: &BigDecimal) -> String {
    let dec10 = BigDecimal::from(10);
    let dec1 = BigDecimal::from(1);

    let mut dec = dec.clone();

    let mut digits = 0;
    loop {
        if dec.abs() >= dec10 {
            digits += 1;
            dec /= 10;
            continue;
        } else if dec.abs() < dec1 {
            digits -= 1;
            dec *= 10;
            continue;
        }

        break;
    }

    format!("{}e{}", dec.calc_precision(None), digits)
}

pub fn decimal_format_scientific_when(dec: &BigDecimal, max_digits: u32) -> String {
    let dec10 = BigDecimal::from(10);
    let dec1 = BigDecimal::from(1);

    let mut normalized_dec = dec.clone();

    let mut digits = 0i32;
    loop {
        if normalized_dec.abs() >= dec10 {
            digits += 1;
            normalized_dec /= 10;
            continue;
        } else if normalized_dec.abs() < dec1 {
            digits -= 1;
            normalized_dec *= 10;
            continue;
        }

        break;
    }

    let abs_digits = digits.abs() as u32;

    match abs_digits > max_digits {
        true => format!("{}e{}", normalized_dec.calc_precision(None), digits),
        false => format!("{}", dec.clone().calc_precision(None)),
    }
}
