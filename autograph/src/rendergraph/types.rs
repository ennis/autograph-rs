//! Types

#[derive(Copy,Clone,Debug)]
enum PrimitiveType {
    Float,
    Int,
    UnsignedInt,
    Half,
    Double,
}

#[derive(Clone,Debug)]
struct StructMember {
    name: String,
    ty: Type
}

#[derive(Clone,Debug)]
struct StructType {
    members: Vec<StructMember>
}

#[derive(Clone,Debug)]
enum Type {
    Primitive(PrimitiveType),
    Array(Box<Type>,usize),
    Vector(PrimitiveType,u8),
    Matrix(PrimitiveType,u8,u8),
    Struct(StructType)
}

// get type from a shorthand like rgb16f, rgba32f, etc.