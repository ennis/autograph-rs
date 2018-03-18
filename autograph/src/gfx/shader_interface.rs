use super::format::Format;
use super::buffer_data::BufferData;
use super::state_cache::StateCache;
use super::shader::UniformBinder;
use super::pipeline::GraphicsPipeline;
use failure::Error;

#[derive(Copy,Clone,Debug,Eq,PartialEq)]
pub enum PrimitiveType {
    Int,
    UnsignedInt,
    Half,
    Float,
    Double,
}

#[derive(Clone,Debug,Eq,PartialEq)]
pub enum Type {
    Primitive(PrimitiveType),
    Array(Box<Type>,usize),
    Vector(PrimitiveType,u8),
    Matrix(PrimitiveType,u8,u8),
    Unknown
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

// vertex type: interpretation (FLOAT,UNORM,SNORM,INTEGER)

#[derive(Clone, Debug)]
pub struct RenderTargetDesc
{
    pub name: Option<String>,
    pub index: Option<i32>,
    pub format: Option<Format>
}

#[derive(Clone, Debug)]
pub struct NamedUniformDesc
{
    pub name: String,
    pub ty: Type
}

/// An input buffer for vertex data
#[derive(Clone, Debug)]
pub struct VertexBufferDesc
{
    pub name: Option<String>,
    pub index: Option<i32>,
    pub layout: &'static VertexLayout
}

/// An input buffer for indices
#[derive(Clone, Debug)]
pub struct IndexBufferDesc
{
    pub name: Option<String>,
    pub index_ty: Type
}

/// Texture basic data type (NOT storage format)
#[derive(Copy, Clone, Debug)]
pub enum TextureDataType
{
    Float,  // and also depth
    Integer,
    UnsignedInteger,
    Unknown
}

/// Represents a texture binding expected by a shader.
/// The shader should not specify a particular storage format but rather the component type
/// of the texture elements (either float, integer, or unsigned)
#[derive(Clone,Debug)]
pub struct TextureBindingDesc
{
    /// name of the texture binding in shader, can be None.
    /// name and index should not be both None
    pub name: Option<String>,
    /// index (texture uint) of the texture binding in shader, can be None.
    /// name and index should not be both None
    pub index: Option<i32>,
    /// Basic data type of the texture
    pub data_type: TextureDataType
}

/// Trait implemented by types that can serve as a vertex element.
pub unsafe trait VertexElementType {
    /// Returns the equivalent OpenGL type (the type seen by the shader).
    fn get_equivalent_type() -> Type;
    /// Returns the corresponding data format (the layout of the data in memory).
    fn get_format() -> Format;
}

macro_rules! impl_vertex_element_type {
    ($t:ty, $equiv:expr, $fmt:ident) => {
        unsafe impl VertexElementType for $t {
            fn get_equivalent_type() -> Type { $equiv }
            fn get_format() -> Format { Format::$fmt }
        }
    };
}

impl_vertex_element_type!(f32, Type::Primitive(PrimitiveType::Float), R32_SFLOAT);
impl_vertex_element_type!([f32;2], Type::Vector(PrimitiveType::Float,2), R32G32_SFLOAT);
impl_vertex_element_type!([f32;3], Type::Vector(PrimitiveType::Float,3), R32G32B32_SFLOAT);
impl_vertex_element_type!([f32;4], Type::Vector(PrimitiveType::Float,4), R32G32B32A32_SFLOAT);

/// Description of a vertex attribute.
#[derive(Clone,Debug)]
pub struct VertexAttributeDesc
{
    /// Attribute name.
    pub name: Option<String>,
    /// Location.
    pub loc: u8,
    /// The equivalent OpenGL type.
    pub ty: Type,
    /// Storage format of the vertex attribute.
    pub format: Format,
    /// Relative offset.
    pub offset: u8
}

/// The layout of vertex data in a vertex buffer.
#[derive(Clone, Debug)]
pub struct VertexLayout
{
    pub attributes: &'static [VertexAttributeDesc],
    pub stride: usize
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
pub trait VertexType: BufferData
{
    fn get_layout() -> &'static VertexLayout;
}

/// Descriptions of shader interfaces.
///
/// This trait is a facade to recover information about the bindings defined in a shader interface.
/// It is meant to be derived automatically with `#[derive(ShaderInterface)]`, but you can implement it by hand.
///
/// TODO replace it with a simple struct?
pub trait ShaderInterfaceDesc: Sync + 'static
{
    /// Returns the list of named uniform items (`#[named_uniform]`)
    fn get_named_uniforms(&self) -> &[NamedUniformDesc];
    /// Returns the list of render target items (`#[render_target(...)]`)
    fn get_render_targets(&self) -> &[RenderTargetDesc];
    /// Returns the list of vertex buffer items (`#[vertex_buffer(index=...)]`)
    fn get_vertex_buffers(&self) -> &[VertexBufferDesc];
    /// Returns the index buffer item, if any (`#[index_buffer]`)
    fn get_index_buffer(&self) -> Option<IndexBufferDesc>;
    /// Returns the list of texture/sampler pairs (`#[texture_binding(index=...,data_type=...]`)
    fn get_texture_bindings(&self) -> &[TextureBindingDesc];
}

pub trait InterfaceBinder<T: ShaderInterface>
{
    /// Binds the contents of the shader interface to the OpenGL pipeline, without any validation.
    /// Validation is intended to be done when creating the graphics/compute pipeline.
    unsafe fn bind_unchecked(&self, interface: &T, uniform_binder: &UniformBinder);
}

/// Trait implemented by types that represent a shader interface.
///
/// A shader interface is a set of uniforms, vertex attributes, render targets
/// that describe the inputs and outputs of a graphics pipeline.
///
/// ```rust
/// #[derive(ShaderInterface)]
/// #[shader_interface(deny_incomplete)]
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
    fn get_description() -> &'static ShaderInterfaceDesc;
    /// Creates an _interface binder_ object that will handle binding interfaces of this specific
    /// type to the OpenGL pipeline.
    fn create_interface_binder(pipeline: &GraphicsPipeline) -> Result<Box<InterfaceBinder<Self>>, Error> where Self: Sized;
}

//
// pipeline.into_typed::<T: ShaderInterface>()
//      let binder = <T as ShaderInterface>::create_interface_binder(self)
//      TypedGraphicsPipeline { orig: pipeline.clone(), binder }
//
// draw()
//   typed_pipeline.bind()
//      pipeline.bind()
//      interface_binder.bind(binder)
//