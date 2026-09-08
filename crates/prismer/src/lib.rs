//! Safe Rust bindings to prism (<https://github.com/ethindp/prism>), for
//! speech, braille, and screen reader output in Rust applications.
//!
//! ```no_run
//! let prism = prismer::Prism::new()?;
//! let backend = prism.create_best()?;
//! backend.speak("Hello from Rust!", false)?;
//! # Ok::<(), prismer::Error>(())
//! ```

mod backend;
mod context;
mod error;
mod features;
mod id;
mod util;

use prism_sys as sys;

use crate::util::copy_cstr;
pub use crate::{
	backend::{Backend, Voice},
	context::{Builder, Prism},
	error::{Error, Result},
	features::Features,
	id::BackendId,
};

/// Returns `true` if prism supports automatic power management of
/// availability polling on this platform.
#[must_use]
pub fn auto_power_management_supported() -> bool {
	// SAFETY: takes no arguments and reads no context state.
	unsafe { sys::prism_availability_auto_power_supported() }
}

/// Returns prism's version as a packed integer.
#[must_use]
pub fn version() -> u32 {
	// SAFETY: takes no arguments and reads no context state.
	unsafe { sys::prism_version() }
}

/// Returns prism's version as a human-readable string.
#[must_use]
pub fn version_string() -> String {
	// SAFETY: takes no arguments and returns a static NUL-terminated string
	// owned by prism.
	copy_cstr(unsafe { sys::prism_version_string() })
}
