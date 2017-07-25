extern crate autograph;

// The `vulkano` crate is the main crate that you must use to use Vulkan.
#[macro_use] extern crate vulkano;
#[macro_use] extern crate vulkano_shaders;
extern crate winit;
extern crate vulkano_win;
extern crate time;
extern crate pretty_env_logger;
extern crate glsl_to_spirv;
#[macro_use] extern crate log;
extern crate rspirv;
extern crate spirv_reflect;
extern crate spirv_headers as spirv;

use std::path::Path;
use std::fs::File;
use std::io::Read;
use std::sync::Arc;
use std::collections::BTreeSet;
use std::cmp::Ord;
use rspirv::binary::Disassemble;
use vulkano::pipeline::shader::*;
use vulkano::descriptor::descriptor::*;
use vulkano::descriptor::descriptor_set::*;
use vulkano::descriptor::pipeline_layout::*;
use vulkano::device::Device;
use vulkano::device::DeviceOwned;


use autograph::shader_preprocessor::preprocess_combined_shader_source;


const COMBINED_SHADER_PATH: &str = "data/shaders/DeferredGeometry450.glsl";

#[derive(Clone,Default)]
struct DescriptorSet
{
    bindings: Vec<Option<DescriptorDesc>>   // None => empty descriptor (hole)
}

struct RuntimePipelineLayout
{
    sets: Vec<DescriptorSet>,
}

impl RuntimePipelineLayout
{
    pub fn from_spirv(spirv: &[u8]) -> RuntimePipelineLayout
    {
        let reflect = spirv_reflect::Reflect::from_bytes(spirv).unwrap();

        let mut desc_sets: Vec<DescriptorSet> = Vec::new();

        let mut inputs = Vec::new();
        let mut outputs = Vec::new();
        let mut uniforms = Vec::new();
        for v in reflect.variables.values() {
            match v.storage_class {
                spirv::StorageClass::Input => {inputs.push(v);},
                spirv::StorageClass::Output => {outputs.push(v);},
                spirv::StorageClass::Uniform => {uniforms.push(v);},
                _ => ()
            }
        }
        inputs.sort_by(|a,b| { Ord::cmp(&a.deco.location.unwrap(), &b.deco.location.unwrap()) });
        outputs.sort_by(|a,b| { Ord::cmp(&a.deco.location.unwrap_or(0), &b.deco.location.unwrap_or(0)) });
        uniforms.sort_by(|a,b| { Ord::cmp(&a.deco.location.unwrap_or(0), &b.deco.location.unwrap_or(0)) });

        for i in inputs {
            println!("Input {}: ty {:?} def {:?}", if reflect.is_builtin_variable(i) { "[BUILTIN] "} else {""}, reflect.describe_type(i.ty), i);
        }
        for o in outputs {
            println!("Output {}: ty {:?} def {:?}", if reflect.is_builtin_variable(o) { "[BUILTIN] "} else {""}, reflect.describe_type(o.ty), o);
        }

        for u in uniforms {
            match u.deco.descriptor {
                Some((set,binding)) => {
                    desc_sets.resize(set as usize, DescriptorSet { bindings: Vec::new() });
                    /*desc_sets[set as usize].push(DescriptorDesc {
                        array_count: 1,
                        readonly: false,
                        stages:
                    });*/
                    unimplemented!()
                },
                _ => unimplemented!()
            }

            println!("Uniform: ty {:?} def {:?}", reflect.describe_type(u.ty), u);
        }


        unimplemented!()
    }
}

unsafe impl PipelineLayoutDesc for RuntimePipelineLayout
{
    /// Returns the number of sets in the layout. Includes possibly empty sets.
    ///
    /// In other words, this should be equal to the highest set number plus one.
    fn num_sets(&self) -> usize
    {
        self.sets.len()
    }

    /// Returns the number of descriptors in the set. Includes possibly empty descriptors.
    ///
    /// Returns `None` if the set is out of range.
    fn num_bindings_in_set(&self, set: usize) -> Option<usize>
    {
        self.sets.get(set).map(|set| set.bindings.len())
    }

    /// Returns the descriptor for the given binding of the given set.
    ///
    /// Returns `None` if out of range or if the descriptor is empty.
    fn descriptor(&self, set: usize, binding: usize) -> Option<DescriptorDesc>
    {
        self.sets.get(set).and_then(|set| set.bindings.get(binding).and_then(|desc| desc.clone()))
    }

    /// If the `PipelineLayoutDesc` implementation is able to provide an existing
    /// `UnsafeDescriptorSetLayout` for a given set, it can do so by returning it here.
    #[inline]
    fn provided_set_layout(&self, set: usize) -> Option<Arc<UnsafeDescriptorSetLayout>> {
        None
    }

    /// Returns the number of push constant ranges of the layout.
    fn num_push_constants_ranges(&self) -> usize
    {
        unimplemented!()
    }

