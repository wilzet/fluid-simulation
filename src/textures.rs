use wasm_bindgen::prelude::*;
use web_sys::{
    WebGlRenderingContext,
    WebGl2RenderingContext,
    WebGlTexture,
    WebGlFramebuffer,
    OesTextureHalfFloat,
};
use std::mem;
use crate::Renderer;
use crate::shader_program::ShaderProgram;
use crate::shaders;

pub struct TextureFramebuffer {
    texture: WebGlTexture,
    framebuffer: WebGlFramebuffer,
    width: u32,
    height: u32,
}

impl TextureFramebuffer {
    pub fn new_webgl(
        gl: &WebGlRenderingContext,
        width: u32,
        height: u32,
        param: u32,
    ) -> Result<TextureFramebuffer, JsValue> {
        gl.active_texture(WebGlRenderingContext::TEXTURE0);
        let texture = gl.create_texture().unwrap();
        gl.bind_texture(WebGlRenderingContext::TEXTURE_2D, Some(&texture));

        gl.tex_parameteri(
            WebGlRenderingContext::TEXTURE_2D,
            WebGlRenderingContext::TEXTURE_MIN_FILTER,
            param as i32,
        );
        gl.tex_parameteri(
            WebGlRenderingContext::TEXTURE_2D,
            WebGlRenderingContext::TEXTURE_MAG_FILTER,
            param as i32,
        );
        gl.tex_parameteri(
            WebGlRenderingContext::TEXTURE_2D,
            WebGlRenderingContext::TEXTURE_WRAP_S,
            WebGlRenderingContext::CLAMP_TO_EDGE as i32,
        );
        gl.tex_parameteri(
            WebGlRenderingContext::TEXTURE_2D,
            WebGlRenderingContext::TEXTURE_WRAP_T,
            WebGlRenderingContext::CLAMP_TO_EDGE as i32,
        );

        let data = unsafe { js_sys::Uint16Array::view(&vec![0; (width * height * 4) as usize]) };
        gl.tex_image_2d_with_i32_and_i32_and_i32_and_format_and_type_and_opt_array_buffer_view(
            WebGlRenderingContext::TEXTURE_2D,
            0,
            WebGlRenderingContext::RGBA as i32,
            width as i32,
            height as i32,
            0,
            WebGlRenderingContext::RGBA,
            OesTextureHalfFloat::HALF_FLOAT_OES,
            Some(&data),
        )?;
        
        let framebuffer = gl.create_framebuffer().unwrap();
        gl.bind_framebuffer(WebGlRenderingContext::FRAMEBUFFER, Some(&framebuffer));
        gl.framebuffer_texture_2d(
            WebGlRenderingContext::FRAMEBUFFER,
            WebGlRenderingContext::COLOR_ATTACHMENT0,
            WebGlRenderingContext::TEXTURE_2D,
            Some(&texture),
            0,
        );

        Ok(TextureFramebuffer {
            texture,
            framebuffer,
            width,
            height,
        })
    }

    pub fn new_webgl2(
        gl: &WebGl2RenderingContext,
        width: u32,
        height: u32,
        param: u32,
    ) -> Result<TextureFramebuffer, JsValue> {
        gl.active_texture(WebGl2RenderingContext::TEXTURE0);
        let texture = gl.create_texture().unwrap();
        gl.bind_texture(WebGl2RenderingContext::TEXTURE_2D, Some(&texture));

        gl.tex_parameteri(
            WebGl2RenderingContext::TEXTURE_2D,
            WebGl2RenderingContext::TEXTURE_MIN_FILTER,
            param as i32,
        );
        gl.tex_parameteri(
            WebGl2RenderingContext::TEXTURE_2D,
            WebGl2RenderingContext::TEXTURE_MAG_FILTER,
            param as i32,
        );
        gl.tex_parameteri(
            WebGl2RenderingContext::TEXTURE_2D,
            WebGl2RenderingContext::TEXTURE_WRAP_S,
            WebGl2RenderingContext::CLAMP_TO_EDGE as i32,
        );
        gl.tex_parameteri(
            WebGl2RenderingContext::TEXTURE_2D,
            WebGl2RenderingContext::TEXTURE_WRAP_T,
            WebGl2RenderingContext::CLAMP_TO_EDGE as i32,
        );

        let data = unsafe { js_sys::Uint16Array::view(&vec![0; (width * height * 4) as usize]) };
        gl.tex_image_2d_with_i32_and_i32_and_i32_and_format_and_type_and_opt_array_buffer_view(
            WebGl2RenderingContext::TEXTURE_2D,
            0,
            WebGl2RenderingContext::RGBA16F as i32,
            width as i32,
            height as i32,
            0,
            WebGl2RenderingContext::RGBA,
            WebGl2RenderingContext::HALF_FLOAT,
            Some(&data),
        )?;
        
        let framebuffer = gl.create_framebuffer().unwrap();
        gl.bind_framebuffer(WebGl2RenderingContext::FRAMEBUFFER, Some(&framebuffer));
        gl.framebuffer_texture_2d(
            WebGl2RenderingContext::FRAMEBUFFER,
            WebGl2RenderingContext::COLOR_ATTACHMENT0,
            WebGl2RenderingContext::TEXTURE_2D,
            Some(&texture),
            0,
        );

        Ok(TextureFramebuffer {
            texture,
            framebuffer,
            width,
            height,
        })
    }

