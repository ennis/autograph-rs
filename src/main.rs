#![feature(const_fn)]
#![feature(intrinsics)]
#![feature(box_syntax)]

extern crate typed_arena;
extern crate glutin;
extern crate gl;
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


use typed_arena::Arena;
use std::clone;
use gl::types::*;

mod framegraph;
mod gfx;
mod id_table;
mod scene_object;
mod aabb;
mod scene_loader;
mod cache;
mod unsafe_any;
mod mesh;
mod rc_cache;

// Remaining:
// - Camera stuff
// - Meshes
// - Materials
// - Load images (openimageio bindings?)
// - Test scene
// - imgui (vulkan backend???)

fn main() {
    pretty_env_logger::init().unwrap();

    info!("Testing log output");

    let event_loop = glutin::EventsLoop::new();
    let window = glutin::WindowBuilder::new()
        .with_title("Glutin test")
        .with_dimensions(640, 480)
        .with_gl_profile(glutin::GlProfile::Core)
        .with_gl(glutin::GlRequest::Specific(glutin::Api::OpenGl, (4, 5)))
        .build(&event_loop)
        .unwrap();

    unsafe {
        window.make_current()
    }.unwrap();

    unsafe {
        gl::load_with(|symbol| window.get_proc_address(symbol) as *const _);
        gl::ClearColor(0.0, 1.0, 0.0, 1.0);
    }

    // for posterity
    println!("Hello, world!");
    let mut running = true;

    // create a context
    let ctx = gfx::Context::new(&gfx::ContextConfig {
        default_upload_buffer_size: 3 * 1024 * 1024,
        max_frames_in_flight: 3
    });

    // create a texture bound to this context
    let tex = gfx::Texture::new(ctx.clone(), &gfx::TextureDesc {
        dimensions: gfx::TextureDimensions::Tex2D,
        format: gfx::TextureFormat::R8G8B8A8_UNORM,
        width: 640,
        height: 480,
        ..Default::default()
    });


    while running {
        // poll events
        event_loop.poll_events(|event| {
            match event {
                glutin::Event::WindowEvent { event: glutin::WindowEvent::Closed, .. } => {
                    running = false;
                }
                _ => ()
            }
        });

        // once all events in the queue are dispatched, render stuff
        unsafe {
            gl::Clear(gl::COLOR_BUFFER_BIT);
        }

        // swap buffers
        window.swap_buffers().unwrap();
    }
}
