use std::path::Path;

extern crate image;
use image::{ImageBuffer, Rgba};

extern crate time;
use time::PreciseTime;

mod color;
mod material;
mod point;
mod vector;

pub use color::Color;
pub use material::Material;
pub use point::Point;
pub use vector::Vector3;

const SHADOW_BIAS: f64 = 1e-13;

pub struct Sphere {
    pub center: Point,
    pub radius: f64,
    pub material: Material,
}

pub struct Plane {
    pub origin: Point,
    pub normal: Vector3,
    pub material: Material,
}

pub enum Body {
    Sphere(Sphere),
    Plane(Plane),
}

impl Sphere {
    pub fn surface_normal(&self, hit_point: &Point) -> Vector3 {
        (*hit_point - self.center).normalize()
    }
}

impl Plane {
    pub fn surface_normal(&self, _hit_point: &Point) -> Vector3 {
        -self.normal
    }
}

impl Body {
    pub fn material(&self) -> &Material {
        match *self {
            Body::Sphere(ref sphere) => &sphere.material,
            Body::Plane(ref plane) => &plane.material,
        }
    }

    pub fn color(&self) -> &Color {
        &self.material().color
    }

    pub fn albedo(&self) -> f32 {
        self.material().albedo
    }

    pub fn surface_normal(&self, hit_point: &Point) -> Vector3 {
        match *self {
            Body::Sphere(ref sphere) => sphere.surface_normal(hit_point),
            Body::Plane(ref plane) => plane.surface_normal(hit_point),
        }
    }
}

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
                spherical.intensity / (4.0 * std::f32::consts::PI * radius_squared)
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
            Light::Directional(_) => std::f64::INFINITY,
            Light::Spherical(ref spherical) => (spherical.position - point).length(),
        }
    }
}

pub struct Scene {
    pub width: u32,
    pub height: u32,
    pub fov: f64,
    pub bodies: Vec<Body>,
    pub lights: Vec<Light>,
}

impl Scene {
    pub fn trace(&self, ray: &Ray) -> Option<Intersection> {
        self.bodies
            .iter()
            .filter_map(|body| body.intersect(ray).map(|d| Intersection::new(d, body)))
            .min_by(|i1, i2| i1.distance.partial_cmp(&i2.distance).unwrap())
    }
}

pub struct Ray {
    pub origin: Point,
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
            origin: Point::zero(),
            direction: Vector3 {
                    x: sensor_x,
                    y: sensor_y,
                    z: -1.0, // Ray direction is straight into image
                }
                .normalize(),
        }
    }
}

trait Intersectable {
    fn intersect(&self, ray: &Ray) -> Option<f64>;
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
        let adjacent_squared = hypothenuse.dot(&ray.direction);
        // This is the same as hypothenuse.length()² - (adjacent²)², but faster.
        let opposite_squared = hypothenuse.dot(&hypothenuse) -
                               (adjacent_squared * adjacent_squared);

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
        } else {
            Some(distance0.min(distance1))
        }
    }
}

