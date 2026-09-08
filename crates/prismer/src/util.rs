use std::ffi::{CStr, CString, c_char};

use crate::error::{Error, Result};

pub(crate) fn to_cstring(text: &str) -> Result<CString> {
	CString::new(text).map_err(|_| Error::InvalidParam)
}

/// # Safety
///
/// `ptr` must be null, or a NUL-terminated string that stays valid for `'a`.
pub(crate) unsafe fn str_from_ptr<'a>(ptr: *const c_char) -> &'a str {
	if ptr.is_null() {
		return "";
	}
	// SAFETY: the caller guarantees `ptr` is a NUL-terminated string valid for
	// `'a`. Non-UTF-8 content degrades to an empty string rather than an error.
	unsafe { CStr::from_ptr(ptr) }.to_str().unwrap_or("")
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
