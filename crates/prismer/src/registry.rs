use core::{ffi::c_int, marker::PhantomData, mem::size_of, ptr::NonNull};
use std::{
	ffi::{CString, c_char, c_void},
	path::Path,
};

use prism_sys as sys;

use crate::{
	BackendId, Features,
	error::{Error, Result, check},
	util::{str_from_ptr, to_cstring},
};

/// The audio destination handed to [`CustomBackend::speak_to_memory`].
///
/// Every [`write`](Self::write) must happen before `speak_to_memory` returns.
/// The lifetime enforces that: the sink cannot outlive the call.
pub struct AudioSink<'a> {
	callback: sys::PrismAudioCallback,
	userdata: *mut c_void,
	_call: PhantomData<&'a ()>,
}

impl AudioSink<'_> {
	/// Delivers a chunk of interleaved samples, normalized to `[-1.0, 1.0]`.
	///
	/// prism clamps a finite out-of-range sample and replaces a non-finite one
	/// with silence, but do not lean on that.
	pub fn write(&mut self, samples: &[f32], channels: usize, sample_rate: usize) {
		let Some(callback) = self.callback else {
			return;
		};
		// SAFETY: prism supplied this callback and userdata for the duration of
		// the speak_to_memory call, which `'a` keeps this sink inside, and
		// `samples` is readable for its own length.
		unsafe { callback(self.userdata, samples.as_ptr(), samples.len(), channels, sample_rate) };
	}
}

impl core::fmt::Debug for AudioSink<'_> {
	fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
		f.debug_struct("AudioSink").finish_non_exhaustive()
	}
}

