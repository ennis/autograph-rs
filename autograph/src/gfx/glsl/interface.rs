use super::shader_interface::*;
use super::SpirvGraphicsShaderPipeline;
use super::spirv_parse::*;
use std::collections::{HashMap, hash_map::Entry};
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
    if multiple == 0 { return value; }
    let remainder = value % multiple;
    if remainder == 0 { return value; }
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
            }
            Instruction::TypeInt(ty) => {
                assert!(ty.width == 32);
                (4, 4)
            }
            Instruction::TypeFloat(ty) => {
                assert!(ty.width == 32);
                (4, 4)
            }
            Instruction::TypeVector(ty) => {
                let compty = module.find_type(ty.component_id).unwrap();
                let (_, n) = self.get_align_and_size(module, compty);
                match ty.count {
                    2 => (2 * n, 2 * n),
                    3 => (4 * n, 3 * n),
                    4 => (4 * n, 4 * n),
                    _ => panic!("unsupported vector size")
                }
            }
            Instruction::TypeMatrix(ty) => {
                let column_ty = module.find_type(ty.column_type_id).unwrap();
                let (col_align, col_size) = self.get_align_and_size(module, column_ty);
                // alignment = column type align rounded up to vec4 align (16 bytes)
                let base_align = max(16, col_align);
                let stride = col_size + align_offset(col_size, col_align);
                // total array size = num columns * stride, rounded up to the next multiple of the base alignment
                let array_size = round_up(ty.column_count * stride, base_align);
            }
            Instruction::TypeImage(_) => panic!("unsupported type"),
            Instruction::TypeSampler(_) => panic!("unsupported type"),
            Instruction::TypeSampledImage(_) => panic!("unsupported type"),
            Instruction::TypeArray(ty) => {
                panic!("unsupported type")
            }
            Instruction::TypeRuntimeArray(_) => panic!("unsupported type"),
            Instruction::TypeStruct(_) => {
                panic!("unsupported type")
            }
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

#[derive(Fail, Debug)]
#[fail(display = "interface mismatch: {}", info)]
pub struct InterfaceError {
    // the underlying cause of the mismatch: type error, or another member mismatch
    #[cause] cause: Option<Box<InterfaceError>>,
    info: String,
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

