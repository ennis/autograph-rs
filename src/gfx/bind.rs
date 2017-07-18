use super::state_group::StateGroup;
use super::frame::Frame;
use super::buffer::*;

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
pub struct DrawCommandBuilder<'frame>
{
    frame: &'frame Frame,
    sg: StateGroup,
}

// Trait with blanket impls to interpret a value as an uniform
// e.g. Vec3: [f32; 3], Vector3, (f32,f32,f32)
// same with matrices

// uniform_vecN(glsl_type,

impl<'frame> DrawCommandBuilder<'frame>
{
    fn new<'a>(frame: &'a Frame) -> DrawCommandBuilder<'a> {
        DrawCommandBuilder { frame: frame, sg: Default::default() }
    }

    fn uniform_buffer<'buf>(self, slot: i32, slice: BufferSlice<'buf>) -> Self where 'frame:'buf
    {
        unimplemented!()
    }

    // TODO impl uniform_vecN, storage_buffer<'buf: 'frame>, uniform_buffer<'buf, 'frame:'buf> (the frame must outlive the

}
