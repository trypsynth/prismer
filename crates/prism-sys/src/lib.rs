//! Raw FFI bindings to prism (<https://github.com/ethindp/prism>), a
//! platform-agnostic speech and screen reader output library.
#![no_std]
#![allow(non_camel_case_types, clippy::missing_safety_doc)]

use core::ffi::{c_char, c_int, c_void};

#[repr(C)]
pub struct PrismContext {
	_opaque: [u8; 0],
}

#[repr(C)]
pub struct PrismBackend {
	_opaque: [u8; 0],
}

#[repr(C)]
pub struct PrismRegistry {
	_opaque: [u8; 0],
}

#[repr(C)]
pub struct PrismRegistryBuilder {
	_opaque: [u8; 0],
}

pub type PrismBackendId = u64;
pub type PrismError = c_int;
pub type PrismLogLevel = c_int;

pub const PRISM_OK: PrismError = 0;
pub const PRISM_ERROR_NOT_INITIALIZED: PrismError = 1;
pub const PRISM_ERROR_INVALID_PARAM: PrismError = 2;
pub const PRISM_ERROR_NOT_IMPLEMENTED: PrismError = 3;
pub const PRISM_ERROR_NO_VOICES: PrismError = 4;
pub const PRISM_ERROR_VOICE_NOT_FOUND: PrismError = 5;
pub const PRISM_ERROR_SPEAK_FAILURE: PrismError = 6;
pub const PRISM_ERROR_MEMORY_FAILURE: PrismError = 7;
pub const PRISM_ERROR_RANGE_OUT_OF_BOUNDS: PrismError = 8;
pub const PRISM_ERROR_INTERNAL: PrismError = 9;
pub const PRISM_ERROR_NOT_SPEAKING: PrismError = 10;
pub const PRISM_ERROR_NOT_PAUSED: PrismError = 11;
pub const PRISM_ERROR_ALREADY_PAUSED: PrismError = 12;
pub const PRISM_ERROR_INVALID_UTF8: PrismError = 13;
pub const PRISM_ERROR_INVALID_OPERATION: PrismError = 14;
pub const PRISM_ERROR_ALREADY_INITIALIZED: PrismError = 15;
pub const PRISM_ERROR_BACKEND_NOT_AVAILABLE: PrismError = 16;
pub const PRISM_ERROR_UNKNOWN: PrismError = 17;
pub const PRISM_ERROR_INVALID_AUDIO_FORMAT: PrismError = 18;
pub const PRISM_ERROR_INTERNAL_BACKEND_LIMIT_EXCEEDED: PrismError = 19;
pub const PRISM_ERROR_BACKEND_ENTERED_UNDEFINED_STATE: PrismError = 20;
pub const PRISM_ERROR_LIBRARY_LOAD_FAILED: PrismError = 21;
pub const PRISM_ERROR_LIBRARY_INVALID: PrismError = 22;
pub const PRISM_ERROR_INCOMPATIBLE_ABI: PrismError = 23;
pub const PRISM_ERROR_COUNT: PrismError = 24;

pub const PRISM_LOG_LEVEL_TRACE: PrismLogLevel = 0;
pub const PRISM_LOG_LEVEL_DEBUG: PrismLogLevel = 1;
pub const PRISM_LOG_LEVEL_INFO: PrismLogLevel = 2;
pub const PRISM_LOG_LEVEL_WARN: PrismLogLevel = 3;
pub const PRISM_LOG_LEVEL_ERROR: PrismLogLevel = 4;
pub const PRISM_LOG_LEVEL_NONE: PrismLogLevel = 5;

