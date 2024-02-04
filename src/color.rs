#[derive(Debug, Clone, Copy)]
pub struct RGBA {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl RGBA {
    pub fn new(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self { r, g, b, a }
    }

    pub fn to_rgb(self) -> (RGB, u8) {
        (
            RGB {
                r: self.r,
                g: self.g,
                b: self.b,
            },
            self.a,
        )
    }
}

#[derive(Debug, Clone, Copy)]
pub struct RGB {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl RGB {
    /// Adds an RGBA value onto a RGB value returning the result.
    /// This simply performs a linear interpolation between the two.
    pub fn add_rgba(self, other: RGBA) -> Self {
        let (other, alpha) = other.to_rgb();
        self.lerp(&other, alpha as f64 / 255.0)
    }

    /// Performs a linear interpolation between two RGB values returning the result.
    pub fn lerp(&self, other: &Self, a: f64) -> Self {
        RGB {
            r: ((1.0 - a) * self.r as f64 + a * other.r as f64) as u8,
            g: ((1.0 - a) * self.g as f64 + a * other.g as f64) as u8,
            b: ((1.0 - a) * self.b as f64 + a * other.b as f64) as u8,
        }
    }
}
