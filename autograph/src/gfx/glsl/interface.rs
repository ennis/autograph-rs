use super::shader_interface::*;
use super::spirv_parse::*;
use super::SpirvGraphicsShaderPipeline;
use failure::{Compat, Error, Fail, ResultExt};
use gfx;
use spirv;
use std::cmp::max;
use std::collections::{hash_map::Entry, HashMap};

#[derive(Deref)]
struct ModuleWrapper(SpirvModule);

fn parse_module(bytecode: &[u32]) -> ModuleWrapper {
    ModuleWrapper(parse_spirv_u32s(bytecode).unwrap())
}

struct Std140LayoutBuilder {
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
    if multiple == 0 {
        return value;
    }
    let remainder = value % multiple;
    if remainder == 0 {
        return value;
    }
    value + multiple - remainder
}

impl Std140LayoutBuilder {
    fn new() -> Std140LayoutBuilder {
        Std140LayoutBuilder { next_offset: 0 }
    }

    fn align(&mut self, a: usize) -> usize {
        self.next_offset += align_offset(self.next_offset, a);
        self.next_offset
    }

    fn get_align_and_size(&self, module: &ModuleWrapper, inst: &Instruction) -> (usize, usize) {
        match *inst {
            Instruction::TypeBool(ref ty) => (4, 4),
            Instruction::TypeInt(ref ty) => {
                assert!(ty.width == 32);
                (4, 4)
            }
            Instruction::TypeFloat(ref ty) => {
                assert!(ty.width == 32);
                (4, 4)
            }
            Instruction::TypeVector(ref ty) => {
                let compty = module.find_type(ty.component_id).unwrap();
                let (_, n) = self.get_align_and_size(module, compty);
                match ty.count {
                    2 => (2 * n, 2 * n),
                    3 => (4 * n, 3 * n),
                    4 => (4 * n, 4 * n),
                    _ => panic!("unsupported vector size"),
                }
            }
            Instruction::TypeMatrix(ref ty) => {
                let column_ty = module.find_type(ty.column_type_id).unwrap();
                let (col_align, col_size) = self.get_align_and_size(module, column_ty);
                // alignment = column type align rounded up to vec4 align (16 bytes)
                let base_align = max(16, col_align);
                let stride = col_size + align_offset(col_size, col_align);
                // total array size = num columns * stride, rounded up to the next multiple of the base alignment
                let array_size = round_up(ty.column_count as usize * stride, base_align);
                (base_align, array_size)
            }
            Instruction::TypeImage(_) => panic!("unsupported type"),
            Instruction::TypeSampler(_) => panic!("unsupported type"),
            Instruction::TypeSampledImage(_) => panic!("unsupported type"),
            Instruction::TypeArray(ref ty) => panic!("unsupported type"),
            Instruction::TypeRuntimeArray(_) => panic!("unsupported type"),
            Instruction::TypeStruct(_) => panic!("unsupported type"),
            Instruction::TypeOpaque(_) => panic!("unsupported type"),
            Instruction::TypePointer(_) => panic!("unsupported type"),
            _ => panic!("not a type instruction"),
        }
    }

    fn add_member(&mut self, module: &ModuleWrapper, ty_inst: &Instruction) -> usize {
        let (align, size) = self.get_align_and_size(module, ty_inst);
        let current_offset = self.align(align);
        self.next_offset += size;
        current_offset
    }
}

#[derive(Fail, Debug)]
pub enum TypeCheckError {
    #[fail(display = "member offset mismatch")]
    MemberOffsetMismatch,
    #[fail(display = "member type mismatch")]
    TypeMismatch,
}

