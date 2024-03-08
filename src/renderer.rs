use super::*;
use crate::shaders;

impl Renderer {
    pub fn new(
        gl: js_sys::Object,
        canvas: HtmlCanvasElement,
        sim_resolution: Resolution,
        dye_resolution: Resolution,
    ) -> Result<Renderer, JsValue> {
        let gl = gl.dyn_into::<WebGl2RenderingContext>().unwrap();

        gl.get_extension("EXT_color_buffer_float")?;
        gl.disable(WebGl2RenderingContext::BLEND);

        let copy_program = ShaderProgram::new(
            &gl,
            shaders::COPY_SHADER_SOURCE,
            shaders::VERTEX_SHADER_SOURCE,
        )?;
        let advection_program = ShaderProgram::new(
            &gl,
            shaders::ADVECTION_SHADER_SOURCE,
            shaders::VERTEX_SHADER_SOURCE,
        )?;
        let jacobi_program = ShaderProgram::new(
            &gl,
            shaders::PRESSURE_SOLVER_SHADER_SOURCE,
            shaders::VERTEX_SHADER_SOURCE,
        )?;
        let divergence_program = ShaderProgram::new(
            &gl,
            shaders::DIVERGENCE_SHADER_SOURCE,
            shaders::VERTEX_SHADER_SOURCE,
        )?;
        let subtraction_program = ShaderProgram::new(
            &gl,
            shaders::GRADIENT_SUBTRACT_SHADER_SOURCE,
            shaders::VERTEX_SHADER_SOURCE,
        )?;
        let curl_program = ShaderProgram::new(
            &gl,
            shaders::CURL_SHADER_SOURCE,
            shaders::VERTEX_SHADER_SOURCE,
        )?;
        let vorticity_program = ShaderProgram::new(
            &gl,
            shaders::VORTICITY_SHADER_SOURCE,
            shaders::VERTEX_SHADER_SOURCE,
        )?;
        let splat_program = ShaderProgram::new(
            &gl,
            shaders::SPLAT_SHADER_SOURCE,
            shaders::VERTEX_SHADER_SOURCE,
        )?;
        let obstacle_program = ShaderProgram::new(
            &gl,
            shaders::OBSTACLE_SHADER_SOURCE,
            shaders::VERTEX_SHADER_SOURCE,
        )?;

        let (width, height) = Renderer::resolution_size(&canvas, sim_resolution);
        let velocity_buffer = RWTextureBuffer::new(
            &gl,
            width,
            height,
            Some(WebGl2RenderingContext::LINEAR),
        )?;

        let pressure_buffer = RWTextureBuffer::new(
            &gl,
            width,
            height,
            Some(WebGl2RenderingContext::LINEAR),
        )?;

        let temp_store = TextureFramebuffer::new(
            &gl,
            width,
            height,
            WebGl2RenderingContext::LINEAR,
        )?;

        let (width, height) = Renderer::resolution_size(&canvas, dye_resolution);
        let dye_buffer = RWTextureBuffer::new(
            &gl,
            width,
            height,
            Some(WebGl2RenderingContext::LINEAR),
        )?;

        let obstacle_store = TextureFramebuffer::new(
            &gl,
            width,
            height,
            WebGl2RenderingContext::NEAREST,
        )?;

        Renderer::init_quad_buffers(&gl)?;

        let renderer = Renderer {
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
            obstacle_program,
            velocity_buffer,
            pressure_buffer,
            dye_buffer,
            obstacle_store,
            temp_store,
            last_time: 0.0,
        };

        renderer.set_obstacle(None, &[0.0, 0.0], true)?;

        Ok(renderer)
    }

    fn init_quad_buffers(gl: &WebGl2RenderingContext) -> Result<(), JsValue> {
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

    pub fn blit(
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

    fn pressure_solve(
        gl: &WebGl2RenderingContext,
        jacobi_program: &ShaderProgram,
        iterations: usize,
        resolution: &[f32; 2],
        alpha: f32,
        r_beta: f32,
        x: &mut RWTextureBuffer,
        b: Option<&TextureFramebuffer>,
        obstacle: &TextureFramebuffer,
    ) -> Result<(), JsValue> {
        jacobi_program.bind(gl);

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
                b.bind(gl, 1).ok()
            ).unwrap_or(0),
        );
        gl.uniform1i(
            jacobi_program.uniforms.get(shaders::U_OBSTACLES),
            obstacle.bind(gl, 2)?,
        );

        for _ in 0..iterations {
            gl.uniform1i(
                jacobi_program.uniforms.get(shaders::U_X),
                x.read().bind(gl, 0)?,
            );

            Renderer::blit(
                gl,
                Some(x.write()),
                None,
            );
            x.swap();
        }

        Ok(())
    }

