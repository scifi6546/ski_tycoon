use nalgebra::{Matrix4, Perspective3, Vector3};
pub struct Camera {
    matrix: Perspective3<f32>,
    /// Origin about which camera rotates
    origin: Vector3<f32>,
    /// how far away the camera is from its origin
    radius: f32,
    //angle
    phi: f32,
    theta: f32,
}
impl Camera {
    pub fn new(origin: Vector3<f32>, radius: f32, phi: f32, theta: f32) -> Self {
        Self {
            matrix: Perspective3::new(1.0, 3.14 / 4.0, 0.1, 1000.0),
            origin,
            radius,
            phi,
            theta,
        }
    }
    pub fn get_mat(&self) -> Matrix4<f32> {
        self.matrix.to_homogeneous()
    }
}
