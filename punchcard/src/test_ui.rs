use rand;
use rand::Rng;
use ui::*;
use yoga;
use yoga::prelude::*;

pub fn make_ui(ui: &mut Ui, data: &mut i32) {
    ui.root(|ui| {
        ui.scroll("main", |ui| {
            // ui.vbox("main", |ui| {
            for i in 0..20 {
                //ui.item(format!("{}", i), (), |ui| {
                let mut val = 0.0;
                ui.slider_f32(format!("{}", i), &mut val, 0.0, 1.0);
                //});
            }
            if *data < 500 {
                *data += 1;
            }
            //});
        });
    });
}
