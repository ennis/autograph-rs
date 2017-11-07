use super::queue::{Queue, FrameResources};
use super::buffer_data::BufferData;
use super::sampler::SamplerDesc;
use super::context::Context;
use super::texture::RawTexture;
use super::upload_buffer::UploadBuffer;
use super::buffer::{BufferSlice,RawBuffer,RawBufferSlice};
use super::framebuffer::{Framebuffer, FramebufferObject};
use super::bind::{VertexInput, Uniforms, Scissors};
use super::pipeline::GraphicsPipeline;
use super::bind::{bind_target, bind_vertex_input, bind_uniforms, bind_graphics_pipeline, bind_scissors, SG_ALL};
use super::fence::FenceValue;

use std::marker::PhantomData;
use std::cell::RefCell;
use std::sync::Arc;
use std::mem;

use gl;
use gl::types::*;

/// A slice of a buffer that cannot be used outside the frame it has been allocated in.
/// This is statically prevented by the lifetime bound.
pub struct TransientBufferSlice<'a, T>
    where
        T: BufferData + ?Sized,
{
    // don't make this public: the user should not be able to extend the lifetime of slice
    slice: BufferSlice<T>,
    _phantom: PhantomData<&'a T>,
}

/// Trait that provides a method for getting a strong reference to the underlying resource
/// of a slice.
/// The method is unsafe because it allows extending resources beyond their static lifetime
/// bounds, and should only be used internally by `Frame`, which does its own synchronization.
/// Not meant to be implemented in user code.
/// Can't use deref, because an user could then accidentally extend the lifetime of
/// a TransientBufferSlice outside the frame.
pub unsafe trait ToRawBufferSlice {
    type Target: BufferData + ?Sized;
    unsafe fn to_raw_slice(&self) -> RawBufferSlice;
}

unsafe impl<'a,T> ToRawBufferSlice for TransientBufferSlice<'a,T> where T: BufferData + ?Sized
{
    type Target = T;
    unsafe fn to_raw_slice(&self) -> RawBufferSlice {
        self.slice.to_raw_slice()
    }
}

unsafe impl<T> ToRawBufferSlice for BufferSlice<T> where T: BufferData + ?Sized
{
    type Target = T;
    unsafe fn to_raw_slice(&self) -> RawBufferSlice {
        let clone = self.clone();
        clone.into_raw()
    }
}

/// A frame: instances are alive until the frame is complete
pub struct Frame<'q> {
    // Associated queue
    queue: &'q mut Queue,
    // Built-in upload buffer for convenience
    // Resources held onto by this frame
    pub(super) ref_buffers: RefCell<Vec<RawBuffer>>,
    pub(super) ref_textures: RefCell<Vec<RawTexture>>,
    state_cache: RefCell<StateCache>,
}


