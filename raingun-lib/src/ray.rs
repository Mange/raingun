use scene::Scene;
use super::{Point3, Vector3, SHADOW_BIAS};
use cgmath::prelude::*;

pub struct Ray {
    pub origin: Point3,
    pub direction: Vector3,

    // Inverted direction is sometimes used to calculate intersections, so pre-calculate it to
    // optimize those checks.
    pub inverted_direction: Vector3,

    // This is a special structure for AABB boxes intersection checks. As AABB boxes (e.g. hit
    // boxes) will be very very common, it is worth pre-calculating some things for them and store
    // them inside the rays.
    //
    // What it represents is [x, y, z] signs, where the sign is either 0 or 1. It's 0 when the
    // direction is negative or zero in that axis, or 1 if the direction is positive.
    pub signs: [usize; 3],
}

impl Ray {
    pub fn new(origin: Point3, direction: Vector3) -> Ray {
        let invdir = 1.0 / direction;
        let x_sign = if invdir.x < 0.0 { 1 } else { 0 };
        let y_sign = if invdir.y < 0.0 { 1 } else { 0 };
        let z_sign = if invdir.z < 0.0 { 1 } else { 0 };

        Ray {
            origin: origin,
            direction: direction,
            inverted_direction: invdir,
            signs: [x_sign, y_sign, z_sign],
        }
    }

    pub fn create_prime(x: u32, y: u32, scene: &Scene, width: u32, height: u32) -> Ray {
        // Represent the camera's sensor with -1.0 at 0,0 and 1.0 at width,height.
        // Then adjust for aspect ratio and FoV

        // TODO: Support portrait images too
        assert!(width >= height);
        let aspect_ratio = width as f64 / height as f64;

        let fov_adjustment = (scene.fov.to_radians() / 2.0).tan();

        let sensor_x = (((x as f64 + 0.5) / width as f64) * 2.0 - 1.0) * aspect_ratio *
                       fov_adjustment;
        let sensor_y = (1.0 - ((y as f64 + 0.5) / height as f64) * 2.0) * fov_adjustment;

        // Ray direction is straight into image (z = -1.0)
        let direction = Vector3::new(sensor_x, sensor_y, -1.0).normalize();
        Ray::new(Point3::origin(), direction)
    }

    pub fn create_reflection(normal: Vector3, incident: Vector3, intersection: Point3) -> Ray {
        let origin = intersection + (normal * SHADOW_BIAS);
        let direction = incident - (2.0 * incident.dot(normal) * normal);
        Ray::new(origin, direction)
    }

    pub fn create_transmission(normal: Vector3,
                               incident: Vector3,
                               intersection: Point3,
                               bias: f64,
                               index: f32)
                               -> Option<Ray> {
        let mut ref_n = normal;
        let mut eta_t = index as f64;
        let mut eta_i = 1.0f64;
        let mut i_dot_n = incident.dot(normal);

        if i_dot_n < 0.0 {
            // Outside the surface
            i_dot_n = -i_dot_n;
        } else {
            // Inside the surface; invert the normal and swap the indices of refraction
            ref_n = -normal;
            eta_t = 1.0;
            eta_i = index as f64;
        }

        let eta = eta_i / eta_t;
        let k = 1.0 - (eta * eta) * (1.0 - i_dot_n * i_dot_n);

        if k < 0.0 {
            None
        } else {
            let origin = intersection + (ref_n * -bias);
            let direction = (incident + i_dot_n * ref_n) * eta - ref_n * k.sqrt();

            Some(Ray::new(origin, direction))
        }
    }

    pub fn x_sign(&self) -> usize {
        self.signs[0]
    }

    pub fn y_sign(&self) -> usize {
        self.signs[1]
    }

    pub fn z_sign(&self) -> usize {
        self.signs[2]
    }
}
