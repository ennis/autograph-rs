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

enum ResourcePayload {
    Buffer {
        byte_size: usize,
        buffer: Option<gfx::BufferSliceAny>
    },
    Texture {
        desc: gfx::TextureDesc,
        texture: Option<Rc<gfx::Texture>>
    }
}

struct Resource
{
    lifetime: Option<Lifetime>,
    name: String,
    payload: ResourcePayload,
}

#[derive(Clone, Debug)]
enum Node
{
    Pass {
        name: String
    },
    Resource {
        resource_ptr: *mut Resource, // lifetime is bound to the framegraph
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
    resources: Arena<Resource>,
    graph: Graph<Node, Edge, Directed>,
}

impl FrameGraph
{
    pub fn new() -> FrameGraph {
        FrameGraph {
            resources: Arena::new(),
            graph: Graph::new()
        }
    }

    // create a pass node
    fn create_pass_node(&mut self, name: String) -> NodeIndex
    {
        self.graph.add_node(Node::Pass { name })
    }

    // Create a resource node
    fn create_resource_node(&mut self, name: String, payload: ResourcePayload) -> NodeIndex
    {
        // Create a new resource
        let ptr = self.resources.alloc(Resource {
            name,
            lifetime: None,
            payload
        }) as *mut Resource;

        // add resource node
        self.graph.add_node(Node::Resource {
            resource_ptr: ptr,
            rename_index: 0
        })
    }

    // Clone a resource node and increase its rename index
    fn clone_resource_node(&mut self, resource: NodeIndex) -> NodeIndex {
        let (resource_ptr,rename_index) = {
            let node = self.graph.node_weight(resource).unwrap();
            if let &Node::Resource { resource_ptr, rename_index } = node {
                (resource_ptr, rename_index)
            } else {
                panic!("Not a resource node")
            }
        };
        self.graph.add_node(Node::Resource { resource_ptr, rename_index: rename_index + 1 })
    }

    // add an input to a pass node
    fn link_input(&mut self, pass: NodeIndex, input: NodeIndex, usage: ResourceUsage)
    {
        self.graph.add_edge(input, pass, Edge { usage });
    }

    // add an output to a pass node
    fn link_output(&mut self, pass: NodeIndex, output: NodeIndex, usage: ResourceUsage)
    {
        self.graph.add_edge( pass, output, Edge { usage });
    }

    // returns the read-write dependency of the node to the given resource
    fn is_resource_modified_by_pass(&self, pass_node: NodeIndex, resource: *const Resource) -> bool {
        // For all outgoing neighbors
        self.graph.neighbors_directed(pass_node, Direction::Outgoing)
            // Return true if any...
            .any(|n| match self.graph.node_weight(n).unwrap() {
                // ... outgoing neighbor is a node pointing to the same resource
                // (i.e. the pass writes to the node)
                &Node::Resource { resource_ptr, .. } if resource_ptr as *const _ == resource => true,
                &Node::Pass { .. } => panic!("Malformed frame graph"),
                _ => false
            })
    }

