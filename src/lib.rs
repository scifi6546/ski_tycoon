mod camera;
mod game;
mod graphics_engine;
mod utils;
use camera::Camera;
use graphics_engine::GraphicsEngine;
use js_sys::{Array as JsArray, Map as JsMap};
use nalgebra::{Matrix4, Vector2, Vector3,Vector4};
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
pub use graphics_engine::{Mesh,RGBATexture};
use web_sys::{
    WebGl2RenderingContext, WebGlBuffer, WebGlProgram, WebGlShader, WebGlTexture,
    WebGlUniformLocation, WebGlVertexArrayObject,
};
pub struct Model{
    pub mesh: Mesh,
    pub texture:  RGBATexture,
}
pub fn log(s: &str) {
    web_sys::console::log(&JsArray::from(&JsValue::from(s)));
}
pub fn log_js_value(s: &JsValue) {
    web_sys::console::log(&JsArray::from(s));
}
#[derive(PartialEq)]
pub enum MouseButton {
    LeftClick,
    MiddleClick,
    RightClick,
}
#[derive(Clone, Debug)]
pub struct RenderModel {
    vertex_array_object: Option<WebGlVertexArrayObject>,
    position_buffer: Option<WebGlBuffer>,
    count: i32,
    texture: Option<WebGlTexture>,
}
#[derive(Clone, Debug)]
pub struct RenderTransform {
    matrix: Matrix4<f32>,
}
impl RenderTransform {
    pub fn new_scale(scale: &Vector3<f32>) -> Self {
        Self {
            matrix: Matrix4::new_nonuniform_scaling(scale),
        }
    }
}
pub enum Event {
    MouseMove {
        delta_x: f32,
        delta_y: f32,
        delta_time_ms: f32,
        buttons_pressed: Vec<MouseButton>,
    },
    Scroll {
        delta_y: f32,
        delta_time_ms: f32,
    },
}
impl Event {
    pub fn from_map(map: JsMap) -> Self {
        let name: String = map.get(&JsValue::from_str("name")).as_string().unwrap();
        match name.as_str() {
            "mouse_move" => Self::from_mouse_move_map(map),
            "wheel" => Self::from_wheel_map(map),
            _ => panic!("invalid name"),
        }
    }
    pub fn from_wheel_map(map: JsMap) -> Self {
        let delta_y = map.get(&JsValue::from_str("delta_y")).as_f64().unwrap() as f32;
        let delta_time_ms = map
            .get(&JsValue::from_str("delta_time_ms"))
            .as_f64()
            .unwrap() as f32;
        Event::Scroll {
            delta_y,
            delta_time_ms,
        }
    }
    pub fn from_mouse_move_map(map: JsMap) -> Self {
        let buttons_pressed_number: i32 =
            map.get(&JsValue::from_str("buttons")).as_f64().unwrap() as i32;
        let buttons_pressed = match buttons_pressed_number {
            0 => vec![],
            1 => vec![MouseButton::LeftClick],
            2 => vec![MouseButton::RightClick],
            3 => vec![MouseButton::LeftClick, MouseButton::RightClick],
            4 => vec![MouseButton::MiddleClick],
            5 => vec![MouseButton::LeftClick, MouseButton::MiddleClick],
            6 => vec![MouseButton::MiddleClick, MouseButton::RightClick],
            7 => vec![
                MouseButton::LeftClick,
                MouseButton::MiddleClick,
                MouseButton::RightClick,
            ],
            _ => panic!("invalid button number"),
        };
        let delta_x = map.get(&JsValue::from_str("delta_x")).as_f64().unwrap() as f32;
        let delta_y = map.get(&JsValue::from_str("delta_y")).as_f64().unwrap() as f32;
        let delta_time_ms = map
            .get(&JsValue::from_str("delta_time_ms"))
            .as_f64()
            .unwrap() as f32;
        Event::MouseMove {
            delta_x,
            delta_y,
            buttons_pressed,
            delta_time_ms,
        }
    }
}

pub struct GraphicsContext<E:GraphicsEngine> {
    game_objects: Vec<Box<dyn game::GameObject<(E::RuntimeMesh,E::RuntimeTexture)>>>,
    camera: Camera,
    engine: E
}
impl<E:GraphicsEngine> GraphicsContext<E> {
    pub fn process_events(&mut self, events: &Vec<Event>) {
        for event in events {
            match event {
                Event::MouseMove {
                    delta_x,
                    delta_y,
                    buttons_pressed,
                    delta_time_ms,
                } => {
                    if buttons_pressed.contains(&MouseButton::RightClick) {
                        self.camera.rotate_phi(delta_x * delta_time_ms * 0.0001);
                        self.camera.rotate_theta(delta_y * delta_time_ms * 0.0001);
                    }
                }
                Event::Scroll {
                    delta_y,
                    delta_time_ms,
                } => self.camera.update_radius(delta_y * delta_time_ms * 0.0001),
            }
        }
    }
    pub fn render_frame(&mut self, events: Vec<Event>) -> Result<(), JsValue> {
        self.process_events(&events);
        self.engine.clear_screen(Vector4::new(0.2,0.2,0.2,1.0));
        self.engine.send_view_matrix(self.camera.get_mat());
        for object in self.game_objects.iter() {
            let (model_opt,trans) = object.get_render_model();
            if let Some((model,texture)) = model_opt{
                self.engine.send_model_matrix(trans.matrix);
                self.engine.bind_texture(texture);
                self.engine.draw_mesh(model);
            }
        }
        Ok(())
    }
    pub fn init_models(&mut self)->Result<(),E::ErrorType>{
        for object in self.game_objects.iter_mut(){
            let model = object.get_model();
            let mesh = self.engine.build_mesh(model.mesh)?;
            let texture = self.engine.build_texture(model.texture)?;
            object.submit_render_model((mesh,texture));
        }
        Ok(())
    }
}
pub fn start() -> Result<GraphicsContext<graphics_engine::WebGl>, JsValue> {
    let graphics = graphics_engine::WebGl::init()?;
    
    let mut g = GraphicsContext {
        engine:graphics,
        camera: Camera::new(Vector3::new(0.0, 0.0, 0.0), 40.0, 0.0, 0.0),
        game_objects: vec![
            Box::new(game::WorldGrid::new(Vector2::new(10, 10))),
            game::Skiier::new(),
        ],
    };
    g.init_models()?;
    Ok(g)
}
#[wasm_bindgen]
pub struct WebGame{
    engine: GraphicsContext<graphics_engine::WebGl>,
}
#[wasm_bindgen]
impl WebGame{
    #[wasm_bindgen]
    pub fn render_frame(&mut self,events: JsArray){
        let events = events.iter().map(|v| Event::from_map(v.into())).collect();
        
        self.engine.render_frame(events);
    }
}
// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
// allocator.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;
#[wasm_bindgen]
pub fn init_game() -> WebGame {
    let r = start();
    if r.is_ok() {
        WebGame{
            engine: r.ok().unwrap()
        }
        
    } else {
        log(&format!("{:?}", r.err().unwrap()));
        panic!()
    }
}
