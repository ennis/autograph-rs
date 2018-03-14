use super::texture_format::TextureFormat;

#[derive(Copy,Clone,Debug,Eq,PartialEq)]
pub enum PrimitiveType {
    Float,
    Int,
    UnsignedInt,
    Half,
    Double,
}

#[derive(Clone,Debug,Eq,PartialEq)]
pub enum Type {
    Primitive(PrimitiveType),
    Array(Box<Type>,usize),
    Vector(PrimitiveType,u8),
    Matrix(PrimitiveType,u8,u8),
    //Struct(String)
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

#[derive(Clone, Debug)]
pub struct RenderTargetDesc
{
    name: Option<String>,
    index: Option<i32>,
    format: Option<TextureFormat>
}

#[derive(Clone, Debug)]
pub struct NamedUniformDesc
{
    name: String,
    ty: Type
}

/// An input buffer for vertex data
#[derive(Clone, Debug)]
pub struct VertexBufferDesc
{
    name: Option<String>,
    index: Option<i32>,
    layout: &'static [Type]
}

/// An input buffer for indices
#[derive(Clone, Debug)]
pub struct IndexBufferDesc
{
    indexty: Type
}

/// Texture basic data type (NOT storage format)
#[derive(Copy, Clone, Debug)]
pub enum TextureDataType
{
    Float,  // and also depth
    Integer,
    UnsignedInteger
}

/// Represents a texture binding expected by a shader.
/// The shader should not specify a particular storage format but rather the component type
/// of the texture elements (either float, integer, or unsigned)
#[derive(Clone,Debug)]
pub struct TextureBindingDesc
{
    /// name of the texture binding in shader, can be None.
    /// name and index should not be both None
    name: Option<String>,
    /// index (texture uint) of the texture binding in shader, can be None.
    /// name and index should not be both None
    index: Option<i32>,
    /// Basic data type of the texture
    data_type: TextureDataType
}


///
/// Trait implemented by types that represent vertex data in a vertex buffer.
/// This is used to automatically infer the vertex layout.
///
/// ```rust
/// #[derive(VertexType)]
/// #[repr(C)]
/// struct MyVertexType {
///     position: Vec3,
///     normals: Vec3,
///     tangents: Vec3,
///     texcoords: Vec2,
/// }
/// ```
pub trait VertexType
{
    fn get_layout() -> &'static [Type];
}

/// Trait implemented by types that represent a shader interface.
/// A shader interface is a set of uniforms, vertex attributes, render targets
/// that describe the inputs and outputs of a graphics pipeline.
///
/// ```rust
/// #[derive(ShaderInterface)]
/// struct MyShaderInterface {
///     #[vertex_buffer(index=0)]
///     vbuf: VertexBuffer<MyVertexType>,    // a type-safe wrapper around a buffer
///     #[index_buffer]
///     ibuf: IndexBuffer<u16>,
///     #[texture(index=0)],
///     albedo: FloatTexture,
///     #[texture(index=1)],
///     specularMap: FloatTexture
/// }
/// ```
pub trait ShaderInterface
{
    fn get_named_uniforms() -> &'static [NamedUniformDesc];
    fn get_render_targets() -> &'static [RenderTargetDesc];
    fn get_vertex_buffers() -> &'static [VertexBufferDesc];
    fn get_index_buffer() -> Option<IndexBufferDesc>;
}

