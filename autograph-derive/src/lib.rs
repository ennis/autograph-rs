#![recursion_limit="128"]

extern crate proc_macro;
extern crate proc_macro2;
#[macro_use] extern crate darling;  // this is a _good crate_
extern crate autograph;
#[macro_use] extern crate syn;
#[macro_use] extern crate quote;

use darling::FromField;
use proc_macro::TokenStream;
use autograph::gfx::shader_interface::*;

#[proc_macro_derive(VertexType, attributes(rename))]
pub fn vertex_type_derive(input: TokenStream) -> TokenStream {
    let ast: syn::DeriveInput = syn::parse(input).expect("Couldn't parse item");

    let result = match ast.data {
        syn::Data::Struct(ref s) => process_vertex_struct(&ast, &s.fields),
        _ => panic!("VertexType trait can only be derived on structs"),
    };

    result.into()

}

fn process_vertex_struct(ast: &syn::DeriveInput,
                        fields: &syn::Fields) -> quote::Tokens
{
    let struct_name = &ast.ident;

    let fields = match *fields {
        syn::Fields::Named(ref fields_named) => { &fields_named.named },
        syn::Fields::Unnamed(ref fields_unnamed) => { &fields_unnamed.unnamed },
        syn::Fields::Unit => { panic!("VertexType trait cannot be derived on unit structs") }
    };

    let mut attrib_descs = Vec::new();
    let mut attrib_sizes = Vec::new();

    for (i,f) in fields.iter().enumerate() {
        println!("{} => {:?}", i, f.ident);
        let field_ty = &f.ty;
        let field_name = f.ident.clone().unwrap_or(syn::Ident::from(format!("unnamed_{}",i)));
        let field_offset = if let Some(ref name) = f.ident {
            quote!(offset_of!(#struct_name,#name))
        } else {
            quote!(offset_of!(#struct_name,#i))
        };
        attrib_descs.push(quote!(::autograph::gfx::VertexAttributeDesc {
            name: Some(stringify!(#field_name).into()),
            loc: #i as u8,
            ty: <#field_ty as ::autograph::gfx::VertexElementType>::get_equivalent_type(),
            format: <#field_ty as ::autograph::gfx::VertexElementType>::get_format(),
            offset: #field_offset as u8
        }));
        attrib_sizes.push(quote!(::std::mem::size_of::<#field_ty>()));
    }

    let num_attribs = attrib_descs.len();

    let private_module_name = syn::Ident::new(&format!("__vertex_type_{}", struct_name), proc_macro2::Span::call_site());

    quote! {
        #[allow(non_snake_case)]
        mod #private_module_name {
            use super::#struct_name;

            lazy_static!{
                pub(super) static ref ATTRIBUTES: [::autograph::gfx::VertexAttributeDesc;#num_attribs] = {[#(#attrib_descs),*]};
                pub(super) static ref STRIDE: usize = #(#attrib_sizes)+*;
                pub(super) static ref LAYOUT: ::autograph::gfx::VertexLayout =
                    ::autograph::gfx::VertexLayout {
                        attributes: &*ATTRIBUTES,
                        stride: *STRIDE
                    };
            }
        }

        impl ::autograph::gfx::VertexType for #struct_name {
            fn get_layout() -> &'static ::autograph::gfx::VertexLayout {
                &*#private_module_name::LAYOUT
            }
        }
    }
}

#[proc_macro_derive(ShaderInterface, attributes(named_uniform, texture_binding, vertex_buffer, index_buffer))]
pub fn shader_interface_derive(input: TokenStream) -> TokenStream {
    let ast: syn::DeriveInput = syn::parse(input).expect("Couldn't parse item");

    let result = match ast.data {
        syn::Data::Struct(ref s) => process_struct(&ast, &s.fields),
        _ => panic!("ShaderInterface trait can only be derived on structs"),
    };

    result.into()
}

#[derive(FromField)]
#[darling(attributes(named_uniform))]
struct NamedUniform {
    ident: Option<syn::Ident>,
    ty: syn::Type,
    vis: syn::Visibility,
    #[darling(default)] rename: Option<String>,
    #[darling(default)] index: Option<i32>
}

#[derive(FromField)]
#[darling(attributes(texture_binding))]
struct TextureBinding {
    ident: Option<syn::Ident>,
    ty: syn::Type,
    vis: syn::Visibility,
    #[darling(default)] rename: Option<String>,
    #[darling(default)] index: Option<i32>
}

#[derive(FromField)]
#[darling(attributes(vertex_buffer))]
struct VertexBuffer {
    ident: Option<syn::Ident>,
    ty: syn::Type,
    vis: syn::Visibility,
    #[darling(default)] rename: Option<String>,
    #[darling(default)] index: Option<i32>
}

#[derive(FromField)]
#[darling(attributes(index_buffer))]
struct IndexBuffer {
    ident: Option<syn::Ident>,
    ty: syn::Type,
    vis: syn::Visibility,
    #[darling(default)] rename: Option<String>,
}

fn error_multiple_interface_attrs()
{
    panic!("Multiple interface attributes on field.");
}

fn make_option_tokens<T: quote::ToTokens>(v: &Option<T>) -> quote::Tokens {
    if let Some(v) = v.as_ref() {
        quote!(Some(#v))
    } else {
        quote!(None)
    }
}

fn process_struct(ast: &syn::DeriveInput,
                  fields: &syn::Fields) -> quote::Tokens
{
    let struct_name = &ast.ident;

    let mut named_uniforms = Vec::new();
    let mut texture_bindings = Vec::new();
    let mut vertex_buffers = Vec::new();
    let mut index_buffer = None;

    match *fields {
        syn::Fields::Named(ref fields) => {
            for f in fields.named.iter() {
                let field_name = f.ident.unwrap();
                for a in f.attrs.iter() {
                    let mut seen_interface_attr = false;
                    let meta = a.interpret_meta();
                    let meta = if let Some(meta) = meta { meta } else { continue };

                    match meta.name().as_ref() {
                        "named_uniform" => {
                            if seen_interface_attr { error_multiple_interface_attrs(); }
                            let named_uniform = <NamedUniform as FromField>::from_field(f).unwrap();
                            named_uniforms.push(named_uniform);
                            seen_interface_attr = true;
                        },
                        "texture_binding" => {
                            if seen_interface_attr { error_multiple_interface_attrs(); }
                            let texture_binding = <TextureBinding as FromField>::from_field(f).unwrap();
                            texture_bindings.push(texture_binding);
                            seen_interface_attr = true;
                        },
                        "vertex_buffer" => {
                            if seen_interface_attr { error_multiple_interface_attrs(); }
                            let vb = <VertexBuffer as FromField>::from_field(f).unwrap();
                            vertex_buffers.push(vb);
                            seen_interface_attr = true;
                        },
                        "index_buffer" => {
                            if seen_interface_attr { error_multiple_interface_attrs(); }
                            if index_buffer.is_some() { panic!("Only one index buffer can be specified.") }
                            let ib = <IndexBuffer as FromField>::from_field(f).unwrap();
                            index_buffer = Some(ib);
                            seen_interface_attr = true;
                        },
                        _ => {}
                    }
                }
            }
        },
        _ => panic!("ShaderInterface trait cannot be derived on unit structs or tuple structs."),
    }

    let named_uniform_items =
        named_uniforms.iter().map(|named_uniform| {
            let name = named_uniform.rename.as_ref().map_or(named_uniform.ident.unwrap(), |s| syn::Ident::from(s.as_str()));
            //let index_tokens = make_option_tokens(texbind.index);
            quote! {
                NamedUniformDesc {
                    name: stringify!(#name).into(),
                    ty: Type::Unknown
                }
            }
        }).collect::<Vec<_>>();
    let num_named_uniform_items = named_uniform_items.len();

    let texture_binding_items =
        texture_bindings.iter().map(|texbind| {
            let name = texbind.rename.as_ref().map_or(texbind.ident.unwrap(), |s| syn::Ident::from(s.as_str()));
            let index_tokens = make_option_tokens(&texbind.index);

            quote! {
                ::autograph::gfx::shader_interface::TextureBindingDesc {
                    name: Some(stringify!(#name).into()),
                    index: #index_tokens,
                    data_type: TextureDataType::Unknown
                }
            }
        }).collect::<Vec<_>>();
    let num_texture_binding_items = texture_binding_items.len();

    let vertex_buffer_items =
        vertex_buffers.iter().map(|vb| {
            let name = vb.rename.as_ref().map_or(vb.ident.unwrap(), |s| syn::Ident::from(s.as_str()));
            let index_tokens = make_option_tokens(&vb.index);

            quote! {
                ::autograph::gfx::shader_interface::VertexBufferDesc {
                    name: Some(stringify!(#name).into()),
                    index: #index_tokens,
                    data_type: TextureDataType::Unknown
                }
            }
        }).collect::<Vec<_>>();
    let num_vertex_buffer_items = vertex_buffer_items.len();

    let private_module_name = syn::Ident::new(&format!("__shader_interface_{}", struct_name), proc_macro2::Span::call_site());

    // generate impls
    quote!{
        #[allow(non_snake_case)]
        mod #private_module_name {
            use super::#struct_name;
            use ::autograph::gfx::shader_interface::*;

            pub(super) struct Desc;
            pub(super) struct Binder;

            lazy_static!{
                static ref NAMED_UNIFORMS: [NamedUniformDesc;#num_named_uniform_items] = [#(#named_uniform_items),*];
                static ref TEXTURE_BINDINGS: [TextureBindingDesc;#num_texture_binding_items] = [#(#texture_binding_items),*];
            }

            impl ShaderInterfaceDesc for Desc {
                fn get_named_uniforms(&self) -> &'static [NamedUniformDesc] {
                    &*NAMED_UNIFORMS
                }

                fn get_render_targets(&self) -> &'static [RenderTargetDesc] {
                     unimplemented!()
                }
                fn get_vertex_buffers(&self) -> &'static [VertexBufferDesc] {
                    unimplemented!()
                }
                fn get_index_buffer(&self) -> Option<IndexBufferDesc> {
                    unimplemented!()
                }

                fn get_texture_bindings(&self) -> &'static [TextureBindingDesc] {
                    &*TEXTURE_BINDINGS
                }
            }

            impl InterfaceBinder<#struct_name> for Binder {
                unsafe fn bind_unchecked(&self, interface: &#struct_name, uniform_binder: &::autograph::gfx::UniformBinder) {
                    unimplemented!()
                }
            }

        }

        impl ::autograph::gfx::ShaderInterface for #struct_name {
            fn get_description() -> &'static ::autograph::gfx::ShaderInterfaceDesc {
                static INSTANCE: &'static ::autograph::gfx::ShaderInterfaceDesc = &#private_module_name::Desc;
                INSTANCE
            }

            fn create_interface_binder(pipeline: &::autograph::gfx::GraphicsPipeline) -> Result<Box<::autograph::gfx::InterfaceBinder<Self>>, ::failure::Error> where Self: Sized {
                // TODO: verify interface
                Ok(Box::new(#private_module_name::Binder))
            }
        }
    }
}


/*fn parse_named_uniform(field: &syn::Field, meta: &syn::Meta) {
    let field_name = field.ident.unwrap();
    match *meta {
        syn::Meta::Word(ref ident) => {
            println!("{:?} => named uniform", field_name);
        },
        syn::Meta::List(ref metalist) => {
            for nested in metalist.nested.iter() {
                match *nested {
                    syn::NestedMeta::Meta(syn::Meta::NameValue(ref nv)) => {
                        match nv.ident.as_ref() {
                            "name" => {
                                match nv.lit {
                                    syn::Lit::Str(ref renamed) => {
                                        println!("rename {:?} => {:?}", field_name, renamed.value());
                                    },
                                    _ => panic!("String literal expected.")
                                }
                            },
                            _ => panic!("Unrecognized meta item in named_uniform attribute.")
                        }
                    },
                    _ => panic!("Unrecognized literal in named_uniform attribute.")
                }
            }
        },
        syn::Meta::NameValue(_) => panic!("Invalid format for named_uniform attribute.")
    }
}*/

/*fn parse_texture_binding(field: &syn::Field, meta: &syn::Meta)  {
    let field_name = field.ident.unwrap();
    match *meta {
        syn::Meta::Word(ref ident) => {
            println!("{:?} => texture_binding", field_name);
        },
        syn::Meta::List(ref metalist) => {
            for nested in metalist.nested.iter() {
                match *nested {
                    syn::NestedMeta::Meta(syn::Meta::NameValue(syn::MetaNameValue { ref ident, ref lit, .. })) => {
                        match ident.as_ref() {
                            // index="0"
                            "index" => {
                                match *lit {
                                    syn::Lit::Str(ref index_str) => {
                                        if let Ok(index) = index_str.value().parse::<u32>() {
                                            println!("binding_index={}",index);
                                        } else {
                                            panic!("Failed to parse texture_binding index")
                                        }
                                    },
                                    _ => panic!("String literal expected.")
                                }
                            },
                            "name" => {
                                match *lit {
                                    syn::Lit::Str(ref renamed) => {
                                        println!("rename {:?} => {:?}", field_name, renamed.value());
                                    },
                                    _ => panic!("String literal expected.")
                                }
                            },
                            _ => panic!("Unrecognized meta item in texture_binding attribute.")
                        }
                    },
                    _ => panic!("Unrecognized literal in texture_binding attribute.")
                }
            }
        },
        syn::Meta::NameValue(_) => panic!("Invalid format for texture_binding attribute.")
    }
}

fn parse_vertex_buffer(field: &syn::Field, meta: &syn::Meta) {
    let field_name = field.ident.unwrap();
    match *meta {
        syn::Meta::Word(ref ident) => {
            println!("{:?} => vertex_buffer", field_name);
        },
        syn::Meta::List(ref metalist) => {
            for nested in metalist.nested.iter() {
                match *nested {
                    syn::NestedMeta::Meta(syn::Meta::NameValue(syn::MetaNameValue { ref ident, ref lit, .. })) => {
                        match ident.as_ref() {
                            // index="0"
                            "index" => {
                                match *lit {
                                    syn::Lit::Str(ref index_str) => {
                                        if let Ok(index) = index_str.value().parse::<u32>() {
                                            println!("binding_index={}",index);
                                        } else {
                                            panic!("Failed to parse vertex_buffer index")
                                        }
                                    },
                                    _ => panic!("String literal expected.")
                                }
                            },
                            _ => panic!("Unrecognized meta item in vertex_buffer attribute.")
                        }
                    },
                    _ => panic!("Unrecognized literal in vertex_buffer attribute.")
                }
            }
        },
        syn::Meta::NameValue(_) => panic!("Invalid format for vertex_buffer attribute.")
    }
}

fn parse_index_buffer(field: &syn::Field, meta: &syn::Meta) {
    let field_name = field.ident.unwrap();
    println!("index_buffer {}", field_name);
}

fn parse_attribs(field: &syn::Field)
{


        /*match meta {
            Word(ref ident) => {
                println!("Word {:?}", ident);

            },
            List(ref metalist) => {

            },
            NameValue(ref namevalue) => {
                println!()
            }
        }*/

        //println!("meta = {:?}", meta);
    }
}

*/