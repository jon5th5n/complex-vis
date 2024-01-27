use std::ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Neg, Sub, SubAssign};

use crate::quaternion::Quaternion;

#[derive(Debug, Clone, Copy)]
pub struct Vector {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

impl Vector {
    pub fn new(x: f64, y: f64, z: f64) -> Self {
        Self { x, y, z }
    }

    pub fn zero() -> Self {
        Self {
            x: 0.0,
            y: 0.0,
            z: 0.0,
        }
    }

    pub fn unit_x() -> Self {
        Self {
            x: 1.0,
            y: 0.0,
            z: 0.0,
        }
    }

    pub fn unit_y() -> Self {
        Self {
            x: 0.0,
            y: 1.0,
            z: 0.0,
        }
    }

    pub fn unit_z() -> Self {
        Self {
            x: 0.0,
            y: 0.0,
            z: 1.0,
        }
    }
}

impl Vector {
    pub fn length(self) -> f64 {
        (self.x * self.x + self.y * self.y + self.z * self.z).sqrt()
    }
}

impl Vector {
    pub fn normalize(self) -> Self {
        self / self.length()
    }
    pub fn normalize_assign(&mut self) {
        *self /= self.length();
    }
}

impl Neg for Vector {
    type Output = Self;

    fn neg(self) -> Self {
        Self {
            x: -self.x,
            y: -self.y,
            z: -self.z,
        }
    }
}

impl Add for Vector {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        Self {
            x: self.x + other.x,
            y: self.y + other.y,
            z: self.z + other.z,
        }
    }
}
impl AddAssign for Vector {
    fn add_assign(&mut self, other: Self) {
        self.x += other.x;
        self.y += other.y;
        self.z += other.z;
    }
}

impl Sub for Vector {
    type Output = Self;

    fn sub(self, other: Self) -> Self {
        Self {
            x: self.x - other.x,
            y: self.y - other.y,
            z: self.z - other.z,
        }
    }
}
impl SubAssign for Vector {
    fn sub_assign(&mut self, other: Self) {
        self.x -= other.x;
        self.y -= other.y;
        self.z -= other.z;
    }
}

impl Mul<f64> for Vector {
    type Output = Self;

    fn mul(self, scalar: f64) -> Self {
        Self {
            x: self.x * scalar,
            y: self.y * scalar,
            z: self.z * scalar,
        }
    }
}
impl MulAssign<f64> for Vector {
    fn mul_assign(&mut self, scalar: f64) {
        self.x *= scalar;
        self.y *= scalar;
        self.z *= scalar;
    }
}

impl Div<f64> for Vector {
    type Output = Self;

    fn div(self, divisor: f64) -> Self {
        Self {
            x: self.x / divisor,
            y: self.y / divisor,
            z: self.z / divisor,
        }
    }
}
impl DivAssign<f64> for Vector {
    fn div_assign(&mut self, divisor: f64) {
        self.x /= divisor;
        self.y /= divisor;
        self.z /= divisor;
    }
}

impl Vector {
    pub fn rotate(self, rot_axis: Vector, angle: f64) -> Self {
        let self_quat = Quaternion::from_vector(self);
        let rot_quat = Quaternion::new_rotation(rot_axis, angle);

        let rotated = rot_quat * self_quat * rot_quat.conj();

        rotated.to_vector()
    }
}

impl Vector {
    pub fn dot_product(self, v: Vector) -> f64 {
        self.x * v.x + self.y * v.y + self.z * v.z
    }

    pub fn angle_plane(self, n: Vector) -> f64 {
        (self.dot_product(n) / (self.length() * n.length())).asin()
    }
}
