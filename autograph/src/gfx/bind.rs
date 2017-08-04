use gl;
use gl::types::*;
use super::queue::Frame;
use super::buffer::*;
use super::Texture;
use super::pipeline::GraphicsPipeline;
use super::upload_buffer::{UploadBuffer,TransientBuffer};
use super::context::Context;
use std::rc::Rc;
use super::buffer_data::BufferData;
use super::framebuffer::Framebuffer;
use std::mem;

const MAX_TEXTURE_UNITS: usize = 16;
const MAX_IMAGE_UNITS: usize = 8;
const MAX_VERTEX_BUFFER_SLOTS: usize = 8;
const MAX_UNIFORM_BUFFER_SLOTS: usize = 8;
const MAX_SHADER_STORAGE_BUFFER_SLOTS: usize = 8;

///
/// Never use that directly since it does not hold references
#[derive(Copy,Clone,Debug,PartialEq,Default)]
struct Uniforms
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
}

#[derive(Copy,Clone,Debug,PartialEq,Default)]
struct VertexInput
{
    vertex_buffers: [GLuint; MAX_VERTEX_BUFFER_SLOTS],
    vertex_buffer_strides: [GLsizei; MAX_VERTEX_BUFFER_SLOTS],
    vertex_buffer_offsets: [GLintptr; MAX_VERTEX_BUFFER_SLOTS],
    index_buffer: GLuint,
    index_buffer_offset: usize,
    index_buffer_size: usize,
    index_buffer_type: GLenum
}

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

