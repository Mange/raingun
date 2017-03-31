use std::fs::File;
use std::path::Path;

extern crate image;
use image::{DynamicImage, GenericImage, Pixel, Rgba};

mod point;
mod vector;

pub use point::Point;
pub use vector::Vector3;

pub struct Color {
    pub red: f32,
    pub green: f32,
    pub blue: f32,
}

impl Color {
    pub fn rgba(&self) -> Rgba<u8> {
        Rgba::from_channels((self.red * 255.0) as u8,
                            (self.green * 255.0) as u8,
                            (self.blue * 255.0) as u8,
                            255)
    }
}

pub struct Sphere {
    pub center: Point,
    pub radius: f64,
    pub color: Color,
}

pub struct Scene {
    pub width: u32,
    pub height: u32,
    pub fov: f64,
    pub sphere: Sphere,
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
    fn intersect(&self, ray: &Ray) -> bool;
}

impl Intersectable for Sphere {
    fn intersect(&self, ray: &Ray) -> bool {
        // Draw a line between origin and the center of the sphere.
        // Then draw a line along the ray.
        // Treat those two lines as sides of a right-angled triangle (hypothenuse and adjacent
        // side), and calculate the third line. If this third line is longer than the radius of the
        // sphere the ray does not intersect with the sphere. If the line if shorter than the
        // radius, then the ray must pass within the sphere.

        let hypothenuse = self.center - ray.origin;
        let adjacent_squared = hypothenuse.dot(&ray.direction);
        // This is the same as hypothenuse.length()² - (adjacent²)², but faster.
        let opposite_squared = hypothenuse.dot(&hypothenuse) -
                               (adjacent_squared * adjacent_squared);

        opposite_squared < (self.radius * self.radius)
    }
}

pub fn render(scene: &Scene) -> DynamicImage {
    let mut image = DynamicImage::new_rgb8(scene.width, scene.height);
    let black = Rgba::from_channels(0, 0, 0, 0);

    for x in 0..scene.width {
        for y in 0..scene.height {
            let ray = Ray::create_prime(x, y, scene);
            if scene.sphere.intersect(&ray) {
                image.put_pixel(x, y, scene.sphere.color.rgba());
            } else {
                image.put_pixel(x, y, black);
            }
        }
    }

    image
}

#[test]
fn it_renders_scene() {
    let scene = Scene {
        width: 800,
        height: 600,
        fov: 90.0,
        sphere: Sphere {
            center: Point {
                x: 0.0,
                y: 0.0,
                z: -5.0,
            },
            radius: 1.0,
            color: Color {
                red: 0.1,
                green: 1.0,
                blue: 0.8,
            },
        },
    };

    let image: DynamicImage = render(&scene);
    assert_eq!(scene.width, image.width());
    assert_eq!(scene.height, image.height());

    let ref mut output_file =
        File::create(&Path::new("test1.png")).expect("Could not create output file");
    image
        .save(output_file, image::PNG)
        .expect("Could not encode image");
}


fn main() {
    println!("Hello, world!");
}
