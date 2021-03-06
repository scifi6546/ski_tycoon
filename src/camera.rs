use super::log;
use nalgebra::{Matrix4, Point, Vector3};
pub struct Camera {
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
        log(&format!(
            "origin: {}, radius: {} phi: {} theta: {}",
            origin, radius, phi, theta
        ));
        Self {
            origin,
            radius,
            phi,
            theta,
        }
    }
    pub fn rotate_phi(&mut self, delta_phi: f32) {
        self.phi += delta_phi;
    }
    pub fn rotate_theta(&mut self, delta_theta: f32) {
        self.theta += delta_theta;
    }
    /// Increases by value proportional to delta radius
    pub fn update_radius(&mut self, delta_radius: f32) {
        self.radius += delta_radius * self.radius;
    }
    pub fn get_mat(&self) -> Matrix4<f32> {
        let delta_position = self.radius
            * Vector3::new(
                self.phi.cos() * self.theta.cos(),
                self.theta.sin(),
                (self.phi).sin() * self.theta.cos(),
            );
        let face = Matrix4::look_at_rh(
            &Point::from(delta_position),
            &Point::from(self.origin),
            &Vector3::new(0.0, 1.0, 0.0),
        );
        let cam = Matrix4::new_perspective(1.0, 3.14 / 3.0, 0.1, 100.0);
        let mat = cam * face;
        mat
    }
}
