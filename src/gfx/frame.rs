
use std::marker::PhantomData;

pub struct Frame {
    // The ID of the current frame
    pub id: i64,
    // TODO list of drop continuations
}


// A 'frame' object begins when the user starts sending draw commands
// and 'dies' when the GPU has finished rendering and presented the results to the screen
// As such, it has a dynamic lifetime
// However, the user can access frame-bound resources only **before** the commands are submitted
// to the GPU
//
// gfx::draw(frame).uniform(...)
//            ...
//      .submit()
//
// Then, at the end:
// context.end_frame(frame)     // take ownership of the frame
//
// context.end_frame returns a GpuFuture (awaitable object)
//
// Each frame has an associated fence object that tells whether this frame has ended or not
// dropping a frame waits for the frame to end
//
// An object that outlives a frame can register a callback for when the
// frame has finished (i.e. a continuation / future 'then')
// e.g. UploadBuffers can register a callback for when the frame has finished rendering
// so that it can free its resources (on_destroy(|f| { self.reclaim(f); })
//
// Issue: upload buffer work with only one stream of frames (one 'queue'): it relies on
// strictly increasing frame IDs for cleanup
//
// What about async compute? => compute that runs outside a frame
// Maybe a compute frame?
//
//

//
// Consider using vulkano?
// => Not yet usable
// We would only have to provide the frame graph; keep the shader preprocessor (already working)
// TODO: analyze whether it is useful to have dynamic states for things like vertex layouts and such
//