pub const PRISM_BACKEND_IS_SUPPORTED_AT_RUNTIME: u64 = 1 << 0;
pub const PRISM_BACKEND_SUPPORTS_SPEAK: u64 = 1 << 2;
pub const PRISM_BACKEND_SUPPORTS_SPEAK_TO_MEMORY: u64 = 1 << 3;
pub const PRISM_BACKEND_SUPPORTS_BRAILLE: u64 = 1 << 4;
pub const PRISM_BACKEND_SUPPORTS_OUTPUT: u64 = 1 << 5;
pub const PRISM_BACKEND_SUPPORTS_IS_SPEAKING: u64 = 1 << 6;
pub const PRISM_BACKEND_SUPPORTS_STOP: u64 = 1 << 7;
pub const PRISM_BACKEND_SUPPORTS_PAUSE: u64 = 1 << 8;
pub const PRISM_BACKEND_SUPPORTS_RESUME: u64 = 1 << 9;
pub const PRISM_BACKEND_SUPPORTS_SET_VOLUME: u64 = 1 << 10;
pub const PRISM_BACKEND_SUPPORTS_GET_VOLUME: u64 = 1 << 11;
pub const PRISM_BACKEND_SUPPORTS_SET_RATE: u64 = 1 << 12;
pub const PRISM_BACKEND_SUPPORTS_GET_RATE: u64 = 1 << 13;
pub const PRISM_BACKEND_SUPPORTS_SET_PITCH: u64 = 1 << 14;
pub const PRISM_BACKEND_SUPPORTS_GET_PITCH: u64 = 1 << 15;
pub const PRISM_BACKEND_SUPPORTS_REFRESH_VOICES: u64 = 1 << 16;
pub const PRISM_BACKEND_SUPPORTS_COUNT_VOICES: u64 = 1 << 17;
pub const PRISM_BACKEND_SUPPORTS_GET_VOICE_NAME: u64 = 1 << 18;
pub const PRISM_BACKEND_SUPPORTS_GET_VOICE_LANGUAGE: u64 = 1 << 19;
pub const PRISM_BACKEND_SUPPORTS_GET_VOICE: u64 = 1 << 20;
pub const PRISM_BACKEND_SUPPORTS_SET_VOICE: u64 = 1 << 21;
pub const PRISM_BACKEND_SUPPORTS_GET_CHANNELS: u64 = 1 << 22;
pub const PRISM_BACKEND_SUPPORTS_GET_SAMPLE_RATE: u64 = 1 << 23;
pub const PRISM_BACKEND_SUPPORTS_GET_BIT_DEPTH: u64 = 1 << 24;
pub const PRISM_BACKEND_PERFORMS_SILENCE_TRIMMING_ON_SPEAK: u64 = 1 << 25;
pub const PRISM_BACKEND_PERFORMS_SILENCE_TRIMMING_ON_SPEAK_TO_MEMORY: u64 = 1 << 26;
pub const PRISM_BACKEND_SUPPORTS_SPEAK_SSML: u64 = 1 << 27;
pub const PRISM_BACKEND_SUPPORTS_SPEAK_TO_MEMORY_SSML: u64 = 1 << 28;
pub const PRISM_BACKEND_FEATURE_MAX_BIT: u64 = 1 << 63;

pub const PRISM_BACKEND_INVALID: PrismBackendId = 0;
pub const PRISM_BACKEND_SAPI: PrismBackendId = 0x1D6D_F724_22CE_EE66;
pub const PRISM_BACKEND_AV_SPEECH: PrismBackendId = 0x28E3_4295_7780_5C24;
pub const PRISM_BACKEND_VOICE_OVER: PrismBackendId = 0xCB48_9796_1A75_4BCB;
pub const PRISM_BACKEND_SPEECH_DISPATCHER: PrismBackendId = 0xE3D6_F895_D949_EBFE;
pub const PRISM_BACKEND_NVDA: PrismBackendId = 0x89CC_19C5_C4AC_1A56;
pub const PRISM_BACKEND_JAWS: PrismBackendId = 0xAC3D_60E9_BD84_B53E;
pub const PRISM_BACKEND_ONE_CORE: PrismBackendId = 0x6797_D32F_0D99_4CB4;
pub const PRISM_BACKEND_ORCA: PrismBackendId = 0x10AA_1FC0_5A17_F96C;
pub const PRISM_BACKEND_ANDROID_SCREEN_READER: PrismBackendId = 0xD199_C175_AEEC_494B;
pub const PRISM_BACKEND_ANDROID_TTS: PrismBackendId = 0xBC17_5831_BFE4_E5CC;
pub const PRISM_BACKEND_WEB_SPEECH: PrismBackendId = 0x3572_538D_44D4_4A8F;
pub const PRISM_BACKEND_UIA: PrismBackendId = 0x6238_F019_DB67_8F8E;
pub const PRISM_BACKEND_ZDSR: PrismBackendId = 0x3D93_C56C_9E7F_2A2E;
pub const PRISM_BACKEND_ZOOM_TEXT: PrismBackendId = 0xAE43_9D62_DC7B_1479;
pub const PRISM_BACKEND_BOY_PC_READER: PrismBackendId = 0x285A_BA1C_16F3_300F;
pub const PRISM_BACKEND_PC_TALKER: PrismBackendId = 0x344B_9519_62E3_B835;
pub const PRISM_BACKEND_SENSE_READER: PrismBackendId = 0xED47_6089_0B55_C2F2;
pub const PRISM_BACKEND_SYSTEM_ACCESS: PrismBackendId = 0x8380_F2A3_7B2C_3EB6;
pub const PRISM_BACKEND_WINDOW_EYES: PrismBackendId = 0x9120_D899_0878_5C13;
pub const PRISM_BACKEND_SPIEL: PrismBackendId = 0x478B_44F1_4AD3_D89C;

