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

pub use self::DeferredDebug::*;

