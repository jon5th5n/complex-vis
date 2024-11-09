use fraction::{generic::GenericInteger, BigDecimal, BigUint};

pub fn decimal_log10_ceil(val: &BigDecimal) -> i32 {
    let mut dec = val.clone();

    let mut digits = 0;
    loop {
        if dec > BigDecimal::from(1.0) {
            digits += 1;
            dec /= 10;
            continue;
        } else if dec <= BigDecimal::from(0.1) {
            digits -= 1;
            dec *= 10;
            continue;
        }

        break;
    }

    digits
}

pub fn decimal_exp10(exp: i32) -> BigDecimal {
    let exp_sign = exp.signum();
    let exp_abs = exp.abs() as u32;

    let dec = match exp_sign {
        1 => {
            let res = BigUint::_10().pow(exp_abs);
            BigDecimal::from(res)
        }
        -1 => {
            let mut dec = BigDecimal::from(1);

            for _ in 0..exp_abs {
                dec /= BigDecimal::from(10);
            }

            dec
        }
        _ => BigDecimal::from(1),
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
