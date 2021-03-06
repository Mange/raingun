use std::sync::mpsc::Sender;

use image::{ImageBuffer, Rgba, Pixel};

use bodies::*;
use color::Color;
use material::*;
use ray::Ray;
use scene::Scene;

use cgmath::prelude::*;
use super::{Point3, Vector3};

use super::SHADOW_BIAS;

use std::f32::consts::PI;

pub struct RenderedPixel {
    pub x: u32,
    pub y: u32,
    pub color: Color,
}

pub fn render_image(scene: &Scene, width: u32, height: u32) -> ImageBuffer<Rgba<u8>, Vec<u8>> {
    use rayon::prelude::*;

    let raw_image: Vec<u8> = (0u32..width * height)
        .into_par_iter()
        .map(|i| {
                 let y = i / width;
                 let x = i - y * width;
                 render_pixel(scene, x, y, width, height).rgba()
             })
        .flat_map(|pixel| pixel.channels().to_owned())
        .collect();

    ImageBuffer::from_raw(width, height, raw_image).unwrap()
}

pub fn render_image_stream(scene: &Scene,
                           width: u32,
                           height: u32,
                           channel_tx: Sender<RenderedPixel>)
                           -> () {
    use std::sync::Arc;
    use parking_lot::Mutex;
    use rayon::prelude::*;

    let channel = Arc::new(Mutex::new(channel_tx));

    (0u32..width * height)
        .into_par_iter()
        // Use .all so we can try to cancel processing if channel is closed.
        .all(|i| {
            let y = i / width;
            let x = i - y * width;
            let color = render_pixel(scene, x, y, width, height);

            let receipt = channel
                .lock()
                .send(RenderedPixel {
                          x: x,
                          y: y,
                          color: color,
                      });

            receipt.is_ok()
        });
}

fn render_pixel(scene: &Scene, x: u32, y: u32, width: u32, height: u32) -> Color {
    let ray = Ray::create_prime(x, y, scene, width, height);
    if let Some(intersection) = scene.trace(&ray) {
        get_color(scene, &ray, &intersection, 0)
    } else {
        scene.default_color
    }
}

fn get_color(scene: &Scene, ray: &Ray, intersection: &Intersection, depth: u32) -> Color {
    let hit_point = ray.origin + (ray.direction * intersection.distance);
    let body = intersection.body;
    let surface_normal = body.surface_normal(&hit_point);
    let material = body.material();

    match material.surface {
        Surface::Diffuse => shade_diffuse(scene, body, &hit_point, &surface_normal),
        Surface::Reflecting { reflectivity } => {
            let diffuse_color = shade_diffuse(scene, body, &hit_point, &surface_normal);
            let reflection_ray = Ray::create_reflection(surface_normal, ray.direction, hit_point);
            (diffuse_color * (1.0 - reflectivity)) +
            (cast_ray(scene, &reflection_ray, depth + 1) * reflectivity)
        }
        Surface::Refractive { index, transparency } => {
            let refraction_color;

            let kr = fresnel(ray.direction, surface_normal, index) as f32;
            let surface_color = material.coloration.color(&body.texture_coords(&hit_point));

            if kr < 1.0 {
                let transmission_ray = Ray::create_transmission(surface_normal,
                                                                ray.direction,
                                                                hit_point,
                                                                SHADOW_BIAS,
                                                                index)
                        .unwrap();
                refraction_color = cast_ray(scene, &transmission_ray, depth + 1);
            } else {
                refraction_color = scene.default_color;
            }

            let reflection_ray = Ray::create_reflection(surface_normal, ray.direction, hit_point);
            let reflection_color = cast_ray(scene, &reflection_ray, depth + 1);

            let mut color = reflection_color * kr + refraction_color * (1.0 - kr);
            color = color * transparency * surface_color;
            color
        }
    }
}

fn cast_ray(scene: &Scene, ray: &Ray, depth: u32) -> Color {
    if depth >= scene.max_recursion_depth {
        scene.default_color
    } else {
        let intersection = scene.trace(&ray);
        intersection.map(|intersection| get_color(scene, &ray, &intersection, depth))
            .unwrap_or(scene.default_color)
    }
}

fn shade_diffuse(scene: &Scene,
                 body: &Body,
                 hit_point: &Point3,
                 surface_normal: &Vector3)
                 -> Color {
    let texture_coords = body.texture_coords(&hit_point);
    let body_color = body.color(&texture_coords);

    let mut final_color = Color::black();
    for light in &scene.lights {
        let direction_to_light = light.direction_from(&hit_point);

        // Calculate shadow by casting a ray from the hit point to the light and see if it's occluded
        // by a body.
        // Place origin ever so slightly above the hitpoint to avoid floating point errors where the
        // origin is inside the body itself, so the ray intersects with itself.
        let shadow_ray = Ray::new(hit_point + (surface_normal * SHADOW_BIAS),
                                  direction_to_light);
        let shadow_intersection = scene.trace(&shadow_ray);

        let in_light = match shadow_intersection {
            None => true,
            Some(intersection) => intersection.distance > light.distance(&hit_point),
        };

        let light_intensity = if in_light {
            light.intensity(&hit_point)
        } else {
            0.0
        };

        let light_power = (surface_normal.dot(direction_to_light) as f32).max(0.0) *
                          light_intensity;
        let light_reflected = body.albedo() / PI;
        let light_color = light.color() * light_power * light_reflected;

        final_color = final_color + (body_color * light_color);
    }

    final_color.clamp()
}

fn fresnel(incident: Vector3, normal: Vector3, index: f32) -> f64 {
    let i_dot_n = incident.dot(normal);
    let eta_i;
    let eta_t;

    if i_dot_n > 0.0 {
        eta_i = index as f64;
        eta_t = 1.0;
    } else {
        eta_i = 1.0;
        eta_t = index as f64;
    }

    let sin_t = eta_i / eta_t * (1.0 - i_dot_n * i_dot_n).max(0.0).sqrt();

    if sin_t > 1.0 {
        // Total internal reflection
        return 1.0;
    } else {
        let cos_t = (1.0 - sin_t * sin_t).max(0.0).sqrt();
        let cos_i = cos_t.abs();
        let r_s = ((eta_t * cos_i) - (eta_i * cos_t)) / ((eta_t * cos_i) + (eta_i * cos_t));
        let r_p = ((eta_i * cos_i) - (eta_t * cos_t)) / ((eta_i * cos_i) + (eta_t * cos_t));

        return (r_s * r_s + r_p * r_p) / 2.0;
    }
}
