use bodies::*;
use color::Color;
use image::{ImageBuffer, Rgba};
use lights::*;
use ray::Ray;
use rendering;
use rendering::RenderedPixel;

use std::sync::mpsc::Sender;

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields, default, rename_all = "camelCase")]
pub struct Scene {
    pub fov: f64,
    pub default_color: Color,
    pub max_recursion_depth: u32,
    pub bodies: Vec<Body>,
    pub lights: Vec<Light>,
}

impl Default for Scene {
    fn default() -> Scene {
        Scene {
            fov: 90.0,
            default_color: Color::default(),
            max_recursion_depth: 10,
            lights: Vec::default(),
            bodies: Vec::default(),
        }
    }
}

impl Scene {
    pub fn trace(&self, ray: &Ray) -> Option<Intersection> {
        self.bodies
            .iter()
            .filter_map(|body| body.intersect(ray).map(|d| Intersection::new(d, body)))
            .min_by(|i1, i2| i1.distance.partial_cmp(&i2.distance).unwrap())
    }

    pub fn render_image(&self, width: u32, height: u32) -> ImageBuffer<Rgba<u8>, Vec<u8>> {
        rendering::render_image(self, width, height)
    }

    pub fn streaming_render(&self,
                            width: u32,
                            height: u32,
                            channel_tx: Sender<RenderedPixel>)
                            -> () {
        rendering::render_image_stream(self, width, height, channel_tx)
    }
}
