use std::any;
use typed_arena::Arena;
use std::cell::UnsafeCell;
use petgraph::*;
use petgraph::graph::*;
use petgraph::dot::*;
use gfx;
use std::rc::Rc;
use std::marker::PhantomData;

#[derive(Copy,Clone,Debug)]
struct Lifetime {
    begin: i32,
    end: i32
}

#[derive(Copy,Clone,Debug)]
enum ResourceUsage {
    Default,
    ImageReadWrite,
    SampledTexture,
    RenderTarget,
    TransformFeedbackOutput
}

#[derive(Debug)]
struct LogicalResource
{
    lifetime: Lifetime,
    name: String
}

#[derive(Debug)]
enum Node
{
    Pass {
        name: String
    },
    Resource {
        logical_resource: *mut LogicalResource, // lifetime is bound to the framegraph
        rename_index: i32
    }
}

#[derive(Copy,Clone,Debug)]
struct Edge
{
    usage: ResourceUsage,
    //access: ResourceAccess
}

struct FrameGraph
{
    logical_resources: Arena<LogicalResource>,
    graph: Graph<Node, Edge, Directed>,
}

impl FrameGraph
{
    pub fn new() -> FrameGraph {
        FrameGraph {
            logical_resources: Arena::new(),
            graph: Graph::new()
        }
    }

