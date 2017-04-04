use scene::Scene;
use super::{Point3, Vector3, SHADOW_BIAS};
use cgmath::prelude::*;

pub struct Ray {
    pub origin: Point3,
    pub direction: Vector3,
}

impl Ray {
    pub fn create_prime(x: u32, y: u32, scene: &Scene) -> Ray {
        // Represent the camera's sensor with -1.0 at 0,0 and 1.0 at width,height.
        // Then adjust for aspect ratio and FoV

        // TODO: Support portrait images too
        assert!(scene.width >= scene.height);
        let aspect_ratio = scene.width as f64 / scene.height as f64;

        let fov_adjustment = (scene.fov.to_radians() / 2.0).tan();

        let sensor_x = (((x as f64 + 0.5) / scene.width as f64) * 2.0 - 1.0) * aspect_ratio *
                       fov_adjustment;
        let sensor_y = (1.0 - ((y as f64 + 0.5) / scene.height as f64) * 2.0) * fov_adjustment;

        Ray {
            origin: Point3::origin(),
            // Ray direction is straight into image (z = -1.0)
            direction: Vector3::new(sensor_x, sensor_y, -1.0).normalize(),
        }
    }

    pub fn create_reflection(normal: Vector3, incident: Vector3, intersection: Point3) -> Ray {
        Ray {
            origin: intersection + (normal * SHADOW_BIAS),
            direction: incident - (2.0 * incident.dot(normal) * normal),
        }
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
            Some(Ray {
                     origin: intersection + (ref_n * -bias),
                     direction: (incident + i_dot_n * ref_n) * eta - ref_n * k.sqrt(),
                 })
        }
    }
}
