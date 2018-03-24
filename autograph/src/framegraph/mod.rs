//! Frame graphs
//! TODO document
//!
use petgraph::*;
use petgraph::graph::*;
use gfx;
use std::cell::{Cell, RefCell};
use std::collections::HashMap;

mod execution;
pub use self::execution::ExecutionContext;

/// Lifetime of a frame graph resource.
/// TODO document
#[derive(Copy, Clone, Debug)]
struct Lifetime {
    begin: i32,
    end: i32, // inclusive
}

impl Lifetime {
    fn overlaps(&self, other: &Lifetime) -> bool {
        (self.begin <= other.end) && (other.begin <= self.end)
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
    TransformFeedbackOutput,
}

/// Describes creation details of a frame graph resource.
pub enum ResourceInfo {
    Buffer { byte_size: usize },
    Texture { desc: gfx::TextureDesc },
}

/// A resource managed by a frame graph.
struct UnversionedResource {
    lifetime: Option<Lifetime>,
    name: String,
    info: ResourceInfo,
    aliased_index: Cell<Option<AliasedResourceIndex>>,
}

#[derive(Copy,Clone,Hash,Debug,Eq,PartialEq,Ord,PartialOrd)]
struct UnversionedResourceIndex(u32);
// TODO we don't need this
impl UnversionedResourceIndex {
    pub fn new(x: usize) -> Self { UnversionedResourceIndex(x as u32) }
    pub fn index(&self) -> usize { self.0 as usize }
}

/// Render pass callbacks
pub trait RenderPassCallbacks
{
    fn execute(&self, frame: &gfx::Frame, ectx: &ExecutionContext);
}

/// A node of the frame graph.
/// Can be either a `Pass` or a `Resource`.
/// Passes can only be connected to Resources and vice-versa.
enum Node<'a> {
    RenderPass {
        name: String,
        callbacks: Box<RenderPassCallbacks + 'a>,
    },
    Resource {
        index: UnversionedResourceIndex,
        // lifetime is bound to the framegraph
        version: i32,
    },
}

impl<'a> ::std::fmt::Debug for Node<'a> {
    fn fmt(&self, _formatter: &mut ::std::fmt::Formatter) -> Result<(), ::std::fmt::Error> {
        // TODO
        Ok(())
    }
}

#[derive(Copy, Clone, Hash, Debug)]
pub struct ResourceVersion(NodeIndex);
#[derive(Copy, Clone, Hash, Debug)]
pub struct RenderPass(NodeIndex);

/// An edge linking a Pass to a Resource, or a Resource to a Pass.
/// For Resource -> Pass links, represents the state in which the pass expects
/// the resource to be into.
/// For Pass -> Resource links, represents the state in which the pass should leave the resource.
#[derive(Copy, Clone, Debug)]
struct Edge {
    usage: ResourceUsage,
    //access: ResourceAccess
}

/// An AliasedResource represents the GPU memory allocated for a frame graph resource.
/// As their name suggest, AliasedResources can be aliased between resources if the frame graph detects
/// that there are no usage conflicts.
pub enum AliasedResource {
    Buffer { buf: gfx::RawBuffer },
    Texture { tex: gfx::TextureAny },
}

#[derive(Copy, Clone, Hash, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct AliasedResourceIndex(u32);
impl AliasedResourceIndex {
    pub fn new(x: usize) -> Self { AliasedResourceIndex(x as u32) }
    pub fn index(&self) -> usize { self.0 as usize }
}

const FRAMEBUFFER_CACHE_KEY_NUM_COLOR_ATTACHEMENTS: usize = 8;

/// Key used to lookup an existing framebuffer in the cache
#[derive(Copy,Clone,Hash,Debug,Eq,PartialEq)]
struct FramebufferCacheKey {
    color_attachements: [Option<AliasedResourceIndex>; FRAMEBUFFER_CACHE_KEY_NUM_COLOR_ATTACHEMENTS], // TODO non-arbitrary limit
    depth_attachement: Option<AliasedResourceIndex>
}

