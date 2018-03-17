use gl;
use gl::types::*;
use super::state_group::*;
use super::context::Context;
use gfx;
use gfx::state_cache::StateCache;
use gfx::shader::{GraphicsShaderPipeline, UniformBinder};
use gfx::shader_interface::{ShaderInterface,ShaderInterfaceDesc};
use std::sync::Arc;
use std::ops::Deref;
use std::path::Path;
use failure::Error;

#[derive(Copy, Clone, Debug)]
pub struct VertexAttribute {
    pub slot: u32,
    pub ty: GLenum,
    pub size: u32,
    pub relative_offset: i32,
    pub normalized: bool,
}

pub(super) mod inner {
    use gl;
    use gl::types::*;
    use gfx::state_group::*;
    use gfx::Context;
    use gfx::shader::GraphicsShaderPipeline;

    pub struct GraphicsPipeline {
        // TODO fix public access
        pub gctx: Context,
        pub blend_states: [BlendState; 8], // TODO hardcoded limit
        pub rasterizer_state: RasterizerState,
        pub depth_stencil_state: DepthStencilState,
        pub shader_pipeline: Box<GraphicsShaderPipeline>,
        pub vao: GLuint,
        pub primitive_topology: GLenum,
    }

    impl ::std::fmt::Debug for GraphicsPipeline {
        fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
            unimplemented!()
        }
    }

    impl Drop for GraphicsPipeline {
        fn drop(&mut self) {
            unsafe {
                //gl::DeleteProgram(self.program);
                gl::DeleteVertexArrays(1, &mut self.vao);
            }
        }
    }


}

/// trait GraphicsPipeline: Clone
///     - bind(
/// struct UntypedGraphicsPipeline: GraphicsPipeline
/// struct TypedGraphicsPipeline<T>: UntypedGraphicsPipeline
///

#[derive(Clone,Debug)]
pub struct GraphicsPipeline(Arc<inner::GraphicsPipeline>);

impl Deref for GraphicsPipeline
{
    type Target = Arc<inner::GraphicsPipeline>;
    fn deref(&self) -> &Arc<inner::GraphicsPipeline> {
        &self.0
    }
}

impl GraphicsPipeline
{
    /// Sets the OpenGL pipeline states (all of them with the exception of uniform bindings)
    pub(super) unsafe fn bind<'a>(&'a self, state_cache: &mut StateCache) -> &'a UniformBinder {
        state_cache.set_graphics_pipeline(self);
        self.shader_pipeline.bind()
    }
}

/// A graphics pipeline with an attached interface type
//pub struct TypedGraphicsPipeline<T: ShaderInterface>(Arc<inner::GraphicsPipeline>);

/// The topology of the primitives passed to the GPU in vertex buffers.
#[derive(Debug)]
pub enum PrimitiveTopology {
    Triangle,
    Line,
    Point,
    Patch
}

/// Builder for graphics pipelines
pub struct GraphicsPipelineBuilder {
    blend_states: [BlendState; 8], // TODO hardcoded limit
    rasterizer_state: RasterizerState,
    depth_stencil_state: DepthStencilState,
    shader_pipeline: Option<Box<gfx::shader::GraphicsShaderPipeline>>,
    input_layout: Option<Vec<VertexAttribute>>,
    primitive_topology: GLenum,
}

unsafe fn gen_vertex_array(attribs: &[VertexAttribute]) -> GLuint {
    let mut vao = 0;
    gl::CreateVertexArrays(1, &mut vao);

    debug!("attribs: {:#?}", attribs);

    for (i, a) in attribs.iter().enumerate() {
        gl::EnableVertexArrayAttrib(vao, i as u32);
        gl::VertexArrayAttribFormat(
            vao,
            i as u32,
            a.size as i32,
            a.ty,
            a.normalized as u8,
            a.relative_offset as u32,
        );
        gl::VertexArrayAttribBinding(vao, i as u32, a.slot);
    }

    vao
}

#[derive(Debug, Fail)]
pub enum GraphicsPipelineBuildError {
    #[fail(display = "Input layout was not specified")]
    MissingInputLayout,
    #[fail(display = "Shader pipeline was not specified")]
    MissingShaderPipeline,
}

impl GraphicsPipelineBuilder {
    /// Starts building a new graphics pipeline.
    pub fn new() -> Self {
        GraphicsPipelineBuilder {
            blend_states: Default::default(),
            rasterizer_state: Default::default(),
            depth_stencil_state: Default::default(),
            shader_pipeline: None,
            input_layout: None,
            primitive_topology: gl::TRIANGLES,
        }
    }

    pub fn with_shader_pipeline(mut self, shader_pipeline: Box<gfx::shader::GraphicsShaderPipeline>) -> Self {
        self.shader_pipeline = Some(shader_pipeline);
        self
    }

    pub fn with_all_blend_states(mut self, blend_state: &BlendState) -> Self {
        self.blend_states = [*blend_state; 8];
        self
    }

    pub fn with_blend_state(mut self, index: usize, blend_state: &BlendState) -> Self {
        self.blend_states[index] = *blend_state;
        self
    }

    pub fn with_input_layout<VA: Into<Vec<VertexAttribute>>>(mut self, attribs: VA) -> Self {
        self.input_layout = Some(attribs.into());
        self
    }

    pub fn with_rasterizer_state(mut self, rasterizer_state: &RasterizerState) -> Self {
        self.rasterizer_state = *rasterizer_state;
        self
    }

    pub fn with_depth_stencil_state(mut self, depth_stencil_state: &DepthStencilState) -> Self {
        self.depth_stencil_state = *depth_stencil_state;
        self
    }

    pub fn with_primitive_topology(mut self, primitive_topology: GLuint) -> Self {
        self.primitive_topology = primitive_topology;
        self
    }

    pub fn build(self, gctx: &Context) -> Result<GraphicsPipeline, Error> {
        let vao =
            unsafe { gen_vertex_array(&self.input_layout.ok_or(GraphicsPipelineBuildError::MissingInputLayout)?) };

        Ok(GraphicsPipeline(Arc::new(inner::GraphicsPipeline {
            depth_stencil_state: self.depth_stencil_state,
            rasterizer_state: self.rasterizer_state,
            blend_states: self.blend_states,
            vao,
            shader_pipeline: self.shader_pipeline.ok_or(GraphicsPipelineBuildError::MissingShaderPipeline)?,
            primitive_topology: self.primitive_topology,
            gctx: gctx.clone(),
        })))
    }
}

