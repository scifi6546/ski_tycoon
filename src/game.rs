use super::log;
use super::{RenderModel, RenderTransform};
use nalgebra::{Vector2, Vector3, Vector4};
pub struct Model {
    pub vertices: Vec<(Vector3<f32>, Vector2<f32>)>,
    pub texture: Image,
}
pub struct Image {
    pub dimensions: Vector2<u32>,
    pub data: Vec<Vector3<u8>>,
}
impl Image {
    pub fn constant_color(color: Vector3<u8>, dimensions: Vector2<u32>) -> Self {
        let mut data = vec![];
        data.reserve((dimensions.x * dimensions.y) as usize);
        for _x in 0..dimensions.x {
            for _y in 0..dimensions.y {
                data.push(color.clone());
            }
        }
        Self { data, dimensions }
    }
    pub fn get_raw_vector(&self) -> Vec<u8> {
        let mut data =
            vec![];
        data.reserve((self.dimensions.x * self.dimensions.y) as usize * 3);
        for x in 0..self.dimensions.x {
            for y in 0..self.dimensions.y {

                data.push(self.data[(x*self.dimensions.y+y) as usize].x);
                data.push(self.data[(x*self.dimensions.y+y) as usize].y);
                data.push(self.data[(x*self.dimensions.y+y) as usize].z);
            }
        }
        let s:String = self.data.iter().map(|c|format!("({} {} {})\t",c.x,c.y,c.z)).fold("".to_string(),|acc,s|acc+&s);
        let s = (0..data.len()/3).map(|i| format!("( {} {} {})\t",data[i*3+0],data[i*3+1],data[i*3+2])).fold("".to_string(),|acc,s|acc+&s);
        assert_eq!(data.len() as u32,self.dimensions.x*self.dimensions.y*3);
        return data;
    }
}
pub trait GameObject {
    fn get_model(&self) -> Model;
    fn is_initilized(&self) -> bool;
    fn get_render_model(&self) -> (Option<RenderModel>, RenderTransform);
    fn submit_render_model(&mut self, model: RenderModel);
}
pub struct WorldGrid {
    dim: Vector2<i32>,
    model: Option<RenderModel>,
}
impl WorldGrid {
    pub fn new(dim: Vector2<i32>) -> Self {
        Self { dim, model: None }
    }
}
impl GameObject for WorldGrid {
    fn get_model(&self) -> Model {
        let mut verticies = vec![];
        let scale = 1.0;
        for x in 0..self.dim.x {
            for y in 0..self.dim.y {
                let pos = Vector3::new(x as f32, 0.0, y as f32);
                //first traigne
                verticies.push((
                    scale * (Vector3::new(0.0, 0.0, 0.0) + pos),
                    Vector2::new(0.0, 0.0),
                ));
                verticies.push((
                    scale * (Vector3::new(1.0, 0.0, 1.0) + pos),
                    Vector2::new(1.0, 1.0),
                ));
                verticies.push((
                    scale * (Vector3::new(1.0, 0.0, 0.0) + pos),
                    Vector2::new(1.0, 0.0),
                ));
                //second triangle
                verticies.push((
                    scale * (Vector3::new(0.0, 0.0, 0.0) + pos),
                    Vector2::new(0.0, 0.0),
                ));
                verticies.push((
                    scale * (Vector3::new(0.0, 0.0, 1.0) + pos),
                    Vector2::new(0.0, 1.0),
                ));
                verticies.push((
                    scale * (Vector3::new(1.0, 0.0, 1.0) + pos),
                    Vector2::new(1.0, 1.0),
                ));
            }
        }
        Model {
            vertices: verticies,
            texture: Image::constant_color(
                Vector3::new(0, 255, 255),
                Vector2::new(8, 8),
            ),
        }
    }
    fn is_initilized(&self) -> bool {
        self.model.is_some()
    }
    fn get_render_model(&self) -> (Option<RenderModel>, RenderTransform) {
        (
            self.model.clone(),
            RenderTransform::new_scale(&Vector3::new(1.0, 1.0, 1.0)),
        )
    }
    fn submit_render_model(&mut self, model: RenderModel) {
        self.model = Some(model);
    }
}
/// Used for a simple Actor that moves in the game world
struct SimpleActor<Actor: ActorBehavior> {
    actor: Actor,
    render_model: Option<RenderModel>,
}
impl<Actor: ActorBehavior> GameObject for SimpleActor<Actor> {
    fn get_model(&self) -> Model {
        self.actor.get_model()
    }
    fn submit_render_model(&mut self, model: RenderModel) {
        self.render_model = Some(model);
    }
    fn is_initilized(&self) -> bool {
        self.render_model.is_some()
    }
    fn get_render_model(&self) -> (Option<RenderModel>, RenderTransform) {
        (self.render_model.clone(), self.actor.get_render_transform())
    }
}
impl<Actor: ActorBehavior> SimpleActor<Actor> {
    pub fn new(actor: Actor) -> Self {
        Self {
            actor,
            render_model: None,
        }
    }
}
trait ActorBehavior {
    fn get_model(&self) -> Model;
    // Type todo. Will get the render transform every frame
    fn get_render_transform(&self) -> RenderTransform;
}
pub struct Skiier {}
impl Skiier {
    pub fn new() -> Box<dyn GameObject> {
        Box::new(SimpleActor::new(Self {}))
    }
}
impl ActorBehavior for Skiier {
    fn get_model(&self) -> Model {
        let vertices = vec![
            (Vector3::new(-1.0, -1.0, 1.0), Vector2::new(0.0, 0.0)),
            (Vector3::new(1.0, 1.0, 1.0), Vector2::new(1.0, 1.0)),
            (Vector3::new(1.0, -1.0, 1.0), Vector2::new(1.0, 0.0)),
            //second triangle
            (Vector3::new(-1.0, -1.0, 1.0), Vector2::new(0.0, 0.0)),
            (Vector3::new(-1.0, 1.0, 1.0), Vector2::new(0.0, 1.0)),
            (Vector3::new(1.0, 1.0, 1.0), Vector2::new(1.0, 1.0)),
            //third triangle
            (Vector3::new(1.0, -1.0, 1.0), Vector2::new(0.0, 0.0)),
            (Vector3::new(1.0, 1.0, -1.0), Vector2::new(0.0, 1.0)),
            (Vector3::new(1.0, -1.0, -1.0), Vector2::new(0.0, 1.0)),
            //fourth triangle
            (Vector3::new(1.0, -1.0, 1.0), Vector2::new(0.0, 0.0)),
            (Vector3::new(1.0, 1.0, 1.0), Vector2::new(0.0, 1.0)),
            (Vector3::new(1.0, 1.0, -1.0), Vector2::new(1.0, 1.0)),
            //fith triangle
            (Vector3::new(1.0, -1.0, -1.0), Vector2::new(0.0, 0.0)),
            (Vector3::new(-1.0, -1.0, -1.0), Vector2::new(1.0, 0.0)),
            (Vector3::new(1.0, 1.0, -1.0), Vector2::new(0.0, 1.0)),
            //sixth triangle
            (Vector3::new(-1.0, -1.0, -1.0), Vector2::new(1.0, 0.0)),
            (Vector3::new(-1.0, 1.0, -1.0), Vector2::new(1.0, 1.0)),
            (Vector3::new(1.0, 1.0, -1.0), Vector2::new(0.0, 1.0)),
            //seventh triangle
            (Vector3::new(-1.0, -1.0, -1.0), Vector2::new(0.0, 0.0)),
            (Vector3::new(-1.0, -1.0, 1.0), Vector2::new(1.0, 0.0)),
            (Vector3::new(-1.0, 1.0, 1.0), Vector2::new(1.0, 1.0)),
            //eighth triangle
            (Vector3::new(-1.0, -1.0, -1.0), Vector2::new(0.0, 0.0)),
            (Vector3::new(-1.0, 1.0, 1.0), Vector2::new(1.0, 1.0)),
            (Vector3::new(-1.0, 1.0, -1.0), Vector2::new(0.0, 1.0)),
            //9th triangle
            (Vector3::new(1.0, 1.0, 1.0), Vector2::new(0.0, 0.0)),
            (Vector3::new(1.0, 1.0, -1.0), Vector2::new(0.0, 1.0)),
            (Vector3::new(-1.0, 1.0, -1.0), Vector2::new(1.0, 1.0)),
            //10th triangle
            (Vector3::new(1.0, 1.0, 1.0), Vector2::new(0.0, 0.0)),
            (Vector3::new(-1.0, 1.0, -1.0), Vector2::new(1.0, 1.0)),
            (Vector3::new(-1.0, 1.0, 1.0), Vector2::new(0.0, 1.0)),
            //11th triangle
            (Vector3::new(1.0, -1.0, 1.0), Vector2::new(0.0, 0.0)),
            (Vector3::new(-1.0, -1.0, 1.0), Vector2::new(1.0, 0.0)),
            (Vector3::new(-1.0, -1.0, -1.0), Vector2::new(1.0, 1.0)),
            //12th triangle
            (Vector3::new(1.0, -1.0, 1.0), Vector2::new(0.0, 0.0)),
            (Vector3::new(-1.0, -1.0, -1.0), Vector2::new(1.0, 1.0)),
            (Vector3::new(1.0, -1.0, -1.0), Vector2::new(0.0, 1.0)),
        ];
        Model {
            vertices,
            texture: Image::constant_color(
                Vector3::new(255, 0, 0),
                Vector2::new(8, 8),
            ),
        }
    }
    fn get_render_transform(&self) -> RenderTransform {
        RenderTransform::new_scale(&Vector3::new(0.1, 0.1, 0.1))
    }
}
