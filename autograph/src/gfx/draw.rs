use gfx::bind::StateCache;
use gfx::bind::{bind_graphics_pipeline, bind_scissors, bind_target, bind_uniforms,
                bind_vertex_input, SG_ALL};
use gfx::bind::{Scissors, Uniforms, VertexInput};
use gfx::buffer_data::BufferData;
use gfx::pipeline::{GraphicsPipeline, TypedGraphicsPipeline};
use gfx::shader_interface::ShaderInterface;
use gfx::Frame;
use gfx::Framebuffer;
use gfx::TextureAny;
use gfx::{ToBufferSlice, ToBufferSliceAny};
use gfx::{BufferSlice, BufferSliceAny, SamplerDesc};
use gl;
use gl::types::*;

use std::cell::RefMut;
use std::marker::PhantomData;
use std::mem;

pub enum DrawCmd {
    DrawArrays {
        first: usize,
        count: usize,
    },
    DrawIndexed {
        first: usize,
        count: usize,
        base_vertex: usize,
    },
}

pub trait DrawExt<'queue> {
    fn clear_texture(
        &self,
        texture: &TextureAny,
        mip_level: usize,
        clear_color: &[f32; 4],
    ) -> &Self;

    fn clear_texture_integer(
        &self,
        texture: &TextureAny,
        mip_level: usize,
        clear_color: &[i32; 4],
    ) -> &Self;

    fn clear_depth_texture(
        &self,
        texture: &TextureAny,
        mip_level: usize,
        clear_depth: f32,
    ) -> &Self;

    fn clear_framebuffer_color(
        &self,
        framebuffer: &Framebuffer,
        drawbuffer: usize,
        clear_color: &[f32; 4],
    ) -> &Self;

    fn clear_framebuffer_depth(&self, framebuffer: &Framebuffer, clear_depth: f32) -> &Self;

    /// Begins building a draw command.
    /// This function does not perform any type checking.
    ///fn begin_draw<'frame>(&'frame self, target: &Framebuffer, pipeline: &GraphicsPipeline) -> DrawCommandBuilder<'frame,'queue> where 'queue:'frame;
    /// V2 API
    fn draw<'frame, 'pipeline>(
        &'frame self,
        target: &Framebuffer,
        pipeline: &'pipeline GraphicsPipeline,
        cmd: DrawCmd,
    ) -> DrawCmdBuilder<'frame, 'queue, 'pipeline>
    where
        'queue: 'frame;

    fn draw2<'frame, 'pipeline, T: ShaderInterface>(
        &'frame self,
        target: &Framebuffer,
        pipeline: &'pipeline TypedGraphicsPipeline<T>,
        interface: &T,
        cmd: DrawCmd,
    ) -> DrawCmdBuilder<'frame, 'queue, 'pipeline>
        where
            'queue: 'frame;
}

impl<'queue> DrawExt<'queue> for Frame<'queue> {
    //====================== COMMANDS =======================
    fn clear_texture(
        &self,
        texture: &TextureAny,
        mip_level: usize,
        clear_color: &[f32; 4],
    ) -> &Self {
        unsafe {
            gl::ClearTexImage(
                texture.obj,
                mip_level as i32,
                gl::RGBA,
                gl::FLOAT,
                clear_color as *const _ as *const _,
            );
        }
        self
    }

    fn clear_texture_integer(
        &self,
        texture: &TextureAny,
        mip_level: usize,
        clear_color: &[i32; 4],
    ) -> &Self {
        unsafe {
            gl::ClearTexImage(
                texture.obj,
                mip_level as i32,
                gl::RGBA_INTEGER,
                gl::INT,
                clear_color as *const _ as *const _,
            );
        }
        self
    }

    fn clear_depth_texture(
        &self,
        texture: &TextureAny,
        mip_level: usize,
        clear_depth: f32,
    ) -> &Self {
        unsafe {
            gl::ClearTexImage(
                texture.obj,
                mip_level as i32,
                gl::DEPTH_COMPONENT,
                gl::FLOAT,
                &clear_depth as *const _ as *const _,
            );
        }
        self
    }

    fn clear_framebuffer_color(
        &self,
        framebuffer: &Framebuffer,
        drawbuffer: usize,
        clear_color: &[f32; 4],
    ) -> &Self {
        unsafe {
            gl::ClearNamedFramebufferfv(
                framebuffer.gl_object(),
                gl::COLOR,
                drawbuffer as i32,
                clear_color as *const _ as *const f32,
            );
        }
        self
    }

    fn clear_framebuffer_depth(&self, framebuffer: &Framebuffer, clear_depth: f32) -> &Self {
        unsafe {
            gl::ClearNamedFramebufferfv(
                framebuffer.gl_object(),
                gl::DEPTH,
                0,
                &clear_depth as *const f32,
            );
        }
        self
    }

    /// V2 API
    fn draw<'frame, 'pipeline>(
        &'frame self,
        target: &Framebuffer,
        pipeline: &'pipeline GraphicsPipeline,
        cmd: DrawCmd,
    ) -> DrawCmdBuilder<'frame, 'queue, 'pipeline>
    where
        'queue: 'frame,
    {
        let mut state_cache = self.state_cache.borrow_mut();
        let fb_size = target.size();
        let viewports = [(0f32, 0f32, fb_size.0 as f32, fb_size.1 as f32); 8];
        unsafe {
            state_cache.set_graphics_pipeline(pipeline);
            state_cache.set_target(target, &viewports);
        }

        DrawCmdBuilder {
            frame: self,
            state_cache,
            cmd,
            pipeline: &pipeline,
            index_buffer_offset: None,
            index_stride: None,
            index_buffer_type: None,
        }
    }

    /// V2 API
    fn draw2<'frame, 'pipeline, T: ShaderInterface>(
        &'frame self,
        target: &Framebuffer,
        pipeline: &'pipeline TypedGraphicsPipeline<T>,
        interface: &T,
        cmd: DrawCmd,
    ) -> DrawCmdBuilder<'frame, 'queue, 'pipeline>
        where
            'queue: 'frame,
    {
        unsafe {
            let mut state_cache = self.state_cache.borrow_mut();
            pipeline.binder.bind_unchecked(interface, self, &mut state_cache);
        }

        let mut state_cache = self.state_cache.borrow_mut();
        let fb_size = target.size();
        let viewports = [(0f32, 0f32, fb_size.0 as f32, fb_size.1 as f32); 8];
        unsafe {
            state_cache.set_graphics_pipeline(&pipeline.pipeline);
            state_cache.set_target(target, &viewports);
        }

        DrawCmdBuilder {
            frame: self,
            state_cache,
            cmd,
            pipeline: &pipeline.pipeline,
            index_buffer_offset: None,
            index_stride: None,
            index_buffer_type: None,
        }
    }
}

