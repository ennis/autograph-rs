//! Types

#[derive(Copy,Clone,Debug,Eq,PartialEq)]
pub enum PrimitiveType {
    Float,
    Int,
    UnsignedInt,
    Half,
    Double,
    Sampler2D,
}

#[derive(Debug)]
pub enum Value {
    Ident(String),
    Int(i64),
    Float(f64),
    Str(String),
}

#[derive(Debug)]
pub enum Metadata {
    Custom(String, Vec<Value>)
}

#[derive(Debug)]
pub struct StructMember {
    pub ty: String,
    pub name: String,
    pub metadata: Vec<Metadata>
}

#[derive(Debug)]
pub struct Struct {
    pub name: String,
    pub members: Vec<StructMember>,
    pub metadata: Vec<Metadata>
}

#[derive(Clone,Debug,Eq,PartialEq)]
pub enum Type {
    Primitive(PrimitiveType),
    Array(Box<Type>,usize),
    Vector(PrimitiveType,u8),
    Matrix(PrimitiveType,u8,u8),
    Struct(String)
}

pub const TYPE_FLOAT: Type = Type::Primitive(PrimitiveType::Float);
pub const TYPE_INT: Type = Type::Primitive(PrimitiveType::Int);
pub const TYPE_VEC2: Type = Type::Vector(PrimitiveType::Float, 2);
pub const TYPE_VEC3: Type = Type::Vector(PrimitiveType::Float, 3);
pub const TYPE_VEC4: Type = Type::Vector(PrimitiveType::Float, 4);
pub const TYPE_IVEC2: Type = Type::Vector(PrimitiveType::Int, 2);
pub const TYPE_IVEC3: Type = Type::Vector(PrimitiveType::Int, 3);
pub const TYPE_IVEC4: Type = Type::Vector(PrimitiveType::Int, 4);
pub const TYPE_MAT2: Type = Type::Matrix(PrimitiveType::Float, 2, 2);
pub const TYPE_MAT3: Type = Type::Matrix(PrimitiveType::Float, 3, 3);
pub const TYPE_MAT4: Type = Type::Matrix(PrimitiveType::Float, 4, 4);
pub const TYPE_SAMPLER2D: Type = Type::Primitive(PrimitiveType::Sampler2D);

// get type from a shorthand like rgb16f, rgba32f, etc.