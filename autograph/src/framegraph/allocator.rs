use gfx;
use std::rc::Rc;

/// A frame graph resource
/// can be one of several types
/// can be aliased at different stages of the graph
/// if the algorithm does not detect any concurrent modification
/// conflicts
pub(super) enum Alloc
{
    Buffer {
        buf: gfx::BufferSliceAny
    },
    Texture {
        tex: Rc<gfx::Texture>
    }
}

// Structure that holds all the resources allocated by a frame graph
// a frame graph holds exclusive mutable access to the allocator
pub struct FrameGraphAllocator
{
    alloc: Vec<Alloc>,
}

impl FrameGraphAllocator
{
    //pub fn alloc(&self, )
}