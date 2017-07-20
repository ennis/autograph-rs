extern crate spirv_headers as spirv;
extern crate num_traits;

mod parse;
mod types;

struct Variable
{
    id: u32,
    name: Option<String>,
    storage_class: spirv::StorageClass,
    ty: types::Type
}

struct Interface
{
    loc: u32,
    var_id: u32,
}

struct EntryPoint
{
    execution_model: spirv::ExecutionModel,
    name: String,
    interface: Vec<Interface>,          // interface (input/output, location, variable)
}

impl EntryPoint
{
    pub fn describe_interface<'a>(&'a self) -> Vec<(spirv::StorageClass, u32, &'a types::Type, &'a str)>
    {
        unimplemented!()
    }
}

//
// Store instructions, etc.
// 'describe' functions returns information in a more compact form for processing (flattens types, resolves names, etc.)
struct Reflect
{
    structs: Vec<types::Struct>,
    entry_points: Vec<EntryPoint>,
    global_variables: Vec<Variable>
    // structs
    // interfaces
    // descriptor sets & descriptors
    //
}

pub fn parse_interface(doc: &parse::Spirv, interface: &[u32]) -> Vec<Interface>
{
    interface.iter().map(|id| {
        if let Some(loc_deco) = types::find_decoration(doc, *id, spirv::Decoration::Location) {
            Interface {
                loc: loc_deco[0],
                var_id: *id
            }
        } else {
            panic!("No location decoration for interface variable")
        }
    }).collect()
}

pub fn parse_entry_points(doc: &parse::Spirv)
{
    let mut entry_points = Vec::new();

    for instruction in doc.instructions.iter() {
        match instruction {
            &parse::Instruction::EntryPoint {
                execution,
                id,
                ref name,
                ref interface } => {
                entry_points.push(EntryPoint {
                    execution_model: execution,
                    name: name.clone(),
                })
            },
            _ => ()
        }
    }

    panic!("No entry point found in binary")
}