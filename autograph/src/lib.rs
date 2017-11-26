#![feature(const_fn)]
#![feature(intrinsics)]
#![feature(box_syntax)]
#![feature(plugin, custom_attribute)]
#![feature(trace_macros)]
#![feature(log_syntax)]
#![feature(const_unsafe_cell_new)]

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
extern crate lazy_static;
#[macro_use]
extern crate bitflags;
#[macro_use]
extern crate itertools;
extern crate petgraph;
extern crate num_traits;
extern crate url;
extern crate notify;
extern crate imgui;

pub mod rendergraph;
pub mod framegraph;
pub mod id_table;
pub mod scene_object;
pub mod aabb;
pub mod cache;
pub mod shader_preprocessor;
pub mod gl;
pub mod gfx;
pub mod scene_loader;
pub mod mesh;
pub mod camera;
pub mod image;
pub mod shader_compiler;
pub mod lazy;
