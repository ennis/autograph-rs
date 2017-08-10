use petgraph::*;
use petgraph::graph::*;
use super::*;

pub struct CompiledGraph<'a>
{
    allocator: &'a mut FrameGraphAllocator,
    graph: FrameGraph,
    execution_plan: Vec<NodeIndex>
}

impl<'a> CompiledGraph<'a>
{
    // Should the compiled graph be consumed during the frame? or can it outlive a frame?
    pub fn new<'b>(graph: FrameGraph, execution_plan: Vec<NodeIndex>, allocator: &'b mut FrameGraphAllocator) -> CompiledGraph<'b> {
        CompiledGraph {
            allocator, graph, execution_plan
        }
    }

    // Q: Should execute consume the compiled graph?
    // => be able to reuse the execution plan from frame to frame
    pub fn execute(self) {
        // Go through the execution plan and call the execute() closure
        unimplemented!()
    }

    pub fn get_alloc_for_resource(&self, node: NodeIndex) -> &Alloc {
        // fetch node
        // get resource index
        // lookup resource index in allocator.allocations
        unimplemented!()
    }
}
