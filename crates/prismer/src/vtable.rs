use core::mem::size_of;
use std::ffi::{CString, c_char, c_void};

use prism_sys as sys;

use crate::{AudioSink, CustomBackend, Features, error::Result, util::str_from_ptr};

/// Per-instance state prism holds on our behalf.
///
/// `voice_string` backs the one pointer prism expects to stay valid until the
/// next voice name or language query on the same instance.
struct Instance<B> {
	backend: B,
	voice_string: Option<CString>,
}

pub(crate) type Factory<B> = Box<dyn FnMut() -> Option<B> + Send>;

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
pub(crate) unsafe extern "C" fn free_factory<B: CustomBackend>(userdata: *mut c_void) {
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
			let mut sink = AudioSink::new(callback, callback_userdata);
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
pub(crate) fn vtable_for<B: CustomBackend>(features: Features) -> sys::PrismBackendVTable {
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
