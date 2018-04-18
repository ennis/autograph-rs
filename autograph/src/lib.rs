#![feature(const_fn)]
#![feature(intrinsics)]
#![feature(box_syntax)]
#![feature(plugin, custom_attribute)]
#![feature(trace_macros)]
#![feature(log_syntax)]
#![feature(const_unsafe_cell_new)]
#![feature(ascii_ctype)]
#![feature(macro_reexport)]


#[macro_use]
extern crate failure;
extern crate glutin;
extern crate typed_arena;
extern crate smallvec;
extern crate libc;
extern crate assimp_sys;
extern crate nalgebra;
extern crate alga;
extern crate regex;
extern crate pretty_env_logger;
#[macro_use]
extern crate log;
#[macro_use]
#[macro_reexport(lazy_static)]
extern crate lazy_static;
#[macro_use]
extern crate bitflags;
extern crate itertools;
extern crate petgraph;
extern crate num_traits;
extern crate url;
extern crate notify;
extern crate rspirv;
extern crate spirv_headers;
extern crate shaderc;
#[macro_use]
extern crate derive_deref;
#[macro_use]
#[macro_reexport(offset_of)]
extern crate memoffset;

// Hack for autograph-derive
/*#[macro_export]
#[doc(hidden)]
macro_rules! vertex_type_offset_of {
    ($father:ty, $($field:tt)+) => { offset_of!($father,$($field)+) }
}*/


//pub mod rendergraph;
pub mod framegraph;
pub mod id_table;
pub mod scene_object;
pub mod aabb;
pub mod cache;
pub mod gl;
pub mod gfx;
pub mod scene_loader;
pub mod mesh;
pub mod camera;
pub mod image;
pub mod lazy;
pub mod rect_transform;