extern crate spirv_headers as spirv;
extern crate num_traits;

mod parse;
mod types;

use std::collections::HashMap;


//
// Store instructions, etc.
// 'describe' functions returns information in a more compact form for processing (flattens types, resolves names, etc.)
#[derive(Debug)]
pub struct Reflect
{
    struct_types: HashMap<u32, types::Struct>,
    entry_points: HashMap<u32, types::EntryPoint>,
    variables: HashMap<u32, types::Variable>,
    types: HashMap<u32, types::Type>,
}

impl Reflect
{
    pub fn reflect(blob: &[u8]) -> Result<Reflect, parse::ParseError>
    {
        let mut r = Reflect {
            struct_types: HashMap::new(),
            entry_points: HashMap::new(),
            variables: HashMap::new(),
            types: HashMap::new()
        };

        let parsed = parse::parse_spirv(blob)?;
        types::parse_entry_points(&parsed, &mut r.entry_points);
        types::parse_variables(&parsed, &mut r.variables);
        Ok(r)
    }
}
