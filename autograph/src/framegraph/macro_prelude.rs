use std::mem;
use super::*;
use gfx;

#[doc(hidden)]
pub trait ToResourceInfo
{
    fn to_resource_info(&self) -> ResourceInfo;
    fn from_resource_info(ri: &ResourceInfo) -> Option<Self> where Self: Sized;
}

#[doc(hidden)]
pub struct Texture2DInit
{
    pub usage: ResourceUsage,
    pub format: gfx::TextureFormat,
    pub width: u32,
    pub height: u32,
    pub sample_count: u32,
    pub mip_map_count: gfx::MipMaps,
    pub options: gfx::TextureOptions
}

impl Default for Texture2DInit
{
    fn default() -> Texture2DInit {
        Texture2DInit {
            usage: ResourceUsage::Default,
            format: gfx::TextureFormat::R8G8B8A8_SRGB,
            width: 512,
            height: 512,
            sample_count: 1,
            mip_map_count: gfx::MipMaps::Count(1),
            options: gfx::TextureOptions::empty()
        }
    }
}

// TODO: other initializers
impl ToResourceInfo for Texture2DInit
{
    fn to_resource_info(&self) -> ResourceInfo {
        ResourceInfo::Texture {
            desc: gfx::TextureDesc {
                dimensions: gfx::TextureDimensions::Tex2D,
                format: self.format,
                width: self.width,
                height: self.height,
                depth: 1,
                sample_count: self.sample_count,
                mip_map_count: self.mip_map_count,
                options: self.options
            }
        }
    }

    fn from_resource_info(ri: &ResourceInfo) -> Option<Self>
    {
        if let &ResourceInfo::Texture { ref desc } = ri {
            Some(Texture2DInit {
                usage: ResourceUsage::Default,
                format: desc.format,
                width: desc.width,
                height: desc.height,
                sample_count: desc.sample_count,
                mip_map_count: desc.mip_map_count,
                options: desc.options,
            })
        } else {
            None
        }
    }
}


#[doc(hidden)]
pub struct BufferArrayInit<T: gfx::BufferData + ?Sized>
{
    pub inner: BufferArrayInitInner,
    _phantom: PhantomData<T>
}

#[doc(hidden)]
pub struct BufferArrayInitInner
{
    pub usage: ResourceUsage,
    pub len: usize
}

impl<T: gfx::BufferData + ?Sized> ToResourceInfo for BufferArrayInit<T>
{
    fn to_resource_info(&self) -> ResourceInfo {
        ResourceInfo::Buffer {
            byte_size: self.inner.len * mem::size_of::<T::Element>()
        }
    }

    fn from_resource_info(ri: &ResourceInfo) -> Option<Self>
    {
        unimplemented!()
    }
}

#[doc(hidden)]
pub fn alloc_as_texture(alloc: &Alloc) -> Option<&Rc<gfx::Texture>>
{
    unimplemented!()
}

#[doc(hidden)]
pub fn alloc_as_buffer_slice(alloc: &Alloc) -> Option<&gfx::BufferSliceAny>
{
    unimplemented!()
}

/*#[macro_export]
macro_rules! gfx_pass {
    // end rule
    // root
    (pass $PassName:ident ( $( $ParamName:ident : $ParamType:ty ),* ) {
        read {
            $( #[$ReadUsage:ident] $ReadName:ident : $ReadTy:ty = $ReadInit:expr),*
        }
        write {
            $( #[$WriteUsage:ident] $WriteName:ident : $WriteTy:ty = $WriteInit:expr),*
        }
        create {
            $( #[$CreateUsage:ident] $CreateName:ident : $CreateTy:ty = $CreateInit:expr),*
        }
        // Other items go into the Pass impl
    }) => {
        // Dummy struct
        //log_syntax!($($ReadInit)*);
        pub mod $PassName {
            use $crate::framegraph::*;
            use $crate::framegraph::macros::*;

            pub struct Inputs {
                $(pub $ReadName : NodeIndex,)*
                $(pub $WriteName : NodeIndex,)*
            }

            pub struct Outputs {
                $(pub $WriteName : NodeIndex,)*
                $(pub $CreateName : NodeIndex,)*
            }

            pub struct Resources {
                $(pub $ReadName : <$ReadTy as PassConstraintType>::Target,)*
                $(pub $WriteName : <$WriteTy as PassConstraintType>::Target,)*
                $(pub $CreateName : <$CreateTy as ResourceDesc>::Target,)*
            }

            pub struct Parameters {
                $(pub $ParamName : $ParamType,)*
            }

            pub struct Pass();

            impl $crate::framegraph::Pass for Pass {
                type Inputs = Inputs;
                type Outputs = Outputs;
                type Resources = Resources;
            }

            impl Pass {
                pub fn new(frame_graph: &mut $crate::framegraph::FrameGraph, $($ParamName : $ParamType,)* $($ReadName : NodeIndex,)* $($WriteName : NodeIndex,)* ) -> Outputs
                {
                    // move inputs into their own struct for convenience
                    // within this macro, we can explicitly name the type
                    let inputs = Inputs {
                        $($ReadName,)*
                        $($WriteName,)*
                    };

                    // fetch resourceinfos of inputs

                    // Read constraints
                    $(let mut $ReadName : $ReadTy = $ReadInit;)*
                    // Write constraints
                    $(let mut $WriteName : $WriteTy = $WriteInit;)*
                    // Create info
                    $(let mut $CreateName : $CreateTy = $CreateInit;)*

                    // 1. Create pass node
                    let node = frame_graph.create_pass_node(stringify!($PassName).to_owned());
                    // 2. link inputs
                    $( frame_graph.link_input(node, inputs.$ReadName,  $crate::framegraph::ResourceUsage::$ReadUsage); )*
                    $( frame_graph.link_input(node, inputs.$WriteName, $crate::framegraph::ResourceUsage::$WriteUsage); )*
                    // 3. create new resource nodes
                    let outputs = Outputs {
                        $( $CreateName: frame_graph.create_resource_node(stringify!($CreateName).to_owned(), $CreateName.to_resource_info() ), )*
                        $( $WriteName: frame_graph.clone_resource_node(inputs.$WriteName), )*
                    };

                    // 4. link outputs
                    $(frame_graph.link_output(node, outputs.$CreateName, $crate::framegraph::ResourceUsage::$CreateUsage);)*
                    $(frame_graph.link_output(node, outputs.$WriteName, $crate::framegraph::ResourceUsage::$WriteUsage);)*

                    // 5. return outputs
                    outputs
                }
            }
        }
    };
}
*/