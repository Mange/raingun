use serde;
use image::{DynamicImage, GenericImage};
use std::fmt;

use color::Color;

#[derive(Clone, Debug, Deserialize)]
pub struct Material {
    pub coloration: Coloration,
    pub albedo: f32,
    pub surface: Surface,
}

#[derive(Clone, Copy, Debug)]
pub struct TextureCoords {
    pub x: f32,
    pub y: f32,
}

#[derive(Clone, Debug, Deserialize)]
pub enum Coloration {
    Color(Color),
    Texture(Texture),
}

#[derive(Clone, Deserialize)]
pub struct Texture {
    #[serde(deserialize_with = "load_texture")]
    pub image: DynamicImage,
    pub x_offset: f32,
    pub y_offset: f32,
}

fn load_texture<D>(deserializer: D) -> Result<DynamicImage, D::Error>
    where D: serde::Deserializer
{
    use image;
    use serde::de::Error;
    use serde::Deserialize;

    let path_string = String::deserialize(deserializer)?;
    image::open(&path_string).map_err(|err| {
                                         Error::custom(format!("Could not load texture file {}: {}",
                                                               path_string,
                                                               err))
                                     })
}

#[derive(Clone, Copy, Debug, Deserialize)]
pub enum Surface {
    Diffuse,
    Reflecting { reflectivity: f32 },
    Refractive { index: f32, transparency: f32 },
}

impl Material {
    pub fn color(&self, texture_coords: &TextureCoords) -> Color {
        self.coloration.color(texture_coords)
    }
}

impl Texture {
    pub fn color(&self, texture_coords: &TextureCoords) -> Color {
        let x = Texture::wrap(texture_coords.x + self.x_offset, self.image.width());
        let y = Texture::wrap(texture_coords.y + self.y_offset, self.image.height());

        Color::from_rgba(self.image.get_pixel(x, y))
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

impl Coloration {
    pub fn color(&self, texture_coords: &TextureCoords) -> Color {
        match self {
            &Coloration::Color(ref c) => c.clone(),
            &Coloration::Texture(ref texture) => texture.color(texture_coords),
        }
    }
}

impl fmt::Debug for Texture {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f,
               "Texture {}×{} (offset x {}, y {})",
               self.image.width(),
               self.image.height(),
               self.x_offset,
               self.y_offset)
    }
}
