use ui::*;

pub fn make_ui(ui: &mut Ui, data: &mut i32) {

    ui.root(|ui| {
        ui.vbox("main", |ui| {
            ui.vbox("Reset", |ui| {});
        });
    });
}
