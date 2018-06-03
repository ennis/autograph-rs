//! Buttons
use super::super::*;

impl<'a> UiContainer<'a> {
    ///
    /// Button.
    ///
    pub fn button<S>(&mut self, label: S)
    where
        S: Into<String>,
    {
        let label = label.into();
        struct Button;
        impl Behavior for Button {}
        self.item(label.clone(), "button", Button, |ui, _, _| {
            ui.text_class(label, "button-label");
        });
    }
}