pub const PRISM_CONFIG_VERSION: u8 = 3;
pub const PRISM_PLUGIN_ABI_VERSION: u64 = 1;

pub type PrismAvailabilityCallback =
	Option<unsafe extern "C" fn(userdata: *mut c_void, backend: PrismBackendId, name: *const c_char, available: bool)>;

pub type PrismAudioCallback = Option<
	unsafe extern "C" fn(
		userdata: *mut c_void,
		samples: *const f32,
		sample_count: usize,
		channels: usize,
		sample_rate: usize,
	),
>;

pub type PrismLogCallback = Option<
	unsafe extern "C" fn(userdata: *mut c_void, level: PrismLogLevel, source: *const c_char, message: *const c_char),
>;

#[repr(C)]
#[derive(Clone, Copy)]
pub struct PrismConfig {
	pub version: u8,
	pub registry: *mut PrismRegistry,
	pub availability_callback: PrismAvailabilityCallback,
	pub availability_userdata: *mut c_void,
	pub availability_poll_interval_ms: u32,
	pub availability_debounce_samples: u32,
	pub availability_backoff_max_ms: u32,
	pub availability_auto_power_manage: bool,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct PrismBackendVTable {
	pub size: usize,
	pub create: Option<unsafe extern "C" fn(userdata: *mut c_void) -> *mut c_void>,
	pub destroy: Option<unsafe extern "C" fn(instance: *mut c_void)>,
	pub is_supported: Option<unsafe extern "C" fn(instance: *mut c_void) -> bool>,
	pub initialize: Option<unsafe extern "C" fn(instance: *mut c_void) -> PrismError>,
	pub speak: Option<unsafe extern "C" fn(instance: *mut c_void, text: *const c_char, interrupt: bool) -> PrismError>,
	pub speak_to_memory: Option<
		unsafe extern "C" fn(
			instance: *mut c_void,
			text: *const c_char,
			callback: PrismAudioCallback,
			callback_userdata: *mut c_void,
		) -> PrismError,
	>,
	pub braille: Option<unsafe extern "C" fn(instance: *mut c_void, text: *const c_char) -> PrismError>,
	pub output: Option<unsafe extern "C" fn(instance: *mut c_void, text: *const c_char, interrupt: bool) -> PrismError>,
	pub stop: Option<unsafe extern "C" fn(instance: *mut c_void) -> PrismError>,
	pub pause: Option<unsafe extern "C" fn(instance: *mut c_void) -> PrismError>,
	pub resume: Option<unsafe extern "C" fn(instance: *mut c_void) -> PrismError>,
	pub is_speaking: Option<unsafe extern "C" fn(instance: *mut c_void, out_speaking: *mut bool) -> PrismError>,
	pub set_volume: Option<unsafe extern "C" fn(instance: *mut c_void, volume: f32) -> PrismError>,
	pub get_volume: Option<unsafe extern "C" fn(instance: *mut c_void, out_volume: *mut f32) -> PrismError>,
	pub set_rate: Option<unsafe extern "C" fn(instance: *mut c_void, rate: f32) -> PrismError>,
	pub get_rate: Option<unsafe extern "C" fn(instance: *mut c_void, out_rate: *mut f32) -> PrismError>,
	pub set_pitch: Option<unsafe extern "C" fn(instance: *mut c_void, pitch: f32) -> PrismError>,
	pub get_pitch: Option<unsafe extern "C" fn(instance: *mut c_void, out_pitch: *mut f32) -> PrismError>,
	pub refresh_voices: Option<unsafe extern "C" fn(instance: *mut c_void) -> PrismError>,
	pub count_voices: Option<unsafe extern "C" fn(instance: *mut c_void, out_count: *mut usize) -> PrismError>,
	pub get_voice_name: Option<
		unsafe extern "C" fn(instance: *mut c_void, voice_id: usize, out_name: *mut *const c_char) -> PrismError,
	>,
	pub get_voice_language: Option<
		unsafe extern "C" fn(instance: *mut c_void, voice_id: usize, out_language: *mut *const c_char) -> PrismError,
	>,
	pub set_voice: Option<unsafe extern "C" fn(instance: *mut c_void, voice_id: usize) -> PrismError>,
	pub get_voice: Option<unsafe extern "C" fn(instance: *mut c_void, out_voice_id: *mut usize) -> PrismError>,
	pub get_channels: Option<unsafe extern "C" fn(instance: *mut c_void, out_channels: *mut usize) -> PrismError>,
	pub get_sample_rate: Option<unsafe extern "C" fn(instance: *mut c_void, out_sample_rate: *mut usize) -> PrismError>,
	pub get_bit_depth: Option<unsafe extern "C" fn(instance: *mut c_void, out_bit_depth: *mut usize) -> PrismError>,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct PrismLogHandler {
	pub fn_: PrismLogCallback,
	pub userdata: *mut c_void,
}

#[repr(C)]
pub struct PrismPluginServices {
	pub struct_size: u32,
	pub reserved: u32,
	pub log: Option<unsafe extern "C" fn(self_: *const Self, level: PrismLogLevel, message: *const c_char)>,
}

#[repr(C)]
pub struct PrismPluginInstanceContext {
	pub struct_size: u32,
	pub reserved: u32,
	pub services: *const PrismPluginServices,
	pub userdata: *mut c_void,
}

#[repr(C)]
pub struct PrismPluginHost {
	pub abi_version: u64,
	pub struct_size: u32,
	pub reserved: u32,
	pub log: Option<unsafe extern "C" fn(self_: *const Self, level: PrismLogLevel, message: *const c_char)>,
}

#[repr(C)]
pub struct PrismPluginBackend {
	pub abi_version: u64,
	pub struct_size: u32,
	pub reserved: u32,
	pub name: *const c_char,
	pub priority: c_int,
	pub features: u64,
	pub vtable: *const PrismBackendVTable,
	pub userdata: *mut c_void,
	pub plugin_version: u64,
}

pub type PrismPluginQueryFn =
	Option<unsafe extern "C" fn(host: *const PrismPluginHost, index: usize) -> *const PrismPluginBackend>;

unsafe extern "C" {
	pub fn prism_config_init() -> PrismConfig;
	pub fn prism_init(cfg: *mut PrismConfig) -> *mut PrismContext;
	pub fn prism_shutdown(ctx: *mut PrismContext);
	pub fn prism_availability_poll_pause(ctx: *mut PrismContext);
	pub fn prism_availability_poll_resume(ctx: *mut PrismContext);
	pub fn prism_availability_auto_power_supported() -> bool;
	pub fn prism_registry_count(ctx: *mut PrismContext) -> usize;
	pub fn prism_registry_id_at(ctx: *mut PrismContext, index: usize) -> PrismBackendId;
	pub fn prism_registry_id(ctx: *mut PrismContext, name: *const c_char) -> PrismBackendId;
	pub fn prism_registry_name(ctx: *mut PrismContext, id: PrismBackendId) -> *const c_char;
	pub fn prism_registry_priority(ctx: *mut PrismContext, id: PrismBackendId) -> c_int;
	pub fn prism_registry_exists(ctx: *mut PrismContext, id: PrismBackendId) -> bool;
	pub fn prism_registry_get(ctx: *mut PrismContext, id: PrismBackendId) -> *mut PrismBackend;
	pub fn prism_registry_create(ctx: *mut PrismContext, id: PrismBackendId) -> *mut PrismBackend;
	pub fn prism_registry_create_best(ctx: *mut PrismContext) -> *mut PrismBackend;
	pub fn prism_registry_acquire(ctx: *mut PrismContext, id: PrismBackendId) -> *mut PrismBackend;
	pub fn prism_registry_acquire_best(ctx: *mut PrismContext) -> *mut PrismBackend;
	pub fn prism_backend_free(backend: *mut PrismBackend);
	pub fn prism_backend_name(backend: *mut PrismBackend) -> *const c_char;
	pub fn prism_backend_get_features(backend: *mut PrismBackend) -> u64;
	pub fn prism_backend_initialize(backend: *mut PrismBackend) -> PrismError;
	pub fn prism_backend_speak(backend: *mut PrismBackend, text: *const c_char, interrupt: bool) -> PrismError;
	pub fn prism_backend_speak_to_memory(
		backend: *mut PrismBackend,
		text: *const c_char,
		callback: PrismAudioCallback,
		userdata: *mut c_void,
	) -> PrismError;
	pub fn prism_backend_braille(backend: *mut PrismBackend, text: *const c_char) -> PrismError;
	pub fn prism_backend_output(backend: *mut PrismBackend, text: *const c_char, interrupt: bool) -> PrismError;
	pub fn prism_backend_stop(backend: *mut PrismBackend) -> PrismError;
	pub fn prism_backend_pause(backend: *mut PrismBackend) -> PrismError;
	pub fn prism_backend_resume(backend: *mut PrismBackend) -> PrismError;
	pub fn prism_backend_is_speaking(backend: *mut PrismBackend, out_speaking: *mut bool) -> PrismError;
	pub fn prism_backend_set_volume(backend: *mut PrismBackend, volume: f32) -> PrismError;
	pub fn prism_backend_get_volume(backend: *mut PrismBackend, out_volume: *mut f32) -> PrismError;
	pub fn prism_backend_set_rate(backend: *mut PrismBackend, rate: f32) -> PrismError;
	pub fn prism_backend_get_rate(backend: *mut PrismBackend, out_rate: *mut f32) -> PrismError;
	pub fn prism_backend_set_pitch(backend: *mut PrismBackend, pitch: f32) -> PrismError;
	pub fn prism_backend_get_pitch(backend: *mut PrismBackend, out_pitch: *mut f32) -> PrismError;
	pub fn prism_backend_refresh_voices(backend: *mut PrismBackend) -> PrismError;
	pub fn prism_backend_count_voices(backend: *mut PrismBackend, out_count: *mut usize) -> PrismError;
	pub fn prism_backend_get_voice_name(
		backend: *mut PrismBackend,
		voice_id: usize,
		out_name: *mut *const c_char,
	) -> PrismError;
	pub fn prism_backend_get_voice_language(
		backend: *mut PrismBackend,
		voice_id: usize,
		out_language: *mut *const c_char,
	) -> PrismError;
	pub fn prism_backend_set_voice(backend: *mut PrismBackend, voice_id: usize) -> PrismError;
	pub fn prism_backend_get_voice(backend: *mut PrismBackend, out_voice_id: *mut usize) -> PrismError;
	pub fn prism_backend_get_channels(backend: *mut PrismBackend, out_channels: *mut usize) -> PrismError;
	pub fn prism_backend_get_sample_rate(backend: *mut PrismBackend, out_sample_rate: *mut usize) -> PrismError;
	pub fn prism_backend_get_bit_depth(backend: *mut PrismBackend, out_bit_depth: *mut usize) -> PrismError;
	pub fn prism_error_string(error: PrismError) -> *const c_char;
	pub fn prism_registry_builder_new() -> *mut PrismRegistryBuilder;
	pub fn prism_registry_builder_add_backend(
		builder: *mut PrismRegistryBuilder,
		name: *const c_char,
		priority: c_int,
		features: u64,
		vtable: *const PrismBackendVTable,
		userdata: *mut c_void,
		userdata_free: Option<unsafe extern "C" fn(*mut c_void)>,
		out_id: *mut PrismBackendId,
	) -> PrismError;
	pub fn prism_registry_builder_add_library(
		builder: *mut PrismRegistryBuilder,
		path: *const c_char,
		priority_override: c_int,
		out_count: *mut usize,
	) -> PrismError;
	pub fn prism_registry_freeze(builder: *mut PrismRegistryBuilder) -> *mut PrismRegistry;
	pub fn prism_registry_builder_free(builder: *mut PrismRegistryBuilder);
	pub fn prism_registry_retain(registry: *mut PrismRegistry) -> *mut PrismRegistry;
	pub fn prism_registry_release(registry: *mut PrismRegistry);
	pub fn prism_set_log_handler(handler: PrismLogHandler) -> PrismLogHandler;
	pub fn prism_set_log_level(level: PrismLogLevel) -> PrismLogLevel;
	pub fn prism_log(level: PrismLogLevel, source: *const c_char, message: *const c_char);
	pub fn prism_log_flush();
	pub fn prism_log_shutdown();
	pub fn prism_version() -> u32;
	pub fn prism_version_string() -> *const c_char;
}
