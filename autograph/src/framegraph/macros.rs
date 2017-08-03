use super::*;

#[derive(Clone,Debug)]
pub struct TextureConstraints
{
    usage: ResourceUsage,
    dimensions: Option<gfx::TextureDimensions>,
    allowed_formats: Option<Vec<gfx::TextureFormat>>,
    width: Option<u32>,
    height: Option<u32>,
    depth: Option<u32>
}

impl Default for TextureConstraints
{
    fn default() -> TextureConstraints {
        TextureConstraints {
            dimensions: None,
            usage: ResourceUsage::Default,
            allowed_formats: None,
            width: None,
            depth: None,
            height: None
        }
    }
}

#[derive(Debug)]
pub struct BufferConstraints<T: gfx::BufferData+?Sized>
{
    pub len: Option<usize>,
    _phantom: PhantomData<T>
}

impl<T: gfx::BufferData+?Sized> Default for BufferConstraints<T>
{
    fn default() -> BufferConstraints<T> {
        BufferConstraints {
            len: None,
            _phantom: PhantomData
        }
    }
}

pub trait PassConstraintType
{
    type Target;
}

impl PassConstraintType for TextureConstraints
{
    type Target = Rc<gfx::Texture>;
}

impl<T: gfx::BufferData+?Sized> PassConstraintType for BufferConstraints<T>
{
    type Target = gfx::BufferSliceAny;
}

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

            pub struct Inputs {
                $(pub $ReadName : $crate::petgraph::graph::NodeIndex,)*
                $(pub $WriteName : $crate::petgraph::graph::NodeIndex,)*
            }

            pub struct Outputs {
                $(pub $WriteName : $crate::petgraph::graph::NodeIndex,)*
                $(pub $CreateName : $crate::petgraph::graph::NodeIndex,)*
            }

            pub struct Resources {
                $(pub $ReadName : <$ReadTy as $crate::framegraph::PassConstraintType>::Target,)*
                $(pub $WriteName : <$WriteTy as $crate::framegraph::PassConstraintType>::Target,)*
                $(pub $CreateName : <$CreateTy as $crate::framegraph::ResourceDesc>::Target,)*
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
                pub fn new(frame_graph: &mut $crate::framegraph::FrameGraph, $($ParamName : $ParamType,)* $($ReadName : $crate::petgraph::graph::NodeIndex,)* $($WriteName : $crate::petgraph::graph::NodeIndex,)* ) -> Outputs
                {
                    // move inputs into their own struct for convenience
                    // within this macro, we can explicitly name the type
                    let inputs = Inputs {
                        $($ReadName,)*
                        $($WriteName,)*
                    };

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
                        $( $CreateName: frame_graph.create_resource_node(stringify!($CreateName).to_owned(), $CreateName.to_payload() ), )*
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
