extern crate image;

// TODO: Make this an attribute of the scene
const SHADOW_BIAS: f64 = 1e-13;

mod bodies;
mod color;
mod lights;
mod point;
mod ray;
mod scene;
mod vector;
pub mod material;

pub use bodies::{Body, Sphere, Plane};
pub use color::Color;
pub use lights::{Light, DirectionalLight, SphericalLight};
pub use point::Point;
pub use ray::Ray;
pub use scene::{Scene, render};
pub use vector::Vector3;
