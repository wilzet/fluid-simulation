use super::*;
use web_sys::{
    WebGlRenderingContext,
    WebGlTexture,
    WebGlFramebuffer,
};
use std::mem;

pub struct TextureFramebuffer {
    texture: WebGlTexture,
    framebuffer: WebGlFramebuffer,
    width: u32,
    height: u32,
}

impl TextureFramebuffer {
    pub fn new(
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

        let data = unsafe { js_sys::Float32Array::view(&vec![0.0; (width * height * 4) as usize]) };
        gl.tex_image_2d_with_i32_and_i32_and_i32_and_format_and_type_and_opt_array_buffer_view(
            WebGlRenderingContext::TEXTURE_2D,
            0,
            WebGlRenderingContext::RGBA as i32,
            width as i32,
            height as i32,
            0,
            WebGlRenderingContext::RGBA,
            WebGlRenderingContext::FLOAT,
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

    pub fn bind(
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

    pub fn delete(&self, gl: &WebGlRenderingContext) {
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
    pub fn new(
        gl: &WebGlRenderingContext,
        width: u32,
        height: u32,
        param: Option<u32>,
    ) -> Result<RWTextureBuffer, JsValue> {
        let param = param.unwrap_or(WebGlRenderingContext::LINEAR);

        let read = TextureFramebuffer::new(
            gl,
            width,
            height,
            param,
        )?;
        let write = TextureFramebuffer::new(
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

    pub fn resize(
        &mut self,
        gl: &WebGlRenderingContext,
        width: u32,
        height: u32,
    ) -> Result<(), JsValue> {
        if width == self.read.width && height == self.read.height {
            return Ok(());
        }
        self.read.delete(&gl);
        self.write.delete(&gl);

        let new_buffer = RWTextureBuffer::new(
            &gl,
            width,
            height,
            Some(self.param),
        )?;

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