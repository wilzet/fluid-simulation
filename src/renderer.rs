use wasm_bindgen::prelude::*;
use web_sys::{
    HtmlCanvasElement,
    WebGlRenderingContext,
    WebGl2RenderingContext,
};
use crate::{
    Renderer,
    Resolution,
    Mode,
    PRESSURE_ITERATIONS,
};
use crate::shader_program::ShaderProgram;
use crate::textures::*;
use crate::shaders;

// SHARED
impl Renderer {
    pub fn resolution_size(canvas: &HtmlCanvasElement, resolution: Resolution) -> (u32, u32) {
        let (width, height) = (canvas.width(), canvas.height());
        (width / resolution as u32, height / resolution as u32)
    }
}

// WebGL
impl Renderer {
    pub fn new_webgl(
        gl: js_sys::Object,
        canvas: HtmlCanvasElement,
        sim_resolution: Resolution,
        dye_resolution: Resolution,
    ) -> Result<Renderer, JsValue> {
        let gl = gl.dyn_into::<WebGlRenderingContext>().unwrap();

        gl.get_extension("OES_texture_float")?;
        gl.get_extension("OES_texture_float_linear")?;
        gl.get_extension("WEBGL_color_buffer_float")?;
        gl.disable(WebGlRenderingContext::BLEND);

        let copy_program = ShaderProgram::new_webgl(
            &gl,
            shaders::COPY_SHADER_SOURCE,
            shaders::VERTEX_SHADER_SOURCE,
        )?;
        let advection_program = ShaderProgram::new_webgl(
            &gl,
            shaders::ADVECTION_SHADER_SOURCE,
            shaders::VERTEX_SHADER_SOURCE,
        )?;
        let jacobi_program = ShaderProgram::new_webgl(
            &gl,
            shaders::JACOBI_SOLVER_SHADER_SOURCE,
            shaders::VERTEX_SHADER_SOURCE,
        )?;
        let divergence_program = ShaderProgram::new_webgl(
            &gl,
            shaders::DIVERGENCE_SHADER_SOURCE,
            shaders::VERTEX_SHADER_SOURCE,
        )?;
        let subtraction_program = ShaderProgram::new_webgl(
            &gl,
            shaders::GRADIENT_SUBTRACT_SHADER_SOURCE,
            shaders::VERTEX_SHADER_SOURCE,
        )?;
        let curl_program = ShaderProgram::new_webgl(
            &gl,
            shaders::CURL_SHADER_SOURCE,
            shaders::VERTEX_SHADER_SOURCE,
        )?;
        let vorticity_program = ShaderProgram::new_webgl(
            &gl,
            shaders::VORTICITY_SHADER_SOURCE,
            shaders::VERTEX_SHADER_SOURCE,
        )?;
        let splat_program = ShaderProgram::new_webgl(
            &gl,
            shaders::SPLAT_SHADER_SOURCE,
            shaders::VERTEX_SHADER_SOURCE,
        )?;

        let (width, height) = Renderer::resolution_size(&canvas, sim_resolution);
        let velocity_buffer = RWTextureBuffer::new_webgl(
            &gl,
            width,
            height,
            Some(WebGlRenderingContext::FLOAT),
            Some(WebGlRenderingContext::LINEAR),
        )?;

        let pressure_buffer = RWTextureBuffer::new_webgl(
            &gl,
            width,
            height,
            Some(WebGlRenderingContext::FLOAT),
            Some(WebGlRenderingContext::LINEAR),
        )?;

        let temp_store = TextureFramebuffer::new_webgl(
            &gl,
            width,
            height,
            WebGlRenderingContext::FLOAT,
            WebGlRenderingContext::LINEAR,
        )?;

        let (width, height) = Renderer::resolution_size(&canvas, dye_resolution);
        let dye_buffer = RWTextureBuffer::new_webgl(
            &gl,
            width,
            height,
            Some(WebGlRenderingContext::UNSIGNED_BYTE),
            Some(WebGlRenderingContext::LINEAR),
        )?;

        Renderer::init_quad_buffers_webgl(&gl)?;

        Ok(Renderer {
            gl: Some(gl),
            gl2: None,
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
    
    fn init_quad_buffers_webgl(gl: &WebGlRenderingContext) -> Result<(), JsValue> {
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

    pub fn blit_webgl(
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
    
    fn jacobi_solve_webgl(
        gl: &WebGlRenderingContext,
        jacobi_program: &ShaderProgram,
        iterations: usize,
        resolution: &[f32; 2],
        alpha: f32,
        r_beta: f32,
        x: &mut RWTextureBuffer,
        b: Option<&TextureFramebuffer>,
    ) -> Result<(), JsValue> {
        jacobi_program.bind_webgl(gl);

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
                b.bind_webgl(gl, 1).ok()
            ).unwrap_or(0),
        );

        for _ in 0..iterations {
            gl.uniform1i(
                jacobi_program.uniforms.get(shaders::U_X),
                x.read().bind_webgl(gl, 0)?,
            );

            Renderer::blit_webgl(
                gl,
                Some(x.write()),
                None,
            );
            x.swap();
        }

        Ok(())
    }
    
    fn draw_pass_webgl(
        &self,
        gl: &WebGlRenderingContext,
        mode: Mode,
    ) -> Result<(), JsValue> {
        self.copy_program.bind_webgl(gl);

        gl.uniform1f(
            self.copy_program.uniforms.get(shaders::U_FACTOR),
            match mode {
                Mode::DYE => 1.0,
                _ => 0.5,
            },
        );
        gl.uniform1f(
            self.copy_program.uniforms.get(shaders::U_OFFSET),
            match mode {
                Mode::DYE => 0.0,
                _ => 0.5,
            },
        );
        gl.uniform1i(
            self.copy_program.uniforms.get(shaders::U_TEXTURE),
            match mode {
                Mode::DYE => self.dye_buffer.read().bind_webgl(gl, 0)?,
                Mode::VELOCITY => self.velocity_buffer.read().bind_webgl(gl, 0)?,
                Mode::PRESSURE => self.pressure_buffer.read().bind_webgl(gl, 0)?,
            },
        );

        Renderer::blit_webgl(
            gl,
            None,
            Some(true),
        );

        Ok(())
    }
    
    fn advect_webgl(
        gl: &WebGlRenderingContext,
        advection_program: &ShaderProgram,
        sim_resolution: &[f32; 2],
        delta_time: f32,
        dissipation: f32,
        velocity_buffer: Option<&RWTextureBuffer>,
        quantity: &mut RWTextureBuffer,
    ) -> Result<(), JsValue> {
        advection_program.bind_webgl(gl);

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
                b.read().bind_webgl(gl, 1).ok()
            ).unwrap_or(0),
        );
        gl.uniform1i(
            advection_program.uniforms.get(shaders::U_QUANTITY),
            quantity.read().bind_webgl(gl, 0)?,
        );

        Renderer::blit_webgl(
            gl,
            Some(quantity.write()),
            None,
        );
        quantity.swap();

        Ok(())
    }
   
    fn project_velocity_webgl(
        &mut self,
        gl: &WebGlRenderingContext,
        sim_resolution: &[f32; 2],
        iterations: usize,
        pressure: f32,
    ) -> Result<(), JsValue> {
        let r_half_texel = 0.5 / (self.sim_resolution as u32 as f32);

        // DIVERGENCE
        self.divergence_program.bind_webgl(gl);

        gl.uniform1f(
            self.divergence_program.uniforms.get(shaders::U_R_HALF_TEXEL_SIZE),
            r_half_texel,
        );
        gl.uniform2fv_with_f32_array(
            self.divergence_program.uniforms.get(shaders::U_RESOLUTION),
            sim_resolution,
        );
        gl.uniform1i(
            self.divergence_program.uniforms.get(shaders::U_VELOCITY),
            self.velocity_buffer.read().bind_webgl(gl, 0)?,
        );

        Renderer::blit_webgl(
            gl,
            Some(&self.temp_store),
            None,
        );

        // PRESSURE
        self.copy_program.bind_webgl(gl);

        gl.uniform1f(
            self.copy_program.uniforms.get(shaders::U_FACTOR),
            pressure,
        );
        gl.uniform1f(
            self.copy_program.uniforms.get(shaders::U_OFFSET),
            0.0,
        );
        gl.uniform1i(
            self.copy_program.uniforms.get(shaders::U_TEXTURE),
            self.pressure_buffer.read().bind_webgl(gl, 0)?,
        );

        Renderer::blit_webgl(
            gl,
            Some(self.pressure_buffer.write()),
            None,
        );
        self.pressure_buffer.swap();

        let alpha = self.sim_resolution as u32 as f32;
        let alpha = -alpha * alpha;
        let r_beta = 0.25;
        Renderer::jacobi_solve_webgl(
            gl,
            &self.jacobi_program,
            iterations,
            sim_resolution,
            alpha,
            r_beta,
            &mut self.pressure_buffer,
            Some(&self.temp_store),
        )?;

        // SUBTRACTION
        self.subtraction_program.bind_webgl(gl);

        gl.uniform1f(
            self.subtraction_program.uniforms.get(shaders::U_R_HALF_TEXEL_SIZE),
            r_half_texel,
        );
        gl.uniform2fv_with_f32_array(
            self.subtraction_program.uniforms.get(shaders::U_RESOLUTION),
            sim_resolution,
        );
        gl.uniform1i(
            self.subtraction_program.uniforms.get(shaders::U_VELOCITY),
            self.velocity_buffer.read().bind_webgl(gl, 0)?,
        );
        gl.uniform1i(
            self.subtraction_program.uniforms.get(shaders::U_PRESSURE),
            self.pressure_buffer.read().bind_webgl(gl, 1)?,
        );

        Renderer::blit_webgl(
            gl,
            Some(self.velocity_buffer.write()),
            None,
        );
        self.velocity_buffer.swap();

        Ok(())
    }

    fn vorticity_confinement_webgl(
        &mut self,
        gl: &WebGlRenderingContext,
        sim_resolution: &[f32; 2],
        curl: f32,
    ) -> Result<(), JsValue> {
        let r_half_texel = 0.5 / (self.sim_resolution as u32 as f32);

        // CURL
        self.curl_program.bind_webgl(gl);

        gl.uniform1f(
            self.curl_program.uniforms.get(shaders::U_R_HALF_TEXEL_SIZE),
            r_half_texel,
        );
        gl.uniform2fv_with_f32_array(
            self.curl_program.uniforms.get(shaders::U_RESOLUTION),
            sim_resolution,
        );
        gl.uniform1i(
            self.curl_program.uniforms.get(shaders::U_VELOCITY),
            self.velocity_buffer.read().bind_webgl(gl, 0)?,
        );

        Renderer::blit_webgl(
            gl,
            Some(&self.temp_store),
            None,
        );

        // VORTICITY CONFINEMENT
        self.vorticity_program.bind_webgl(gl);

        gl.uniform1f(
            self.vorticity_program.uniforms.get(shaders::U_CURL_SCALE),
            curl,
        );
        gl.uniform1f(
            self.vorticity_program.uniforms.get(shaders::U_R_HALF_TEXEL_SIZE),
            r_half_texel,
        );
        gl.uniform2fv_with_f32_array(
            self.vorticity_program.uniforms.get(shaders::U_RESOLUTION),
            sim_resolution,
        );
        gl.uniform1i(
            self.vorticity_program.uniforms.get(shaders::U_CURL),
            self.temp_store.bind_webgl(gl, 0)?,
        );
        gl.uniform1i(
            self.vorticity_program.uniforms.get(shaders::U_VELOCITY),
            self.velocity_buffer.read().bind_webgl(gl, 1)?,
        );

        Renderer::blit_webgl(
            gl,
            Some(self.velocity_buffer.write()),
            None,
        );
        self.velocity_buffer.swap();

        Ok(())
    }

    pub fn update_webgl(
        &mut self,
        gl: &WebGlRenderingContext,
        pause: bool,
        delta_time: f32,
        sim_resolution: &[f32; 2],
        mode: Mode,
        viscosity: f32,
        dissipation: f32,
        curl: f32,
        pressure: f32,
    ) -> Result<(), JsValue> {
        // SIMULATION
        if !pause {
            // UPDATE VELOCITY
            self.vorticity_confinement_webgl(
                gl,
                &sim_resolution,
                curl,
            )?;

            Renderer::advect_webgl(
                gl,
                &self.advection_program,
                &sim_resolution,
                delta_time,
                viscosity,
                None,
                &mut self.velocity_buffer,
            )?;

            self.project_velocity_webgl(
                gl,
                &sim_resolution,
                PRESSURE_ITERATIONS,
                pressure,
            )?;

            // UPDATE DYE
            Renderer::advect_webgl(
                gl,
                &self.advection_program,
                &sim_resolution,
                delta_time,
                dissipation,
                Some(&self.velocity_buffer),
                &mut self.dye_buffer,
            )?;
        }

        // RENDER
        // DRAW TO CANVAS
        self.draw_pass_webgl(gl, mode)?;

        Ok(())
    }

    pub fn resize_webgl(
        &mut self,
        gl: &WebGlRenderingContext,
        sim_resolution: Resolution,
        dye_resolution: Resolution,
    ) -> Result<(), JsValue> {
        // SIMULATION
        let (width, height) = Renderer::resolution_size(&self.canvas, sim_resolution);
        self.velocity_buffer.resize_webgl(
            gl,
            Some(&self.copy_program),
            width,
            height,
        )?;

        self.pressure_buffer.resize_webgl(
            gl,
            None,
            width,
            height,
        )?;

        if width != self.temp_store.width() || height != self.temp_store.height() {
            self.temp_store.delete_webgl(gl);
            self.temp_store = TextureFramebuffer::new_webgl(
                gl,
                width,
                height,
                WebGlRenderingContext::FLOAT,
                WebGlRenderingContext::LINEAR,
            )?;
        }
        
        // DYE
        let (width, height) = Renderer::resolution_size(&self.canvas, dye_resolution);
        self.dye_buffer.resize_webgl(
            gl,
            Some(&self.copy_program),
            width,
            height,
        )?;

        Ok(())
    }

    pub fn splat_webgl(
        &mut self,
        gl: &WebGlRenderingContext,
        radius: f32,
        position: &[f32],
        velocity: &[f32],
        color: &[f32],
    ) -> Result<(), JsValue> {
        // APPLY FORCE
        let resolution = self.sim_resolution as u32 as f32;
        self.splat_program.bind_webgl(gl);

        gl.uniform1f(
            self.splat_program.uniforms.get(shaders::U_SCALED_RADIUS),
            radius / (resolution * resolution),
        );
        gl.uniform2f(
            self.splat_program.uniforms.get(shaders::U_POSITION),
            position[0] / resolution,
            position[1] / resolution,
        );
        gl.uniform3f(
            self.splat_program.uniforms.get(shaders::U_COLOR),
            velocity[0] / resolution,
            velocity[1] / resolution,
            0.0,
        );
        gl.uniform1i(
            self.splat_program.uniforms.get(shaders::U_TEXTURE),
            self.velocity_buffer.read().bind_webgl(gl, 0)?,
        );

        Renderer::blit_webgl(
            gl,
            Some(self.velocity_buffer.write()),
            None,
        );
        self.velocity_buffer.swap();

        // APPLY COLOR
        let resolution = self.dye_resolution as u32 as f32;
        gl.uniform1f(
            self.splat_program.uniforms.get(shaders::U_SCALED_RADIUS),
            radius / (resolution * resolution),
        );
        gl.uniform2f(
            self.splat_program.uniforms.get(shaders::U_POSITION),
            position[0] / resolution,
            position[1] / resolution,
        );
        gl.uniform3fv_with_f32_array(
            self.splat_program.uniforms.get(shaders::U_COLOR),
            color,
        );
        gl.uniform1i(
            self.splat_program.uniforms.get(shaders::U_TEXTURE),
            self.dye_buffer.read().bind_webgl(gl, 0)?,
        );

        Renderer::blit_webgl(
            gl,
            Some(self.dye_buffer.write()),
            None,
        );
        self.dye_buffer.swap();
        
        Ok(())
    }
}

// WebGL2
impl Renderer {
    pub fn new_webgl2(
        gl: js_sys::Object,
        canvas: HtmlCanvasElement,
        sim_resolution: Resolution,
        dye_resolution: Resolution,
    ) -> Result<Renderer, JsValue> {
        let gl = gl.dyn_into::<WebGl2RenderingContext>().unwrap();

        gl.get_extension("OES_texture_float_linear")?;
        gl.get_extension("EXT_color_buffer_float")?;
        gl.disable(WebGl2RenderingContext::BLEND);

        let copy_program = ShaderProgram::new_webgl2(
            &gl,
            shaders::COPY_SHADER_SOURCE,
            shaders::VERTEX_SHADER_SOURCE,
        )?;
        let advection_program = ShaderProgram::new_webgl2(
            &gl,
            shaders::ADVECTION_SHADER_SOURCE,
            shaders::VERTEX_SHADER_SOURCE,
        )?;
        let jacobi_program = ShaderProgram::new_webgl2(
            &gl,
            shaders::JACOBI_SOLVER_SHADER_SOURCE,
            shaders::VERTEX_SHADER_SOURCE,
        )?;
        let divergence_program = ShaderProgram::new_webgl2(
            &gl,
            shaders::DIVERGENCE_SHADER_SOURCE,
            shaders::VERTEX_SHADER_SOURCE,
        )?;
        let subtraction_program = ShaderProgram::new_webgl2(
            &gl,
            shaders::GRADIENT_SUBTRACT_SHADER_SOURCE,
            shaders::VERTEX_SHADER_SOURCE,
        )?;
        let curl_program = ShaderProgram::new_webgl2(
            &gl,
            shaders::CURL_SHADER_SOURCE,
            shaders::VERTEX_SHADER_SOURCE,
        )?;
        let vorticity_program = ShaderProgram::new_webgl2(
            &gl,
            shaders::VORTICITY_SHADER_SOURCE,
            shaders::VERTEX_SHADER_SOURCE,
        )?;
        let splat_program = ShaderProgram::new_webgl2(
            &gl,
            shaders::SPLAT_SHADER_SOURCE,
            shaders::VERTEX_SHADER_SOURCE,
        )?;

        let (width, height) = Renderer::resolution_size(&canvas, sim_resolution);
        let velocity_buffer = RWTextureBuffer::new_webgl2(
            &gl,
            width,
            height,
            Some(WebGl2RenderingContext::FLOAT),
            Some(WebGl2RenderingContext::LINEAR),
        )?;

        let pressure_buffer = RWTextureBuffer::new_webgl2(
            &gl,
            width,
            height,
            Some(WebGl2RenderingContext::FLOAT),
            Some(WebGl2RenderingContext::LINEAR),
        )?;

        let temp_store = TextureFramebuffer::new_webgl2(
            &gl,
            width,
            height,
            WebGl2RenderingContext::FLOAT,
            WebGl2RenderingContext::LINEAR,
        )?;

        let (width, height) = Renderer::resolution_size(&canvas, dye_resolution);
        let dye_buffer = RWTextureBuffer::new_webgl2(
            &gl,
            width,
            height,
            Some(WebGl2RenderingContext::UNSIGNED_BYTE),
            Some(WebGl2RenderingContext::LINEAR),
        )?;

        Renderer::init_quad_buffers_webgl2(&gl)?;

        Ok(Renderer {
            gl: None,
            gl2: Some(gl),
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

    fn init_quad_buffers_webgl2(gl: &WebGl2RenderingContext) -> Result<(), JsValue> {
        let vertex_buffer = gl.create_buffer().ok_or("Failed to create buffer")?;
        gl.bind_buffer(WebGl2RenderingContext::ARRAY_BUFFER, Some(&vertex_buffer));

        let vertices = [
            -1.0, -1.0, 0.0, 0.0,
             1.0, -1.0, 1.0, 0.0,
            -1.0,  1.0, 0.0, 1.0,
             1.0,  1.0, 1.0, 1.0,
        ];
        let vertices = unsafe { js_sys::Float32Array::view(&vertices) };
        gl.buffer_data_with_array_buffer_view(
            WebGl2RenderingContext::ARRAY_BUFFER,
            &vertices,
            WebGl2RenderingContext::STATIC_DRAW,
        );

        gl.vertex_attrib_pointer_with_i32(
            0,
            2,
            WebGl2RenderingContext::FLOAT,
            false,
            16,
            0,
        );
        gl.vertex_attrib_pointer_with_i32(
            1,
            2,
            WebGl2RenderingContext::FLOAT,
            false,
            16,
            8,
        );

        gl.enable_vertex_attrib_array(0);
        gl.enable_vertex_attrib_array(1);

        Ok(())
    }

    pub fn blit_webgl2(
        gl: &WebGl2RenderingContext,
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
                gl.bind_framebuffer(WebGl2RenderingContext::FRAMEBUFFER, Some(tfb.buffer()));
            }
            None => {
                gl.viewport(
                    0,
                    0,
                    gl.drawing_buffer_width(),
                    gl.drawing_buffer_height(),
                );
                gl.bind_framebuffer(WebGl2RenderingContext::FRAMEBUFFER, None);
            }
        }

        if clear.unwrap_or(false) {
            gl.clear_color(0.0, 0.0, 0.0, 1.0);
            gl.clear(WebGl2RenderingContext::COLOR_BUFFER_BIT);
        }

        gl.draw_arrays(
            WebGl2RenderingContext::TRIANGLE_STRIP,
            0,
            4,
        );
    }

    fn jacobi_solve_webgl2(
        gl: &WebGl2RenderingContext,
        jacobi_program: &ShaderProgram,
        iterations: usize,
        resolution: &[f32; 2],
        alpha: f32,
        r_beta: f32,
        x: &mut RWTextureBuffer,
        b: Option<&TextureFramebuffer>,
    ) -> Result<(), JsValue> {
        jacobi_program.bind_webgl2(gl);

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
                b.bind_webgl2(gl, 1).ok()
            ).unwrap_or(0),
        );

