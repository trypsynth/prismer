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
mod error;

use core::{
	ffi::c_char,
	ptr::{self, NonNull},
};
use std::ffi::{CStr, c_void};

use prism_sys as sys;

use crate::backend::{copy_cstr, to_cstring};
pub use crate::{
	backend::{Backend, Features, Voice},
	error::{Error, Result},
};

/// The identifier of a backend in the registry.
///
/// The associated constants cover the backends built into prism; plugin
/// backends receive ids outside this set.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct BackendId(pub u64);

impl BackendId {
	/// The reserved invalid backend id.
	pub const INVALID: Self = Self(sys::PRISM_BACKEND_INVALID);
	/// Microsoft Speech API (Windows).
	pub const SAPI: Self = Self(sys::PRISM_BACKEND_SAPI);
	/// `AVSpeechSynthesizer` (macOS and iOS).
	pub const AV_SPEECH: Self = Self(sys::PRISM_BACKEND_AV_SPEECH);
	/// `VoiceOver` (macOS and iOS).
	pub const VOICE_OVER: Self = Self(sys::PRISM_BACKEND_VOICE_OVER);
	/// speech-dispatcher (Linux and BSD).
	pub const SPEECH_DISPATCHER: Self = Self(sys::PRISM_BACKEND_SPEECH_DISPATCHER);
	/// NVDA (Windows).
	pub const NVDA: Self = Self(sys::PRISM_BACKEND_NVDA);
	/// JAWS (Windows).
	pub const JAWS: Self = Self(sys::PRISM_BACKEND_JAWS);
	/// Windows `OneCore` speech.
	pub const ONE_CORE: Self = Self(sys::PRISM_BACKEND_ONE_CORE);
	/// Orca (Linux).
	pub const ORCA: Self = Self(sys::PRISM_BACKEND_ORCA);
	/// The active Android screen reader, such as `TalkBack`.
	pub const ANDROID_SCREEN_READER: Self = Self(sys::PRISM_BACKEND_ANDROID_SCREEN_READER);
	/// Android text-to-speech.
	pub const ANDROID_TTS: Self = Self(sys::PRISM_BACKEND_ANDROID_TTS);
	/// The Web Speech API (WebAssembly).
	pub const WEB_SPEECH: Self = Self(sys::PRISM_BACKEND_WEB_SPEECH);
	/// UI Automation notifications (Windows).
	pub const UIA: Self = Self(sys::PRISM_BACKEND_UIA);
	/// Zhengdu Screen Reader (Windows).
	pub const ZDSR: Self = Self(sys::PRISM_BACKEND_ZDSR);
	/// `ZoomText` (Windows).
	pub const ZOOM_TEXT: Self = Self(sys::PRISM_BACKEND_ZOOM_TEXT);
	/// `BoyPCReader` (Windows).
	pub const BOY_PC_READER: Self = Self(sys::PRISM_BACKEND_BOY_PC_READER);
	/// PC-Talker (Windows).
	pub const PC_TALKER: Self = Self(sys::PRISM_BACKEND_PC_TALKER);
	/// `SenseReader` (Windows).
	pub const SENSE_READER: Self = Self(sys::PRISM_BACKEND_SENSE_READER);
	/// System Access (Windows).
	pub const SYSTEM_ACCESS: Self = Self(sys::PRISM_BACKEND_SYSTEM_ACCESS);
	/// Window-Eyes (Windows).
	pub const WINDOW_EYES: Self = Self(sys::PRISM_BACKEND_WINDOW_EYES);
	/// Spiel (Linux).
	pub const SPIEL: Self = Self(sys::PRISM_BACKEND_SPIEL);
}

type AvailabilityHook = Box<dyn FnMut(BackendId, &str, bool) + Send>;

/// An initialized prism context, holding the backend registry.
///
/// Create one with [`Prism::new`] or configure it first via
/// [`Prism::builder`]. The context shuts down when dropped; [`Backend`]s
/// borrow from it and must be dropped first.
pub struct Prism {
	raw: NonNull<sys::PrismContext>,
	_availability: Option<Box<AvailabilityHook>>,
}

/// A builder for a [`Prism`] context, created by [`Prism::builder`].
pub struct Builder {
	poll_interval_ms: Option<u32>,
	debounce_samples: Option<u32>,
	backoff_max_ms: Option<u32>,
	auto_power_manage: Option<bool>,
	availability: Option<Box<AvailabilityHook>>,
}

