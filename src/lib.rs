mod camera;
mod game;
mod utils;
use camera::Camera;
use js_sys::{Array as JsArray, Map as JsMap, Object as JsObject};
use nalgebra::{Matrix4, Perspective3, Vector2, Vector3};
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{WebGlProgram, WebGlRenderingContext, WebGlShader};
pub fn log(s: &str) {
    web_sys::console::log(&JsArray::from(&JsValue::from(s)));
}
#[derive(PartialEq)]
pub enum MouseButton {
    LeftClick,
    MiddleClick,
    RightClick,
}
pub enum Event {
    MouseMove {
        delta_x: f32,
        delta_y: f32,
        delta_time_ms: f32,
        buttons_pressed: Vec<MouseButton>,
    },
}
impl Event {
    pub fn from_map(map: JsMap) -> Self {
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
        log(&format!("delta_x: {} delta_y: {}", delta_x, delta_y));
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
#[wasm_bindgen]
pub struct GraphicsContext {
    game_objects: Vec<Box<dyn game::GameObject>>,
    gl_context: WebGlRenderingContext,
    program: WebGlProgram,
    camera: Camera,
}
impl GraphicsContext {
    fn render_model(&self, model: &game::Model) -> Result<(), JsValue> {
        let buffer = self
            .gl_context
            .create_buffer()
            .ok_or("failed to create buffer")?;
        self.gl_context
            .bind_buffer(WebGlRenderingContext::ARRAY_BUFFER, Some(&buffer));

        // Note that `Float32Array::view` is somewhat dangerous (hence the
        // `unsafe`!). This is creating a raw view into our module's
        // `WebAssembly.Memory` buffer, but if we allocate more pages for ourself
        // (aka do a memory allocation in Rust) it'll cause the buffer to change,
        // causing the `Float32Array` to be invalid.
        //
        // As a result, after `Float32Array::view` we have to be very careful not to
        // do any memory allocations before it's dropped.
        let mut array: Vec<f32> = vec![];
        for v in model.vertices.iter() {
            array.push(v.x);
            array.push(v.y);
            array.push(v.z);
        }
        unsafe {
            let vert_array = js_sys::Float32Array::view(&array);

            self.gl_context.buffer_data_with_array_buffer_view(
                WebGlRenderingContext::ARRAY_BUFFER,
                &vert_array,
                WebGlRenderingContext::STATIC_DRAW,
            );
        }

        self.gl_context.vertex_attrib_pointer_with_i32(
            0,
            3,
            WebGlRenderingContext::FLOAT,
            false,
            0,
            0,
        );
        self.gl_context.enable_vertex_attrib_array(0);

        self.gl_context.draw_arrays(
            WebGlRenderingContext::TRIANGLES,
            0,
            (model.vertices.len() / 3) as i32,
        );
        self.gl_context.delete_buffer(Some(&buffer));
        Ok(())
    }
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
                        log(&format!("contains delta_x:{} delta_y:{}", delta_x, delta_y));
                        self.camera.rotate_phi(delta_x * delta_time_ms * 0.0001);
                        self.camera.rotate_theta(delta_y * delta_time_ms * 0.0001);
                    }
                }
            }
        }
    }
    pub fn render_frame(&mut self, events: Vec<Event>) -> Result<(), JsValue> {
        self.process_events(&events);
        let camera_uniform = self
            .gl_context
            .get_uniform_location(&self.program, "camera");
        self.gl_context.uniform_matrix4fv_with_f32_array(
            camera_uniform.as_ref(),
            false,
            self.camera.get_mat().as_slice(),
        );
        self.gl_context.clear_color(0.0, 0.0, 0.0, 1.0);
        self.gl_context
            .clear(WebGlRenderingContext::COLOR_BUFFER_BIT);
        let model: Vec<game::Model> = self.game_objects.iter().map(|o| o.get_model()).collect();
        for m in model.iter() {
            self.render_model(m)?;
        }

        Ok(())
    }
}
pub fn start() -> Result<GraphicsContext, JsValue> {
    let window = web_sys::window().unwrap();
    let document = window.document().unwrap();
    let canvas = document.get_element_by_id("canvas").unwrap();
    let canvas: web_sys::HtmlCanvasElement = canvas.dyn_into::<web_sys::HtmlCanvasElement>()?;

    let context = canvas
        .get_context("webgl")?
        .unwrap()
        .dyn_into::<WebGlRenderingContext>()?;

    let vert_shader = compile_shader(
        &context,
        WebGlRenderingContext::VERTEX_SHADER,
        r#"
        attribute vec4 position;
        uniform mat4 camera;
        void main() {
            gl_Position = camera*position;
        }
    "#,
    )?;
    let frag_shader = compile_shader(
        &context,
        WebGlRenderingContext::FRAGMENT_SHADER,
        r#"
        void main() {
            gl_FragColor = vec4(1.0, 1.0, 1.0, 1.0);
        }
    "#,
    )?;
    let program = link_program(&context, &vert_shader, &frag_shader)?;
    context.use_program(Some(&program));
    Ok(GraphicsContext {
        gl_context: context,
        program,
        camera: Camera::new(Vector3::new(0.0, 0.0, 0.0), 3.0, 0.0, 0.0),
        game_objects: vec![Box::new(game::WorldGrid::new(Vector2::new(10, 10)))],
    })
}

pub fn compile_shader(
    context: &WebGlRenderingContext,
    shader_type: u32,
    source: &str,
) -> Result<WebGlShader, String> {
    let shader = context
        .create_shader(shader_type)
        .ok_or_else(|| String::from("Unable to create shader object"))?;
    context.shader_source(&shader, source);
    context.compile_shader(&shader);

    if context
        .get_shader_parameter(&shader, WebGlRenderingContext::COMPILE_STATUS)
        .as_bool()
        .unwrap_or(false)
    {
        Ok(shader)
    } else {
        Err(context
            .get_shader_info_log(&shader)
            .unwrap_or_else(|| String::from("Unknown error creating shader")))
    }
}

pub fn link_program(
    context: &WebGlRenderingContext,
    vert_shader: &WebGlShader,
    frag_shader: &WebGlShader,
) -> Result<WebGlProgram, String> {
    let program = context
        .create_program()
        .ok_or_else(|| String::from("Unable to create shader object"))?;

    context.attach_shader(&program, vert_shader);
    context.attach_shader(&program, frag_shader);
    context.link_program(&program);

    if context
        .get_program_parameter(&program, WebGlRenderingContext::LINK_STATUS)
        .as_bool()
        .unwrap_or(false)
    {
        Ok(program)
    } else {
        Err(context
            .get_program_info_log(&program)
            .unwrap_or_else(|| String::from("Unknown error creating program object")))
    }
}
// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
// allocator.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[wasm_bindgen]
extern "C" {
    fn alert(s: &str);
}
#[wasm_bindgen]
pub fn render_frame(context: &mut GraphicsContext, events: JsArray) {
    let events = events.iter().map(|v| Event::from_map(v.into())).collect();
    context.render_frame(events).ok().unwrap();
}
#[wasm_bindgen]
pub fn init_game() -> GraphicsContext {
    start().ok().unwrap()
}
