use ui::*;
use rand;
use rand::Rng;

pub fn make_ui(ui: &mut Ui, data: &mut i32) {
    ui.root(|ui| {
        ui.vbox("main", |ui| {
            let num_children = rand::thread_rng().gen_range(0, 10);
            for i in 0..num_children {
                ui.vbox(format!("Reset {}",i), |ui| {});
            }
        });
    });
}