    fn find_name(&self, id: u32) -> Option<&str> {
        self.0.instructions.iter().find_map(|&inst|
            match *inst {
                Instruction::Name(IName { target_id, ref name }) if target_id == id => Some(name),
                _ => None
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
    fn compare_types(&self, ty_inst: &Instruction, host_ty: &TypeDesc) -> Result<(), InterfaceError>
    {
        fn type_mismatch_err(device_ty: &Instruction, host_ty: &TypeDesc) -> InterfaceError {
            InterfaceError {
                cause: None,
                info: format!("type mismatch: {:?} (device) and {:?} (host)", device_ty, host_ty),
            }
        }

        match *ty_inst {
            //Instruction::TypePointer(ptr) => { panic!()}, // no pointers in interface, for now
            Instruction::TypeFloat(ty) => {
                if !(ty.width == 32 && host_ty == &TypeDesc::Primitive(PrimitiveType::Float)) {
                    Err(type_mismatch_err(ty_inst, host_ty))
                } else {
                    Ok(())
                }
            },
            Instruction::TypeInt(ty) => {
                if !(ty.width == 32 && host_ty == &TypeDesc::Primitive(if ty.signedness == 1 { PrimitiveType::Int } else { PrimitiveType::UnsignedInt })) {
                    Err(type_mismatch_err(ty_inst, host_ty))
                } else {
                    Ok(())
                }
            },
            Instruction::TypeVector(ty) => {
                if let TypeDesc::Vector(host_comp_ty, host_comp_count) = host_ty {
                    let comp_ty = self.find_type(ty.component_id).unwrap();
                    let comp_count = ty.count;
                    assert!(ty.count <= 4);
                    self.compare_types(comp_ty, &TypeDesc::Primitive(host_comp_ty)).map_err(|e| {
                        type_mismatch_err(ty_inst, host_ty)
                    })?;
                    if comp_count != host_comp_count {
                        return Err(InterfaceError {
                            cause: None,
                            info: format!("vector size mismatch: {} (device) and {} (host)", comp_count, host_comp_count),
                        });
                    }
                    Ok(())
                } else {
                    Err(type_mismatch_err(ty_inst, host_ty))
                }
            },
            Instruction::TypeMatrix(ty) => {
                if let TypeDesc::Matrix(host_comp_ty, host_row_count, host_col_count) = host_ty {
                    let column_ty = self.find_type(ty.column_type_id).unwrap();
                    let col_count = ty.column_count;
                    if let Instruction::TypeVector(column_ty) = column_ty {
                        let comp_ty = self.find_type(column_ty.component_id).unwrap();
                        let row_count = column_ty.count;
                        self.compare_types(comp_ty, &TypeDesc::Primitive(host_comp_ty)).map_err(|e| {
                            type_mismatch_err(ty_inst, host_ty)
                        })?;
                        if !(row_count == host_row_count && col_count == host_col_count) {
                            return Err(type_mismatch_err(ty_inst, host_ty));
                        }
                        Ok(())
                    } else {
                        panic!("malformed SPIR-V bytecode")
                    }
                } else {
                    Err(type_mismatch_err(ty_inst, host_ty))
                }
            },
            Instruction::TypeStruct(ty) => {
                if let TypeDesc::Struct(ref host_members) = host_ty {
                    // build layout
                    let mut std140_layout = Std140LayoutBuilder::new();
                    let mut device_member_index = 0;
                    let mut host_member_index = 0;
                    for member_ty in ty.member_types {
                        let member_ty = self.find_type(member_ty).unwrap();
                        // get next member in reference struct
                        let &(host_member_offset, host_member_ty) =
                            if let Some(v) = member_iter.next() {
                                v
                            } else {
                                return Err(InterfaceError {
                                    cause: None,
                                    info: format!("struct layout mismatch: {:?} (device) and {:?} (host)", ty_inst, host_ty),
                                });
                            };

                        self.compare_types(member_ty, host_member_ty).map_err(|e| {
                            Err(InterfaceError {
                                cause: Some(Box::new(e)),
                                info: format!("member type mismatch: device: member #{}({}) | host: member #{}({})",
                                              device_member_index, "<unnamed>",
                                              host_member_index, "<unnamed>"),
                            })
                        })?;

                        let member_offset = std140_layout.add_member(self, member_ty);
                        if member_offset != host_member_offset {
                            return Err(InterfaceError {
                                cause: None,
                                info: format!("member offset mismatch: device: member index #{}({}), offset {} | host: member index #{}({}), offset: {}",
                                              device_member_index, "<unnamed>",
                                              member_offset,
                                              host_member_index, "<unnamed>",
                                              host_member_offset)
                            });
                        }

                        device_member_index += 1;
                        host_member_index += 1;
                    }
                    Ok(())
                } else {
                    Err(type_mismatch_err(ty_inst, host_ty))
                }
            },
            _ => { panic!("unsupported type") }
        }
    }
}

struct SpirvGraphicsPipelineModules
{
    vs: ModuleWrapper,
    fs: ModuleWrapper,
    gs: Option<ModuleWrapper>,
    tcs: Option<ModuleWrapper>,
    tes: Option<ModuleWrapper>,
}

enum VerifyResult
{
    NotFound,
    AlreadyMatched { loc: u32 },
    Match { loc: u32, bad_rematch: bool },
    Mismatch { loc: u32, err: InterfaceError, bad_rematch: bool },
}

macro_rules! try_verify {
    ($r:expr) => {
        match $r {
            VerifyResult::AlreadyMatched { .. } => { return false },
            VerifyResult::Mismatch { .. } => { return false },
            VerifyResult::Match { bad_rematch, .. } if bad_rematch => { return false },
            // Not found or correct match, continue (to detect potential bad rematches)
            _ => {}
        }};
}

impl SpirvGraphicsPipelineModules
{
    /// Look for the specified uniform constant in the shader modules
    /// and verify that the types on both sides (shader and host) match.
    /// If the host code specifies a location then the shader and host constants
    /// are matched by location (and the names are ignored), otherwise
    /// they are matched by name.
    ///
    /// This function stops searching as soon as a matching uniform is found in
    /// any shader. It does not detect potential different definitions of the
    /// same variable in different modules, which is a linker error.
    fn verify_named_uniform<'a>(&self, u: &'a UniformConstantDesc, matched_locations: &mut HashMap<u32, &'a UniformConstantDesc>) -> Result<(),InterfaceError> {
        let verify = |module: &ModuleWrapper| -> VerifyResult {
            for inst in module.0.instructions.iter() {
                // filter out anything that is not a variable
                if let Instruction::Variable(var) = inst {
                    // must have uniform storage class
                    if var.storage_class != spirv::StorageClass::Uniform { continue }
                    // get underlying location and check that it matches, otherwise skip
                    let loc = module.find_location_decoration(var.result_id);
                    let loc = if let Some(loc) = loc { loc } else { panic!("uniform constant with no location in SPIR-V") };
                    if let Some(host_loc) = u.index {
                        if loc != host_loc { continue }
                    } else {
                        // host code did not specify a location, try to match by name
                        // This is very brittle, as SPIR-V allows two uniforms with the same name, but different locations
                        let shader_name = module.find_name(var.result_id);
                        if let (Some(shader_name), Some(ref host_name)) = (shader_name, u.name) {
                            if shader_name != host_name {
                                continue
                            }
                        } else {
                            // cannot match, skip
                            continue
                        }
                    }

                    // we have a match, check if this location is already bound
                    let previous_match = matched_locations.get(&loc);

                    // must be a pointer (SPIR-V is malformed otherwise)
                    let ptr_ty = if let Some(Instruction::TypePointer(ref ptr_ty)) = module.find_type(var.result_type_id) { ptr_ty } else { panic!("malformed SPIR-V") };
                    // get underlying type
                    let uniform_ty_inst = module.find_type(ptr_ty.type_id).expect("malformed SPIR-V");
                    if let Err(e) = module.compare_types(uniform_ty_inst, u.ty) {
                        // ooops, this is a rematch of the same location, but it failed this time: possible linking error or ambiguous match by name?
                        let bad_rematch = if let Some((previous_match_was_successful, _)) = previous_match { true } else { false };
                        return VerifyResult::Mismatch { loc, err: e, bad_rematch }
                    } else {
                        let bad_rematch = if let Some((previous_match_was_successful, _)) = previous_match { false } else { true };
                        return VerifyResult::Match { loc, bad_rematch }
                    }
                }
            }
            return VerifyResult::NotFound
        };


        let mut found = false;
        let check_result = |r| match r {
            VerifyResult::AlreadyMatched { loc } => {
                Err(InterfaceError { cause: None, info: format!("binding location {} was already matched", loc) } )
            },
            VerifyResult::Mismatch { loc, err, bad_rematch } if bad_rematch == false => {
                Err(InterfaceError { cause: None, info: format!("interface mismatch for binding location {}", loc) } )
            },
            VerifyResult::Mismatch { loc, err, bad_rematch } if bad_rematch == true => {
                Err(InterfaceError { cause: None, info: format!("interface mismatch for binding location {} (was previously matched correctly: linking error?)", loc) } )
            },
            VerifyResult::Match { .. } => { found = true; Ok(()) },
            // Not found or correct match, continue (to detect potential bad rematches)
            VerifyResult::NotFound => Ok(())
        };

        check_result(verify(&self.vs))?;
        check_result(verify(&self.fs))?;
        if let Some(ref gs) = self.gs { check_result(verify(gs))? };
        if let Some(ref tcs) = self.tcs { check_result(verify(tcs))? };
        if let Some(ref tes) = self.tes { check_result(verify(tes))? };

        // should only be a warning, though
        if found { Ok(()) } else { Err(InterfaceError { cause: None, info: "uniform constant not found in shader".to_owned() } ) }
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