/// Holds Allocs for a frame graph
pub struct FrameGraphAllocator {
    allocations: Vec<AliasedResource>,
    fbcache: RefCell<HashMap<FramebufferCacheKey, gfx::Framebuffer>>
}


impl FrameGraphAllocator {
    pub fn new() -> FrameGraphAllocator {
        FrameGraphAllocator {
            allocations: Vec::new(),
            fbcache: RefCell::new(HashMap::new())
        }
    }

    // Get a framebuffer for the given texture allocs (first looks into the cache to see if there is one)
    // TODO: don't pass alloc indices: directly pass Arc<Textures>
    fn get_cached_framebuffer(&self, context: &gfx::Context, color_attachements: &[Option<AliasedResourceIndex>], depth_attachement: Option<AliasedResourceIndex>) -> gfx::Framebuffer
    {
        // build key
        assert!(color_attachements.len() <= FRAMEBUFFER_CACHE_KEY_NUM_COLOR_ATTACHEMENTS);
        let key = FramebufferCacheKey {
            color_attachements: {
                let mut array = [None; FRAMEBUFFER_CACHE_KEY_NUM_COLOR_ATTACHEMENTS];
                for i in 0..color_attachements.len() {
                    array[i] = color_attachements[i];
                }
                array
            },
            depth_attachement
        };

        let mut fbcache = self.fbcache.borrow_mut();
        fbcache.entry(key).or_insert_with(|| {
            let mut fbo_builder = gfx::FramebufferBuilder::new(context);
            for (i,color_att) in color_attachements.iter().enumerate() {
                // get texture alloc
                if let &Some(color_att) = color_att {
                    let tex = match self.allocations[color_att.index()] {
                        AliasedResource::Texture { ref tex } => tex,
                        _ => panic!("expected a texture alloc, got something else")
                    };
                    fbo_builder.attach(i as u32, gfx::FramebufferAttachment::Texture(tex));
                }
            }
            if let Some(depth_attachement) = depth_attachement {
                let tex = match self.allocations[depth_attachement.index()] {
                    AliasedResource::Texture { ref tex } => tex,
                    _ => panic!("expected a texture alloc, got something else")
                };
                fbo_builder.attach_depth(gfx::FramebufferAttachment::Texture(tex));
            }
            fbo_builder.build()
        }).clone()
    }
}


/// Pass builder
pub struct RenderPassBuilder<'fg>
{
    pass: RenderPass,
    framegraph: &'fg mut FrameGraph<'fg>,
}

impl<'fg> RenderPassBuilder<'fg>
{
    pub fn read(&mut self, res: ResourceVersion, usage: ResourceUsage)
    {
        self.framegraph.link_input(self.pass, res, usage)
    }

    pub fn write(&mut self, res: ResourceVersion, usage: ResourceUsage) -> ResourceVersion
    {
        let res_v2 = self.framegraph.clone_resource(res);
        self.framegraph.link_input(self.pass, res, usage);
        self.framegraph.link_output(self.pass, res_v2, usage);
        res_v2
    }

    pub fn create_texture<S: Into<String>>(&mut self, name: S, desc: &gfx::TextureDesc, usage: ResourceUsage) -> ResourceVersion
    {
        let res = self.framegraph.create_resource(name.into(), ResourceInfo::Texture { desc: *desc });
        self.framegraph.link_output(self.pass, res, usage);
        res
    }

    pub fn create_buffer<S: Into<String>>(&mut self, name: S, byte_size: usize, usage: ResourceUsage) -> ResourceVersion
    {
        let res = self.framegraph.create_resource(name.into(), ResourceInfo::Buffer { byte_size });
        self.framegraph.link_output(self.pass, res, usage);
        res
    }

    pub fn build(self) -> RenderPass
    {
        self.pass
    }
}

