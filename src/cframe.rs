use std::ops::Mul;

use rbx_dom_weak::types::{CFrame, Matrix3, Vector3};

#[derive(Clone)]
pub struct CoordinateFrame {
    matrix: [[f32; 4]; 4],
}

impl Default for CoordinateFrame {
    fn default() -> Self {
        Self {
            matrix: [
                [1.0, 0.0, 0.0, 0.0],
                [0.0, 1.0, 0.0, 0.0],
                [0.0, 0.0, 1.0, 0.0],
                [0.0, 0.0, 0.0, 1.0],
            ],
        }
    }
}

impl From<CoordinateFrame> for CFrame {
    fn from(value: CoordinateFrame) -> Self {
        Self::new(value.position(), value.rotation_matrix())
    }
}

impl CoordinateFrame {
    pub fn new(x: f32, y: f32, z: f32) -> Self {
        Self {
            matrix: [
                [1.0, 0.0, 0.0, x],
                [0.0, 1.0, 0.0, y],
                [0.0, 0.0, 1.0, z],
                [0.0, 0.0, 0.0, 1.0],
            ],
        }
    }

    pub fn from_rotation(x: f32, y: f32, z: f32, rot: [f32; 9]) -> Self {
        Self {
            matrix: [
                [rot[0], rot[1], rot[2], x],
                [rot[3], rot[4], rot[5], y],
                [rot[6], rot[7], rot[8], z],
                [0.0, 0.0, 0.0, 1.0],
            ],
        }
    }

    pub fn angles(x: f32, y: f32, z: f32) -> Self {
        Self::rz(z) * Self::ry(y) * Self::rx(x)
    }

    pub fn rx(angle: f32) -> Self {
        Self {
            matrix: [
                [1.0, 0.0, 0.0, 0.0],
                [0.0, angle.cos(), -angle.sin(), 0.0],
                [0.0, angle.sin(), angle.cos(), 0.0],
                [0.0, 0.0, 0.0, 1.0],
            ],
        }
    }

    pub fn ry(angle: f32) -> Self {
        Self {
            matrix: [
                [angle.cos(), 0.0, angle.sin(), 0.0],
                [0.0, 1.0, 0.0, 0.0],
                [-angle.sin(), 0.0, angle.cos(), 0.0],
                [0.0, 0.0, 0.0, 1.0],
            ],
        }
    }

    pub fn rz(angle: f32) -> Self {
        Self {
            matrix: [
                [angle.cos(), -angle.sin(), 0.0, 0.0],
                [angle.sin(), angle.cos(), 0.0, 0.0],
                [0.0, 0.0, 1.0, 0.0],
                [0.0, 0.0, 0.0, 1.0],
            ],
        }
    }

    pub fn position(&self) -> Vector3 {
        Vector3::new(self.matrix[0][3], self.matrix[1][3], self.matrix[2][3])
    }

    pub fn rotation_matrix(&self) -> Matrix3 {
        Matrix3::new(
            Vector3::new(self.matrix[0][0], self.matrix[0][1], self.matrix[0][2]),
            Vector3::new(self.matrix[1][0], self.matrix[1][1], self.matrix[1][2]),
            Vector3::new(self.matrix[2][0], self.matrix[2][1], self.matrix[2][2]),
        )
    }
}

impl Mul for CoordinateFrame {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        let mut matrix = [[0.0; 4]; 4];
        let a = self.matrix;
        let b = rhs.matrix;

        for i in 0..4 {
            for j in 0..4 {
                for k in 0..4 {
                    matrix[i][j] += a[i][k] * b[k][j];
                }
            }
        }

        Self { matrix }
    }
}
