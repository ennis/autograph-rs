use gl;
use gl::types::*;
use gfx::RawTexture;
use gfx::Framebuffer;
use gfx::Frame;
use gfx::state_cache::StateCache;
use gfx::ToRawBufferSlice;
use gfx::pipeline::GraphicsPipeline;
use gfx::bind::{VertexInput, Uniforms, Scissors};
use gfx::bind::{bind_target, bind_vertex_input, bind_uniforms, bind_graphics_pipeline, bind_scissors, SG_ALL};
use gfx::buffer_data::BufferData;
use gfx::{BufferSlice, RawBufferSlice, SamplerDesc};
use gfx::shader::UniformBinder;

use std::marker::PhantomData;
use std::mem;

pub enum DrawCmd
{
    DrawArrays { first: usize, count: usize },
    DrawIndexed { first: usize, count: usize, base_vertex: usize }
}

pub trait DrawExt<'queue>
{
    fn clear_texture(
        &self,
        texture: &RawTexture,
        mip_level: usize,
        clear_color: &[f32; 4],
    ) -> &Self;

    fn clear_texture_integer(
        &self,
        texture: &RawTexture,
        mip_level: usize,
        clear_color: &[i32; 4],
    ) -> &Self;

    fn clear_depth_texture(
        &self,
        texture: &RawTexture,
        mip_level: usize,
        clear_depth: f32,
    ) -> &Self;

    fn clear_framebuffer_color(
        &self,
        framebuffer: &Framebuffer,
        drawbuffer: usize,
        clear_color: &[f32; 4],
    ) -> &Self;

    fn clear_framebuffer_depth(
        &self,
        framebuffer: &Framebuffer,
        clear_depth: f32
    ) -> &Self;

    /// Begins building a draw command.
    /// This function does not perform any type checking.
    fn begin_draw<'frame>(&'frame self, target: &Framebuffer, pipeline: &GraphicsPipeline) -> DrawCommandBuilder<'frame,'queue> where 'queue:'frame;
    /// V2 API
    fn draw<'frame, 'pipeline>(&'frame self, target: &Framebuffer, pipeline: &'pipeline GraphicsPipeline, cmd: DrawCmd) -> DrawCmdBuilder<'frame, 'queue, 'pipeline> where 'queue:'frame;
}

impl<'queue> DrawExt<'queue> for Frame<'queue>
{
    //====================== COMMANDS =======================
    fn clear_texture(
        &self,
        texture: &RawTexture,
        mip_level: usize,
        clear_color: &[f32; 4],
    ) -> &Self
    {
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
        texture: &RawTexture,
        mip_level: usize,
        clear_color: &[i32; 4],
    ) -> &Self
    {
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
        texture: &RawTexture,
        mip_level: usize,
        clear_depth: f32,
    ) -> &Self
    {
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
    ) -> &Self
    {
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

    fn clear_framebuffer_depth(
        &self,
        framebuffer: &Framebuffer,
        clear_depth: f32
    ) -> &Self
    {
        unsafe {
            gl::ClearNamedFramebufferfv(framebuffer.gl_object(), gl::DEPTH, 0, &clear_depth as *const f32);
        }
        self
    }

    /// Begin building a draw command.
    /// This function does not perform any type checking.
    fn begin_draw<'frame>(&'frame self, target: &Framebuffer, pipeline: &GraphicsPipeline) -> DrawCommandBuilder<'frame,'queue> where 'queue:'frame
    {
        DrawCommandBuilder::new(self, target, pipeline)
    }

    /// V2 API
    fn draw<'frame, 'pipeline>(&'frame self, target: &Framebuffer, pipeline: &'pipeline GraphicsPipeline, cmd: DrawCmd) -> DrawCmdBuilder<'frame, 'queue, 'pipeline> where 'queue:'frame
    {
        let binder = unsafe {
            let mut state_cache = self.state_cache.borrow_mut();
            pipeline.bind(&mut state_cache)
        };
        let fb_size = target.size();
        let viewports = [(0f32, 0f32, fb_size.0 as f32, fb_size.1 as f32); 8];
        unsafe {
            self.state_cache.borrow_mut().set_target(target, &viewports);
        }

        DrawCmdBuilder {
            frame: self,
            uniform_binder: unsafe { binder },
            cmd,
            pipeline: &pipeline,
            index_buffer_offset: None,
            index_stride: None,
            index_buffer_type: None
        }
    }
}

/// Draw command builder.
/// Statically locks the frame object: allocate your buffers before starting a command!
pub struct DrawCommandBuilder<'frame,'queue:'frame> {
    frame: &'frame Frame<'queue>,
    uniforms: Uniforms,        // holds arrays of uniforms
    vertex_input: VertexInput, // vertex buffers + index buffer (optional)
    framebuffer: Framebuffer,
    pipeline: GraphicsPipeline,
    scissors: Scissors,
    viewports: [(f32, f32, f32, f32); 8]
}

