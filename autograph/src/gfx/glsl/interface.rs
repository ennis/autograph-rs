use super::shader_interface::*;
use super::SpirvGraphicsShaderPipeline;
use super::spirv_parse::*;
use std::cmp::max;
//use rspirv::binary::ParseAction;
//use rspirv::grammar::reflect::*;
//use rspirv::mr::{Module,ModuleHeader,Instruction,Operand};

//macro_rules! unwrap_operand {
//    ($inst:expr, $o:expr, $v:path) => { if let $v(ref inside) = $inst.operands[$o] { inside } else { panic!("unexpected operand type")}};
//}

#[derive(Deref)]
struct ModuleWrapper(SpirvModule);

fn parse_module(bytecode: &[u32]) -> ModuleWrapper {
    ModuleWrapper(parse_spirv_u32s(bytecode).unwrap())
}

struct Std140LayoutBuilder
{
    next_offset: usize,
}

fn align_offset(ptr: usize, align: usize) -> usize {
    let offset = ptr % align;
    if offset == 0 {
        0
    } else {
        align - offset
    }
}


impl Std140LayoutBuilder
{
    fn new() -> Std140LayoutBuilder { Std140LayoutBuilder { next_offset: 0 } }

    fn align(&mut self, a: usize) -> usize {
        self.next_offset += align_offset(self.next_offset, a);
        self.next_offset
    }

    fn get_align_and_size(&self, module: &ModuleWrapper, inst: &Instruction) -> (usize, usize)
    {
        match *inst {
            Instruction::TypeBool(ty) => {
                (4, 4)
            },
            Instruction::TypeInt(ty) => {
                assert!(ty.width == 32);
                (4, 4)
            },
            Instruction::TypeFloat(ty) => {
                assert!(ty.width == 32);
                (4, 4)
            }
            Instruction::TypeVector(ty) => {
                let compty = module.find_type(ty.component_id).unwrap();
                let (_, n) = self.get_align_and_size(module, compty);
                match ty.count {
                    2 => (2*n, 2*n),
                    3 => (4*n, 3*n),
                    4 => (4*n, 4*n),
                    _ => panic!("unsupported vector size")
                }
            },
            Instruction::TypeMatrix(ty) => {
                let column_ty = module.find_type(ty.column_type_id).unwrap();
                let (col_align, col_size) = self.get_align_and_size(module, column_ty);
                // alignment = column type align rounded up to vec4 align (16 bytes)
                let base_align = max(16, col_align);
                let stride = col_size + align_offset(col_size, col_align);
                // total array size = num columns * stride, rounded up to the next multiple of the base alignment

            },
            Instruction::TypeImage { result_id, .. } if result_id == id => true,
            Instruction::TypeSampler { result_id, .. } if result_id == id => true,
            Instruction::TypeSampledImage { result_id, .. } if result_id == id => true,
            Instruction::TypeArray { result_id, .. } if result_id == id => true,
            Instruction::TypeRuntimeArray { result_id, .. } if result_id == id => true,
            Instruction::TypeStruct { result_id, .. } if result_id == id => true,
            Instruction::TypeOpaque { result_id, .. } if result_id == id => true,
            Instruction::TypePointer { result_id, .. } if result_id == id => true,
        }
    }

    fn add_member(&mut self, module: &ModuleWrapper, inst: &Instruction) -> u32 {
        let current_offset = self.next_offset;

        let (off, size) = match *inst {
            Instruction::TypeBool(ty) => {
                (align(4), 4)
            },
            Instruction::TypeInt(ty) => {
                assert!(ty.width == 32);
                (align(4), 4)
            },
            Instruction::TypeFloat(ty) => {
                assert!(ty.width == 32);
                (align(4), 4)
            }
            Instruction::TypeVector(ty)  => {
                let basety =
            },
            Instruction::TypeMatrix { result_id, .. } if result_id == id => true,
            Instruction::TypeImage { result_id, .. } if result_id == id => true,
            Instruction::TypeSampler { result_id, .. } if result_id == id => true,
            Instruction::TypeSampledImage { result_id, .. } if result_id == id => true,
            Instruction::TypeArray { result_id, .. } if result_id == id => true,
            Instruction::TypeRuntimeArray { result_id, .. } if result_id == id => true,
            Instruction::TypeStruct { result_id, .. } if result_id == id => true,
            Instruction::TypeOpaque { result_id, .. } if result_id == id => true,
            Instruction::TypePointer { result_id, .. } if result_id == id => true,
        }
        0
    }
}

impl ModuleWrapper
{
    fn find_decoration(&self, id: u32, deco: spirv::Decoration) -> Option<&Instruction> {
        self.0.instructions.iter().find(|&inst|
            match *inst {
                Instruction::Decorate { target_id, decoration } if target_id == id && decoration == deco => true,
                _ => false
            })
    }

