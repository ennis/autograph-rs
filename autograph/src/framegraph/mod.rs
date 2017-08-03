use std::any;
use typed_arena::Arena;
use std::cell::UnsafeCell;
use petgraph::*;
use petgraph::graph::*;
use petgraph::dot::*;
use gfx;
use std::rc::Rc;
use std::marker::PhantomData;
use std::mem;

pub mod allocator;
pub mod macros;
pub mod compiled;

use self::compiled::CompiledGraph;
use self::allocator::FrameGraphAllocator;

#[derive(Copy,Clone,Debug)]
struct Lifetime {
    begin: i32,
    end: i32
}

#[derive(Copy,Clone,Debug)]
enum ResourceUsage {
    Default,
    RWImage,
    SampledImage,
    RenderTarget,
    UniformBuffer,
    ShaderStorageBuffer,
    TransformFeedbackOutput
}

/*enum ResourcePayload {
    Buffer {
        byte_size: usize,
        buffer: Option<gfx::BufferSliceAny>
    },
    Texture {
        desc: gfx::TextureDesc,
        texture: Option<Rc<gfx::Texture>>
    }
}*/

enum ResourceInfo
{
    Buffer {
        byte_size: usize,
    },
    Texture {
        desc: gfx::TextureDesc
    }
}

struct Resource
{
    lifetime: Option<Lifetime>,
    name: String,
    info: ResourceInfo
}

#[derive(Clone, Debug)]
enum Node
{
    Pass {
        name: String
    },
    Resource {
        index: usize, // lifetime is bound to the framegraph
        rename_index: i32
    }
}

#[derive(Copy,Clone,Debug)]
struct Edge
{
    usage: ResourceUsage,
    //access: ResourceAccess
}

pub struct FrameGraph
{
    resources: Vec<Resource>,
    graph: Graph<Node, Edge, Directed>,
}

impl FrameGraph
{
    pub fn new() -> FrameGraph {
        FrameGraph {
            resources: Vec::new(),
            graph: Graph::new()
        }
    }

    // create a pass node
    fn create_pass_node(&mut self, name: String) -> NodeIndex
    {
        self.graph.add_node(Node::Pass { name })
    }

    // Create a resource node
    fn create_resource_node(&mut self, name: String, info: ResourceInfo) -> NodeIndex
    {
        // Create a new resource
        self.resources.push(Resource {
            name,
            lifetime: None,
            info
        });

        // add resource node
        self.graph.add_node(Node::Resource {
            index: self.resources.len()-1,
            rename_index: 0
        })
    }

    // Clone a resource node and increase its rename index
    fn clone_resource_node(&mut self, resource: NodeIndex) -> NodeIndex {
        let (index,rename_index) = {
            let node = self.graph.node_weight(resource).unwrap();
            if let &Node::Resource { index, rename_index } = node {
                (index, rename_index)
            } else {
                panic!("Not a resource node")
            }
        };
        self.graph.add_node(Node::Resource { index, rename_index: rename_index + 1 })
    }

    // add an input to a pass node
    fn link_input(&mut self, pass: NodeIndex, input: NodeIndex, usage: ResourceUsage) {
        self.graph.add_edge(input, pass, Edge { usage });
    }

    // add an output to a pass node
    fn link_output(&mut self, pass: NodeIndex, output: NodeIndex, usage: ResourceUsage) {
        self.graph.add_edge( pass, output, Edge { usage });
    }

    // returns the read-write dependency of the node to the given resource
    fn is_resource_modified_by_pass(&self, pass_node: NodeIndex, resource_index: usize) -> bool {
        // For all outgoing neighbors
        self.graph.neighbors_directed(pass_node, Direction::Outgoing)
            // Return true if any...
            .any(|n| match self.graph.node_weight(n).unwrap() {
                // ... outgoing neighbor is a node pointing to the same resource
                // (i.e. the pass writes to the node)
                &Node::Resource { index, .. } if index == resource_index => true,
                &Node::Pass { .. } => panic!("Malformed frame graph"),
                _ => false
            })
    }

