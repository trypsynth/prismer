use prism_sys as sys;

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
