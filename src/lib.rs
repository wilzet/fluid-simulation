//! A fluid simulation that is compiled using `wasm-pack` and runs in the browser

mod shaders;
mod textures;
mod renderer;
mod shader_program;

use wasm_bindgen::prelude::*;
use web_sys::{ HtmlCanvasElement, WebGl2RenderingContext };
use crate::textures::{ TextureFramebuffer, RWTextureBuffer };
use crate::shader_program::ShaderProgram;

const MIN_PRESSURE_ITERATIONS: usize = 20;
const FPS_30: f32 = 0.0333333;

#[repr(u8)]
#[derive(Clone, Copy)]
#[wasm_bindgen]
/// Describes the scaling of a texture used by the [renderer](Renderer)
pub enum Resolution {
    ONE = 1,
    TWO = 2,
    FOUR = 4,
    EIGHT = 8,
    SIXTEEN = 16,
}

#[derive(Clone, Copy)]
#[wasm_bindgen]
/// Mode for the draw pass of the [renderer](Renderer)
pub enum Mode {
    DYE,
    VELOCITY,
}

#[wasm_bindgen]
/// Renderer for the fluid simulation
pub struct Renderer {
    gl: WebGl2RenderingContext,
    canvas: HtmlCanvasElement,
    sim_resolution: Resolution,
    dye_resolution: Resolution,
    copy_program: ShaderProgram,
    advection_program: ShaderProgram,
    jacobi_program: ShaderProgram,
    divergence_program: ShaderProgram,
    subtraction_program: ShaderProgram,
    curl_program: ShaderProgram,
    vorticity_program: ShaderProgram,
    splat_program: ShaderProgram,
    obstacle_program: ShaderProgram,
    color_obstacle_program: ShaderProgram,
    velocity_buffer: RWTextureBuffer,
    pressure_buffer: RWTextureBuffer,
    dye_buffer: RWTextureBuffer,
    obstacle_store: TextureFramebuffer,
    temp_store: TextureFramebuffer,
    last_time: f32,
    obstacle_color: [f32; 3],
}

#[wasm_bindgen]
impl Renderer {
    /// Create a new renderer
    ///
    /// There should really only ever exist one renderer.
    /// 
    /// # Arguments
    /// * `canvas_id` - id of the canvas element
    /// * `sim_resolution` - A [Resolution](Resolution) describing the scaling of the simulation in relation to the window size
    /// * `dye_resolution` - A [Resolution](Resolution) describing the scaling of the dye in relation to the window size
    /// 
    /// # Returns
    /// The renderer object, or an error if neither the WebGL, nor the WebGL2, rendering context can be found.
    ///
    /// # Panics
    /// May panic if no html elements can be found.
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

