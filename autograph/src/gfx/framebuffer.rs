use gl;
use gl::types::*;
use super::context::Context;
use std::sync::Arc;
use std::cmp::max;
use super::format::Format;
use super::texture::RawTexture;
use glutin::GlWindow;
use std::ops::Deref;

#[derive(Debug)]
pub struct RenderbufferObject {
    context: Context,
    format: Format,
    size: (u32, u32),
    obj: GLuint,
}

impl RenderbufferObject {
    pub fn new(
        _gctx: &Context,
        _width: u32,
        _height: u32,
        _format: Format,
        _num_samples: u32,
    ) -> RenderbufferObject {
        unimplemented!()
    }
}

#[derive(Debug)]
pub struct FramebufferObject {
    pub(super) gctx: Context,
    pub(super) size: (u32, u32),
    pub(super) obj: GLuint,
    pub(super) attachments: Vec<FramebufferAttachment>,
    pub(super) depth_attachment: FramebufferAttachment,
}


// TODO: FramebufferAttachment trait?
#[derive(Clone, Debug)]
pub enum FramebufferAttachment {
    Renderbuffer(Arc<RenderbufferObject>),
    Texture(RawTexture),    // TODO Texture2dAny
    TextureLayer(RawTexture, u32),  // TODO Texture2dAny
    Default,
    Empty,
}

impl FramebufferObject {
    pub fn from_gl_window(gctx: &Context, window: &GlWindow) -> FramebufferObject {
        let pixel_size = window.get_inner_size().unwrap();
        FramebufferObject {
            gctx: gctx.clone(),
            size: pixel_size,
            attachments: Vec::new(),
            depth_attachment: FramebufferAttachment::Default,
            obj: 0,
        }
    }

    pub fn size(&self) -> (u32, u32) {
        self.size
    }

    pub fn gl_object(&self) -> GLuint {
        self.obj
    }
}

impl Drop for FramebufferObject {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteFramebuffers(1, &mut self.obj);
        }
    }
}


#[derive(Clone, Debug)]
pub struct Framebuffer(Arc<FramebufferObject>);

impl Deref for Framebuffer
{
    type Target = Arc<FramebufferObject>;
    fn deref(&self) -> &Arc<FramebufferObject>
    {
        &self.0
    }
}

impl Framebuffer {
    /// Returns the default framebuffer object associated to the given GlWindow
    pub fn from_gl_window(gctx: &Context, window: &GlWindow) -> Framebuffer {
        Framebuffer(Arc::new(FramebufferObject::from_gl_window(gctx,window)))
    }
}

pub struct FramebufferBuilder {
    gctx: Context,
    size: (u32, u32),
    attachments: Vec<FramebufferAttachment>,
    depth_attachment: FramebufferAttachment,
}

impl FramebufferBuilder {
    pub fn new(gctx: &Context) -> Self {
        FramebufferBuilder {
            gctx: gctx.clone(),
            size: (0, 0),
            attachments: Vec::new(),
            depth_attachment: FramebufferAttachment::Empty,
        }
    }

    fn check_or_update_size(&mut self, size: (u32, u32)) -> bool {
        if self.size == (0, 0) {
            self.size = size;
            true
        } else {
            self.size == size
        }
    }

    pub fn attach(&mut self, slot: u32, attachment: FramebufferAttachment) {
        let len = self.attachments.len();
        self.attachments
            .resize(max(slot as usize + 1, len), FramebufferAttachment::Empty);
        self.attachments.insert(slot as usize, attachment);
    }

    pub fn attach_renderbuffer(mut self, slot: u32, renderbuffer: &Arc<RenderbufferObject>) -> Self {
        assert!(self.check_or_update_size(renderbuffer.size));
        self.attach(
            slot,
            FramebufferAttachment::Renderbuffer(renderbuffer.clone()),
        );
        self
    }

    pub fn attach_texture(mut self, slot: u32, texture: &RawTexture) -> Self {
        assert!(self.check_or_update_size(
            (texture.width(), texture.height())
        ));
        self.attach(slot, FramebufferAttachment::Texture(texture.clone()));
        self
    }

    pub fn attach_depth_renderbuffer(mut self, renderbuffer: &Arc<RenderbufferObject>) -> Self {
        assert!(self.check_or_update_size(renderbuffer.size));
        self.depth_attachment = FramebufferAttachment::Renderbuffer(renderbuffer.clone());
        self
    }

    pub fn attach_depth_texture(mut self, texture: &RawTexture) -> Self {
        assert!(self.check_or_update_size(
            (texture.width(), texture.height())
        ));
        self.depth_attachment = FramebufferAttachment::Texture(texture.clone());
        self
    }

    pub fn attach_texture_layer(self, _slot: u32) -> Self {
        unimplemented!()
    }

    pub fn build(self) -> Framebuffer {
        assert!(self.attachments.len() < 8);
        let mut obj = 0;
        unsafe {
            gl::CreateFramebuffers(1, &mut obj);
        }

        for (index, attachment) in self.attachments.iter().enumerate() {
            match attachment {
                &FramebufferAttachment::Texture(ref tex) => unsafe {
                    gl::NamedFramebufferTexture(
                        obj,
                        gl::COLOR_ATTACHMENT0 + index as u32,
                        tex.gl_object(),
                        0,
                    );
                },
                &FramebufferAttachment::Renderbuffer(ref renderbuffer) => unsafe {
                    gl::NamedFramebufferRenderbuffer(
                        obj,
                        gl::COLOR_ATTACHMENT0 + index as u32,
                        gl::RENDERBUFFER,
                        renderbuffer.obj,
                    );
                },
                &FramebufferAttachment::Empty => (),
                _ => unimplemented!("Framebuffer attachment not implemented"),
            }
        }

        unsafe {
            gl::NamedFramebufferDrawBuffers(
                obj,
                8,
                [
                    gl::COLOR_ATTACHMENT0,
                    gl::COLOR_ATTACHMENT0 + 1,
                    gl::COLOR_ATTACHMENT0 + 2,
                    gl::COLOR_ATTACHMENT0 + 3,
                    gl::COLOR_ATTACHMENT0 + 4,
                    gl::COLOR_ATTACHMENT0 + 5,
                    gl::COLOR_ATTACHMENT0 + 6,
                    gl::COLOR_ATTACHMENT0 + 7,
                ].as_ptr(),
            );
        }

        Framebuffer(Arc::new(FramebufferObject {
            obj,
            attachments: self.attachments,
            gctx: self.gctx,
            depth_attachment: self.depth_attachment,
            size: self.size,
        }))
    }
}
