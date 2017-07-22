#![feature(const_fn)]
#![feature(intrinsics)]
#![feature(box_syntax)]

extern crate glutin;
extern crate gl;
extern crate typed_arena;
extern crate smallvec;
extern crate libc;
extern crate assimp;
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
extern crate itertools;
extern crate petgraph;

use std::iter;
use std::sync::Arc;
use typed_arena::Arena;
use std::clone;

pub mod framegraph;
pub mod id_table;
pub mod scene_object;
pub mod aabb;
//mod scene_loader;
pub mod cache;
pub mod unsafe_any;
//mod mesh;
pub mod rc_cache;
pub mod shader_preprocessor;
pub mod gfx;