/// A speech backend written in Rust and registered through a
/// [`RegistryBuilder`].
///
/// Every method defaults to [`Error::NotImplemented`]. Implement only the
/// operations you declare in the [`Features`] passed to
/// [`RegistryBuilder::add_backend`]: prism installs a function pointer for a
/// declared feature and nothing for an undeclared one, so an undeclared
/// operation never reaches your code.
///
/// prism never calls these methods concurrently for one instance, but it may
/// call them from a thread you did not create, which is why the trait requires
/// [`Send`].
///
/// A panic that escapes one of these methods aborts the process, because it
/// would otherwise unwind into C. Catch what you cannot prove will not panic.
pub trait CustomBackend: Send + 'static {
	/// Reports whether the backend is usable right now.
	///
	/// prism may call this before [`initialize`](Self::initialize), and from
	/// its availability poll thread, so it must not assume any setup has run.
	fn is_supported(&mut self) -> bool {
		true
	}

	/// Prepares the backend for use. Defaults to succeeding with no work.
	///
	/// # Errors
	///
	/// Returns an [`Error`] if setup fails.
	fn initialize(&mut self) -> Result<()> {
		Ok(())
	}

	/// Speaks `text`, optionally interrupting speech in progress.
	///
	/// # Errors
	///
	/// Returns an [`Error`] if speaking fails.
	fn speak(&mut self, text: &str, interrupt: bool) -> Result<()> {
		let _ = (text, interrupt);
		Err(Error::NotImplemented)
	}

	/// Synthesizes `text` and delivers the audio to `sink`.
	///
	/// All delivery must finish before this returns.
	///
	/// # Errors
	///
	/// Returns an [`Error`] if synthesis fails.
	fn speak_to_memory(&mut self, text: &str, sink: &mut AudioSink<'_>) -> Result<()> {
		let _ = (text, sink);
		Err(Error::NotImplemented)
	}

	/// Sends `text` to a braille display.
	///
	/// # Errors
	///
	/// Returns an [`Error`] if output fails.
	fn braille(&mut self, text: &str) -> Result<()> {
		let _ = text;
		Err(Error::NotImplemented)
	}

	/// Sends `text` through both speech and braille.
	///
	/// # Errors
	///
	/// Returns an [`Error`] if output fails.
	fn output(&mut self, text: &str, interrupt: bool) -> Result<()> {
		let _ = (text, interrupt);
		Err(Error::NotImplemented)
	}

	/// Stops speech in progress.
	///
	/// # Errors
	///
	/// Returns an [`Error`] if stopping fails.
	fn stop(&mut self) -> Result<()> {
		Err(Error::NotImplemented)
	}

	/// Pauses speech. prism will not call this on an instance it already
	/// considers paused.
	///
	/// # Errors
	///
	/// Returns an [`Error`] if pausing fails.
	fn pause(&mut self) -> Result<()> {
		Err(Error::NotImplemented)
	}

	/// Resumes paused speech. prism will not call this on an instance it does
	/// not consider paused.
	///
	/// # Errors
	///
	/// Returns an [`Error`] if resuming fails.
	fn resume(&mut self) -> Result<()> {
		Err(Error::NotImplemented)
	}

	/// Reports whether speech is in progress.
	///
	/// # Errors
	///
	/// Returns an [`Error`] if the state cannot be determined.
	fn is_speaking(&mut self) -> Result<bool> {
		Err(Error::NotImplemented)
	}

	/// Sets the volume, which prism has already validated as finite and within
	/// `[0.0, 1.0]`.
	///
	/// # Errors
	///
	/// Returns an [`Error`] if the volume cannot be set.
	fn set_volume(&mut self, volume: f32) -> Result<()> {
		let _ = volume;
		Err(Error::NotImplemented)
	}

	/// Returns the current volume.
	///
	/// # Errors
	///
	/// Returns an [`Error`] if the volume cannot be read.
	fn volume(&mut self) -> Result<f32> {
		Err(Error::NotImplemented)
	}

	/// Sets the speech rate, already validated as finite and within
	/// `[0.0, 1.0]`.
	///
	/// # Errors
	///
	/// Returns an [`Error`] if the rate cannot be set.
	fn set_rate(&mut self, rate: f32) -> Result<()> {
		let _ = rate;
		Err(Error::NotImplemented)
	}

	/// Returns the current speech rate.
	///
	/// # Errors
	///
	/// Returns an [`Error`] if the rate cannot be read.
	fn rate(&mut self) -> Result<f32> {
		Err(Error::NotImplemented)
	}

	/// Sets the pitch, already validated as finite and within `[0.0, 1.0]`.
	///
	/// # Errors
	///
	/// Returns an [`Error`] if the pitch cannot be set.
	fn set_pitch(&mut self, pitch: f32) -> Result<()> {
		let _ = pitch;
		Err(Error::NotImplemented)
	}

	/// Returns the current pitch.
	///
	/// # Errors
	///
	/// Returns an [`Error`] if the pitch cannot be read.
	fn pitch(&mut self) -> Result<f32> {
		Err(Error::NotImplemented)
	}

	/// Rebuilds the voice list from the underlying engine.
	///
	/// # Errors
	///
	/// Returns an [`Error`] if the list cannot be refreshed.
	fn refresh_voices(&mut self) -> Result<()> {
		Err(Error::NotImplemented)
	}

	/// Returns how many voices the backend offers.
	///
	/// # Errors
	///
	/// Returns an [`Error`] if the count cannot be determined.
	fn voice_count(&mut self) -> Result<usize> {
		Err(Error::NotImplemented)
	}

	/// Returns the name of the voice at `voice_id`.
	///
	/// # Errors
	///
	/// Returns an [`Error`] if the voice does not exist.
	fn voice_name(&mut self, voice_id: usize) -> Result<String> {
		let _ = voice_id;
		Err(Error::NotImplemented)
	}

	/// Returns the language tag of the voice at `voice_id`.
	///
	/// # Errors
	///
	/// Returns an [`Error`] if the voice does not exist.
	fn voice_language(&mut self, voice_id: usize) -> Result<String> {
		let _ = voice_id;
		Err(Error::NotImplemented)
	}

	/// Switches to the voice at `voice_id`.
	///
	/// # Errors
	///
	/// Returns an [`Error`] if the voice cannot be selected.
	fn set_voice(&mut self, voice_id: usize) -> Result<()> {
		let _ = voice_id;
		Err(Error::NotImplemented)
	}

	/// Returns the index of the current voice.
	///
	/// # Errors
	///
	/// Returns an [`Error`] if the voice cannot be read.
	fn voice(&mut self) -> Result<usize> {
		Err(Error::NotImplemented)
	}

	/// Returns the channel count of the audio the backend produces.
	///
	/// # Errors
	///
	/// Returns an [`Error`] if the format cannot be read.
	fn channels(&mut self) -> Result<usize> {
		Err(Error::NotImplemented)
	}

	/// Returns the sample rate of the audio the backend produces.
	///
	/// # Errors
	///
	/// Returns an [`Error`] if the format cannot be read.
	fn sample_rate(&mut self) -> Result<usize> {
		Err(Error::NotImplemented)
	}

	/// Returns the bit depth of the audio the backend produces.
	///
	/// # Errors
	///
	/// Returns an [`Error`] if the format cannot be read.
	fn bit_depth(&mut self) -> Result<usize> {
		Err(Error::NotImplemented)
	}
}

