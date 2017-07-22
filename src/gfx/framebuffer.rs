use gl;
use gl::types::*;
use super::context::Context;
use std::rc::Rc;
use std::cmp::max;
use super::texture_format::TextureFormat;
use super::texture::Texture;
use glutin::GlWindow;

#[derive(Debug)]
struct Renderbuffer
{
    context: Rc<Context>,
    format: TextureFormat,
    size: (u32,u32),
    obj: GLuint
}

impl Renderbuffer
{
    pub fn new(context: Rc<Context>, width: u32, height: u32, format: TextureFormat, num_samples: u32) {

    }
}

#[derive(Debug)]
struct Framebuffer
{
    context: Rc<Context>,
    size: (u32,u32),
    obj: GLuint,
    attachments: Vec<FramebufferAttachment>,
    depth_attachment: FramebufferAttachment
}

// TODO: FramebufferAttachment trait?
#[derive(Clone,Debug)]
enum FramebufferAttachment
{
    Renderbuffer(Rc<Renderbuffer>),
    Texture(Rc<Texture>),
    TextureLayer(Rc<Texture>, u32),
    Default,
    Empty
}

struct FramebufferBuilder
{
    context: Rc<Context>,
    size: (u32,u32),
    attachments: Vec<FramebufferAttachment>,
    depth_attachment: FramebufferAttachment
}

impl FramebufferBuilder {
    pub fn new(ctx: Rc<Context>) -> Self {
        FramebufferBuilder {
            context: ctx.clone(),
            size: (0,0),
            attachments: Vec::new(),
            depth_attachment: FramebufferAttachment::Empty
        }
    }

    pub fn check_or_update_size(&mut self, size: (u32,u32)) -> bool {
        if self.size == (0,0) {
            self.size = size;
            true
        } else {
            self.size == size
        }
    }

    pub fn attach(&mut self, slot: u32, attachment: FramebufferAttachment) {
        let len = self.attachments.len();
        self.attachments.resize(max(slot as usize + 1, len), FramebufferAttachment::Empty);
        self.attachments.insert(slot as usize, attachment);
    }

    pub fn attach_renderbuffer(mut self, slot: u32, renderbuffer: Rc<Renderbuffer>) -> Self {
        assert!(self.check_or_update_size(renderbuffer.size));
        self.attach(slot, FramebufferAttachment::Renderbuffer(renderbuffer.clone()));
        self
    }

    pub fn attach_texture(mut self, slot: u32, texture: Rc<Texture>) -> Self {
        assert!(self.check_or_update_size((texture.width(), texture.height())));
        self.attach(slot, FramebufferAttachment::Texture(texture.clone()));
        self
    }

    pub fn attach_depth_renderbuffer(mut self, renderbuffer: Rc<Renderbuffer>) -> Self {
        assert!(self.check_or_update_size(renderbuffer.size));
        self.depth_attachment = FramebufferAttachment::Renderbuffer(renderbuffer.clone());
        self
    }

    pub fn attach_depth_texture(mut self, texture: Rc<Texture>) -> Self {
        assert!(self.check_or_update_size((texture.width(), texture.height())));
        self.depth_attachment = FramebufferAttachment::Texture(texture.clone());
        self
    }

    pub fn attach_texture_layer(mut self, slot: u32) -> Self {
        unimplemented!()
    }

    pub fn build(self) -> Framebuffer
    {
        assert!(self.attachments.len() < 8);
        let mut obj = 0;
        unsafe {
            gl::CreateFramebuffers(1, &mut obj);
        }

        for (index,attachment) in self.attachments.iter().enumerate() {
            match attachment {
                &FramebufferAttachment::Texture(ref tex) => unsafe {
                    gl::NamedFramebufferTexture(obj, gl::COLOR_ATTACHMENT0 + index as u32, tex.object(), 0);
                },
                &FramebufferAttachment::Renderbuffer(ref renderbuffer) => unsafe {
                    gl::NamedFramebufferRenderbuffer(obj, gl::COLOR_ATTACHMENT0 + index as u32, gl::RENDERBUFFER, renderbuffer.obj);
                },
                &FramebufferAttachment::Empty => (),
                _ => unimplemented!("Framebuffer attachment not implemented")
            }
        }

        unsafe {
            gl::NamedFramebufferDrawBuffers(obj, 8, [
                gl::COLOR_ATTACHMENT0, gl::COLOR_ATTACHMENT0 + 1,
                gl::COLOR_ATTACHMENT0 + 2, gl::COLOR_ATTACHMENT0 + 3,
                gl::COLOR_ATTACHMENT0 + 4, gl::COLOR_ATTACHMENT0 + 5,
                gl::COLOR_ATTACHMENT0 + 6, gl::COLOR_ATTACHMENT0 + 7].as_ptr());
        }

        Framebuffer {
            obj,
            attachments: self.attachments,
            context: self.context,
            depth_attachment: self.depth_attachment,
            size: self.size
        }
    }

}

impl Framebuffer
{
    pub fn from_gl_window(context: Rc<Context>, window: GlWindow) -> Framebuffer {
        let pixel_size = window.get_inner_size_pixels().unwrap();
        Framebuffer {
            context: context.clone(),
            size: pixel_size,
            attachments: Vec::new(),
            depth_attachment: FramebufferAttachment::Default,
            obj: 0
        }
    }
}

impl Drop for Framebuffer
{
    fn drop(&mut self) {
        unsafe {
            gl::DeleteFramebuffers(1, &mut self.obj);
        }
    }
}

