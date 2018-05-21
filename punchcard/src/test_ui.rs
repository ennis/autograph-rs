use rand;
use rand::Rng;
use ui::*;
use yoga;
use yoga::prelude::*;

pub fn make_ui(ui: &mut Ui, data: &mut i32) {
    ui.root(|ui| {
        ui.scroll("main", |ui| {
            // ui.vbox("main", |ui| {
            for i in 0..100 {
                ui.hbox(format!("{}", i), |ui| {
                    ui.slider("slider0", data, 0, 50);
                    ui.slider("slider", data, 0, 50);
                });
            }
            //if *data < 500 {
            //    *data += 1;
           // }
            //});
        });
    });
}