impl<'frame,'queue:'frame> DrawCommandBuilder<'frame,'queue>
{
    fn new(frame: &'frame Frame<'queue>,
                  target: &Framebuffer,
                  pipeline: &GraphicsPipeline,
    ) -> DrawCommandBuilder<'frame,'queue>
    {
        let fb_size = target.size();
        DrawCommandBuilder {
            frame,
            uniforms: Default::default(),
            vertex_input: Default::default(),
            pipeline: pipeline.clone(),
            framebuffer: target.clone(),
            scissors: Scissors::All(None),
            viewports: [(0f32, 0f32, fb_size.0 as f32, fb_size.1 as f32); 8]
        }
    }


    //======================= BIND COMMANDS ============================
    // TODO struct type check?
    pub fn with_storage_buffer<S: ToRawBufferSlice>(
        mut self,
        slot: usize,
        buffer: &S,
    ) -> Self {
        let buffer = unsafe {
            buffer.to_raw_slice()
        };
        // reference this buffer in the frame
        self.frame
            .ref_buffers
            .borrow_mut()
            .push(buffer.owner.clone());
        self.uniforms.shader_storage_buffers[slot] = buffer.owner.gl_object();
        self.uniforms.shader_storage_buffer_offsets[slot] = buffer.offset as GLintptr;
        self.uniforms.shader_storage_buffer_sizes[slot] = buffer.byte_size as GLsizeiptr;
        self
    }

    pub fn with_uniform_buffer<U: ToRawBufferSlice>(
        mut self,
        slot: usize,
        buffer: &U,
    ) -> Self {
        let buffer = unsafe {
            buffer.to_raw_slice()
        };
        self.frame
            .ref_buffers
            .borrow_mut()
            .push(buffer.owner.clone());
        self.uniforms.uniform_buffers[slot] = buffer.owner.gl_object();
        self.uniforms.uniform_buffer_offsets[slot] = buffer.offset as GLintptr;
        self.uniforms.uniform_buffer_sizes[slot] = buffer.byte_size as GLsizeiptr;
        self
    }

    pub fn with_image(mut self, slot: usize, tex: &RawTexture) -> Self {
        self.uniforms.images[slot] = tex.gl_object();
        self
    }

    pub fn with_all_viewports(mut self, _v: (f32, f32, f32, f32)) -> Self {
        unimplemented!()
    }

    pub fn with_viewport(mut self, _index: i32, _v: (f32, f32, f32, f32)) -> Self {
        unimplemented!()
    }

    pub fn with_texture(mut self, slot: usize, tex: &RawTexture, sampler: &SamplerDesc) -> Self {
        {
            let gctx = self.frame.queue().context();
            self.uniforms.textures[slot] = tex.gl_object();
            // sampler objects are never deleted, and the context still lives
            // while the frame is still in flight
            self.uniforms.samplers[slot] = gctx.get_sampler(sampler).obj;
        }
        self
    }

    pub fn with_vertex_buffer<V: ToRawBufferSlice>(
        mut self,
        slot: usize,
        vertices: &V,
    ) -> Self {
        // TODO layout check w.r.t pipeline
        // TODO alignment check
        let vertices = unsafe {
            vertices.to_raw_slice()
        };
        self.frame
            .ref_buffers
            .borrow_mut()
            .push(vertices.owner.clone());
        self.vertex_input.vertex_buffers[slot] = vertices.owner.gl_object();
        self.vertex_input.vertex_buffer_offsets[slot] = vertices.offset as GLintptr;
        self.vertex_input.vertex_buffer_strides[slot] = mem::size_of::<<<V as ToRawBufferSlice>::Target as BufferData>::Element>() as GLsizei;
        self
    }

    pub fn with_index_buffer<I: ToRawBufferSlice>(mut self, indices: &I) -> Self {
        let indices = unsafe {
            indices.to_raw_slice()
        };
        self.frame
            .ref_buffers
            .borrow_mut()
            .push(indices.owner.clone());
        self.vertex_input.index_buffer = indices.owner.gl_object();
        self.vertex_input.index_buffer_size = indices.byte_size;
        self.vertex_input.index_buffer_offset = indices.offset;
        self.vertex_input.index_buffer_type = match mem::size_of::<<<I as ToRawBufferSlice>::Target as BufferData>::Element>() {
            4 => gl::UNSIGNED_INT,
            2 => gl::UNSIGNED_SHORT,
            // TODO We can verify that at compile-time
            _ => panic!("size of index element type does not match any supported formats"),
        };
        self
    }

    pub fn with_all_scissors(mut self, scissor: Option<(i32, i32, i32, i32)>) -> Self {
        self.scissors = Scissors::All(scissor);
        self
    }

    unsafe fn bind_all(&mut self) {
        let state_cache = &mut self.frame.state_cache.borrow_mut();
        state_cache.set_graphics_pipeline(&self.pipeline);
        state_cache.set_uniforms(&self.uniforms);
        state_cache.set_vertex_input(&self.vertex_input);
        state_cache.set_target(&self.framebuffer, &self.viewports);
    }

