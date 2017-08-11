use gfx;
use cache::{Cache, CacheTrait, ReloadReason};
use std::cell::RefCell;
use std::sync::Arc;

pub enum TargetFormat {
    // Texture format = the one that needs the least conversion from the file
    Auto,
    // May fail
    Required(gfx::TextureFormat),
}

/// A cached image
struct CachedImage {
    data: Vec<u8>,
    format: gfx::TextureFormat,
    texture: RefCell<Arc<gfx::Texture>>,
}

fn load_image() {}