        for _ in 0..iterations {
            gl.uniform1i(
                jacobi_program.uniforms.get(shaders::U_X),
                x.read().bind_webgl2(gl, 0)?,
            );

            Renderer::blit_webgl2(
                gl,
                Some(x.write()),
                None,
            );
            x.swap();
        }

        Ok(())
    }

    fn draw_pass_webgl2(
        &self,
        gl: &WebGl2RenderingContext,
        mode: Mode,
    ) -> Result<(), JsValue> {
        self.copy_program.bind_webgl2(gl);

        gl.uniform1f(
            self.copy_program.uniforms.get(shaders::U_FACTOR),
            match mode {
                Mode::DYE => 1.0,
                _ => 0.5,
            },
        );
        gl.uniform1f(
            self.copy_program.uniforms.get(shaders::U_OFFSET),
            match mode {
                Mode::DYE => 0.0,
                _ => 0.5,
            },
        );
        gl.uniform1i(
            self.copy_program.uniforms.get(shaders::U_TEXTURE),
            match mode {
                Mode::DYE => self.dye_buffer.read().bind_webgl2(gl, 0)?,
                Mode::VELOCITY => self.velocity_buffer.read().bind_webgl2(gl, 0)?,
                Mode::PRESSURE => self.pressure_buffer.read().bind_webgl2(gl, 0)?,
            },
        );

        Renderer::blit_webgl2(
            gl,
            None,
            Some(true),
        );

        Ok(())
    }

