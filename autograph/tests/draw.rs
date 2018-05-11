#[macro_use]
extern crate autograph;
#[macro_use]
extern crate autograph_derive;

mod common;

use autograph::gfx;
use autograph::gl;
use common::*;

////////////////////////////////////////////////////////////////////////////////////////////////////
#[test]
fn test_simple_window() {
    let cfg = TestWindowConfig {
        name: "simple_draw",
        width: 256,
        height: 256,
    };

    run_test(&cfg, |frame_info| false);
}