    pub fn compile(&mut self)
    {
        //--------------------------------------
        // STEP 1: Toposort nodes
        use petgraph::algo::toposort;
        let sorted_nodes = toposort(&self.graph, None);
        let sorted_nodes = sorted_nodes.expect("Frame graph contains cycles (how is that possible?)");

        //--------------------------------------
        // STEP 2: Concurrent resource write detection
        for &n in sorted_nodes.iter() {
            if let &Node::Resource { resource_ptr, .. } = self.graph.node_weight(n).unwrap() {
                //let mut write_count = 0;
                let mut writers = Vec::new();
                //let mut read_count = 0;
                let mut readers = Vec::new();
                for pass in self.graph.neighbors_directed(n, Direction::Outgoing) {
                    if self.is_resource_modified_by_pass(pass, resource_ptr) {
                        writers.push(pass);
                    } else {
                        readers.push(pass);
                    }
                }
                if (writers.len() > 1) || (readers.len() > 0 && writers.len() > 0) {
                    error!("Concurrent read/write hazard detected in frame graph");
                    unsafe {
                        error!("Resource {:?} readers {:?} writers {:?}", &(*resource_ptr).name, readers, writers);
                    }
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
                        if let &Node::Resource { resource_ptr, .. } = self.graph.node_weight(dep).unwrap() {
                            let resource_ptr = unsafe { &mut *resource_ptr };
                            resource_ptr.lifetime = match resource_ptr.lifetime {
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
        // TODO

    }
}

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
}

#[derive(Clone,Debug)]
pub struct TextureConstraints
{
    usage: ResourceUsage,
    dimensions: Option<gfx::TextureDimensions>,
    allowed_formats: Option<Vec<gfx::TextureFormat>>,
    width: Option<u32>,
    height: Option<u32>,
    depth: Option<u32>
}

impl Default for TextureConstraints
{
    fn default() -> TextureConstraints {
        TextureConstraints {
            dimensions: None,
            usage: ResourceUsage::Default,
            allowed_formats: None,
            width: None,
            depth: None,
            height: None
        }
    }
}

#[derive(Debug)]
pub struct BufferConstraints<T: gfx::BufferData+?Sized>
{
    pub len: Option<usize>,
    _phantom: PhantomData<T>
}

impl<T: gfx::BufferData+?Sized> Default for BufferConstraints<T>
{
    fn default() -> BufferConstraints<T> {
        BufferConstraints {
            len: None,
            _phantom: PhantomData
        }
    }
}

pub trait PassConstraintType
{
    type Target;
}

impl PassConstraintType for TextureConstraints
{
    type Target = Rc<gfx::Texture>;
}

impl<T: gfx::BufferData+?Sized> PassConstraintType for BufferConstraints<T>
{
    type Target = gfx::BufferSliceAny;
}

macro_rules! gfx_pass {
    // end rule
    // root
    (pass $PassName:ident ( $( $ParamName:ident : $ParamType:ty ),* ) {
        read {
            $( #[$ReadUsage:ident] $ReadName:ident : $ReadTy:ty = $ReadInit:expr),*
        }
        write {
            $( #[$WriteUsage:ident] $WriteName:ident : $WriteTy:ty = $WriteInit:expr),*
        }
        create {
            $( #[$CreateUsage:ident] $CreateName:ident : $CreateTy:ty = $CreateInit:expr),*
        }
        // Other items go into the Pass impl
    }) => {
        // Dummy struct
        //log_syntax!($($ReadInit)*);
        pub mod $PassName {
            use $crate::framegraph::*;

            pub struct Inputs {
                $(pub $ReadName : $crate::petgraph::graph::NodeIndex,)*
                $(pub $WriteName : $crate::petgraph::graph::NodeIndex,)*
            }

            pub struct Outputs {
                $(pub $WriteName : $crate::petgraph::graph::NodeIndex,)*
                $(pub $CreateName : $crate::petgraph::graph::NodeIndex,)*
            }

            pub struct Resources {
                $(pub $ReadName : <$ReadTy as $crate::framegraph::PassConstraintType>::Target,)*
                $(pub $WriteName : <$WriteTy as $crate::framegraph::PassConstraintType>::Target,)*
                $(pub $CreateName : <$CreateTy as $crate::framegraph::ResourceDesc>::Target,)*
            }

            pub struct Parameters {
                $(pub $ParamName : $ParamType,)*
            }

            pub struct Pass();

            impl $crate::framegraph::Pass for Pass {
                type Inputs = Inputs;
                type Outputs = Outputs;
                type Resources = Resources;
            }

            impl Pass {
                pub fn new(frame_graph: &mut $crate::framegraph::FrameGraph, $($ParamName : $ParamType,)* $($ReadName : $crate::petgraph::graph::NodeIndex,)* $($WriteName : $crate::petgraph::graph::NodeIndex,)* ) -> Outputs
                {
                    // move inputs into their own struct for convenience
                    // within this macro, we can explicitly name the type
                    let inputs = Inputs {
                        $($ReadName,)*
                        $($WriteName,)*
                    };

                    // Read constraints
                    $(let mut $ReadName : $ReadTy = $ReadInit;)*
                    // Write constraints
                    $(let mut $WriteName : $WriteTy = $WriteInit;)*
                    // Create info
                    $(let mut $CreateName : $CreateTy = $CreateInit;)*

                    // 1. Create pass node
                    let node = frame_graph.create_pass_node(stringify!($PassName).to_owned());
                    // 2. link inputs
                    $( frame_graph.link_input(node, inputs.$ReadName,  $crate::framegraph::ResourceUsage::$ReadUsage); )*
                    $( frame_graph.link_input(node, inputs.$WriteName, $crate::framegraph::ResourceUsage::$WriteUsage); )*
                    // 3. create new resource nodes
                    let outputs = Outputs {
                        $( $CreateName: frame_graph.create_resource_node(stringify!($CreateName).to_owned(), $CreateName.to_payload() ), )*
                        $( $WriteName: frame_graph.clone_resource_node(inputs.$WriteName), )*
                    };

                    // 4. link outputs
                    $(frame_graph.link_output(node, outputs.$CreateName, $crate::framegraph::ResourceUsage::$CreateUsage);)*
                    $(frame_graph.link_output(node, outputs.$WriteName, $crate::framegraph::ResourceUsage::$WriteUsage);)*

                    // 5. return outputs
                    outputs
                }
            }
        }
    };
}

impl PassExecute for GBufferSetup::Pass
{
    fn execute(queue: &gfx::FrameQueue, resources: &GBufferSetup::Resources)
    {
        unimplemented!()
    }
}


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


#[cfg(test)] use std::fs::File;
#[cfg(test)] use std::path::Path;
#[cfg(test)] use std::io::Write;

#[test]
fn test_frame_graph_borrows()
{
    let mut fg = FrameGraph::new();

    let gbuffers = GBufferSetup::Pass::new(&mut fg, 640, 480);
    let after_scene = RenderScene::Pass::new(&mut fg, 640, 480, gbuffers.diffuse, gbuffers.normals, gbuffers.material_id);
    let after_deferred = DeferredEval::Pass::new(&mut fg, 640, 480, after_scene.diffuse, after_scene.normals, after_scene.material_id);
    OutputToScreen::Pass::new(&mut fg, 640, 480, after_deferred.color0, after_scene.material_id);

    let path = Path::new("debug_graph.dot");
    let mut out = File::create(path).unwrap();
    write!(out, "{:#?}", Dot::new(&fg.graph));
}


