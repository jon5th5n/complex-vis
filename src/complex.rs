use std::{
    f64::consts::E,
    ops::{Add, Div, Mul, Neg, Sub},
};

#[derive(Debug, Clone, Copy)]
/// Descibes a complex number in cartesion form _`re` + `im`i_.
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
    /// Converts a complex number from polar to cartesion form.
    fn from_polar(polar: &ComplexPolar) -> Self {
        let re = polar.mag * polar.ang.cos();
        let im = polar.mag * polar.ang.sin();
        return Self { re, im };
    }
}

#[derive(Debug, Clone, Copy)]
/// Describes a complex number in polar form _`mag`e^(`ang`i)_
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
    /// Converts a complex number from cartesian to polar form.
    fn from_cartesian(cartesian: &ComplexCartesian) -> Self {
        let mag = (cartesian.re * cartesian.re + cartesian.im * cartesian.im).sqrt();
        let ang = (cartesian.re / mag).acos();
        return Self { mag, ang };
    }
}

#[derive(Debug, Clone, Copy)]
/// Describes a complex number in both cartesian and polar form.
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
    /// Creates a complex number from its cartesian parts.
    pub fn new_cartesian(re: f64, im: f64) -> Self {
        let cartesian = ComplexCartesian { re, im };
        let polar = ComplexPolar::from_cartesian(&cartesian);

        Self { cartesian, polar }
    }

    /// Creates a complex number from its polar parts.
    pub fn new_polar(mag: f64, ang: f64) -> Self {
        let polar = ComplexPolar { mag, ang };
        let cartesian = ComplexCartesian::from_polar(&polar);

        Self { cartesian, polar }
    }

    /// Creates a complex number just from its real cartesian part leaving the imaginary part 0.
    pub fn new_real(re: f64) -> Self {
        Self::new_cartesian(re, 0.0)
    }

    /// Creates a complex number just from its imaginary cartesian part leaving the real part 0.
    pub fn new_imaginary(im: f64) -> Self {
        Self::new_cartesian(0.0, im)
    }

    /// Creates a complex number representing the value 0.
    pub fn zero() -> Self {
        Self::new_cartesian(0.0, 0.0)
    }

    /// Creates a complex number representing the value 1.
    pub fn one() -> Self {
        Self::new_cartesian(1.0, 0.0)
    }

    /// Creates a complex number representing the value i.
    pub fn i() -> Self {
        Self::new_cartesian(0.0, 1.0)
    }

    /// Creates a complex number representing the value e.
    pub fn e() -> Self {
        Self::new_real(E)
    }
}

impl Complex {
    /// Returns the complex number in cartesian form.
    pub fn cartesian(&self) -> ComplexCartesian {
        self.cartesian
    }

    /// Returns the complex number in polar form.
    pub fn polar(&self) -> ComplexPolar {
        self.polar
    }

    /// Returns just the real part of the complex numbers cartesian form.
    pub fn re(&self) -> f64 {
        self.cartesian.re
    }

    /// Returns just the imaginary part of the complex numbers cartesian form.
    pub fn im(&self) -> f64 {
        self.cartesian.im
    }

    /// Returns just the magnitude of the complex numbers polar form.
    pub fn mag(&self) -> f64 {
        self.polar.mag
    }

    /// Returns just the angle of the complex numbers polar form.
    pub fn ang(&self) -> f64 {
        self.polar.ang
    }
}

impl Complex {
    /// Returns the negation of the complex number.
    /// Same as using the unary negation operator `-`.
    pub fn opposite(self) -> Self {
        let re = -self.cartesian.re;
        let im = -self.cartesian.im;
        Self::new_cartesian(re, im)
    }

    // Returns the reciprocal of the complex number.
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
    /// Returns the natural logarithm of the complex number.
    pub fn ln(self) -> Option<Self> {
        if self.polar.mag == 0.0 {
            return None;
        }

        let re = self.polar.mag.ln();
        let im = self.polar.ang;

        Some(Self::new_cartesian(re, im))
    }

    /// Returns the logarithm to any other base of the complex number.
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
    /// Raises the complex number to any other complex number and returns the result.
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

    /// Retuns the nth root of the complex number with n being any other complex number.
    pub fn root(self, other: Self) -> Option<Self> {
        self.pow(other.reciprocal()?)
    }
}

impl Complex {
    /// Returns the sine of the complex number.
    pub fn sin(self) -> Option<Self> {
        let re = self.cartesian.re.sin() * self.cartesian.im.cosh();
        let im = self.cartesian.re.cos() * self.cartesian.im.sinh();

        Some(Self::new_cartesian(re, im))
    }

    /// Returns the cosine of the complex number.
    pub fn cos(self) -> Option<Self> {
        let re = self.cartesian.re.cos() * self.cartesian.im.cosh();
        let im = -(self.cartesian.re.sin() * self.cartesian.im.sinh());

        Some(Self::new_cartesian(re, im))
    }
}
