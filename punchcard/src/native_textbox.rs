use glutin::Window;
use winapi::shared::minwindef::{HINSTANCE, LPARAM};
use winapi::shared::windef::{HMENU, HWND};
use winapi::um::winuser::{CreateWindowExW, GetWindowLongW, SendMessageW, WNDCLASSW, WS_CHILD, WS_VISIBLE, WS_VSCROLL,
                          ES_LEFT, ES_MULTILINE, ES_AUTOVSCROLL, WM_SETTEXT, GWL_HINSTANCE};

use std::ffi::OsStr;
use std::os::windows::ffi::OsStrExt;
use std::iter::once;
use std::ptr::{null, null_mut};

fn win32_string( value : &str ) -> Vec<u16> {
    OsStr::new( value ).encode_wide().chain( once( 0 ) ).collect()
}

pub fn create_edit_box(window: &Window)
{
    let hwnd = unsafe { window.platform_window() as HWND };

    let edit_class = win32_string("EDIT");
    let sample_text = win32_string("みすちードーナツ");

    unsafe {
       /* let edit_handle = CreateWindowExW(
            0,
            edit_class.as_ptr(),   // predefined class
            null(),         // no window title
            WS_CHILD | WS_VISIBLE | WS_VSCROLL |
                ES_LEFT | ES_MULTILINE | ES_AUTOVSCROLL,
            0, 0, 100, 100,   // set size in WM_SIZE message
            hwnd,         // parent window
            1001 as HMENU,   // edit control ID
            GetWindowLongW(hwnd, GWL_HINSTANCE) as HINSTANCE,
            null_mut());        // pointer not needed
        debug!("edit_handle={:?}", edit_handle);

        SendMessageW(edit_handle, WM_SETTEXT, 0, sample_text.as_ptr() as LPARAM);*/
    }
}