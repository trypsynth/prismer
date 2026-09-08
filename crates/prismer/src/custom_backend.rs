use core::{fmt, marker::PhantomData};
use std::ffi::c_void;

use prism_sys as sys;

use crate::error::{Error, Result};

#[cfg(doc)]
use crate::{Features, RegistryBuilder};

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
	pub(crate) const fn new(callback: sys::PrismAudioCallback, userdata: *mut c_void) -> Self {
		Self { callback, userdata, _call: PhantomData }
	}

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

impl fmt::Debug for AudioSink<'_> {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
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
