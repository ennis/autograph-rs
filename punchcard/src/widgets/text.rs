//! Text
use super::super::*;

impl<'a> UiContainer<'a> {
    ///
    /// Text with class.
    ///
    pub fn text_class<S>(&mut self, text: S, class: &str) -> ItemResult
    where
        S: Into<String> + Clone,
    {
        //=====================================
        // behavior
        struct Text(String);
        impl Behavior for Text {
            fn draw(&mut self, item: &mut Item, draw_list: &mut DrawList) {
                draw_list.add_text(
                    self.0.clone(),
                    item.layout.clone(),
                    item.style.clone(),
                );
            }

            fn measure(&mut self, item: &mut Item, renderer: &Renderer) -> ContentMeasurement {
                let m = renderer.measure_text(&self.0, &item.style);
                ContentMeasurement {
                    width: Some(m),
                    height: Some(item.style.font.font_size),
                }
            }
        }

        //=====================================
        // hierarchy
        self.item(text.clone(), class, Text(text.into()), |_, _, _| {});

        //=====================================
        // result
        ItemResult {
            clicked: false,
            hover: false,
        }
    }

    ///
    /// Text.
    ///
    pub fn text<S>(&mut self, text: S) -> ItemResult
    where
        S: Into<String> + Clone,
    {
        self.text_class(text, "text")
    }
}
