use super::shader_interface::{ShaderInterfaceDesc, VertexLayout, Type};
use failure::Error;
use gl;
use gl::types::*;
use gfx::{RawBufferSlice,RawTexture,Sampler};

/// A trait representing a shader
pub trait Shader {}
pub trait VertexShader: Shader {}
pub trait FragmentShader: Shader {}
pub trait GeometryShader: Shader {}
pub trait TessControlShader: Shader {}
pub trait TessEvalShader: Shader {}
pub trait ComputeShader: Shader {}

/// Provides methods for safely binding uniforms to the OpenGL pipeline.
pub unsafe trait UniformBinder
{
    unsafe fn bind_uniform_f32_unchecked(&self, location: u32, v: f32);
    unsafe fn bind_uniform_vec2_unchecked(&self, location: u32, v: [f32; 2]);
    unsafe fn bind_uniform_vec3_unchecked(&self, location: u32, v: [f32; 3]);
    unsafe fn bind_uniform_vec4_unchecked(&self, location: u32, v: [f32; 4]);
    unsafe fn bind_uniform_i32_unchecked(&self, location: u32, v: i32);
    unsafe fn bind_uniform_ivec2_unchecked(&self, location: u32, v: [i32; 2]);
    unsafe fn bind_uniform_ivec3_unchecked(&self, location: u32, v: [i32; 3]);
    unsafe fn bind_uniform_ivec4_unchecked(&self, location: u32, v: [i32; 4]);
    unsafe fn bind_uniform_mat2_unchecked(&self, location: u32, v: [f32; 2*2]);
    unsafe fn bind_uniform_mat3_unchecked(&self, location: u32, v: [f32; 3*3]);
    unsafe fn bind_uniform_mat4_unchecked(&self, location: u32, v: [f32; 4*4]);

    unsafe fn bind_uniform_buffer_unchecked(&self, slot: u32, buffer: &RawBufferSlice);
    unsafe fn bind_shader_storage_buffer_unchecked(&self, slot: u32, buffer: &RawBufferSlice);
    unsafe fn bind_sampler(&self, index: u32, sampler: GLuint);
    unsafe fn bind_vertex_buffer_unchecked(&self, slot: u32, buffer: &RawBufferSlice, stride: usize, layout: Option<VertexLayout>);
    unsafe fn bind_index_buffer_unchecked(&self, buffer: &RawBufferSlice, index_type: Option<Type>);
    unsafe fn bind_texture_unchecked(&self, slot: u32, texture: &RawTexture, sampler: &Sampler);
    //unsafe fn bind_uniform_buffers_unchecked(&self, start_slot: i32, )
}

pub struct DefaultUniformBinder;

unsafe impl UniformBinder for DefaultUniformBinder
{
    unsafe fn bind_uniform_f32_unchecked(&self, location: u32, v: f32) {
        gl::Uniform1f(location as i32, v);
    }

    unsafe fn bind_uniform_vec2_unchecked(&self, location: u32, v: [f32; 2]) {
        gl::Uniform2f(location as i32, v[0], v[1]);
    }

    unsafe fn bind_uniform_vec3_unchecked(&self, location: u32, v: [f32; 3]) {
        gl::Uniform3f(location as i32, v[0], v[1], v[2]);
    }

    unsafe fn bind_uniform_vec4_unchecked(&self, location: u32, v: [f32; 4]) {
        unimplemented!()
    }

    unsafe fn bind_uniform_i32_unchecked(&self, location: u32, v: i32) {
        unimplemented!()
    }

    unsafe fn bind_uniform_ivec2_unchecked(&self, location: u32, v: [i32; 2]) {
        unimplemented!()
    }

    unsafe fn bind_uniform_ivec3_unchecked(&self, location: u32, v: [i32; 3]) {
        unimplemented!()
    }

    unsafe fn bind_uniform_ivec4_unchecked(&self, location: u32, v: [i32; 4]) {
        unimplemented!()
    }

    unsafe fn bind_uniform_mat2_unchecked(&self, location: u32, v: [f32; 2 * 2]) {
        unimplemented!()
    }

    unsafe fn bind_uniform_mat3_unchecked(&self, location: u32, v: [f32; 3 * 3]) {
        unimplemented!()
    }

    unsafe fn bind_uniform_mat4_unchecked(&self, location: u32, v: [f32; 4 * 4]) {
        unimplemented!()
    }

    unsafe fn bind_uniform_buffer_unchecked(&self, slot: u32, buffer: &RawBufferSlice) {
        gl::BindBufferRange(
            gl::UNIFORM_BUFFER,
            slot,
            buffer.owner.gl_object(),
            buffer.offset as isize,
            buffer.byte_size as isize
        );
    }

    unsafe fn bind_shader_storage_buffer_unchecked(&self, slot: u32, buffer: &RawBufferSlice) {
        gl::BindBufferRange(
            gl::SHADER_STORAGE_BUFFER,
            slot,
            buffer.owner.gl_object(),
            buffer.offset as isize,
            buffer.byte_size as isize
        );
    }

    unsafe fn bind_sampler(&self, index: u32, sampler: GLuint) {
        unimplemented!()
    }

    unsafe fn bind_vertex_buffer_unchecked(&self, slot: u32, buffer: &RawBufferSlice, stride: usize, layout: Option<VertexLayout>) {
        gl::BindVertexBuffer(
            slot,
            buffer.owner.gl_object(),
            buffer.offset as isize,
            stride as i32
        );
    }

    unsafe fn bind_index_buffer_unchecked(&self, buffer: &RawBufferSlice, index_type: Option<Type>) {
        gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, buffer.owner.gl_object());
    }

    unsafe fn bind_texture_unchecked(&self, slot: u32, texture: &RawTexture, sampler: &Sampler)
    {
        gl::BindTextureUnit(slot, texture.gl_object());
        gl::BindSampler(slot, sampler.obj);
    }
}

/// A trait representing a collection of shaders (of the same type)
/// for the whole graphics pipeline.
/// Provides methods for binding data to the OpenGL pipeline.
///
/// Potential implementations: GlslShaderPipeline, GlslBinaryPipeline, SpirvBinaryPipeline, etc.
pub trait GraphicsShaderPipeline
{
    fn vertex_shader(&self) -> &VertexShader;
    fn fragment_shader(&self) -> &FragmentShader;
    fn geometry_shader(&self) -> Option<&GeometryShader>;
    fn tess_control_shader(&self) -> Option<&TessControlShader>;
    fn tess_eval_shader(&self) -> Option<&TessEvalShader>;
    fn is_compatible_with(&self, interface: &ShaderInterfaceDesc) -> bool;
    fn get_program(&self) -> Result<GLuint, Error>;
    /// Must not allocate
    unsafe fn bind(&self) -> &UniformBinder;
}

pub trait ComputeShaderPipeline
{
    fn compute_shader(&self) -> &ComputeShader;
    fn is_compatible_with(&self, interface: &ShaderInterfaceDesc) -> bool;
    fn get_program(&self) -> Result<GLuint, Error>;
    unsafe fn bind(&self) -> &UniformBinder;
}
