use nalgebra::{Vector2, Vector3};
use super::RenderModel;
pub struct Model {
    pub vertices: Vec<Vector3<f32>>,
}
pub trait GameObject {
    fn get_model(&self) -> Model;
    fn is_initilized(&self)->bool;
    fn get_render_model(&self)->Option<RenderModel>;
    fn submit_render_model(&mut self,model:RenderModel);
}
pub struct WorldGrid {
    dim: Vector2<i32>,
    model: Option<RenderModel>,
}
impl WorldGrid {
    pub fn new(dim: Vector2<i32>) -> Self {
        Self { dim ,model:None}
    }
}
impl GameObject for WorldGrid {
    fn get_model(&self) -> Model {
        let mut verticies = vec![];
        for x in 0..self.dim.x {
            for y in 0..self.dim.y {
                let pos = Vector3::new(x as f32, 0.0, y as f32);
                //first traigne
                verticies.push(Vector3::new(0.0, 0.0, 0.0) + pos);
                verticies.push(Vector3::new(1.0, 0.0, 1.0) + pos);
                verticies.push(Vector3::new(1.0, 0.0, 0.0) + pos);
                //second triangle
                verticies.push(Vector3::new(0.0, 0.0, 0.0) + pos);
                verticies.push(Vector3::new(0.0, 0.0, 1.0) + pos);
                verticies.push(Vector3::new(1.0, 0.0, 1.0) + pos);
            }
        }
        Model {
            vertices: verticies,
        }
    }
    fn is_initilized(&self)->bool{
        self.model.is_some()
    }
    fn get_render_model(&self)->Option<RenderModel>{
        self.model.clone()
    }
    fn submit_render_model(&mut self,model: RenderModel){
        self.model=Some(model);
    }
}