/// Per-instance state prism holds on our behalf.
///
/// `voice_string` backs the one pointer prism expects to stay valid until the
/// next voice name or language query on the same instance.
struct Instance<B> {
	backend: B,
	voice_string: Option<CString>,
}

type Factory<B> = Box<dyn FnMut() -> Option<B> + Send>;

const fn to_code(result: Result<()>) -> sys::PrismError {
	match result {
		Ok(()) => sys::PRISM_OK,
		Err(error) => error.to_raw(),
	}
}

/// # Safety
///
/// `instance` must be a pointer produced by [`create`] for the same `B`, and
/// still alive.
unsafe fn with<B: CustomBackend, T>(instance: *mut c_void, f: impl FnOnce(&mut Instance<B>) -> T) -> T {
	// SAFETY: the caller guarantees `instance` is a live Instance<B>, and prism
	// never invokes vtable functions concurrently for one instance, so this is
	// the only reference while it is held.
	let instance = unsafe { &mut *instance.cast::<Instance<B>>() };
	f(instance)
}

/// # Safety
///
/// `userdata` must be the [`Factory<B>`] registered alongside this vtable.
unsafe extern "C" fn create<B: CustomBackend>(userdata: *mut c_void) -> *mut c_void {
	// SAFETY: the caller guarantees `userdata` points at the live factory, and
	// prism calls this on one thread at a time for a given registration.
	let factory = unsafe { &mut *userdata.cast::<Factory<B>>() };
	factory().map_or(core::ptr::null_mut(), |backend| {
		Box::into_raw(Box::new(Instance { backend, voice_string: None })).cast::<c_void>()
	})
}

/// # Safety
///
/// `instance` must be a pointer produced by [`create`] for the same `B`, not
/// yet destroyed.
unsafe extern "C" fn destroy<B: CustomBackend>(instance: *mut c_void) {
	// SAFETY: the caller guarantees this pointer came from create::<B> and that
	// prism calls destroy exactly once per instance.
	drop(unsafe { Box::from_raw(instance.cast::<Instance<B>>()) });
}

/// # Safety
///
/// `userdata` must be the [`Factory<B>`] passed to `add_backend`, not yet
/// freed.
unsafe extern "C" fn free_factory<B: CustomBackend>(userdata: *mut c_void) {
	// SAFETY: prism calls this exactly once per add_backend call, after every
	// destroy arising from the registration.
	drop(unsafe { Box::from_raw(userdata.cast::<Factory<B>>()) });
}

