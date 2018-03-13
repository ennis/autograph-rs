extern crate lalrpop;

use std::env;
use std::fs::File;
use std::path::Path;

fn main() {
    lalrpop::process_root().unwrap();
}