impl Intersectable for Plane {
    fn intersect(&self, ray: &Ray) -> Option<f64> {
        let normal = &self.normal;
        let denominator = normal.dot(&ray.direction);
        if denominator > 1e-6 {
            let v = self.origin - ray.origin;
            let distance = v.dot(&normal) / denominator;
            if distance >= 0.0 {
                Some(distance)
            } else {
                None
            }
        } else {
            None
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

pub fn get_color(scene: &Scene, ray: &Ray, intersection: &Intersection) -> Color {
    let hit_point = ray.origin + (ray.direction * intersection.distance);
    let surface_normal = intersection.body.surface_normal(&hit_point);

    let mut final_color = Color::black();
    let body_color = intersection.body.color();

    for light in &scene.lights {
        let direction_to_light = light.direction_from(&hit_point);

        // Calculate shadow by casting a ray from the hit point to the light and see if it's occluded
        // by a body.
        // Place origin ever so slightly above the hitpoint to avoid floating point errors where the
        // origin is inside the body itself, so the ray intersects with itself.
        let shadow_ray = Ray {
            origin: hit_point + (surface_normal * SHADOW_BIAS),
            direction: direction_to_light,
        };
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

        let light_power = (surface_normal.dot(&direction_to_light) as f32).max(0.0) *
                          light_intensity;
        let light_reflected = intersection.body.albedo() / std::f32::consts::PI;
        let light_color = light.color() * light_power * light_reflected;

        final_color = final_color + (body_color * light_color);
    }

    final_color.clamp()
}

pub fn render(scene: &Scene) -> ImageBuffer<Rgba<u8>, Vec<u8>> {
    let mut image = ImageBuffer::new(scene.width, scene.height);
    for (x, y, pixel) in image.enumerate_pixels_mut() {
        let ray = Ray::create_prime(x, y, scene);
        if let Some(intersection) = scene.trace(&ray) {
            *pixel = get_color(scene, &ray, &intersection).rgba();
        }
    }

    image
}

fn main() {
    let scene = Scene {
        width: 800,
        height: 600,
        fov: 90.0,
        bodies: vec![Body::Plane(Plane {
                                     origin: Point {
                                         x: 0.0,
                                         y: -2.0,
                                         z: -5.0,
                                     },
                                     normal: Vector3 {
                                         x: 0.0,
                                         y: -1.0,
                                         z: 0.0,
                                     },
                                     material: Material {
                                         color: Color {
                                             red: 0.7,
                                             green: 0.7,
                                             blue: 0.7,
                                         },
                                         albedo: 0.15,
                                     },
                                 }),
                     Body::Sphere(Sphere {
                                      center: Point {
                                          x: 0.0,
                                          y: 0.0,
                                          z: -5.0,
                                      },
                                      radius: 1.0,
                                      material: Material {
                                          color: Color {
                                              red: 0.1,
                                              green: 1.0,
                                              blue: 0.8,
                                          },
                                          albedo: 0.6,
                                      },
                                  }),
                     Body::Sphere(Sphere {
                                      center: Point {
                                          x: 0.0,
                                          y: 3.7,
                                          z: -5.2,
                                      },
                                      radius: 2.0,
                                      material: Material {
                                          color: Color {
                                              red: 1.0,
                                              green: 1.0,
                                              blue: 0.8,
                                          },
                                          albedo: 0.5,
                                      },
                                  }),
                     Body::Sphere(Sphere {
                                      center: Point {
                                          x: 2.0,
                                          y: 1.0,
                                          z: -8.0,
                                      },
                                      radius: 2.2,
                                      material: Material {
                                          color: Color {
                                              red: 1.0,
                                              green: 0.0,
                                              blue: 0.0,
                                          },
                                          albedo: 0.35,
                                      },
                                  })],
        lights: vec![Light::Directional(DirectionalLight {
                                            direction: Vector3 {
                                                x: 0.4,
                                                y: -1.0,
                                                z: -0.9,
                                            },
                                            color: Color {
                                                red: 1.0,
                                                green: 1.0,
                                                blue: 0.7,
                                            },
                                            intensity: 7.0,
                                        }),
                     Light::Spherical(SphericalLight {
                                          position: Point {
                                              x: -6.0,
                                              y: 3.2,
                                              z: -5.0,
                                          },
                                          color: Color {
                                              red: 0.9,
                                              blue: 0.7,
                                              green: 0.0,
                                          },
                                          intensity: 4000.0,
                                      })],
    };

    let start = PreciseTime::now();
    let image = render(&scene);
    let end = PreciseTime::now();

    println!("Took {} μseconds to render scene",
             start.to(end).num_microseconds().unwrap());

    image
        .save(&Path::new("test1.png"))
        .expect("Could not encode image");
}
