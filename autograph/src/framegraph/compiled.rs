//! Compiled frame graphs
//!
use super::*;

/// Compiled frame graph that is ready to execute
pub struct CompiledGraph<'a> {
    allocator: &'a mut FrameGraphAllocator,
    graph: FrameGraph<'a>,
    execution_plan: Vec<NodeIndex>,
}

impl<'a> CompiledGraph<'a> {
    // Should the compiled graph be consumed during the frame? or can it outlive a frame?
    pub fn new(
        graph: FrameGraph<'a>,
        execution_plan: Vec<NodeIndex>,
        allocator: &'a mut FrameGraphAllocator,
    ) -> CompiledGraph<'a> {
        CompiledGraph {
            allocator,
            graph,
            execution_plan,
        }
    }

    // Q: Should execute consume the compiled graph?
    // => be able to reuse the execution plan from frame to frame
    pub fn execute(self, frame: &gfx::Frame) {
        // Go through the execution plan and call the execute() closure
        for node in self.execution_plan.iter() {
            let node = self.graph.graph.node_weight(*node).unwrap();
            match node {
                &Node::Pass {
                    ref name,
                    ref execute,
                } => {
                    execute(frame, &self);
                }
                _ => continue,
            }
        }
    }

    pub fn get_alloc_for_resource(&self, node: NodeIndex) -> Option<&Alloc> {
        // fetch node
        // get resource index
        // lookup resource index in allocator.allocations
        let node = self.graph.graph.node_weight(node).unwrap();
        if let &Node::Resource { index, .. } = node {
            let res = &self.graph.resources[index.index()];
            Some(
                &self.allocator.allocations
                    [res.alloc_index.get().expect("resource was not allocated").index()],
            )
        } else {
            None
        }
    }
}
