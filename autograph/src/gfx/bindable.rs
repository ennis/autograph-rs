use super::buffer::*;
use super::texture::*;
use std::rc::Rc;
use super::buffer_data::BufferData;

/// Represents a resource that can be bound to the pipeline,
/// and whose lifetime can be extended
pub trait PipelineResource
{}