/// Draw command builder.
/// Statically locks the frame object: allocate your buffers before starting a command!
#[must_use]
pub struct DrawCmdBuilder<'frame, 'queue: 'frame, 'binder> {
    frame: &'frame Frame<'queue>,
    pipeline: &'binder GraphicsPipeline,
    state_cache: RefMut<'frame, StateCache>,
    index_buffer_type: Option<GLenum>,
    index_buffer_offset: Option<usize>,
    index_stride: Option<usize>,
    cmd: DrawCmd,
}


// Drop bomb
impl<'frame, 'queue: 'frame, 'binder> Drop for DrawCmdBuilder<'frame, 'queue, 'binder> {
    fn drop(&mut self) {
        panic!("unsubmitted draw command")
    }
}

impl<'frame, 'queue: 'frame, 'binder> DrawCmdBuilder<'frame, 'queue, 'binder> {
    /// Set a uniform buffer to use for this command.
    pub fn with_uniform_buffer<U: ToBufferSlice>(mut self, slot: u32, buffer: &U) -> Self {
        let buffer = unsafe { buffer.to_slice_any() };
        unsafe {
            self.state_cache.set_uniform_buffer(slot, &buffer);
        }
        self.frame
            .resource_tracker
            .borrow_mut()
            .ref_buffer(buffer.owner);
        self
    }

    pub fn with_texture(mut self, slot: u32, tex: &TextureAny, sampler: &SamplerDesc) -> Self {
            let gctx = self.frame.queue().context();
            unsafe {
                self.state_cache
                    .set_texture(slot, tex, &gctx.get_sampler(sampler));
            }

        self.frame
            .resource_tracker
            .borrow_mut()
            .ref_texture(tex.clone());
        self
    }

    pub fn with_vertex_buffer<V: ToBufferSlice>(mut self, slot: u32, vertices: &V) -> Self {
        let vertices = unsafe { vertices.to_slice_any() };
        let stride = mem::size_of::<<<V as ToBufferSlice>::Target as BufferData>::Element>();
        unsafe {
            self.state_cache.set_vertex_buffer(slot, &vertices, stride);
        }
        self.frame
            .resource_tracker
            .borrow_mut()
            .ref_buffer(vertices.owner);
        self
    }

    pub fn with_index_buffer<I: ToBufferSlice>(mut self, indices: &I) -> Self {
        let indices = unsafe { indices.to_slice_any() };
        let index_stride =
            mem::size_of::<<<I as ToBufferSlice>::Target as BufferData>::Element>();
        self.index_buffer_type = Some(match index_stride {
            4 => gl::UNSIGNED_INT,
            2 => gl::UNSIGNED_SHORT,
            // TODO We can verify that at compile-time
            _ => panic!("size of index element type does not match any supported formats"),
        });
        self.index_buffer_offset = Some(indices.offset);
        self.index_stride = Some(index_stride);
        unsafe {
            self.state_cache.set_index_buffer(&indices);
        }
        self.frame
            .resource_tracker
            .borrow_mut()
            .ref_buffer(indices.owner);
        self
    }

    pub fn submit(mut self)
    {
        unsafe {
            self.state_cache.commit();
        }
        match self.cmd {
            DrawCmd::DrawArrays { first, count } => unsafe {
                gl::DrawArrays(self.pipeline.primitive_topology, first as i32, count as i32);
            },
            DrawCmd::DrawIndexed {
                first,
                count,
                base_vertex,
            } => unsafe {
                gl::DrawElementsBaseVertex(
                    self.pipeline.primitive_topology,
                    count as i32,
                    self.index_buffer_type.unwrap(),
                    (self.index_buffer_offset.unwrap() + first * self.index_stride.unwrap())
                        as *const GLvoid,
                    base_vertex as i32,
                );
            },
        }

        // extract everything that needs to be dropped, and drop it there
        // then, forget (leak) the DrawCmdBuilder to prevent the destructor bomb from going off
        let mut state_cache = unsafe { mem::uninitialized() };
        let mut cmd = unsafe { mem::uninitialized() };
        mem::swap(&mut self.state_cache, &mut state_cache);
        mem::swap(&mut self.cmd, &mut cmd);
        drop(state_cache);
        drop(cmd);
        mem::forget(self);
    }
}

