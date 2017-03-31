#[derive(Copy, Clone, Debug)]
pub struct Vector3 {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

impl Vector3 {
    pub fn zero() -> Vector3 {
        Vector3::from_one(0.0)
    }

    pub fn from_one(value: f64) -> Vector3 {
        Vector3 {
            x: value,
            y: value,
            z: value,
        }
    }

    pub fn length(&self) -> f64 {
        self.norm().sqrt()
    }

    pub fn norm(&self) -> f64 {
        (self.x * self.x + self.y * self.y + self.z * self.z)
    }

    pub fn normalize(&self) -> Vector3 {
        let inverse_length = self.length().recip();
        Vector3 {
            x: self.x * inverse_length,
            y: self.y * inverse_length,
            z: self.z * inverse_length,
        }
    }

    pub fn dot(&self, other: &Vector3) -> f64 {
        self.x * other.x + self.y * other.y + self.z * other.z
    }
}
