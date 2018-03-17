use gfx;
use std::fs::File;
use gl;
use gl::types::*;
use std::io::Read;
use failure::Error;
use std::path::{Path, PathBuf};
use regex::Regex;
use gfx::pipeline::VertexAttribute;
use gfx::shader;
use gfx::shader_interface;

bitflags! {
    #[derive(Default)]
    pub struct PipelineStages: u32 {
        ///
        const PS_VERTEX = (1 << 0);
        const PS_GEOMETRY = (1 << 1);
        const PS_FRAGMENT = (1 << 2);
        const PS_TESS_CONTROL = (1 << 3);
        const PS_TESS_EVAL = (1 << 4);
        const PS_COMPUTE = (1 << 5);
    }
}

struct SourceMapEntry {
    index: i32,
    path: Option<PathBuf>,
}

struct IncludeFile<'a> {
    parent: Option<&'a IncludeFile<'a>>,
    path: &'a Path,
}

/// Preprocesses a combined GLSL source file: extract the additional informations in the custom pragmas
/// and returns the result in (last_seen_version, enabled_pipeline_stages, input_layout, topology)
fn preprocess_shader_internal<'a>(
    preprocessed: &mut String,
    source: &str,
    last_seen_version: &mut Option<i32>,
    enabled_pipeline_stages: &mut PipelineStages,
    input_layout: &mut Option<Vec<VertexAttribute>>,
    topology: &mut Option<GLenum>,
    this_file: &IncludeFile<'a>,
    source_map: &mut Vec<SourceMapEntry>,
) -> i32 {
    lazy_static! {
        static ref SHADER_STAGE_PRAGMA_RE: Regex = Regex::new(r#"^stages\s*\(\s*(\w+(?:\s*,\s*\w+)*)\s*\)\s*?$"#).unwrap();
        static ref INPUT_LAYOUT_PRAGMA_RE: Regex = Regex::new(r#"^input_layout\s*\(\s*(\w+(?:\s*,\s*\w+)*)\s*\)\s*?$"#).unwrap();
        static ref PRIMITIVE_TOPOLOGY_PRAGMA_RE: Regex = Regex::new(r#"^primitive_topology\s*\(\s*(\w+)\s*\)\s*?$"#).unwrap();
        static ref INCLUDE_RE: Regex = Regex::new(r#"^\s*#include\s+"(.*)"\s*?$"#).unwrap();
        static ref VERSION_RE: Regex = Regex::new(r#"^\s*#version\s+([0-9]*)\s*?$"#).unwrap();
        static ref PRAGMA_RE: Regex = Regex::new(r#"^\s*#pragma\s+(.*)\s*?$"#).unwrap();
    }

    let this_file_index = source_map.len() as i32;
    source_map.push(SourceMapEntry {
        index: this_file_index,
        path: Some(this_file.path.to_owned()),
    });

    let dir = this_file.path.parent().unwrap();
    let mut cur_line = 1;
    let mut should_output_line_directive = false;
    let mut num_errors = 0;

    'line: for line in source.lines() {
        if let Some(captures) = INCLUDE_RE.captures(line) {
            let mut inc_path = dir.to_owned();
            inc_path.push(&captures[1]);
            debug!("include path = {:?}", &inc_path);

            match File::open(&inc_path) {
                Ok(mut file) => {
                    let mut text = String::new();
                    file.read_to_string(&mut text);
                    let next_include = IncludeFile {
                        path: &inc_path,
                        parent: Some(&this_file),
                    };
                    preprocess_shader_internal(
                        preprocessed,
                        &text,
                        last_seen_version,
                        enabled_pipeline_stages,
                        input_layout,
                        topology,
                        &next_include,
                        source_map,
                    );
                }
                Err(e) => {
                    error!(
                        "{:?}({:?}): Could not open include file {:?}: {:?}",
                        this_file.path,
                        cur_line,
                        inc_path,
                        e
                    );
                    num_errors += 1;
                }
            };

            should_output_line_directive = true;
            cur_line += 1;
            continue;
            //debug!();
        } else if let Some(captures) = VERSION_RE.captures(line) {
            match captures[1].parse::<i32>() {
                Ok(ver) => if let Some(previous_ver) = *last_seen_version {
                    if previous_ver != ver {
                        warn!(
                            "{:?}({:?}): version differs from previously specified version ({:?}, was {:?})",
                            this_file.path,
                            cur_line,
                            previous_ver,
                            ver
                        );
                        *last_seen_version = Some(ver);
                    }
                } else {
                    *last_seen_version = Some(ver);
                },
                Err(_err) => {
                    error!(
                        "{:?}({:?}): Malformed version directive: \" {:?} \"",
                        this_file.path,
                        cur_line,
                        line
                    );
                    num_errors += 1;
                }
            }
            should_output_line_directive = true;
            cur_line += 1;
        } else if let Some(captures) = PRAGMA_RE.captures(line) {
            debug!("Pragma directive");
            let pragma_str = &captures[1];
            if let Some(captures) = SHADER_STAGE_PRAGMA_RE.captures(pragma_str) {
                let stages = &captures[1];
                for stage in stages.split(",").map(|s| s.trim()) {
                    match stage {
                        "vertex" => {
                            *enabled_pipeline_stages |= PS_VERTEX;
                        }
                        "fragment" => {
                            *enabled_pipeline_stages |= PS_FRAGMENT;
                        }
                        "geometry" => {
                            *enabled_pipeline_stages |= PS_GEOMETRY;
                        }
                        "tess_control" => {
                            *enabled_pipeline_stages |= PS_TESS_CONTROL;
                        }
                        "tess_eval" => {
                            *enabled_pipeline_stages |= PS_TESS_EVAL;
                        }
                        "compute" => {
                            *enabled_pipeline_stages |= PS_COMPUTE;
                        }
                        _ => {
                            error!(
                                "{:?}({:?}): Unknown shader stage in `#pragma stage` directive: `{:?}`. Expected `vertex`, `fragment`, `tess_control`, `tess_eval`, `geometry` or `compute`",
                                this_file.path,
                                cur_line,
                                stage
                            );
                            num_errors += 1;
                        }
                    }
                }
            } else if let Some(captures) = INPUT_LAYOUT_PRAGMA_RE.captures(pragma_str) {
                let entries = &captures[1];
                let mut iter = entries.split(",").map(|s| s.trim());
                //let mut index = 0;
                let mut layout = Vec::new();

                if input_layout.is_some() {
                    error!(
                        "{:?}({:?}): Duplicate input_layout directive",
                        this_file.path,
                        cur_line
                    );
                    num_errors += 1;
                    continue 'line; // ignore this directive
                }

                while let Some(fmt) = iter.next() {
                    let slot = iter.next().and_then(|slot| slot.parse::<u32>().ok());
                    let relative_offset = iter.next().and_then(|ro| ro.parse::<u32>().ok());

                    if slot.is_none() || relative_offset.is_none() {
                        error!(
                            "{:?}({:?}): Error parsing input_layout directive",
                            this_file.path,
                            cur_line
                        );
                        num_errors += 1;
                        continue 'line;
                    }

                    let attrib_format = match fmt {
                        "rgba32f" => (gl::FLOAT, 4, false),
                        "rgb32f" => (gl::FLOAT, 3, false),
                        "rg32f" => (gl::FLOAT, 2, false),
                        "r32f" => (gl::FLOAT, 1, false),
                        "rgba16_snorm" => (gl::SHORT, 4, true),
                        "rgb16_snorm" => (gl::SHORT, 3, true),
                        "rg16_snorm" => (gl::SHORT, 2, true),
                        "r16_snorm" => (gl::SHORT, 1, true),
                        "rgba8_unorm" => (gl::UNSIGNED_BYTE, 4, true),
                        "rgba8_snorm" => (gl::BYTE, 4, true),
                        _ => {
                            error!(
                                "{:?}({:?}): Error parsing input_layout directive (unsupported format?)",
                                this_file.path,
                                cur_line
                            );
                            num_errors += 1;
                            continue 'line;
                        }
                    };

                    layout.push(VertexAttribute {
                        ty: attrib_format.0,
                        relative_offset: relative_offset.unwrap() as i32,
                        slot: slot.unwrap(),
                        size: attrib_format.1,
                        normalized: attrib_format.2,
                    });

                    //index += 1;
                }

                *input_layout = Some(layout);
            } else if let Some(captures) = PRIMITIVE_TOPOLOGY_PRAGMA_RE.captures(pragma_str) {
                let topo_str = &captures[1];

                if topology.is_some() {
                    error!(
                        "{:?}({:?}): Duplicate primitive_topology directive",
                        this_file.path,
                        cur_line
                    );
                    num_errors += 1;
                    continue 'line; // ignore this directive
                }

                *topology = Some(match topo_str {
                    "triangle" => gl::TRIANGLES,
                    "line" => gl::LINES,
                    _ => {
                        error!(
                            "{:?}({:?}): Unsupported primitive topology: {:?}",
                            this_file.path,
                            cur_line,
                            topo_str
                        );
                        num_errors += 1;
                        continue 'line;
                    }
                });
            } else {
                error!(
                    "{:?}({:?}): Malformed `#pragma` directive: `{:?}`",
                    this_file.path,
                    cur_line,
                    pragma_str
                );
                num_errors += 1;
            }
        } else {
            if should_output_line_directive {
                preprocessed.push_str(&format!("#line {} {}\n", cur_line, this_file_index));
                should_output_line_directive = false;
            }
            preprocessed.push_str(line);
            preprocessed.push('\n');
            cur_line += 1;
        }
    }

    num_errors
}

#[derive(Debug)]
struct PreprocessedShaders {
    pub vertex: Option<String>,
    pub fragment: Option<String>,
    pub geometry: Option<String>,
    pub tess_control: Option<String>,
    pub tess_eval: Option<String>,
    pub compute: Option<String>,
    pub input_layout: Option<Vec<VertexAttribute>>,
    pub primitive_topology: Option<GLenum>,
}

fn preprocess_combined_shader_source<P: AsRef<Path>>(
    source: &str,
    path: P,
    macros: &[&str],
    _include_paths: &[&Path],
) -> (PipelineStages, PreprocessedShaders) {
    lazy_static! {
        static ref MACRO_DEF_RE: Regex = Regex::new(r"^(\w+)(?:=(\w*))?$").unwrap();
    }

    let this_file = IncludeFile {
        parent: None,
        path: path.as_ref(),
    };
    let mut source_map = Vec::new();
    let mut enabled_pipeline_stages = PipelineStages::empty();
    let mut glsl_version = None;
    let mut preprocessed = String::new();
    let mut input_layout = None;
    let mut primitive_topology = None;
    let num_errors = preprocess_shader_internal(
        &mut preprocessed,
        source,
        &mut glsl_version,
        &mut enabled_pipeline_stages,
        &mut input_layout,
        &mut primitive_topology,
        &this_file,
        &mut source_map,
    );
    debug!("PP: enabled stages: {:?}", enabled_pipeline_stages);
    debug!("PP: number of errors: {}", num_errors);

    let glsl_version = match glsl_version {
        Some(ver) => ver,
        None => {
            warn!("No #version directive found while preprocessing; defaulting to version 3.30");
            330
        }
    };

    debug!("PP: GLSL version = {}", glsl_version);
    debug!("PP: Source map:");
    for (i, f) in source_map.iter().enumerate() {
        debug!(" {} -> {:?} ", i, f.path);
    }

    let mut out_header = String::new();
    out_header.push_str(&format!("#version {}\n", glsl_version));
    for m in macros {
        if let Some(captures) = MACRO_DEF_RE.captures(m) {
            out_header.push_str("#define ");
            out_header.push_str(&captures[1]);
            if let Some(m) = captures.get(2) {
                out_header.push_str(" ");
                out_header.push_str(m.as_str());
                out_header.push('\n');
            }
        } else {
            // malformed macro
            panic!("Malformed macro definition: {}", m);
        }
    }

    let gen_variant = |stage: PipelineStages| if enabled_pipeline_stages.contains(stage) {
        let stage_def = match stage {
            PS_VERTEX => "_VERTEX_",
            PS_GEOMETRY => "_GEOMETRY_",
            PS_FRAGMENT => "_FRAGMENT_",
            PS_TESS_CONTROL => "_TESS_CONTROL_",
            PS_TESS_EVAL => "_TESS_EVAL_",
            PS_COMPUTE => "_COMPUTE_",
            _ => panic!("Unexpected pattern"),
        };
        let mut out = out_header.clone();
        out.push_str(&format!("#define {}\n", stage_def));
        out.push_str("#line 0 0\n");
        out.push_str(&preprocessed);
        Some(out)
    } else {
        None
    };

    (
        enabled_pipeline_stages,
        PreprocessedShaders {
            vertex: gen_variant(PS_VERTEX),
            geometry: gen_variant(PS_GEOMETRY),
            fragment: gen_variant(PS_FRAGMENT),
            tess_control: gen_variant(PS_TESS_CONTROL),
            tess_eval: gen_variant(PS_TESS_EVAL),
            compute: gen_variant(PS_COMPUTE),
            input_layout,
            primitive_topology,
        },
    )
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
}

impl Drop for Shader {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteShader(self.obj);
        }
    }
}

fn link_program(obj: GLuint) -> Result<GLuint, String> {
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
            Ok(obj)
        }
    }
}

impl shader::Shader for Shader {}
impl shader::VertexShader for Shader {}
impl shader::FragmentShader for Shader {}
impl shader::GeometryShader for Shader {}
impl shader::TessControlShader for Shader {}
impl shader::TessEvalShader for Shader {}
impl shader::ComputeShader for Shader {}

pub struct GlslGraphicsShaderPipeline {
    pub vertex: Shader,
    pub fragment: Shader,
    pub geometry: Option<Shader>,
    pub tess_control: Option<Shader>,
    pub tess_eval: Option<Shader>,
    pub program: GLuint
}

impl ::std::fmt::Debug for GlslGraphicsShaderPipeline {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        unimplemented!()
    }
}

