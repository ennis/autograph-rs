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

#[derive(FrameGraphResources)]
struct DeferredDebug
{
    #[access(read)]
    diffuse: gfx::Texture2D,
    #[access(read)]
    normals: gfx::Texture2D,
    #[access(read)]
    material_id: gfx::Texture2D,
    #[access(read)]
    depth: gfx::Texture2D,
}

impl FrameGraphCallbacks for DeferredDebug
{
    fn execute<'pass>(frame: &'pass gfx::Frame, target: &'pass Arc<gfx::Framebuffer>) {

    }
}

pub use self::DeferredDebug::*;

