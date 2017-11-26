//! Types

#[derive(Copy,Clone,Debug)]
enum PrimitiveType {
    Float,
    Int,
    UnsignedInt,
    Half,
    Double,
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

#[derive(Clone,Debug)]
enum Type {
    Primitive(PrimitiveType),
    Array(Box<Type>,usize),
    Vector(PrimitiveType,u8),
    Matrix(PrimitiveType,u8,u8),
    Struct(String)
}

// get type from a shorthand like rgb16f, rgba32f, etc.