use std::ops::{Add, Div, Mul, Neg, Sub};

#[derive(Debug, Clone, Copy)]
pub struct ComplexCartesian {
    re: f64,
    im: f64,
}

impl std::fmt::Display for ComplexCartesian {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{} + {}i", self.re, self.im)
    }
}

impl ComplexCartesian {
    fn from_polar(polar: &ComplexPolar) -> Self {
        let re = polar.mag * polar.ang.cos();
        let im = polar.mag * polar.ang.sin();
        return Self { re, im };
    }
}

#[derive(Debug, Clone, Copy)]
pub struct ComplexPolar {
    mag: f64,
    ang: f64,
}

impl std::fmt::Display for ComplexPolar {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}e^({}i)", self.mag, self.ang)
    }
}

impl ComplexPolar {
    fn from_cartesian(cartesian: &ComplexCartesian) -> Self {
        let mag = (cartesian.re * cartesian.re + cartesian.im * cartesian.im).sqrt();
        let ang = (cartesian.re / mag).acos();
        return Self { mag, ang };
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Complex {
    cartesian: ComplexCartesian,
    polar: ComplexPolar,
}

impl std::fmt::Display for Complex {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Cartesian: {}\nPolar: {}", self.cartesian, self.polar)
    }
}

impl Complex {
    pub fn new_cartesian(re: f64, im: f64) -> Self {
        let cartesian = ComplexCartesian { re, im };
        let polar = ComplexPolar::from_cartesian(&cartesian);

        Self { cartesian, polar }
    }

    pub fn new_polar(mag: f64, ang: f64) -> Self {
        let polar = ComplexPolar { mag, ang };
        let cartesian = ComplexCartesian::from_polar(&polar);

        Self { cartesian, polar }
    }

    pub fn new_real(re: f64) -> Self {
        Self::new_cartesian(re, 0.0)
    }

    pub fn new_imaginary(im: f64) -> Self {
        Self::new_cartesian(0.0, im)
    }

    pub fn zero() -> Self {
        Self::new_cartesian(0.0, 0.0)
    }

    pub fn one() -> Self {
        Self::new_cartesian(1.0, 0.0)
    }

    pub fn i() -> Self {
        Self::new_cartesian(0.0, 1.0)
    }
}

impl Complex {
    pub fn cartesian(&self) -> ComplexCartesian {
        self.cartesian
    }

    pub fn polar(&self) -> ComplexPolar {
        self.polar
    }

    pub fn re(&self) -> f64 {
        self.cartesian.re
    }

    pub fn im(&self) -> f64 {
        self.cartesian.im
    }

    pub fn mag(&self) -> f64 {
        self.polar.mag
    }

    pub fn ang(&self) -> f64 {
        self.polar.ang
    }
}

impl Complex {
    pub fn opposite(self) -> Self {
        let re = -self.cartesian.re;
        let im = -self.cartesian.im;
        Self::new_cartesian(re, im)
    }

    pub fn reciprocal(self) -> Option<Self> {
        Self::one() / self
    }
}

impl Neg for Complex {
    type Output = Self;

    fn neg(self) -> Self {
        self.opposite()
    }
}

impl Add for Complex {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        let re = self.cartesian.re + other.cartesian.re;
        let im = self.cartesian.im + other.cartesian.im;

        Self::new_cartesian(re, im)
    }
}
impl Sub for Complex {
    type Output = Self;

    fn sub(self, other: Self) -> Self {
        let re = self.cartesian.re - other.cartesian.re;
        let im = self.cartesian.im - other.cartesian.im;

        Self::new_cartesian(re, im)
    }
}

impl Mul for Complex {
    type Output = Self;

    fn mul(self, other: Self) -> Self {
        let mag = self.polar.mag * other.polar.mag;
        let ang = self.polar.ang + other.polar.ang;

        Self::new_polar(mag, ang)
    }
}
impl Div for Complex {
    type Output = Option<Self>;

    fn div(self, other: Self) -> Option<Self> {
        if other.polar.mag == 0.0 {
            return None;
        }

        let mag = self.polar.mag / other.polar.mag;
        let ang = self.polar.ang - other.polar.ang;

        Some(Self::new_polar(mag, ang))
    }
}

impl Complex {
    pub fn ln(self) -> Option<Self> {
        if self.polar.mag == 0.0 {
            return None;
        }

        let re = self.polar.mag.ln();
        let im = self.polar.ang;

        Some(Self::new_cartesian(re, im))
    }

    pub fn log(self, other: Self) -> Option<Self> {
        if other.polar.mag == 1.0 || self.polar.mag == 0.0 {
            return None;
        }

        if other.polar.mag == 0.0 {
            return Some(Self::zero());
        }

        let mag_s = self.polar.mag;
        let ang_s = self.polar.ang;

        let mag_o = other.polar.mag;
        let ang_o = other.polar.ang;

        let divisor = mag_o.ln().powi(2) + ang_o.powi(2);

        let re = (mag_s.ln() * mag_o.ln() + ang_s * ang_o) / divisor;
        let im = (ang_s * mag_o.ln() - ang_o * mag_s.ln()) / divisor;

        Some(Self::new_cartesian(re, im))
    }
}

impl Complex {
    pub fn pow(self, other: Self) -> Option<Self> {
        if self.polar.mag == 0.0 && other.polar.mag == 0.0 {
            return None;
        }

        if other.polar.mag == 0.0 {
            return Some(Self::one());
        }

        if self.polar.mag == 0.0 {
            return Some(Self::zero());
        }

        let b_mag = self.polar.mag;
        let b_ang = self.polar.ang;

        let e_re = other.cartesian.re;
        let e_im = other.cartesian.im;

        let mag = (e_re * b_mag.ln() - e_im * b_ang).exp();
        let ang = e_re * b_ang + e_im * b_mag.ln();

        Some(Self::new_polar(mag, ang))
    }

    pub fn root(self, other: Self) -> Option<Self> {
        self.pow(other.reciprocal()?)
    }
}
