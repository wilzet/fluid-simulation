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
pub enum Resolution {
    ONE = 1,
    TWO = 2,
    FOUR = 4,
    EIGHT = 8,
    SIXTEEN = 16,
}

#[derive(Clone, Copy)]
#[wasm_bindgen]
pub enum Mode {
    DYE,
    VELOCITY,
    PRESSURE,
}

#[wasm_bindgen]
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
    last_time: f32,
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