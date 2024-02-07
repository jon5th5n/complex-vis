use std::ops::{Div, Mul};

use crate::vector::Vector3;

#[derive(Debug, Clone, Copy)]
/// A simple Quaternion implementation just meant for rotation of vectors.
pub struct Quaternion {
    re: f64,
    i: f64,
    j: f64,
    k: f64,
}

impl Quaternion {
    /// Creates a new quaternion.
    pub fn new(re: f64, i: f64, j: f64, k: f64) -> Self {
        Self { re, i, j, k }
    }

    /// Creates a quaternion describing a rotation around an axis given by a vector.
    pub fn new_rotation(rot_axis: Vector3, angle: f64) -> Self {
        let rot_axis = rot_axis.normalize();

        let sin_a = (angle / 2.0).sin();
        let cos_a = (angle / 2.0).sin();

        Self {
            re: cos_a,
            i: rot_axis.x * sin_a,
            j: rot_axis.y * sin_a,
            k: rot_axis.z * sin_a,
        }
    }

    /// Creates a quaternion from a vector with its real part set to 0.
    pub fn from_vector(v: Vector3) -> Self {
        Self {
            re: 0.0,
            i: v.x,
            j: v.y,
            k: v.z,
        }
    }

    /// Turns the quaternion into a vector disregarding its real part.
    pub fn to_vector(self) -> Vector3 {
        Vector3 {
            x: self.i,
            y: self.j,
            z: self.k,
        }
    }
}

impl Quaternion {
    /// Returns the length of the quaternion.
    pub fn length(&self) -> f64 {
        (self.re * self.re + self.i * self.i + self.j * self.j + self.k * self.k).sqrt()
    }
}

impl Quaternion {
    /// Returns the conjugate of the quaternion.
    pub fn conj(self) -> Self {
        Self {
            re: self.re,
            i: -self.i,
            j: -self.j,
            k: -self.k,
        }
    }

    /// Normalizes the quaternion and returns it.
    pub fn normalize(self) -> Self {
        self / self.length()
    }
}

impl Mul<f64> for Quaternion {
    type Output = Self;

    fn mul(self, scalar: f64) -> Self {
        Self {
            re: self.re * scalar,
            i: self.i * scalar,
            j: self.j * scalar,
            k: self.k * scalar,
        }
    }
}

impl Div<f64> for Quaternion {
    type Output = Self;

    fn div(self, divisor: f64) -> Self {
        Self {
            re: self.re / divisor,
            i: self.i / divisor,
            j: self.j / divisor,
            k: self.k / divisor,
        }
    }
}

impl Mul for Quaternion {
    type Output = Self;

    fn mul(self, other: Quaternion) -> Self {
        let re = self.re * other.re - self.i * other.i - self.j * other.j - self.k * other.k;
        let i = self.re * other.i + self.i * other.re + self.j * other.k - self.k * other.j;
        let j = self.re * other.j - self.i * other.k + self.j * other.re + self.k * other.i;
        let k = self.re * other.k + self.i * other.j - self.j * other.i + self.k * other.re;

        Quaternion { re, i, j, k }
    }
}
