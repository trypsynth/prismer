use core::{
	marker::PhantomData,
	ops,
	ptr::{self, NonNull},
	slice,
};
use std::ffi::{CStr, CString, c_char, c_void};

use prism_sys as sys;

use crate::{
	Prism,
	error::{Error, Result, check},
};

/// A bitset of capabilities a [`Backend`] advertises.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct Features(pub u64);

impl Features {
	/// The backend is usable on the current system right now.
	pub const IS_SUPPORTED_AT_RUNTIME: Self = Self(sys::PRISM_BACKEND_IS_SUPPORTED_AT_RUNTIME);
	/// Supports [`Backend::speak`].
	pub const SPEAK: Self = Self(sys::PRISM_BACKEND_SUPPORTS_SPEAK);
	/// Supports [`Backend::speak_to_memory`].
	pub const SPEAK_TO_MEMORY: Self = Self(sys::PRISM_BACKEND_SUPPORTS_SPEAK_TO_MEMORY);
	/// Supports [`Backend::braille`].
	pub const BRAILLE: Self = Self(sys::PRISM_BACKEND_SUPPORTS_BRAILLE);
	/// Supports [`Backend::output`].
	pub const OUTPUT: Self = Self(sys::PRISM_BACKEND_SUPPORTS_OUTPUT);
	/// Supports [`Backend::is_speaking`].
	pub const IS_SPEAKING: Self = Self(sys::PRISM_BACKEND_SUPPORTS_IS_SPEAKING);
	/// Supports [`Backend::stop`].
	pub const STOP: Self = Self(sys::PRISM_BACKEND_SUPPORTS_STOP);
	/// Supports [`Backend::pause`].
	pub const PAUSE: Self = Self(sys::PRISM_BACKEND_SUPPORTS_PAUSE);
	/// Supports [`Backend::resume`].
	pub const RESUME: Self = Self(sys::PRISM_BACKEND_SUPPORTS_RESUME);
	/// Supports [`Backend::set_volume`].
	pub const SET_VOLUME: Self = Self(sys::PRISM_BACKEND_SUPPORTS_SET_VOLUME);
	/// Supports [`Backend::volume`].
	pub const GET_VOLUME: Self = Self(sys::PRISM_BACKEND_SUPPORTS_GET_VOLUME);
	/// Supports [`Backend::set_rate`].
	pub const SET_RATE: Self = Self(sys::PRISM_BACKEND_SUPPORTS_SET_RATE);
	/// Supports [`Backend::rate`].
	pub const GET_RATE: Self = Self(sys::PRISM_BACKEND_SUPPORTS_GET_RATE);
	/// Supports [`Backend::set_pitch`].
	pub const SET_PITCH: Self = Self(sys::PRISM_BACKEND_SUPPORTS_SET_PITCH);
	/// Supports [`Backend::pitch`].
	pub const GET_PITCH: Self = Self(sys::PRISM_BACKEND_SUPPORTS_GET_PITCH);
	/// Supports [`Backend::refresh_voices`].
	pub const REFRESH_VOICES: Self = Self(sys::PRISM_BACKEND_SUPPORTS_REFRESH_VOICES);
	/// Supports [`Backend::voice_count`].
	pub const COUNT_VOICES: Self = Self(sys::PRISM_BACKEND_SUPPORTS_COUNT_VOICES);
	/// Supports [`Backend::voice_name`].
	pub const GET_VOICE_NAME: Self = Self(sys::PRISM_BACKEND_SUPPORTS_GET_VOICE_NAME);
	/// Supports [`Backend::voice_language`].
	pub const GET_VOICE_LANGUAGE: Self = Self(sys::PRISM_BACKEND_SUPPORTS_GET_VOICE_LANGUAGE);
	/// Supports [`Backend::voice`].
	pub const GET_VOICE: Self = Self(sys::PRISM_BACKEND_SUPPORTS_GET_VOICE);
	/// Supports [`Backend::set_voice`].
	pub const SET_VOICE: Self = Self(sys::PRISM_BACKEND_SUPPORTS_SET_VOICE);
	/// Supports [`Backend::channels`].
	pub const GET_CHANNELS: Self = Self(sys::PRISM_BACKEND_SUPPORTS_GET_CHANNELS);
	/// Supports [`Backend::sample_rate`].
	pub const GET_SAMPLE_RATE: Self = Self(sys::PRISM_BACKEND_SUPPORTS_GET_SAMPLE_RATE);
	/// Supports [`Backend::bit_depth`].
	pub const GET_BIT_DEPTH: Self = Self(sys::PRISM_BACKEND_SUPPORTS_GET_BIT_DEPTH);
	/// The backend trims leading and trailing silence when speaking.
	pub const SILENCE_TRIMMING_ON_SPEAK: Self = Self(sys::PRISM_BACKEND_PERFORMS_SILENCE_TRIMMING_ON_SPEAK);
	/// The backend trims leading and trailing silence when speaking to memory.
	pub const SILENCE_TRIMMING_ON_SPEAK_TO_MEMORY: Self =
		Self(sys::PRISM_BACKEND_PERFORMS_SILENCE_TRIMMING_ON_SPEAK_TO_MEMORY);
	/// [`Backend::speak`] accepts SSML markup.
	pub const SPEAK_SSML: Self = Self(sys::PRISM_BACKEND_SUPPORTS_SPEAK_SSML);
	/// [`Backend::speak_to_memory`] accepts SSML markup.
	pub const SPEAK_TO_MEMORY_SSML: Self = Self(sys::PRISM_BACKEND_SUPPORTS_SPEAK_TO_MEMORY_SSML);

