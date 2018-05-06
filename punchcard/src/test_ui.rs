use ui::*;
use rand;
use rand::Rng;
use yoga;
use yoga::prelude::*;

pub fn make_ui(ui: &mut Ui, data: &mut i32) {
    ui.root(|ui| {
        ui.scroll("main", |ui| {
           // ui.vbox("main", |ui| {
                for i in 0..*data {
                    ui.vbox(format!("{}", i), |ui| {
                        style!(ui.item,
                        //Width(200 pt),
                        Padding(2 pt),
                        AlignSelf(yoga::Align::Center));
                        ui.text("aaa");
                        // ui.vbox("Reset", |_|{});
                    });
                }
                if *data < 500 {
                    *data += 1;
                }
            //});
        });
    });
}
