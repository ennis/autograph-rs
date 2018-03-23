use gfx;
use std::fs::File;
use gl;
use gl::types::*;
use std::io::Read;
use failure::Error;
use std::path::{Path, PathBuf};
use regex::Regex;
use gfx::pipeline::VertexAttribute;
use gfx::pipeline::GraphicsPipelineBuilder;
use gfx::shader;
use gfx::shader_interface;
use gfx::shader::DefaultUniformBinder;

