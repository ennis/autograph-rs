
// Resources:
// Can be aliased
// - textures (images)
// - buffers

// Textures:
// - owned (GL,VK)
// - aliased to object pool (GL,VK)
// - aliased to memory region (VK only)
//
// Buffers:
// - owned (GL,VK)
// - slice of larger buffer (GL,VK)
// - slice of ring buffer
//
// Resources:
// - is transient?
//
// On requesting a new resource, the system chooses where it comes from:
// - new object
// - aliased in object pool
//  (object pools: Rc<Texture|Buffer>)
// - allocated in transient pool: has a ref to buffer and texture and an "expiry date"
//
// => ONLY WAY TO GET A RESOURCE, the actual storage is abstracted away

/// Represents a resource.
/// The storage for the resource is abstracted away.
trait Resource
{
    /// A type that contains a description of the resource (e.g. TextureDesc, BufferDesc, ...)
    type Desc;
}

/// Represents a texture resource.
/// The storage for the texture may vary:
/// it may be aliased with other texture resources in the frame.
/// The task scheduler makes sure that no conflicts occur.
/// All texture resource objects are owned by the frame graph, and cannot
/// be moved away.
///
/// References to this are given to tasks when they are executing.
/// The frame graph ensure that they are in the correct state.
struct Texture
{
    desc: TextureDesc,
    /// Reference to the underlying texture object.
    /// None if not yet allocated.
    storage: RefCell<Option<gl::Texture>>,
}

enum BufferStorage
{
    Buffer()
}

// 1. frame graph, with nodes that declare their inputs and outputs and created resources
// (2. schedule nodes on different queues for best async)
// 3. each resource has an associated queue and lifetime (first-last task in the toposorted graph)
// 4. realization: determine the memory requirements of the pipeline, and assign actual storage to resources, creating new ones as needed.
//      -> realize.rs
//
// External updates?
// -> re-create the graph each frame (but maybe cache some things?)
// -> vulkan renderpasses?
//
// Individual draw calls?
// -> no need for synchronization
//
// Static geometry buffers?
// -> persistent resources
//
// Dynamic geometry buffers?
// -> update should be done inside the graph.
// -> two steps: request size (when building the graph), then execute.
//
// Two categories of allocations:
// -> available immediately for upload (UPLOAD)
// -> available when the command executes (CREATE)
//
// Two passes:
// -> SETUP pass: set sizes, upload stuff
// -> COMMAND pass: send commands
//
// Interaction with shaderinterfaces?
//
// Graph inputs and outputs:
// executed in a loop
// while true {
//      ... update ...
//      ... build graph ...
//          - create frame
//          - create node
//              - declare resources for each node
//      ... schedule graph ...
//      scheduler.schedule(graph) -> impl Future<GraphOutput>
//      ... get (wait for) results on the CPU ...
// }
// Be able to synchronize on a result
// execution is triggered by awaiting a GpuFuture?
//
// API for setup
//      .create_buffer("name", ...)
//      .upload("name2", ...)
// API for command:
//      .get_buffer("name")
//
// context -> frame -> node
//
//
// Context::create_frame() -> Frame<'ctx>
// Frame::create_node(|builder| {
//  let a = builder.create_buffer();
//  let b = builder.create_texture();
//
// })
// node.create_buffer(...)
// node.create_texture(...)
// node.upload_buffer(...)
// node.upload_texture(...)
// node.update_texture(...) // staged texture update
// node.draw(xxx)
// context.schedule_frame(...) -> impl Future<Output> // submit to render, releases all borrows
//
// start from low-level API, no interface checking
// Each frame, build a list of operation nodes
// each node
//
// Persistent resources?


struct Frame
{
    resources: Vec<Resource>,
    graph: Graph<Node, Dependency, Directed>,
}

#[derive(Copy, Clone, Debug)]
#[repr(u8)]
enum DependencyKind
{
    Read,
    Write
}

#[derive(Copy, Clone, Debug)]
struct Dependency
{
    resource_index: ResourceIndex,
    kind: DependencyKind,
}

/// A reference to a transient resource, with a revision number.
struct ResourceHandle<'a>
{
    revision: u32,
    index: ResourceIndex,
    phantom: PhantomData<&'a ()>,
}

type BufferVersion = ResourceVersion<Buffer>;
type TextureVersion = ResourceVersion<Texture>;

impl Frame
{
    pub fn create_buffer(&mut self, node_id: NodeId, ) -> BufferRevision
    {

    }
}
