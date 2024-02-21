mod shaders;
mod textures;

use wasm_bindgen::prelude::*;
use web_sys::{
    HtmlCanvasElement,
    WebGlProgram,
    WebGlRenderingContext,
    WebGlShader,
    WebGlUniformLocation,
};
use std::collections::HashMap;
use crate::textures::*;

const PRESSURE_ITERATIONS: usize = 20;
const FPS_30: f32 = 0.0333333;

#[repr(u8)]
#[derive(Clone, Copy)]
#[wasm_bindgen]
pub enum Resolution {
    ONE = 1,
    TWO = 2,
    FOUR = 4,
    EIGHT = 8,
    SIXTEEN = 16,
}

struct Program {
    program: WebGlProgram,
    uniforms: HashMap<String, WebGlUniformLocation>,
}

impl Program {
    fn create_shader(
        gl: &WebGlRenderingContext,
        shader_type: u32,
        source: &str,
    ) -> Result<WebGlShader, JsValue> {
        let shader = gl.create_shader(shader_type)
        .ok_or_else(|| JsValue::from_str("Unable to create shader object"))?;

        gl.shader_source(&shader, source);
        gl.compile_shader(&shader);

        if gl.get_shader_parameter(&shader, WebGlRenderingContext::COMPILE_STATUS)
            .as_bool()
            .unwrap_or(false)
        {
            return Ok(shader);
        }

        Err(JsValue::from_str(
            &gl.get_shader_info_log(&shader)
                .unwrap_or_else(|| "Unknown error creating shader".into())
        ))
    }

    fn new(
        gl: &WebGlRenderingContext,
        fragment_shader: &str,
        vertex_shader: &str,
    ) -> Result<Program, JsValue> {
        let vertex_shader = Program::create_shader(
            &gl,
            WebGlRenderingContext::VERTEX_SHADER,
            vertex_shader
        )?;
    
        let fragment_shader = Program::create_shader(
            &gl,
            WebGlRenderingContext::FRAGMENT_SHADER,
            &fragment_shader
        )?;
    
        let shader_program = gl.create_program()
            .ok_or_else(|| JsValue::from_str("Unable to create program"))?;
        gl.attach_shader(&shader_program, &vertex_shader);
        gl.attach_shader(&shader_program, &fragment_shader);
        gl.link_program(&shader_program);
    
        if gl.get_program_parameter(&shader_program, WebGlRenderingContext::LINK_STATUS)
            .as_bool()
            .unwrap_or(false)
        {
            let count = gl.get_program_parameter(&shader_program, WebGlRenderingContext::ACTIVE_UNIFORMS)
                .as_f64()
                .ok_or_else(|| JsValue::from_str("Unable to get program parameters"))? as u32;
            let mut uniforms = HashMap::with_capacity(count as usize);
            for i in 0..count {
                let name = gl.get_active_uniform(&shader_program, i).unwrap().name();
                uniforms.insert(name.clone(), gl.get_uniform_location(&shader_program, &name).unwrap());
            }

            return Ok(Program {
                program: shader_program,
                uniforms,
            });
        }
    
        Err(JsValue::from_str(
            &gl.get_program_info_log(&shader_program)
                .unwrap_or_else(|| "Unknown error linking program".into())
        ))
    }
    
    fn bind(&self, gl: &WebGlRenderingContext) {
        gl.use_program(Some(&self.program));
    }
}

#[wasm_bindgen]
pub struct Renderer {
    gl: WebGlRenderingContext,
    canvas: HtmlCanvasElement,
    sim_resolution: Resolution,
    dye_resolution: Resolution,
    copy_program: Program,
    advection_program: Program,
    jacobi_program: Program,
    divergence_program: Program,
    subtraction_program: Program,
    curl_program: Program,
    vorticity_program: Program,
    splat_program: Program,
    velocity_buffer: RWTextureBuffer,
    pressure_buffer: RWTextureBuffer,
    dye_buffer: RWTextureBuffer,
    temp_store: TextureFramebuffer,
    last_time: f32,
}

