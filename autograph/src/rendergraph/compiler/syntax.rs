//! AST for the rendergraph text format

// TODO Components, Passes, Metadata
use gfx::{BlendState, MAX_BLEND_STATES, DepthStencilState, RasterizerState, PrimitiveTopology};

#[derive(Debug)]
pub struct GraphicsPass {
    pub primitive_topology: PrimitiveTopology,
    pub blend_states: [BlendState; MAX_BLEND_STATES],
    pub depth_stencil_state: DepthStencilState,
    pub rasterizer_state: RasterizerState,
    pub vs_entry: String,
    pub fs_entry: String,
    pub gs_entry: Option<String>,
    pub tcs_entry: Option<String>,
    pub tes_entry: Option<String>,
}

#[derive(Debug)]
pub struct ComputePass {
    pub entry: String,
    pub local_size: [u32; 3]
}

#[derive(Debug)]
pub struct Pass {
    pub name: String,
    pub directives: Vec<PassDirective>
}


#[derive(Debug)]
pub enum PassDirective {
    PrimitiveTopology(PrimitiveTopology),
    DepthTest(bool),
    VertexShader(String),
    FragmentShader(String),
    GeometryShader(String),
    TessControlShader(String),
    TessEvalShader(String),
    ComputeShader(String),
}