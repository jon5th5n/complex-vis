use crate::quaternion::Quaternion;
use std::ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Neg, Sub, SubAssign};

#[derive(Debug, Clone, Copy)]
/// A 2-dimensional vector of 64-bit floating point values.
pub struct Vector2 {
    pub x: f64,
    pub y: f64,
}

impl Vector2 {
    /// Creates a new vector.
    pub fn new(x: f64, y: f64) -> Self {
        Self { x, y }
    }

    /// Creates a vector from a point.
    pub fn from_point(p: (f64, f64)) -> Self {
        Self { x: p.0, y: p.1 }
    }

    /// Creates a zero vector.
    pub fn zero() -> Self {
        Self { x: 0.0, y: 0.0 }
    }

    /// Creates a unit vector pointing into x direction.
    pub fn unit_x() -> Self {
        Self { x: 1.0, y: 0.0 }
    }

    /// Creates a unit vector pointing into y direction.
    pub fn unit_y() -> Self {
        Self { x: 0.0, y: 1.0 }
    }
}

impl Vector2 {
    /// Returns the length of the vector.
    pub fn length(self) -> f64 {
        (self.x * self.x + self.y * self.y).sqrt()
    }
}

impl Vector2 {
    /// Normalizes the vector and returns the result.
    pub fn normalize(self) -> Self {
        self / self.length()
    }

    /// Normalizes the vector and assigns the result.
    pub fn normalize_assign(&mut self) {
        *self /= self.length();
    }
}

impl Neg for Vector2 {
    type Output = Self;

    fn neg(self) -> Self {
        Self {
            x: -self.x,
            y: -self.y,
        }
    }
}

impl Add for Vector2 {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        Self {
            x: self.x + other.x,
            y: self.y + other.y,
        }
    }
}
impl AddAssign for Vector2 {
    fn add_assign(&mut self, other: Self) {
        self.x += other.x;
        self.y += other.y;
    }
}

impl Sub for Vector2 {
    type Output = Self;

    fn sub(self, other: Self) -> Self {
        Self {
            x: self.x - other.x,
            y: self.y - other.y,
        }
    }
}
impl SubAssign for Vector2 {
    fn sub_assign(&mut self, other: Self) {
        self.x -= other.x;
        self.y -= other.y;
    }
}

impl Mul<f64> for Vector2 {
    type Output = Self;

    fn mul(self, scalar: f64) -> Self {
        Self {
            x: self.x * scalar,
            y: self.y * scalar,
        }
    }
}
impl MulAssign<f64> for Vector2 {
    fn mul_assign(&mut self, scalar: f64) {
        self.x *= scalar;
        self.y *= scalar;
    }
}

impl Div<f64> for Vector2 {
    type Output = Self;

    fn div(self, divisor: f64) -> Self {
        Self {
            x: self.x / divisor,
            y: self.y / divisor,
        }
    }
}
impl DivAssign<f64> for Vector2 {
    fn div_assign(&mut self, divisor: f64) {
        self.x /= divisor;
        self.y /= divisor;
    }
}

impl Vector2 {
    /// Calculates the dot product with another vector and returns the result.
    pub fn dot_product(self, v: Vector2) -> f64 {
        self.x * v.x + self.y * v.y
    }
}

#[derive(Debug, Clone, Copy)]
/// A 3-dimensional vector of 64-bit floating point values.
pub struct Vector3 {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

impl Vector3 {
    /// Creates a new vector.
    pub fn new(x: f64, y: f64, z: f64) -> Self {
        Self { x, y, z }
    }

    /// Creates a vector from a point.
    pub fn from_point(p: (f64, f64, f64)) -> Self {
        Self {
            x: p.0,
            y: p.1,
            z: p.2,
        }
    }

    /// Creates a zero vector.
    pub fn zero() -> Self {
        Self {
            x: 0.0,
            y: 0.0,
            z: 0.0,
        }
    }

    /// Creates a unit vector pointing into x direction.
    pub fn unit_x() -> Self {
        Self {
            x: 1.0,
            y: 0.0,
            z: 0.0,
        }
    }

    /// Creates a unit vector pointing into y direction.
    pub fn unit_y() -> Self {
        Self {
            x: 0.0,
            y: 1.0,
            z: 0.0,
        }
    }

    /// Creates a unit vector pointing into z direction.
    pub fn unit_z() -> Self {
        Self {
            x: 0.0,
            y: 0.0,
            z: 1.0,
        }
    }
}

impl Vector3 {
    /// Returns the length of the vector.
    pub fn length(self) -> f64 {
        (self.x * self.x + self.y * self.y + self.z * self.z).sqrt()
    }
}

impl Vector3 {
    /// Normalizes the vector and returns the result.
    pub fn normalize(self) -> Self {
        self / self.length()
    }

    /// Normalizes the vector and assigns the result.
    pub fn normalize_assign(&mut self) {
        *self /= self.length();
    }
}

impl Neg for Vector3 {
    type Output = Self;

    fn neg(self) -> Self {
        Self {
            x: -self.x,
            y: -self.y,
            z: -self.z,
        }
    }
}

impl Add for Vector3 {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        Self {
            x: self.x + other.x,
            y: self.y + other.y,
            z: self.z + other.z,
        }
    }
}
impl AddAssign for Vector3 {
    fn add_assign(&mut self, other: Self) {
        self.x += other.x;
        self.y += other.y;
        self.z += other.z;
    }
}

impl Sub for Vector3 {
    type Output = Self;

    fn sub(self, other: Self) -> Self {
        Self {
            x: self.x - other.x,
            y: self.y - other.y,
            z: self.z - other.z,
        }
    }
}
impl SubAssign for Vector3 {
    fn sub_assign(&mut self, other: Self) {
        self.x -= other.x;
        self.y -= other.y;
        self.z -= other.z;
    }
}

impl Mul<f64> for Vector3 {
    type Output = Self;

    fn mul(self, scalar: f64) -> Self {
        Self {
            x: self.x * scalar,
            y: self.y * scalar,
            z: self.z * scalar,
        }
    }
}
impl MulAssign<f64> for Vector3 {
    fn mul_assign(&mut self, scalar: f64) {
        self.x *= scalar;
        self.y *= scalar;
        self.z *= scalar;
    }
}

impl Div<f64> for Vector3 {
    type Output = Self;

    fn div(self, divisor: f64) -> Self {
        Self {
            x: self.x / divisor,
            y: self.y / divisor,
            z: self.z / divisor,
        }
    }
}
impl DivAssign<f64> for Vector3 {
    fn div_assign(&mut self, divisor: f64) {
        self.x /= divisor;
        self.y /= divisor;
        self.z /= divisor;
    }
}

impl Vector3 {
    /// Rotates the vector around a given axis and returns the result.
    pub fn rotate(self, rot_axis: Vector3, angle: f64) -> Self {
        let self_quat = Quaternion::from_vector(self);
        let rot_quat = Quaternion::new_rotation(rot_axis, angle);

        let rotated = rot_quat * self_quat * rot_quat.conj();

        rotated.to_vector()
    }
}

impl Vector3 {
    /// Calculates the dot product with another vector and returns the result.
    pub fn dot_product(self, v: Vector3) -> f64 {
        self.x * v.x + self.y * v.y + self.z * v.z
    }

    /// Calculates the angle to a plane given by its normal vector and returns it.
    pub fn angle_plane(self, n: Vector3) -> f64 {
        (self.dot_product(n) / (self.length() * n.length())).asin()
    }
}
