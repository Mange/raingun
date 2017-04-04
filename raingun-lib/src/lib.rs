extern crate image;
extern crate cgmath;

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
pub use scene::{Scene, render};
pub use cgmath::prelude::*;

pub type Point2 = cgmath::Point2<f64>;
pub type Vector2 = cgmath::Vector2<f64>;

pub type Point3 = cgmath::Point3<f64>;
pub type Vector3 = cgmath::Vector3<f64>;
