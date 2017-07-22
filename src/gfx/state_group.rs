use bitflags;
use super::context::Context;
use super::buffer::RawBufferSlice;
use gl::types::*;
use gl;
use std::mem;

bitflags! {
    #[derive(Default)]
    pub struct StateGroupMask: u32 {
        ///
        const SG_VIEWPORTS = (1 << 0); // DONE
        const SG_FRAMEBUFFER = (1 << 1); // DONE
        const SG_SCISSOR_RECT = (1 << 2);
        const SG_BLEND_STATE = (1 << 3); // DONE
        const SG_RASTERIZER_STATE = (1 << 4); // DONE
        const SG_DEPTH_STENCIL_STATE = (1 << 5); // DONE
        const SG_TEXTURES = (1 << 6); // DONE
        const SG_SAMPLERS = (1 << 7);
        const SG_UNIFORM_BUFFERS = (1 << 8); // DONE
        const SG_SHADER_STORAGE_BUFFERS = (1 << 9); // DONE
        const SG_VERTEX_ARRAY = (1 << 10); // DONE
        const SG_PROGRAM = (1 << 11); // DONE
        const SG_VERTEX_BUFFERS = (1 << 12); // DONE
        const SG_INDEX_BUFFER = (1 << 13); // DONE
        const SG_IMAGE = (1 << 14); // DONE
        const SG_ALL_COMPUTE = SG_IMAGE.bits | SG_TEXTURES.bits | SG_SAMPLERS.bits | SG_PROGRAM.bits | SG_UNIFORM_BUFFERS.bits | SG_SHADER_STORAGE_BUFFERS.bits;
        const SG_ALL = 0xFFFFFFF;
    }
}

#[derive(Copy,Clone,Debug,Hash,Eq,PartialEq)]
pub struct BlendState
{
    enabled: bool,
    mode_rgb: GLenum,
    mode_alpha: GLenum,
    func_src_rgb: GLenum,
    func_dst_rgb: GLenum,
    func_src_alpha: GLenum,
    func_dst_alpha: GLenum
}

impl Default for BlendState
{
    fn default() -> BlendState
    {
        BlendState {
            enabled: false,
            mode_rgb: 0,
            mode_alpha: 0,
            func_src_rgb: 0,
            func_dst_rgb: 0,
            func_src_alpha: 0,
            func_dst_alpha: 0
        }
    }
}

impl BlendState
{
    fn alpha_blending() -> BlendState {
        BlendState {
            enabled: true,
            mode_rgb: gl::FUNC_ADD,
            mode_alpha: gl::FUNC_ADD,
            func_src_rgb: gl::SRC_ALPHA,
            func_dst_rgb: gl::ONE_MINUS_SRC_ALPHA,
            func_src_alpha: gl::ONE,
            func_dst_alpha: gl::ZERO
        }
    }
}

#[derive(Copy,Clone,Debug,Hash,Eq,PartialEq)]
pub struct DepthStencilState
{
    depth_test_enable: bool,
    depth_write_enable: bool,
    stencil_enable: bool,
    depth_test_func: GLenum,
    stencil_face: GLenum,
    stencil_func: GLenum,
    stencil_ref: i32,
    stencil_mask: u32,
    stencil_op_s_fail: GLenum,
    stencil_op_dp_fail: GLenum,
    stencil_op_dp_pass: GLenum
}

impl Default for DepthStencilState
{
    fn default() -> DepthStencilState
    {
        DepthStencilState {
            depth_test_enable: false,
            depth_write_enable: false,
            stencil_enable: false,
            depth_test_func: gl::LEQUAL,
            stencil_face: gl::FRONT_AND_BACK,
            stencil_func: 0,
            stencil_ref: 0,
            stencil_mask: 0xFFFFFFFF,
            stencil_op_s_fail: 0,
            stencil_op_dp_fail: 0,
            stencil_op_dp_pass: 0
        }
    }
}

#[derive(Copy,Clone,Debug,PartialEq)]
pub struct RasterizerState
{
    fill_mode: GLenum,
    cull_mode: GLenum,
    front_face: GLenum,
    depth_bias: f32,
    slope_scaled_depth_bias: f32,
    depth_clip_enable: bool,
    scissor_enable: bool
}

impl Default for RasterizerState
{
    fn default() -> RasterizerState
    {
        RasterizerState {
            fill_mode: gl::FILL,
            cull_mode: gl::NONE,
            front_face: gl::CCW,
            depth_bias: 1.0f32,
            slope_scaled_depth_bias: 1.0f32,
            depth_clip_enable: false,
            scissor_enable: false
        }
    }
}

const MAX_TEXTURE_UNITS: usize = 16;
const MAX_IMAGE_UNITS: usize = 8;
const MAX_VERTEX_BUFFER_SLOTS: usize = 8;
const MAX_UNIFORM_BUFFER_SLOTS: usize = 8;
const MAX_SHADER_STORAGE_BUFFER_SLOTS: usize = 8;

///
/// Never use that directly since it does not hold references
#[derive(Copy,Clone,Debug,PartialEq,Default)]
pub struct Uniforms
{
    textures: [GLuint; MAX_TEXTURE_UNITS],
    samplers: [GLuint; MAX_TEXTURE_UNITS],
    images: [GLuint; MAX_IMAGE_UNITS],
    uniform_buffers: [GLuint; MAX_UNIFORM_BUFFER_SLOTS],
    uniform_buffer_sizes: [GLsizeiptr; MAX_UNIFORM_BUFFER_SLOTS],
    uniform_buffer_offsets: [GLintptr; MAX_UNIFORM_BUFFER_SLOTS],
    shader_storage_buffers: [GLuint; MAX_SHADER_STORAGE_BUFFER_SLOTS],
    shader_storage_buffer_sizes: [GLsizeiptr; MAX_SHADER_STORAGE_BUFFER_SLOTS],
    shader_storage_buffer_offsets: [GLintptr; MAX_SHADER_STORAGE_BUFFER_SLOTS],
    vertex_buffers: [GLuint; MAX_VERTEX_BUFFER_SLOTS],
    vertex_buffer_strides: [GLsizei; MAX_VERTEX_BUFFER_SLOTS],
    vertex_buffer_offsets: [GLintptr; MAX_VERTEX_BUFFER_SLOTS],
    index_buffer: GLuint,
    index_buffer_size: usize,
    index_buffer_type: GLenum,
}