unsafe extern "C" fn availability_trampoline(
	userdata: *mut c_void,
	backend: sys::PrismBackendId,
	name: *const c_char,
	available: bool,
) {
	let hook = unsafe { &mut *userdata.cast::<AvailabilityHook>() };
	let name = if name.is_null() { "" } else { unsafe { CStr::from_ptr(name) }.to_str().unwrap_or("") };
	hook(BackendId(backend), name, available);
}

impl Builder {
	/// Sets how often, in milliseconds, prism polls backend availability.
	#[must_use]
	pub const fn poll_interval_ms(mut self, ms: u32) -> Self {
		self.poll_interval_ms = Some(ms);
		self
	}

	/// Sets how many consecutive identical samples an availability change
	/// must survive before it is reported.
	#[must_use]
	pub const fn debounce_samples(mut self, samples: u32) -> Self {
		self.debounce_samples = Some(samples);
		self
	}

	/// Sets the maximum poll interval, in milliseconds, that availability
	/// polling backs off to while idle.
	#[must_use]
	pub const fn backoff_max_ms(mut self, ms: u32) -> Self {
		self.backoff_max_ms = Some(ms);
		self
	}

	/// Enables or disables automatic pausing of availability polling while
	/// the system is on battery power.
	#[must_use]
	pub const fn auto_power_manage(mut self, enabled: bool) -> Self {
		self.auto_power_manage = Some(enabled);
		self
	}

	/// Registers a callback invoked whenever a backend becomes available or
	/// unavailable.
	#[must_use]
	pub fn on_availability<F>(mut self, hook: F) -> Self
	where
		F: FnMut(BackendId, &str, bool) + Send + 'static,
	{
		self.availability = Some(Box::new(Box::new(hook)));
		self
	}

	/// Initializes prism with this configuration.
	///
	/// # Errors
	///
	/// Returns [`Error::Internal`] if prism fails to initialize.
	pub fn build(self) -> Result<Prism> {
		let mut cfg = unsafe { sys::prism_config_init() };
		if let Some(ms) = self.poll_interval_ms {
			cfg.availability_poll_interval_ms = ms;
		}
		if let Some(samples) = self.debounce_samples {
			cfg.availability_debounce_samples = samples;
		}
		if let Some(ms) = self.backoff_max_ms {
			cfg.availability_backoff_max_ms = ms;
		}
		if let Some(enabled) = self.auto_power_manage {
			cfg.availability_auto_power_manage = enabled;
		}
		let mut availability = self.availability;
		if let Some(hook) = availability.as_mut() {
			cfg.availability_callback = Some(availability_trampoline);
			cfg.availability_userdata = ptr::from_mut::<AvailabilityHook>(hook.as_mut()).cast::<c_void>();
		}
		let raw = unsafe { sys::prism_init(&raw mut cfg) };
		NonNull::new(raw).map_or(Err(Error::Internal), |raw| Ok(Prism { raw, _availability: availability }))
	}
}

impl Prism {
	/// Initializes prism with the default configuration.
	///
	/// # Errors
	///
	/// Returns [`Error::Internal`] if prism fails to initialize.
	pub fn new() -> Result<Self> {
		Self::builder().build()
	}

	/// Returns a [`Builder`] for configuring a context before initializing
	/// it.
	#[must_use]
	pub const fn builder() -> Builder {
		Builder {
			poll_interval_ms: None,
			debounce_samples: None,
			backoff_max_ms: None,
			auto_power_manage: None,
			availability: None,
		}
	}

	const fn ptr(&self) -> *mut sys::PrismContext {
		self.raw.as_ptr()
	}

	/// Returns the number of backends in the registry.
	#[must_use]
	pub fn backend_count(&self) -> usize {
		unsafe { sys::prism_registry_count(self.ptr()) }
	}

	/// Returns the ids of every backend in the registry.
	#[must_use]
	pub fn backend_ids(&self) -> Vec<BackendId> {
		let count = self.backend_count();
		(0..count).map(|i| BackendId(unsafe { sys::prism_registry_id_at(self.ptr(), i) })).collect()
	}

	/// Looks up a backend id by its registered name.
	///
	/// # Errors
	///
	/// Returns [`Error::BackendNotAvailable`] if no backend has that name, or
	/// [`Error::InvalidParam`] if `name` contains an interior NUL.
	pub fn backend_id_by_name(&self, name: &str) -> Result<BackendId> {
		let name = to_cstring(name)?;
		let id = unsafe { sys::prism_registry_id(self.ptr(), name.as_ptr()) };
		if id == sys::PRISM_BACKEND_INVALID { Err(Error::BackendNotAvailable) } else { Ok(BackendId(id)) }
	}

