use rand;
use rand::Rng;
use ui::*;
use yoga;
use yoga::prelude::*;

pub fn make_ui(ui: &mut Ui, data: &mut i32) {
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
            //if *data < 500 {
            //    *data += 1;
           // }
            //});
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