    // consumes self, return a 'compiled frame graph' that is ready to execute
    // borrows FrameGraphAlloc mutably, borrow is dropped when the compiled graph is executed
    pub fn compile<'a>(mut self, allocator: &'a mut FrameGraphAllocator) -> CompiledGraph<'a>
    {
        //--------------------------------------
        // STEP 1: Toposort nodes
        use petgraph::algo::toposort;
        let sorted_nodes = toposort(&self.graph, None);
        let sorted_nodes = sorted_nodes.expect("Frame graph contains cycles (how is that possible?)");

        //--------------------------------------
        // STEP 2: Concurrent resource write detection
        for &n in sorted_nodes.iter() {
            if let &Node::Resource { index, .. } = self.graph.node_weight(n).unwrap() {
                //let mut write_count = 0;
                let mut writers = Vec::new();
                //let mut read_count = 0;
                let mut readers = Vec::new();
                for pass in self.graph.neighbors_directed(n, Direction::Outgoing) {
                    if self.is_resource_modified_by_pass(pass, index) {
                        writers.push(pass);
                    } else {
                        readers.push(pass);
                    }
                }
                if (writers.len() > 1) || (readers.len() > 0 && writers.len() > 0) {
                    error!("Concurrent read/write hazard detected in frame graph");
                    error!(" -> resource {:?} readers {:?} writers {:?}", index, readers, writers);
                }
            }
        }

        //--------------------------------------
        // STEP 3: Lifetime calculation
        let mut pass_index = 0;
        for (topo_index,&n) in sorted_nodes.iter().enumerate() {
            match self.graph.node_weight(n).unwrap() {
                &Node::Resource { .. } => (),
                &Node::Pass { .. } => {
                    for dep in self.graph.neighbors(n) {
                        if let &Node::Resource { index, .. } = self.graph.node_weight(dep).unwrap() {
                            let resource = &mut self.resources[index];
                            resource.lifetime = match resource.lifetime {
                                None => Some(Lifetime { begin: topo_index as i32, end: topo_index as i32 }),
                                Some(Lifetime { begin, end }) => {
                                    assert!(end < topo_index as i32);
                                    Some(Lifetime { begin, end: topo_index as i32 })
                                }
                            }
                        } else {
                            panic!("Malformed graph")
                        }
                    }
                }
            }
        }

        // allocation map:
        // resource_id (FrameGraph) => alloc_id (FrameGraphAlloc)
        // however, many ids; could do instead:
        // resource: *Resource => alloc: &Alloc
        // however, must put resources in a vec, since we want to iterate on them
        // resource_id => &Alloc
        // where Alloc can be Alloc::Texture or Alloc::Buffer
        // allocator.alloc(&self, &Resource) -> &Alloc



        //--------------------------------------
        // STEP 4: Resource allocation
        for (index,resource) in self.resources.iter_mut().enumerate() {
            // it is allocated?
            // lookup index in cache, if present => continue
            // else: match resource
            // if texture => iter over texture entries, find matching desc
            // check all other users of the concrete resource (alloc) for lifetime conflicts
            // self.check_lifetime_conflicts(resource_id, alloc_id)
            // if any => continue iter
            // none found => create new texture
            // always create new buffers
        }

        unimplemented!()
    }

    //fn check_lifetime_conflict(&self, )
}


