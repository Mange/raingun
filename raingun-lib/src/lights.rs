use color::Color;
use point::Point;
use vector::Vector3;

use std::f32::consts::PI;
use std::f64::INFINITY;

pub struct DirectionalLight {
    pub direction: Vector3,
    pub color: Color,
    pub intensity: f32,
}

pub struct SphericalLight {
    pub position: Point,
    pub color: Color,
    pub intensity: f32,
}

pub enum Light {
    Directional(DirectionalLight),
    Spherical(SphericalLight),
}

impl Light {
    pub fn color(&self) -> &Color {
        match *self {
            Light::Directional(ref directional) => &directional.color,
            Light::Spherical(ref spherical) => &spherical.color,
        }
    }

    pub fn intensity(&self, hit_point: &Point) -> f32 {
        match *self {
            Light::Directional(ref directional) => directional.intensity,
            Light::Spherical(ref spherical) => {
                let radius_squared = (spherical.position - hit_point).norm() as f32;
                spherical.intensity / (4.0 * PI * radius_squared)
            }
        }
    }

    pub fn direction_from(&self, point: &Point) -> Vector3 {
        match *self {
            Light::Directional(ref directional) => (-directional.direction).normalize(),
            Light::Spherical(ref spherical) => (spherical.position - point).normalize(),
        }
    }

    pub fn distance(&self, point: &Point) -> f64 {
        match *self {
            Light::Directional(_) => INFINITY,
            Light::Spherical(ref spherical) => (spherical.position - point).length(),
        }
    }
}
