use petgraph::*;
use petgraph::graph::*;
use super::*;
use super::allocator::{FrameGraphAllocator,Alloc};

pub struct CompiledGraph<'a>
{
    allocator: &'a mut FrameGraphAllocator,
    graph: FrameGraph,
    execution_plan: Vec<NodeIndex>
}

impl<'a> CompiledGraph<'a>
{
    pub fn new<'b>(graph: FrameGraph, execution_plan: Vec<NodeIndex>, allocator: &'b mut FrameGraphAllocator) -> CompiledGraph<'b> {
        CompiledGraph {
            allocator, graph, execution_plan
        }
    }

    pub fn execute(self) {
        // Go through the execution plan and call the execute() closure
        unimplemented!()
    }
}