///
/// A shader resource: a resource that can be bound to any of the stages.
#[derive(Copy, Clone, Debug)]
enum ShaderResource<'a> {
    UniformBuffer(&'a UniformBufferDesc),
    ShaderStorageBuffer(&'a UniformBufferDesc),
    UniformConstant(&'a UniformConstantDesc),
    Texture(&'a TextureBindingDesc),
    Image(&'a TextureBindingDesc),
}

impl ModuleWrapper {
    fn find_decoration(&self, id: u32, deco: spirv::Decoration) -> Option<&IDecorate> {
        self.0.instructions.iter().find_map(|inst| match *inst {
            Instruction::Decorate(ref deco_inst) => {
                if deco_inst.target_id == id && deco_inst.decoration == deco {
                    Some(deco_inst)
                } else {
                    None
                }
            }
            _ => None,
        })
    }

    fn find_name(&self, id: u32) -> Option<&str> {
        self.0.instructions.iter().find_map(|inst| match *inst {
            Instruction::Name(IName {
                target_id,
                ref name,
            }) if target_id == id =>
            {
                Some(name.as_ref())
            }
            _ => None,
        })
    }

    fn find_type(&self, id: u32) -> Option<&Instruction> {
        self.0.instructions.iter().find(|&inst| match *inst {
            Instruction::TypeVoid(ITypeVoid { result_id }) if result_id == id => true,
            Instruction::TypeBool(ITypeBool { result_id }) if result_id == id => true,
            Instruction::TypeInt(ITypeInt { result_id, .. }) if result_id == id => true,
            Instruction::TypeFloat(ITypeFloat { result_id, .. }) if result_id == id => true,
            Instruction::TypeVector(ITypeVector { result_id, .. }) if result_id == id => true,
            Instruction::TypeMatrix(ITypeMatrix { result_id, .. }) if result_id == id => true,
            Instruction::TypeImage(ITypeImage { result_id, .. }) if result_id == id => true,
            Instruction::TypeSampler(ITypeSampler { result_id }) if result_id == id => true,
            Instruction::TypeSampledImage(ITypeSampledImage { result_id, .. })
                if result_id == id =>
            {
                true
            }
            Instruction::TypeArray(ITypeArray { result_id, .. }) if result_id == id => true,
            Instruction::TypeRuntimeArray(ITypeRuntimeArray { result_id, .. })
                if result_id == id =>
            {
                true
            }
            Instruction::TypeStruct(ITypeStruct { result_id, .. }) if result_id == id => true,
            Instruction::TypeOpaque(ITypeOpaque { result_id, .. }) if result_id == id => true,
            Instruction::TypePointer(ITypePointer { result_id, .. }) if result_id == id => true,
            _ => false,
        })
    }

    fn find_location_decoration(&self, id: u32) -> Option<u32> {
        self.find_decoration(id, spirv::Decoration::Location)
            .map(|deco| deco.params[0])
    }

    fn find_binding_decoration(&self, id: u32) -> Option<u32> {
        self.find_decoration(id, spirv::Decoration::Binding)
            .map(|deco| deco.params[0])
    }

    fn find_location_or_binding_decoration(&self, id: u32) -> Option<u32> {
        self.find_decoration(id, spirv::Decoration::Location)
            .map(|deco| deco.params[0])
            .or_else(|| {
                self.find_decoration(id, spirv::Decoration::Binding)
                    .map(|deco| deco.params[0])
            })
    }

    /*fn match_by_name_or_location(&self, id: u32, shader_binding_or_location: u32, host_name: Option<&str>, host_binding_or_location: Option<u32>) -> bool
    {
        if let Some(host_binding_or_location) = host_binding_or_location {
            shader_binding_or_location == host_binding_or_location
        } else {
            // host code did not specify a location, try to match by name
            // This is very brittle, as SPIR-V allows two uniforms with the same name, but different locations
            let shader_name = module.find_name(id);
            if let (Some(shader_name), Some(host_name)) = (shader_name, host_name) {
                shader_name == host_name
            } else {
                // cannot match, skip
                false
            }
        }
    }*/

    /// Finds a shader resource of the specified kind at the specified location
    fn verify_shader_resource<'a>(&'a self, sr: &ShaderResource) -> Result<bool, Error> {
        for inst in self.0.instructions.iter() {
            // filter out anything that is not a variable
            let var = if let Instruction::Variable(var) = inst {
                var
            } else {
                continue
            };
            // let location_or_binding = self.find_location_or_binding_decoration(var.result_id);
            // get the underlying type of the uniform
            // the primary type must be a pointer (the SPIR-V binary is malformed otherwise)
            let ptr_ty = if let Some(Instruction::TypePointer(ref ptr_ty)) =
                self.find_type(var.result_type_id)
            {
                ptr_ty
            } else {
                panic!("malformed SPIR-V")
            };
            // get underlying type
            let resource_ty_id = ptr_ty.type_id;
            let resource_ty_inst = self.find_type(resource_ty_id).expect("malformed SPIR-V");

            //eprintln!("{:?}", inst);

            match *sr {
                ShaderResource::UniformConstant(desc) => {
                    // looking for a uniform constant
                    let location = desc.index
                        .expect("must provide a location for uniform constants");
                    if var.storage_class != spirv::StorageClass::UniformConstant {
                        continue
                    }
                    let shader_loc = if let Some(loc) = self.find_location_decoration(var.result_id)
                    {
                        loc
                    } else {
                        continue
                    };
                    if shader_loc != location {
                        continue
                    }
                    self.compare_types(resource_ty_inst, desc.ty)?;
                    return Ok(true)
                }
                ShaderResource::UniformBuffer(desc) => {
                    // Storage class is uniform, and has a Block decoration
                    let binding = desc.index
                        .expect("must provide a buffer binding index for uniform buffers");
                    if !(var.storage_class == spirv::StorageClass::Uniform
                        && self.find_decoration(resource_ty_id, spirv::Decoration::Block)
                            .is_some())
                    {
                        continue
                    }
                    let shader_binding = self.find_binding_decoration(var.result_id)
                        .expect("uniform buffer with no binding");
                    if shader_binding != binding {
                        continue
                    }
                    self.compare_types(resource_ty_inst, desc.tydesc)?;
                    return Ok(true)
                }
                ShaderResource::ShaderStorageBuffer(desc) => {
                    // Storage class is uniform, and has a bufferBlock decoration (deprecated) OR Storage class is StorageBuffer
                    let binding = desc.index
                        .expect("must provide a buffer binding index for shader storage buffers");
                    if !((var.storage_class == spirv::StorageClass::Uniform
                        && self.find_decoration(resource_ty_id, spirv::Decoration::BufferBlock)
                            .is_some())
                        || var.storage_class == spirv::StorageClass::StorageBuffer)
                    {
                        continue
                    }
                    let shader_binding = self.find_binding_decoration(var.result_id)
                        .expect("SSBO with no binding");
                    if shader_binding != binding {
                        continue
                    }
                    unimplemented!()
                }
                ShaderResource::Image(desc) => {
                    // storage class is uniformconstant
                    if var.storage_class != spirv::StorageClass::UniformConstant {
                        continue
                    }
                    let binding = desc.index.expect("must provide an image unit");
                    // type is TypeImage
                    if let &Instruction::TypeImage(_) = resource_ty_inst {
                        let shader_binding = self.find_binding_decoration(var.result_id)
                            .expect("image with no binding in shader");
                        if shader_binding != binding {
                            continue
                        }
                        unimplemented!()
                    } else {
                        continue
                    }
                }
                ShaderResource::Texture(desc) => {
                    // storage class is uniformconstant
                    if var.storage_class != spirv::StorageClass::UniformConstant {
                        continue
                    }
                    let binding = desc.index.expect("must provide a texture unit");
                    // type is TypeSampledImage
                    if let &Instruction::TypeSampledImage(ref sampled_image) = resource_ty_inst {
                        let shader_binding = self.find_binding_decoration(var.result_id)
                            .expect("texture with no binding in shader");
                        if shader_binding != binding {
                            continue
                        }
                        // load OpTypeImage and check dimensions
                        let image_ty = self.find_type(sampled_image.image_type_id).unwrap();
                        if let &Instruction::TypeImage(ref img) = image_ty {
                            let dim_ok = match desc.dimensions {
                                Some(gfx::TextureDimensions::Tex1D) => img.dim == spirv::Dim::Dim1D,
                                Some(gfx::TextureDimensions::Tex2D) => img.dim == spirv::Dim::Dim2D,
                                Some(gfx::TextureDimensions::Tex3D) => img.dim == spirv::Dim::Dim3D,
                                Some(gfx::TextureDimensions::TexCube) => {
                                    img.dim == spirv::Dim::DimCube
                                }
                                None => true,
                                _ => bail!("unsupported texture dimension"),
                            };
                            if !dim_ok {
                                bail!(
                                    "texture dimensions do not match (shader: {:?}, host: {:?})",
                                    img.dim,
                                    desc.dimensions
                                );
                            }
                            return Ok(true)
                        } else {
                            panic!("malformed SPIR-V")
                        }
                    } else {
                        continue
                    }
                }
            };
        }

        Ok(false)
    }

    /// Check that the type described by the spirv instruction
    /// is layout-compatible with the given host type description
    fn compare_types(&self, ty_inst: &Instruction, host_ty: &TypeDesc) -> Result<(), Error> {
        match *ty_inst {
            //Instruction::TypePointer(ptr) => { panic!()}, // no pointers in interface, for now
            Instruction::TypeFloat(ref ty) => {
                if !(ty.width == 32 && host_ty == &TypeDesc::Primitive(PrimitiveType::Float)) {
                    bail!(
                            "type mismatch: {:?} (shader) and {:?} (host)",
                        ty_inst, host_ty)
                } else {
                    Ok(())
                }
            }
            Instruction::TypeInt(ref ty) => {
                if !(ty.width == 32 && host_ty == &TypeDesc::Primitive(if ty.signedness {
                    PrimitiveType::Int
                } else {
                    PrimitiveType::UnsignedInt
                })) {
                    bail!(
                            "type mismatch: {:?} (shader) and {:?} (host)",
                        ty_inst, host_ty)
                } else {
                    Ok(())
                }
            }
            Instruction::TypeVector(ref ty) => {
                if let TypeDesc::Vector(host_comp_ty, host_comp_count) = host_ty {
                    let comp_ty = self.find_type(ty.component_id).unwrap();
                    let comp_count = ty.count;
                    assert!(ty.count <= 4);
                    self.compare_types(comp_ty, &TypeDesc::Primitive(*host_comp_ty))
                        .context(format!(
                            "type mismatch: {:?} (shader) and {:?} (host)",
                            ty_inst, host_ty
                        ))?;
                    if comp_count != *host_comp_count as u32 {
                         bail!("vector size mismatch: {} (shader) and {} (host)",
                                comp_count, host_comp_count)
                    }
                    Ok(())
                } else {
                    bail!("type mismatch: {:?} (shader) and {:?} (host)",
                        ty_inst, host_ty)
                }
            }
            Instruction::TypeMatrix(ref ty) => {
                if let TypeDesc::Matrix(host_comp_ty, host_row_count, host_col_count) = host_ty {
                    let column_ty = self.find_type(ty.column_type_id).unwrap();
                    let col_count = ty.column_count;
                    if let Instruction::TypeVector(column_ty) = column_ty {
                        let comp_ty = self.find_type(column_ty.component_id).unwrap();
                        let row_count = column_ty.count;
                        self.compare_types(comp_ty, &TypeDesc::Primitive(*host_comp_ty))
                            .context(format!(
                                "type mismatch: {:?} (shader) and {:?} (host)",
                                ty_inst, host_ty
                            ))?;
                        if !(row_count == *host_row_count as u32
                            && col_count == *host_col_count as u32)
                        {
                            return Err(TypeCheckError::TypeMismatch
                                .context(format!(
                                    "type mismatch: {:?} (shader) and {:?} (host)",
                                    ty_inst, host_ty
                                ))
                                .into());
                        }
                        Ok(())
                    } else {
                        panic!("malformed SPIR-V bytecode")
                    }
                } else {
                    return Err(TypeCheckError::TypeMismatch
                        .context(format!(
                            "type mismatch: {:?} (shader) and {:?} (host)",
                            ty_inst, host_ty
                        ))
                        .into());
                }
            }
            Instruction::TypeStruct(ref ty) => {
                if let TypeDesc::Struct(ref host_members) = host_ty {
                    // build layout
                    let mut std140_layout = Std140LayoutBuilder::new();
                    let mut device_member_index = 0;
                    let mut host_member_index = 0;
                    for &member_ty in ty.member_types.iter() {
                        let member_ty = self.find_type(member_ty).unwrap();
                        // get next member in reference struct
                        let (host_member_offset, host_member_ty) =
                            if let Some(v) = host_members.get(host_member_index) {
                                v
                            } else {
                                return Err(TypeCheckError::MemberOffsetMismatch
                                    .context(format!(
                                        "struct layout mismatch: {:?} (shader) and {:?} (host)",
                                        ty_inst, host_ty
                                    ))
                                    .into());
                            };

                        self.compare_types(member_ty, host_member_ty)
                            .context(format!(
                            "member type mismatch: #{}({}) (shader) | #{}({}) (host)",
                            device_member_index, "<unnamed>", host_member_index, "<unnamed>"
                        ))?;

                        let member_offset = std140_layout.add_member(self, member_ty);
                        if member_offset != *host_member_offset {
                            return Err(TypeCheckError::MemberOffsetMismatch.context(
                                format!("member offset mismatch: #{}({}) @ {} (shader) and #{}({}) @ {} (host)",
                                        device_member_index, "<unnamed>",
                                        member_offset,
                                        host_member_index, "<unnamed>",
                                        host_member_offset)).into());
                        }

                        device_member_index += 1;
                        host_member_index += 1;
                    }
                    Ok(())
                } else {
                    return Err(TypeCheckError::TypeMismatch
                        .context(format!(
                            "type mismatch: {:?} (shader) and {:?} (host)",
                            ty_inst, host_ty
                        ))
                        .into());
                }
            }
            _ => panic!("unsupported type"),
        }
    }
}

pub struct SpirvGraphicsPipelineModules {
    vs: ModuleWrapper,
    fs: ModuleWrapper,
    gs: Option<ModuleWrapper>,
    tcs: Option<ModuleWrapper>,
    tes: Option<ModuleWrapper>,
}

// texture -> binding (texture unit)
// image -> binding (image unit)
// ubo -> binding
// ssbo -> binding
// uniform constant -> location
// input -> location
// output -> location

#[derive(Fail, Debug)]
#[fail(display = "Shader interface verification error")]
pub struct ShaderInterfaceVerificationError(pub Vec<Error>);

impl SpirvGraphicsPipelineModules {
    /// Look for the specified uniform constant in the shader modules
    /// and verify that the types on both sides (shader and host) match.
    /// If the host code specifies a location then the shader and host constants
    /// are matched by location (and the names are ignored), otherwise
    /// they are matched by name.
    ///
    /// This function stops searching as soon as a matching uniform is found in
    /// any shader. It does not detect potential different definitions of the
    /// same variable in different modules, which is a linker error.
    ///
    fn verify_shader_resource<'a>(&self, sr: &ShaderResource<'a>) -> Result<(), Error> {
        let found_in_vs = self.vs.verify_shader_resource(sr)?;
        let found_in_fs = self.fs.verify_shader_resource(sr)?;
        let found_in_gs = if let Some(ref gs) = self.gs {
            gs.verify_shader_resource(sr)?
        } else {
            false
        };
        let found_in_tcs = if let Some(ref tcs) = self.tcs {
            tcs.verify_shader_resource(sr)?
        } else {
            false
        };
        let found_in_tes = if let Some(ref tes) = self.tes {
            tes.verify_shader_resource(sr)?
        } else {
            false
        };

        let mut found = found_in_vs | found_in_fs | found_in_gs | found_in_tcs | found_in_tes;

        debug!(
            "Shader resource {:?} {} {} {} {} {}",
            sr,
            if found_in_vs { "VS" } else { "vs" },
            if found_in_fs { "FS" } else { "fs" },
            if found_in_gs { "GS" } else { "gs" },
            if found_in_tcs { "TCS" } else { "tcs" },
            if found_in_tes { "TES" } else { "tes" },
        );

        if !found {
            eprintln!(
                "Could not find a corresponding shader resource for {:?}",
                sr
            );
        }

        Ok(())
    }

    fn verify(
        &self,
        interface: &ShaderInterfaceDesc,
    ) -> Result<(), ShaderInterfaceVerificationError> {
        let mut errors = Vec::new();
        for u in interface.get_uniform_constants().iter() {
            if let Err(e) = self.verify_shader_resource(&ShaderResource::UniformConstant(u)) {
                errors.push(e);
            }
        }
        for tex in interface.get_texture_bindings().iter() {
            if let Err(e) = self.verify_shader_resource(&ShaderResource::Texture(tex)) {
                errors.push(e);
            }
        }
        for ubo in interface.get_uniform_buffers().iter() {
            if let Err(e) = self.verify_shader_resource(&ShaderResource::UniformBuffer(ubo)) {
                errors.push(e);
            }
        }
        if !errors.is_empty() {
            Err(ShaderInterfaceVerificationError(errors))
        } else {
            Ok(())
        }
    }
}

pub fn verify_spirv_interface(
    interface: &ShaderInterfaceDesc,
    vert_bytecode: &[u32],
    frag_bytecode: &[u32],
    geom_bytecode: Option<&[u32]>,
    tess_control_bytecode: Option<&[u32]>,
    tess_eval_bytecode: Option<&[u32]>,
) -> Result<(), ShaderInterfaceVerificationError> {
    let vs_module = parse_module(vert_bytecode);
    let fs_module = parse_module(frag_bytecode);
    let gs_module = geom_bytecode.map(|bytecode| parse_module(bytecode));
    let tcs_module = tess_control_bytecode.map(|bytecode| parse_module(bytecode));
    let tes_module = tess_eval_bytecode.map(|bytecode| parse_module(bytecode));

    let modules = SpirvGraphicsPipelineModules {
        vs: vs_module,
        fs: fs_module,
        gs: gs_module,
        tcs: tcs_module,
        tes: tes_module,
    };

    modules.verify(interface)?;
    Ok(())
}