        match canvas.get_context_with_context_options("webgl2", &context_options) {
            Ok(Some(gl)) => Renderer::new(
                gl,
                canvas,
                sim_resolution,
                dye_resolution,
            ),
            _ => Err(JsValue::from_str("WebGL 2 seems to not be enabled in the browser")),
        }
    }

    /// Update the renderer
    /// 
    /// Updates the simulation according to the provided arguments.
    /// 
    /// # Arguments
    /// * `pause` - Should the simulation be paused?
    /// * `time` - Current time (may be current datetime or time since the beginning of the program run but needs to be consistent)
    /// * `mode` - Rendering [mode](Mode)
    /// * `viscosity` - Energy loss of the fluid due to friction (>= 0) 
    /// * `dissipation` - Colored dye fading amount (>= 0)
    /// * `curl` - Curl amount [0, 1]
    /// * `pressure` - Pressure coefficient for converging pressure calculation
    /// 
    /// # Returns
    /// May return an error if something in the WebGL pipeline were to break.
    pub fn update(
        &mut self,
        pause: bool,
        time: f32,
        mode: Mode,
        iterations: usize,
        viscosity: f32,
        dissipation: f32,
        curl: f32,
        pressure: f32,
    ) -> Result<(), JsValue> {
        let delta_time = FPS_30.min(time - self.last_time);
        self.last_time = time;

        let (width, height) = Renderer::resolution_size(&self.canvas, self.sim_resolution);
        let sim_resolution = [width as f32, height as f32];

        // SIMULATION
        if !pause {
            // UPDATE VELOCITY
            self.vorticity_confinement(
                &sim_resolution,
                curl,
            )?;

            Renderer::advect(
                &self.gl,
                &self.advection_program,
                &sim_resolution,
                delta_time,
                viscosity,
                None,
                &mut self.velocity_buffer,
                &self.obstacle_store,
            )?;

            self.project_velocity(
                &sim_resolution,
                MIN_PRESSURE_ITERATIONS.max(iterations),
                pressure,
            )?;

            // UPDATE DYE
            Renderer::color_obstacle(
                &self.gl,
                &self.color_obstacle_program,
                &self.obstacle_store,
                &mut self.dye_buffer,
                &[0.0, 0.0, 0.0],
            )?;

            Renderer::advect(
                &self.gl,
                &self.advection_program,
                &sim_resolution,
                delta_time,
                dissipation,
                Some(&self.velocity_buffer),
                &mut self.dye_buffer,
                &self.obstacle_store,
            )?;

            Renderer::color_obstacle(
                &self.gl,
                &self.color_obstacle_program,
                &self.obstacle_store,
                &mut self.dye_buffer,
                &self.obstacle_color,
            )?;
        }

        // RENDER
        // DRAW TO CANVAS
        self.draw_pass(mode)?;

        Ok(())
    }

    /// Resize the renderer
    /// 
    /// Resizes the textures and buffers used in the simulation.
    /// 
    /// # Arguments
    /// * `sim_resolution` - A [Resolution](Resolution) describing the scaling of the simulation in relation to the window size
    /// * `dye_resolution` - A [Resolution](Resolution) describing the scaling of the dye in relation to the window size
    /// 
    /// # Returns
    /// May return an error if something in the WebGL pipeline were to break.
    pub fn resize(
        &mut self,
        sim_resolution: Resolution,
        dye_resolution: Resolution,
    ) -> Result<(), JsValue> {
        let gl = &self.gl;
        self.sim_resolution = sim_resolution;
        self.dye_resolution = dye_resolution;
        
        // SIMULATION
        let (width, height) = Renderer::resolution_size(&self.canvas, sim_resolution);
        self.velocity_buffer.resize(
            gl,
            Some(&self.copy_program),
            width,
            height,
        )?;

        self.pressure_buffer.resize(
            gl,
            None,
            width,
            height,
        )?;

        if width != self.temp_store.width() || height != self.temp_store.height() {
            self.temp_store.delete(gl);
            self.temp_store = TextureFramebuffer::new(
                gl,
                width,
                height,
                WebGl2RenderingContext::LINEAR,
            )?;
        }
        
        // DYE
        let (width, height) = Renderer::resolution_size(&self.canvas, dye_resolution);
        self.dye_buffer.resize(
            gl,
            Some(&self.copy_program),
            width,
            height,
        )?;

        if width != self.obstacle_store.width() || height != self.obstacle_store.height() {
            self.obstacle_store.delete(gl);
            self.obstacle_store = TextureFramebuffer::new(
                gl,
                width,
                height,
                WebGl2RenderingContext::NEAREST,
            )?;

            self.set_obstacle(None, &[0.0, 0.0], &[0.0, 0.0, 0.0], true)?;
        }

        Ok(())
    }

    /// Create a splat
    /// 
    /// Adds a splat of force and color to the simulation.
    /// 
    /// # Arguments
    /// * `radius` - Radius of the splat in pixels
    /// * `position` - A float array that should have two values, an x and a y position in screen coordinates
    /// * `velocity` - A float array that should have two values, an x and a y velocity
    /// * `color` - A float array that should have three values, a red, a green, and a blue color value
    /// 
    /// # Returns
    /// May return an error if something in the WebGL pipeline were to break.
    ///
    /// # Panics
    /// If either `position` or `velocity` contains fewer than two values, or if `color` contains fewer than three values.
    pub fn splat(
        &mut self,
        radius: f32,
        position: &[f32],
        velocity: &[f32],
        color: &[f32],
    ) -> Result<(), JsValue> {
        let gl = &self.gl;
        self.splat_program.bind(gl);

        // APPLY FORCE
        let resolution = self.sim_resolution as u32 as f32;
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
            self.velocity_buffer.read().bind(gl, 0)?,
        );
        gl.uniform1i(
            self.splat_program.uniforms.get(shaders::U_OBSTACLES),
            self.obstacle_store.bind(gl, 1)?,
        );

        Renderer::blit(
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
            self.dye_buffer.read().bind(gl, 0)?,
        );

        Renderer::blit(
            gl,
            Some(self.dye_buffer.write()),
            None,
        );
        self.dye_buffer.swap();

        Ok(())
    }

    /// Set obstacle
    /// 
    /// Set either a circular or square obstacle.
    /// 
    /// # Arguments
    /// * `radius` - Radius of the obstacle in pixels (in the case of a square it is half the sidelength in pixels). If this value is `undefined`, no obstacle will be set
    /// * `position` - A float array that should have two values, an x and a y position in screen coordinates
    /// * `color` - A float array that should have three values, a red, a green, and a blue color value
    /// * `is_circle` - A boolean value deciding whether the obstacle is a circle or a square
    /// 
    /// # Returns
    /// May return an error if something in the WebGL pipeline were to break.
    ///
    /// # Panics
    /// If `position` contains fewer than two values, or if `color` contains fewer than three values.
    pub fn set_obstacle(
        &mut self,
        radius: Option<f32>,
        position: &[f32],
        color: &[f32],
        is_circle: bool,
    ) -> Result<(), JsValue> {
        let gl = &self.gl;

        Renderer::color_obstacle(
            gl,
            &self.color_obstacle_program,
            &self.obstacle_store,
            &mut self.dye_buffer,
            &[0.0, 0.0, 0.0],
        )?;

        self.obstacle_program.bind(gl);

        // SET OBSTACLE
        let resolution = self.dye_resolution as u32 as f32;
        gl.uniform1i(
            self.obstacle_program.uniforms.get(shaders::U_IS_CIRCLE),
            is_circle as i32,
        );
        gl.uniform1f(
            self.obstacle_program.uniforms.get(shaders::U_SCALED_RADIUS_SQR),
            radius.map_or(-10.0, |r| r * r / (resolution * resolution)),
        );
        gl.uniform2f(
            self.obstacle_program.uniforms.get(shaders::U_POSITION),
            position[0] / resolution,
            position[1] / resolution,
        );

        Renderer::blit(
            gl,
            Some(&self.obstacle_store),
            None,
        );

        // SET COLOR
        self.obstacle_color = [color[0], color[1], color[2]];

        Ok(())
    }
}