/// # Safety
///
/// Every function below takes an `instance` produced by [`create`] for the same
/// `B`. String arguments are non-null, NUL-terminated and valid for the call,
/// and out-parameters are writable. prism guarantees all of this before it
/// consults a vtable.
unsafe extern "C" fn is_supported<B: CustomBackend>(instance: *mut c_void) -> bool {
	// SAFETY: see the module's vtable contract above.
	unsafe { with::<B, _>(instance, |i| i.backend.is_supported()) }
}

/// # Safety
///
/// See [`is_supported`].
unsafe extern "C" fn initialize<B: CustomBackend>(instance: *mut c_void) -> sys::PrismError {
	// SAFETY: see the module's vtable contract above.
	unsafe { with::<B, _>(instance, |i| to_code(i.backend.initialize())) }
}

/// # Safety
///
/// See [`is_supported`].
unsafe extern "C" fn speak<B: CustomBackend>(
	instance: *mut c_void,
	text: *const c_char,
	interrupt: bool,
) -> sys::PrismError {
	// SAFETY: see the module's vtable contract above; prism validates `text` as
	// non-null, NUL-terminated UTF-8 before calling.
	unsafe { with::<B, _>(instance, |i| to_code(i.backend.speak(str_from_ptr(text), interrupt))) }
}

/// # Safety
///
/// See [`is_supported`].
unsafe extern "C" fn speak_to_memory<B: CustomBackend>(
	instance: *mut c_void,
	text: *const c_char,
	callback: sys::PrismAudioCallback,
	callback_userdata: *mut c_void,
) -> sys::PrismError {
	// SAFETY: see the module's vtable contract above. The sink borrows for this
	// call only, which is what prism requires of custom backends.
	unsafe {
		with::<B, _>(instance, |i| {
			let mut sink = AudioSink { callback, userdata: callback_userdata, _call: PhantomData };
			to_code(i.backend.speak_to_memory(str_from_ptr(text), &mut sink))
		})
	}
}

/// # Safety
///
/// See [`is_supported`].
unsafe extern "C" fn braille<B: CustomBackend>(instance: *mut c_void, text: *const c_char) -> sys::PrismError {
	// SAFETY: see the module's vtable contract above.
	unsafe { with::<B, _>(instance, |i| to_code(i.backend.braille(str_from_ptr(text)))) }
}

/// # Safety
///
/// See [`is_supported`].
unsafe extern "C" fn output<B: CustomBackend>(
	instance: *mut c_void,
	text: *const c_char,
	interrupt: bool,
) -> sys::PrismError {
	// SAFETY: see the module's vtable contract above.
	unsafe { with::<B, _>(instance, |i| to_code(i.backend.output(str_from_ptr(text), interrupt))) }
}

/// # Safety
///
/// See [`is_supported`].
unsafe extern "C" fn stop<B: CustomBackend>(instance: *mut c_void) -> sys::PrismError {
	// SAFETY: see the module's vtable contract above.
	unsafe { with::<B, _>(instance, |i| to_code(i.backend.stop())) }
}

/// # Safety
///
/// See [`is_supported`].
unsafe extern "C" fn pause<B: CustomBackend>(instance: *mut c_void) -> sys::PrismError {
	// SAFETY: see the module's vtable contract above.
	unsafe { with::<B, _>(instance, |i| to_code(i.backend.pause())) }
}

/// # Safety
///
/// See [`is_supported`].
unsafe extern "C" fn resume<B: CustomBackend>(instance: *mut c_void) -> sys::PrismError {
	// SAFETY: see the module's vtable contract above.
	unsafe { with::<B, _>(instance, |i| to_code(i.backend.resume())) }
}

/// # Safety
///
/// See [`is_supported`]. `out` must be writable.
unsafe extern "C" fn is_speaking<B: CustomBackend>(instance: *mut c_void, out: *mut bool) -> sys::PrismError {
	// SAFETY: see the module's vtable contract above; `out` is writable and is
	// only written on success.
	unsafe {
		with::<B, _>(instance, |i| match i.backend.is_speaking() {
			Ok(value) => {
				out.write(value);
				sys::PRISM_OK
			}
			Err(error) => error.to_raw(),
		})
	}
}

