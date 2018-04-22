use num_traits::FromPrimitive;
use parse;
use spirv;
use std::collections::HashMap;

#[derive(Copy, Clone, Debug)]
pub enum PrimitiveType {
    Int,
    Float,
    Bool,
    Void,
}

#[derive(Clone, Debug)]
pub struct StructMember {
    pub struct_id: u32,
    pub index: u32,
    pub name: Option<String>,
    pub ty: u32,
    pub builtin: Option<spirv::BuiltIn>, // TODO locations, etc: they can also be attached to struct members if it's an interface block
}

#[derive(Clone, Debug)]
pub struct Struct {
    pub id: u32,
    pub members: Vec<StructMember>,
    pub block: Option<spirv::Decoration>, // is it an interface block
}

impl Struct {
    pub fn has_builtin_members(&self) -> bool {
        self.members
            .iter()
            .fold(false, |a, m| a || m.builtin.is_some())
    }
}

// 'unfolded' type description
#[derive(Clone, Debug)]
pub enum Type {
    Primitive(PrimitiveType),
    Vector(PrimitiveType, i8),
    Matrix(PrimitiveType, i8, i8), // R,C
    Array(Box<Type>, usize),
    Struct(u32), // struct type-ID
    Pointer(Box<Type>),
}

// TODO should be a vec of decorations
#[derive(Debug)]
pub struct VariableDecorations {
    pub location: Option<u32>,
    pub descriptor: Option<(u32, u32)>,
    pub input_attachment_index: Option<u32>,
    pub constant_id: Option<u32>,
    pub builtin: Option<spirv::BuiltIn>,
}

#[derive(Debug)]
pub struct Variable {
    pub id: u32,
    pub name: Option<String>,
    pub storage_class: spirv::StorageClass,
    pub ty: u32,
    pub deco: VariableDecorations,
}

#[derive(Clone, Debug)]
pub struct EntryPoint {
    pub execution_model: spirv::ExecutionModel,
    pub name: String,
    pub interface: Vec<u32>,
}

pub fn parse_entry_points(doc: &parse::Spirv, entry_points: &mut HashMap<u32, EntryPoint>) {
    for instruction in doc.instructions.iter() {
        match instruction {
            &parse::Instruction::EntryPoint {
                execution,
                id,
                ref name,
                ref interface,
            } => {
                entry_points.insert(
                    id,
                    EntryPoint {
                        execution_model: execution,
                        name: name.clone(),
                        interface: interface.clone(),
                    },
                );
            }
            _ => (),
        }
    }
}

pub fn parse_variables(doc: &parse::Spirv, variables: &mut HashMap<u32, Variable>) {
    for instruction in doc.instructions.iter() {
        match instruction {
            &parse::Instruction::Variable {
                result_type_id,
                result_id,
                storage_class,
                initializer,
            } => {
                // get decorations
                let deco = VariableDecorations {
                    location: find_decoration(doc, result_id, spirv::Decoration::Location)
                        .map(|op| op[0]),
                    builtin: find_decoration(doc, result_id, spirv::Decoration::BuiltIn)
                        .map(|op| spirv::BuiltIn::from_u32(op[0]).unwrap()),
                    descriptor: {
                        let ds = find_decoration(doc, result_id, spirv::Decoration::DescriptorSet)
                            .map(|op| op[0]);
                        let binding = find_decoration(doc, result_id, spirv::Decoration::Binding)
                            .map(|op| op[0]);
                        match (ds, binding) {
                            (Some(ds), Some(binding)) => Some((ds, binding)),
                            (_, _) => None,
                        }
                    },
                    input_attachment_index: find_decoration(
                        doc,
                        result_id,
                        spirv::Decoration::InputAttachmentIndex,
                    ).map(|op| op[0]),
                    constant_id: None, // TODO
                };
                variables.insert(
                    result_id,
                    Variable {
                        storage_class: storage_class,
                        id: result_id,
                        ty: result_type_id,
                        name: find_name(doc, result_id),
                        deco,
                    },
                );
            }
            _ => (),
        }
    }
}

pub fn find_decoration<'a>(
    doc: &'a parse::Spirv,
    id: u32,
    deco: spirv::Decoration,
) -> Option<&'a [u32]> {
    for instruction in doc.instructions.iter() {
        match instruction {
            &parse::Instruction::Decorate {
                target_id,
                decoration,
                ref params,
            } if target_id == id && decoration == deco =>
            {
                return Some(params);
            }
            _ => (),
        }
    }

    None
}

