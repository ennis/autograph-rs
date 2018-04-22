#![feature(box_syntax, box_patterns)]

extern crate num_traits;
extern crate spirv_headers as spirv;

mod parse;
mod types;

use std::collections::HashMap;

//
// Store instructions, etc.
// 'describe' functions returns information in a more compact form for processing (flattens types, resolves names, etc.)
#[derive(Debug)]
pub struct Reflect {
    parsed: parse::Spirv,
    pub struct_types: HashMap<u32, types::Struct>,
    pub entry_points: HashMap<u32, types::EntryPoint>,
    pub variables: HashMap<u32, types::Variable>,
    pub primitive_types: HashMap<u32, types::Type>,
}

impl Reflect {
    pub fn from_bytes(blob: &[u8]) -> Result<Reflect, parse::ParseError> {
        let mut struct_types = HashMap::new();
        let mut entry_points = HashMap::new();
        let mut variables = HashMap::new();
        let mut primitive_types = HashMap::new();

        let parsed = parse::parse_spirv(blob)?;
        types::parse_entry_points(&parsed, &mut entry_points);
        types::parse_variables(&parsed, &mut variables);
        //types::parse_types(&parsed, &mut )
        Ok(Reflect {
            struct_types,
            entry_points,
            variables,
            primitive_types,
            parsed,
        })
    }

    pub fn describe_type(&self, ty: u32) -> types::Type {
        types::type_from_id(&self.parsed, ty)
    }

    // does the variable has a builtin declaration, or if the type is struct,
    // does it have members decorated with builtin
    pub fn is_builtin_variable(&self, v: &types::Variable) -> bool {
        v.deco.builtin.is_some() || {
            let tydesc = self.describe_type(v.ty);
            if let &types::Type::Struct(struct_id) = &tydesc {
                types::parse_struct(&self.parsed, struct_id).has_builtin_members()
            } else if let &types::Type::Pointer(box types::Type::Struct(struct_id)) = &tydesc {
                types::parse_struct(&self.parsed, struct_id).has_builtin_members()
            } else {
                false
            }
        }
    }
}
