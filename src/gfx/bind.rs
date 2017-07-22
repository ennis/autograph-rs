use super::state_group::{StateGroup};
use super::frame::Frame;
use super::buffer::*;
use super::Texture;
use super::pipeline::GraphicsPipeline;
use super::upload_buffer::{UploadBuffer,TransientBuffer};
use std::rc::Rc;

/// Trait for objects that can be bound to the pipeline as a resource
pub trait Bind {
    fn bind(&self, sg: &mut StateGroup);
}


// draw macro with dynamic pipelines
// <binding-type> <name> = initializer
// OR: <binding-type> <index> = initializer
// OR: <binding-type>: initializer

/*gfx_draw!(
    target:                     fbo,
    command:                    DrawArrays { ..unimplemented!() },
    uniform uPrevModelMatrix:   unimplemented!(),
    uniform uObjectID:          unimplemented!(),
    uniform_buffer[0]:          unimplemented!(),
    sampled_texture[0]:         (tex, sampler),
);*/

/*gfx_draw!(
    target:         fbo,
    command:        DrawArrays { ... },
    pipeline:       DynamicPipeline,
    vertex_buffer(0):  ,
    index_buffer:   ,
    uniform Name = "...",
    uniform_buffer Struct = "...",
    ssbo Name = <some slice>,
);*/

/*
  gfx_pipeline!(
    blend[index]: BlendState,

  )
*/

///
/// Draw command builder
/// lifetime-bound to a frame
pub struct DrawCommandBuilder<'a>
{
    frame: &'a Frame,
    sg: StateGroup,
    pipeline: Rc<GraphicsPipeline>
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
    pub fn new<'b>(frame: &'b Frame, pipeline: Rc<GraphicsPipeline>) -> DrawCommandBuilder<'b>
    {
        unimplemented!()
        //DrawCommandBuilder { frame: frame, sg: Default::default(), pipeline:  }
    }

    // XXX TransientBuffer should really be T where T: BindableBufferResource
    // BindableBufferResource would have an unsafe get_slice()
    // BindableResource would have a add_ref(Frame)
    pub fn storage_buffer<'frame>(self, slot: i32, slice: &'a TransientBuffer<'frame>) -> Self
    {
        unimplemented!()
    }

    pub fn uniform_buffer<'frame>(self, slot: i32, slice: &'a TransientBuffer<'frame>) -> Self
    {
        unimplemented!()
    }

    pub fn image(self, slot: i32, tex: Rc<Texture>) -> Self
    {
        unimplemented!()
    }

    pub fn all_viewports(self, v: (f32,f32,f32,f32)) -> Self
    {
         unimplemented!()
    }

    pub fn viewport(self, index: i32, v: (f32,f32,f32,f32)) -> Self
    {
        unimplemented!()
    }

    //pub fn named_uniform(self, name: &str, )

    // TODO impl uniform_vecN, storage_buffer<'buf: 'frame>, uniform_buffer<'buf, 'frame:'buf> (the frame must outlive the buf)

}
