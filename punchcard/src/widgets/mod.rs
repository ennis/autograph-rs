pub mod button;
pub mod panel;
pub mod collapsing;
pub mod floating;
//pub mod scroll;
//pub mod slider;
//pub mod text;
//pub mod text_edit;

use super::*;
pub use self::button::*;
pub use self::panel::*;
pub use self::collapsing::*;
pub use self::floating::*;

/// Adds children to the dom if condition is true. Equivalent to:
/// ```
/// if condition {
///     f(dom);
/// }
/// ```
/// This is a convenience function to be used within the dom!() macro,
/// which does not handle if-statements.
pub fn condition(dom: &mut DomSink, condition: bool, f: impl FnOnce(&mut DomSink)) {
    if condition {
        f(dom);
    }
}

