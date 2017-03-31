use image::{DynamicImage, GenericImage};
use std::fmt;

use color::Color;

#[derive(Clone, Debug)]
pub struct Material {
    pub coloration: Coloration,
    pub albedo: f32,
}

#[derive(Clone, Copy, Debug)]
pub struct TextureCoords {
    pub x: f32,
    pub y: f32,
}

#[derive(Clone)]
pub enum Coloration {
    Color(Color),
    Texture(DynamicImage),
}

impl Material {
    pub fn color(&self, texture_coords: &TextureCoords) -> Color {
        self.coloration.color(texture_coords)
    }
}

impl Coloration {
    pub fn color(&self, texture_coords: &TextureCoords) -> Color {
        match self {
            &Coloration::Color(ref c) => c.clone(),
            &Coloration::Texture(ref texture) => {
                let x = Coloration::wrap(texture_coords.x, texture.width());
                let y = Coloration::wrap(texture_coords.y, texture.height());

                Color::from_rgba(texture.get_pixel(x, y))
            }
        }
    }

    fn wrap(val: f32, max: u32) -> u32 {
        let signed_max = max as i32;
        let float_coord = val * max as f32;
        let wrapped_coord = (float_coord as i32) % signed_max;
        if wrapped_coord < 0 {
            (wrapped_coord + signed_max) as u32
        } else {
            wrapped_coord as u32
        }
    }
}

impl fmt::Debug for Coloration {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            &Coloration::Color(ref c) => c.fmt(f),
            &Coloration::Texture(ref texture) => {
                write!(f, "Texture {}×{}", texture.width(), texture.height())
            }
        }
    }
}
