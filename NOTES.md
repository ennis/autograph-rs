TODO
====

#### Recap of binding resources (here: buffer slices) to the graphics pipeline
    //
    // The output slice should be valid until the GPU has finished rendering the current frame
    // However, we do not expose a dynamic lifetime to the user: we simply say that
    // the buffer slice can be used until the frame is finalized.
    //
    // TL;DR
    // There are two lifetimes:
    // - logical: slice accessible until the frame is finalized
    // - actual: until the GPU has finished rendering the frame and the resources are reclaimed
    //
    // Issue:
    // Currently, nothing prevents a user from passing a buffer slice that **actually** lives
    // only during the current frame, and not long enough for the GPU operation to complete
    // => extend the lifetime of the buffer by passing an Arc<Buffer> into the buffer slice?
    //
    // Add an Arc<stuff> into the frame each time a resource is referenced in the frame
    //
    // TL;DR: the problem is that any resource reference passed into the GPU pipeline
    // escapes all known static lifetimes
    // Possible solution: bind the lifetime of all 'transient objects' to a GPUFuture object
    // Dropping the GPUFuture object means waiting for the frame to be finished
    // Thus, logical lifetime of the fence = actual lifetime of a frame
    //
    // TL;DR 2: 'Outliving the frame object' is not enough for resources
    // buffer slice of an Arc<Buffer> lives as long as the Arc => should live as long as the buffer?
    // draw pipelines should NOT consume a transient buffer slice
    // Binding a buffer to the pipeline: take a Arc<Buffer> + Buffer slice
    //
    // TL;DR 3 - Conclusion: Resources bound to the pipeline must be Arc, since they escape all known static lifetimes


#### NPR rendering primitives
* Gradation offset (means: shader)
* Gradation offset maps
* HQ Temporal AA (no need for reprojection right now)
* Some work on highlights (shape? anisotropy?)
* Normal simplification
* Local modulation
* Silhouette extension (through)
* Deep G-buffers (optional render output)
* Inflated G-buffers (optional render passes)
* Contour maps

#### Advanced stylization pipeline
* Silhouette extension V2
    * Hybrid geometry / screen-space processing 
    * Arbitrary amount of overlapping screen-space silhouettes
    * Multi-fragment effects: k-buffer
    * NOTE: If rendering with strokes, then probably no need for extended silhouettes
* FOCUS: Render/synthesize/place temporally coherent long strokes
    * Partial stroke fade in/out: never flicker
    * High Quality (AA) stroke rendering
    * Divide the picture in blocks: in each block, a maximum amount of strokes affecting the pixel
        * Sparse convolution noise?
    * Continuum between stroke / continuous shading
        * Stroke/dab = local color correlation
    * Basically ensure that features have a minimum visible stroke size
    * Stroke geometry: endpoints?
* PAINTING: Unlimited level of detail
    * Not limited by texture map resolution
    * Ptex?
* Abstract / simplify geometry in screen-space
    * Not limited to warping / adding details: can also be used to simplify, abstract, stylize
    * Employ tesselation?
    * Can have view-dependent geometry
    * Layered composition
* Shading VS Geometry strokes?

#### Interface
* Editor for gradation
* Paint to texture
* Paint vertex colors?
* Assimp importer
* Frame graph
* Shader bits (live editable)
* Save/load 
	* Cameras
	* Objects
	* Shadings
* Undo/Redo + Interface
* Status bar

#### Renderer
* 'Blackboard' with optionally evaluated passes
	* geometry passes
* Main G-buffer pass
* TAA node


#### Shading file format
* JSON/protobuf?
* Layers
* Light dep
* Associated maps: gradations, local maps

#### Idea


#### Scene
HashMap<EntityID, Component>
Operations:
* Collect: remove all expired ids
* Iterate
* Lookup

Limitations:
* Cannot borrow a reference to a component that outlives the frame (collect() may rehash)
* Cannot add new components in place: must do a functional update / staging area for new components (deferred creation)
* Cannot delete components: that's OK (deferred cleanup)
* References must be IDs

#### Frame graph: custom derive?

##### inputs
- any kind of resource
    - Image
    - Buffer: CpuAccessibleBuffer (UPLOAD), DeviceLocalBuffer (DEFAULT), ImmutableBuffer (DEFAULT+READONLY)
- how it should be used in the pass (bound where / resource state)
    - SampledImage, RWImage, Render target attachement, etc.
    - Views:
        BufferView (automatically created?)
        ImageView
    - Clear value

##### outputs
- same as inputs

##### creates:
- metadata (width, height, etc.)

##### execute:
- access to resources
- the function should be able to access views of the declared resources in the correct state

##### pipeline:
- how should it be managed? can it be dynamically selected?
- use a lazy_static in the execute() function
- subpasses automatically created
- issue: vulkan pipelines must be created with a reference to a subpass
- layout? push values?

