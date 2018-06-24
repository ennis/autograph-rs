extern crate punchcard;

#[macro_use]
extern crate log;
extern crate pretty_env_logger;
#[macro_use]
extern crate failure;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate bitflags;
extern crate num;
extern crate rand;
extern crate time;

use punchcard::*;

use rand::Rng;

mod common;



fn main() {
    common::main_wrapper("Simple example", 1280, 720, |dom| {
        static mut DATA: u32 = 0;
        let data = unsafe { &mut DATA };

        /*vbox(dom, |dom| {
            collapsing_panel(dom, "panel", |dom| {
                hbox(dom, |dom| {
                    dummy(dom, (55, 20));
                    dummy(dom, (55, 20));
                    dummy(dom, (55, 20));
                    dummy(dom, (55, 20));
                    dummy(dom, (55, 20));
                });
            });
            dummy(dom, (55, 20));
            dummy(dom, (55, 20));
            dummy(dom, (55, 20));
            dummy(dom, (55, 20));
            dummy(dom, (55, 20));
        });*/

        // this actually works
        dom! (dom;
            @vbox {
                @collapsing_panel("panel") {
                    @hbox {
                        @dummy((55,20));
                        @dummy((55,20));
                        @dummy((55,20));
                        @dummy((55,20));
                        @dummy((55,20));
                    }
                }
                @condition(true) {
                    @dummy((55,20));
                    @dummy((55,20));
                    @dummy((55,20));
                    @dummy((55,20));
                    @dummy((55,20));
                }
            }
        );

        //debug!("vdom={:?}", dom.children());
    });
}


// with macros:
/*@root {
    @scroll("main") {
        @collapsing_panel("main") {
            @hbox {
                // TODO allow state change here through ui.item
                @text("hello");
                @button("click");
                @slider("");
            }
        }
    }
}*/


    // Issue: how/when to update data?
    // -> lifetime-bound callback
    // Issue: how to compare two versions of a node (reconciliation?)
    // -> ID (local hash)
    // -> Mount/Unmount handler
    // Issue: how to generate ID now that there is no ID stack?
    // -> not an issue
    // Issue: components accessing the internal state of other widgets in the immediate path.
    // External vs internal state.
    // The presence of a child item may depend on the internal state of a parent widget.
    // i.e. the whole thing is actually stateless internally
    // at least, cannot access internal state inline.
    // but, type safety is only available inline.
    // Can return a renderable instead (a component with a render() method)


    /*Ui::new(|ui| {
       ui.scroll(|ui| {
           for i in 0..10 {
               ui.floating_panel(format!("Floating {}", i), |ui| {
                   ui.text("panel contents");
               });
           }
       });
    });*/


    /*ui!{
        @root {
            @scroll("main") {
                @collapsing_panel("main") {
                    @item.align = ...;
                    @hbox {
                        // TODO allow state change here through ui.item
                        @text("hello");
                        @button("click");
                        @slider("");
                    }
                }
            }
        }
    }*/


/*
Two types of GUI:
- allow external changes
    - changes to the data model may happen outside the environment controlled by the UI
    - related: don't own the event loop
- fully manage data
    - changes to the data model only happen within the UI
    - UI manages data model changes
        - schedule tasks, etc.
    - UI owns the event loop.
    - UI owns the application state.
Case study:
- slider and text box, linked.

Two types of immediate specification:
- function calls (ui.xxx)
    - directly modify an existing tree == update-in-place
    - issue with object identity?
- object creation (Dom::new())
    - create sub-tree first, then transfer ownership
    - diff with previous version
    - composable, but hard to cache?
    - input?
*/