    fn advect_webgl2(
        gl: &WebGl2RenderingContext,
        advection_program: &ShaderProgram,
        sim_resolution: &[f32; 2],
        delta_time: f32,
        dissipation: f32,
        velocity_buffer: Option<&RWTextureBuffer>,
        quantity: &mut RWTextureBuffer,
    ) -> Result<(), JsValue> {
        advection_program.bind_webgl2(gl);

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
                b.read().bind_webgl2(gl, 1).ok()
            ).unwrap_or(0),
        );
        gl.uniform1i(
            advection_program.uniforms.get(shaders::U_QUANTITY),
            quantity.read().bind_webgl2(gl, 0)?,
        );

        Renderer::blit_webgl2(
            gl,
            Some(quantity.write()),
            None,
        );
        quantity.swap();

        Ok(())
    }

    fn project_velocity_webgl2(
        &mut self,
        gl: &WebGl2RenderingContext,
        sim_resolution: &[f32; 2],
        iterations: usize,
        pressure: f32,
    ) -> Result<(), JsValue> {
        let r_half_texel = 0.5 / (self.sim_resolution as u32 as f32);

        // DIVERGENCE
        self.divergence_program.bind_webgl2(gl);

        gl.uniform1f(
            self.divergence_program.uniforms.get(shaders::U_R_HALF_TEXEL_SIZE),
            r_half_texel,
        );
        gl.uniform2fv_with_f32_array(
            self.divergence_program.uniforms.get(shaders::U_RESOLUTION),
            sim_resolution,
        );
        gl.uniform1i(
            self.divergence_program.uniforms.get(shaders::U_VELOCITY),
            self.velocity_buffer.read().bind_webgl2(gl, 0)?,
        );

        Renderer::blit_webgl2(
            gl,
            Some(&self.temp_store),
            None,
        );

        // PRESSURE
        self.copy_program.bind_webgl2(gl);

        gl.uniform1f(
            self.copy_program.uniforms.get(shaders::U_FACTOR),
            pressure,
        );
        gl.uniform1f(
            self.copy_program.uniforms.get(shaders::U_OFFSET),
            0.0,
        );
        gl.uniform1i(
            self.copy_program.uniforms.get(shaders::U_TEXTURE),
            self.pressure_buffer.read().bind_webgl2(gl, 0)?,
        );

        Renderer::blit_webgl2(
            gl,
            Some(self.pressure_buffer.write()),
            None,
        );
        self.pressure_buffer.swap();

        let alpha = self.sim_resolution as u32 as f32;
        let alpha = -alpha * alpha;
        let r_beta = 0.25;
        Renderer::jacobi_solve_webgl2(
            gl,
            &self.jacobi_program,
            iterations,
            sim_resolution,
            alpha,
            r_beta,
            &mut self.pressure_buffer,
            Some(&self.temp_store),
        )?;

        // SUBTRACTION
        self.subtraction_program.bind_webgl2(gl);

        gl.uniform1f(
            self.subtraction_program.uniforms.get(shaders::U_R_HALF_TEXEL_SIZE),
            r_half_texel,
        );
        gl.uniform2fv_with_f32_array(
            self.subtraction_program.uniforms.get(shaders::U_RESOLUTION),
            sim_resolution,
        );
        gl.uniform1i(
            self.subtraction_program.uniforms.get(shaders::U_VELOCITY),
            self.velocity_buffer.read().bind_webgl2(gl, 0)?,
        );
        gl.uniform1i(
            self.subtraction_program.uniforms.get(shaders::U_PRESSURE),
            self.pressure_buffer.read().bind_webgl2(gl, 1)?,
        );

        Renderer::blit_webgl2(
            gl,
            Some(self.velocity_buffer.write()),
            None,
        );
        self.velocity_buffer.swap();

        Ok(())
    }

    fn vorticity_confinement_webgl2(
        &mut self,
        gl: &WebGl2RenderingContext,
        sim_resolution: &[f32; 2],
        curl: f32,
    ) -> Result<(), JsValue> {
        let r_half_texel = 0.5 / (self.sim_resolution as u32 as f32);

        // CURL
        self.curl_program.bind_webgl2(gl);

        gl.uniform1f(
            self.curl_program.uniforms.get(shaders::U_R_HALF_TEXEL_SIZE),
            r_half_texel,
        );
        gl.uniform2fv_with_f32_array(
            self.curl_program.uniforms.get(shaders::U_RESOLUTION),
            sim_resolution,
        );
        gl.uniform1i(
            self.curl_program.uniforms.get(shaders::U_VELOCITY),
            self.velocity_buffer.read().bind_webgl2(gl, 0)?,
        );

        Renderer::blit_webgl2(
            gl,
            Some(&self.temp_store),
            None,
        );

        // VORTICITY CONFINEMENT
        self.vorticity_program.bind_webgl2(gl);

        gl.uniform1f(
            self.vorticity_program.uniforms.get(shaders::U_CURL_SCALE),
            curl,
        );
        gl.uniform1f(
            self.vorticity_program.uniforms.get(shaders::U_R_HALF_TEXEL_SIZE),
            r_half_texel,
        );
        gl.uniform2fv_with_f32_array(
            self.vorticity_program.uniforms.get(shaders::U_RESOLUTION),
            sim_resolution,
        );
        gl.uniform1i(
            self.vorticity_program.uniforms.get(shaders::U_CURL),
            self.temp_store.bind_webgl2(gl, 0)?,
        );
        gl.uniform1i(
            self.vorticity_program.uniforms.get(shaders::U_VELOCITY),
            self.velocity_buffer.read().bind_webgl2(gl, 1)?,
        );

        Renderer::blit_webgl2(
            gl,
            Some(self.velocity_buffer.write()),
            None,
        );
        self.velocity_buffer.swap();

        Ok(())
    }

    pub fn update_webgl2(
        &mut self,
        gl: &WebGl2RenderingContext,
        pause: bool,
        delta_time: f32,
        sim_resolution: &[f32; 2],
        mode: Mode,
        viscosity: f32,
        dissipation: f32,
        curl: f32,
        pressure: f32,
    ) -> Result<(), JsValue> {
        // SIMULATION
        if !pause {
            // UPDATE VELOCITY
            self.vorticity_confinement_webgl2(
                gl,
                sim_resolution,
                curl,
            )?;

            Renderer::advect_webgl2(
                gl,
                &self.advection_program,
                &sim_resolution,
                delta_time,
                viscosity,
                None,
                &mut self.velocity_buffer,
            )?;

            self.project_velocity_webgl2(
                gl,
                &sim_resolution,
                PRESSURE_ITERATIONS,
                pressure,
            )?;

            // UPDATE DYE
            Renderer::advect_webgl2(
                gl,
                &self.advection_program,
                sim_resolution,
                delta_time,
                dissipation,
                Some(&self.velocity_buffer),
                &mut self.dye_buffer,
            )?;
        }

        // RENDER
        // DRAW TO CANVAS
        self.draw_pass_webgl2(gl, mode)?;

        Ok(())
    }

    pub fn resize_webgl2(
        &mut self,
        gl: &WebGl2RenderingContext,
        sim_resolution: Resolution,
        dye_resolution: Resolution,
    ) -> Result<(), JsValue> {
        // SIMULATION
        let (width, height) = Renderer::resolution_size(&self.canvas, sim_resolution);
        self.velocity_buffer.resize_webgl2(
            gl,
            Some(&self.copy_program),
            width,
            height,
        )?;

        self.pressure_buffer.resize_webgl2(
            gl,
            None,
            width,
            height,
        )?;

        if width != self.temp_store.width() || height != self.temp_store.height() {
            self.temp_store.delete_webgl2(gl);
            self.temp_store = TextureFramebuffer::new_webgl2(
                gl,
                width,
                height,
                WebGl2RenderingContext::FLOAT,
                WebGl2RenderingContext::LINEAR,
            )?;
        }
        
        // DYE
        let (width, height) = Renderer::resolution_size(&self.canvas, dye_resolution);
        self.dye_buffer.resize_webgl2(
            gl,
            Some(&self.copy_program),
            width,
            height,
        )?;

        Ok(())
    }

    pub fn splat_webgl2(
        &mut self,
        gl: &WebGl2RenderingContext,
        radius: f32,
        position: &[f32],
        velocity: &[f32],
        color: &[f32],
    ) -> Result<(), JsValue> {
        // APPLY FORCE
        let resolution = self.sim_resolution as u32 as f32;
        self.splat_program.bind_webgl2(gl);

        gl.uniform1f(
            self.splat_program.uniforms.get(shaders::U_SCALED_RADIUS),
            radius / (resolution * resolution),
        );
        gl.uniform2f(
            self.splat_program.uniforms.get(shaders::U_POSITION),
            position[0] / resolution,
            position[1] / resolution,
        );
        gl.uniform3f(
            self.splat_program.uniforms.get(shaders::U_COLOR),
            velocity[0] / resolution,
            velocity[1] / resolution,
            0.0,
        );
        gl.uniform1i(
            self.splat_program.uniforms.get(shaders::U_TEXTURE),
            self.velocity_buffer.read().bind_webgl2(gl, 0)?,
        );

        Renderer::blit_webgl2(
            gl,
            Some(self.velocity_buffer.write()),
            None,
        );
        self.velocity_buffer.swap();

        // APPLY COLOR
        let resolution = self.dye_resolution as u32 as f32;
        gl.uniform1f(
            self.splat_program.uniforms.get(shaders::U_SCALED_RADIUS),
            radius / (resolution * resolution),
        );
        gl.uniform2f(
            self.splat_program.uniforms.get(shaders::U_POSITION),
            position[0] / resolution,
            position[1] / resolution,
        );
        gl.uniform3fv_with_f32_array(
            self.splat_program.uniforms.get(shaders::U_COLOR),
            color,
        );
        gl.uniform1i(
            self.splat_program.uniforms.get(shaders::U_TEXTURE),
            self.dye_buffer.read().bind_webgl2(gl, 0)?,
        );

        Renderer::blit_webgl2(
            gl,
            Some(self.dye_buffer.write()),
            None,
        );
        self.dye_buffer.swap();

        Ok(())
    }
}