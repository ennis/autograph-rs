use std::any;
use typed_arena::Arena;
use std::cell::UnsafeCell;
use petgraph::Graph;

#[derive(Copy,Clone,Debug)]
struct Lifetime {
    begin: i32,
    end: i32
}

#[derive(Copy,Clone,Debug)]
struct BufferMetadata {
    size: isize
}

#[derive(Copy,Clone,Debug)]
struct TextureMetadata {
    width: i32,
    height: i32,
    depth: i32
}

#[derive(Copy,Clone,Debug)]
struct Buffer {
    obj: u32
}

#[derive(Copy,Clone,Debug)]
struct Texture {
    obj: u32
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
enum LogicalResourceEnum<'fg> {
    Buffer { metadata: BufferMetadata, buf: Option<&'fg Buffer> },
    Texture { metadata: TextureMetadata, tex: Option<&'fg Texture> }
}

#[derive(Debug)]
struct LogicalResource<'fg>
{
    lifetime: Lifetime,
    name: String,
    res: LogicalResourceEnum<'fg>
}

#[derive(Debug)]
struct Handle<'fg> {
    usage: ResourceUsage,
    resource: &'fg LogicalResource<'fg>,
    rename_index: i32,
}

#[derive(Debug)]
struct Pass<'fg>
{
    name: String,
    read: Vec<Handle<'fg>>,
    write: Vec<Handle<'fg>>,
    create: Vec<Handle<'fg>>
}

impl<'fg> Pass<'fg> {
    fn make_default(name: &str) -> Pass {
        Pass { name: name.to_owned(), read: Vec::new(), write: Vec::new(), create: Vec::new() }
    }
}

struct FrameGraph<'fg>
{
    logical_resources: Arena<LogicalResource<'fg>>,
    passes: UnsafeCell<Vec<Pass<'fg>>>  // must be mutated while there are still references to LogicalResources outside
    // but that's okay, since there are no refs to passes leaving the FrameGraph
}

impl<'fg> FrameGraph<'fg>
{
    fn make() -> FrameGraph<'fg> {
        FrameGraph { logical_resources: Arena::new(), passes: UnsafeCell::new(Vec::new()) }
    }

    // Create a logical resource and return a handle to it: this does not mutably borrow
    // The logical resource returned has the same lifetime as the framegraph
    fn create_resource(&'fg self) -> LogicalResource<'fg>
    {
        LogicalResource {
            lifetime: Lifetime { begin: 0, end: 0 },
            name: "Hello".to_owned(),
            res: LogicalResourceEnum::Buffer {
                metadata: BufferMetadata { size: 40 },
                buf: None
            }
        }
    }

    // cannot mutably borrow here:
    fn add_pass(&self)
    {
        unsafe {
            (*self.passes.get()).push(Pass::make_default("Stuff"));
        }
    }

    fn compile(&self)
    {
        // somehow do something on the resources? (the registered passes)
    }
}

/*gfx_pass! {
    struct GBufferSetupPass
    {
        // input, type, name, usage, metadata
        // metadata:
        //  usage, format, width, height, allowed_formats, ...
        //

        @create texture2D diffuse {
            usage: ...,
            format: ...,
        },

        @read texture2D previous {
            usage: ... ,
            allowed_formats: [...],
        },

        width: i32,
        height: i32,
    }
}*/
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


#[test]
fn test_frame_graph_borrows()
{
    //{
    let mut fg = FrameGraph::make();
    let resource1 = fg.create_resource();
    let resource2 = fg.create_resource();
    fg.add_pass();
    //}
    println!("{}", resource1.name);
}
