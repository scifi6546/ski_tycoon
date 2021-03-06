use super::prelude::{Camera, GetGuiOutput, GuiMessage, GuiParent, Triangle};
use super::{Mesh, Model, RGBATexture, RenderTransform};
use nalgebra::{Vector2, Vector3, Vector4};
pub struct ObjectTickOutput<'a, RenderModel> {
    pub model: Option<&'a RenderModel>,
    pub transform: RenderTransform,
}
pub trait GameObject<RenderModel: std::marker::Sized> {
    fn get_model(&self) -> Model;
    fn is_initilized(&self) -> bool;
    fn get_render_model(&self) -> ObjectTickOutput<RenderModel>;
    fn submit_render_model(&mut self, model: RenderModel);
    ///GuiParent Implementation
    fn get_gui_g(&self) -> GetGuiOutput<RenderModel>;
    fn process_message_g(&mut self, message: &GuiMessage);
    /// Gets collider triangle in screen coordinates
    fn get_screen_collider_g(&self, camera: &Camera) -> Vec<Triangle>;
}
impl<RenderModel: std::marker::Sized> GuiParent<RenderModel> for Box<dyn GameObject<RenderModel>> {
    fn get_gui(&self) -> GetGuiOutput<RenderModel> {
        self.get_gui_g()
    }
    fn process_message(&mut self, message: &GuiMessage) {
        self.process_message_g(message)
    }
    /// Gets collider triangle in screen coordinates
    fn get_screen_collider(&self, camera: &Camera) -> Vec<Triangle> {
        self.get_screen_collider_g(camera)
    }
}
pub struct WorldGrid<RenderModel: std::marker::Sized> {
    dim: Vector2<i32>,
    model: Option<RenderModel>,
}
impl<RenderModel: std::marker::Sized> WorldGrid<RenderModel> {
    pub fn new(dim: Vector2<i32>) -> Self {
        Self { dim, model: None }
    }
}
impl<RenderModel: std::marker::Sized> GameObject<RenderModel> for WorldGrid<RenderModel> {
    fn get_model(&self) -> Model {
        let mut vertices = vec![];
        let scale = 1.0;
        for x in 0..self.dim.x {
            for y in 0..self.dim.y {
                let pos = Vector3::new(x as f32, 0.0, y as f32);
                //first traigne
                vertices.push((
                    scale * (Vector3::new(0.0, 0.0, 0.0) + pos),
                    Vector2::new(0.0, 0.0),
                ));
                vertices.push((
                    scale * (Vector3::new(1.0, 0.0, 1.0) + pos),
                    Vector2::new(1.0, 1.0),
                ));
                vertices.push((
                    scale * (Vector3::new(1.0, 0.0, 0.0) + pos),
                    Vector2::new(1.0, 0.0),
                ));
                //second triangle
                vertices.push((
                    scale * (Vector3::new(0.0, 0.0, 0.0) + pos),
                    Vector2::new(0.0, 0.0),
                ));
                vertices.push((
                    scale * (Vector3::new(0.0, 0.0, 1.0) + pos),
                    Vector2::new(0.0, 1.0),
                ));
                vertices.push((
                    scale * (Vector3::new(1.0, 0.0, 1.0) + pos),
                    Vector2::new(1.0, 1.0),
                ));
            }
        }
        Model {
            mesh: Mesh { vertices },
            texture: RGBATexture::constant_color(
                Vector4::new(0, 255, 255, 255),
                Vector2::new(8, 8),
            ),
        }
    }
    fn is_initilized(&self) -> bool {
        self.model.is_some()
    }
    fn get_render_model(&self) -> ObjectTickOutput<RenderModel> {
        ObjectTickOutput {
            model: self.model.as_ref(),
            transform: RenderTransform::new_scale(&Vector3::new(1.0, 1.0, 1.0)),
        }
    }
    fn submit_render_model(&mut self, model: RenderModel) {
        self.model = Some(model);
    }
    fn get_gui_g(&self) -> GetGuiOutput<RenderModel> {
        GetGuiOutput::None
    }
    fn process_message_g(&mut self, message: &GuiMessage) {}
    /// Gets collider triangle in screen coordinates
    fn get_screen_collider_g(&self, camera: &Camera) -> Vec<Triangle> {
        vec![]
    }
}
/// Used for a simple Actor that moves in the game world
struct SimpleActor<Actor: ActorBehavior, RenderModel: std::marker::Sized> {
    actor: Actor,
    render_model: Option<RenderModel>,
    collider: Collider,
}
impl<Actor: ActorBehavior, RenderModel: std::marker::Sized> GameObject<RenderModel>
    for SimpleActor<Actor, RenderModel>
{
    fn get_model(&self) -> Model {
        self.actor.get_model()
    }
    fn submit_render_model(&mut self, model: RenderModel) {
        self.render_model = Some(model);
    }
    fn is_initilized(&self) -> bool {
        self.render_model.is_some()
    }
    fn get_render_model(&self) -> ObjectTickOutput<RenderModel> {
        ObjectTickOutput {
            model: self.render_model.as_ref(),
            transform: self.actor.get_render_transform(),
        }
    }
    fn get_gui_g(&self) -> GetGuiOutput<RenderModel> {
        GetGuiOutput::None
    }
    fn process_message_g(&mut self, message: &GuiMessage) {}
    /// Gets collider triangle in screen coordinates
    fn get_screen_collider_g(&self, camera: &Camera) -> Vec<Triangle> {
        self.collider.get_screenspace_collider(camera)
    }
}
struct Collider {
    mesh: Vec<Vector3<f32>>,
}
impl Collider {
    fn get_screenspace_collider(&self, camera: &Camera) -> Vec<Triangle> {
        //probabbly something to do with convex hull
        panic!("get screenspace failed")
    }
}
impl<Actor: ActorBehavior, RenderModel: std::marker::Sized> SimpleActor<Actor, RenderModel> {
    pub fn new(actor: Actor) -> Self {
        let collider = actor.get_collider();
        Self {
            actor,
            render_model: None,
            collider,
        }
    }
}
trait ActorBehavior {
    fn get_model(&self) -> Model;
    fn get_collider(&self) -> Collider;
    fn get_render_transform(&self) -> RenderTransform;
}
pub struct Skiier {}
impl Skiier {
    pub fn new<RenderModel: 'static>() -> Box<dyn GameObject<RenderModel>> {
        Box::new(SimpleActor::new(Self {}))
    }
}
impl ActorBehavior for Skiier {
    fn get_collider(&self) -> Collider {
        Collider {
            mesh: vec![
                Vector3::new(-1.0, -1.0, 1.0),
                Vector3::new(1.0, 1.0, 1.0),
                Vector3::new(1.0, -1.0, 1.0),
                //second triangle
                Vector3::new(-1.0, -1.0, 1.0),
                Vector3::new(-1.0, 1.0, 1.0),
                Vector3::new(1.0, 1.0, 1.0),
                //third triangle
                Vector3::new(1.0, -1.0, 1.0),
                Vector3::new(1.0, 1.0, -1.0),
                Vector3::new(1.0, -1.0, -1.0),
                //fourth triangle
                Vector3::new(1.0, -1.0, 1.0),
                Vector3::new(1.0, 1.0, 1.0),
                Vector3::new(1.0, 1.0, -1.0),
                //fith triangle
                Vector3::new(1.0, -1.0, -1.0),
                Vector3::new(-1.0, -1.0, -1.0),
                Vector3::new(1.0, 1.0, -1.0),
                //sixth triangle
                Vector3::new(-1.0, -1.0, -1.0),
                Vector3::new(-1.0, 1.0, -1.0),
                Vector3::new(1.0, 1.0, -1.0),
                //seventh triangle
                Vector3::new(-1.0, -1.0, -1.0),
                Vector3::new(-1.0, -1.0, 1.0),
                Vector3::new(-1.0, 1.0, 1.0),
                //eighth triangle
                Vector3::new(-1.0, -1.0, -1.0),
                Vector3::new(-1.0, 1.0, 1.0),
                Vector3::new(-1.0, 1.0, -1.0),
                //9th triangle
                Vector3::new(1.0, 1.0, 1.0),
                Vector3::new(1.0, 1.0, -1.0),
                Vector3::new(-1.0, 1.0, -1.0),
                //10th triangle
                Vector3::new(1.0, 1.0, 1.0),
                Vector3::new(-1.0, 1.0, -1.0),
                Vector3::new(-1.0, 1.0, 1.0),
                //11th triangle
                Vector3::new(1.0, -1.0, 1.0),
                Vector3::new(-1.0, -1.0, 1.0),
                Vector3::new(-1.0, -1.0, -1.0),
                //12th triangle
                Vector3::new(1.0, -1.0, 1.0),
                Vector3::new(-1.0, -1.0, -1.0),
                Vector3::new(1.0, -1.0, -1.0),
            ],
        }
    }
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
            mesh: Mesh { vertices },
            texture: RGBATexture::constant_color(Vector4::new(255, 0, 0, 255), Vector2::new(8, 8)),
        }
    }
    fn get_render_transform(&self) -> RenderTransform {
        RenderTransform::new_scale(&Vector3::new(0.1, 0.1, 0.1))
    }
}
