use super::shader_interface::*;
use super::SpirvGraphicsShaderPipeline;
use super::spirv_parse::*;
use std::cmp::max;
use failure::Error;
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

fn round_up(value: usize, multiple: usize) -> usize {
    if multiple == 0 { return value }
    let remainder = value % multiple;
    if remainder == 0 { return value }
    value + multiple - remainder
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
                let array_size = round_up(ty.column_count * stride, base_align);
            },
            Instruction::TypeImage(_) => panic!("unsupported type"),
            Instruction::TypeSampler(_) => panic!("unsupported type"),
            Instruction::TypeSampledImage(_) => panic!("unsupported type"),
            Instruction::TypeArray(ty)  => {
                panic!("unsupported type")
            },
            Instruction::TypeRuntimeArray(_) => panic!("unsupported type"),
            Instruction::TypeStruct(_) => {
                panic!("unsupported type")
            },
            Instruction::TypeOpaque(_) => panic!("unsupported type"),
            Instruction::TypePointer(_) => panic!("unsupported type"),
        }
    }

    fn add_member(&mut self, module: &ModuleWrapper, ty_inst: &Instruction) -> usize
    {
        let (align, size) = self.get_align_and_size(module, ty_inst);
        let current_offset = self.align(align);
        self.next_offset += size;
        current_offset
    }
}

#[derive(Fail,Debug)]
pub enum InterfaceError {
    #[fail(display = "struct member mismatch between {} (host) and {} (device)", host_ty, device_ty)]
    MemberMismatch {
        // the underlying cause of the mismatch: type error, or another member mismatch
        #[cause] cause: Box<InterfaceError>,

        device_ty: String,
        device_member_index: u32,
        device_member_name: Option<String>,

        host_ty: String,
        host_member_index: u32,
        host_member_name: Option<String>,
    },
    //#[fail(display = "mismatching member offsets", host_ty, device_ty)]
    MemberOffsetMismatch {
        device_ty: String,
        device_member_index: u32,
        device_member_name: Option<String>,
        device_offset: usize,

        host_ty: String,
        host_member_index: u32,
        host_member_name: Option<String>,
        host_offset: usize
    },
    #[fail(display = "host and device types are incompatible: {} (host) and {} (device)", host_ty, device_ty)]
    PrimitiveTypeMismatch {
        device_ty: String,
        host_ty: String
    }
}

// interface mismatch between structX and structX_host
// -> caused by: interface mismatch in Y
// -> caused by: interface mismatch in Z
// -> caused by: mismatching member offsets in W

impl ModuleWrapper
{
    fn find_decoration(&self, id: u32, deco: spirv::Decoration) -> Option<&Instruction> {
        self.0.instructions.iter().find(|&inst|
            match *inst {
                Instruction::Decorate(IDecorate { target_id, decoration }) if target_id == id && decoration == deco => true,
                _ => false
            })
    }

    fn find_type(&self, id: u32) -> Option<&Instruction> {
        self.0.instructions.iter().find(|&inst|
            match *inst {
                Instruction::TypeVoid(ITypeVoid { result_id }) if result_id == id => true,
                Instruction::TypeBool(ITypeBool { result_id }) if result_id == id => true,
                Instruction::TypeInt(ITypeInt { result_id, .. }) if result_id == id => true,
                Instruction::TypeFloat(ITypeFloat { result_id, .. }) if result_id == id => true,
                Instruction::TypeVector(ITypeVector { result_id, .. }) if result_id == id => true,
                Instruction::TypeMatrix(ITypeMatrix { result_id, .. }) if result_id == id => true,
                Instruction::TypeImage(ITypeImage { result_id, .. }) if result_id == id => true,
                Instruction::TypeSampler(ITypeSampler { result_id }) if result_id == id => true,
                Instruction::TypeSampledImage(ITypeSampledImage { result_id, .. }) if result_id == id => true,
                Instruction::TypeArray(ITypeArray { result_id, .. }) if result_id == id => true,
                Instruction::TypeRuntimeArray(ITypeRuntimeArray { result_id, .. }) if result_id == id => true,
                Instruction::TypeStruct(ITypeStruct { result_id, .. }) if result_id == id => true,
                Instruction::TypeOpaque(ITypeOpaque { result_id, .. }) if result_id == id => true,
                Instruction::TypePointer(ITypePointer { result_id, .. }) if result_id == id => true,
                _ => false
            })
    }