    //======================= DRAW COMMANDS ============================
    pub fn draw_arrays(mut self,
                       first: usize,
                       count: usize) -> &'frame Frame<'queue> {
        unsafe {
            self.bind_all();
            gl::DrawArrays(
                self.pipeline.primitive_topology,
                first as i32,
                count as i32,
            );
        }
        self.frame
    }

    pub fn draw_indexed(mut self,
                        first: usize,
                        count: usize,
                        base_vertex: usize
    ) -> &'frame Frame<'queue>
    {
        let index_stride = match self.vertex_input.index_buffer_type {
            gl::UNSIGNED_INT => 4,
            gl::UNSIGNED_SHORT => 2,
            _ => panic!("Unexpected index type"),
        };
        unsafe {
            self.bind_all();
            gl::DrawElementsBaseVertex(
                self.pipeline.primitive_topology,
                count as i32,
                self.vertex_input.index_buffer_type,
                (self.vertex_input.index_buffer_offset + first * index_stride) as *const GLvoid,
                base_vertex as i32,
            );
        }
        self.frame
    }

    /// Draw a quad. This overrides any vertex buffer set on slot 0.
    pub fn draw_quad(mut self) -> &'frame Frame<'queue>
    {
        unimplemented!()
    }
}

/// Draw command builder.
/// Statically locks the frame object: allocate your buffers before starting a command!
pub struct DrawCmdBuilder<'frame,'queue:'frame,'binder> {
    frame: &'frame Frame<'queue>,
    pipeline: &'binder GraphicsPipeline,
    uniform_binder: &'binder UniformBinder,
    index_buffer_type: Option<GLenum>,
    index_buffer_offset: Option<usize>,
    index_stride: Option<usize>,
    cmd: DrawCmd,
}

impl<'frame,'queue:'frame,'binder> DrawCmdBuilder<'frame,'queue,'binder>
{
    pub fn with_uniform_buffer<U: ToRawBufferSlice>(
        mut self,
        slot: u32,
        buffer: &U,
    ) -> Self {
        let buffer = unsafe { buffer.to_raw_slice() };
        self.frame.ref_buffers.borrow_mut().push(buffer.owner.clone());
        unsafe {
            self.uniform_binder.bind_uniform_buffer_unchecked(slot, &buffer);
        }
        self
    }

    pub fn with_texture(mut self, slot: u32, tex: &RawTexture, sampler: &SamplerDesc) -> Self {
        {
            let gctx = self.frame.queue().context();
            unsafe {
                self.uniform_binder.bind_texture_unchecked(slot, tex, &gctx.get_sampler(sampler));
            }
        }
        self
    }

    pub fn with_vertex_buffer<V: ToRawBufferSlice>(mut self, slot: u32, vertices: &V) -> Self {
        let vertices = unsafe { vertices.to_raw_slice() };
        self.frame.ref_buffers.borrow_mut().push(vertices.owner.clone());
        let stride = mem::size_of::<<<V as ToRawBufferSlice>::Target as BufferData>::Element>();
        unsafe {
            self.uniform_binder.bind_vertex_buffer_unchecked(slot, &vertices, stride, None);
        }
        self
    }

    pub fn with_index_buffer<I: ToRawBufferSlice>(mut self, indices: &I) -> Self {
        let indices = unsafe { indices.to_raw_slice() };
        self.frame.ref_buffers.borrow_mut().push(indices.owner.clone());
        let index_stride = mem::size_of::<<<I as ToRawBufferSlice>::Target as BufferData>::Element>();
        self.index_buffer_type = Some(match index_stride {
            4 => gl::UNSIGNED_INT,
            2 => gl::UNSIGNED_SHORT,
            // TODO We can verify that at compile-time
            _ => panic!("size of index element type does not match any supported formats"),
        });
        self.index_buffer_offset = Some(indices.offset);
        self.index_stride = Some(index_stride);
        unsafe {
            self.uniform_binder.bind_index_buffer_unchecked(&indices, None);
        }
        self
    }
}

/// Submit on drop
impl<'frame,'queue:'frame,'binder> Drop for DrawCmdBuilder<'frame,'queue,'binder>
{
    fn drop(&mut self) {
        match self.cmd {
            DrawCmd::DrawArrays { first, count } => {
                unsafe {
                    gl::DrawArrays(
                        self.pipeline.primitive_topology,
                        first as i32,
                        count as i32,
                    );
                }
            },
            DrawCmd::DrawIndexed { first, count, base_vertex } => {
                unsafe {
                    gl::DrawElementsBaseVertex(
                        self.pipeline.primitive_topology,
                        count as i32,
                        self.index_buffer_type.unwrap(),
                        (self.index_buffer_offset.unwrap() + first * self.index_stride.unwrap()) as *const GLvoid,
                        base_vertex as i32,
                    );
                }
            }
        }
    }
}