**must be able to recreate the framegraph on each frame**
- issue: must recreate pipelines
- the pipelines are valid for one frame only (but likely more than one)

must have an interface to dynamically create passes (variable number of resources, different access flags, etc.)


### Scene graphs / Scene components
- Must not manage hashmaps by hand for every scene component
- Automatically manage atomic updates
    - updates that require exclusive write access
    - done at the end of the frame to keep coherence

### Module organization
- autograph::core
    - shared types
- autograph::gfx
    - OpenGL wrapper
    - textures, buffers, draw commands
- autograph::scene
    - Scene objects
    - scene_object, scene_loader, light, mesh
- autograph::frame_graph
    - Frame graph
- autograph::render
    - Render nodes
- autograph_main
    - Test program crate, WIP modules before integration into autograph (lib)
    - e.g. imgui_glue, test render passes

### Pipeline hot-reload
- Modify cached objects?
- Proposition: cached objects can listen for changes on a file, then trigger a reload if necessary
    - The previous cached object is still there, but clients are signaled that a more recent version is available
    - `Cached<T>::update(&mut self)`: updates/upgrades the resource
    - `Cached<T>::updated(&self) -> Cached<T>`
    - Must have mutable exclusive access to the object: `RefCell<Cached?>`
    - Cannot one-sidedly request an exclusive write borrow to cached objects
    - Other solution: `Cached<RefCell<T>>` for hot-reloadable resources

### Remote user interface through FFI or sockets
- Remotely inspect/modify the scene graph
- undo/redo
- macros to simplify
- Architecture: Model (Rust) - View (Kotlin)
- expose rust types implementing the `Model` trait
    - serialize/deserialize
    - use serde
    - generate corresponding model class on the Kotlin side
    - how to expose UI?
        - create socket
        - call expose(object, objectName)
        - e.g. expose(open_files) 
    - issue: borrow of exposed objects
        - observables? 
    - ideally: atomic operations
        - call `ui.expose(object, name)` in a loop
        - bikeshed: `ui.synchronize(...)`
        - will fetch and apply all modifications done by the UI, and send back the result
            - issue: which version to keep? client version? server version?
            - issue: the client has a cached version of the model
            - can have divergent modification trees (like merge conflicts)
            - `pull` vs `push`
            - compare and coalesce changes before sending to UI
        - can expose anything as long as we have exclusive mutable access
        - can expose at *any time* in the program => IMGUI?
        - modifications are only seen after calling `synchronize`
            - must be called in a loop
            - alternative: observables
- e.g. Frame graph:
    - serialize: convert into list of nodes + edges, then send to UI (through socket?)
    - deserialize: update model data / commit
    - references? must be turned to IDs before serialization

- simple case:
```
    let mut i = 0
    ui.expose("i")
```
- UI side:
```
    var ui = remote("i").observable<Int>("i")
    <some thread receives updates and sends updates to the observable>
```

ui.expose("i", i)
    compare i with cached version
    if not the same, send message Modify("i", Serialize(i))
    

- More complex example: lists
    - basically, do a diff between two lists?
    - send the diff as a sequence of modifications

- Issue: string ID
- Issue: Queries
    - must not send large amounts of updates that are not needed
    - on-demand update
    - lag?

- Undo/redo model changes? Could be implemented automatically

### Native user interface?
- HiDPI support
- IME support
- Popup
- Configurable rendering backend
    - Display list of graphics elements
- Retained or Immediate?
    - at least not purely immediate
    - observables?
    - easy to implement behaviors => state machines?
    - immediate incurs frame lag
- Excellent text rendering
- Must easily handle dynamic data

### Command submission reform

##### Goals
- Goal: be less verbose
    - more implicit stuff?
- now: `DrawCallBuilder` and `DrawCommand`, plus some free functions: `gfx::clear_*`
- they all take a `Frame`
- proposal: merge `Frame` and draw command submission
    - `CommandBuffer` trait?
    - as usual, mimic vulkan (and vulkano?)
    - transient allocation functions in `CommandBuffer`?
        - Do we need separate `UploadBuffers` or just one bound to a command buffer?
- `cmd.transient_alloc<T>()`: automatically synchronized, correct lifetimes
    - `UploadBuffer` now becomes a manually-synchronized impl detail
- `cmd.draw(target, ...)`
- `cmd.clear_texture()`
- `cmd.clear_framebuffer()`
- About upload buffers:
    - we could track every allocation in a list
        - this way it's possible to get rid of the frame-based synchronization => fine-grained
        - but: costly/overhead when there are many small buffers
            - note: the overhead is already there! Vec<FencedRegion>
        - mitigation: batches?
