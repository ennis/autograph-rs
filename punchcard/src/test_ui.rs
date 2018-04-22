use ui::*;

pub fn make_ui(ui: &mut Ui, data: &mut i32) {
    ui.vbox("main", |ui| {
        ui.text(format!("data={}", data));
        if ui.button("Reset").clicked {
            ui.text("Done");
        }
    });
}
