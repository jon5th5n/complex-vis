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