impl<'q> Frame<'q> {
    /// Creates a new frame, mut-borrows the queue
    /// Since we can't build multiple command streams in parallel in OpenGL
    pub fn new<'a>(queue: &'a mut Queue) -> Frame<'a>
    {
        Frame {
            queue,
            //upload_buffer: UploadBuffer::new(queue.context(), DEFAULT_UPLOAD_BUFFER_SIZE),
            ref_textures: RefCell::new(Vec::new()),
            ref_buffers: RefCell::new(Vec::new()),
            state_cache: RefCell::new(StateCache::new())
        }
    }

    /// Allocates and uploads data to a *transient buffer* that live only until the GPU has finished using them.
    /// Can be used for uniform buffers, shader storage buffers, vertex buffers, etc.
    /// TODO specify target usage
    /// The lifetime of the returned resource is bound to the lifetime of self:
    /// this allows to statically limit the usage of the buffer to the current frame only.
    pub fn upload_into<'a, T: BufferData + ?Sized>(&'a self, upload_buffer: &'a UploadBuffer, data: &T) -> TransientBufferSlice<'a, T>
    {
        TransientBufferSlice {
            slice: unsafe {
                // TODO infer alignment from usage
                upload_buffer.upload(data, 256, self.queue.next_frame_fence_value(), self.queue.last_completed_frame())
            },
            _phantom: PhantomData
        }
    }


    /// Allocates and uploads data to the default upload buffer of the queue.
    /// Effectively calls `upload_into` with `self.queue().default_upload_buffer()`
    pub fn upload<'a, T: BufferData + ?Sized>(&'a self, data: &T) -> TransientBufferSlice<'a, T> {
        self.upload_into(self.queue.default_upload_buffer(), data)
    }

    /// Returns the current value of the fence of the queue.
    pub fn fence_value(&self) -> FenceValue {
        self.queue.fence.borrow().next_value()
    }

    /// Returns the queue of this frame.
    /// Note that you can't do any operations on the frame while the returned borrow
    /// still lives
    pub fn queue<'a>(&'a self) -> &'a Queue {
        self.queue
    }

    /// Consumes self, also releases the borrow on queue
    /// This function hands off all resources referenced during command stream construction
    /// to the queue, which will drop the references to the resources
    /// as soon as they are no longer in use by the GPU
    pub fn submit(self) {
        //debug!("Submit frame: sync index={:?}", self.queue.fence.borrow().next_value());
        // setup fence in command stream
        self.queue.submit(FrameResources {
            ref_buffers: self.ref_buffers.into_inner(),
            ref_textures: self.ref_textures.into_inner(),
        });
    }

    //====================== COMMANDS =======================
    pub fn clear_texture(
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

    pub fn clear_texture_integer(
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

    pub fn clear_depth_texture(
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

    pub fn clear_framebuffer_color(
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

    pub fn clear_framebuffer_depth(
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
    pub fn begin_draw<'a>(&'a self, target: &Framebuffer, pipeline: &GraphicsPipeline) -> DrawCommandBuilder<'a,'q>
    where 'q:'a
    {
        DrawCommandBuilder::new(self, target, pipeline)
    }
}

// TODO move this into its own module
struct StateCache
{
    uniforms: Option<Uniforms>,
    vertex_input: Option<VertexInput>,
    framebuffer: Option<*const FramebufferObject>,
    pipeline: Option<*const super::pipeline::inner::GraphicsPipeline>,
    scissors: Option<Scissors>,
}

impl StateCache
{
    fn new() -> StateCache {
        StateCache {
            uniforms: None,
            vertex_input: None,
            pipeline: None,
            framebuffer: None,
            scissors: None
        }
    }

    unsafe fn set_graphics_pipeline(&mut self, pipe: &GraphicsPipeline) {
        // same pipeline as before?
        if self.pipeline.map_or(true, |prev_pipe| prev_pipe != pipe.as_ref() as *const _) {
            // nope, bind it
            bind_graphics_pipeline(pipe, SG_ALL);
            self.pipeline = Some(pipe.as_ref() as *const _);
        }
    }

    unsafe fn set_uniforms(&mut self, uniforms: &Uniforms)
    {
        // TODO
        bind_uniforms(uniforms);
    }

    unsafe fn set_vertex_input(&mut self, vertex_input: &VertexInput)
    {
        // TODO
        bind_vertex_input(vertex_input);
    }

    unsafe fn set_target(&mut self, framebuffer: &Framebuffer, viewport: &[(f32, f32, f32, f32)]) {
        // same framebuffer as before?
        if self.framebuffer.map_or(true, |prev_framebuffer| prev_framebuffer != framebuffer.as_ref() as *const _) {
            // nope, bind it
            bind_target(framebuffer, viewport);
            self.framebuffer = Some(framebuffer.as_ref() as *const _);
        }
    }

    unsafe fn set_scissors(&mut self, scissors: &Scissors) {
        // TODO
        bind_scissors(scissors);
    }
}


/// Draw command builder
/// statically locks the frame object: allocate your buffers before starting a command!
// TODO move this into its own module
// TODO simplify lifetimes
pub struct DrawCommandBuilder<'a,'q:'a> {
    frame: &'a Frame<'q>,
    uniforms: Uniforms,        // holds arrays of uniforms
    vertex_input: VertexInput, // vertex buffers + index buffer (optional)
    framebuffer: Framebuffer,
    pipeline: GraphicsPipeline,
    scissors: Scissors,
    viewports: [(f32, f32, f32, f32); 8]
}

impl<'a,'y> DrawCommandBuilder<'a,'y> {
    fn new<'b,'q>(frame: &'b Frame<'q>,
                  target: &Framebuffer,
                  pipeline: &GraphicsPipeline,
    ) -> DrawCommandBuilder<'b,'q>
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

    pub fn with_all_viewports(mut self, v: (f32, f32, f32, f32)) -> Self {
        unimplemented!()
    }

    pub fn with_viewport(mut self, index: i32, v: (f32, f32, f32, f32)) -> Self {
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
                       count: usize) -> &'a Frame<'y> {
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
    ) -> &'a Frame<'y>
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
    pub fn draw_quad(mut self) -> &'a Frame<'a>
    {
        unimplemented!()
    }
}