    fn find_type(&self, id: u32) -> Option<&Instruction> {
        self.0.instructions.iter().find(|&inst|
            match *inst {
                Instruction::TypeVoid { result_id } if result_id == id => true,
                Instruction::TypeBool { result_id, .. } if result_id == id => true,
                Instruction::TypeInt { result_id, .. } if result_id == id => true,
                Instruction::TypeFloat { result_id, .. } if result_id == id => true,
                Instruction::TypeVector { result_id, .. } if result_id == id => true,
                Instruction::TypeMatrix { result_id, .. } if result_id == id => true,
                Instruction::TypeImage { result_id, .. } if result_id == id => true,
                Instruction::TypeSampler { result_id, .. } if result_id == id => true,
                Instruction::TypeSampledImage { result_id, .. } if result_id == id => true,
                Instruction::TypeArray { result_id, .. } if result_id == id => true,
                Instruction::TypeRuntimeArray { result_id, .. } if result_id == id => true,
                Instruction::TypeStruct { result_id, .. } if result_id == id => true,
                Instruction::TypeOpaque { result_id, .. } if result_id == id => true,
                Instruction::TypePointer { result_id, .. } if result_id == id => true,
                _ => false
            })
    }

    fn find_location_decoration(&self, id: u32) -> Option<u32>
    {
        self.find_decoration(id, spirv::Decoration::Location).map(|inst| unwrap_operand!(inst, 0, Operand::LiteralInt32))
    }

    fn compare_types(&self, ty_inst: &Instruction, ty_ref: &TypeDesc) -> bool
    {
        match *ty_inst {
            Instruction::TypePointer(ptr) => false, // no pointers in interface, for now
            Instruction::TypeFloat(ty) => {
                ty.width == 32 && ty_ref == &TypeDesc::Primitive(PrimitiveType::Float)
            },
            Instruction::TypeInt(ty) => {
                ty.width == 32 && ty_ref == &TypeDesc::Primitive(if ty.signedness == 1 { PrimitiveType::Int } else { PrimitiveType::UnsignedInt })
            },
            Instruction::TypeVector(ty) => {
                if let TypeDesc::Vector(comp_ty_ref, comp_count_ref) = ty_ref {
                    let comp_ty = self.find_type(ty.component_id).unwrap();
                    let comp_count = ty.count;
                    assert!(ty.count <= 4);
                    comp_count == comp_count_ref && self.compare_types(comp_ty, &TypeDesc::Primitive(comp_ty_ref))
                } else {
                    false
                }
            },
            Instruction::TypeMatrix(ty) => {
                if let TypeDesc::Matrix(comp_ty_ref, row_count_ref, col_count_ref) = ty_ref {
                    let column_ty = self.find_type(ty.column_type_id).unwrap();
                    let col_count = ty.column_count;
                    if let Instruction::TypeVector(column_ty) = column_ty {
                        let comp_ty = self.find_type(column_ty.component_id).unwrap();
                        let row_count = column_ty.count;
                        row_count == row_count_ref &&
                            col_count == col_count_ref &&
                            self.compare_types(comp_ty, &TypeDesc::Primitive(comp_ty_ref))
                    } else {
                        panic!("malformed SPIR-V bytecode")
                    }

                } else {
                    false
                }
            },
            Instruction::TypeStruct(ty) => {
                if let TypeDesc::Struct(ref members) = ty_ref {
                    //
                    true
                } else {
                    false
                }
            }
        }
    }
}

struct SpirvGraphicsPipelineModules
{
    vs: ModuleWrapper,
    fs: ModuleWrapper,
    gs: Option<ModuleWrapper>,
    tcs: Option<ModuleWrapper>,
    tes: Option<ModuleWrapper>
}


enum VerifyResult
{
    Skip,
    Match,
    Mismatch { reason: String }
}


impl SpirvGraphicsPipelineModules
{
    fn verify_named_uniform(&self, u: &UniformConstantDesc) {

        let verify = |module: &ModuleWrapper, inst: Instruction| -> VerifyResult {
            // filter out anything that is not a variable
            if inst.class.op != spirv::Op::Variable { return VerifyResult::Skip }
            // must have storage class uniform
            if inst.operands[0] != spirv::StorageClass::Uniform { return VerifyResult::Skip }
            let id = u.result_id;
            // check that type is a pointer
            let ptr_ty_inst = module.find_type(u.result_type).expect("malformed SPIR-V");
            assert!(ptr_ty_inst.class.op == spirv::Op::TypePointer, "malformed SPIR-V");
            let uniform_ty_inst = unwrap_operand!(ptr_ty_inst, 1, Operand::IdRef);

            // if an explicit location is provided, check that it matches
            let loc = module.find_location_decoration(id);
            //



            VerifyResult::Skip
        };

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
