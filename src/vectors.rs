use std::fmt::Display;

use crate::Coords;

#[derive(Clone, Copy)]
pub struct Vector3D {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

impl Vector3D {
    pub fn get_magnitude(&self) -> f64 {
        return f64::sqrt(self.x * self.x + self.y * self.y + self.z * self.z);
    }

    pub fn get_normalized(&self) -> Vector3D {
        let magnitude = self.get_magnitude();

        Vector3D {
            x: self.x / magnitude,
            y: self.y / magnitude,
            z: self.z / magnitude,
        }
    }

    pub fn to_angle(&self) -> f64 {
        let angle_radians = f64::atan2(self.x, self.z);
        let angle_degrees = angle_radians.to_degrees();

        let adjusted_angle_degrees = if angle_degrees > 180.0 {
            angle_degrees - 360.0
        } else if angle_degrees < -180.0 {
            angle_degrees + 360.0
        } else {
            angle_degrees
        };

        adjusted_angle_degrees * -1.
    }
}

impl std::ops::Sub for Vector3D {
    type Output = Vector3D;

    fn sub(self, rhs: Self) -> Self::Output {
        Vector3D {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
            z: self.z - rhs.z,
        }
    }
}

impl Display for Vector3D {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "X: {}, Y: {}, Z: {}", self.x, self.y, self.z)
    }
}

impl From<&Coords> for Vector3D {
    fn from(value: &Coords) -> Self {
        Vector3D {
            x: value.x,
            y: 0.,
            z: value.z,
        }
    }
}

impl From<(f64, f64, f64)> for Vector3D {
    fn from(value: (f64, f64, f64)) -> Self {
        Vector3D {
            x: value.0,
            y: value.1,
            z: value.2,
        }
    }
}

// pub fn estimative_walking_time(from: Vector3D, to: Vector3D) -> f64 {
//     let direction = from - to;
//     return direction.get_magnitude() / (0.21585 * 20.);
// }