    pub fn draw_pass(&self, mode: Mode) -> Result<(), JsValue> {
        let gl = &self.gl;

        self.copy_program.bind(gl);

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
                Mode::DYE => self.dye_buffer.read().bind(gl, 0)?,
                Mode::VELOCITY => self.velocity_buffer.read().bind(gl, 0)?,
                Mode::PRESSURE => self.pressure_buffer.read().bind(gl, 0)?,
            },
        );

        Renderer::blit(
            gl,
            None,
            Some(true),
        );

        Ok(())
    }

    pub fn advect(
        gl: &WebGl2RenderingContext,
        advection_program: &ShaderProgram,
        sim_resolution: &[f32; 2],
        delta_time: f32,
        dissipation: f32,
        velocity_buffer: Option<&RWTextureBuffer>,
        quantity: &mut RWTextureBuffer,
        obstacle: &TextureFramebuffer,
    ) -> Result<(), JsValue> {
        advection_program.bind(gl);

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
                b.read().bind(gl, 1).ok()
            ).unwrap_or(0),
        );
        gl.uniform1i(
            advection_program.uniforms.get(shaders::U_QUANTITY),
            quantity.read().bind(gl, 0)?,
        );
        gl.uniform1i(
            advection_program.uniforms.get(shaders::U_OBSTACLES),
            obstacle.bind(gl, 2)?,
        );

        Renderer::blit(
            gl,
            Some(quantity.write()),
            None,
        );
        quantity.swap();

        Ok(())
    }

    pub fn project_velocity(
        &mut self,
        sim_resolution: &[f32; 2],
        iterations: usize,
        pressure: f32,
    ) -> Result<(), JsValue> {
        let gl = &self.gl;
        let r_half_texel = 0.5 / (self.sim_resolution as u32 as f32);

        // DIVERGENCE
        self.divergence_program.bind(gl);

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
            self.velocity_buffer.read().bind(gl, 0)?,
        );
        gl.uniform1i(
            self.divergence_program.uniforms.get(shaders::U_OBSTACLES),
            self.obstacle_store.bind(gl, 1)?,
        );

        Renderer::blit(
            gl,
            Some(&self.temp_store),
            None,
        );

        // PRESSURE
        self.copy_program.bind(gl);

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
            self.pressure_buffer.read().bind(gl, 0)?,
        );

        Renderer::blit(
            gl,
            Some(self.pressure_buffer.write()),
            None,
        );
        self.pressure_buffer.swap();

        let alpha = self.sim_resolution as u32 as f32;
        let alpha = -alpha * alpha;
        let r_beta = 0.25;
        Renderer::pressure_solve(
            gl,
            &self.jacobi_program,
            iterations,
            sim_resolution,
            alpha,
            r_beta,
            &mut self.pressure_buffer,
            Some(&self.temp_store),
            &self.obstacle_store,
        )?;

        // SUBTRACTION
        self.subtraction_program.bind(gl);

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
            self.velocity_buffer.read().bind(gl, 0)?,
        );
        gl.uniform1i(
            self.subtraction_program.uniforms.get(shaders::U_PRESSURE),
            self.pressure_buffer.read().bind(gl, 1)?,
        );
        gl.uniform1i(
            self.subtraction_program.uniforms.get(shaders::U_OBSTACLES),
            self.obstacle_store.bind(gl, 2)?,
        );

        Renderer::blit(
            gl,
            Some(self.velocity_buffer.write()),
            None,
        );
        self.velocity_buffer.swap();

        Ok(())
    }

    pub fn vorticity_confinement(
        &mut self,
        sim_resolution: &[f32; 2],
        curl: f32,
    ) -> Result<(), JsValue> {
        let gl = &self.gl;
        let r_half_texel = 0.5 / (self.sim_resolution as u32 as f32);

        // CURL
        self.curl_program.bind(gl);

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
            self.velocity_buffer.read().bind(gl, 0)?,
        );

        Renderer::blit(
            gl,
            Some(&self.temp_store),
            None,
        );

        // VORTICITY CONFINEMENT
        self.vorticity_program.bind(gl);

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
            self.temp_store.bind(gl, 0)?,
        );
        gl.uniform1i(
            self.vorticity_program.uniforms.get(shaders::U_VELOCITY),
            self.velocity_buffer.read().bind(gl, 1)?,
        );

        Renderer::blit(
            gl,
            Some(self.velocity_buffer.write()),
            None,
        );
        self.velocity_buffer.swap();

        Ok(())
    }

    pub fn resolution_size(canvas: &HtmlCanvasElement, resolution: Resolution) -> (u32, u32) {
        let (width, height) = (canvas.width(), canvas.height());
        (width / resolution as u32, height / resolution as u32)
    }
}