use super::log;
use nalgebra::{Matrix4, Perspective3, Point, Vector3};
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
    pub fn rotate_phi(&mut self, delta_phi: f32) {
        self.phi += delta_phi;
    }
    pub fn rotate_theta(&mut self, delta_theta: f32) {
        self.theta += delta_theta;
    }
    pub fn get_mat(&self) -> Matrix4<f32> {
        let delta_position = self.radius
            * Vector3::new(
                self.phi.cos() * self.theta.cos(),
                self.theta.sin(),
                (-1.0*self.phi).sin() * self.theta.cos(),
            );
        //log(&format!("phi: {}, theta: {} radius: {}", self.phi, self.theta,self.radius));
        log(&format!("delta position: {} origin: {}",delta_position,self.origin));

        Matrix4::new_translation(&(1.0*(delta_position + self.origin)))
             *Matrix4::face_towards(&Point::from(delta_position),
                &Point::from(self.origin),
                
                &Vector3::new(0.0, 1.0, 0.0),
            )
            * Matrix4::new_perspective(1.0, 3.14 / 4.0, 0.1, 1000.0)
        //log(&format!("{}",mat));
        //mat
    }
}