/// # Safety
///
/// See [`is_supported`].
unsafe extern "C" fn set_volume<B: CustomBackend>(instance: *mut c_void, volume: f32) -> sys::PrismError {
	// SAFETY: see the module's vtable contract above.
	unsafe { with::<B, _>(instance, |i| to_code(i.backend.set_volume(volume))) }
}

/// # Safety
///
/// See [`is_supported`]. `out` must be writable.
unsafe extern "C" fn get_volume<B: CustomBackend>(instance: *mut c_void, out: *mut f32) -> sys::PrismError {
	// SAFETY: see the module's vtable contract above.
	unsafe {
		with::<B, _>(instance, |i| match i.backend.volume() {
			Ok(value) => {
				out.write(value);
				sys::PRISM_OK
			}
			Err(error) => error.to_raw(),
		})
	}
}

/// # Safety
///
/// See [`is_supported`].
unsafe extern "C" fn set_rate<B: CustomBackend>(instance: *mut c_void, rate: f32) -> sys::PrismError {
	// SAFETY: see the module's vtable contract above.
	unsafe { with::<B, _>(instance, |i| to_code(i.backend.set_rate(rate))) }
}

/// # Safety
///
/// See [`is_supported`]. `out` must be writable.
unsafe extern "C" fn get_rate<B: CustomBackend>(instance: *mut c_void, out: *mut f32) -> sys::PrismError {
	// SAFETY: see the module's vtable contract above.
	unsafe {
		with::<B, _>(instance, |i| match i.backend.rate() {
			Ok(value) => {
				out.write(value);
				sys::PRISM_OK
			}
			Err(error) => error.to_raw(),
		})
	}
}

/// # Safety
///
/// See [`is_supported`].
unsafe extern "C" fn set_pitch<B: CustomBackend>(instance: *mut c_void, pitch: f32) -> sys::PrismError {
	// SAFETY: see the module's vtable contract above.
	unsafe { with::<B, _>(instance, |i| to_code(i.backend.set_pitch(pitch))) }
}

/// # Safety
///
/// See [`is_supported`]. `out` must be writable.
unsafe extern "C" fn get_pitch<B: CustomBackend>(instance: *mut c_void, out: *mut f32) -> sys::PrismError {
	// SAFETY: see the module's vtable contract above.
	unsafe {
		with::<B, _>(instance, |i| match i.backend.pitch() {
			Ok(value) => {
				out.write(value);
				sys::PRISM_OK
			}
			Err(error) => error.to_raw(),
		})
	}
}

/// # Safety
///
/// See [`is_supported`].
unsafe extern "C" fn refresh_voices<B: CustomBackend>(instance: *mut c_void) -> sys::PrismError {
	// SAFETY: see the module's vtable contract above.
	unsafe { with::<B, _>(instance, |i| to_code(i.backend.refresh_voices())) }
}

/// # Safety
///
/// See [`is_supported`]. `out` must be writable.
unsafe extern "C" fn count_voices<B: CustomBackend>(instance: *mut c_void, out: *mut usize) -> sys::PrismError {
	// SAFETY: see the module's vtable contract above.
	unsafe {
		with::<B, _>(instance, |i| match i.backend.voice_count() {
			Ok(value) => {
				out.write(value);
				sys::PRISM_OK
			}
			Err(error) => error.to_raw(),
		})
	}
}

