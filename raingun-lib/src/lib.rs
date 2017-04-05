extern crate image;
extern crate cgmath;

extern crate serde;
#[macro_use]
extern crate serde_derive;

// TODO: Make this an attribute of the scene
const SHADOW_BIAS: f64 = 1e-13;

mod bodies;
mod color;
mod lights;
mod ray;
mod scene;
pub mod material;

pub use bodies::{Body, Sphere, Plane};
pub use color::Color;
pub use lights::{Light, DirectionalLight, SphericalLight};
pub use ray::Ray;
pub use scene::Scene;
pub use cgmath::prelude::*;

pub type Point3 = cgmath::Point3<f64>;
pub type Vector3 = cgmath::Vector3<f64>;
