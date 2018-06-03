extern crate punchcard;

#[macro_use]
extern crate log;
extern crate pretty_env_logger;
extern crate glutin;
#[macro_use]
extern crate failure;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate bitflags;
extern crate gl;
extern crate indexmap;
extern crate nanovg as nvg;
extern crate num;
extern crate petgraph;
extern crate rand;
extern crate time;
extern crate yoga;
extern crate cssparser;
extern crate warmy;
extern crate winapi;

use punchcard::*;
use rand::Rng;

mod common;

fn main() {
    common::gui_test(|ui| {
        static mut DATA: u32 = 0;
        let data = unsafe { &mut DATA };

        ui.root(|ui| {
            ui.scroll("main", |ui| {
                // ui.vbox("main", |ui| {
                for i in 0..10 {
                    ui.floating_panel(format!("Floating {}", i), |ui| {
                        ui.text("panel contents");
                    });
                    ui.hbox(format!("{}", i), |ui| {
                        for i in 0..2 {
                            ui.collapsing_panel(format!("Panel {}", i), |ui| {
                                for i in 0..10 {
                                    ui.hbox(format!("{}", i), |ui| {
                                        ui.text("hello");
                                        ui.button("click");
                                        ui.slider("slider0", data, 0, 50);
                                        ui.slider("slider", data, 0, 50);
                                    });
                                }
                            });
                        }
                    });
                }
            });
        });
    });
}

// with macros:
/*@root {
    @scroll("main") {
        @collapsing_panel("main") {
            @hbox {
                @text("hello");
                @button("click");
                @slider("");
            }
        }
    }
}*/
