use std::ffi::{CStr, CString, c_char};

use crate::error::{Error, Result};

pub(crate) fn to_cstring(text: &str) -> Result<CString> {
	CString::new(text).map_err(|_| Error::InvalidParam)
}

pub(crate) fn copy_cstr(ptr: *const c_char) -> String {
	if ptr.is_null() {
		return String::new();
	}

	// SAFETY: `ptr` is non-null here, and every prism function that returns one
	// documents it as a NUL-terminated string prism owns. The copy finishes
	// before this returns, so no borrow outlives the call.
	unsafe { CStr::from_ptr(ptr) }.to_string_lossy().into_owned()
}
