use super::shader_interface::*;
use super::SpirvGraphicsShaderPipeline;
use rspirv::binary::ParseAction;
use rspirv::mr::{Module,ModuleHeader,Instruction};

fn parse_module(bytecode: &[u32]) -> Module {
    let mut loader = rspirv::mr::Loader::new();
    rspirv::binary::parse_words(&bytecode, &mut loader).unwrap();
    loader.module()
}

struct SpirvGraphicsPipelineModules
{
    vs: Module,
    fs: Module,
    gs: Option<Module>,
    tcs: Option<Module>,
    tes: Option<Module>
}

impl SpirvGraphicsPipelineModules
{
    fn verify_named_uniform(&self, u: &NamedUniformDesc) {
        let verify = |u: Instruction| {
            match u.class.op {

            }
        }

    }
}

pub fn verify_spirv_interface(shaders: &SpirvGraphicsShaderPipeline, interface: &ShaderInterfaceDesc) -> bool
{

    let vs_module = parse_module(&shaders.vertex_bytecode);
    let fs_module = parse_module(&shaders.fragment_bytecode);
    let gs_module = shaders.geometry_bytecode.as_ref().map(|bytecode| parse_module(bytecode));
    let tcs_module = shaders.tess_control_bytecode.as_ref().map(|bytecode| parse_module(bytecode));
    let tes_module = shaders.tess_eval_bytecode.as_ref().map(|bytecode| parse_module(bytecode));

    // how to link (i.e. say that a variable is the same) an uniform between stages?
    // - with the name: same name => same variable

    // verify named uniform: look for an uniform with the given name in all modules, check type
    // uniform buffers: OpVariable <pointer to struct> Decorated with binding
    //
}
