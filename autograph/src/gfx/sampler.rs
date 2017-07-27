use gl;
use gl::types::*;

#[derive(Copy, Clone, Debug, Hash)]
pub enum TextureAddressMode
{
    Clamp = gl::CLAMP_TO_EDGE as isize,
    Mirror = gl::MIRRORED_REPEAT as isize,
    Wrap = gl::REPEAT as isize
}

#[derive(Copy, Clone, Debug, Hash)]
pub enum TextureMinFilter
{
    Nearest = gl::NEAREST as isize,
    Linear = gl::LINEAR as isize
}


#[derive(Copy, Clone, Debug, Hash)]
pub enum TextureMagFilter
{
    Nearest = gl::NEAREST as isize,
    Linear = gl::LINEAR as isize
}

// 2D sampler
#[derive(Copy, Clone, Hash)]
pub struct SamplerDesc
{
    pub addr_u: TextureAddressMode,
    pub addr_v: TextureAddressMode,
    pub addr_w: TextureAddressMode,
    pub min_filter: TextureMinFilter,
    pub mag_filter: TextureMagFilter,
}

impl Default for SamplerDesc
{
    fn default() -> SamplerDesc {
        SamplerDesc {
            addr_u: TextureAddressMode::Clamp,
            addr_v: TextureAddressMode::Clamp,
            addr_w: TextureAddressMode::Clamp,
            min_filter: TextureMinFilter::Nearest,
            mag_filter: TextureMagFilter::Linear
        }
    }
}

pub struct Sampler
{
    desc: SamplerDesc,
    obj: GLuint
}

impl Sampler
{
    pub fn new(desc: &SamplerDesc) -> Sampler {
        let mut obj: GLuint = 0;
        unsafe
        {
            gl::GenSamplers(1, &mut obj);
            gl::SamplerParameteri(obj, gl::TEXTURE_MIN_FILTER, desc.min_filter as i32);
            gl::SamplerParameteri(obj, gl::TEXTURE_MAG_FILTER, desc.mag_filter as i32);
            gl::SamplerParameteri(obj, gl::TEXTURE_WRAP_R, desc.addr_u as i32);
            gl::SamplerParameteri(obj, gl::TEXTURE_WRAP_S, desc.addr_v as i32);
            gl::SamplerParameteri(obj, gl::TEXTURE_WRAP_T, desc.addr_w as i32);
        }
        Sampler { desc: *desc, obj }
    }
}

/*pub fn build(&self) -> Sampler2D
    {
        let mut sampler: GLuint = 0;
        unsafe {
            gl::GenSamplers(1, &mut sampler);
            gl::SamplerParameteri(sampler, gl::TEXTURE_MIN_FILTER, self.min_filter.to_gl() as i32);
            gl::SamplerParameteri(sampler, gl::TEXTURE_MAG_FILTER, self.mag_filter.to_gl() as i32);
            gl::SamplerParameteri(sampler, gl::TEXTURE_WRAP_R, gl::CLAMP_TO_EDGE as i32);
            gl::SamplerParameteri(sampler, gl::TEXTURE_WRAP_S, gl::CLAMP_TO_EDGE as i32);
            gl::SamplerParameteri(sampler, gl::TEXTURE_WRAP_T, gl::CLAMP_TO_EDGE as i32);
        }

        Sampler2D {
            desc: self.clone(),
            obj: sampler
        }
    }*/