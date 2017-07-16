use std::ffi::CString;
use std::fmt::{Debug, Formatter, Result};
use std::str;

use std::os::raw::c_uchar;
pub const MAXLEN: usize = 1024;

#[repr(C)]
#[derive(Copy)]
pub struct AiString {
    pub length: usize,
    pub data: [c_uchar; MAXLEN],
}

impl Default for AiString {
    fn default() -> AiString {
        AiString {
            length: 0,
            data: [0; MAXLEN],
        }
    }
}

impl AsRef<str> for AiString {
    fn as_ref(&self) -> &str {
        str::from_utf8(&self.data[0..self.length]).unwrap()
    }
}

impl<'a> From<&'a str> for AiString {
    fn from(s: &str) -> AiString {
        assert!(s.len() < MAXLEN);

        let cstr = CString::new(s).unwrap();
        let bytes = cstr.to_bytes();

        let mut aistr = AiString {
            length: s.len() as usize,
            data: [0; MAXLEN],
        };
        for i in 0..s.len() {
            aistr.data[i] = bytes[i];
        }
        aistr
    }
}

impl Clone for AiString {
    fn clone(&self) -> AiString {
        *self
    }
}

impl Debug for AiString {
    fn fmt(&self, f: &mut Formatter) -> Result {
        let s: &str = self.as_ref();
        write!(f, "{:?}", s)
    }
}

impl PartialEq for AiString {
    fn eq(&self, other: &AiString) -> bool {
        self.as_ref() == other.as_ref()
    }
}

impl Eq for AiString {}