	/// Returns `true` if every feature in `other` is present in `self`.
	#[must_use]
	pub const fn contains(self, other: Self) -> bool {
		self.0 & other.0 == other.0
	}
}

impl ops::BitOr for Features {
	type Output = Self;
	fn bitor(self, rhs: Self) -> Self {
		Self(self.0 | rhs.0)
	}
}

impl ops::BitAnd for Features {
	type Output = Self;
	fn bitand(self, rhs: Self) -> Self {
		Self(self.0 & rhs.0)
	}
}

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
		copy_cstr(unsafe { sys::prism_backend_name(self.ptr()) })
	}

	/// Returns the set of features this backend advertises.
	#[must_use]
	pub fn features(&self) -> Features {
		Features(unsafe { sys::prism_backend_get_features(self.ptr()) })
	}

	/// Returns `true` if the backend advertises every feature in `features`.
	#[must_use]
	pub fn supports(&self, features: Features) -> bool {
		self.features().contains(features)
	}

	/// Initializes the backend, if it was created uninitialized.
	///
	/// # Errors
	///
	/// Returns an [`Error`] if the backend fails to initialize.
	pub fn initialize(&self) -> Result<()> {
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
		check(unsafe { sys::prism_backend_speak(self.ptr(), text.as_ptr(), interrupt) })
	}

	/// Synthesizes `text` to memory, invoking `on_audio` with chunks of
	/// interleaved `f32` samples along with their channel count and sample
	/// rate.
	///
	/// # Errors
	///
	/// Returns an [`Error`] if the backend does not support synthesis to
	/// memory, `text` contains an interior NUL, or synthesis fails.
	pub fn speak_to_memory<F>(&self, text: &str, mut on_audio: F) -> Result<()>
	where
		F: FnMut(&[f32], usize, usize),
	{
		unsafe extern "C" fn trampoline<F: FnMut(&[f32], usize, usize)>(
			userdata: *mut c_void,
			samples: *const f32,
			sample_count: usize,
			channels: usize,
			sample_rate: usize,
		) {
			let on_audio = unsafe { &mut *userdata.cast::<F>() };
			let samples = if samples.is_null() || sample_count == 0 {
				&[]
			} else {
				unsafe { slice::from_raw_parts(samples, sample_count) }
			};
			on_audio(samples, channels, sample_rate);
		}
		let text = to_cstring(text)?;
		let userdata = (&raw mut on_audio).cast::<c_void>();
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
		check(unsafe { sys::prism_backend_output(self.ptr(), text.as_ptr(), interrupt) })
	}

	/// Stops any speech in progress.
	///
	/// # Errors
	///
	/// Returns an [`Error`] if the backend does not support stopping or is not
	/// speaking.
	pub fn stop(&self) -> Result<()> {
		check(unsafe { sys::prism_backend_stop(self.ptr()) })
	}

	/// Pauses speech in progress.
	///
	/// # Errors
	///
	/// Returns an [`Error`] if the backend does not support pausing, is not
	/// speaking, or is already paused.
	pub fn pause(&self) -> Result<()> {
		check(unsafe { sys::prism_backend_pause(self.ptr()) })
	}

	/// Resumes paused speech.
	///
	/// # Errors
	///
	/// Returns an [`Error`] if the backend does not support resuming or is not
	/// paused.
	pub fn resume(&self) -> Result<()> {
		check(unsafe { sys::prism_backend_resume(self.ptr()) })
	}

	/// Returns `true` if the backend is currently speaking.
	///
	/// # Errors
	///
	/// Returns an [`Error`] if the backend cannot report its speaking state.
	pub fn is_speaking(&self) -> Result<bool> {
		let mut speaking = false;
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
		check(unsafe { sys::prism_backend_refresh_voices(self.ptr()) })
	}

	/// Returns the number of voices the backend offers.
	///
	/// # Errors
	///
	/// Returns an [`Error`] if the backend does not support counting voices.
	pub fn voice_count(&self) -> Result<usize> {
		let mut count = 0;
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
		check(unsafe { sys::prism_backend_get_bit_depth(self.ptr(), &raw mut bit_depth) })?;
		Ok(bit_depth)
	}
}

impl Drop for Backend<'_> {
	fn drop(&mut self) {
		unsafe { sys::prism_backend_free(self.ptr()) };
	}
}

pub fn to_cstring(text: &str) -> Result<CString> {
	CString::new(text).map_err(|_| Error::InvalidParam)
}

pub fn copy_cstr(ptr: *const c_char) -> String {
	if ptr.is_null() {
		return String::new();
	}

	unsafe { CStr::from_ptr(ptr) }.to_string_lossy().into_owned()
}
