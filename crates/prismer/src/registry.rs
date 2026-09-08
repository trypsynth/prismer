use core::{ffi::c_int, fmt, ptr::NonNull};
use std::{ffi::c_void, path::Path};

use prism_sys as sys;

use crate::{
	BackendId, CustomBackend, Features,
	error::{Error, Result, check},
	util::to_cstring,
	vtable::{Factory, free_factory, vtable_for},
};

/// An immutable set of backends, frozen from a [`RegistryBuilder`].
///
/// Bind one to a context with [`Builder::registry`](crate::Builder::registry).
/// Registries are reference counted; [`Clone`] takes another reference and
/// dropping releases one. A registry outlives the contexts bound to it.
pub struct Registry {
	// Invariant: a live registry we hold one reference to. Drop releases that
	// reference, and it runs at most once.
	raw: NonNull<sys::PrismRegistry>,
}

impl Registry {
	pub(crate) const fn ptr(&self) -> *mut sys::PrismRegistry {
		self.raw.as_ptr()
	}
}

impl Clone for Registry {
	fn clone(&self) -> Self {
		// SAFETY: `raw` is a live registry, and retain returns the same pointer
		// with the count raised by one, which our Drop balances.
		let _retained = unsafe { sys::prism_registry_retain(self.ptr()) };
		Self { raw: self.raw }
	}
}

impl fmt::Debug for Registry {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		f.debug_struct("Registry").finish_non_exhaustive()
	}
}

impl Drop for Registry {
	fn drop(&mut self) {
		// SAFETY: `raw` is live and Drop runs at most once, so this releases
		// exactly the one reference this value owns.
		unsafe { sys::prism_registry_release(self.ptr()) };
	}
}

/// Collects custom backends into a [`Registry`].
///
/// A new builder already holds every backend compiled into prism, so a frozen
/// registry is always a superset of the default one.
pub struct RegistryBuilder {
	// Invariant: a live builder from prism_registry_builder_new. Drop frees it,
	// and it runs at most once, whether or not the builder was frozen first.
	raw: NonNull<sys::PrismRegistryBuilder>,
}

impl RegistryBuilder {
	/// Creates a builder seeded with prism's compiled-in backends.
	///
	/// # Errors
	///
	/// Returns [`Error::MemoryFailure`] if allocation fails.
	pub fn new() -> Result<Self> {
		// SAFETY: takes no arguments and returns an owned builder or null.
		let raw = unsafe { sys::prism_registry_builder_new() };
		NonNull::new(raw).map_or(Err(Error::MemoryFailure), |raw| Ok(Self { raw }))
	}

	const fn ptr(&self) -> *mut sys::PrismRegistryBuilder {
		self.raw.as_ptr()
	}

	/// Registers a custom backend, returning the id prism derives from `name`.
	///
	/// `factory` produces one backend value per instance prism creates, so
	/// instances never share state. Returning `None` fails that instance's
	/// creation.
	///
	/// `features` must list exactly the operations the backend implements.
	/// prism rejects a registration whose declared features do not match the
	/// installed operations, and only declared operations are ever called.
	///
	/// # Errors
	///
	/// Returns [`Error::InvalidParam`] if `name` contains an interior NUL or
	/// `priority` exceeds [`i32::MAX`], [`Error::InvalidOperation`] if the name
	/// or its id is already registered, or another [`Error`] from prism.
	pub fn add_backend<B, F>(&mut self, name: &str, priority: u32, features: Features, factory: F) -> Result<BackendId>
	where
		B: CustomBackend,
		F: FnMut() -> Option<B> + Send + 'static,
	{
		let name = to_cstring(name)?;
		let priority = c_int::try_from(priority).map_err(|_| Error::InvalidParam)?;
		let vtable = vtable_for::<B>(features);
		let factory: Factory<B> = Box::new(factory);
		let userdata = Box::into_raw(Box::new(factory)).cast::<c_void>();
		let mut id = sys::PRISM_BACKEND_INVALID;
		// SAFETY: `raw` is a live builder, `name` and `vtable` outlive the call
		// (prism copies both), and `userdata` is a leaked Factory<B> that prism
		// takes ownership of: it calls free_factory::<B> exactly once for this
		// registration whether or not the call succeeds, so we never free it.
		let code = unsafe {
			sys::prism_registry_builder_add_backend(
				self.ptr(),
				name.as_ptr(),
				priority,
				features.0,
				&raw const vtable,
				userdata,
				Some(free_factory::<B>),
				&raw mut id,
			)
		};
		check(code)?;
		Ok(BackendId(id))
	}

	/// Loads a prism plugin library and registers every backend it supplies,
	/// returning how many were added.
	///
	/// Pass `Some(priority)` to give every backend from the library that
	/// priority, or `None` to honor the priority each one declares.
	///
	/// Loading a library runs that library's initialization code, so treat this
	/// as running code of the library's choosing. Some platforms restrict or
	/// forbid loading code at run time.
	///
	/// # Errors
	///
	/// Returns [`Error::InvalidParam`] if `path` is not UTF-8, contains an
	/// interior NUL, or `priority_override` exceeds [`i32::MAX`],
	/// [`Error::LibraryLoadFailed`] if the library cannot be opened,
	/// [`Error::LibraryInvalid`] if it is not a prism plugin, or
	/// [`Error::IncompatibleAbi`] if it was built against a different ABI.
	pub fn add_library(&mut self, path: &Path, priority_override: Option<u32>) -> Result<usize> {
		let path = to_cstring(path.to_str().ok_or(Error::InvalidParam)?)?;
		let priority = match priority_override {
			Some(priority) => c_int::try_from(priority).map_err(|_| Error::InvalidParam)?,
			None => -1,
		};
		let mut count = 0;
		// SAFETY: `raw` is a live builder, `path` is a NUL-terminated CString
		// that outlives the call, and `count` is a live local prism writes only
		// on success.
		let code =
			unsafe { sys::prism_registry_builder_add_library(self.ptr(), path.as_ptr(), priority, &raw mut count) };
		check(code)?;
		Ok(count)
	}

	/// Freezes the builder into an immutable [`Registry`].
	///
	/// # Errors
	///
	/// Returns [`Error::MemoryFailure`] if allocation fails.
	pub fn freeze(self) -> Result<Registry> {
		// SAFETY: `raw` is a live builder that has not been frozen: freeze
		// consumes it, so it cannot be reached again. Our Drop still frees the
		// spent builder afterwards, which prism requires.
		let raw = unsafe { sys::prism_registry_freeze(self.ptr()) };
		NonNull::new(raw).map_or(Err(Error::MemoryFailure), |raw| Ok(Registry { raw }))
	}
}

impl fmt::Debug for RegistryBuilder {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		f.debug_struct("RegistryBuilder").finish_non_exhaustive()
	}
}

impl Drop for RegistryBuilder {
	fn drop(&mut self) {
		// SAFETY: `raw` is a live builder in either state and Drop runs at most
		// once. Freeing a live one discards its registrations, which is how
		// their factories get released.
		unsafe { sys::prism_registry_builder_free(self.ptr()) };
	}
}
