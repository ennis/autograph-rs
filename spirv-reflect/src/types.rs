use parse;
use spirv;

#[derive(Copy,Clone,Debug)]
pub enum PrimitiveType
{
    Int,
    Float,
    Bool,
    Void
}

#[derive(Clone,Debug)]
pub struct Struct
{
    id: u32,
    members: Vec<(Option<String>, Type)>
}

#[derive(Clone,Debug)]
pub enum Type
{
    Primitive(PrimitiveType),
    Vector(PrimitiveType, i8),
    Matrix(PrimitiveType, i8, i8),  // R,C
    Array(Box<Type>, usize),
    Struct(u32)     // struct type-ID
}

pub fn find_decoration<'a>(doc: &'a parse::Spirv, id: u32, deco: spirv::Decoration) -> Option<&'a [u32]>
{
    for instruction in doc.instructions.iter() {
        match instruction {
            &parse::Instruction::Decorate {
                target_id,
                decoration,
                ref params
            } if target_id == id && decoration == deco => {
                return Some(params);
            },
            _ => ()
        }
    }

    None
}

pub fn find_name(doc: &parse::Spirv, id: u32) -> Option<String>
{
    for instruction in doc.instructions.iter() {
        match instruction {
            &parse::Instruction::Name {
                target_id,
                ref name,
            } if target_id == id => {
                return Some(name.clone());
            },
            _ => (),
        }
    }

    None
}

// TODO: resolve initializers?
pub fn describe_variable(doc: &parse::Spirv, id: u32) -> (Option<String>, Type, spirv::StorageClass)
{
    for instruction in doc.instructions.iter() {
        match instruction {
            &parse::Instruction::Variable {
                result_type_id,
                result_id,
                storage_class,
                initializer
            } if result_id == id => {
                return (find_name(doc, result_id), type_from_id(doc, result_type_id), storage_class);
            },
            _ => (),
        }
    }

    panic!("Variable not found")
}



pub fn describe_struct_member(doc: &parse::Spirv, struct_id: u32, member_type_id: u32, member_index: u32) -> (Option<String>, Type)
{
    // find membername annotation
    let mut member_name = None;

    for instruction in doc.instructions.iter() {
        match instruction {
            &parse::Instruction::MemberName {
                target_id,
                member,
                ref name,
            } if target_id == struct_id && member == member_index => {
                member_name = Some(name.clone());
            },
            _ => (),
        }
    };

    (member_name, type_from_id(doc, member_type_id))
}

pub fn describe_struct(doc: &parse::Spirv, id: u32) -> Struct
{
    for instruction in doc.instructions.iter() {
        match instruction {
            &parse::Instruction::TypeStruct {
                result_id,
                ref member_types
            } if result_id == id => {
                return Struct {
                    id,
                    members: member_types.iter().enumerate().map(|(index, member_type_id)| {
                        describe_struct_member(doc, id, *member_type_id, index as u32)
                    }).collect()
                };
            },
            _ => (),
        }
    }

    panic!("Struct not found")
}

pub fn get_constant_bits(doc: &parse::Spirv, id: u32) -> u64
{
    for instruction in doc.instructions.iter() {
        match instruction {
            &parse::Instruction::Constant {
                result_id,
                result_type_id,
                ref data,
                ..
            } if result_id == id => {
                assert!(data.len() < 2);
                let data = data.iter().rev().fold(0u64, |a, &b| (a << 32) | b as u64);
                return data;
            },
            _ => ()
        }
    }

    panic!("Constant not found")
}

fn type_from_id(doc: &parse::Spirv, searched: u32) -> Type
{
    for instruction in doc.instructions.iter() {
        match instruction {
            &parse::Instruction::TypeInt {
                result_id,
                width,
                signedness,
            } if result_id == searched => {
                return Type::Primitive(PrimitiveType::Int);
            },

            &parse::Instruction::TypeFloat { result_id, width } if result_id == searched => {
                return Type::Primitive(PrimitiveType::Float);
            },

            &parse::Instruction::TypeVector {
                result_id,
                component_id,
                count,
            } if result_id == searched => {
                let component_ty = type_from_id(doc, component_id);
                if let Type::Primitive(prim_ty) = component_ty {
                    return Type::Vector(prim_ty, count as i8);
                } else {
                    panic!("Unexpected vector component type")
                }
            },

            &parse::Instruction::TypeMatrix {
                result_id,
                column_type_id,
                column_count,
            } if result_id == searched => {
                let column_ty = type_from_id(doc, column_type_id);
                if let Type::Vector(prim_ty, row_count) = column_ty {
                    return Type::Matrix(prim_ty, row_count, column_count as i8)
                } else {
                    panic!("Unexpected matrix column type")
                }
            },

            &parse::Instruction::TypeArray {
                result_id,
                type_id,
                length_id,
            } if result_id == searched => {
                let elem_ty = type_from_id(doc, type_id);
                let length = get_constant_bits(doc, length_id) as usize;
                return Type::Array(Box::new(elem_ty), length);
            },

            &parse::Instruction::TypeStruct {
                result_id,
                ref member_types,
            } if result_id == searched => {
                return Type::Struct(result_id);
            },

            &parse::Instruction::TypePointer {
                result_id, type_id, ..
            }  if result_id == searched => {
                unimplemented!()
            },

            _ => (),
        }
    }

    unimplemented!()
}