    fn create_resource_node(&mut self, name: String) -> NodeIndex {
        let ptr = self.logical_resources.alloc(LogicalResource {
            name,
            lifetime: Lifetime {
                begin: 0,
                end: 0
            },
        }) as *mut LogicalResource;
        self.graph.add_node(Node::Resource {
            logical_resource: ptr,
            rename_index: 0
        })
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

// Dummy marker types for gfx_pass macro
struct Texture
{
    pub usage: ResourceUsage,
    pub dimensions: gfx::TextureDimensions,
    pub format: gfx::TextureFormat,
    pub width: u32,
    pub height: u32,
    pub depth: u32,
    pub sample_count: u32,
    pub mip_map_count: u32,
    pub options: gfx::TextureOptions
}

struct Texture2D
{
    pub usage: ResourceUsage,
    pub format: gfx::TextureFormat,
    pub width: u32,
    pub height: u32,
    pub sample_count: u32,
    pub mip_map_count: u32,
    pub options: gfx::TextureOptions
}

struct Buffer<T: gfx::BufferData+?Sized>
{
    pub len: usize,
    _phantom: PhantomData<T>
}

#[derive(Clone,Debug)]
struct TextureConstraints
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

struct BufferConstraints
{
    pub len: Option<usize>
}

impl Default for BufferConstraints
{
    fn default() -> BufferConstraints {
        BufferConstraints {
            len: None
        }
    }
}

trait FrameGraphPassResourceType
{
    type Resource;
    type Constraints: Default;
}

impl FrameGraphPassResourceType for Texture2D {
    type Resource = gfx::Texture;
    type Constraints = TextureConstraints;
}

impl<T: gfx::BufferData+?Sized> FrameGraphPassResourceType for Buffer<T>
{
    type Resource = gfx::Buffer<T>;
    type Constraints = BufferConstraints;
}

macro_rules! gfx_pass {
    // end rule
    // root
    (pass $PassName:ident ( $( $ParamName:ident : $ParamType:ty ),* ) {
        read {
            $( $ReadName:ident : $ReadTy:ident = { $($ReadInit:tt)* } ),*
        }
        write {
            $( $WriteName:ident :  $WriteTy:ident = { $($WriteInit:tt)* } ),*
        }
        create {
            $( $CreateName:ident : $CreateTy:ident = { $($CreateInit:tt)* } ),*
        }
    }) => {
        // Dummy struct
        log_syntax!($($ReadInit)*);
        struct $PassName ();
        mod types {
            pub(super) struct Inputs {
                $($ReadName : u32,)*
                $($WriteName : u32,)*
            }

            pub(super) struct Outputs {
                $($WriteName : u32,)*
                $($CreateName : u32,)*
            }

            pub(super) struct Resources {
                $($ReadName : u32,)*
                $($WriteName : u32,)*
                $($CreateName : u32,)*
            }

            pub(super) struct Parameters {
                $($ParamName : $ParamType,)*
            }
        }

        impl Pass for $PassName {
            type Inputs = types::Inputs;
            type Outputs = types::Outputs;
            type Resources = types::Resources;
        }

        impl $PassName {
            pub fn new( $($ParamName : $ParamType,)* inputs: <$PassName as Pass>::Inputs) -> <$PassName as Pass>::Outputs
            {
                // Read constraints
                //$(let $ReadName = <$ReadTy as FrameGraphPassResourceType>::Constraints { $ReadInit .. Default::default() };)*
                // Write constraints
                //$(let $WriteName = <$WriteTy as FrameGraphPassResourceType>::Constraints { $WriteInit  .. Default::default() };)*
                // Create info
                $(let $CreateName = $CreateTy { $($CreateInit)* };)*

                // Create output nodes
                // Check constraints on inputs
                // Evaluate parameters for created resources
                // Return outputs
                // Optionally call PassSetup::setup() if $PassName implements PassSetup
                unimplemented!()
            }
        }
    };
}

/*impl TextureCreateInfo
{
    fn default_2d() -> TextureCreateInfo {
        TextureCreateInfo {
            usage: ResourceUsage::SampledTexture,
            dimensions: gfx::TextureDimensions::Tex2D,
            format: gfx::TextureFormat::R8G8B8A8_SRGB,
            width: 1,
            height: 1,
            depth: 1,
            sample_count: 1,
            mip_map_count: 1,
            options: gfx::TextureOptions::empty()
        }
    }
}*/

gfx_pass! {
    pass GBufferSetupPass(width: u32, height: u32)
    {
        read {

        }
        write {
            ttrtrtrtr: Texture2D = {
                usage: ResourceUsage::RenderTarget,
                format: gfx::TextureFormat::R16G16B16A16_SFLOAT,
                width,
                height,
                sample_count: 1,
                mip_map_count: 1,
                options: gfx::TextureOptions::empty()
            }
        }
        create {
            diffuse: Texture2D = {
                usage: ResourceUsage::RenderTarget,
                format: gfx::TextureFormat::R16G16B16A16_SFLOAT,
                width,
                height,
                sample_count: 1,
                mip_map_count: 1,
                options: gfx::TextureOptions::empty()
            },
            normals: Texture2D = {
                usage: ResourceUsage::RenderTarget,
                format: gfx::TextureFormat::R16G16B16A16_SFLOAT,
                width,
                height,
                sample_count: 1,
                mip_map_count: 1,
                options: gfx::TextureOptions::empty()
            }
        }
    }
}

impl PassExecute for GBufferSetupPass
{
    fn execute(queue: &gfx::FrameQueue, resources: &<Self as Pass>::Resources)
    {
        unimplemented!()
    }
}

// will impl:
// impl PassParameters for GBuffersSetupPass
//      PassParameters::Parameters (Generated type)
// impl PassResources for GBuffersSetupPass
//



/*
impl Pass for GBufferSetupPass
{
    fn setup(&mut self, inputs: &GBuffersSetupPass::Inputs)
    {
        // self.diffuse.
    }

    fn execute(&self, resources: &GBufferSetupPass::Resources)
    {

    }
}*/

// maybe a custom derive?
// will derive impl RenderPass for GBuffersSetupPass
//

#[cfg(test)] use std::fs::File;
#[cfg(test)] use std::path::Path;
#[cfg(test)] use std::io::Write;

#[test]
fn test_frame_graph_borrows()
{
    let mut fg = FrameGraph::new();

    let p1 = fg.graph.add_node(Node::Pass { name: "GBuffersSetup".to_owned() });
    let p1_out0 = fg.create_resource_node("Diffuse".to_owned());
    let p1_out1 = fg.create_resource_node("Normals".to_owned());
    let p1_out2 = fg.create_resource_node("MaterialID".to_owned());
    fg.graph.add_edge(p1, p1_out0, Edge {usage: ResourceUsage::RenderTarget});
    fg.graph.add_edge(p1, p1_out1, Edge {usage: ResourceUsage::RenderTarget});
    fg.graph.add_edge(p1, p1_out2, Edge {usage: ResourceUsage::RenderTarget});

    let path = Path::new("debug_graph.dot");
    let mut out = File::create(path).unwrap();
    write!(out, "{:#?}", Dot::new(&fg.graph));
}
