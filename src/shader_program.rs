use wasm_bindgen::prelude::*;
use web_sys::{
    WebGlProgram,
    WebGlRenderingContext,
    WebGl2RenderingContext,
    WebGlShader,
    WebGlUniformLocation,
};
use std::collections::HashMap;

pub struct ShaderProgram {
    program: WebGlProgram,
    pub uniforms: HashMap<String, WebGlUniformLocation>,
}

impl ShaderProgram {
    pub fn create_shader_webgl(
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

    pub fn create_shader_webgl2(
        gl: &WebGl2RenderingContext,
        shader_type: u32,
        source: &str,
    ) -> Result<WebGlShader, JsValue> {
        let shader = gl.create_shader(shader_type)
            .ok_or_else(|| JsValue::from_str("Unable to create shader object"))?;

        gl.shader_source(&shader, source);
        gl.compile_shader(&shader);

        if gl.get_shader_parameter(&shader, WebGl2RenderingContext::COMPILE_STATUS)
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

    pub fn new_webgl(
        gl: &WebGlRenderingContext,
        fragment_shader: &str,
        vertex_shader: &str,
    ) -> Result<ShaderProgram, JsValue> {
        let vertex_shader = ShaderProgram::create_shader_webgl(
            gl,
            WebGl2RenderingContext::VERTEX_SHADER,
            vertex_shader
        )?;
    
        let fragment_shader = ShaderProgram::create_shader_webgl(
            gl,
            WebGl2RenderingContext::FRAGMENT_SHADER,
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

            return Ok(ShaderProgram {
                program: shader_program,
                uniforms,
            });
        }
    
        Err(JsValue::from_str(
            &gl.get_program_info_log(&shader_program)
                .unwrap_or_else(|| "Unknown error linking program".into())
        ))
    }

    pub fn new_webgl2(
        gl: &WebGl2RenderingContext,
        fragment_shader: &str,
        vertex_shader: &str,
    ) -> Result<ShaderProgram, JsValue> {
        let vertex_shader = ShaderProgram::create_shader_webgl2(
            gl,
            WebGl2RenderingContext::VERTEX_SHADER,
            vertex_shader
        )?;
    
        let fragment_shader = ShaderProgram::create_shader_webgl2(
            gl,
            WebGl2RenderingContext::FRAGMENT_SHADER,
            &fragment_shader
        )?;
    
        let shader_program = gl.create_program()
            .ok_or_else(|| JsValue::from_str("Unable to create program"))?;
        gl.attach_shader(&shader_program, &vertex_shader);
        gl.attach_shader(&shader_program, &fragment_shader);
        gl.link_program(&shader_program);
    
        if gl.get_program_parameter(&shader_program, WebGl2RenderingContext::LINK_STATUS)
            .as_bool()
            .unwrap_or(false)
        {
            let count = gl.get_program_parameter(&shader_program, WebGl2RenderingContext::ACTIVE_UNIFORMS)
                .as_f64()
                .ok_or_else(|| JsValue::from_str("Unable to get program parameters"))? as u32;
            let mut uniforms = HashMap::with_capacity(count as usize);
            for i in 0..count {
                let name = gl.get_active_uniform(&shader_program, i).unwrap().name();
                uniforms.insert(name.clone(), gl.get_uniform_location(&shader_program, &name).unwrap());
            }

            return Ok(ShaderProgram {
                program: shader_program,
                uniforms,
            });
        }
    
        Err(JsValue::from_str(
            &gl.get_program_info_log(&shader_program)
                .unwrap_or_else(|| "Unknown error linking program".into())
        ))
    }
    
    pub fn bind_webgl(&self, gl: &WebGlRenderingContext) {
        gl.use_program(Some(&self.program));
    }

    pub fn bind_webgl2(&self, gl: &WebGl2RenderingContext) {
        gl.use_program(Some(&self.program));
    }
}