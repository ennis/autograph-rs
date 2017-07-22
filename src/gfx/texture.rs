use gl;
use gl::types::*;
use std::mem;
use super::texture_format::{TextureFormat, TextureDimensions};
use std::marker::PhantomData;
use bitflags;
use std::cmp::*;
use super::context::Context;
use std::rc::Rc;

bitflags! {
    #[derive(Default)]
    pub struct TextureOptions: u32 {
        ///
        const SPARSE_STORAGE = 0b00000001;
    }
}

#[derive(Copy,Clone,Debug)]
pub struct TextureDesc
{
    /// Texture dimensions
    pub dimensions: TextureDimensions,
    /// Texture format
    pub format: TextureFormat,
    /// Width in pixels
    pub width: u32,
    /// Height in pixels, or array size of 1D texture arrays
    pub height: u32,
    /// Depth in pixels, or array size of 2D texture arrays
    pub depth: u32,
    /// Number of samples for multisample textures
    /// 0 means that the texture will not be allocated with multisampling
    pub sample_count: u32,
    /// Number of mipmap levels that should be allocated for this texture
    /// See also: `get_texture_mip_map_count`
    pub mip_map_count: u32,
    ///
    pub options: TextureOptions
}

impl Default for TextureDesc {
    fn default() -> TextureDesc {
        TextureDesc { dimensions: TextureDimensions::Tex2D, format: TextureFormat::R8G8B8A8_UNORM, width: 0, height: 0, depth: 0, sample_count: 0, mip_map_count: 0, options: TextureOptions::empty() }
    }
}


///
/// Wrapper for OpenGL textures
///
/// To create a `Texture` object, use the constructor with a `TextureDesc`
/// object describing the texture
/// The underlying GL texture object is created with immutable storage, meaning
/// that it is impossible to reallocate the storage (resizing, adding mip
/// levels) once the texture is created
///
/// The texture object is bound to the context lifetime. It is checked dynamically.
#[derive(Debug)]
pub struct Texture
{
    pub obj: GLuint,
    desc: TextureDesc
}

impl Texture
{
    /// Returns the TextureDesc object describing this texture
    pub fn desc(&self) -> &TextureDesc {
        &self.desc
    }

    /// The width in pixels of this texture
    pub fn width(&self) -> u32 {
        self.desc.width
    }

    /// The height in pixels of this texture or the array size of 1D textures
    /// Returns 1 if it is a 1D texture
    pub fn height(&self) -> u32 {
        self.desc.height
    }

    /// The depth in pixels of this texture for 3D textures or the array size of 2D texture arrays
    pub fn depth(&self) -> u32 {
        self.desc.depth
    }

    /// Create a new texture object based on the given description
    pub fn new(ctx: Rc<Context>, desc: &TextureDesc) -> Rc<Texture> {
        Rc::new(Texture { obj: 0, desc: desc.clone() })
    }

    pub unsafe fn object(&self) -> GLuint {
        self.obj
    }
}

impl Drop for Texture
{
    fn drop(&mut self)
    {
        unsafe {
            gl::DeleteTextures(1, &self.obj);
        }
    }
}

///
/// Get the maximum number of mip map levels for a 2D texture of size (width,height)
/// numLevels = 1 + floor(log2(max(w, h, d)))
///
/// # References
///
/// https://stackoverflow.com/questions/9572414/how-many-mipmaps-does-a-texture-have-in-opengl
fn get_texture_mip_map_count(width: u32, height: u32) -> u32
{
    1 + f32::floor(f32::log2(max(width, height) as f32)) as u32
}