    /// Returns a description of the given push constants range.
    ///
    /// Contrary to the descriptors, a push constants range can't be empty.
    ///
    /// Returns `None` if out of range.
    ///
    /// Each bit of `stages` must only be present in a single push constants range of the
    /// description.
    fn push_constants_range(&self, num: usize) -> Option<PipelineLayoutDescPcRange>
    {
        unimplemented!()
    }

    /// Builds the union of this layout and another.
    #[inline]
    fn union<T>(self, other: T) -> PipelineLayoutDescUnion<Self, T>
        where Self: Sized
    {
        PipelineLayoutDescUnion::new(self, other)
    }

    /// Turns the layout description into a `PipelineLayout` object that can be used by Vulkan.
    ///
    /// > **Note**: This is just a shortcut for `PipelineLayout::new`.
    #[inline]
    fn build(self, device: Arc<Device>) -> Result<PipelineLayout<Self>, PipelineLayoutCreationError>
        where Self: Sized
    {
        PipelineLayout::new(device, self)
    }
}

fn main()
{
    // load combined shader source
    pretty_env_logger::init().unwrap();
    let mut src = String::new();
    File::open(COMBINED_SHADER_PATH).unwrap().read_to_string(&mut src).unwrap();

    // preprocess
    let (stages, sources) = preprocess_combined_shader_source(&src, Path::new(COMBINED_SHADER_PATH), &[], &[]);

    // debug output
    println!("Vertex shader: {}", sources.vertex.as_ref().map(|x| x.as_str()).unwrap_or("Not present"));
    println!("Tess control shader: {}", sources.tess_control.as_ref().map(|x| x.as_str()).unwrap_or("Not present"));
    println!("Tess eval shader: {}", sources.tess_eval.as_ref().map(|x| x.as_str()).unwrap_or("Not present"));
    println!("Geometry shader: {}", sources.geometry.as_ref().map(|x| x.as_str()).unwrap_or("Not present"));
    println!("Fragment shader: {}", sources.fragment.as_ref().map(|x| x.as_str()).unwrap_or("Not present"));
    println!("Compute shader: {}", sources.compute.as_ref().map(|x| x.as_str()).unwrap_or("Not present"));

    // compile to SPIR-V

    fn try_compile(src: Option<String>, shader_type: glsl_to_spirv::ShaderType) -> Option<Vec<u8>> {
        if let Some(ref src) = src {
            match glsl_to_spirv::compile(src.as_str(), shader_type.clone()) {
                Err(e) => { println!("Shader compilation error ({:?}): {}", shader_type, e);
                    None
                },
                Ok(mut result) => {
                    println!("SPIR-V result ({:?}): {:?}", shader_type, result);
                    // read back file
                    let mut blob = Vec::new();
                    result.read_to_end(&mut blob).unwrap();
                    println!("Disassembly: ");
                    let module = rspirv::mr::load_bytes(&blob).expect("Invalid SPIR-V binary blob");
                    println!("{}", module.disassemble());
                    println!("\n");
                    // parse spir-v
                    let reflection = spirv_reflect::Reflect::from_bytes(&blob).unwrap();
                    //println!("{:#?}", reflection);

                    // dump interface and types
                    let mut inputs = Vec::new();
                    let mut outputs = Vec::new();
                    let mut uniforms = Vec::new();
                    for v in reflection.variables.values() {
                        match v.storage_class {
                            spirv::StorageClass::Input => {inputs.push(v);},
                            spirv::StorageClass::Output => {outputs.push(v);},
                            spirv::StorageClass::Uniform => {uniforms.push(v);},
                            _ => ()
                        }
                    }
                    inputs.sort_by(|a,b| { Ord::cmp(&a.deco.location.unwrap(), &b.deco.location.unwrap()) });
                    outputs.sort_by(|a,b| { Ord::cmp(&a.deco.location.unwrap_or(0), &b.deco.location.unwrap_or(0)) });

                    for i in inputs {
                        println!("Input {}: ty {:?} def {:?}", if reflection.is_builtin_variable(i) { "[BUILTIN] "} else {""}, reflection.describe_type(i.ty), i);
                    }
                    for o in outputs {
                        println!("Output {}: ty {:?} def {:?}", if reflection.is_builtin_variable(o) { "[BUILTIN] "} else {""}, reflection.describe_type(o.ty), o);
                    }
                    for u in uniforms {
                        println!("Uniform: ty {:?} def {:?}", reflection.describe_type(u.ty), u);
                    }

                    // extract interface
                    Some(blob)
                }
            }
        } else {
            None
        }
    }

    try_compile(sources.vertex, glsl_to_spirv::ShaderType::Vertex);
    try_compile(sources.tess_control, glsl_to_spirv::ShaderType::TessellationControl);
    try_compile(sources.tess_eval, glsl_to_spirv::ShaderType::TessellationEvaluation);
    try_compile(sources.geometry, glsl_to_spirv::ShaderType::Geometry);
    try_compile(sources.fragment, glsl_to_spirv::ShaderType::Fragment);
    try_compile(sources.compute, glsl_to_spirv::ShaderType::Compute);

    // now reflect

}