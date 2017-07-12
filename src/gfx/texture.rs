use gl;
use gl::types::*;
use std::mem;
use super::texture_format::{TextureFormat, TextureDimensions};
use std::marker::PhantomData;
use bitflags;
use std::cmp::*;
use super::context::Context;

bitflags! {
    #[derive(Default)]
    pub struct TextureOptions: u32 {
        ///
        const SPARSE_STORAGE = 0b00000001;
    }
}

#[derive(Copy,Clone)]
pub struct TextureDesc
{
    /// Texture dimensions
    pub dimensions: TextureDimensions,
    /// Texture format
    pub format: TextureFormat,
    /// Width in pixels
    pub width: i32,
    /// Height in pixels, or array size of 1D texture arrays
    pub height: i32,
    /// Depth in pixels, or array size of 2D texture arrays
    pub depth: i32,
    /// Number of samples for multisample textures
    /// 0 means that the texture will not be allocated with multisampling
    pub sampleCount: i32,
    /// Number of mipmap levels that should be allocated for this texture
    /// See also: `get_texture_mip_map_count`
    pub mipMapCount: i32,
    ///
    pub options: TextureOptions
}

impl Default for TextureDesc {
    fn default() -> TextureDesc {
        TextureDesc { dimensions: TextureDimensions::Tex2D, format: TextureFormat::R8G8B8A8_UNORM, width: 0, height: 0, depth: 0, sampleCount: 0, mipMapCount: 0, options: TextureOptions::empty() }
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
/// The texture object is bound to the context lifetime `'ctx`
pub struct Texture<'ctx>
{
    pub obj: GLuint,
    desc: TextureDesc,
    _phantom: PhantomData<&'ctx ()>
}

impl<'ctx> Texture<'ctx>
{
    /// Returns the TextureDesc object describing this texture
    pub fn desc(&self) -> &TextureDesc {
        &self.desc
    }

    /// The width in pixels of this texture
    pub fn width(&self) -> i32 {
        self.desc.width
    }

    /// The height in pixels of this texture or the array size of 1D textures
    /// Returns 1 if it is a 1D texture
    pub fn height(&self) -> i32 {
        self.desc.height
    }

    /// The depth in pixels of this texture for 3D textures or the array size of 2D texture arrays
    pub fn depth(&self) -> i32 {
        self.desc.depth
    }

    /// Create a new texture object based on the given description
    pub fn new<'a>(ctx: &'a Context, desc: &TextureDesc) -> Texture<'a> {
        Texture { obj: 0, desc: desc.clone(), _phantom: PhantomData }
    }
}

impl<'ctx> Drop for Texture<'ctx>
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
fn get_texture_mip_map_count(width: i32, height: i32) -> i32
{
    1 + f32::floor(f32::log2(max(width, height) as f32)) as i32
}
