//! Scroll panels.
use super::super::*;

impl<'a> UiContainer<'a> {
    ///
    /// Scrollable panel.
    ///
    pub fn scroll<S, F>(&mut self, id: S, f: F)
    where
        S: Into<String>,
        F: FnOnce(&mut UiContainer),
    {
        //=====================================
        // behavior
        struct ScrollState {
            pub pos: f32,
        };

        impl Behavior for ScrollState {
            fn event(
                &mut self,
                _item: &mut Item,
                event: &WindowEvent,
                _input_state: &mut InputState,
            ) -> bool {
                match event {
                    &WindowEvent::KeyboardInput { input, .. } => match input.virtual_keycode {
                        Some(VirtualKeyCode::Up) => {
                            debug!("Scroll up");
                            self.pos -= 10.0;
                        }
                        Some(VirtualKeyCode::Down) => {
                            debug!("Scroll down");
                            self.pos += 10.0;
                        }
                        _ => {}
                    },
                    _ => {}
                }
                // always capture?
                false
            }
        }

        self.item(
            id,
            "scroll",
            ScrollState { pos: 0.0 },
            |ui, _, scroll| {
                //let top = -scroll.pos;
                f(ui);
            },
        );
    }
}
