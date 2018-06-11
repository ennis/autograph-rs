//! Layout panels (vbox and hbox), collapsible panels and floating panels.
use super::super::*;
use container::WindowEventExt;

impl<'a> UiContainer<'a> {
    ///
    /// Vertical layout box.
    ///
    pub fn vbox<S, F>(&mut self, id: S, f: F) -> ItemResult
    where
        S: Into<String>,
        F: FnOnce(&mut UiContainer),
    {
        struct VBox;
        impl Behavior for VBox {}

        self.item(id, "vbox", VBox, |ui, _, _| {
            f(ui);
        });

        ItemResult {
            clicked: false,
            hover: false,
        }
    }

    ///
    /// Horizontal layout box.
    ///
    pub fn hbox<S, F>(&mut self, id: S, f: F) -> ItemResult
    where
        S: Into<String>,
        F: FnOnce(&mut UiContainer),
    {
        struct HBox;
        impl Behavior for HBox {}

        self.item(id, "hbox", HBox, |ui, _, _| {
            f(ui);
        });

        ItemResult {
            clicked: false,
            hover: false,
        }
    }

    ///
    /// Collapsing header.
    ///
    pub fn collapsing_panel<S, F>(&mut self, id: S, f: F)
    where
        S: Into<String>,
        F: FnOnce(&mut UiContainer),
    {
        let label = id.into();

        self.item(
            label.clone(),
            "collapsing-panel",
            CheckboxBehavior::new(),
            |ui, _, behavior| {
                ui.item("header", "collapsing-panel-header", (), |ui, _, _| {
                    ui.text(label.clone());
                });

                if !behavior.checked {
                    ui.item(
                        "contents",
                        "collapsing-panel-contents",
                        (),
                        |ui, _, _| {
                            f(ui);
                        },
                    );
                }
            },
        );
    }

    ///
    /// Draggable panel.
    /// Very similar to collapsing panels.
    ///
    pub fn floating_panel<S, F>(&mut self, id: S, f: F)
    where
        S: Into<String>,
        F: FnOnce(&mut UiContainer),
    {
        let label = id.into();

        //============================================
        // Panel
        struct FloatingPanel {
            /// Is the panel collapsed (only title bar shown).
            collapsed: bool,
            /// Position drag.
            drag: DragBehavior,
        }

        impl FloatingPanel {
            fn new() -> Self {
                FloatingPanel {
                    collapsed: false,
                    drag: DragBehavior::new(),
                }
            }
        }

        impl Behavior for FloatingPanel {
            fn event(
                &mut self,
                item: &mut Item,
                event: &WindowEvent,
                input_state: &mut InputState,
            ) -> bool {
                if event.mouse_down() {
                    item.bring_to_front();
                }
                self.drag.event(item, event, input_state)
                // TODO collapsing behavior
            }
        }

        self.popup(
            label.clone(),
            "floating-panel",
            FloatingPanel::new(),
            |ui, panel_item, panel_behavior| {
                let mut position = (panel_item.layout.left, panel_item.layout.top);
                if panel_behavior.drag.handle_drag(&mut position) {
                    panel_item.set_position(Some(position.0.point()), Some(position.1.point()));
                }

                ui.item("header", "floating-panel-header", (), |ui, _, _| {
                    ui.text(label.clone());
                });

                ui.item("contents", "floating-panel-contents", (), |ui, _, _| {
                    if !panel_behavior.collapsed {
                        f(ui);
                    }
                });
                ui.item(
                    "resize-handle",
                    "floating-panel-resize-handle",
                    DragBehavior::new(),
                    |_, _, handle_behavior| {
                        let mut size = (panel_item.layout.width(), panel_item.layout.height());
                        if handle_behavior.handle_drag(&mut size) {
                            //debug!("DRAG SIZE {}x{}", size.0, size.1);
                            panel_item.set_size(Some(size.0.point()), Some(size.1.point()));
                        }
                    },
                );
            },
        );
    }
}
