use std::ops::{Add, Mul};

use image::{Pixel, Rgba};

#[derive(Clone, Copy)]
pub struct Color {
    pub red: f32,
    pub green: f32,
    pub blue: f32,
}

impl Color {
    pub fn black() -> Color {
        Color {
            red: 0.0,
            green: 0.0,
            blue: 0.0,
        }
    }

    pub fn rgba(&self) -> Rgba<u8> {
        Rgba::from_channels((self.red * 255.0) as u8,
                            (self.green * 255.0) as u8,
                            (self.blue * 255.0) as u8,
                            255)
    }

    pub fn clamp(&self) -> Color {
        Color {
            red: self.red.min(1.0).max(0.0),
            green: self.green.min(1.0).max(0.0),
            blue: self.blue.min(1.0).max(0.0),
        }
    }
}

impl Add for Color {
    type Output = Color;

    fn add(self, other: Color) -> Color {
        Color {
            red: self.red + other.red,
            green: self.green + other.green,
            blue: self.blue + other.blue,
        }
    }
}

impl Mul for Color {
    type Output = Color;

    fn mul(self, other: Color) -> Color {
        Color {
            red: self.red * other.red,
            green: self.green * other.green,
            blue: self.blue * other.blue,
        }
    }
}

impl<'a> Mul for &'a Color {
    type Output = Color;

    fn mul(self, other: &'a Color) -> Color {
        self * (*other)
    }
}

impl<'a> Mul<Color> for &'a Color {
    type Output = Color;

    fn mul(self, other: Color) -> Color {
        (*self) * other
    }
}

impl Mul<f32> for Color {
    type Output = Color;

    fn mul(self, other: f32) -> Color {
        Color {
            red: self.red * other,
            green: self.green * other,
            blue: self.blue * other,
        }
    }
}
