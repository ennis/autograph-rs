// Pipeline =
//  - shaders
//  - (parsed from shader) input layout
//  - (optional) uniform slot reflection
//  - (optional) outputs reflection
//  - (optional, parsed from shader) samplers
//  - Rasterizer state, Blend states, etc.
// Binary program can be saved to a file

use gl;
use gl::types::*;
use super::state_group::*;
use super::context::Context;
use std::sync::Arc;
use std::ops::Deref;

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

    # [derive(Debug)]
    pub struct GraphicsPipeline {
        // TODO fix public access
        pub gctx: Context,
        pub blend_states: [BlendState; 8], // TODO hardcoded limit
        pub rasterizer_state: RasterizerState,
        pub depth_stencil_state: DepthStencilState,
        pub vao: GLuint,
        pub program: GLuint,
        pub primitive_topology: GLenum,
    }

    impl Drop for GraphicsPipeline {
        fn drop(&mut self) {
            unsafe {
                gl::DeleteProgram(self.program);
                gl::DeleteVertexArrays(1, &mut self.vao);
            }
        }
    }
}

#[derive(Clone,Debug)]
pub struct GraphicsPipeline(Arc<inner::GraphicsPipeline>);

impl Deref for GraphicsPipeline
{
    type Target = Arc<inner::GraphicsPipeline>;
    fn deref(&self) -> &Arc<inner::GraphicsPipeline> {
        &self.0
    }
}

#[derive(Debug)]
pub enum PrimitiveTopology {
    Triangle,
    Line,
}

pub struct GraphicsPipelineBuilder<'a> {
    // with_vertex_shader
    // with_fragment_shader
    // with_combined_shader_source
    // with_input_layout
    // with_rasterizer_state
    // with_depth_stencil_state
    // with_all_blend_states(...)
    // with_blend_state
    blend_states: [BlendState; 8], // TODO hardcoded limit
    rasterizer_state: RasterizerState,
    depth_stencil_state: DepthStencilState,
    vertex_shader: Option<Shader>,
    fragment_shader: Option<Shader>,
    geometry_shader: Option<Shader>,
    tess_control_shader: Option<Shader>,
    tess_eval_shader: Option<Shader>,
    input_layout: Option<&'a [VertexAttribute]>,
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

pub struct Shader {
    obj: GLuint,
    stage: GLenum,
}

impl Shader {
    pub fn compile(source: &str, stage: GLenum) -> Result<Shader, String> {
        unsafe {
            let obj = gl::CreateShader(stage);
            let srcs = [source.as_ptr() as *const i8];
            let lens = [source.len() as GLint];
            gl::ShaderSource(
                obj,
                1,
                &srcs[0] as *const *const i8,
                &lens[0] as *const GLint,
            );
            gl::CompileShader(obj);
            let mut status: GLint = 0;
            let mut log_size: GLint = 0;
            gl::GetShaderiv(obj, gl::COMPILE_STATUS, &mut status);
            gl::GetShaderiv(obj, gl::INFO_LOG_LENGTH, &mut log_size);
            if status != gl::TRUE as GLint {
                error!("Error compiling shader");
                let mut log_buf: Vec<u8> = Vec::with_capacity(log_size as usize);
                gl::GetShaderInfoLog(
                    obj,
                    log_size,
                    &mut log_size,
                    log_buf.as_mut_ptr() as *mut i8,
                );
                log_buf.set_len(log_size as usize);
                gl::DeleteShader(obj);
                Err(String::from_utf8(log_buf).unwrap())
            } else {
                Ok(Shader { stage, obj })
            }
        }
    }

    // TODO: from_spirv, from_binary?
    // (runtime) reflection for spirv
}

impl Drop for Shader {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteShader(self.obj);
        }
    }
}

fn link_program(obj: GLuint) -> Result<(), String> {
    unsafe {
        gl::LinkProgram(obj);
        let mut status: GLint = 0;
        let mut log_size: GLint = 0;
        gl::GetProgramiv(obj, gl::LINK_STATUS, &mut status);
        gl::GetProgramiv(obj, gl::INFO_LOG_LENGTH, &mut log_size);
        //trace!("LINK_STATUS: log_size: {}, status: {}", log_size, status);
        if status != gl::TRUE as GLint {
            let mut log_buf: Vec<u8> = Vec::with_capacity(log_size as usize);
            gl::GetProgramInfoLog(
                obj,
                log_size,
                &mut log_size,
                log_buf.as_mut_ptr() as *mut i8,
            );
            log_buf.set_len(log_size as usize);
            Err(String::from_utf8(log_buf).unwrap())
        } else {
            Ok(())
        }
    }
}

pub enum GraphicsPipelineBuildError {
    ProgramLinkError(String),
}

impl<'a> GraphicsPipelineBuilder<'a> {
    pub fn new() -> Self {
        GraphicsPipelineBuilder {
            blend_states: Default::default(),
            rasterizer_state: Default::default(),
            depth_stencil_state: Default::default(),
            tess_eval_shader: None,
            tess_control_shader: None,
            fragment_shader: None,
            vertex_shader: None,
            geometry_shader: None,
            input_layout: None,
            primitive_topology: gl::TRIANGLES,
        }
    }

    pub fn with_vertex_shader(mut self, shader: Shader) -> Self {
        self.vertex_shader = Some(shader);
        self
    }

    pub fn with_fragment_shader(mut self, shader: Shader) -> Self {
        self.fragment_shader = Some(shader);
        self
    }

    pub fn with_tess_control_shader(mut self, shader: Option<Shader>) -> Self {
        self.tess_control_shader = shader;
        self
    }

    pub fn with_tess_eval_shader(mut self, shader: Option<Shader>) -> Self {
        self.tess_eval_shader = shader;
        self
    }

    pub fn with_geometry_shader(mut self, shader: Option<Shader>) -> Self {
        self.geometry_shader = shader;
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

    pub fn with_input_layout<'b: 'a>(mut self, attribs: &'b [VertexAttribute]) -> Self {
        self.input_layout = Some(attribs);
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

    pub fn build(self, gctx: &Context) -> Result<GraphicsPipeline, GraphicsPipelineBuildError> {
        let vao =
            unsafe { gen_vertex_array(self.input_layout.expect("No input layout specified!")) };

        let program = unsafe { gl::CreateProgram() };

        unsafe {
            gl::AttachShader(
                program,
                self.vertex_shader
                    .expect("must specify a vertex shader")
                    .obj,
            );
            gl::AttachShader(
                program,
                self.fragment_shader
                    .expect("must specify a fragment shader")
                    .obj,
            );
            if let Some(s) = self.geometry_shader {
                gl::AttachShader(program, s.obj);
            }
            if let Some(s) = self.tess_control_shader {
                gl::AttachShader(program, s.obj);
            }
            if let Some(s) = self.tess_eval_shader {
                gl::AttachShader(program, s.obj);
            }
        }
        // link shaders
        link_program(program)
            .map_err(|log| GraphicsPipelineBuildError::ProgramLinkError(log))?;

        Ok(GraphicsPipeline(Arc::new(inner::GraphicsPipeline {
            depth_stencil_state: self.depth_stencil_state,
            rasterizer_state: self.rasterizer_state,
            blend_states: self.blend_states,
            vao,
            program,
            primitive_topology: self.primitive_topology,
            gctx: gctx.clone(),
        })))
    }
}

