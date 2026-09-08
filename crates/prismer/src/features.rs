use core::ops;

use prism_sys as sys;

#[cfg(doc)]
use crate::Backend;

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
