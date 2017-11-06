use gl;
use gl::types::*;
use super::pipeline::GraphicsPipeline;
use super::context::Context;
use super::buffer_data::BufferData;
use super::framebuffer::Framebuffer;

const MAX_TEXTURE_UNITS: usize = 16;
const MAX_IMAGE_UNITS: usize = 8;
const MAX_VERTEX_BUFFER_SLOTS: usize = 8;
const MAX_UNIFORM_BUFFER_SLOTS: usize = 8;
const MAX_SHADER_STORAGE_BUFFER_SLOTS: usize = 8;

#[derive(Copy, Clone, Debug, PartialEq, Default)]
pub(super) struct Uniforms {
    pub(super) textures: [GLuint; MAX_TEXTURE_UNITS],
    pub(super) samplers: [GLuint; MAX_TEXTURE_UNITS],
    pub(super) images: [GLuint; MAX_IMAGE_UNITS],
    pub(super) uniform_buffers: [GLuint; MAX_UNIFORM_BUFFER_SLOTS],
    pub(super) uniform_buffer_sizes: [GLsizeiptr; MAX_UNIFORM_BUFFER_SLOTS],
    pub(super) uniform_buffer_offsets: [GLintptr; MAX_UNIFORM_BUFFER_SLOTS],
    pub(super) shader_storage_buffers: [GLuint; MAX_SHADER_STORAGE_BUFFER_SLOTS],
    pub(super) shader_storage_buffer_sizes: [GLsizeiptr; MAX_SHADER_STORAGE_BUFFER_SLOTS],
    pub(super) shader_storage_buffer_offsets: [GLintptr; MAX_SHADER_STORAGE_BUFFER_SLOTS],
}

#[derive(Copy, Clone, Debug, PartialEq, Default)]
pub(super) struct VertexInput {
    pub(super) vertex_buffers: [GLuint; MAX_VERTEX_BUFFER_SLOTS],
    pub(super) vertex_buffer_strides: [GLsizei; MAX_VERTEX_BUFFER_SLOTS],
    pub(super) vertex_buffer_offsets: [GLintptr; MAX_VERTEX_BUFFER_SLOTS],
    pub(super) index_buffer: GLuint,
    pub(super) index_buffer_offset: usize,
    pub(super) index_buffer_size: usize,
    pub(super) index_buffer_type: GLenum
}

// TODO is this useless?
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

pub(super) unsafe fn bind_graphics_pipeline(pipe: &GraphicsPipeline, mask: StateGroupMask) {
    if mask.contains(SG_BLEND_STATE) {
        gl::Enable(gl::BLEND); // XXX is this necessary
        for (i, bs) in pipe.blend_states.iter().enumerate() {
            if bs.enabled {
                gl::Enablei(gl::BLEND, i as u32);
                gl::BlendEquationSeparatei(i as u32, bs.mode_rgb, bs.mode_alpha);
                gl::BlendFuncSeparatei(
                    i as u32,
                    bs.func_src_rgb,
                    bs.func_dst_rgb,
                    bs.func_src_alpha,
                    bs.func_dst_alpha,
                );
            } else {
                gl::Disablei(gl::BLEND, i as u32);
            }
        }
    }

    if mask.contains(SG_DEPTH_STENCIL_STATE) {
        if pipe.depth_stencil_state.depth_test_enable {
            gl::Enable(gl::DEPTH_TEST);
        } else {
            gl::Disable(gl::DEPTH_TEST);
        }

        if pipe.depth_stencil_state.depth_write_enable {
            gl::DepthMask(gl::TRUE);
        } else {
            gl::DepthMask(gl::FALSE);
        }

        gl::DepthFunc(pipe.depth_stencil_state.depth_test_func);

        if pipe.depth_stencil_state.stencil_enable {
            unimplemented!("Stencil buffers")
        } else {
            gl::Disable(gl::STENCIL_TEST);
        }
    }

    if mask.contains(SG_RASTERIZER_STATE) {
        gl::PolygonMode(gl::FRONT_AND_BACK, pipe.rasterizer_state.fill_mode);
        gl::Disable(gl::CULL_FACE);
    }

    if mask.contains(SG_VERTEX_ARRAY) {
        gl::BindVertexArray(pipe.vao);
    }

    if mask.contains(SG_PROGRAM) {
        gl::UseProgram(pipe.program);
    }
}

pub(super) unsafe fn bind_uniforms(uniforms: &Uniforms) {
    // Textures
    gl::BindTextures(0, MAX_TEXTURE_UNITS as i32, uniforms.textures.as_ptr());
    // Samplers
    gl::BindSamplers(0, MAX_TEXTURE_UNITS as i32, uniforms.samplers.as_ptr());
    // Images
    gl::BindImageTextures(0, MAX_IMAGE_UNITS as i32, uniforms.images.as_ptr());

    // UBOs
    for i in 0..MAX_UNIFORM_BUFFER_SLOTS {
        if uniforms.uniform_buffers[i] != 0 {
            gl::BindBufferRange(
                gl::UNIFORM_BUFFER,
                i as u32,
                uniforms.uniform_buffers[i],
                uniforms.uniform_buffer_offsets[i],
                uniforms.uniform_buffer_sizes[i],
            );
        } else {
            gl::BindBufferBase(gl::UNIFORM_BUFFER, i as u32, 0);
        }
    }

    // SSBOs
    for i in 0..MAX_SHADER_STORAGE_BUFFER_SLOTS {
        if uniforms.shader_storage_buffers[i] != 0 {
            gl::BindBufferRange(
                gl::SHADER_STORAGE_BUFFER,
                i as u32,
                uniforms.shader_storage_buffers[i],
                uniforms.shader_storage_buffer_offsets[i],
                uniforms.shader_storage_buffer_sizes[i],
            );
        } else {
            gl::BindBufferBase(gl::SHADER_STORAGE_BUFFER, i as u32, 0);
        }
    }
}

pub(super) unsafe fn bind_vertex_input(vertex_input: &VertexInput) {
    for i in 0..vertex_input.vertex_buffers.len() {
        if vertex_input.vertex_buffers[i] != 0 {
            gl::BindVertexBuffer(
                i as u32,
                vertex_input.vertex_buffers[i],
                vertex_input.vertex_buffer_offsets[i],
                vertex_input.vertex_buffer_strides[i],
            );
        } else {
            gl::BindVertexBuffer(i as u32, 0, 0, 0);
        }
    }

    if vertex_input.index_buffer != 0 {
        gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, vertex_input.index_buffer);
    }
}

pub enum Scissors {
    All(Option<(i32, i32, i32, i32)>),
}

pub(super) unsafe fn bind_scissors(scissors: &Scissors) {
    match scissors {
        &Scissors::All(None) => gl::Disable(gl::SCISSOR_TEST),
        &Scissors::All(Some((x, y, w, h))) => {
            gl::Enable(gl::SCISSOR_TEST);
            gl::Scissor(x, y, w, h);
        }
    }
}

pub(super) unsafe fn bind_target(framebuffer: &Framebuffer, viewport: &[(f32, f32, f32, f32)]) {
    gl::BindFramebuffer(gl::DRAW_FRAMEBUFFER, framebuffer.obj);
    gl::ViewportArrayv(0, 8, viewport.as_ptr() as *const GLfloat);
}