    pub fn bind_webgl(
        &self,
        gl: &WebGlRenderingContext,
        id: u32,
    ) -> Result<i32, JsValue> {
        if id >= 32 {
            return Err(JsValue::from_str(
                "id >= 32".into()
            ));
        }

        gl.active_texture(WebGlRenderingContext::TEXTURE0 + id);
        gl.bind_texture(WebGlRenderingContext::TEXTURE_2D, Some(&self.texture));

        Ok(id as i32)
    }

    pub fn bind_webgl2(
        &self,
        gl: &WebGl2RenderingContext,
        id: u32,
    ) -> Result<i32, JsValue> {
        if id >= 32 {
            return Err(JsValue::from_str(
                "id >= 32".into()
            ));
        }

        gl.active_texture(WebGl2RenderingContext::TEXTURE0 + id);
        gl.bind_texture(WebGl2RenderingContext::TEXTURE_2D, Some(&self.texture));

        Ok(id as i32)
    }

    pub fn delete_webgl(&self, gl: &WebGlRenderingContext) {
        gl.delete_texture(Some(&self.texture));
        gl.delete_framebuffer(Some(&self.framebuffer));
    }

    pub fn delete_webgl2(&self, gl: &WebGl2RenderingContext) {
        gl.delete_texture(Some(&self.texture));
        gl.delete_framebuffer(Some(&self.framebuffer));
    }

    pub fn buffer(&self) -> &WebGlFramebuffer {
        &self.framebuffer
    }

    pub fn width(&self) -> u32 {
        self.width
    }

    pub fn height(&self) -> u32 {
        self.height
    }
}

pub struct RWTextureBuffer {
    read: TextureFramebuffer,
    write: TextureFramebuffer,
    param: u32,
}

impl RWTextureBuffer {
    pub fn new_webgl(
        gl: &WebGlRenderingContext,
        width: u32,
        height: u32,
        param: Option<u32>,
    ) -> Result<RWTextureBuffer, JsValue> {
        let param = param.unwrap_or(WebGlRenderingContext::LINEAR);

        let read = TextureFramebuffer::new_webgl(
            gl,
            width,
            height,
            param,
        )?;
        let write = TextureFramebuffer::new_webgl(
            gl,
            width,
            height,
            param,
        )?;

        Ok(RWTextureBuffer {
            read,
            write,
            param,
        })
    }

    pub fn new_webgl2(
        gl: &WebGl2RenderingContext,
        width: u32,
        height: u32,
        param: Option<u32>,
    ) -> Result<RWTextureBuffer, JsValue> {
        let param = param.unwrap_or(WebGl2RenderingContext::LINEAR);

        let read = TextureFramebuffer::new_webgl2(
            gl,
            width,
            height,
            param,
        )?;
        let write = TextureFramebuffer::new_webgl2(
            gl,
            width,
            height,
            param,
        )?;

        Ok(RWTextureBuffer {
            read,
            write,
            param,
        })
    }

    pub fn resize_webgl(
        &mut self,
        gl: &WebGlRenderingContext,
        copy_program: Option<&ShaderProgram>,
        width: u32,
        height: u32,
    ) -> Result<(), JsValue> {
        if width == self.read.width && height == self.read.height {
            return Ok(());
        }

        let new_buffer = RWTextureBuffer::new_webgl(
            gl,
            width,
            height, 
            Some(self.param),
        )?;

        // COPY
        if let Some(copy_program) = copy_program {
            copy_program.bind_webgl(gl);
            gl.uniform1f(
                copy_program.uniforms.get(shaders::U_FACTOR),
                1.0,
            );
            gl.uniform1f(
                copy_program.uniforms.get(shaders::U_OFFSET),
                0.0,
            );
            gl.uniform1i(
                copy_program.uniforms.get(shaders::U_TEXTURE),
                self.read.bind_webgl(gl, 0)?,
            );

            Renderer::blit_webgl(
                gl,
                Some(&new_buffer.read),
                None,
            );
        }

        // DELETE AND SET
        self.read.delete_webgl(gl);
        self.write.delete_webgl(gl);

        self.read = new_buffer.read;
        self.write = new_buffer.write;

        Ok(())
    }

    pub fn resize_webgl2(
        &mut self,
        gl: &WebGl2RenderingContext,
        copy_program: Option<&ShaderProgram>,
        width: u32,
        height: u32,
    ) -> Result<(), JsValue> {
        if width == self.read.width && height == self.read.height {
            return Ok(());
        }

        let new_buffer = RWTextureBuffer::new_webgl2(
            gl,
            width,
            height, 
            Some(self.param),
        )?;

        // COPY
        if let Some(copy_program) = copy_program {
            copy_program.bind_webgl2(gl);
            gl.uniform1f(
                copy_program.uniforms.get(shaders::U_FACTOR),
                1.0,
            );
            gl.uniform1f(
                copy_program.uniforms.get(shaders::U_OFFSET),
                0.0,
            );
            gl.uniform1i(
                copy_program.uniforms.get(shaders::U_TEXTURE),
                self.read.bind_webgl2(gl, 0)?,
            );

            Renderer::blit_webgl2(
                gl,
                Some(&new_buffer.read),
                None,
            );
        }

        // DELETE AND SET
        self.read.delete_webgl2(gl);
        self.write.delete_webgl2(gl);

        self.read = new_buffer.read;
        self.write = new_buffer.write;

        Ok(())
    }

    pub fn swap(&mut self) {
        mem::swap(&mut self.read, &mut self.write);
    }

    pub fn read(&self) -> &TextureFramebuffer {
        &self.read
    }

    pub fn write(&self) -> &TextureFramebuffer {
        &self.write
    }
}