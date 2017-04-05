use std::ops::{Add, Mul};

use image::{Pixel, Rgba};
use serde;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Color {
    pub red: f32,
    pub green: f32,
    pub blue: f32,
}

impl Color {
    pub fn new(red: f32, green: f32, blue: f32) -> Color {
        Color {
            red: red,
            green: green,
            blue: blue,
        }
    }

    pub fn black() -> Color {
        Color::new(0.0, 0.0, 0.0)
    }

    pub fn from_rgba(rgba: Rgba<u8>) -> Color {
        Color::new(rgba.data[0] as f32 / 255.0,
                   rgba.data[1] as f32 / 255.0,
                   rgba.data[2] as f32 / 255.0)
    }

    pub fn rgba(&self) -> Rgba<u8> {
        Rgba::from_channels((self.red * 255.0) as u8,
                            (self.green * 255.0) as u8,
                            (self.blue * 255.0) as u8,
                            255)
    }

    pub fn clamp(&self) -> Color {
        Color::new(self.red.min(1.0).max(0.0),
                   self.green.min(1.0).max(0.0),
                   self.blue.min(1.0).max(0.0))
    }
}

impl Default for Color {
    fn default() -> Color {
        Color::black()
    }
}

impl ::std::fmt::Display for Color {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        let red_byte = (self.red as f32 * 255.0).floor() as u8;
        let green_byte = (self.green as f32 * 255.0).floor() as u8;
        let blue_byte = (self.blue as f32 * 255.0).floor() as u8;

        write!(f, "#{:02x}{:02x}{:02x}", red_byte, green_byte, blue_byte)
    }
}

impl Add for Color {
    type Output = Color;

    fn add(self, other: Color) -> Color {
        Color::new(self.red + other.red,
                   self.green + other.green,
                   self.blue + other.blue)
    }
}

impl Mul for Color {
    type Output = Color;

    fn mul(self, other: Color) -> Color {
        Color::new(self.red * other.red,
                   self.green * other.green,
                   self.blue * other.blue)
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
        Color::new(self.red * other, self.green * other, self.blue * other)
    }
}

impl<'a> Mul<f32> for &'a Color {
    type Output = Color;

    fn mul(self, other: f32) -> Color {
        (*self) * other
    }
}

impl ::std::str::FromStr for Color {
    type Err = String;

    fn from_str(s: &str) -> Result<Color, String> {
        if s.len() == "#123456".len() && s.starts_with("#") {
            if let Ok(num) = u64::from_str_radix(&s[1..], 16) {
                let red = ((num & 0xff0000) >> 16) as f32;
                let grn = ((num & 0x00ff00) >> 8) as f32;
                let blu = ((num & 0x0000ff) >> 0) as f32;

                return Ok(Color::new(red / 255.0, grn / 255.0, blu / 255.0));
            }
        }

        Err(format!("{} is not a valid color", s))
    }
}

struct ColorVisitor;

impl serde::de::Visitor for ColorVisitor {
    type Value = Color;

    fn expecting(&self, formatter: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        formatter.write_str("a string of a simple hex color (#000000 - #ffffff)")
    }

    fn visit_str<E>(self, value: &str) -> Result<Color, E>
        where E: serde::de::Error
    {
        value.parse().map_err(|error| E::custom(error))
    }

    fn visit_string<E>(self, value: String) -> Result<Color, E>
        where E: serde::de::Error
    {
        value.parse().map_err(|error| E::custom(error))
    }
}

impl serde::Deserialize for Color {
    fn deserialize<D>(deserializer: D) -> Result<Color, D::Error>
        where D: serde::Deserializer
    {
        deserializer.deserialize_string(ColorVisitor)
    }
}

#[test]
fn it_parses_strings() {
    use std::str::FromStr;
    assert_eq!(Color::from_str("#000000").unwrap(),
               Color {
                   red: 0.0,
                   green: 0.0,
                   blue: 0.0,
               });

    assert_eq!(Color::from_str("#ffffff").unwrap(),
               Color {
                   red: 1.0,
                   green: 1.0,
                   blue: 1.0,
               });

    assert_eq!(Color::from_str("#ff7f11").unwrap(),
               Color {
                   red: 1.0,
                   green: 0.498039216,
                   blue: 0.066666667,
               });
}

#[test]
fn it_displays_strings() {
    assert_eq!(Color::black().to_string(), "#000000");
    assert_eq!(Color::new(1.0, 0.5, 0.0).to_string(), "#ff7f00");
}

#[test]
fn it_returns_same_color_as_input() {
    use std::str::FromStr;
    let samples = &["#000000", "#123456", "#ffeecc", "#fef0fa", "#010203"];

    for sample in samples {
        let parsed_color: Color = sample.parse().unwrap();
        let back_again = sample.to_string();
        assert_eq!(&back_again, sample);
    }
}
