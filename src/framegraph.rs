use std::any;
use typed_arena::Arena;
use std::cell::UnsafeCell;

#[derive(Debug)]
struct Lifetime {
    begin: i32,
    end: i32
}

#[derive(Debug)]
struct BufferMetadata {
    size: isize
}

#[derive(Debug)]
struct TextureMetadata {
    width: i32,
    height: i32,
    depth: i32
}

#[derive(Debug)]
struct Buffer {
    obj: u32
}

#[derive(Debug)]
struct Texture {
    obj: u32
}

#[derive(Debug)]
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
    renameIndex: i32,
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
    logicalResources: Arena<LogicalResource<'fg>>,
    passes: UnsafeCell<Vec<Pass<'fg>>>  // must be mutated while there are still references to LogicalResources outside
    // but that's okay, since there are no refs to passes leaving the FrameGraph
}

impl<'fg> FrameGraph<'fg>
{
    fn make() -> FrameGraph<'fg> {
        FrameGraph { logicalResources: Arena::new(), passes: UnsafeCell::new(Vec::new()) }
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

/*
Issue: the handles returned by the framegraph will borrow the framegraph (immutably)
So, as long as the handles are alive, no modifications can be done on the frame graph
i.e. can't compile while the handles are alive

Issue 2:
Can't add passes (mutable borrow) when 

Solutions: 
    1. handles should not borrow the framegraph
    2. unsafe code***
    3. RefCells?
    4. Cells?

*/