/// # Safety
///
/// See [`is_supported`]. `out` must be writable.
unsafe extern "C" fn get_voice_name<B: CustomBackend>(
	instance: *mut c_void,
	voice_id: usize,
	out: *mut *const c_char,
) -> sys::PrismError {
	// SAFETY: see the module's vtable contract above. The CString is parked in
	// the instance, so the pointer stays valid until the next name or language
	// query on this instance, which is exactly what prism promises callers.
	unsafe {
		with::<B, _>(instance, |i| match i.backend.voice_name(voice_id) {
			Ok(name) => match CString::new(name) {
				Ok(name) => {
					out.write(i.voice_string.insert(name).as_ptr());
					sys::PRISM_OK
				}
				Err(_) => sys::PRISM_ERROR_INVALID_PARAM,
			},
			Err(error) => error.to_raw(),
		})
	}
}

/// # Safety
///
/// See [`is_supported`]. `out` must be writable.
unsafe extern "C" fn get_voice_language<B: CustomBackend>(
	instance: *mut c_void,
	voice_id: usize,
	out: *mut *const c_char,
) -> sys::PrismError {
	// SAFETY: see the module's vtable contract above, and the note on
	// get_voice_name about how long the returned pointer stays valid.
	unsafe {
		with::<B, _>(instance, |i| match i.backend.voice_language(voice_id) {
			Ok(language) => match CString::new(language) {
				Ok(language) => {
					out.write(i.voice_string.insert(language).as_ptr());
					sys::PRISM_OK
				}
				Err(_) => sys::PRISM_ERROR_INVALID_PARAM,
			},
			Err(error) => error.to_raw(),
		})
	}
}

/// # Safety
///
/// See [`is_supported`].
unsafe extern "C" fn set_voice<B: CustomBackend>(instance: *mut c_void, voice_id: usize) -> sys::PrismError {
	// SAFETY: see the module's vtable contract above.
	unsafe { with::<B, _>(instance, |i| to_code(i.backend.set_voice(voice_id))) }
}

/// # Safety
///
/// See [`is_supported`]. `out` must be writable.
unsafe extern "C" fn get_voice<B: CustomBackend>(instance: *mut c_void, out: *mut usize) -> sys::PrismError {
	// SAFETY: see the module's vtable contract above.
	unsafe {
		with::<B, _>(instance, |i| match i.backend.voice() {
			Ok(value) => {
				out.write(value);
				sys::PRISM_OK
			}
			Err(error) => error.to_raw(),
		})
	}
}

/// # Safety
///
/// See [`is_supported`]. `out` must be writable.
unsafe extern "C" fn get_channels<B: CustomBackend>(instance: *mut c_void, out: *mut usize) -> sys::PrismError {
	// SAFETY: see the module's vtable contract above.
	unsafe {
		with::<B, _>(instance, |i| match i.backend.channels() {
			Ok(value) => {
				out.write(value);
				sys::PRISM_OK
			}
			Err(error) => error.to_raw(),
		})
	}
}

/// # Safety
///
/// See [`is_supported`]. `out` must be writable.
unsafe extern "C" fn get_sample_rate<B: CustomBackend>(instance: *mut c_void, out: *mut usize) -> sys::PrismError {
	// SAFETY: see the module's vtable contract above.
	unsafe {
		with::<B, _>(instance, |i| match i.backend.sample_rate() {
			Ok(value) => {
				out.write(value);
				sys::PRISM_OK
			}
			Err(error) => error.to_raw(),
		})
	}
}

/// # Safety
///
/// See [`is_supported`]. `out` must be writable.
unsafe extern "C" fn get_bit_depth<B: CustomBackend>(instance: *mut c_void, out: *mut usize) -> sys::PrismError {
	// SAFETY: see the module's vtable contract above.
	unsafe {
		with::<B, _>(instance, |i| match i.backend.bit_depth() {
			Ok(value) => {
				out.write(value);
				sys::PRISM_OK
			}
			Err(error) => error.to_raw(),
		})
	}
}

