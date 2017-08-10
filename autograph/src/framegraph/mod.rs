use std::any;
use typed_arena::Arena;
use petgraph::*;
use petgraph::graph::*;
use petgraph::dot::*;
use gfx;
use std::rc::Rc;
use std::cell::{RefCell,Cell};
use std::marker::PhantomData;
use std::mem;
use std::collections::HashMap;
use std::collections::hash_map::Entry;

pub mod macro_prelude;
pub mod compiled;
pub use self::compiled::CompiledGraph;
pub use petgraph::graph::NodeIndex;

#[derive(Copy, Clone, Debug)]
struct Lifetime {
    begin: i32,
    end: i32    // inclusive
}

impl Lifetime {
    fn overlaps(&self, other: &Lifetime) -> bool {
        self.begin <= other.end && other.begin <= self.end
    }
}

#[derive(Copy, Clone, Debug)]
pub enum ResourceUsage {
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

pub enum ResourceInfo
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
    info: ResourceInfo,
    alloc_index: Cell<Option<usize>>
}

enum Node
{
    Pass {
        name: String,
        execute: Box<Fn(&CompiledGraph)>
    },
    Resource {
        index: usize,
        // lifetime is bound to the framegraph
        rename_index: i32
    }
}

impl ::std::fmt::Debug for Node {
    fn fmt(&self, formatter: &mut ::std::fmt::Formatter) -> Result<(), ::std::fmt::Error>{
        // TODO
        Ok(())
    }
}

#[derive(Copy, Clone, Debug)]
struct Edge
{
    usage: ResourceUsage,
    //access: ResourceAccess
}

pub enum Alloc
{
    Buffer {
        buf: gfx::BufferSliceAny
    },
    Texture {
        tex: Rc<gfx::Texture>
    }
}

pub struct FrameGraphAllocator
{
    allocations: Vec<Alloc>
}

impl FrameGraphAllocator
{
    pub fn new() -> FrameGraphAllocator {
        FrameGraphAllocator {
            allocations: Vec::new()
        }
    }
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
    pub fn create_pass_node(&mut self, name: String, execute: Box<Fn(&CompiledGraph)>) -> NodeIndex
    {
        self.graph.add_node(Node::Pass { name, execute })
    }

    // Create a resource node
    pub fn create_resource_node(&mut self, name: String, info: ResourceInfo) -> NodeIndex
    {
        // Create a new resource
        self.resources.push(Resource {
            name,
            lifetime: None,
            info,
            alloc_index: Cell::new(None)
        });
        let index = self.resources.len() - 1;
        // add resource node
        self.graph.add_node(Node::Resource {
            index,
            rename_index: 0
        })
    }

    pub fn get_resource_info(&self, node: NodeIndex) -> Option<&ResourceInfo>
    {
        self.graph.node_weight(node).and_then(|node| {
           if let &Node::Resource { index, .. } = node {
               Some(&self.resources[index].info)
           } else {
               None
           }
        })
    }

    // Clone a resource node and increase its rename index
    pub fn clone_resource_node(&mut self, resource: NodeIndex) -> NodeIndex {
        let (index, rename_index) = {
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
    pub fn link_input(&mut self, pass: NodeIndex, input: NodeIndex, usage: ResourceUsage) {
        self.graph.add_edge(input, pass, Edge { usage });
    }

    // add an output to a pass node
    pub fn link_output(&mut self, pass: NodeIndex, output: NodeIndex, usage: ResourceUsage) {
        self.graph.add_edge(pass, output, Edge { usage });
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
    // The compiled graph's lifetime is bound to the frame queue
    // Issue: the nodes may contain references
    pub fn compile<'a>(mut self, queue: &'a gfx::FrameQueue, allocator: &'a mut FrameGraphAllocator) -> CompiledGraph<'a>
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
        for (topo_index, &n) in sorted_nodes.iter().enumerate() {
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

        //--------------------------------------
        // STEP 4: Resource allocation
        // Assign 'allocations' (concrete buffers or textures) to resources
        for (index, resource) in self.resources.iter().enumerate() {
            if resource.alloc_index.get().is_none() {
                // No allocation for the resource, create one
                match resource.info {
                    ResourceInfo::Texture { desc: ref texdesc } => {
                        // iter over texture entries, find matching desc
                        let alloc_index = allocator.allocations.iter().enumerate().find(|&(alloc_index, alloc)| if let &Alloc::Texture { ref tex } = alloc {
                            // check if desc matches, and that...
                            *tex.desc() == *texdesc && {
                                // ... the lifetime does not conflict with other users of the alloc
                                self.resources.iter().find(|&other| {
                                    resource.lifetime.unwrap().overlaps(&other.lifetime.unwrap())
                                }).is_none()
                            }
                        } else {
                            false   // not a texture alloc
                        }).map(|(alloc_index, _)| alloc_index); // keep only index, drop borrow of allocations

                        match alloc_index {
                            Some(index) => {
                                debug!("alloc: reusing texture {}", index);
                                resource.alloc_index.set(Some(index));
                            },
                            None => {
                                // alloc a new texture
                                debug!("alloc: new texture {:?}", texdesc);
                                allocator.allocations.push(Alloc::Texture { tex: Rc::new(gfx::Texture::new(queue.context(), texdesc)) });
                                resource.alloc_index.set(Some(allocator.allocations.len()-1));
                            }
                        }
                    }
                    ResourceInfo::Buffer { byte_size } => {
                        use gfx::buffer::AsSlice;
                        // allocating a buffer
                        let buffer = Rc::new(gfx::Buffer::<[u8]>::new(queue.context(), byte_size, gfx::BufferUsage::UPLOAD));
                        allocator.allocations.push(Alloc::Buffer {
                            // TODO allocate in transient pool?
                            // TODO reuse buffers?
                            buf: buffer.as_slice_any()
                        });
                    }
                }
            }
        }

        // now everything should be allocated, build the CompiledGraph object
        CompiledGraph::new(self, sorted_nodes, allocator)
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

*/

#[cfg(test)]
use std::fs::File;
#[cfg(test)]
use std::path::Path;
#[cfg(test)]
use std::io::Write;

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