impl Renderer {
    fn init_quad_buffers(gl: &WebGlRenderingContext) -> Result<(), JsValue> {
        let vertex_buffer = gl.create_buffer().ok_or("Failed to create buffer")?;
        gl.bind_buffer(WebGlRenderingContext::ARRAY_BUFFER, Some(&vertex_buffer));

        let vertices = [
            -1.0, -1.0, 0.0, 0.0,
             1.0, -1.0, 1.0, 0.0,
            -1.0,  1.0, 0.0, 1.0,
             1.0,  1.0, 1.0, 1.0,
        ];
        let vertices = unsafe { js_sys::Float32Array::view(&vertices) };
        gl.buffer_data_with_array_buffer_view(
            WebGlRenderingContext::ARRAY_BUFFER,
            &vertices,
            WebGlRenderingContext::STATIC_DRAW,
        );

        gl.vertex_attrib_pointer_with_i32(
            0,
            2,
            WebGlRenderingContext::FLOAT,
            false,
            16,
            0,
        );
        gl.vertex_attrib_pointer_with_i32(
            1,
            2,
            WebGlRenderingContext::FLOAT,
            false,
            16,
            8,
        );

        gl.enable_vertex_attrib_array(0);
        gl.enable_vertex_attrib_array(1);

        Ok(())
    }

    fn blit(
        gl: &WebGlRenderingContext,
        target: Option<&TextureFramebuffer>,
        clear: Option<bool>,
    ) {
        match target {
            Some(tfb) => {
                gl.viewport(
                    0,
                    0,
                    tfb.width() as i32,
                    tfb.height() as i32,
                );
                gl.bind_framebuffer(WebGlRenderingContext::FRAMEBUFFER, Some(tfb.buffer()));
            }
            None => {
                gl.viewport(
                    0,
                    0,
                    gl.drawing_buffer_width(),
                    gl.drawing_buffer_height(),
                );
                gl.bind_framebuffer(WebGlRenderingContext::FRAMEBUFFER, None);
            }
        }

        if clear.unwrap_or(false) {
            gl.clear_color(0.0, 0.0, 0.0, 1.0);
            gl.clear(WebGlRenderingContext::COLOR_BUFFER_BIT);
        }

        gl.draw_arrays(
            WebGlRenderingContext::TRIANGLE_STRIP,
            0,
            4,
        );
    }

    fn draw_pass(&self) -> Result<(), JsValue> {
        self.copy_program.bind(&self.gl);

        self.gl.uniform1f(
            self.copy_program.uniforms.get(shaders::U_FACTOR),
            1.0,
        );
        self.gl.uniform1f(
            self.copy_program.uniforms.get(shaders::U_OFFSET),
            0.0,
        );

        Renderer::blit(
            &self.gl,
            None,
            Some(true),
        );

        Ok(())
    }

    fn advect(
        gl: &WebGlRenderingContext,
        advection_program: &Program,
        sim_resolution: &[f32; 2],
        delta_time: f32,
        dissipation: f32,
        velocity_buffer: Option<&RWTextureBuffer>,
        quantity: &mut RWTextureBuffer,
    ) -> Result<(), JsValue> {
        advection_program.bind(&gl);

        gl.uniform1f(
            advection_program.uniforms.get(shaders::U_DISSIPATION),
            1.0 / (1.0 + dissipation * delta_time),
        );
        gl.uniform1f(
            advection_program.uniforms.get(shaders::U_DELTA_TIME),
            delta_time,
        );
        gl.uniform2fv_with_f32_array(
            advection_program.uniforms.get(shaders::U_RESOLUTION),
            sim_resolution,
        );
        gl.uniform1i(
            advection_program.uniforms.get(shaders::U_VELOCITY),
            velocity_buffer.and_then(|b|
                b.read().bind(&gl, 1).ok()
            ).unwrap_or(0),
        );
        gl.uniform1i(
            advection_program.uniforms.get(shaders::U_QUANTITY),
            quantity.read().bind(&gl, 0)?,
        );

        Renderer::blit(
            &gl,
            Some(quantity.write()),
            None,
        );
        quantity.swap();

        Ok(())
    }

