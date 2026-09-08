use core::{fmt, result};
use std::error::Error as StdError;

use prism_sys as sys;

/// An error reported by prism or one of its backends.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum Error {
	/// The library or backend has not been initialized.
	NotInitialized,
	/// A parameter was invalid, such as a string containing an interior NUL.
	InvalidParam,
	/// The operation is not implemented by this backend.
	NotImplemented,
	/// The backend has no voices available.
	NoVoices,
	/// The requested voice does not exist.
	VoiceNotFound,
	/// The backend failed to speak the text.
	SpeakFailure,
	/// A memory allocation failed.
	MemoryFailure,
	/// A value was outside the accepted range.
	RangeOutOfBounds,
	/// An internal error occurred inside prism.
	Internal,
	/// The backend is not currently speaking.
	NotSpeaking,
	/// The backend is not currently paused.
	NotPaused,
	/// The backend is already paused.
	AlreadyPaused,
	/// Text passed to the backend was not valid UTF-8.
	InvalidUtf8,
	/// The operation is invalid in the current state.
	InvalidOperation,
	/// The library or backend was already initialized.
	AlreadyInitialized,
	/// The requested backend is not available on this system.
	BackendNotAvailable,
	/// The backend reported an invalid audio format.
	InvalidAudioFormat,
	/// An internal backend limit was exceeded.
	InternalBackendLimitExceeded,
	/// The backend entered an undefined state.
	BackendEnteredUndefinedState,
	/// A plugin library failed to load.
	LibraryLoadFailed,
	/// A plugin library was not a valid prism plugin.
	LibraryInvalid,
	/// A plugin library was built against an incompatible ABI.
	IncompatibleAbi,
	/// An error code this binding does not know about.
	Unknown(i32),
}

/// A specialized [`Result`](core::result::Result) type for prism operations.
pub type Result<T> = result::Result<T, Error>;

pub(crate) const fn check(code: sys::PrismError) -> Result<()> {
	match code {
		sys::PRISM_OK => Ok(()),
		sys::PRISM_ERROR_NOT_INITIALIZED => Err(Error::NotInitialized),
		sys::PRISM_ERROR_INVALID_PARAM => Err(Error::InvalidParam),
		sys::PRISM_ERROR_NOT_IMPLEMENTED => Err(Error::NotImplemented),
		sys::PRISM_ERROR_NO_VOICES => Err(Error::NoVoices),
		sys::PRISM_ERROR_VOICE_NOT_FOUND => Err(Error::VoiceNotFound),
		sys::PRISM_ERROR_SPEAK_FAILURE => Err(Error::SpeakFailure),
		sys::PRISM_ERROR_MEMORY_FAILURE => Err(Error::MemoryFailure),
		sys::PRISM_ERROR_RANGE_OUT_OF_BOUNDS => Err(Error::RangeOutOfBounds),
		sys::PRISM_ERROR_INTERNAL => Err(Error::Internal),
		sys::PRISM_ERROR_NOT_SPEAKING => Err(Error::NotSpeaking),
		sys::PRISM_ERROR_NOT_PAUSED => Err(Error::NotPaused),
		sys::PRISM_ERROR_ALREADY_PAUSED => Err(Error::AlreadyPaused),
		sys::PRISM_ERROR_INVALID_UTF8 => Err(Error::InvalidUtf8),
		sys::PRISM_ERROR_INVALID_OPERATION => Err(Error::InvalidOperation),
		sys::PRISM_ERROR_ALREADY_INITIALIZED => Err(Error::AlreadyInitialized),
		sys::PRISM_ERROR_BACKEND_NOT_AVAILABLE => Err(Error::BackendNotAvailable),
		sys::PRISM_ERROR_INVALID_AUDIO_FORMAT => Err(Error::InvalidAudioFormat),
		sys::PRISM_ERROR_INTERNAL_BACKEND_LIMIT_EXCEEDED => Err(Error::InternalBackendLimitExceeded),
		sys::PRISM_ERROR_BACKEND_ENTERED_UNDEFINED_STATE => Err(Error::BackendEnteredUndefinedState),
		sys::PRISM_ERROR_LIBRARY_LOAD_FAILED => Err(Error::LibraryLoadFailed),
		sys::PRISM_ERROR_LIBRARY_INVALID => Err(Error::LibraryInvalid),
		sys::PRISM_ERROR_INCOMPATIBLE_ABI => Err(Error::IncompatibleAbi),
		other => Err(Error::Unknown(other)),
	}
}

impl fmt::Display for Error {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		let msg = match self {
			Self::NotInitialized => "not initialized",
			Self::InvalidParam => "invalid parameter",
			Self::NotImplemented => "not implemented",
			Self::NoVoices => "no voices available",
			Self::VoiceNotFound => "voice not found",
			Self::SpeakFailure => "speak failure",
			Self::MemoryFailure => "memory allocation failure",
			Self::RangeOutOfBounds => "range out of bounds",
			Self::Internal => "internal error",
			Self::NotSpeaking => "not speaking",
			Self::NotPaused => "not paused",
			Self::AlreadyPaused => "already paused",
			Self::InvalidUtf8 => "invalid UTF-8",
			Self::InvalidOperation => "invalid operation",
			Self::AlreadyInitialized => "already initialized",
			Self::BackendNotAvailable => "backend not available",
			Self::InvalidAudioFormat => "invalid audio format",
			Self::InternalBackendLimitExceeded => "internal backend limit exceeded",
			Self::BackendEnteredUndefinedState => "backend entered undefined state",
			Self::LibraryLoadFailed => "library load failed",
			Self::LibraryInvalid => "library invalid",
			Self::IncompatibleAbi => "incompatible plugin ABI",
			Self::Unknown(code) => return write!(f, "unknown prism error {code}"),
		};
		f.write_str(msg)
	}
}

impl StdError for Error {}
