//! Compiled frame graphs
//!
use super::*;

/// Context for evaluating a frame graph (i.e. send the actual rendering commands to the GPU)
pub struct ExecutionContext<'node> {
    fg: FrameGraph<'node>,
    allocator: &'node mut FrameGraphAllocator,
    toposort: Vec<NodeIndex>,
}

impl<'a> ExecutionContext<'a> {
    // Should the compiled graph be consumed during the frame? or can it outlive a frame?
    pub fn new(
        fg: FrameGraph<'a>,
        allocator: &'a mut FrameGraphAllocator,
        toposort: Vec<NodeIndex>,
    ) -> ExecutionContext<'a> {
        ExecutionContext {
            fg,
            allocator,
            toposort,
        }
    }

    // Q: Should execute consume the compiled graph?
    // => be able to reuse the execution plan from frame to frame
    pub fn execute(self, frame: &gfx::Frame) {
        // Go through the execution plan and call the execute() closure
        for node in self.toposort.iter() {
            let node = self.fg.graph.node_weight(*node).unwrap();
            match node {
                &Node::RenderPass {
                    ref name,
                    ref callbacks,
                } => {
                    callbacks.execute(frame, &self);
                }
                _ => continue,
            }
        }
    }

    pub fn texture_resource(&self, res: ResourceVersion) -> gfx::TextureAny {
        let aliasedres = self.aliased_resource(res);
        if let &AliasedResource::Texture { ref tex } = aliasedres {
            tex.clone()
        } else {
            panic!("not a valid texture resource")
        }
    }

    pub fn aliased_resource(&self, res: ResourceVersion) -> &AliasedResource {
        // fetch node
        // get resource index
        // lookup resource index in allocator.allocations
        let node = self.fg.graph.node_weight(res.0).unwrap();
        if let &Node::Resource { index, .. } = node {
            let res = &self.fg.resources[index.index()];
            &self.allocator.allocations[res
                                            .aliased_index
                                            .get()
                                            .expect("resource was not allocated")
                                            .index()]
        } else {
            panic!("not a valid resource")
        }
    }
}
