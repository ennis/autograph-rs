extern crate embed_resource;

use std::env;
use std::fs::File;
use std::path::Path;

fn main() {
    embed_resource::compile("hidpi.rc");
}