impl Drop for GlslGraphicsShaderPipeline {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteProgram(self.program);
        }
    }
}

impl shader::GraphicsShaderPipeline for GlslGraphicsShaderPipeline
{
    fn vertex_shader(&self) -> &shader::VertexShader {
        &self.vertex
    }

    fn fragment_shader(&self) -> &shader::FragmentShader {
        &self.fragment
    }

    fn geometry_shader(&self) -> Option<&shader::GeometryShader> {
        self.geometry.as_ref().map(|x| x as &shader::GeometryShader)
    }

    fn tess_control_shader(&self) -> Option<&shader::TessControlShader> {
        self.tess_control.as_ref().map(|x| x as &shader::TessControlShader)
    }

    fn tess_eval_shader(&self) -> Option<&shader::TessEvalShader> {
        self.tess_eval.as_ref().map(|x| x as &shader::TessEvalShader)
    }

    fn is_compatible_with(&self, interface: &shader_interface::ShaderInterfaceDesc) -> bool {
        unimplemented!()
    }

    fn get_program(&self) -> Result<GLuint, Error> {
        Ok(self.program)
    }
}

pub struct GlslCombinedSource {
    pub shader_pipeline: GlslGraphicsShaderPipeline,
    pub input_layout: Vec<gfx::VertexAttribute>,
    pub primitive_topology: GLenum,
}

