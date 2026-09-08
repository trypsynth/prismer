use core::ptr;
use std::ffi::{c_char, c_void};

use prism_sys as sys;

use crate::{
	error::Result,
	util::{str_from_ptr, to_cstring},
};

/// The severity of a log message, and the threshold for delivering them.
///
/// The variants are ordered from least to most severe. A message is delivered
/// only when its level is at or above the threshold set by [`set_level`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Level {
	/// Fine-grained tracing of prism's internals.
	Trace,
	/// Diagnostic information useful during development.
	Debug,
	/// Messages describing normal operation.
	Info,
	/// Conditions that are not errors but may point at a problem.
	Warn,
	/// Error conditions.
	Error,
	/// Not a message severity. As a threshold it discards every message.
	None,
}

impl Level {
	const fn to_raw(self) -> sys::PrismLogLevel {
		match self {
			Self::Trace => sys::PRISM_LOG_LEVEL_TRACE,
			Self::Debug => sys::PRISM_LOG_LEVEL_DEBUG,
			Self::Info => sys::PRISM_LOG_LEVEL_INFO,
			Self::Warn => sys::PRISM_LOG_LEVEL_WARN,
			Self::Error => sys::PRISM_LOG_LEVEL_ERROR,
			Self::None => sys::PRISM_LOG_LEVEL_NONE,
		}
	}

	const fn from_raw(raw: sys::PrismLogLevel) -> Self {
		match raw {
			sys::PRISM_LOG_LEVEL_TRACE => Self::Trace,
			sys::PRISM_LOG_LEVEL_DEBUG => Self::Debug,
			sys::PRISM_LOG_LEVEL_INFO => Self::Info,
			sys::PRISM_LOG_LEVEL_WARN => Self::Warn,
			sys::PRISM_LOG_LEVEL_ERROR => Self::Error,
			_ => Self::None,
		}
	}
}

type Hook = Box<dyn FnMut(Level, &str, &str) + Send>;

/// # Safety
///
/// `userdata` must be the pointer to the leaked [`Hook`] registered alongside
/// this function, and `source` and `message` must be null or NUL-terminated
/// strings that stay valid for the call.
unsafe extern "C" fn trampoline(
	userdata: *mut c_void,
	level: sys::PrismLogLevel,
	source: *const c_char,
	message: *const c_char,
) {
	// SAFETY: the caller guarantees `userdata` points at the leaked hook. prism
	// serializes delivery on one logging thread, so no other reference to the
	// hook exists while this one is held, and the leak means it never dangles.
	let hook = unsafe { &mut *userdata.cast::<Hook>() };
	// SAFETY: prism documents both strings as valid for the duration of the
	// call, which ends before the borrows do.
	let (source, message) = unsafe { (str_from_ptr(source), str_from_ptr(message)) };
	hook(Level::from_raw(level), source, message);
}

/// Installs `handler` to receive prism's diagnostic messages, replacing any
/// handler already installed.
///
/// prism calls `handler` from its own logging thread, never concurrently with
/// itself. Keep the work short: the logging thread cannot deliver anything
/// else until it returns, and messages queued meanwhile may be dropped. The
/// handler must not call [`emit`], [`flush`], or [`shutdown`].
///
/// `handler` is leaked. prism may still deliver a message to a replaced
/// handler after the replacing call returns, so there is no point at which
/// freeing it would be sound.
pub fn set_handler<F>(handler: F)
where
	F: FnMut(Level, &str, &str) + Send + 'static,
{
	let hook: Hook = Box::new(handler);
	let userdata = Box::into_raw(Box::new(hook)).cast::<c_void>();
	let handler = sys::PrismLogHandler { fn_: Some(trampoline), userdata };
	// SAFETY: `trampoline` matches the callback ABI prism expects, and
	// `userdata` points at a leaked hook that outlives every delivery.
	let _previous = unsafe { sys::prism_set_log_handler(handler) };
}

/// Removes the installed handler, so messages are discarded again.
///
/// The handler passed to [`set_handler`] stays leaked; see that function.
pub fn clear_handler() {
	let handler = sys::PrismLogHandler { fn_: None, userdata: ptr::null_mut() };
	// SAFETY: a null `fn_` is how prism is told to discard messages.
	let _previous = unsafe { sys::prism_set_log_handler(handler) };
}

/// Sets the lowest severity prism will deliver, returning the previous
/// threshold.
///
/// The threshold is checked on the thread producing the message, before it is
/// queued, so raising it cuts the work done for suppressed messages.
#[must_use]
pub fn set_level(level: Level) -> Level {
	// SAFETY: takes a plain enum value and needs no context.
	Level::from_raw(unsafe { sys::prism_set_log_level(level.to_raw()) })
}

/// Emits a log message through prism's logger.
///
/// The message is copied and queued; this does not call the handler or wait
/// for delivery. Messages below the current threshold are dropped without
/// being queued.
///
/// # Errors
///
/// Returns [`Error::InvalidParam`](crate::Error::InvalidParam) if `source` or
/// `message` contains an interior NUL.
pub fn emit(level: Level, source: &str, message: &str) -> Result<()> {
	let source = to_cstring(source)?;
	let message = to_cstring(message)?;
	// SAFETY: both arguments are NUL-terminated CStrings that outlive the call,
	// and prism copies whatever it queues.
	unsafe { sys::prism_log(level.to_raw(), source.as_ptr(), message.as_ptr()) };
	Ok(())
}

/// Blocks until every message queued before this call has been delivered.
///
/// Must not be called from a log handler.
pub fn flush() {
	// SAFETY: takes no arguments and needs no context.
	unsafe { sys::prism_log_flush() };
}

/// Shuts the logging subsystem down, flushing what is queued.
///
/// Must not be called from a log handler.
pub fn shutdown() {
	// SAFETY: takes no arguments and needs no context.
	unsafe { sys::prism_log_shutdown() };
}
