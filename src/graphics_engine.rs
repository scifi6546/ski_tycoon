use nalgebra::{Matrix4, Vector2, Vector3, Vector4};
use wasm_bindgen::{JsCast, JsValue};
use web_sys::{
    WebGl2RenderingContext, WebGlBuffer, WebGlFramebuffer, WebGlProgram, WebGlShader, WebGlTexture,
    WebGlUniformLocation, WebGlVertexArrayObject,
};
#[derive(Clone)]
pub struct Mesh {
    pub vertices: Vec<(Vector3<f32>, Vector2<f32>)>,
}
#[derive(Clone)]
pub struct RGBATexture {
    dimensions: Vector2<u32>,
    pixels: Vec<Vector4<u8>>,
}
impl RGBATexture {
    pub fn get_raw_vector(&self) -> Vec<u8> {
        let mut v = vec![];
        v.reserve((self.dimensions.x * self.dimensions.y * 4) as usize);
        for pixel in self.pixels.iter() {
            v.push(pixel.x);
            v.push(pixel.y);
            v.push(pixel.z);
            v.push(pixel.w);
        }
        return v;
    }
    pub fn constant_color(color: Vector4<u8>, dimensions: Vector2<u32>) -> Self {
        let pixels = (0..(dimensions.x * dimensions.y))
            .map(|_| color.clone())
            .collect();
        Self { dimensions, pixels }
    }
}
pub trait GraphicsEngine: std::marker::Sized {
    type RuntimeMesh: Clone;
    type RuntimeTexture: Clone;
    type ErrorType;
    type Framebuffer;
    fn init() -> Result<Self, Self::ErrorType>;
    fn build_mesh(&mut self, mesh: Mesh) -> Result<Self::RuntimeMesh, Self::ErrorType>;
    fn build_texture(
        &mut self,
        texture: RGBATexture,
    ) -> Result<Self::RuntimeTexture, Self::ErrorType>;
    fn build_framebuffer(
        &mut self,
        texture_attachment: &mut Self::RuntimeTexture,
    ) -> Self::Framebuffer;
    fn clear_screen(&mut self, color: Vector4<f32>);
    fn bind_framebuffer(&mut self, framebuffer: &Self::Framebuffer);
    /// Binds the screen and all rendercalls made after this calls will draw to the screen.
    fn bind_default_framebuffer(&mut self);
    fn bind_texture(&mut self, texture: &Self::RuntimeTexture);
    fn draw_mesh(&mut self, mesh: &Self::RuntimeMesh);
    fn send_model_matrix(&mut self, matrix: Matrix4<f32>);
    fn send_view_matrix(&mut self, matrix: Matrix4<f32>);
}
pub struct WebGl {
    context: WebGl2RenderingContext,
    position_attribute_location: i32,
    uv_attribute_location: i32,
    texture_sampler_location: Option<WebGlUniformLocation>,
    program: WebGlProgram,
}
#[derive(Clone)]
pub struct WebGlMesh {
    vertex_array_object: Option<WebGlVertexArrayObject>,
    position_buffer: Option<WebGlBuffer>,
    count: i32,
}
#[derive(Clone)]
pub struct WebGlRenderTexture {
    texture: Option<WebGlTexture>,
}
pub struct WebFramebuffer {
    framebuffer: Option<WebGlFramebuffer>,
}
impl GraphicsEngine for WebGl {
    type RuntimeMesh = WebGlMesh;
    type RuntimeTexture = WebGlRenderTexture;
    type ErrorType = JsValue;
    type Framebuffer = WebFramebuffer;
    fn init() -> Result<Self, Self::ErrorType> {
        let window = web_sys::window().unwrap();
        let document = window.document().unwrap();
        let canvas = document.get_element_by_id("canvas").unwrap();
        let canvas: web_sys::HtmlCanvasElement = canvas.dyn_into::<web_sys::HtmlCanvasElement>()?;
        let context = canvas
            .get_context("webgl2")?
            .unwrap()
            .dyn_into::<WebGl2RenderingContext>()?;
        let vert_shader = Self::compile_shader(
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
        let frag_shader = Self::compile_shader(
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
        let program = Self::link_program(&context, &vert_shader, &frag_shader)?;
        context.use_program(Some(&program));
        let position_attribute_location = context.get_attrib_location(&program, "position");
        let uv_attribute_location = context.get_attrib_location(&program, "uv");
        let texture_sampler_location = context.get_uniform_location(&program, "u_texture");
        Ok(Self {
            context,
            position_attribute_location,
            uv_attribute_location,
            texture_sampler_location,
            program,
        })
    }
    fn build_mesh(&mut self, mesh: Mesh) -> Result<Self::RuntimeMesh, Self::ErrorType> {
        let position_buffer = self.context.create_buffer();
        let mut array: Vec<f32> = vec![];

        self.context.bind_buffer(
            WebGl2RenderingContext::ARRAY_BUFFER,
            (&position_buffer).as_ref(),
        );
        for (vertex, uv) in mesh.vertices.iter() {
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

            self.context.buffer_data_with_array_buffer_view(
                WebGl2RenderingContext::ARRAY_BUFFER,
                &vert_array,
                WebGl2RenderingContext::STATIC_DRAW,
            );
        }
        let vao = self.context.create_vertex_array();
        self.context.bind_vertex_array(vao.as_ref());
        self.context
            .enable_vertex_attrib_array(self.position_attribute_location as u32);
        self.context
            .enable_vertex_attrib_array(self.uv_attribute_location as u32);
        self.context.vertex_attrib_pointer_with_f64(
            self.position_attribute_location as u32,
            3,
            WebGl2RenderingContext::FLOAT,
            false,
            5 * std::mem::size_of::<f32>() as i32,
            0.0,
        );
        self.context.vertex_attrib_pointer_with_i32(
            self.uv_attribute_location as u32,
            2,
            WebGl2RenderingContext::FLOAT,
            false,
            5 * std::mem::size_of::<f32>() as i32,
            3 * std::mem::size_of::<f32>() as i32,
        );
        Ok(WebGlMesh {
            vertex_array_object: vao,
            position_buffer,
            count: mesh.vertices.len() as i32,
        })
    }
    fn build_texture(
        &mut self,
        texture: RGBATexture,
    ) -> Result<Self::RuntimeTexture, Self::ErrorType> {
        let gl_texture = self.context.create_texture();
        assert!(gl_texture.is_some());
        self.context
            .bind_texture(WebGl2RenderingContext::TEXTURE_2D, gl_texture.as_ref());
        let texture_unit = 0;
        self.context
            .active_texture(WebGl2RenderingContext::TEXTURE0 + texture_unit);
        let level = 0;
        self.context
            .tex_image_2d_with_i32_and_i32_and_i32_and_format_and_type_and_u8_array_and_src_offset(
                WebGl2RenderingContext::TEXTURE_2D,
                level,
                //  Use RGBA Format
                WebGl2RenderingContext::RGBA as i32,
                //width
                texture.dimensions.x as i32,
                //height
                texture.dimensions.y as i32,
                //must be 0 specifies the border
                0,
                //  Use RGB Format
                WebGl2RenderingContext::RGBA,
                WebGl2RenderingContext::UNSIGNED_BYTE,
                texture.get_raw_vector().as_slice(),
                0,
            )?;
        //self.gl_context.generate_mipmap(WebGl2RenderingContext::TEXTURE_2D);
        //getting location of sampler

        self.context
            .uniform1i(self.texture_sampler_location.as_ref(), texture_unit as i32);
        self.context.tex_parameteri(
            WebGl2RenderingContext::TEXTURE_2D,
            WebGl2RenderingContext::TEXTURE_MIN_FILTER,
            WebGl2RenderingContext::LINEAR as i32,
        );
        self.context.tex_parameteri(
            WebGl2RenderingContext::TEXTURE_2D,
            WebGl2RenderingContext::TEXTURE_WRAP_S,
            WebGl2RenderingContext::CLAMP_TO_EDGE as i32,
        );
        self.context.tex_parameteri(
            WebGl2RenderingContext::TEXTURE_2D,
            WebGl2RenderingContext::TEXTURE_WRAP_T,
            WebGl2RenderingContext::CLAMP_TO_EDGE as i32,
        );
        Ok(WebGlRenderTexture {
            texture: gl_texture,
        })
    }
    fn build_framebuffer(
        &mut self,
        texture_attachment: &mut Self::RuntimeTexture,
    ) -> Self::Framebuffer {
        let framebuffer = self.context.create_framebuffer();
        self.context
            .bind_framebuffer(WebGl2RenderingContext::FRAMEBUFFER, framebuffer.as_ref());
        self.context.framebuffer_texture_2d(
            WebGl2RenderingContext::FRAMEBUFFER,
            WebGl2RenderingContext::COLOR_ATTACHMENT0,
            WebGl2RenderingContext::TEXTURE_2D,
            texture_attachment.texture.as_ref(),
            0,
        );
        // rebinding to default framebuffer to prevent side effects
        self.bind_default_framebuffer();
        WebFramebuffer { framebuffer }
    }
    fn bind_default_framebuffer(&mut self) {
        self.context
            .bind_framebuffer(WebGl2RenderingContext::FRAMEBUFFER, None);
    }
    fn clear_screen(&mut self, color: Vector4<f32>) {
        self.context.clear_color(color.x, color.y, color.z, color.w);
        self.context.clear(WebGl2RenderingContext::COLOR_BUFFER_BIT);
    }
    fn bind_texture(&mut self, texture: &Self::RuntimeTexture) {
        self.context
            .active_texture(WebGl2RenderingContext::TEXTURE0);
        self.context
            .bind_texture(WebGl2RenderingContext::TEXTURE_2D, texture.texture.as_ref());
        self.context
            .uniform1i(self.texture_sampler_location.as_ref(), 0);
    }
    fn bind_framebuffer(&mut self, framebuffer: &Self::Framebuffer) {
        self.context.bind_framebuffer(
            WebGl2RenderingContext::FRAMEBUFFER,
            framebuffer.framebuffer.as_ref(),
        );
    }
    fn draw_mesh(&mut self, mesh: &Self::RuntimeMesh) {
        self.context
            .bind_vertex_array(mesh.vertex_array_object.as_ref());
        self.context
            .draw_arrays(WebGl2RenderingContext::TRIANGLES, 0, mesh.count);
    }
    fn send_model_matrix(&mut self, matrix: Matrix4<f32>) {
        let model_uniform = self.context.get_uniform_location(&self.program, "model");
        self.context.uniform_matrix4fv_with_f32_array(
            model_uniform.as_ref(),
            false,
            matrix.as_slice(),
        );
    }
    fn send_view_matrix(&mut self, matrix: Matrix4<f32>) {
        let model_uniform = self.context.get_uniform_location(&self.program, "camera");
        self.context.uniform_matrix4fv_with_f32_array(
            model_uniform.as_ref(),
            false,
            matrix.as_slice(),
        );
    }
}
impl WebGl {
    fn compile_shader(
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
    fn link_program(
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
    fn bind_gl_texture(&self, texture: Option<&WebGlTexture>) {
        self.context
            .active_texture(WebGl2RenderingContext::TEXTURE0);
        self.context
            .bind_texture(WebGl2RenderingContext::TEXTURE_2D, texture);
        self.context
            .uniform1i(self.texture_sampler_location.as_ref(), 0);
    }
    fn bind_framebuffer(&self, framebuffer: Option<&WebGlFramebuffer>) {
        self.context
            .bind_framebuffer(WebGl2RenderingContext::FRAMEBUFFER, framebuffer);
    }
}
