use color::Color;
use material::*;
use ray::Ray;
use super::{Point3, Vector3};
use cgmath::prelude::*;

use std::f32::consts::PI;

#[derive(Debug, Clone, Deserialize)]
pub struct Sphere {
    pub center: Point3,
    pub radius: f64,
    pub material: Material,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Plane {
    pub origin: Point3,
    pub normal: Vector3,
    pub material: Material,
}

#[derive(Debug, Clone, Deserialize)]
pub enum Body {
    Sphere(Sphere),
    Plane(Plane),
}

impl Body {
    pub fn material(&self) -> &Material {
        match *self {
            Body::Sphere(ref sphere) => &sphere.material,
            Body::Plane(ref plane) => &plane.material,
        }
    }

    pub fn color(&self, texture_coords: &TextureCoords) -> Color {
        self.material().color(texture_coords)
    }

    pub fn albedo(&self) -> f32 {
        self.material().albedo
    }
}

pub trait Intersectable {
    fn intersect(&self, ray: &Ray) -> Option<f64>;

    fn surface_normal(&self, hit_point: &Point3) -> Vector3;
    fn texture_coords(&self, hit_point: &Point3) -> TextureCoords;
}

impl Intersectable for Sphere {
    fn intersect(&self, ray: &Ray) -> Option<f64> {
        // How to determine if we intersect:
        //
        // Draw a line between origin and the center of the sphere.
        // Then draw a line along the ray.
        // Treat those two lines as sides of a right-angled triangle (hypothenuse and adjacent
        // side), and calculate the third line. If this third line is longer than the radius of the
        // sphere the ray does not intersect with the sphere. If the line if shorter than the
        // radius, then the ray must pass within the sphere.
        //
        // How to calculate the distance to the intersection:
        // We know the centerpoint of the circle, as well as the radius. Intersection always
        // happens on the edge (e.g. at the radius), so we can calculate a triangle between the
        // centerpoint, the intersection point and the distance between the centerpoint and the ray
        // (that is the "opposite" line calculated in step 1).

        let hypothenuse = self.center - ray.origin;
        let adjacent_squared = hypothenuse.dot(ray.direction);
        // This is the same as hypothenuse.length()² - (adjacent²)², but faster.
        let opposite_squared = hypothenuse.dot(hypothenuse) - (adjacent_squared * adjacent_squared);

        let radius_squared = self.radius * self.radius;

        if opposite_squared > radius_squared {
            return None;
        }

        // Calculate hypotenuse length between triangle centerpoint, ray that is inside sphere and
        // intersection point.
        let thickness = (radius_squared - opposite_squared).sqrt();

        // Full disclosure: I have no idea what is happening here. I cannot figure it out.
        let distance0 = adjacent_squared - thickness;
        let distance1 = adjacent_squared + thickness;

        if distance0 < 0.0 && distance1 < 0.0 {
            None
        } else if distance0 < 0.0 {
            Some(distance1)
        } else if distance1 < 0.0 {
            Some(distance0)
        } else {
            Some(distance0.min(distance1))
        }
    }

    fn surface_normal(&self, hit_point: &Point3) -> Vector3 {
        (*hit_point - self.center).normalize()
    }

    fn texture_coords(&self, hit_point: &Point3) -> TextureCoords {
        let hit_vec = hit_point - self.center;
        TextureCoords {
            x: (1.0 + (hit_vec.z.atan2(hit_vec.x) as f32) / PI) * 0.5,
            y: (hit_vec.y / self.radius).acos() as f32 / PI,
        }
    }
}

impl Intersectable for Plane {
    fn intersect(&self, ray: &Ray) -> Option<f64> {
        let denominator = self.normal.dot(ray.direction);
        if denominator > 1e-6 {
            let v = self.origin - ray.origin;
            let distance = v.dot(self.normal) / denominator;
            if distance >= 0.0 {
                Some(distance)
            } else {
                None
            }
        } else {
            None
        }
    }

    fn surface_normal(&self, _hit_point: &Point3) -> Vector3 {
        -self.normal
    }

    fn texture_coords(&self, hit_point: &Point3) -> TextureCoords {
        let mut x_axis = self.normal.cross(Vector3::unit_z());

        if x_axis.magnitude2() == 0.0 {
            x_axis = self.normal.cross(Vector3::unit_y());
        }

        let y_axis = self.normal.cross(x_axis);

        let hit_vec = hit_point - self.origin;
        TextureCoords {
            x: hit_vec.dot(x_axis) as f32,
            y: hit_vec.dot(y_axis) as f32,
        }
    }
}

impl Intersectable for Body {
    fn intersect(&self, ray: &Ray) -> Option<f64> {
        match *self {
            Body::Sphere(ref sphere) => sphere.intersect(ray),
            Body::Plane(ref plane) => plane.intersect(ray),
        }
    }

    fn surface_normal(&self, hit_point: &Point3) -> Vector3 {
        match *self {
            Body::Sphere(ref sphere) => sphere.surface_normal(hit_point),
            Body::Plane(ref plane) => plane.surface_normal(hit_point),
        }
    }

    fn texture_coords(&self, hit_point: &Point3) -> TextureCoords {
        match *self {
            Body::Sphere(ref sphere) => sphere.texture_coords(hit_point),
            Body::Plane(ref plane) => plane.texture_coords(hit_point),
        }
    }
}

pub struct Intersection<'a> {
    pub distance: f64,
    pub body: &'a Body,
}

impl<'a> Intersection<'a> {
    pub fn new<'b>(distance: f64, body: &'b Body) -> Intersection<'b> {
        Intersection {
            distance: distance,
            body: body,
        }
    }
}
