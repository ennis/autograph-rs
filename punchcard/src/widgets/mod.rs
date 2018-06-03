pub mod button;
pub mod panel;
pub mod scroll;
pub mod slider;
pub mod text;
pub mod text_edit;

pub use self::button::*;
pub use self::panel::*;
pub use self::scroll::*;
pub use self::slider::*;
pub use self::text::*;
pub use self::text_edit::*;

/// Unused for now.
pub struct ItemResult {
    /// The item was clicked since the last call
    pub clicked: bool,
    /// The mouse is hovering over the item
    pub hover: bool,
}