/// A frame graph
///
/// TODO document
pub struct FrameGraph<'node> {
    resources: Vec<UnversionedResource>,
    graph: Graph<Node<'node>, Edge, Directed>,
}

#[derive(Copy,Clone,Debug)]
pub enum Error
{
    ConcurrentWriteHazard,  // TODO more info
}

impl ::std::error::Error for Error
{
    fn description(&self) -> &str {
        match *self {
            Error::ConcurrentWriteHazard => "concurrent write hazard detected"
        }
    }
}

impl ::std::fmt::Display for Error
{
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        match *self {
            Error::ConcurrentWriteHazard => {
                write!(f, "concurrent write hazard detected");
            }
        }
        Ok(())
    }
}

impl<'node> FrameGraph<'node> {
    /// Creates a new frame graph.
    pub fn new() -> FrameGraph<'node> {
        FrameGraph {
            resources: Vec::new(),
            graph: Graph::new(),
        }
    }

    /// Helper for creating a RenderPass node in the graph.
    fn create_render_pass_node(
        &mut self,
        name: String,
        callbacks: Box<RenderPassCallbacks + 'node>,
    ) -> RenderPass {
        RenderPass(self.graph.add_node(Node::RenderPass { name: name.into(), callbacks }))
    }

    /// Create a renderpassbuilder.
    fn create_render_pass<'fg: 'node, S: Into<String>, C: RenderPassCallbacks + 'fg>(&'fg mut self, name: S, callbacks: C) -> RenderPassBuilder<'fg> {
        let pass = self.create_render_pass_node(name.into(), Box::new(callbacks));
        RenderPassBuilder {
            framegraph: self,
            pass
        }
    }

    /// Creates a resource node.
    fn create_resource<S: Into<String>>(&mut self, name: S, create_info: ResourceInfo) -> ResourceVersion {
        // Create a new resource
        self.resources.push(UnversionedResource {
            name: name.into(),
            lifetime: None,
            info: create_info,
            aliased_index: Cell::new(None),
        });
        let rindex = UnversionedResourceIndex::new(self.resources.len() - 1);
        // add resource node
        ResourceVersion(self.graph.add_node(Node::Resource {
            index: rindex,
            version: 0,
        }))
    }

    /// Gets the `ResourceInfo` for the specified resource node.
    pub fn resource_info(&self, res: ResourceVersion) -> &ResourceInfo {
        self.graph.node_weight(res.0).and_then(
            |node| if let &Node::Resource { index, .. } = node {
                Some(&self.resources[index.index()].info)
            } else {
                None
            },
        ).unwrap()
    }

    /// Clones a resource node and increase its version index.
    pub fn clone_resource(&mut self, res: ResourceVersion) -> ResourceVersion {
        let (index, version) = {
            let node = self.graph.node_weight(res.0).unwrap();
            if let &Node::Resource {
                index,
                version,
            } = node
            {
                (index, version)
            } else {
                panic!("not a resource node")
            }
        };
        ResourceVersion(self.graph.add_node(Node::Resource {
            index,
            version: version + 1,
        }))
    }

    /// Adds an input to a pass node.
    fn link_input(&mut self, pass: RenderPass, input: ResourceVersion, usage: ResourceUsage) {
        self.graph.add_edge(input.0, pass.0, Edge { usage });
    }

    /// Adds an output to a pass node.
    fn link_output(&mut self, pass: RenderPass, output: ResourceVersion, usage: ResourceUsage) {
        self.graph.add_edge(pass.0, output.0, Edge { usage });
    }

    /// Returns true if the pass modifies a version of the specfied resource
    fn is_resource_modified_by_pass(&self, pass: RenderPass, rindex: UnversionedResourceIndex) -> bool {
        // For all outgoing neighbors
        self.graph.neighbors_directed(pass.0, Direction::Outgoing)
            // Return true if any...
            .any(|n| match self.graph.node_weight(n).unwrap() {
                // ... outgoing neighbor is a node pointing to the same resource
                // (i.e. the pass writes to the node)
                &Node::Resource { index, .. } if index == rindex => true,
                &Node::RenderPass { .. } => panic!("malformed frame graph"),
                _ => false
            })
    }

    ///
    ///
    fn toposort_nodes(&self) -> Vec<NodeIndex>
    {
        use petgraph::algo::toposort;
        toposort(&self.graph, None).expect("Frame graph contains cycles (how is that possible?)")
    }

    fn detect_concurrent_write_hazards(&self, toposort: &Vec<NodeIndex>) -> bool
    {
        let mut has_write_hazards = false;
        for &n in toposort.iter() {
            if let &Node::Resource { index, .. } = self.graph.node_weight(n).unwrap() {
                //let mut write_count = 0;
                let mut writers = Vec::new();
                //let mut read_count = 0;
                let mut readers = Vec::new();
                for pass in self.graph.neighbors_directed(n, Direction::Outgoing) {
                    if self.is_resource_modified_by_pass(RenderPass(pass), index) {
                        writers.push(pass);
                    } else {
                        readers.push(pass);
                    }
                }
                if (writers.len() > 1) || (readers.len() > 0 && writers.len() > 0) {
                    has_write_hazards = true;
                    error!("Concurrent read/write hazard detected in frame graph");
                    error!(
                        " -> resource {:?} readers {:?} writers {:?}",
                        index,
                        readers,
                        writers
                    );
                }
            }
        }
        has_write_hazards
    }

    /// Determine the lifetime of the resources referenced in the graph.
    fn determine_resource_lifetimes(&mut self, toposort: &Vec<NodeIndex>)
    {
        for (topo_index, &n) in toposort.iter().enumerate() {
            match self.graph.node_weight(n).unwrap() {
                &Node::Resource { .. } => (),
                &Node::RenderPass { .. } => for dep in self.graph.neighbors(n) {
                    if let &Node::Resource { index, .. } = self.graph.node_weight(dep).unwrap() {
                        let resource = &mut self.resources[index.index()];
                        resource.lifetime = match resource.lifetime {
                            None => Some(Lifetime {
                                begin: topo_index as i32,
                                end: topo_index as i32,
                            }),
                            Some(Lifetime { begin, end }) => {
                                assert!(end < topo_index as i32);
                                Some(Lifetime {
                                    begin,
                                    end: topo_index as i32,
                                })
                            }
                        }
                    } else {
                        panic!("Malformed graph")
                    }
                },
            }
        }
    }

    /// Allocate and assign actual resources in the graph.
    /// Concretely, this function sets the 'alloc_index' fields in UnversionedResources
    fn assign_aliased_resources(&mut self, gctx: &gfx::Context, allocator: &mut FrameGraphAllocator)
    {
        for (index, resource) in self.resources.iter().enumerate() {
            if resource.aliased_index.get().is_none() {
                // No allocation for the resource, create one
                match resource.info {
                    ResourceInfo::Texture { desc: ref texdesc } => {
                        // iter over texture entries, find matching desc
                        let arindex = allocator.allocations.iter().enumerate().find(|&(arindex, ar)| if let &AliasedResource::Texture { ref tex } = ar {
                            // check if desc matches, and that...
                            *tex.desc() == *texdesc && {
                                // ... the lifetime does not conflict with other users of the alloc
                                self.resources.iter().enumerate().find(|&(other_index, ref other)| {
                                    (other_index != index)  // not the same resource...
                                        && other.aliased_index.get().map_or(false, |other_arindex| other_arindex == AliasedResourceIndex::new(arindex))   // same allocation...
                                        && resource.lifetime.unwrap().overlaps(&other.lifetime.unwrap())    // ...and overlapping lifetimes.
                                    // if all of these conditions are true, then there's a conflict
                                }).is_none()    // true if no conflicts
                            }
                        } else {
                            false   // not a texture alloc
                        }).map(|(arindex, _)| AliasedResourceIndex::new(arindex)); // keep only index, drop borrow of allocations

                        match arindex {
                            Some(index) => {
                                /*debug!(
                                    "alloc {}({}-{}) reusing texture {}",
                                    resource.name,
                                    resource.lifetime.unwrap().begin,
                                    resource.lifetime.unwrap().end,
                                    index
                                );*/
                                resource.aliased_index.set(Some(index));
                            }
                            None => {
                                // alloc a new texture
                                debug!(
                                    "alloc {}({}-{}) new texture {:?}",
                                    resource.name,
                                    resource.lifetime.unwrap().begin,
                                    resource.lifetime.unwrap().end,
                                    texdesc
                                );
                                allocator.allocations.push(AliasedResource::Texture {
                                    tex: gfx::TextureAny::new(gctx, texdesc),
                                });
                                resource
                                    .aliased_index
                                    .set(Some(AliasedResourceIndex::new(allocator.allocations.len() - 1)));
                            }
                        }
                    }
                    ResourceInfo::Buffer { byte_size } => {
                        // allocating a buffer
                        let buffer = gfx::RawBuffer::new(
                            gctx,
                            byte_size,
                            gfx::BufferUsage::UPLOAD,
                        );
                        allocator.allocations.push(AliasedResource::Buffer {
                            // TODO allocate in transient pool?
                            // TODO reuse buffers?
                            buf: buffer,
                        });
                    }
                }
            }
        }
    }

    /// Consumes self, return a 'compiled frame graph' that is ready to execute.
    /// Borrows FrameGraphAlloc mutably, borrow is dropped when the compiled graph is executed.
    /// The compiled graph's lifetime is bound to the frame queue.
    pub fn finalize(
        mut self,
        gctx: &gfx::Context,
        allocator: &'node mut FrameGraphAllocator,
    ) -> Result<ExecutionContext<'node>, Error> {
        //--------------------------------------
        // STEP 1: Toposort nodes
        let toposort = self.toposort_nodes();

        //--------------------------------------
        // STEP 2: Concurrent resource write detection
        if self.detect_concurrent_write_hazards(&toposort) {
            return Err(Error::ConcurrentWriteHazard)
        }

        //--------------------------------------
        // STEP 3: Lifetime calculation
        self.determine_resource_lifetimes(&toposort);

        //--------------------------------------
        // STEP 4: Resource allocation
        // Assign 'allocations' (concrete buffers or textures) to resources
        self.assign_aliased_resources(gctx, allocator);

        // now everything should be allocated, build the CompiledGraph object
        Ok(ExecutionContext::new(self, allocator, toposort))
    }
}



#[cfg(test)]
mod tests
{
    use std::fs::File;
    use std::path::Path;
    use std::io::Write;
    use super::*;

    #[test]
    fn test_empty() {
        let mut framegraph = FrameGraph::new();
    }

    #[test]
    fn test_simple() {
        let mut framegraph = FrameGraph::new();
        //framegraph.create_render_pass("test", unimplemented!()).build();
    }

    #[test]
    fn test_borrows() {

        /*let mut fg = FrameGraph::new();
        let gbuffers = GBufferSetup::Pass::new(&mut fg, 640, 480);
        let after_scene = RenderScene::Pass::new(&mut fg, 640, 480, gbuffers.diffuse, gbuffers.normals, gbuffers.material_id);
        let after_deferred = DeferredEval::Pass::new(&mut fg, 640, 480, after_scene.diffuse, after_scene.normals, after_scene.material_id);
        OutputToScreen::Pass::new(&mut fg, 640, 480, after_deferred.color0, after_scene.material_id);
        let path = Path::new("debug_graph.dot");
        let mut out = File::create(path).unwrap();
        write!(out, "{:#?}", Dot::new(&fg.graph));*/
    }
}
