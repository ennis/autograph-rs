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
* **FOCUS**: Render/synthesize/place temporally coherent long and curved strokes
    * Partial stroke fade in/out: **never flicker**
    * High Quality (AA) stroke rendering
    * Divide the picture in blocks: in each block, a maximum amount of strokes affecting the pixel
        * Sparse convolution noise?
    * **Continuum** between stroke / continuous shading
        * Stroke/dab = local color correlation
        * Continuous shading: no spatial color correlation (only shading)
    * Basically ensure that features have a minimum visible stroke size
    * Stroke geometry: endpoints?
* PAINTING: Unlimited level of detail
    * Not limited by texture map resolution
    * Ptex?
    * Details are in strokes, not in the texture
* Abstract / simplify geometry in screen-space
    * Not limited to warping / adding details: can also be used to simplify, abstract, stylize
    * Employ tesselation?
    * Can have view-dependent geometry
    * Layered composition
* Shading VS Geometry strokes?
* Screen-space silhouette-aware "paint" effects
    * Segment the input meshes into topological cylinders/spheres (no inner silhouettes)
    * Detect overlapping silhouette mesh parts, store a list of IDs per pixel
* Take inspiration from photoshop for paint effects

#### Testing
* Curved stroke placement
    * Noise model for anchor points


#### Prioritize
* DONE GlObject type reforms: shorthands for Arc<GlObject>
* Name bikeshedding for the new shader system
    * Be versatile!
    * A component-based environment for creating rendering pipelines with less boilerplate, more encapsulation, more flexibility
        * Shader components
        * Single passes
        * Full multi-pass pipelines
    * TFX? (Destiny's implementation)
    * Something-FX?
    * shadergraph
    * rendergraph
    * HLRG - high-level render graph
    * just call it autograph::rendergraph
        * compiler - in-engine rendergraph compiler
            * parser.rs - nom parser for source files
            * mod.rs - splice GLSL snippets together + instantiate components
            * syntax.rs - AST for HLRG source files
        * loader.rs - load a packaged/serialized rendergraph file
            * self-contained - cannot reference other components
        * cache.rs - cache for compiled pipelines / components
        * mod.rs
            * `RenderGraph`
            * can contain multiple passes
            * passes can share parameters
            * query all passes / subgraphs / exported components
        * ~~component.rs - individual components~~
            * `Component::from_file(...)`
            * Should not need that - components should be manipulated outside the engine
        * gpu_pass.rs - GpuPass/GpuPassSet type 
            * `GpuPassSet::from_file(...)`
            * `GpuPassSet::create_pipeline(...) -> GpuPipeline`   // synthesize a pipeline, possibly pull it from the cache
        * render_pipeline.rs - RenderPipeline type
            * `RenderPipeline::from_file`
            * `RenderPipeline::create_frame_graph`
        * metadata.rs - common metadata types
            * enum Metadata
            * VertexShader/FragmentShader/etc.
        * types.rs - parameter types
            * VecN, MatNxN
            * lambda/components
        * interface.rs - static interface validation macros
            * `shader_interface!`
* Scene submission:
    * TODO
    * autograph::render
        * `auto_params.rs` - auto-binding of shader parameters
* shader reform: complete specification of shader state in data
    * New parser not based on regexps?
        * extend GLSL with custom directives: use phaazon's nom parser
    * keep `GraphicsPipelineBuilder`
    * introduce `load_graphics_pipeline(path) -> GraphicsPipeline`
* shader reform: simplified GLSL-like language
    * No need to specify layout(...) -> automatically added (and statically verified)
        * Add it with metadata?
    * entry points as explicitly named functions with in/out parameters (no globals)
        * no `void main()`
    * keywords: `@vertex @fragment @tess_control @tess_eval @compute @geometry pass rendertarget image buffer param component`
    * flexibility through metadata
    * preprocesses down to GLSL
    * statically verified interfaces
        * extract interface from function signatures and parameters and match against code in rust
        * *never* generate rust code from GLSL
        * rust custom derive to generate interface matching code from struct
            * maps vec types to tuples / cgmath types
    * binding points?
    * Parse in rust?
* shader reform: metadata
* shader reform: shader fragment splicing
    * In rust, so that we don't require an external compiler
* submission reform: (partial) static interface matching
* engine: global- and auto- shader parameters

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

#### Shader reforms
* Low-level vs high-level
* Low-level in rust
    * Static interface matching
    * Easy creation of pipeline from a single file
        * `GpuPipeline::from_file()`
* Communication with engine
    * Auto parameters
* Components
* Specifying states
* Self-contained shader components
    * A function with parameters, inputs, outputs
    * Parameters are exposed to the UI
    * Parameters can be other components/lambdas
    * Reference components by file name
* Uniform scope?
    * As function parameters -> pass down to functions manually
    * As scope variables inside passes or components -> graph rep?
    * Automagically bound?
        * scope variables: some vars must be visible in scope to instantiate a component (injection)
    * As component parameters?

```
component PainterlyFilter: ImageFilter {
    @scope UVSet uvSet0;    // will autobind to uvSet0 if it's visible in scope
    uniform UVSet uvSet1;   // requires a uniform (with scope, it might be e.g. a constant)

    vec4 stuff() { return ...; }    // function that uses uvSet0
    vec4 stuff2() { return ...; }    // same
}
```

* Kotlin as a shading language?
    * DSL? Reflection? Separate?
    * Unify composition and inheritance

```
// This shows as a group
class PainterlyFilter: ImageFilter {
    // This shows as a function node in the graph
    override fun stuff() {
        <<code>>
    }
}

// This shows as a node in the graph editor
class ImageFilter: Pass {
    // ...
}

// Adding an 'ImageFilter' node will show up in code like this:
val Node001 = object : ImageFilter {
    override val XXX = UNBOUND()
    override fun YYY() = UNBOUND()
}

// Nodes are values:
val n001: ImageFilter {
    apply = { ... }
}

```

* Using kotlin directly

* GLSL/HLSL shader snippets
    * The parser should understand GLSL function signatures w/ attributes
        * Issue: this is intrusive; requires an almost complete GLSL parser
* In-engine or off-engine?
    * The engine should know things
    * Should be able to edit nodes in real time
    * Engine only sees a 'compiled' version of the HLG
        * only has to fill the parameters
    * compilation is done off engine by a tool
* Engine: query shaders from the HL graph (HLG)
* Pipeline is a type implementing the `GpuPass` trait
    * Also possibly the `StaticShaderInterface` trait
        * with associated type `Interface`
* Bikeshedding: `GpuPass`, `GpuPipeline` (already used), `Pass` 
* Engine: parameter passing
    * static interface matching
    * dynamic parameter
    * auto-bind from global game state
        * eventually: put CPU-side expressions in shaders
    * auto-bind from file system (using attribute syntax)
        ```
        @filesystem("img/effect/gradient.png")
        uniform sampler2D effectGradient;
        ```
    * No code needed in the engine! Iteration time greatly reduced.
    * metadata to indicate where to look for parameters
        * global, per-object, etc.
        * it is the responsibility of the engine to correctly bind the parameters
    * binding points?
        * match by static interface
        * then match by name
        * can specify binding point as metadata
            * `@binding(0)`
    * specify render targets in shader
        * doable: `@renderTarget(mainDiffuse)`
        * otherwise, bind as static or dynamic interface
* vocabulary
    * attributes(name,values)
    * function(parameters,return-type,attributes)
    * constant
    * component(items)
    * item(declaration|definition)
    * a component that has a declaration item without a definition is a component template
    * definition(function|constant|pass|state)
    * state: blend|rasterizer|depthstencil
    * pass:
* implementation
    * GUI editor?
    * custom language
    * repurposed language
    * rust DSL
    * kotlin DSL
    * dual code<->visual representation
* Serialized form
    * All files are standalone (no cross-refs outside editor - duplicate functions if necessary)
    * SPIR-V bytecode?
* ~~Component instantiation must happen late (in-engine)~~
    * e.g. have a 'ForwardPassTemplate', but instantiate with a lighting function known only at runtime
        * Or: compile permutations in advance?
        * Engine: simply load a SPIR-V binary, read its reflection information
        * => Shader library files
    * instantiation = resolving function names, setting constants
        * resolving functions = linking
        * setting constants = SPIR-V specialization constants
* Simpler choice: precompile permutations

```
let fwdshbase = lib
    .query("ForwardShading")
    .with("VertexDeformation=CharacterVS")
    .with("ShadingModel=Phong")
    .with("BlendMode=XXX");

```

#### Static interface matching
* Goal: make shader parameters (uniforms, samplers, vertex input, render targets) conform to a particular interface
* The shader interface is specified in Rust code as a struct:
    
```
#[derive(VertexInput)]
struct Vertex {
    position: [f32;3],
    normals: [f32;3],
    tangents: [f32;3]
};

#[derive(ShaderInterface)]
struct ShaderParameters 
{
    #[vertex_buffer(0)]
    vertices: BufferSlice<VertexType>,
    #[index_buffer]
    indices: BufferSlice<i32>,
    #[uniform_buffer(0)]
    camera_params: BufferSlice<CameraParams>,
    #[uniform_buffer(1)]
    model_params: BufferSlice<ModelParams>,
    #[texture(0)]
    diffuse_tex: (gfx::Texture, gfx::Sampler),
    #[image(0,read_write,rgba16f)]
    rwimage: gfx::Texture,

    #[viewport(0)]
    viewport: Viewport,

    #[scissor(0)]
    scissor: Scissor,

    // render targets are checked against the fragment shader output signature
    // A relaxed implementation can allow extra outputs in the fragment shader
    // A framebuffer will be automatically constructed from a cache
    #[render_target(0)]
    target_diffuse: gfx::Texture,

    // The depth render target can also be a renderbuffer
    #[depth_render_target]
    target_depth: gfx::Texture,

    // all other parameters are implemented as push constants
    // the order must match
    #[push_constant]
    model_matrix: Matrix4,

    // if the shader contains parameters that are not specified in 
    // the static interface, they can be set as dynamic parameters

}

```

#### Engine-wide parameters
```
let up = UniformParameters::new();
up.add("uCameraParameters", buffer);
up.add("uDefaultTexture", texture);

let cmd_params = DynamicParameters::chain(up);

frame.draw(
    pipeline: impl Pipeline,
    static_interface: impl ShaderInterface,
    cmd_params: impl DynamicParameters,
);

```


#### HLSG: high-level shader graph
* written in some easy-to-parse language
* compiler/editor in kotlin
    * actually just a GLSL splicer
* can describe both:
    * individual passes 
    * post-processing pipelines
* what to splice?
    * Code fragments?
    * or data streams?
    * Support both styles
    * Code-style (function composition) seems less verbose, less prone to complex node graphs
        * Can work without an editor, for now
    * see TFX
* interface with the engine:
    * Scene dispatcher
        * select elements in the scene to render
        * specify the expected shader interface, look for it in the components
        * must create the render targets before
        * in-engine implementation
    * Output node
    * when executing a framegraph, must pass the world state to the scene dispatcher
    * Query full post-proc pipelines...
    * ...or just shader passes, and define the rendering pipeline in code

#### Submission reforms
* draw(pipeline, pipeline params, dynamic params)
* trait PipelineInterface
* macro gpu_interface!{}
* uniform parameter resolvers
    * auto-bind from engine variables
    * auto-bind from file
        * done on pipeline load
    * uniform binding:
        1. bind pipeline-constant uniforms (and render states)
            * shader program(s)
        2. bind static interface uniforms (and render states)
        3. bind remaining dynamic uniforms (and render states)

#### Renderer
* 'Blackboard' with optionally evaluated passes
	* geometry passes
* Main G-buffer pass
* TAA node
* Code through UI and script
    * Use scripts to modify the node graphs (connections, etc.)
    * Options: Lua, Python, Kotlin?
        * dynamically typed
        * Node selectors
        * Procedural modification of nodes through scripts
        * Expose nodes as variables
        * Add/remove/modify variables at runtime
        * Lua is better suited, maybe python?
    * Node = shader snippets, different from framegraph
    * Renderer reads the node graph and creates the framegraph
        * Coalesce shader snippets into one shader
        * must support plugin nodes (C API?)
    * Save individual nodes to a library
    * Save graphs to file
* Edge types
    * Vertex stream
    * Primitive stream
    * Fragment stream
    * Image
    * Structured data block (name+type pairs)
    * Function 
    * Implementation of interface
    * Shader pass
* Node types
    * Vertex shader (vertex stream -> vertex stream)
    * Geometry shader (primitive stream -> primitive stream)
    * Rasterizer (primitive stream -> fragment stream)
    * Output Merge
    * Image pass
* UI
    * node groups
    * edge groups (routing)
* Node inputs
    * Variable number of inputs (varargs)
* Main component: scene renderer node
    * Configurable outputs
* Link parameters between nodes
* Dual representation of nodes
    * Simultaneously through code and UI
* Fast debugging (peek values in the evaluation graph)
* Must support multiple passes
* Load/save node graphs
* Subgraphs
    * Subgraph references
* Global parameters, available everywhere in a subgraph
* Self-contained format for sharing nodes
    * GLSL with pragmas
* Conversion to a frame graph
* Support for dynamic multipass rendering (loops)
    * for each
        * object
        * material mask
    * loop parameters passed as data block
* Should the frame graph backend understand the loops?
    * Possible to have a large number of passes (one per object / object type)
    * Re-submit graph every frame?
        * Too expensive, especially over the network
        * The engine must have __some__ knowledge of the loops
        * Proposition: execution plan
            * Generate minimal linear (no loop) subgraphs
            * Loop over subgraphs
            * Issue: memory aliasing between subgraphs?
    * The frame graph should have full knowledge of loops
        * Subgraphs
            * Seen by the parent graph as only one node
            * Custom resource allocation node
            * Custom lifetime calculation logic?
        * Each node can implement a trait for overriding resource allocation
            * trait AliasedResourceAllocator
        * Some nodes will be executed more than once
        * When custom logic is involved, put it in the framegraph (for now)
            * Material loops
            * Object loops
            * Ping-pong with variable number of iterations (fixed loops)
        * Make it relatively easy to create new framegraph nodes
            * framegraph::NodeBuilder
        * Loop iterations are executed sequentially
            * Basically, replay parts of the execution tree with different input data
            * the same resources are re-used on each iteration
            * must accumulate inside the loop
        * Type-safe inputs and outputs?
            * Any + explicit test
    * Unroll loops on graph creation

        

#### Converting node graphs to a frame graph
* Unfold
    * Unfold subgraphs
* Merge
    * Merge shader snippets into full shaders
    * Fuse GLSL code
    * collect connected vertex/geometry/fragment components
* Pass collection
    * Look for vertex / rasterizer / fragment / output merger subgraphs
    * Detect dependencies
* Frame graph creation
    * Go through all passes and output a frame graph

#### Shader reforms
* Less boilerplate in the rust side
    * GraphicsPipeline::from_file("...") -> GraphicsPipeline
    * more config in GLSL code through pragmas
        * #pragma rasterizer(...)
        * #pragma depth_test(off|on)
        * #pragma stencil_test(off|on)
* Interface matching
    * Define a shader interface in rust code (uniforms, inputs, outputs, etc.)
    * At runtime, match the interface with reflected data from the GLSL side
    * implements the GraphicsPipeline trait
    * see gfx-rs
    * don't be too rigid: allow setting of parameters by name, warning if parameter not found (or panic if we are paranoid)
    * new API for submitting draws
        ```
        draw<P: GraphicsPipeline>(pipeline: P, params: <P as GraphicsPipeline>::Params, other: DynamicParameters)
        ```
    * DynamicParameters are basically unsafe
    * Don't focus on that for now: shaders will be generated at runtime
    * Define 'traits' that specifies the required interface of shaders
        * Can mix-and-match traits
        * Check at runtime, always
        * e.g:
            ```
            trait DeferredEvalShader {
                ...vertex format...
                ...uniforms...
                ...textures...
                ...render targets...
                ...required draw states...
            }
            ```
        * Generate code to safely pass the parameters to the shader
        * Resort to dynamic specification for every extra param that must be set
* Optionally-typed Texture handles
    * Inside: Arc<RawTexture>

#### Frame graph reforms
* Bikeshedding:
    * DONE CompiledGraph -> ExecutionContext
* ExecutionContext
    * RenderPassCallback can query for the allocated resource:
        * ` ectx.texture_resource(index) `
* Safety: 
    * Prevent mixing nodes between frame graphs
        * Fat indices containing a ref to the frame graph
            * issue: borrows the frame graph
    * DONE Prevent mixing resource indices and render pass indices
    * Prevent trying to call ectx.texture_resource(index) with a buffer index
        * Typed resource version indices, implicitly convertible to ResourceVersion
    * OK Prevent concurrent write hazards
    * Prevent invalid bindings of resources
    * Integration with the GPUFuture mechanism
        * Piggyback on some other library for this
        * Vulkano or gfx-rs
* Detect R/W hazards as soon as possible: during creation of the graph
* Use typedef struct for `Arc<gfx::Context>`
* Do not focus on gfx_pass! macro for now   
    * hard to maintain (like maintaining a new language)
* Builder for passes
    * framegraph::PassBuilder
    * builder.read(node-index)
    * builder.write(node-index)
* Types for different node indices
    * Resource(NodeIndex)/MutableResource(NodeIndex) 
        * Node::Resource { UnversionedResourceIndex }
    * Pass(NodeIndex)
    * UnversionedResourceIndex
    * AliasedResourceIndex: actual allocated resource
* Need examples
    * Simple blur (two-pass)
    * SSAO
    * Scene rendering
    * Deferred debug
* Handle loop nodes/subgraphs
    * Node: begin subgraph
    * Node: end subgraph
    * Be able to override the scheduling of a node (i.e. schedule it more than once)
        * might be unsafe?
        * trait Schedule
        * trait RenderPass
        * A -> B -> C -> C.1.E -> C.2.E -> C.3.E -> D: E pass executed 3 times
        * schedule can be done based on dynamic data
            * actual for loop in execution callback
                * override with trait: ScheduleOverride (unsafe)
            * FrameGraphExecutor::run_pass()
            * impl ScheduleOverride for LoopPass 
    * Looping with feedback?
        * cycle in the graph
        * fix this!

#### Implementation details
* Kotlin/TornadoFX
* Rust: remote rendering server
    * Send frame graph to the rust server
    * (through FFI)

#### Shading file format
* JSON/protobuf?
* Layers
* Light dep
* Associated maps: gradations, local maps


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

* simple case

        //----------- RUST ------------
        let mut i = 0
        ui.expose("i", i)

        //----------- KOTLIN ------------
        val i = ui.get("i")
        
* with a model

        //----------- RUST ------------
        // This will generate serialization code
        // and a (protobuf?) schema file for RPC
        // (get_id, set_id)

        #[derive(UiExpose)]
        struct Entity {
            id: i64
        }

        ui.expose("entity", &mut ent)

        //----------- KOTLIN ------------
        val ent = ui.get<Entity>()
        print(ent.i)        // internally: ui.call("entity.get_i", ent).as<int>
        print(ent.s)        // ui.call("entity.get_s", ent).as<string>

* with observables
        
        //----------- RUST ------------
        <same as above>

        //----------- KOTLIN ------------
        val ent = ui.rootEntity
        ent.i = 32          // ui.call("entity.set_i", )
        ent.s = "test"
        ent.i.bind {        // modifications are pushed by the server?
            println("value changed: {}", it)
        }

* Server:
    * `ui.sync("i", &mut i)` 
        * update "i" from the internal model cache
        * can be called at arbitrary locations
    * `ui.expose("ent", &mut ent)`
        * expose an object implementing the Model trait
        * mut-borrows the object
    * `ui.sync()`
        * Read all incoming commands, modify data in exposed models
        * Triggers observables?

* Implementation details
    * Use JSON for messages (for now)
    * Query protocol?
        * struct Query: string id
            * Parse recursively
        * reply: serialized JSON (or error)
        * id: in the form `/root/context1/context2/id/part`
            * e.g. `/componentMaps/transforms/by_id/452210/rotation`
            * `/components/transforms/range/1000/1240`
            * basically an RPC
            * each component returns an interface
            * in server: cache models by ID
            * RPC: call model.rpc(method: &str, params: Json) -> Json
                * parse method parameters
                * automatically generated by a macro, delegates to corresponding rpc_method
                * usually calls recursively into child model handlers
                * #[rpc_method]
                * or manually implemented: impl RpcInterface for Object { ... }
        * The root model is basically a hash map
        * List models can parse paths of the form:
            * `/list/by_id/<id>/<child-query>`
            * if list elements have an RpcHandler, call it with `/<child-query>`
            * otherwise, read and serialize
            * can also query stuff by name
        * Tree models are trivial

    * Subscribe/publish?
    * Issue: write access/modifications
        * Requires mut-references
            * Technically, must mut-borrow the entire application model (!)
            * Separate read and update steps
        * Atomic updates?
        * Update lists?

* Next step:
    * DONE have the value change regularly
    * DONE have a javafx view, query an updated value regularly
    * DONE bind a remote object to an observable
    * Send useful data (current FPS?)
    * Receive useful data (camera movement?)

* All endpoints for an API are stored in an enum

        enum ApiEntryPoints {
            Entity_set_i(),
            Entity_get_i()
            ...
        }

compare i with cached version
if not the same, send message Modify("i", Serialize(i))
    

- More complex example: lists
    - basically, do a diff between two lists?
    - send the diff as a sequence of modifications
    - OR: use ObservableLists on the rust side
        - Observable container

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