impl ::std::fmt::Debug for GlslCombinedSource {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        unimplemented!()
    }
}

#[derive(Fail, Debug)]
#[fail(display = "Compilation of GLSL shader failed (path: {:?}, stage: {:?}).", source_path, stage)]
struct GlslCompilationError {
    source_path: PathBuf,
    stage: PipelineStages,
    log: String,
}

/// The shader "compiler" for combined-source GLSL files.
/// Loads a combined GLSL source from the given path and returns compiled OpenGL shaders along with some pipeline configuration.
pub fn compile_shaders_from_combined_source<P: AsRef<Path>>(src_path: P) -> Result<GlslCombinedSource, Error> {
    // load combined shader source
    let mut src = String::new();
    File::open(src_path.as_ref())?.read_to_string(&mut src)?;

    // preprocess combined source code
    let (_stages, pp) = preprocess_combined_shader_source(&src, src_path.as_ref(), &[], &[]);

    // try to compile shaders
    let print_error_log = |log: &str, stage| {
        error!("====================================================================");
        error!(
            "Shader compilation error ({:?}) | stage: {:?}",
            src_path.as_ref(),
            stage
        );
        error!("{}\n", log);
    };

    // Compile shaders
    let vertex = Shader::compile(&pp.vertex.unwrap(), gl::VERTEX_SHADER)
        .map_err(|log| {
            print_error_log(&log, PS_VERTEX);
            GlslCompilationError { source_path: src_path.as_ref().to_owned(), stage: PS_VERTEX, log }
        })?;
    let fragment = Shader::compile(&pp.fragment.unwrap(), gl::FRAGMENT_SHADER)
        .map_err(|log| {
            print_error_log(&log, PS_FRAGMENT);
            GlslCompilationError { source_path: src_path.as_ref().to_owned(), stage: PS_FRAGMENT, log }
        })?;

    let geometry = if let Some(ref geometry) = pp.geometry {
        Some(Shader::compile(&geometry, gl::GEOMETRY_SHADER)
            .map_err(|log| {
                print_error_log(&log, PS_GEOMETRY);
                GlslCompilationError { source_path: src_path.as_ref().to_owned(), stage: PS_GEOMETRY, log }
            })?)
    } else {
        None
    };

    let tess_control = if let Some(ref tess_control) = pp.tess_control {
        Some(Shader::compile(&tess_control, gl::TESS_CONTROL_SHADER)
            .map_err(|log| {
                print_error_log(&log, PS_TESS_CONTROL);
                GlslCompilationError { source_path: src_path.as_ref().to_owned(), stage: PS_TESS_CONTROL, log }
            })?)
    } else {
        None
    };

    let tess_eval = if let Some(ref tess_eval) = pp.tess_eval {
        Some(Shader::compile(&tess_eval, gl::TESS_EVALUATION_SHADER)
            .map_err(|log| {
                print_error_log(&log, PS_TESS_EVAL);
                GlslCompilationError { source_path: src_path.as_ref().to_owned(), stage: PS_TESS_EVAL, log }
            })?)
    } else {
        None
    };

    // TODO: this leaks on error return
    let program = unsafe { gl::CreateProgram() };

    unsafe {
        gl::AttachShader(program, vertex.obj);
        gl::AttachShader(program, fragment.obj);
        if let Some(ref s) = geometry {
            gl::AttachShader(program, s.obj);
        }
        if let Some(ref s) = tess_control {
            gl::AttachShader(program, s.obj);
        }
        if let Some(ref s) = tess_eval {
            gl::AttachShader(program, s.obj);
        }
    }

    link_program(program)
        .map_err(|log| {
            unsafe {
                gl::DeleteProgram(program);
            }
            format_err!("Program link failed: {}", log)
        })?;

    // Specify layout
    Ok(GlslCombinedSource {
        shader_pipeline: GlslGraphicsShaderPipeline {
            vertex,
            fragment,
            geometry,
            tess_control,
            tess_eval,
            program
        },
        input_layout: pp.input_layout
            .ok_or(format_err!("Missing input layout in combined shader source: {}", src_path.as_ref().display()))?,
        primitive_topology: pp.primitive_topology
            .ok_or(format_err!("Missing primitive topology in combined shader source: {}", src_path.as_ref().display()))?,
    })
}



#[test]
fn test_preprocess_shaders() {
    //pretty_env_logger::init().unwrap();
    let mut src = String::new();
    let path = Path::new("data/shaders/DeferredGeometry.glsl");
    File::open(path).unwrap().read_to_string(&mut src).unwrap();
    let results = preprocess_combined_shader_source(&src, path, &[], &[]);
    println!("{:?}", results);
}