    fn project_velocity(
        &mut self,
        sim_resolution: &[f32; 2],
        iterations: usize,
        pressure: f32,
    ) -> Result<(), JsValue> {
        let r_half_texel = 0.5 / (self.sim_resolution as u32 as f32);

        // DIVERGENCE
        self.divergence_program.bind(&self.gl);

        self.gl.uniform1f(
            self.divergence_program.uniforms.get(shaders::U_R_HALF_TEXEL_SIZE),
            r_half_texel,
        );
        self.gl.uniform2fv_with_f32_array(
            self.divergence_program.uniforms.get(shaders::U_RESOLUTION),
            sim_resolution,
        );
        self.gl.uniform1i(
            self.divergence_program.uniforms.get(shaders::U_VELOCITY),
            self.velocity_buffer.read().bind(&self.gl, 0)?,
        );

        Renderer::blit(
            &self.gl,
            Some(&self.temp_store),
            None,
        );

        // PRESSURE
        self.copy_program.bind(&self.gl);

        self.gl.uniform1f(
            self.copy_program.uniforms.get(shaders::U_FACTOR),
            pressure,
        );
        self.gl.uniform1f(
            self.copy_program.uniforms.get(shaders::U_OFFSET),
            0.0,
        );
        self.gl.uniform1i(
            self.copy_program.uniforms.get(shaders::U_TEXTURE),
            self.pressure_buffer.read().bind(&self.gl, 0)?,
        );

        Renderer::blit(
            &self.gl,
            Some(self.pressure_buffer.write()),
            None,
        );
        self.pressure_buffer.swap();

        let alpha = self.sim_resolution as u32 as f32;
        let alpha = -alpha * alpha;
        let r_beta = 0.25;
        Renderer::jacobi_solve(
            &self.gl,
            &self.jacobi_program,
            iterations,
            sim_resolution,
            alpha,
            r_beta,
            &mut self.pressure_buffer,
            Some(&self.temp_store),
        )?;

        // SUBTRACTION
        self.subtraction_program.bind(&self.gl);

        self.gl.uniform1f(
            self.subtraction_program.uniforms.get(shaders::U_R_HALF_TEXEL_SIZE),
            r_half_texel,
        );
        self.gl.uniform2fv_with_f32_array(
            self.subtraction_program.uniforms.get(shaders::U_RESOLUTION),
            sim_resolution,
        );
        self.gl.uniform1i(
            self.subtraction_program.uniforms.get(shaders::U_VELOCITY),
            self.velocity_buffer.read().bind(&self.gl, 0)?,
        );
        self.gl.uniform1i(
            self.subtraction_program.uniforms.get(shaders::U_PRESSURE),
            self.pressure_buffer.read().bind(&self.gl, 1)?,
        );

        Renderer::blit(
            &self.gl,
            Some(self.velocity_buffer.write()),
            None,
        );
        self.velocity_buffer.swap();

        Ok(())
    }

    fn vorticity_confinement(
        &mut self,
        sim_resolution: &[f32; 2],
        curl: f32,
    ) -> Result<(), JsValue> {
        let r_half_texel = 0.5 / (self.sim_resolution as u32 as f32);

        // CURL
        self.curl_program.bind(&self.gl);

        self.gl.uniform1f(
            self.curl_program.uniforms.get(shaders::U_R_HALF_TEXEL_SIZE),
            r_half_texel,
        );
        self.gl.uniform2fv_with_f32_array(
            self.curl_program.uniforms.get(shaders::U_RESOLUTION),
            sim_resolution,
        );
        self.gl.uniform1i(
            self.curl_program.uniforms.get(shaders::U_VELOCITY),
            self.velocity_buffer.read().bind(&self.gl, 0)?,
        );

        Renderer::blit(
            &self.gl,
            Some(&self.temp_store),
            None,
        );

        // VORTICITY CONFINEMENT
        self.vorticity_program.bind(&self.gl);

        self.gl.uniform1f(
            self.vorticity_program.uniforms.get(shaders::U_CURL_SCALE),
            curl,
        );
        self.gl.uniform1f(
            self.vorticity_program.uniforms.get(shaders::U_R_HALF_TEXEL_SIZE),
            r_half_texel,
        );
        self.gl.uniform2fv_with_f32_array(
            self.vorticity_program.uniforms.get(shaders::U_RESOLUTION),
            sim_resolution,
        );
        self.gl.uniform1i(
            self.vorticity_program.uniforms.get(shaders::U_CURL),
            self.temp_store.bind(&self.gl, 0)?,
        );
        self.gl.uniform1i(
            self.vorticity_program.uniforms.get(shaders::U_VELOCITY),
            self.velocity_buffer.read().bind(&self.gl, 1)?,
        );

        Renderer::blit(
            &self.gl,
            Some(self.velocity_buffer.write()),
            None,
        );
        self.velocity_buffer.swap();

        Ok(())
    }

    fn jacobi_solve(
        gl: &WebGlRenderingContext,
        jacobi_program: &Program,
        iterations: usize,
        resolution: &[f32; 2],
        alpha: f32,
        r_beta: f32,
        x: &mut RWTextureBuffer,
        b: Option<&TextureFramebuffer>,
    ) -> Result<(), JsValue> {
        jacobi_program.bind(&gl);

        gl.uniform1f(
            jacobi_program.uniforms.get(shaders::U_ALPHA),
            alpha,
        );
        gl.uniform1f(
            jacobi_program.uniforms.get(shaders::U_R_BETA),
            r_beta,
        );
        gl.uniform2fv_with_f32_array(
            jacobi_program.uniforms.get(shaders::U_RESOLUTION),
            resolution,        
        );
        gl.uniform1i(
            jacobi_program.uniforms.get(shaders::U_B),
            b.and_then(|b|
                b.bind(&gl, 1).ok()
            ).unwrap_or(0),
        );

        for _ in 0..iterations {
            gl.uniform1i(
                jacobi_program.uniforms.get(shaders::U_X),
                x.read().bind(&gl, 0)?,
            );

            Renderer::blit(
                &gl,
                Some(x.write()),
                None,
            );
            x.swap();
        }

        Ok(())
    }

