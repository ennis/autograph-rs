//! Buttons
use super::super::*;
use super::super::behavior::ButtonBehavior;

impl<'a> UiContainer<'a> {
    ///
    /// Button.
    ///
    pub fn button<S>(&mut self, label: S)
    where
        S: Into<String>,
    {
        let label = label.into();
        self.item(label.clone(), "button", ButtonBehavior::new(), |ui, item, state| {
            ui.text_class(label, "button-label");
            if state.clicked() {
                debug!("button clicked {:016X}", item.id);
            }
        });
    }
}