// TODO: optimize away redundant state changes
unsafe fn bind_graphics_pipeline(pipe: &GraphicsPipeline, ctx: &Context, mask: StateGroupMask)
{
    /*if mask.contains(SG_VIEWPORTS) {
            // TODO: maybe something a bit less drastic than transmute could be possible?
            gl::ViewportArrayv(0, sg.viewports.len() as i32, mem::transmute(&sg.viewports));
        }

    if mask.contains(SG_SCISSOR_RECT) {
        gl::ScissorArrayv(0, sg.scissors.len() as i32, mem::transmute(&sg.scissors));
    }*/

    if mask.contains(SG_BLEND_STATE) {
        gl::Enable(gl::BLEND); // XXX is this necessary
        for (i,bs) in pipe.blend_states.iter().enumerate() {
            if bs.enabled {
                gl::Enablei(gl::BLEND, i as u32);
                gl::BlendEquationSeparatei(i as u32, bs.mode_rgb, bs.mode_alpha);
                gl::BlendFuncSeparatei(i as u32, bs.func_src_rgb, bs.func_dst_rgb, bs.func_src_alpha, bs.func_dst_alpha);
            }
            else {
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

unsafe fn bind_uniforms(uniforms: &Uniforms)
{
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

unsafe fn bind_vertex_input(vertex_input: &VertexInput)
{
    for i in 0..vertex_input.vertex_buffers.len() {
        if vertex_input.vertex_buffers[i] != 0 {
            gl::BindVertexBuffer(i as u32, vertex_input.vertex_buffers[i], vertex_input.vertex_buffer_offsets[i], vertex_input.vertex_buffer_strides[i]);
        } else {
            gl::BindVertexBuffer(i as u32, 0, 0, 0);
        }
    }

    if vertex_input.index_buffer != 0 {
        gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, vertex_input.index_buffer);
    }
}


/// Draw command trait
pub trait DrawCommand
{
    // unsafe because this binds things to the pipeline
    unsafe fn submit(&self, frame: &Frame, builder: &DrawCommandBuilder);
}

pub struct DrawArrays
{
    pub first: usize,
    pub count: usize
}

impl DrawCommand for DrawArrays
{
    unsafe fn submit(&self, frame: &Frame, builder: &DrawCommandBuilder) {
        gl::DrawArrays(builder.pipeline.primitive_topology, self.first as i32, self.count as i32);
    }
}

pub struct DrawIndexed
{
    pub first: usize,
    pub count: usize,
    pub base_vertex: usize
}

impl DrawCommand for DrawIndexed
{
    unsafe fn submit(&self, frame: &Frame, builder: &DrawCommandBuilder) {
        let index_stride = match builder.vertex_input.index_buffer_type {
            gl::UNSIGNED_INT => 4,
            gl::UNSIGNED_SHORT => 2,
            _ => panic!("Unexpected index type")
        };
        gl::DrawElementsBaseVertex(builder.pipeline.primitive_topology,
                                   self.count as i32,
                                   builder.vertex_input.index_buffer_type,
                                   (builder.vertex_input.index_buffer_offset + self.first * index_stride) as *const GLvoid,
                                   self.base_vertex as i32);
    }
}

enum Scissors {
    All(Option<(i32,i32,i32,i32)>)
}

unsafe fn bind_scissors(scissors: &Scissors)
{
    match scissors {
        &Scissors::All(None) => gl::Disable(gl::SCISSOR_TEST),
        &Scissors::All(Some((x,y,w,h))) => {
            gl::Enable(gl::SCISSOR_TEST);
            gl::Scissor(x,y,w,h);
        }
    }
}

unsafe fn bind_target(framebuffer: &Framebuffer, viewport: &[(f32,f32,f32,f32)])
{
    gl::BindFramebuffer(gl::DRAW_FRAMEBUFFER, framebuffer.obj);
    gl::ViewportArrayv(0, 8, viewport.as_ptr() as *const GLfloat);
}

///
/// Draw command builder
/// lifetime-bound to a frame
pub struct DrawCommandBuilder<'frame>
{
    // can't mut-borrow the frame since transient buffers borrow the frame
    frame: &'frame Frame<'frame>,
    uniforms: Uniforms,     // holds arrays of uniforms
    vertex_input: VertexInput,  // vertex buffers + index buffer (optional)
    framebuffer: Rc<Framebuffer>,
    pipeline: Rc<GraphicsPipeline>,
    scissors: Scissors,
    viewports: [(f32,f32,f32,f32);8],
    // TODO: dynamic states?
}

// Trait with blanket impls to interpret a value as an uniform
// e.g. Vec3: [f32; 3], Vector3, (f32,f32,f32)
// same with matrices

// uniform_vecN(glsl_type,
// Should use Rc for all resource types, since we don't really know how long they should live
// (they should live until the GPU command is processed, but we don't know when exactly)
// although OpenGL will wait for all references to an object in the pipeline to drop
// before actually releasing memory for an object, so it's actually useless
// But do it anyway to mimic vulkan

impl<'a> DrawCommandBuilder<'a>
{
    pub fn new<'b>(frame: &'b Frame, framebuffer: Rc<Framebuffer>, pipeline: Rc<GraphicsPipeline>) -> DrawCommandBuilder<'b>
    {
        let fb_size = framebuffer.size();
        DrawCommandBuilder {
            frame: frame,
            uniforms: Default::default(),
            vertex_input: Default::default(),
            pipeline,
            framebuffer,
            scissors: Scissors::All(None),
            viewports: [(0f32,0f32,fb_size.0 as f32, fb_size.1 as f32);8],
        }
    }

    // TODO struct type check?
    pub fn with_storage_buffer<T: BufferData+?Sized>(mut self, slot: usize, resource: &BufferSlice<T>) -> Self
    {
        // reference this buffer in the frame
        self.frame.ref_buffers.borrow_mut().push(resource.owner.clone());
        self.uniforms.shader_storage_buffers[slot] = resource.owner.object();
        self.uniforms.shader_storage_buffer_offsets[slot] = resource.byte_offset as GLintptr;
        self.uniforms.shader_storage_buffer_sizes[slot] = resource.byte_size() as GLsizeiptr;
        self
    }

    pub fn with_uniform_buffer<T: BufferData+?Sized>(mut self, slot: usize, resource: &BufferSlice<T>) -> Self
    {
        self.frame.ref_buffers.borrow_mut().push(resource.owner.clone());
        self.uniforms.uniform_buffers[slot] = resource.owner.object();
        self.uniforms.uniform_buffer_offsets[slot] = resource.byte_offset as GLintptr;
        self.uniforms.uniform_buffer_sizes[slot] = resource.byte_size() as GLsizeiptr;
        self
    }

    pub fn with_image(mut self, slot: i32, tex: Rc<Texture>) -> Self
    {
        unimplemented!()
    }

    pub fn with_all_viewports(mut self, v: (f32,f32,f32,f32)) -> Self
    {
         unimplemented!()
    }

    pub fn with_viewport(mut self, index: i32, v: (f32,f32,f32,f32)) -> Self
    {
        unimplemented!()
    }

    pub fn with_vertex_buffer<T: BufferData+?Sized>(mut self, slot: usize, vertices: &BufferSlice<T>) -> Self {
        // TODO layout check w.r.t pipeline
        // TODO alignment check
        self.frame.ref_buffers.borrow_mut().push(vertices.owner.clone());
        self.vertex_input.vertex_buffers[slot] = vertices.owner.object();
        self.vertex_input.vertex_buffer_offsets[slot] = vertices.byte_offset as GLintptr;
        self.vertex_input.vertex_buffer_strides[slot] = mem::size_of::<T::Element>() as GLsizei;
        self
    }

    pub fn with_index_buffer<T: BufferData+?Sized>(mut self, indices: &BufferSlice<T>) -> Self {
        self.frame.ref_buffers.borrow_mut().push(indices.owner.clone());
        self.vertex_input.index_buffer = indices.owner.object();
        self.vertex_input.index_buffer_size = indices.byte_size();
        self.vertex_input.index_buffer_offset = indices.byte_offset;
        self.vertex_input.index_buffer_type = match mem::size_of::<T::Element>() {
            4 => gl::UNSIGNED_INT,
            2 => gl::UNSIGNED_SHORT,
            _ => panic!("Unexpected index type!")
        };
        self
    }

    pub fn with_all_scissors(mut self, scissor: Option<(i32,i32,i32,i32)>) -> Self {
        self.scissors = Scissors::All(scissor);
        self
    }

    pub fn command<T: DrawCommand>(mut self, cmd: &T) -> Self
    {
        unsafe {
            bind_graphics_pipeline(&self.pipeline, &self.frame.queue().context(), SG_ALL);
            bind_uniforms(&self.uniforms);
            bind_vertex_input(&self.vertex_input);
            // bind dynamic state
            bind_scissors(&self.scissors);
            bind_target(&self.framebuffer, &self.viewports);
            cmd.submit(self.frame, &self);
        }
        self
    }

}
