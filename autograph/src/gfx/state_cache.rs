
use gfx::{Uniforms,Scissors,VertexInput,FramebufferObject,GraphicsPipeline,Framebuffer};
use super::bind::{bind_target, bind_vertex_input, bind_uniforms, bind_graphics_pipeline, bind_scissors, SG_ALL};

// TODO move this into its own module
pub(super) struct StateCache
{
    uniforms: Option<Uniforms>,
    vertex_input: Option<VertexInput>,
    framebuffer: Option<*const FramebufferObject>,
    pipeline: Option<*const super::pipeline::inner::GraphicsPipeline>,
    scissors: Option<Scissors>,
}

impl StateCache
{
    pub fn new() -> StateCache {
        StateCache {
            uniforms: None,
            vertex_input: None,
            pipeline: None,
            framebuffer: None,
            scissors: None
        }
    }

    pub unsafe fn set_graphics_pipeline(&mut self, pipe: &GraphicsPipeline) {
        // same pipeline as before?
        if self.pipeline.map_or(true, |prev_pipe| prev_pipe != pipe.as_ref() as *const _) {
            // nope, bind it
            bind_graphics_pipeline(pipe, SG_ALL);
            self.pipeline = Some(pipe.as_ref() as *const _);
        }
    }

    pub unsafe fn set_uniforms(&mut self, uniforms: &Uniforms)
    {
        // TODO
        bind_uniforms(uniforms);
    }

    pub unsafe fn set_vertex_input(&mut self, vertex_input: &VertexInput)
    {
        // TODO
        bind_vertex_input(vertex_input);
    }

    pub unsafe fn set_target(&mut self, framebuffer: &Framebuffer, viewport: &[(f32, f32, f32, f32)]) {
        // same framebuffer as before?
        if self.framebuffer.map_or(true, |prev_framebuffer| prev_framebuffer != framebuffer.as_ref() as *const _) {
            // nope, bind it
            bind_target(framebuffer, viewport);
            self.framebuffer = Some(framebuffer.as_ref() as *const _);
        }
    }

    pub unsafe fn set_scissors(&mut self, scissors: &Scissors) {
        // TODO
        bind_scissors(scissors);
    }
}
