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
extern crate spirv_headers as spirv;
extern crate rspirv;

use std::path::Path;
use std::fs::File;
use std::io::Read;
use rspirv::binary::Disassemble;
use vulkano::pipeline::shader::ShaderInterfaceDefEntry;

use autograph::shader_preprocessor::preprocess_combined_shader_source;


const COMBINED_SHADER_PATH: &str = "data/shaders/DeferredGeometry450.glsl";

macro_rules! operand_cast {
    ($op:expr, $op_type:ident) => {
        if let &rspirv::mr::Operand::$op_type(ref a) = $op {
            a
        } else {
            panic!("Unexpected operand type")
        }
    };
}

fn as_op_entry_point(insn: &rspirv::mr::Instruction)
{
}

// find insn by result id
fn find_by_id(insns: &[rspirv::mr::Instruction], rid: u32) -> &rspirv::mr::Instruction
{
    insns.iter().find(|i| if let Some(result_id) = i.result_id { result_id == rid } else { false }).unwrap()
}

enum SpirvLeafType
{
    Bool,
    Float,
    Int,
    Void
}

enum SpirvType
{
    Leaf(SpirvLeafType),
    Vector(SpirvLeafType, i32),
    Struct(u32)     // u32 is the type-id of the struct
}

fn describe_spirv_type(module: &rspirv::mr::Module, id: u32) -> SpirvType
{
    let insn = find_by_id(&module.types_global_values, id);
    match insn.class.opcode {
        spirv::Op::TypeVoid => SpirvType::Leaf(SpirvLeafType::Void),
        spirv::Op::TypeBool => SpirvType::Leaf(SpirvLeafType::Bool),
        spirv::Op::TypeInt => SpirvType::Leaf(SpirvLeafType::Int),
        spirv::Op::TypeFloat => SpirvType::Leaf(SpirvLeafType::Float),
        spirv::Op::TypeVector => {
            let base_ty = if let SpirvType::Leaf(leaf_type) = describe_spirv_type(module, operand_cast!(&insn.operands[0], RefId)) {
                leaf_type
            } else {
                panic!("Unexpected vector base type")
            };
            let size = operand_cast!(&insn.operands[0], LiteralInt32);
            SpirvType::Vector(base_ty, *size as i32)
        },
        spirv::Op::TypeMatrix => {
            unimplemented!()
        },
        spirv::Op::TypeImage => unimplemented!(),
        spirv::Op::TypeSampler => unimplemented!(),
        spirv::Op::TypeSampledImage => unimplemented!(),
        spirv::Op::TypeArray => unimplemented!(),
        spirv::Op::TypeRuntimeArray => unimplemented!(),
        spirv::Op::TypeStruct => unimplemented!(),
        spirv::Op::TypeOpaque => unimplemented!(),
        spirv::Op::TypePointer => unimplemented!(),
        spirv::Op::TypeFunction => unimplemented!(),
        spirv::Op::TypeEvent => unimplemented!(),
        spirv::Op::TypeDeviceEvent => unimplemented!(),
        spirv::Op::TypeReserveId => unimplemented!(),
        spirv::Op::TypeQueue => unimplemented!(),
        spirv::Op::TypePipe => unimplemented!(),
        spirv::Op::TypeForwardPointer => unimplemented!(),
        _ => panic!("Whatever")
    }
}

//

fn dump_spirv_blob(blob: &[u8])
{
    let module: rspirv::mr::Module = rspirv::mr::load_bytes(blob).expect("Invalid SPIR-V binary blob");
    println!("{}", module.disassemble());

    //
    
    for ep in module.entry_points {
        let execution_model = operand_cast!(&ep.operands[0], ExecutionModel);
        let function_id = operand_cast!(&ep.operands[1], IdRef);
        let name = operand_cast!(&ep.operands[2], LiteralString);
        let interface : Vec<_> = ep.operands[3..].iter().map(|o| operand_cast!(o, IdRef)).collect();
        println!("Module entry point: {:?},{:?},{:?},{:?}", execution_model, function_id, name, interface);

        // extract interface
        for var in interface {
            let var_insn = find_by_result_id(&module.types_global_values, *var);
            let ty = operand_cast!(&var_insn.operands[0], IdRef);

        }
    }

    // extract interface
    //module.

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
                    dump_spirv_blob(blob.as_ref());
                    println!("\n");
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