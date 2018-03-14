use gfx;
use std::fs::File;
use gl;
use gl::types::*;
use std::io::Read;
use failure::Error;
use std::path::{Path, PathBuf};
use regex::Regex;
use gfx::pipeline::VertexAttribute;

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

// Preprocesses a combined GLSL source file: extract the additional informations in the custom pragmas
// and returns the result in (last_seen_version, enabled_pipeline_stages, input_layout, topology)
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


pub struct CompiledShaders {
    pub vertex: gfx::Shader,
    pub fragment: gfx::Shader,
    pub geometry: Option<gfx::Shader>,
    pub tess_control: Option<gfx::Shader>,
    pub tess_eval: Option<gfx::Shader>,
    pub input_layout: Vec<gfx::VertexAttribute>,
    pub primitive_topology: GLenum,
}

#[derive(Fail, Debug)]
#[fail(display = "Compilation of GLSL shader failed (path: {:?}, stage: {:?}).", source_path, stage)]
struct GLSLCompilationError {
    source_path: PathBuf,
    stage: PipelineStages,
    log: String,
}

/// The shader "compiler" for combined-source GLSL files.
/// Loads a combined GLSL source from the given path and returns compiled OpenGL shaders along with some pipeline configuration.
pub fn compile_shaders_from_combined_source<P: AsRef<Path>>(src_path: P) -> Result<CompiledShaders, Error> {
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
    let vertex = gfx::Shader::compile(&pp.vertex.unwrap(), gl::VERTEX_SHADER)
        .map_err(|log| {
            print_error_log(&log, PS_VERTEX);
            GLSLCompilationError { source_path: src_path.as_ref().to_owned(), stage: PS_VERTEX, log }
        })?;
    let fragment = gfx::Shader::compile(&pp.fragment.unwrap(), gl::FRAGMENT_SHADER)
        .map_err(|log| {
            print_error_log(&log, PS_FRAGMENT);
            GLSLCompilationError { source_path: src_path.as_ref().to_owned(), stage: PS_FRAGMENT, log }
        })?;

    let geometry = if let Some(ref geometry) = pp.geometry {
        Some(gfx::Shader::compile(&geometry, gl::GEOMETRY_SHADER)
            .map_err(|log| {
                print_error_log(&log, PS_GEOMETRY);
                GLSLCompilationError { source_path: src_path.as_ref().to_owned(), stage: PS_GEOMETRY, log }
            })?)
    } else {
        None
    };

    let tess_control = if let Some(ref tess_control) = pp.tess_control {
        Some(gfx::Shader::compile(&tess_control, gl::TESS_CONTROL_SHADER)
            .map_err(|log| {
                print_error_log(&log, PS_TESS_CONTROL);
                GLSLCompilationError { source_path: src_path.as_ref().to_owned(), stage: PS_TESS_CONTROL, log }
            })?)
    } else {
        None
    };

    let tess_eval = if let Some(ref tess_eval) = pp.tess_eval {
        Some(gfx::Shader::compile(&tess_eval, gl::TESS_EVALUATION_SHADER)
            .map_err(|log| {
                print_error_log(&log, PS_TESS_EVAL);
                GLSLCompilationError { source_path: src_path.as_ref().to_owned(), stage: PS_TESS_EVAL, log }
            })?)
    } else {
        None
    };

    // Specify layout
    Ok(CompiledShaders {
        vertex,
        fragment,
        geometry,
        tess_control,
        tess_eval,
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