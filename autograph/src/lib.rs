#![feature(const_fn)]
#![feature(intrinsics)]
#![feature(box_syntax)]
#![feature(plugin, custom_attribute)]
#![feature(trace_macros)]
#![feature(log_syntax)]
#![feature(const_unsafe_cell_new)]
#![feature(ascii_ctype)]
#![feature(use_extern_macros)]
#![feature(iterator_find_map)]

#[macro_use]
extern crate failure;
extern crate alga;
extern crate assimp_sys;
extern crate glutin;
extern crate libc;
extern crate nalgebra;
extern crate pretty_env_logger;
extern crate regex;
extern crate smallvec;
extern crate typed_arena;
#[macro_use]
extern crate log;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate bitflags;
extern crate itertools;
extern crate notify;
extern crate num_traits;
extern crate petgraph;
extern crate rspirv;
extern crate shaderc;
extern crate spirv_headers as spirv;
extern crate url;
#[macro_use]
extern crate derive_deref;
#[macro_use]
extern crate memoffset;

// Hack for autograph-derive
/*#[macro_export]
#[doc(hidden)]
macro_rules! vertex_type_offset_of {
    ($father:ty, $($field:tt)+) => { offset_of!($father,$($field)+) }
}*/

//pub mod rendergraph;
pub mod aabb;
pub mod cache;
pub mod camera;
pub mod framegraph;
pub mod gfx;
pub mod gl;
pub mod id_table;
pub mod image;
pub mod lazy;
pub mod mesh;
pub mod rect_transform;
pub mod scene_loader;
pub mod scene_object;
pub use memoffset::offset_of;