	/// Returns the registered name of a backend, or `None` if the id is not
	/// in the registry.
	#[must_use]
	pub fn backend_name(&self, id: BackendId) -> Option<String> {
		let ptr = unsafe { sys::prism_registry_name(self.ptr(), id.0) };
		if ptr.is_null() { None } else { Some(copy_cstr(ptr)) }
	}

	/// Returns the priority of a backend; higher-priority backends are
	/// preferred by [`Prism::acquire_best`].
	#[must_use]
	pub fn backend_priority(&self, id: BackendId) -> i32 {
		unsafe { sys::prism_registry_priority(self.ptr(), id.0) }
	}

	/// Returns `true` if the registry contains a backend with this id.
	#[must_use]
	pub fn backend_exists(&self, id: BackendId) -> bool {
		unsafe { sys::prism_registry_exists(self.ptr(), id.0) }
	}

	/// Creates a new, independent instance of the backend with this id.
	///
	/// The returned backend is not initialized; call
	/// [`Backend::initialize`] before any other method.
	///
	/// Use this when the backend state (voice, rate, pitch) must be yours
	/// alone. See [`Prism::acquire`] for the shared alternative.
	///
	/// # Errors
	///
	/// Returns [`Error::BackendNotAvailable`] if the backend cannot be
	/// created.
	pub fn create(&self, id: BackendId) -> Result<Backend<'_>> {
		Backend::from_raw(unsafe { sys::prism_registry_create(self.ptr(), id.0) })
	}

	/// Creates a new, independent instance of the highest-priority backend
	/// that initializes successfully.
	///
	/// This is the recommended way to obtain a backend when the application
	/// has no specific preference. The returned backend is already
	/// initialized; do not call [`Backend::initialize`] on it.
	///
	/// # Errors
	///
	/// Returns [`Error::BackendNotAvailable`] if no backend can be created
	/// and initialized.
	pub fn create_best(&self) -> Result<Backend<'_>> {
		Backend::from_raw(unsafe { sys::prism_registry_create_best(self.ptr()) })
	}

	/// Returns a shared instance of the backend with this id, reusing a
	/// cached instance if one is still alive.
	///
	/// Every handle to a cached instance shares one backend state, so a
	/// voice or rate set through one handle is visible through all of them.
	/// Use this only when that sharing is what you want; prefer
	/// [`Prism::create`] otherwise.
	///
	/// The returned backend may already be initialized, depending on whether
	/// it came from the cache. Call [`Backend::initialize`] and treat
	/// [`Error::AlreadyInitialized`] as success.
	///
	/// # Errors
	///
	/// Returns [`Error::BackendNotAvailable`] if the backend cannot be
	/// created.
	pub fn acquire(&self, id: BackendId) -> Result<Backend<'_>> {
		Backend::from_raw(unsafe { sys::prism_registry_acquire(self.ptr(), id.0) })
	}

	/// Returns a shared instance of the highest-priority backend that
	/// initializes successfully, reusing a cached instance if one is still
	/// alive.
	///
	/// Every handle to a cached instance shares one backend state, so a
	/// voice or rate set through one handle is visible through all of them.
	/// Use this only when that sharing is what you want; prefer
	/// [`Prism::create_best`] otherwise.
	///
	/// The returned backend is always initialized; do not call
	/// [`Backend::initialize`] on it.
	///
	/// # Errors
	///
	/// Returns [`Error::BackendNotAvailable`] if no backend can be acquired
	/// and initialized.
	pub fn acquire_best(&self) -> Result<Backend<'_>> {
		Backend::from_raw(unsafe { sys::prism_registry_acquire_best(self.ptr()) })
	}

	/// Pauses background availability polling.
	pub fn pause_availability_polling(&self) {
		unsafe { sys::prism_availability_poll_pause(self.ptr()) };
	}

	/// Resumes background availability polling.
	pub fn resume_availability_polling(&self) {
		unsafe { sys::prism_availability_poll_resume(self.ptr()) };
	}
}

impl Drop for Prism {
	fn drop(&mut self) {
		unsafe { sys::prism_shutdown(self.ptr()) };
	}
}

/// Returns `true` if prism supports automatic power management of
/// availability polling on this platform.
#[must_use]
pub fn auto_power_management_supported() -> bool {
	unsafe { sys::prism_availability_auto_power_supported() }
}

/// Returns prism's version as a packed integer.
#[must_use]
pub fn version() -> u32 {
	unsafe { sys::prism_version() }
}

/// Returns prism's version as a human-readable string.
#[must_use]
pub fn version_string() -> String {
	copy_cstr(unsafe { sys::prism_version_string() })
}
