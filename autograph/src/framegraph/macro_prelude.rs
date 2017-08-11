//! This module contains the support types and functions used internally
//! in the gfx_pass! proc macro, implemented in autograph_codegen
//! # See also
//! The autograph_codegen crate
//!

use std::mem;
use std::path::Path;
use super::*;
use gfx;

#[doc(hidden)]
pub trait ToResourceInfo {
    fn to_resource_info(&self) -> ResourceInfo;
    fn from_resource_info(ri: &ResourceInfo) -> Option<Self>
    where
        Self: Sized;
}

#[doc(hidden)]
pub struct Texture2DInit {
    pub usage: ResourceUsage,
    pub format: gfx::TextureFormat,
    pub width: u32,
    pub height: u32,
    pub sample_count: u32,
    pub mip_map_count: gfx::MipMaps,
    pub options: gfx::TextureOptions,
}

impl Default for Texture2DInit {
    fn default() -> Texture2DInit {
        Texture2DInit {
            usage: ResourceUsage::Default,
            format: gfx::TextureFormat::R8G8B8A8_SRGB,
            width: 512,
            height: 512,
            sample_count: 1,
            mip_map_count: gfx::MipMaps::Count(1),
            options: gfx::TextureOptions::empty(),
        }
    }
}

// TODO: other initializers
impl ToResourceInfo for Texture2DInit {
    fn to_resource_info(&self) -> ResourceInfo {
        ResourceInfo::Texture {
            desc: gfx::TextureDesc {
                dimensions: gfx::TextureDimensions::Tex2D,
                format: self.format,
                width: self.width,
                height: self.height,
                depth: 1,
                sample_count: self.sample_count,
                mip_map_count: self.mip_map_count,
                options: self.options,
            },
        }
    }

    fn from_resource_info(ri: &ResourceInfo) -> Option<Self> {
        if let &ResourceInfo::Texture { ref desc } = ri {
            Some(Texture2DInit {
                usage: ResourceUsage::Default,
                format: desc.format,
                width: desc.width,
                height: desc.height,
                sample_count: desc.sample_count,
                mip_map_count: desc.mip_map_count,
                options: desc.options,
            })
        } else {
            None
        }
    }
}


#[doc(hidden)]
pub struct BufferArrayInit<T: gfx::BufferData + ?Sized> {
    pub inner: BufferArrayInitInner,
    _phantom: PhantomData<T>,
}

#[doc(hidden)]
pub struct BufferArrayInitInner {
    pub usage: ResourceUsage,
    pub len: usize,
}

impl<T: gfx::BufferData + ?Sized> ToResourceInfo for BufferArrayInit<T> {
    fn to_resource_info(&self) -> ResourceInfo {
        ResourceInfo::Buffer {
            byte_size: self.inner.len * mem::size_of::<T::Element>(),
        }
    }

    fn from_resource_info(ri: &ResourceInfo) -> Option<Self> {
        unimplemented!()
    }
}

#[doc(hidden)]
pub fn alloc_as_texture(alloc: &Alloc) -> Option<&Arc<gfx::Texture>> {
    match alloc {
        &Alloc::Texture { ref tex } => Some(tex),
        _ => None,
    }
}

#[doc(hidden)]
pub fn alloc_as_buffer_slice(alloc: &Alloc) -> Option<&gfx::BufferSliceAny> {
    unimplemented!()
}

#[doc(hidden)]
pub struct GraphicsPipelineInit {
    pub path: &'static str,
    pub depth_stencil_state: gfx::DepthStencilState,
    pub rasterizer_state: gfx::RasterizerState,
    pub blend_state: gfx::BlendState,
}

impl Default for GraphicsPipelineInit {
    fn default() -> GraphicsPipelineInit {
        GraphicsPipelineInit {
            path: "",
            depth_stencil_state: Default::default(),
            rasterizer_state: Default::default(),
            blend_state: Default::default()
        }
    }
}

impl GraphicsPipelineInit {
    #[doc(hidden)]
    pub fn to_graphics_pipeline(&self, ctx: &Arc<gfx::Context>) -> Arc<gfx::GraphicsPipeline>
    {
        let compiled_shaders = ::shader_compiler::compile_shaders_from_combined_source(Path::new(self.path)).unwrap();
        Arc::new(gfx::GraphicsPipelineBuilder::new()
            .with_vertex_shader(compiled_shaders.vertex)
            .with_fragment_shader(compiled_shaders.fragment)
            .with_geometry_shader(compiled_shaders.geometry)
            .with_tess_eval_shader(compiled_shaders.tess_eval)
            .with_tess_control_shader(compiled_shaders.tess_control)
            .with_primitive_topology(compiled_shaders.primitive_topology)
            .with_rasterizer_state(&self.rasterizer_state)
            .with_depth_stencil_state(&self.depth_stencil_state)
            .with_all_blend_states(&self.blend_state)
            .with_input_layout(&compiled_shaders.input_layout)
            .build(ctx).map_err(|e| match e {
                gfx::GraphicsPipelineBuildError::ProgramLinkError(ref log) => {
                    println!("Program link error: {}", log);
                }
            })
            .unwrap())
    }
}
