mod camera;
mod game;
mod utils;
use camera::Camera;
use js_sys::{Array as JsArray, Map as JsMap};
use nalgebra::{Matrix4, Vector2, Vector3};
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{
    WebGl2RenderingContext, WebGlBuffer, WebGlProgram, WebGlShader, WebGlTexture,
    WebGlUniformLocation, WebGlVertexArrayObject,
};
pub fn log(s: &str) {
    web_sys::console::log(&JsArray::from(&JsValue::from(s)));
}
pub fn log_js_value(s: &JsValue){
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
#[wasm_bindgen]
pub struct GraphicsContext {
    game_objects: Vec<Box<dyn game::GameObject>>,
    gl_context: WebGl2RenderingContext,
    program: WebGlProgram,
    camera: Camera,
    position_attribute_location: i32,
    uv_attribute_location: i32,
    texture_sampler_location: Option<WebGlUniformLocation>,
}
impl GraphicsContext {
    fn render_model(
        &self,
        model: &RenderModel,
        transform: &RenderTransform,
    ) -> Result<(), JsValue> {
        let model_uniform = self.gl_context.get_uniform_location(&self.program, "model");
        self.gl_context.uniform_matrix4fv_with_f32_array(
            model_uniform.as_ref(),
            false,
            transform.matrix.as_slice(),
        );

        self.gl_context
            .bind_vertex_array(model.vertex_array_object.as_ref());
        //bind texture
        //self.gl_context.bind_texture(WebGl2RenderingContext::TEXTURE_2D,model.texture.as_ref());
        self.gl_context
            .active_texture(WebGl2RenderingContext::TEXTURE0);
        self.gl_context
            .bind_texture(WebGl2RenderingContext::TEXTURE_2D, model.texture.as_ref());
        self.gl_context
            .uniform1i(self.texture_sampler_location.as_ref(), 0);
        self.gl_context
            .draw_arrays(WebGl2RenderingContext::TRIANGLES, 0, model.count);
        Ok(())
    }
    fn init_models(&mut self) -> Result<(), JsValue> {
        for object in self.game_objects.iter_mut() {
            let model = object.get_model();
            let position_buffer = self.gl_context.create_buffer();
            let mut array: Vec<f32> = vec![];

            self.gl_context.bind_buffer(
                WebGl2RenderingContext::ARRAY_BUFFER,
                (&position_buffer).as_ref(),
            );
            for (vertex, uv) in model.vertices.iter() {
                array.push(vertex.x);
                array.push(vertex.y);
                array.push(vertex.z);
                array.push(uv.x);
                array.push(uv.y);
            }
            //  Note that `Float32Array::view` is somewhat dangerous (hence the
            // `unsafe`!). This is creating a raw view into our module's
            // `WebAssembly.Memory` buffer, but if we allocate more pages for ourself
            // (aka do a memory allocation in Rust) it'll cause the buffer to change,
            // causing the `Float32Array` to be invalid.
            unsafe {
                let vert_array = js_sys::Float32Array::view(&array);

                self.gl_context.buffer_data_with_array_buffer_view(
                    WebGl2RenderingContext::ARRAY_BUFFER,
                    &vert_array,
                    WebGl2RenderingContext::STATIC_DRAW,
                );
            }
            let vao = self.gl_context.create_vertex_array();
            self.gl_context.bind_vertex_array(vao.as_ref());
            self.gl_context
                .enable_vertex_attrib_array(self.position_attribute_location as u32);
            self.gl_context
                .enable_vertex_attrib_array(self.uv_attribute_location as u32);
            self.gl_context.vertex_attrib_pointer_with_f64(
                self.position_attribute_location as u32,
                3,
                WebGl2RenderingContext::FLOAT,
                false,
                5 * std::mem::size_of::<f32>() as i32,
                0.0,
            );
            self.gl_context.vertex_attrib_pointer_with_i32(
                self.uv_attribute_location as u32,
                2,
                WebGl2RenderingContext::FLOAT,
                false,
                5 * std::mem::size_of::<f32>() as i32,
                3 * std::mem::size_of::<f32>() as i32,
            );

            let texture = self.gl_context.create_texture();
            assert!(texture.is_some());
            self.gl_context
                .bind_texture(WebGl2RenderingContext::TEXTURE_2D, texture.as_ref());
                let texture_unit = 0;
                self.gl_context
                .active_texture(WebGl2RenderingContext::TEXTURE0 + texture_unit);
            let level = 0;
            self.gl_context.tex_image_2d_with_i32_and_i32_and_i32_and_format_and_type_and_u8_array_and_src_offset(
                WebGl2RenderingContext::TEXTURE_2D,
                level,
                //  Use RGBA Format
                WebGl2RenderingContext::RGBA as i32,
                //width
                model.texture.dimensions.x as i32,
                //height
                model.texture.dimensions.y as i32,
                //must be 0 specifies the border
                0,
                //  Use RGB Format
                WebGl2RenderingContext::RGBA,
                WebGl2RenderingContext::UNSIGNED_BYTE,
                model.texture.get_raw_vector().as_slice(),
                0,

            )?;
            //self.gl_context.generate_mipmap(WebGl2RenderingContext::TEXTURE_2D);
            //getting location of sampler



            self.gl_context
                .uniform1i(self.texture_sampler_location.as_ref(), texture_unit as i32);
            self.gl_context.tex_parameteri(
                WebGl2RenderingContext::TEXTURE_2D,
                WebGl2RenderingContext::TEXTURE_MIN_FILTER,
                WebGl2RenderingContext::LINEAR as i32,
            );
            self.gl_context.tex_parameteri(
                WebGl2RenderingContext::TEXTURE_2D,
                WebGl2RenderingContext::TEXTURE_WRAP_S,WebGl2RenderingContext::CLAMP_TO_EDGE as i32
            );
            self.gl_context.tex_parameteri(
                WebGl2RenderingContext::TEXTURE_2D,
                WebGl2RenderingContext::TEXTURE_WRAP_T,WebGl2RenderingContext::CLAMP_TO_EDGE as i32
            );
            object.submit_render_model(RenderModel {
                vertex_array_object: vao,
                count: model.vertices.len() as i32,
                position_buffer,
                texture,
            });
        }
        return Ok(());
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

        let camera_uniform = self
            .gl_context
            .get_uniform_location(&self.program, "camera");
        self.gl_context.uniform_matrix4fv_with_f32_array(
            camera_uniform.as_ref(),
            false,
            self.camera.get_mat().as_slice(),
        );
        self.gl_context.clear_color(0.2,0.2, 0.2, 1.0);
        self.gl_context
            .clear(WebGl2RenderingContext::COLOR_BUFFER_BIT);
        let model: Vec<(RenderModel, RenderTransform)> = self
            .game_objects
            .iter()
            .map(|o| {
                let (m, t) = o.get_render_model();
                (m.unwrap(), t)
            })
            .collect();
        for (model, transform) in model.iter() {
            self.render_model(model, transform)?;
        }
        let e = self.gl_context.get_error();
        if e != 0 {
            log(&(format!("error: {}", e)));
            panic!();
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
        .get_context("webgl2")?
        .unwrap()
        .dyn_into::<WebGl2RenderingContext>()?;
    let vert_shader = compile_shader(
        &context,
        WebGl2RenderingContext::VERTEX_SHADER,
        r#"#version 300 es
        in vec3 position;
        in vec2 uv;
        out vec2 o_uv;
        uniform mat4 camera;
        uniform mat4 model;
        void main() {
            gl_Position = camera*model*vec4(position,1.0);
            o_uv = uv;
        }
    "#,
    )?;
    let frag_shader = compile_shader(
        &context,
        WebGl2RenderingContext::FRAGMENT_SHADER,
        r#"#version 300 es
        precision highp float;
        out vec4 color;
        in vec2 o_uv;
        uniform sampler2D u_texture;
        void main() {
            color = texture(u_texture,o_uv);
        }
    "#,
    )?;
    let program = link_program(&context, &vert_shader, &frag_shader)?;
    context.use_program(Some(&program));
    let position_attribute_location = context.get_attrib_location(&program, "position");
    let uv_attribute_location = context.get_attrib_location(&program, "uv");
    let texture_sampler_location = context.get_uniform_location(&program, "u_texture");
    let mut g = GraphicsContext {
        gl_context: context,
        program,
        position_attribute_location,
        camera: Camera::new(Vector3::new(0.0, 0.0, 0.0), 40.0, 0.0, 0.0),
        game_objects: vec![
            Box::new(game::WorldGrid::new(Vector2::new(10, 10))),
            game::Skiier::new(),
        ],
        uv_attribute_location,
        texture_sampler_location,
    };
    g.init_models()?;
    Ok(g)
}

pub fn compile_shader(
    context: &WebGl2RenderingContext,
    shader_type: u32,
    source: &str,
) -> Result<WebGlShader, String> {
    let shader = context
        .create_shader(shader_type)
        .ok_or_else(|| String::from("Unable to create shader object"))?;
    context.shader_source(&shader, source);
    context.compile_shader(&shader);

    if context
        .get_shader_parameter(&shader, WebGl2RenderingContext::COMPILE_STATUS)
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
    context: &WebGl2RenderingContext,
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
        .get_program_parameter(&program, WebGl2RenderingContext::LINK_STATUS)
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
pub fn render_frame(context: &mut GraphicsContext, events: JsArray) {
    let events = events.iter().map(|v| Event::from_map(v.into())).collect();
    context.render_frame(events).ok().unwrap();
}
#[wasm_bindgen]
pub fn init_game() -> GraphicsContext {
    let r = start();
    if r.is_ok() {
        r.ok().unwrap()
    } else {
        log(&format!("{:?}", r.err().unwrap()));
        panic!()
    }
}
