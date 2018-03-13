use gfx;
use std::cell::RefCell;

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
    texture: RefCell<gfx::RawTexture>,
}

// TODO
fn load_image() {}
