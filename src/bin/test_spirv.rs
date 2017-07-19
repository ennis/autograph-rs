extern crate autograph;

// The `vulkano` crate is the main crate that you must use to use Vulkan.
#[macro_use]
extern crate vulkano;
#[macro_use]
extern crate vulkano_shader_derive;
extern crate winit;
extern crate vulkano_win;
extern crate time;
extern crate pretty_env_logger;
extern crate glsl_to_spirv;

use std::path::Path;
use std::fs::File;
use std::io::Read;

use autograph::shader_preprocessor::preprocess_combined_shader_source;

const COMBINED_SHADER_PATH: &str = "data/shaders/DeferredGeometry450.glsl";

fn main()
{
    // load combined shader source
    pretty_env_logger::init().unwrap();
    let mut src = String::new();
    File::open(COMBINED_SHADER_PATH).unwrap().read_to_string(&mut src).unwrap();

    // preprocess
    let (stages, sources) = preprocess_combined_shader_source(&src, Path::new(COMBINED_SHADER_PATH), &[], &[]);



}