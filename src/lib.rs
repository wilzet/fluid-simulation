//! A fluid simulation that is compiled using `wasm-pack` and runs in the browser

mod shaders;
mod textures;
mod renderer;
mod shader_program;

use wasm_bindgen::prelude::*;
use web_sys::{
    HtmlCanvasElement,
    WebGlRenderingContext,
    WebGl2RenderingContext,
};
use crate::shader_program::ShaderProgram;
use crate::textures::{ TextureFramebuffer, RWTextureBuffer };

const PRESSURE_ITERATIONS: usize = 20;
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
    PRESSURE,
}

#[wasm_bindgen]
/// Renderer for the fluid simulation
pub struct Renderer {
    gl: Option<WebGlRenderingContext>,
    gl2: Option<WebGl2RenderingContext>,
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
    velocity_buffer: RWTextureBuffer,
    pressure_buffer: RWTextureBuffer,
    dye_buffer: RWTextureBuffer,
    temp_store: TextureFramebuffer,
    render_buffer: TextureFramebuffer,
    last_time: f32,
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
        
        // TRY WEBGL2
        match canvas.get_context_with_context_options("webgl2", &context_options) {
            Ok(Some(gl)) => Renderer::new_webgl2(
                gl,
                canvas,
                sim_resolution,
                dye_resolution,
            ),
            // TRY WEBGL
            _ => match canvas.get_context_with_context_options("webgl", &context_options) {
                Ok(Some(gl)) => Renderer::new_webgl(
                    gl,
                    canvas,
                    sim_resolution,
                    dye_resolution,
                ),
                _ => Err(JsValue::from_str("WebGL seems to not be enabled in the browser")),
            },
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
        viscosity: f32,
        dissipation: f32,
        curl: f32,
        pressure: f32,
    ) -> Result<(), JsValue> {
        let delta_time = FPS_30.min(time - self.last_time);
        self.last_time = time;

        let (width, height) = Renderer::resolution_size(&self.canvas, self.sim_resolution);
        let sim_resolution = [width as f32, height as f32];

        if let Some(gl) = &self.gl2.clone() {
            return self.update_webgl2(
                gl,
                pause,
                delta_time,
                &sim_resolution,
                mode,
                viscosity,
                dissipation,
                curl,
                pressure,
            );
        }

        let gl = &self.gl.clone().ok_or_else(|| JsValue::from_str("No WebGL rendering context found"))?;
        self.update_webgl(
            gl,
            pause,
            delta_time,
            &sim_resolution,
            mode,
            viscosity,
            dissipation,
            curl,
            pressure,
        )
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
        self.sim_resolution = sim_resolution;
        self.dye_resolution = dye_resolution;

        if let Some(gl) = &self.gl2.clone() {
            return self.resize_webgl2(
                gl,
                sim_resolution,
                dye_resolution,
            );
        }
            
        let gl = &self.gl.clone().ok_or_else(|| JsValue::from_str("No WebGL rendering context found"))?;
        self.resize_webgl(
            gl,
            sim_resolution,
            dye_resolution,
        )
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
        if let Some(gl) = &self.gl2.clone() {
            return self.splat_webgl2(
                gl,
                radius,
                position,
                velocity,
                color,
            );
        }
        
        let gl = &self.gl.clone().ok_or_else(|| JsValue::from_str("No WebGL rendering context found"))?;
        self.splat_webgl(
            gl,
            radius,
            position,
            velocity,
            color,
        )
    }
}