#[derive(Copy,Clone,Debug,PartialEq,Default)]
pub struct StateGroup
{
    mask: StateGroupMask,
    depth_stencil_state: DepthStencilState,
    rasterizer_state: RasterizerState,
    blend_state: [BlendState; 8],
    scissors: [(i32,i32,i32,i32);8],
    viewports: [(f32,f32,f32,f32);8],
    vertex_array: GLuint,
    program: GLuint,
    uniforms: Uniforms,
    barrier_bits: GLbitfield
}

///
/// TODO: Should think about how to minimize state changes?
unsafe fn bind_state_group(sg: &StateGroup, ctx: &Context)
{
    if sg.mask.contains(SG_VIEWPORTS)
    {
        // TODO: maybe something a bit less drastic than transmute could be possible?
        gl::ViewportArrayv(0, sg.viewports.len() as i32, mem::transmute(&sg.viewports));
    }

    if sg.mask.contains(SG_SCISSOR_RECT) {
        gl::ScissorArrayv(0, sg.scissors.len() as i32, mem::transmute(&sg.scissors));
    }

    if sg.mask.contains(SG_BLEND_STATE) {
        gl::Enable(gl::BLEND); // XXX is this necessary
        for (i,bs) in sg.blend_state.iter().enumerate()
        {
            if bs.enabled
            {
                gl::Enablei(gl::BLEND, i as u32);
                gl::BlendEquationSeparatei(i as u32, bs.mode_rgb, bs.mode_alpha);
                gl::BlendFuncSeparatei(i as u32, bs.func_src_rgb, bs.func_dst_rgb, bs.func_src_alpha, bs.func_dst_alpha);
            }
            else {
                gl::Disablei(gl::BLEND, i as u32);
            }
        }
    }

    if sg.mask.contains(SG_DEPTH_STENCIL_STATE) {
        if sg.depth_stencil_state.depth_test_enable {
            gl::Enable(gl::DEPTH_TEST);
        } else {
            gl::Disable(gl::DEPTH_TEST);
        }

        if sg.depth_stencil_state.depth_write_enable {
            gl::DepthMask(gl::TRUE);
        } else {
            gl::DepthMask(gl::FALSE);
        }

        gl::DepthFunc(sg.depth_stencil_state.depth_test_func);

        if sg.depth_stencil_state.stencil_enable {
            unimplemented!()
        }
        else {
            gl::Disable(gl::STENCIL_TEST);
        }
    }

    if sg.mask.contains(SG_RASTERIZER_STATE) {
        gl::PolygonMode(gl::FRONT_AND_BACK, sg.rasterizer_state.fill_mode);
        gl::Disable(gl::CULL_FACE);
    }

    if sg.mask.contains(SG_VERTEX_ARRAY) {
        gl::BindVertexArray(sg.vertex_array);
    }

    if sg.mask.contains(SG_PROGRAM) {
        gl::UseProgram(sg.program);
    }

    bind_uniforms(ctx, &sg.uniforms);
}

unsafe fn bind_uniforms(ctx: &Context, uniforms: &Uniforms)
{
    for i in 0..uniforms.vertex_buffers.len() {
        if uniforms.vertex_buffers[i] != 0 {
            gl::BindVertexBuffer(i as u32, uniforms.vertex_buffers[i], uniforms.vertex_buffer_offsets[i], uniforms.vertex_buffer_strides[i]);
        } else {
            gl::BindVertexBuffer(i as u32, 0, 0, 0);
        }
    }

    if uniforms.index_buffer != 0 {
        gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, uniforms.index_buffer);
    }

    // Textures
    gl::BindTextures(0, MAX_TEXTURE_UNITS as i32, uniforms.textures.as_ptr());
    // Samplers
    gl::BindSamplers(0, MAX_TEXTURE_UNITS as i32, uniforms.samplers.as_ptr());
    // Images
    gl::BindImageTextures(0, MAX_IMAGE_UNITS as i32, uniforms.images.as_ptr());

    // UBOs
    for i in 0..MAX_UNIFORM_BUFFER_SLOTS {
        if uniforms.uniform_buffers[i] != 0 {
            gl::BindBufferRange(gl::UNIFORM_BUFFER, i as u32, uniforms.uniform_buffers[i], uniforms.uniform_buffer_offsets[i], uniforms.uniform_buffer_sizes[i]);
        } else {
            gl::BindBufferBase(gl::UNIFORM_BUFFER, i as u32, 0);
        }
    }

    // SSBOs
    for i in 0..MAX_SHADER_STORAGE_BUFFER_SLOTS {
        if uniforms.shader_storage_buffers[i] != 0 {
            gl::BindBufferRange(gl::SHADER_STORAGE_BUFFER, i as u32, uniforms.shader_storage_buffers[i], uniforms.shader_storage_buffer_offsets[i], uniforms.shader_storage_buffer_sizes[i]);
        } else {
            gl::BindBufferBase(gl::SHADER_STORAGE_BUFFER, i as u32, 0);
        }
    }
}