    fn resolution_size(canvas: &HtmlCanvasElement, resolution: Resolution) -> (u32, u32) {
        let (width, height) = (canvas.width(), canvas.height());
        (width / resolution as u32, height / resolution as u32)
    }
}

#[wasm_bindgen]
impl Renderer {
    pub fn create (
        canvas_id: &str,
        sim_resolution: Resolution,
        dye_resolution: Resolution,
    ) -> Result<Renderer, JsValue> {
        console_error_panic_hook::set_once();
        let document = web_sys::window().unwrap().document().unwrap();
        let canvas = document.get_element_by_id(canvas_id).unwrap();
        let canvas = canvas.dyn_into::<HtmlCanvasElement>().unwrap();

        let context_options = js_sys::Object::new();
        js_sys::Reflect::set(&context_options, &"antialias".into(), &JsValue::FALSE)?;
        js_sys::Reflect::set(&context_options, &"alpha".into(), &JsValue::TRUE)?;
        js_sys::Reflect::set(&context_options, &"depth".into(), &JsValue::FALSE)?;
        js_sys::Reflect::set(&context_options, &"stencil".into(), &JsValue::FALSE)?;
        let gl = canvas.get_context_with_context_options("webgl", &context_options,)?.unwrap();
        let gl = gl.dyn_into::<WebGlRenderingContext>().unwrap();

        gl.get_extension("OES_texture_float")?;
        gl.get_extension("OES_texture_float_linear")?;
        gl.get_extension("WEBGL_color_buffer_float")?;
        gl.disable(WebGlRenderingContext::BLEND);

        let copy_program = Program::new(
            &gl,
            shaders::COPY_SHADER_SOURCE,
            shaders::VERTEX_SHADER_SOURCE,
        )?;
        let advection_program = Program::new(
            &gl,
            shaders::ADVECTION_SHADER_SOURCE,
            shaders::VERTEX_SHADER_SOURCE,
        )?;
        let jacobi_program = Program::new(
            &gl,
            shaders::JACOBI_SOLVER_SHADER_SOURCE,
            shaders::VERTEX_SHADER_SOURCE,
        )?;
        let divergence_program = Program::new(
            &gl,
            shaders::DIVERGENCE_SHADER_SOURCE,
            shaders::VERTEX_SHADER_SOURCE,
        )?;
        let subtraction_program = Program::new(
            &gl,
            shaders::GRADIENT_SUBTRACT_SHADER_SOURCE,
            shaders::VERTEX_SHADER_SOURCE,
        )?;
        let curl_program = Program::new(
            &gl,
            shaders::CURL_SHADER_SOURCE,
            shaders::VERTEX_SHADER_SOURCE,
        )?;
        let vorticity_program = Program::new(
            &gl,
            shaders::VORTICITY_SHADER_SOURCE,
            shaders::VERTEX_SHADER_SOURCE,
        )?;
        let splat_program = Program::new(
            &gl,
            shaders::SPLAT_SHADER_SOURCE,
            shaders::VERTEX_SHADER_SOURCE,
        )?;

        let (width, height) = Renderer::resolution_size(&canvas, sim_resolution);
        let velocity_buffer = RWTextureBuffer::new(
            &gl,
            width,
            height,
            Some(WebGlRenderingContext::LINEAR),
        )?;

        let pressure_buffer = RWTextureBuffer::new(
            &gl,
            width,
            height,
            Some(WebGlRenderingContext::LINEAR),
        )?;

        let temp_store = TextureFramebuffer::new(
            &gl,
            width,
            height,
            WebGlRenderingContext::LINEAR,
        )?;

        let (width, height) = Renderer::resolution_size(&canvas, dye_resolution);
        let dye_buffer = RWTextureBuffer::new(
            &gl,
            width,
            height,
            Some(WebGlRenderingContext::LINEAR),
        )?;

        Renderer::init_quad_buffers(&gl)?;

        Ok(Renderer {
            gl,
            canvas,
            sim_resolution,
            dye_resolution,
            copy_program,
            advection_program,
            jacobi_program,
            divergence_program,
            subtraction_program,
            curl_program,
            vorticity_program,
            splat_program,
            velocity_buffer,
            pressure_buffer,
            dye_buffer,
            temp_store,
            last_time: 0.0,
        })
    }