    fn find_location_decoration(&self, id: u32) -> Option<u32>
    {
        self.find_decoration(id, spirv::Decoration::Location).map(|inst| unwrap_operand!(inst, 0, Operand::LiteralInt32))
    }


    /// Check that the type described by the spirv instruction
    /// is layout-compatible with the given host type description
    fn compare_types(&self, ty_inst: &Instruction, host_ty: &TypeDesc) -> Result<(),InterfaceError>
    {
        match *ty_inst {
            Instruction::TypePointer(ptr) => false, // no pointers in interface, for now
            Instruction::TypeFloat(ty) => {
                if !(ty.width == 32 && host_ty == &TypeDesc::Primitive(PrimitiveType::Float)) {
                    Err(InterfaceError::PrimitiveTypeMismatch {
                        device_ty: format!("{:?}", ty),
                        host_ty: format!("{:?}", host_ty),
                    })
                } else {
                    Ok(())
                }
            },
            Instruction::TypeInt(ty) => {
                if !(ty.width == 32 && host_ty == &TypeDesc::Primitive(if ty.signedness == 1 { PrimitiveType::Int } else { PrimitiveType::UnsignedInt })) {
                    Err(InterfaceError::PrimitiveTypeMismatch {
                        device_ty: format!("{:?}", ty),
                        host_ty: format!("{:?}", host_ty),
                    })
                } else {
                    Ok(())
                }
            },
            Instruction::TypeVector(ty) => {
                if let TypeDesc::Vector(host_comp_ty, host_comp_count) = ty_host {
                    let comp_ty = self.find_type(ty.component_id).unwrap();
                    let comp_count = ty.count;
                    assert!(ty.count <= 4);
                    if !(comp_count == host_comp_count && self.compare_types(comp_ty, &TypeDesc::Primitive(host_comp_ty))) {
                        Err(InterfaceError::PrimitiveTypeMismatch {
                            device_ty: format!("{:?}", ty),
                            host_ty: format!("{:?}", host_ty),
                        })
                    } else {
                        Ok(())
                    }
                } else {
                    false
                }
            },
            Instruction::TypeMatrix(ty) => {
                if let TypeDesc::Matrix(host_comp_ty, host_row_count, host_col_count) = host_ty {
                    let column_ty = self.find_type(ty.column_type_id).unwrap();
                    let col_count = ty.column_count;
                    if let Instruction::TypeVector(column_ty) = column_ty {
                        let comp_ty = self.find_type(column_ty.component_id).unwrap();
                        let row_count = column_ty.count;
                        row_count == host_row_count &&
                            col_count == host_col_count &&
                            self.compare_types(comp_ty, &TypeDesc::Primitive(host_comp_ty))
                    } else {
                        panic!("malformed SPIR-V bytecode")
                    }

                } else {
                    false
                }
            },
            Instruction::TypeStruct(ty) => {
                if let TypeDesc::Struct(ref host_members) = host_ty {
                    // build layout
                    let mut std140_layout = Std140LayoutBuilder::new();
                    let mut layout_mismatch = false;
                    let mut device_member_index = 0;
                    let mut host_member_index = 0;
                    for member_ty in ty.member_types {
                        let member_ty = self.find_type(member_ty).unwrap();
                        // get next member in reference struct
                        let &(host_member_offset, host_member_ty) =
                            if let Some(v) = member_iter.next() {
                                v
                            } else {
                                // ran out of members, this is a
                                layout_mismatch = true;
                                break
                            };
                        if !self.compare_types(member_ty, host_member_ty) {

                        }

                        let member_offset = std140_layout.add_member(self, member_ty);
                        //if member_offset != host_member_

                        // check type and offset of member
                        //if !( self.compare_types(member_ty, host_member_ty);

                        device_member_index += 1;
                        host_member_index += 1;
                    }
                } else {
                    false
                }
            }
        }
        unimplemented!()
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