/// Builds the vtable for `B`, installing a pointer for each declared operation
/// and leaving the rest null.
///
/// prism rejects a registration whose declared features and non-null members
/// disagree, so the two are derived from one `features` value here rather than
/// stated twice.
fn vtable_for<B: CustomBackend>(features: Features) -> sys::PrismBackendVTable {
	let mut vtable = sys::PrismBackendVTable {
		size: size_of::<sys::PrismBackendVTable>(),
		create: Some(create::<B>),
		destroy: Some(destroy::<B>),
		is_supported: Some(is_supported::<B>),
		initialize: Some(initialize::<B>),
		speak: None,
		speak_to_memory: None,
		braille: None,
		output: None,
		stop: None,
		pause: None,
		resume: None,
		is_speaking: None,
		set_volume: None,
		get_volume: None,
		set_rate: None,
		get_rate: None,
		set_pitch: None,
		get_pitch: None,
		refresh_voices: None,
		count_voices: None,
		get_voice_name: None,
		get_voice_language: None,
		set_voice: None,
		get_voice: None,
		get_channels: None,
		get_sample_rate: None,
		get_bit_depth: None,
	};
	if features.contains(Features::SPEAK) {
		vtable.speak = Some(speak::<B>);
	}
	if features.contains(Features::SPEAK_TO_MEMORY) {
		vtable.speak_to_memory = Some(speak_to_memory::<B>);
	}
	if features.contains(Features::BRAILLE) {
		vtable.braille = Some(braille::<B>);
	}
	if features.contains(Features::OUTPUT) {
		vtable.output = Some(output::<B>);
	}
	if features.contains(Features::STOP) {
		vtable.stop = Some(stop::<B>);
	}
	if features.contains(Features::PAUSE) {
		vtable.pause = Some(pause::<B>);
	}
	if features.contains(Features::RESUME) {
		vtable.resume = Some(resume::<B>);
	}
	if features.contains(Features::IS_SPEAKING) {
		vtable.is_speaking = Some(is_speaking::<B>);
	}
	if features.contains(Features::SET_VOLUME) {
		vtable.set_volume = Some(set_volume::<B>);
	}
	if features.contains(Features::GET_VOLUME) {
		vtable.get_volume = Some(get_volume::<B>);
	}
	if features.contains(Features::SET_RATE) {
		vtable.set_rate = Some(set_rate::<B>);
	}
	if features.contains(Features::GET_RATE) {
		vtable.get_rate = Some(get_rate::<B>);
	}
	if features.contains(Features::SET_PITCH) {
		vtable.set_pitch = Some(set_pitch::<B>);
	}
	if features.contains(Features::GET_PITCH) {
		vtable.get_pitch = Some(get_pitch::<B>);
	}
	if features.contains(Features::REFRESH_VOICES) {
		vtable.refresh_voices = Some(refresh_voices::<B>);
	}
	if features.contains(Features::COUNT_VOICES) {
		vtable.count_voices = Some(count_voices::<B>);
	}
	if features.contains(Features::GET_VOICE_NAME) {
		vtable.get_voice_name = Some(get_voice_name::<B>);
	}
	if features.contains(Features::GET_VOICE_LANGUAGE) {
		vtable.get_voice_language = Some(get_voice_language::<B>);
	}
	if features.contains(Features::SET_VOICE) {
		vtable.set_voice = Some(set_voice::<B>);
	}
	if features.contains(Features::GET_VOICE) {
		vtable.get_voice = Some(get_voice::<B>);
	}
	if features.contains(Features::GET_CHANNELS) {
		vtable.get_channels = Some(get_channels::<B>);
	}
	if features.contains(Features::GET_SAMPLE_RATE) {
		vtable.get_sample_rate = Some(get_sample_rate::<B>);
	}
	if features.contains(Features::GET_BIT_DEPTH) {
		vtable.get_bit_depth = Some(get_bit_depth::<B>);
	}
	vtable
}

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

impl core::fmt::Debug for Registry {
	fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
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

impl core::fmt::Debug for RegistryBuilder {
	fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
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
