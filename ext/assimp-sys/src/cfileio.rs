use std::os::raw::c_char;

use types::*;

// AiFile Callbacks
pub type AiFileWriteProc =
    Option<unsafe extern "system" fn(*mut AiFile, *const c_char, usize, usize) -> usize>;
pub type AiFileReadProc =
    Option<unsafe extern "system" fn(*mut AiFile, *mut c_char, usize, usize) -> usize>;
pub type AiFileTellProc =
    Option<unsafe extern "system" fn(*mut AiFile) -> usize>;
pub type AiFileFlushProc =
    Option<unsafe extern "system" fn(*mut AiFile)>;
pub type AiFileSeek =
    Option<unsafe extern "system" fn(*mut AiFile, usize, AiOrigin) -> AiReturn>;

// AiFileIO Callbacks
pub type AiFileOpenProc = Option<unsafe extern "system"
    fn(*mut AiFileIO, *const c_char, *const c_char) -> *mut AiFile>;
pub type AiFileCloseProc = Option<unsafe extern "system"
    fn(*mut AiFileIO, *mut AiFile)>;

// User defined data
pub type AiUserData = *const c_char;

#[repr(C)]
pub struct AiFileIO {
    pub open_proc: AiFileOpenProc,
    pub close_proc: AiFileCloseProc,
    pub user_data: AiUserData,
}

#[repr(C)]
pub struct AiFile {
    pub read_proc: AiFileReadProc,
    pub write_proc: AiFileWriteProc,
    pub tell_proc: AiFileTellProc,
    pub file_size_proc: AiFileTellProc,
    pub seek_proc: AiFileSeek,
    pub flush_proc: AiFileFlushProc,
    pub user_data: AiUserData,
}
