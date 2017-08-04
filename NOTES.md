TODO
====

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

##### Implementation agenda:
* Explicit API first
    * setup() + execute()
    * No corresponding allocation, just a prototype
*

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
        - can expose anything as long as we have exclusive mutable access
        - can expose at *any time* in the program => IMGUI?
        - modifications are only seen after calling `synchronize`
            - must be called in a loop
            - alternative: observables
- e.g. Frame graph:
    - serialize: convert into list of nodes + edges, then send to UI (through socket?)
    - deserialize: update model data / commit
    - references? must be turned to IDs before serialization


