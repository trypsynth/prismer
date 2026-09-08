use core::{
	ffi::c_char,
	fmt,
	ptr::{self, NonNull},
};
use std::ffi::c_void;

use prism_sys as sys;

use crate::{
	Backend, BackendId, Registry,
	error::{Error, Result},
	util::{copy_cstr, str_from_ptr, to_cstring},
};

type AvailabilityHook = Box<dyn FnMut(BackendId, &str, bool) + Send>;

/// An initialized prism context, holding the backend registry.
///
/// Create one with [`Prism::new`] or configure it first via
/// [`Prism::builder`]. The context shuts down when dropped; [`Backend`]s
/// borrow from it and must be dropped first.
pub struct Prism {
	// Invariant: a live context from prism_init. Only Drop releases it, and it
	// runs at most once.
	raw: NonNull<sys::PrismContext>,
	_availability: Option<Box<AvailabilityHook>>,
}

/// A builder for a [`Prism`] context, created by [`Prism::builder`].
pub struct Builder {
	registry: Option<Registry>,
	poll_interval_ms: Option<u32>,
	debounce_samples: Option<u32>,
	backoff_max_ms: Option<u32>,
	auto_power_manage: Option<bool>,
	availability: Option<Box<AvailabilityHook>>,
}

/// # Safety
///
/// `userdata` must be the pointer to the live [`AvailabilityHook`] that was
/// registered alongside this function, and `name` must be null or a
/// NUL-terminated string.
unsafe extern "C" fn availability_trampoline(
	userdata: *mut c_void,
	backend: sys::PrismBackendId,
	name: *const c_char,
	available: bool,
) {
	// SAFETY: the caller guarantees `userdata` points at the live hook owned
	// by the Prism this callback was registered with. Prism::drop calls
	// prism_shutdown, which stops the poll thread, before the hook is dropped,
	// so this is the only reference to it while it is held.
	let hook = unsafe { &mut *userdata.cast::<AvailabilityHook>() };
	// SAFETY: prism documents `name` as null or a NUL-terminated backend name
	// that stays valid for the callback.
	let name = unsafe { str_from_ptr(name) };
	hook(BackendId(backend), name, available);
}

impl Builder {
	/// Binds the context to `registry` instead of prism's default set of
	/// backends.
	///
	/// Build one with [`RegistryBuilder`](crate::RegistryBuilder) when the
	/// context should see custom or plugin backends. The context keeps its own
	/// reference for as long as it lives.
	#[must_use]
	pub fn registry(mut self, registry: &Registry) -> Self {
		self.registry = Some(registry.clone());
		self
	}

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
		// SAFETY: takes no arguments and returns a config struct by value.
		let mut cfg = unsafe { sys::prism_config_init() };
		if let Some(registry) = self.registry.as_ref() {
			cfg.registry = registry.ptr();
		}
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
		// SAFETY: `cfg` is a fully initialized config that outlives the call.
		// prism_init copies what it needs. If a hook was set, its userdata points
		// into `availability`, which moves into the returned Prism and so outlives
		// the context.
		let raw = unsafe { sys::prism_init(&raw mut cfg) };
		NonNull::new(raw).map_or(Err(Error::Internal), |raw| Ok(Prism { raw, _availability: availability }))
	}
}

impl fmt::Debug for Builder {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		f.debug_struct("Builder")
			.field("registry", &self.registry)
			.field("poll_interval_ms", &self.poll_interval_ms)
			.field("debounce_samples", &self.debounce_samples)
			.field("backoff_max_ms", &self.backoff_max_ms)
			.field("auto_power_manage", &self.auto_power_manage)
			.field("availability", &self.availability.is_some())
			.finish()
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
			registry: None,
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
		// SAFETY: `raw` is a live context (type invariant).
		unsafe { sys::prism_registry_count(self.ptr()) }
	}

	/// Returns the ids of every backend in the registry.
	#[must_use]
	pub fn backend_ids(&self) -> Vec<BackendId> {
		let count = self.backend_count();
		// SAFETY: `raw` is a live context, and `i` stays below the count just read
		// from the same registry.
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
		// SAFETY: `raw` is a live context, and `name` is a NUL-terminated CString
		// that outlives the call.
		let id = unsafe { sys::prism_registry_id(self.ptr(), name.as_ptr()) };
		if id == sys::PRISM_BACKEND_INVALID { Err(Error::BackendNotAvailable) } else { Ok(BackendId(id)) }
	}

	/// Returns the registered name of a backend, or `None` if the id is not
	/// in the registry.
	#[must_use]
	pub fn backend_name(&self, id: BackendId) -> Option<String> {
		// SAFETY: `raw` is a live context. An unregistered id yields null, which
		// the caller checks.
		let ptr = unsafe { sys::prism_registry_name(self.ptr(), id.0) };
		if ptr.is_null() { None } else { Some(copy_cstr(ptr)) }
	}

	/// Returns the priority of a backend; higher-priority backends are
	/// preferred by [`Prism::acquire_best`].
	#[must_use]
	pub fn backend_priority(&self, id: BackendId) -> i32 {
		// SAFETY: `raw` is a live context (type invariant).
		unsafe { sys::prism_registry_priority(self.ptr(), id.0) }
	}

	/// Returns `true` if the registry contains a backend with this id.
	#[must_use]
	pub fn backend_exists(&self, id: BackendId) -> bool {
		// SAFETY: `raw` is a live context (type invariant).
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
		// SAFETY: `raw` is a live context. A failed lookup yields null, which
		// Backend::from_raw rejects.
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
		// SAFETY: `raw` is a live context. Null when nothing initializes, which
		// Backend::from_raw rejects.
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
		// SAFETY: `raw` is a live context. A failed lookup yields null, which
		// Backend::from_raw rejects.
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
		// SAFETY: `raw` is a live context. Null when nothing initializes, which
		// Backend::from_raw rejects.
		Backend::from_raw(unsafe { sys::prism_registry_acquire_best(self.ptr()) })
	}

	/// Pauses background availability polling.
	pub fn pause_availability_polling(&self) {
		// SAFETY: `raw` is a live context (type invariant).
		unsafe { sys::prism_availability_poll_pause(self.ptr()) };
	}

	/// Resumes background availability polling.
	pub fn resume_availability_polling(&self) {
		// SAFETY: `raw` is a live context (type invariant).
		unsafe { sys::prism_availability_poll_resume(self.ptr()) };
	}
}

impl fmt::Debug for Prism {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		f.debug_struct("Prism").field("backend_count", &self.backend_count()).finish_non_exhaustive()
	}
}

impl Drop for Prism {
	fn drop(&mut self) {
		// SAFETY: `raw` is a live context and Drop runs at most once. Fields drop
		// after this returns, so the availability hook outlives the poll thread
		// that prism_shutdown stops.
		unsafe { sys::prism_shutdown(self.ptr()) };
	}
}