    pub fn update(
        &mut self,
        time: f32,
        position: &[f32],
        velocity: &[f32],
        radius: f32,
        viscosity: f32,
        dissipation: f32,
        curl: f32,
        pressure: f32,
    ) -> Result<(), JsValue> {
        let delta_time = FPS_30.min(time - self.last_time);
        self.last_time = time;
        let velocity = velocity
            .iter()
            .map(|v| {
                v * delta_time
            })
            .collect::<Vec<_>>();

        // SIMULATION
        let (width, height) = Renderer::resolution_size(&self.canvas, self.sim_resolution);
        let sim_resolution = [width as f32, height as f32];

        self.splat(
            radius,
            position,
            &velocity,
            &[0.0, 0.3, 0.5],
        )?;

        self.vorticity_confinement(&sim_resolution, curl)?;

        Renderer::advect(
            &self.gl,
            &self.advection_program,
            &sim_resolution,
            delta_time,
            viscosity,
            None,
            &mut self.velocity_buffer,
        )?;

        self.project_velocity(
            &sim_resolution,
            PRESSURE_ITERATIONS,
            pressure,
        )?;

        // UPDATE DYE
        Renderer::advect(
            &self.gl,
            &self.advection_program,
            &sim_resolution,
            delta_time,
            dissipation,
            Some(&self.velocity_buffer),
            &mut self.dye_buffer,
        )?;

        // RENDER
        // DRAW TO CANVAS
        self.draw_pass()?;

        Ok(())
    }

    pub fn resize(
        &mut self,
        sim_resolution: Resolution,
        dye_resolution: Resolution,
    ) -> Result<(), JsValue> {
        self.sim_resolution = sim_resolution;
        self.dye_resolution = dye_resolution;

        // SIMULATION
        let (width, height) = Renderer::resolution_size(&self.canvas, sim_resolution);
        self.velocity_buffer.resize(
            &self.gl,
            width,
            height,
        )?;

        self.pressure_buffer.resize(
            &self.gl,
            width,
            height,
        )?;

        if width != self.temp_store.width() || height != self.temp_store.height() {
            self.temp_store.delete(&self.gl);
            self.temp_store = TextureFramebuffer::new(
                &self.gl,
                width,
                height,
                WebGlRenderingContext::LINEAR,
            )?;
        }
        
        // DYE
        let (width, height) = Renderer::resolution_size(&self.canvas, dye_resolution);
        self.dye_buffer.resize(
            &self.gl,
            width,
            height,
        )?;

        Ok(())
    }

    fn splat(
        &mut self,
        radius: f32,
        position: &[f32],
        velocity: &[f32],
        color: &[f32],
    ) -> Result<(), JsValue> {
        // APPLY FORCE
        let resolution = self.sim_resolution as u32 as f32;
        self.splat_program.bind(&self.gl);

        self.gl.uniform1f(
            self.splat_program.uniforms.get(shaders::U_SCALED_RADIUS),
            radius / (resolution * resolution),
        );
        self.gl.uniform2f(
            self.splat_program.uniforms.get(shaders::U_POSITION),
            position[0] / resolution,
            position[1] / resolution,
        );
        self.gl.uniform3f(
            self.splat_program.uniforms.get(shaders::U_COLOR),
            velocity[0] / resolution,
            velocity[1] / resolution,
            0.0,
        );
        self.gl.uniform1i(
            self.splat_program.uniforms.get(shaders::U_TEXTURE),
            self.velocity_buffer.read().bind(&self.gl, 0)?,
        );

        Renderer::blit(
            &self.gl,
            Some(self.velocity_buffer.write()),
            None,
        );
        self.velocity_buffer.swap();

        // APPLY COLOR
        let resolution = self.dye_resolution as u32 as f32;
        self.gl.uniform1f(
            self.splat_program.uniforms.get(shaders::U_SCALED_RADIUS),
            radius / (resolution * resolution),
        );
        self.gl.uniform2f(
            self.splat_program.uniforms.get(shaders::U_POSITION),
            position[0] / resolution,
            position[1] / resolution,
        );
        self.gl.uniform3fv_with_f32_array(
            self.splat_program.uniforms.get(shaders::U_COLOR),
            color,
        );
        self.gl.uniform1i(
            self.splat_program.uniforms.get(shaders::U_TEXTURE),
            self.dye_buffer.read().bind(&self.gl, 0)?,
        );

        Renderer::blit(
            &self.gl,
            Some(self.dye_buffer.write()),
            None,
        );
        self.dye_buffer.swap();

        Ok(())
    }
}