use core::{
	fmt,
	marker::PhantomData,
	ptr::{self, NonNull},
	slice,
};
use std::ffi::c_void;

use prism_sys as sys;

use crate::{
	Features, Prism,
	error::{Error, Result, check},
	util::{copy_cstr, to_cstring},
};

/// A voice offered by a backend, as returned by [`Backend::voices`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Voice {
	/// The voice's index, for use with [`Backend::set_voice`].
	pub id: usize,
	/// The voice's human-readable name.
	pub name: String,
	/// The voice's language tag, if the backend reports one.
	pub language: Option<String>,
}

/// A speech, braille, or screen reader output backend.
///
/// Backends are created from a [`Prism`] context and freed on drop. Not every
/// backend supports every operation; query [`Backend::features`] or
/// [`Backend::supports`] before relying on one.
pub struct Backend<'a> {
	// Invariant: a live backend handle from the registry. Only Drop releases
	// it, and it runs at most once.
	raw: NonNull<sys::PrismBackend>,
	_ctx: PhantomData<&'a Prism>,
}

impl Backend<'_> {
	pub(crate) const fn from_raw(raw: *mut sys::PrismBackend) -> Result<Self> {
		match NonNull::new(raw) {
			Some(raw) => Ok(Self { raw, _ctx: PhantomData }),
			None => Err(Error::BackendNotAvailable),
		}
	}

	const fn ptr(&self) -> *mut sys::PrismBackend {
		self.raw.as_ptr()
	}

	/// Returns the backend's human-readable name.
	#[must_use]
	pub fn name(&self) -> String {
		// SAFETY: `raw` is a live backend. prism documents name as valid before
		// initialize.
		copy_cstr(unsafe { sys::prism_backend_name(self.ptr()) })
	}

	/// Returns the set of features this backend advertises.
	#[must_use]
	pub fn features(&self) -> Features {
		// SAFETY: `raw` is a live backend. prism documents get_features as valid
		// in any initialization state.
		Features(unsafe { sys::prism_backend_get_features(self.ptr()) })
	}

	/// Returns `true` if the backend advertises every feature in `features`.
	#[must_use]
	pub fn supports(&self, features: Features) -> bool {
		self.features().contains(features)
	}

	/// Initializes the backend.
	///
	/// Backends from [`Prism::create`] and [`Prism::acquire`] need this.
	/// Backends from [`Prism::create_best`] and [`Prism::acquire_best`] are
	/// already initialized and report [`Error::AlreadyInitialized`] here,
	/// which callers may treat as success.
	///
	/// # Errors
	///
	/// Returns an [`Error`] if the backend fails to initialize.
	pub fn initialize(&self) -> Result<()> {
		// SAFETY: `raw` is a live backend (type invariant).
		check(unsafe { sys::prism_backend_initialize(self.ptr()) })
	}

	/// Speaks `text`, optionally interrupting any speech in progress.
	///
	/// # Errors
	///
	/// Returns an [`Error`] if the backend does not support speech, `text`
	/// contains an interior NUL, or speaking fails.
	pub fn speak(&self, text: &str, interrupt: bool) -> Result<()> {
		let text = to_cstring(text)?;
		// SAFETY: `raw` is a live backend, and `text` is a NUL-terminated CString
		// that outlives the call.
		check(unsafe { sys::prism_backend_speak(self.ptr(), text.as_ptr(), interrupt) })
	}

	/// Synthesizes `text` to memory, invoking `on_audio` with chunks of
	/// interleaved `f32` samples along with their channel count and sample
	/// rate.
	///
	/// This call blocks until synthesis finishes. `on_audio` may run once with
	/// all the audio or many times with parts of it, and prism may run it on a
	/// worker thread, which is why it must be [`Send`].
	///
	/// # Errors
	///
	/// Returns an [`Error`] if the backend does not support synthesis to
	/// memory, `text` contains an interior NUL, or synthesis fails.
	pub fn speak_to_memory<F>(&self, text: &str, mut on_audio: F) -> Result<()>
	where
		F: FnMut(&[f32], usize, usize) + Send,
	{
		/// # Safety
		///
		/// `userdata` must point at a live `F`, and `samples` must be null or
		/// the start of `sample_count` readable `f32`s.
		unsafe extern "C" fn trampoline<F: FnMut(&[f32], usize, usize) + Send>(
			userdata: *mut c_void,
			samples: *const f32,
			sample_count: usize,
			channels: usize,
			sample_rate: usize,
		) {
			// SAFETY: the caller guarantees `userdata` points at a live `F`. The
			// enclosing call is synchronous and passes the only pointer to it.
			let on_audio = unsafe { &mut *userdata.cast::<F>() };
			let samples = if samples.is_null() || sample_count == 0 {
				&[]
			} else {
				// SAFETY: `samples` is non-null with a non-zero count here, and prism
				// documents it as `sample_count` interleaved f32s valid for this call.
				unsafe { slice::from_raw_parts(samples, sample_count) }
			};
			on_audio(samples, channels, sample_rate);
		}
		let text = to_cstring(text)?;
		let userdata = (&raw mut on_audio).cast::<c_void>();
		// SAFETY: `raw` is a live backend, `text` is a NUL-terminated CString that
		// outlives the call, and `userdata` points at `on_audio` on this stack
		// frame. prism documents speak_to_memory as always synchronous, so the
		// callback cannot run after this returns.
		check(unsafe { sys::prism_backend_speak_to_memory(self.ptr(), text.as_ptr(), Some(trampoline::<F>), userdata) })
	}

	/// Sends `text` to the connected braille display.
	///
	/// # Errors
	///
	/// Returns an [`Error`] if the backend does not support braille, `text`
	/// contains an interior NUL, or output fails.
	pub fn braille(&self, text: &str) -> Result<()> {
		let text = to_cstring(text)?;
		// SAFETY: `raw` is a live backend, and `text` is a NUL-terminated CString
		// that outlives the call.
		check(unsafe { sys::prism_backend_braille(self.ptr(), text.as_ptr()) })
	}

	/// Speaks and brailles `text` at once, optionally interrupting any output
	/// in progress.
	///
	/// # Errors
	///
	/// Returns an [`Error`] if the backend does not support output, `text`
	/// contains an interior NUL, or output fails.
	pub fn output(&self, text: &str, interrupt: bool) -> Result<()> {
		let text = to_cstring(text)?;
		// SAFETY: `raw` is a live backend, and `text` is a NUL-terminated CString
		// that outlives the call.
		check(unsafe { sys::prism_backend_output(self.ptr(), text.as_ptr(), interrupt) })
	}

	/// Stops any speech in progress.
	///
	/// # Errors
	///
	/// Returns an [`Error`] if the backend does not support stopping or is not
	/// speaking.
	pub fn stop(&self) -> Result<()> {
		// SAFETY: `raw` is a live backend (type invariant).
		check(unsafe { sys::prism_backend_stop(self.ptr()) })
	}

	/// Pauses speech in progress.
	///
	/// # Errors
	///
	/// Returns an [`Error`] if the backend does not support pausing, is not
	/// speaking, or is already paused.
	pub fn pause(&self) -> Result<()> {
		// SAFETY: `raw` is a live backend (type invariant).
		check(unsafe { sys::prism_backend_pause(self.ptr()) })
	}

	/// Resumes paused speech.
	///
	/// # Errors
	///
	/// Returns an [`Error`] if the backend does not support resuming or is not
	/// paused.
	pub fn resume(&self) -> Result<()> {
		// SAFETY: `raw` is a live backend (type invariant).
		check(unsafe { sys::prism_backend_resume(self.ptr()) })
	}

	/// Returns `true` if the backend is currently speaking.
	///
	/// # Errors
	///
	/// Returns an [`Error`] if the backend cannot report its speaking state.
	pub fn is_speaking(&self) -> Result<bool> {
		let mut speaking = false;
		// SAFETY: `raw` is a live backend, and the out-parameter is a live local
		// that prism only writes on success.
		check(unsafe { sys::prism_backend_is_speaking(self.ptr(), &raw mut speaking) })?;
		Ok(speaking)
	}

	/// Sets the speech volume, typically in the range `0.0..=1.0`.
	///
	/// # Errors
	///
	/// Returns an [`Error`] if the backend does not support setting the volume
	/// or the value is out of range.
	pub fn set_volume(&self, volume: f32) -> Result<()> {
		// SAFETY: `raw` is a live backend (type invariant).
		check(unsafe { sys::prism_backend_set_volume(self.ptr(), volume) })
	}

	/// Returns the current speech volume.
	///
	/// # Errors
	///
	/// Returns an [`Error`] if the backend does not support reporting the
	/// volume.
	pub fn volume(&self) -> Result<f32> {
		let mut volume = 0.0;
		// SAFETY: `raw` is a live backend, and the out-parameter is a live local
		// that prism only writes on success.
		check(unsafe { sys::prism_backend_get_volume(self.ptr(), &raw mut volume) })?;
		Ok(volume)
	}

	/// Sets the speech rate, typically in the range `0.0..=1.0`.
	///
	/// # Errors
	///
	/// Returns an [`Error`] if the backend does not support setting the rate
	/// or the value is out of range.
	pub fn set_rate(&self, rate: f32) -> Result<()> {
		// SAFETY: `raw` is a live backend (type invariant).
		check(unsafe { sys::prism_backend_set_rate(self.ptr(), rate) })
	}

	/// Returns the current speech rate.
	///
	/// # Errors
	///
	/// Returns an [`Error`] if the backend does not support reporting the
	/// rate.
	pub fn rate(&self) -> Result<f32> {
		let mut rate = 0.0;
		// SAFETY: `raw` is a live backend, and the out-parameter is a live local
		// that prism only writes on success.
		check(unsafe { sys::prism_backend_get_rate(self.ptr(), &raw mut rate) })?;
		Ok(rate)
	}

	/// Sets the speech pitch, typically in the range `0.0..=1.0`.
	///
	/// # Errors
	///
	/// Returns an [`Error`] if the backend does not support setting the pitch
	/// or the value is out of range.
	pub fn set_pitch(&self, pitch: f32) -> Result<()> {
		// SAFETY: `raw` is a live backend (type invariant).
		check(unsafe { sys::prism_backend_set_pitch(self.ptr(), pitch) })
	}

	/// Returns the current speech pitch.
	///
	/// # Errors
	///
	/// Returns an [`Error`] if the backend does not support reporting the
	/// pitch.
	pub fn pitch(&self) -> Result<f32> {
		let mut pitch = 0.0;
		// SAFETY: `raw` is a live backend, and the out-parameter is a live local
		// that prism only writes on success.
		check(unsafe { sys::prism_backend_get_pitch(self.ptr(), &raw mut pitch) })?;
		Ok(pitch)
	}

	/// Re-enumerates the backend's voices, picking up any newly installed
	/// ones.
	///
	/// # Errors
	///
	/// Returns an [`Error`] if the backend does not support refreshing voices.
	pub fn refresh_voices(&self) -> Result<()> {
		// SAFETY: `raw` is a live backend (type invariant).
		check(unsafe { sys::prism_backend_refresh_voices(self.ptr()) })
	}

	/// Returns the number of voices the backend offers.
	///
	/// # Errors
	///
	/// Returns an [`Error`] if the backend does not support counting voices.
	pub fn voice_count(&self) -> Result<usize> {
		let mut count = 0;
		// SAFETY: `raw` is a live backend, and the out-parameter is a live local
		// that prism only writes on success.
		check(unsafe { sys::prism_backend_count_voices(self.ptr(), &raw mut count) })?;
		Ok(count)
	}

	/// Returns the name of the voice at index `voice_id`.
	///
	/// # Errors
	///
	/// Returns an [`Error`] if the backend does not support voice names or the
	/// voice does not exist.
	pub fn voice_name(&self, voice_id: usize) -> Result<String> {
		let mut name = ptr::null();
		// SAFETY: `raw` is a live backend, and `name` is a live local that prism
		// only points at a string it owns on success.
		check(unsafe { sys::prism_backend_get_voice_name(self.ptr(), voice_id, &raw mut name) })?;
		Ok(copy_cstr(name))
	}

	/// Returns the language tag of the voice at index `voice_id`.
	///
	/// # Errors
	///
	/// Returns an [`Error`] if the backend does not support voice languages or
	/// the voice does not exist.
	pub fn voice_language(&self, voice_id: usize) -> Result<String> {
		let mut language = ptr::null();
		// SAFETY: `raw` is a live backend, and `language` is a live local that
		// prism only points at a string it owns on success.
		check(unsafe { sys::prism_backend_get_voice_language(self.ptr(), voice_id, &raw mut language) })?;
		Ok(copy_cstr(language))
	}

	/// Switches to the voice at index `voice_id`.
	///
	/// # Errors
	///
	/// Returns an [`Error`] if the backend does not support switching voices
	/// or the voice does not exist.
	pub fn set_voice(&self, voice_id: usize) -> Result<()> {
		// SAFETY: `raw` is a live backend (type invariant).
		check(unsafe { sys::prism_backend_set_voice(self.ptr(), voice_id) })
	}

	/// Returns the index of the current voice.
	///
	/// # Errors
	///
	/// Returns an [`Error`] if the backend does not support reporting the
	/// current voice.
	pub fn voice(&self) -> Result<usize> {
		let mut voice_id = 0;
		// SAFETY: `raw` is a live backend, and the out-parameter is a live local
		// that prism only writes on success.
		check(unsafe { sys::prism_backend_get_voice(self.ptr(), &raw mut voice_id) })?;
		Ok(voice_id)
	}

	/// Enumerates every voice the backend offers.
	///
	/// # Errors
	///
	/// Returns an [`Error`] if the backend does not support enumerating
	/// voices.
	pub fn voices(&self) -> Result<Vec<Voice>> {
		let count = self.voice_count()?;
		let mut voices = Vec::with_capacity(count);
		for id in 0..count {
			let name = self.voice_name(id)?;
			let language = self.voice_language(id).ok();
			voices.push(Voice { id, name, language });
		}
		Ok(voices)
	}

	/// Returns the number of audio channels the backend synthesizes.
	///
	/// # Errors
	///
	/// Returns an [`Error`] if the backend does not support reporting its
	/// audio format.
	pub fn channels(&self) -> Result<usize> {
		let mut channels = 0;
		// SAFETY: `raw` is a live backend, and the out-parameter is a live local
		// that prism only writes on success.
		check(unsafe { sys::prism_backend_get_channels(self.ptr(), &raw mut channels) })?;
		Ok(channels)
	}

	/// Returns the sample rate, in hertz, at which the backend synthesizes.
	///
	/// # Errors
	///
	/// Returns an [`Error`] if the backend does not support reporting its
	/// audio format.
	pub fn sample_rate(&self) -> Result<usize> {
		let mut sample_rate = 0;
		// SAFETY: `raw` is a live backend, and the out-parameter is a live local
		// that prism only writes on success.
		check(unsafe { sys::prism_backend_get_sample_rate(self.ptr(), &raw mut sample_rate) })?;
		Ok(sample_rate)
	}

	/// Returns the bit depth at which the backend synthesizes.
	///
	/// # Errors
	///
	/// Returns an [`Error`] if the backend does not support reporting its
	/// audio format.
	pub fn bit_depth(&self) -> Result<usize> {
		let mut bit_depth = 0;
		// SAFETY: `raw` is a live backend, and the out-parameter is a live local
		// that prism only writes on success.
		check(unsafe { sys::prism_backend_get_bit_depth(self.ptr(), &raw mut bit_depth) })?;
		Ok(bit_depth)
	}
}

impl fmt::Debug for Backend<'_> {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		f.debug_struct("Backend").field("name", &self.name()).finish_non_exhaustive()
	}
}

impl Drop for Backend<'_> {
	fn drop(&mut self) {
		// SAFETY: `raw` is a live backend and Drop runs at most once, so this
		// releases our handle exactly once. For acquired backends prism only
		// destroys the instance when the last handle goes.
		unsafe { sys::prism_backend_free(self.ptr()) };
	}
}
