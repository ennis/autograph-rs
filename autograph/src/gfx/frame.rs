

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
// Bind an upload buffer to a queue before
// Right now, queue = context
//
// Don't rely on IDs => rely on sync objects
// But: don't want a sync object for each
//
// What about async compute? => compute that runs outside a frame
// Maybe a compute frame?
// Different frames for each queue, with different lifetimes
//
// How to prevent a buffer created in one queue to be used in another queue?
// => lifetimes?
//

// In the end, a frame is just a synchronisation primitive to avoid too much granularity in syncs
// so: it should be possible to sync on a frame
// another form of syncing would be to simply store a frame ID and check that the corresponding
// fence has reached that ID => semaphores
//
// TODO: inter-queue synchronization? => tricky


//
// Consider using vulkano?
// => Not yet usable
// We would only have to provide the frame graph; keep the shader preprocessor (already working)
// TODO: analyze whether it is useful to have dynamic states for things like vertex layouts and such
//