Requirements:
- Expose user-specified upload buffers
- First solution: allocation result should be bound to the lifetime of an object (a synchronized object)
    - Cannot use the transient alloc once the bound object has dropped
    - Can bind to:
        - Command buffer
        - Frame
    - Can reduce overhead: tracking is implicit (bound to the command buffer/frame)
        - Use unsafe code internally
        - No need to wrap slices in Arc: the borrow checker will ensure that they are not in use after the
            command buffer has dropped
    - Less flexible?

- Other solution: dynamic lifetime for everything
    - Arc<TransientBuffer> is just another buffer slice
    - lifetime can be extended after the CommandBuffer has dropped: however, this may block
    - References to Arc<Transient> are kept by the CommandBuffer
    - Possibly heavy overhead: every transient is tracked by an Arc 

- Implementation strategy:
    - `Queue` creates `CommandBuffers`
    - use `CommandBuffer::submit` to submit it to the queue
    - No `Arc`, panic if CommandBuffer is not submitted
    - Synchronization is done at the `CommandBuffer` submission level
        - Keep existing UploadBuffer logic for now
        - Make `UploadBuffer` manually synchronized?
            - `SynchronizedRegion`: once this drops, all allocations inside are automatically reclaimed
            - keeps a ref to the UploadBuffer 
            - (before: sync was implicit, just took the current frame index)
            - In UploadBuffer: keep current region min-max
            - Issue: lifetime of transients inside a synchronized region?
                - unsafe: SynchronizedRegion object is not built yet
                - Create a sync object, but don't signal it yet?
            - `UploadBuffer::with_fence(|| {})`
            - `UploadBuffer::with_fence()` => `SynchronizedRegion`
                - exclusive mut-borrow of uploadbuffer
            - Issue: using the same upload buffer in two different CommandBuffers simultaneously
                - cannot coalesce transient slices
                - mut-borrow upload buffer?
                    - `CommandBuffer::with_upload_buffer`
                - issue: CAN use two different UploadBuffers in the same CommandBuffer
                    but CANNOT use the same UploadBuffer in two different CommandBuffers
                - mitigation: use multiple fences for the same SynchronizedRegion
                    then, wait for all fences before dropping
                - but: who ends the SynchronizedRegion? and when?
                    - the first submitted command buffer ends the SynchronizedRegion
                        - actually: all the synchronizedregions it has created
                    - the second command buffer can do the same, but it will just end an empty region
            - Impl is quite complex, but the resulting API is just:
                - `CommandBuffer::upload_with<T>(&upload_buf, data)`, with no further bookkeeping
                    - which is exactly as simple as it is now...
                    - ...except that it should work with multiple command queues

            - Bikeshedding: `SynchronizedRegion`, `TransientBatch`, `TransientRegion`
            - Note: the underlying sync primitive is not fixed yet
                - can be a `GpuFuture`

##### For now:
- change API so that all commands are traits of `Frame`
    - `impl DrawCommands for Frame`
    - `impl TransientAlloc for Frame`
- remove `UploadBuffer::upload(&frame, ...)`
    - replaced by `frame.alloc_with(&buffer, ...)` of TransientAlloc
- add a default upload buffer in `Frame`, for convenience
    - `frame.alloc<T>(...)`
    - this way it's explicit that the allocation lasts only for the frame
- keep the same synchronization primitive (FenceSync + FenceValue)
- frame submission stays the same
- `draw` functions?
    - DrawIndexed { 
        vertex_buffers: &[&buffer1, &buffer2, ...],
        index_buffer: ...,
        first,
        count, 
      }

##### Draw call parameters
- Bundle vertex buffers and command parameters together
    - This is a user-facing API, not meant as an intermediate API
    - NO: vertex buffer interface is part of the pipeline, not a part of the command
- Describe a layout for the uniforms+vertex input (with a macro, then autobind)
 

### Rendering large worlds

#### Voxel data
Challenge: increase the render distance of minecraft by x100
Chunk size: 16 * 16 * 256 = 65536, with 1 byte per block type
Submit chunks to the mesher / renderer
Extreme render distance = 32 chunks radius (4225 chunks according to wiki)
Thus, 65536*4225 bytes to process = approx 276 MB per frame to process 
Of course, caching of static chunks reduces that significantly
Multiply view dist by 100 => 32 million chunks, approx 2 TB of data
Of course, there is much less _interesting_ data in a chunk (most of it is air, stone or water) 

#### Vertex data
Assuming worst-case meshing scenario, without visibility culling: 
3 floats for position info (12 bytes) * 65536
TODO calculate

Conclusion: meshing is ok for short distances, but what about long-range vistas?
Need adaptive meshing, but cannot load all the data
=> needs LOD, as with terrain rendering
=> 2D heightmaps: adaptive virtual texturing
=> 3D: Sparse voxel octrees?

#### Ray-casting?
Reverse costs of rendering: now the cost is proportional to the output resolution,
instead of the size of the input data
Why not?
The issue being to find a good LOD method for voxel data
