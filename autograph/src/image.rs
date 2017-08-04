use gfx;
use rc_cache::{CacheTrait, Cache, ReloadReason};
use std::cell::RefCell;
use std::rc::Rc;

pub enum TargetFormat
{
    // Texture format = the one that needs the least conversion from the file
    Auto,
    // May fail
    Required(gfx::TextureFormat)
}

/// A cached image
struct CachedImage
{
    data: Vec<u8>,
    format: gfx::TextureFormat,
    texture: RefCell<Rc<gfx::Texture>>
}

fn load_image()
{

}

