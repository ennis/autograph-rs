//! Deferred debug pass
//!
//!
use autograph::gfx;
use autograph::scene_object::SceneObjects;
use autograph::camera::Camera;
use nalgebra::*;
use std::sync::Arc;

#[derive(Copy, Clone, Debug)]
pub enum DeferredDebugBuffer {
    Diffuse,
    Normals,
    MaterialID,
    Depth,
}

gfx_pass!{
pass DeferredDebug(frame: &'pass gfx::Frame, target: &'pass Arc<gfx::Framebuffer>, debug: DeferredDebugBuffer)
{
    read {
        texture2D diffuse {},
        texture2D normals {},
        texture2D material_id {},
        texture2D depth {}
    }
    write {
    }
    execute
    {
        frame.clear_framebuffer_color(target, 0, &[0.125, 0.125, 0.48, 1.0]);
    }
}
}

#[derive(GpuTaskResources )]
struct DeferredDebug
{
    //-------------------------
    // automatically bound
    #[usage(texture)]
    #[id("gbuffer:diffuse")]
    diffuse: gfx::Texture2D,
    #[usage(texture)]
    #[id("gbuffer:normal")]
    normals: gfx::Texture2D,
    #[usage(texture)]
    #[id("gbuffer:material_id")]
    material_id: gfx::Texture2D,
    #[usage(texture)]
    #[id("gbuffer:depth")]
    depth: gfx::Texture2D,
    #[usage(render_target, index="0")]
    target: gfx::RenderTarget,


    //-------------------------
    // automatically bound from a file, and statically checked
    #[path="data/shaders/deferredDebug.glsl"]
    pipeline: gfx::GraphicsPipeline,
}

impl FrameGraphCallbacks for DeferredDebug
{
    fn execute(frame: &gfx::Frame, target: &gfx::Framebuffer) {

    }
}

pub use self::DeferredDebug::*;