/*
pub trait Pass
{
    type Inputs;    // Node handles
    type Outputs;   // Node handles
    type Resources;
}

pub trait PassExecute: Pass
{
    fn execute(queue: &gfx::FrameQueue, resources: &<Self as Pass>::Resources);
}

pub trait ResourceDesc
{
    type Target;
}

trait ToResource
{
    fn to_payload(&self) -> ResourcePayload;
}

impl ResourceDesc for gfx::TextureDesc
{
    type Target=Rc<gfx::Texture>;
}

impl ToResource for gfx::TextureDesc {
    fn to_payload(&self) -> ResourcePayload {
        ResourcePayload::Texture { desc: *self, texture: None }
    }
}

struct BufferDesc<T: gfx::BufferData+?Sized>
{
    pub len: usize,
    _phantom: PhantomData<T>
}

impl<T: gfx::BufferData+?Sized> ResourceDesc for BufferDesc<T>
{
    type Target=gfx::BufferSliceAny;
}

impl<T: gfx::BufferData+?Sized> ToResource for BufferDesc<T>
{
    fn to_payload(&self) -> ResourcePayload {
        ResourcePayload::Buffer { byte_size: self.len * mem::size_of::<T::Element>(), buffer: None }
    }
}*/


/*
gfx_pass! {
    pass GBufferSetup(width: u32, height: u32)
    {
        read {
        }
        write {
        }
        create {
            #[RenderTarget]
            diffuse : gfx::TextureDesc = gfx::TextureDesc {
                format: gfx::TextureFormat::R16G16B16A16_SFLOAT,
                width,
                height,
                .. gfx::TextureDesc::default_2d()
            },
            #[RenderTarget]
            normals : gfx::TextureDesc = gfx::TextureDesc {
                format: gfx::TextureFormat::R16G16B16A16_SFLOAT,
                width,
                height,
                .. gfx::TextureDesc::default_2d()
            },
            #[RenderTarget]
            material_id : gfx::TextureDesc = gfx::TextureDesc {
                format: gfx::TextureFormat::R16_UINT,
                width,
                height,
                .. gfx::TextureDesc::default_2d()
            }
        }
    }
}

gfx_pass!{
    pass RenderScene(width: u32, height: u32)
    {
        read {}
        write {
            #[RenderTarget]
            diffuse: TextureConstraints = Default::default(),
            #[RenderTarget]
            normals: TextureConstraints = Default::default(),
            #[RenderTarget]
            material_id: TextureConstraints = Default::default()
        }
        create {}
    }
}

gfx_pass!{
    pass DeferredEval(width: u32, height: u32)
    {
        read {
            #[SampledImage]
            diffuse: TextureConstraints = Default::default(),
            #[SampledImage]
            normals: TextureConstraints = Default::default(),
            #[SampledImage]
            material_id: TextureConstraints = Default::default()
        }
        write {
        }
        create {
            #[RenderTarget]
            color0 : gfx::TextureDesc = gfx::TextureDesc {
                format: gfx::TextureFormat::R16G16B16A16_SFLOAT,
                width,
                height,
                .. gfx::TextureDesc::default_2d()
            }
         }
    }
}

gfx_pass!{
    pass OutputToScreen(width: u32, height: u32)
    {
        read {
            #[SampledImage]
            color0: TextureConstraints = Default::default(),
            #[SampledImage]
            material_id: TextureConstraints = Default::default()
        }
        write {
        }
        create {
        }
    }
}
*/

#[cfg(test)] use std::fs::File;
#[cfg(test)] use std::path::Path;
#[cfg(test)] use std::io::Write;

#[test]
fn test_frame_graph_borrows()
{
    /*let mut fg = FrameGraph::new();

    let gbuffers = GBufferSetup::Pass::new(&mut fg, 640, 480);
    let after_scene = RenderScene::Pass::new(&mut fg, 640, 480, gbuffers.diffuse, gbuffers.normals, gbuffers.material_id);
    let after_deferred = DeferredEval::Pass::new(&mut fg, 640, 480, after_scene.diffuse, after_scene.normals, after_scene.material_id);
    OutputToScreen::Pass::new(&mut fg, 640, 480, after_deferred.color0, after_scene.material_id);

    let path = Path::new("debug_graph.dot");
    let mut out = File::create(path).unwrap();
    write!(out, "{:#?}", Dot::new(&fg.graph));*/
}


