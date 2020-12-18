use super::log;
use super::{RenderModel,RenderTransform};
use nalgebra::{Vector2, Vector3};
pub struct Model {
    pub vertices: Vec<Vector3<f32>>,
}
pub trait GameObject {
    fn get_model(&self) -> Model;
    fn is_initilized(&self) -> bool;
    fn get_render_model(&self) -> (Option<RenderModel>,RenderTransform);
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
                log(&format!("{}", pos));
                //first traigne
                verticies.push(scale * (Vector3::new(0.0, 0.0, 0.0) + pos));
                verticies.push(scale * (Vector3::new(1.0, 0.0, 1.0) + pos));
                verticies.push(scale * (Vector3::new(1.0, 0.0, 0.0) + pos));
                //second triangle
                verticies.push(scale * (Vector3::new(0.0, 0.0, 0.0) + pos));
                verticies.push(scale * (Vector3::new(0.0, 0.0, 1.0) + pos));
                verticies.push(scale * (Vector3::new(1.0, 0.0, 1.0) + pos));
            }
        }
        Model {
            vertices: verticies,
        }
    }
    fn is_initilized(&self) -> bool {
        self.model.is_some()
    }
    fn get_render_model(&self) ->(Option<RenderModel>,RenderTransform) {
        (self.model.clone(),RenderTransform::new_scale(&Vector3::new(1.0,1.0,1.0)))
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
    fn get_render_model(&self) ->(Option<RenderModel>,RenderTransform) {
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
    pub fn new() ->Box<dyn GameObject>{
        Box::new(SimpleActor::new(Self {}))
    }
}
impl ActorBehavior for Skiier {
    fn get_model(&self) -> Model {
        let vertices = vec![
            Vector3::new(-1.0,-1.0,1.0),
            Vector3::new(1.0,1.0,1.0),
            Vector3::new(1.0,-1.0,1.0),
            //second triangle
            Vector3::new(-1.0,-1.0,1.0),
            Vector3::new(-1.0,1.0,1.0),
            Vector3::new(1.0,1.0,1.0),
            //third triangle
            Vector3::new(1.0,-1.0,1.0),
            Vector3::new(1.0,1.0,-1.0),
            Vector3::new(1.0,-1.0,-1.0),
            //fourth triangle
            Vector3::new(1.0,-1.0,1.0),
            Vector3::new(1.0,1.0,1.0),
            Vector3::new(1.0,1.0,-1.0),
            //fith triangle
            Vector3::new(1.0,-1.0,-1.0),
            Vector3::new(-1.0,-1.0,-1.0),
            Vector3::new(1.0,1.0,-1.0),
            //sixth triangle
            Vector3::new(-1.0,-1.0,-1.0),
            Vector3::new(-1.0,1.0,-1.0),
            Vector3::new(1.0,1.0,-1.0),
            //seventh triangle
            Vector3::new(-1.0,-1.0,-1.0),
            Vector3::new(-1.0,-1.0,1.0),
            Vector3::new(-1.0,1.0,1.0),
            //eighth triangle
            Vector3::new(-1.0,-1.0,-1.0),
            Vector3::new(-1.0,1.0,1.0),
            Vector3::new(-1.0,1.0,-1.0),
            //9th triangle
            Vector3::new(1.0,1.0,1.0),
            Vector3::new(1.0,1.0,-1.0),
            Vector3::new(-1.0,1.0,-1.0),
            //10th triangle
            Vector3::new(1.0,1.0,1.0),
            Vector3::new(-1.0,1.0,-1.0),
            Vector3::new(-1.0,1.0,1.0),
            //11th triangle
            Vector3::new(1.0,-1.0,1.0),
            Vector3::new(-1.0,-1.0,1.0),
            Vector3::new(-1.0,-1.0,-1.0),
            //12th triangle
            Vector3::new(1.0,-1.0,1.0),
            Vector3::new(-1.0,-1.0,-1.0),
            Vector3::new(1.0,-1.0,-1.0),

        ];
        Model { vertices }
    }
    fn get_render_transform(&self) -> RenderTransform {
        RenderTransform::new_scale(&Vector3::new(0.1,0.1,0.1))
    }
}
