use nalgebra::{Vector2, Vector3};
pub struct Model {
    pub vertices: Vec<Vector3<f32>>,
}
pub trait GameObject {
    fn get_model(&self) -> Model;
}
pub struct WorldGrid {
    dim: Vector2<i32>,
}
impl WorldGrid {
    pub fn new(dim: Vector2<i32>) -> Self {
        Self { dim }
    }
}
impl GameObject for WorldGrid {
    fn get_model(&self) -> Model {
        let mut verticies = vec![];
        for x in 0..self.dim.x {
            for y in 0..self.dim.y {
                let pos = Vector3::new(x as f32, y as f32, 0.0);
                //first traigne
                verticies.push(Vector3::new(0.0, 0.0, 0.0) + pos);
                verticies.push(Vector3::new(1.0, 1.0, 0.0) + pos);
                verticies.push(Vector3::new(1.0, 0.0, 0.0) + pos);
                //second triangle
                verticies.push(Vector3::new(0.0, 0.0, 0.0) + pos);
                verticies.push(Vector3::new(0.0, 1.0, 0.0) + pos);
                verticies.push(Vector3::new(1.0, 1.0, 0.0) + pos);
            }
        }
        Model {
            vertices: verticies,
        }
    }
}
