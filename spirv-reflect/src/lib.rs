extern crate spirv_headers as spirv;
extern crate num_traits;

mod parse;
mod types;

use std::collections::HashMap;



//
// Store instructions, etc.
// 'describe' functions returns information in a more compact form for processing (flattens types, resolves names, etc.)
struct Reflect
{
    struct_types: HashMap<u32, types::Struct>,
    entry_points: HashMap<u32, types::EntryPoint>,
    global_variables: HashMap<u32, types::Variable>
}