pub fn find_member_decoration<'a>(
    doc: &'a parse::Spirv,
    id: u32,
    member_index: u32,
    deco: spirv::Decoration,
) -> Option<&'a [u32]> {
    for instruction in doc.instructions.iter() {
        match instruction {
            &parse::Instruction::MemberDecorate {
                target_id,
                member,
                decoration,
                ref params,
            } if target_id == id && member == member_index && decoration == deco =>
            {
                println!("Found member decoration {:?}", instruction);
                return Some(params);
            }
            _ => (),
        }
    }

    None
}

pub fn find_name(doc: &parse::Spirv, id: u32) -> Option<String> {
    for instruction in doc.instructions.iter() {
        match instruction {
            &parse::Instruction::Name {
                target_id,
                ref name,
            } if target_id == id =>
            {
                return Some(name.clone());
            }
            _ => (),
        }
    }

    None
}

pub fn find_member_name(doc: &parse::Spirv, struct_id: u32, member_index: u32) -> Option<String> {
    // find membername annotation
    for instruction in doc.instructions.iter() {
        match instruction {
            &parse::Instruction::MemberName {
                target_id,
                member,
                ref name,
            } if target_id == struct_id && member == member_index =>
            {
                return Some(name.clone())
            }
            _ => (),
        }
    }

    None
}

pub fn parse_struct(doc: &parse::Spirv, id: u32) -> Struct {
    for instruction in doc.instructions.iter() {
        match instruction {
            &parse::Instruction::TypeStruct {
                result_id,
                ref member_types,
            } if result_id == id =>
            {
                return Struct {
                    id,
                    block: find_decoration(doc, id, spirv::Decoration::Block)
                        .map(|_| spirv::Decoration::Block),
                    members: member_types
                        .iter()
                        .enumerate()
                        .map(|(index, ty)| StructMember {
                            name: find_member_name(doc, id, index as u32),
                            struct_id: id,
                            ty: *ty,
                            index: index as u32,
                            builtin: find_member_decoration(
                                doc,
                                id,
                                index as u32,
                                spirv::Decoration::BuiltIn,
                            ).map(|v| spirv::BuiltIn::from_u32(v[0]).unwrap()),
                        })
                        .collect(),
                };
            }
            _ => (),
        }
    }

    panic!("Struct not found")
}

pub fn get_constant_bits(doc: &parse::Spirv, id: u32) -> u64 {
    for instruction in doc.instructions.iter() {
        match instruction {
            &parse::Instruction::Constant {
                result_id,
                result_type_id,
                ref data,
                ..
            } if result_id == id =>
            {
                assert!(data.len() < 2);
                let data = data.iter().rev().fold(0u64, |a, &b| (a << 32) | b as u64);
                return data;
            }
            _ => (),
        }
    }

    panic!("Constant not found")
}

pub fn type_from_id(doc: &parse::Spirv, searched: u32) -> Type {
    for instruction in doc.instructions.iter() {
        match instruction {
            &parse::Instruction::TypeInt {
                result_id,
                width,
                signedness,
            } if result_id == searched =>
            {
                return Type::Primitive(PrimitiveType::Int);
            }

            &parse::Instruction::TypeFloat { result_id, width } if result_id == searched => {
                return Type::Primitive(PrimitiveType::Float);
            }

            &parse::Instruction::TypeVector {
                result_id,
                component_id,
                count,
            } if result_id == searched =>
            {
                let component_ty = type_from_id(doc, component_id);
                if let Type::Primitive(prim_ty) = component_ty {
                    return Type::Vector(prim_ty, count as i8);
                } else {
                    panic!("Unexpected vector component type")
                }
            }

            &parse::Instruction::TypeMatrix {
                result_id,
                column_type_id,
                column_count,
            } if result_id == searched =>
            {
                let column_ty = type_from_id(doc, column_type_id);
                if let Type::Vector(prim_ty, row_count) = column_ty {
                    return Type::Matrix(prim_ty, row_count, column_count as i8);
                } else {
                    panic!("Unexpected matrix column type")
                }
            }

            &parse::Instruction::TypeArray {
                result_id,
                type_id,
                length_id,
            } if result_id == searched =>
            {
                let elem_ty = type_from_id(doc, type_id);
                let length = get_constant_bits(doc, length_id) as usize;
                return Type::Array(Box::new(elem_ty), length);
            }

            &parse::Instruction::TypeStruct {
                result_id,
                ref member_types,
            } if result_id == searched =>
            {
                return Type::Struct(result_id);
            }

            &parse::Instruction::TypePointer {
                result_id, type_id, ..
            } if result_id == searched =>
            {
                return Type::Pointer(Box::new(type_from_id(doc, type_id)));
            }

            _ => (),
        }
    }

    unimplemented!()
}
