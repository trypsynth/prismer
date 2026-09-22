//! Checks that prism loads and registers the backends this platform should
//! have, which catches a static link that dropped them.

use prismer::Prism;

#[cfg(windows)]
const EXPECTED: &[&str] = &["SAPI", "NVDA", "JAWS", "OneCore", "ZoomText"];
#[cfg(target_os = "macos")]
const EXPECTED: &[&str] = &["AVSpeech", "VoiceOver"];
#[cfg(target_os = "linux")]
const EXPECTED: &[&str] = &["Speech Dispatcher", "Orca"];

#[test]
fn registers_the_platform_backends() {
	let prism = Prism::new().expect("prism starts");
	let names: Vec<String> = prism.backend_ids().into_iter().filter_map(|id| prism.backend_name(id)).collect();
	for expected in EXPECTED {
		assert!(names.iter().any(|name| name == expected), "{expected} is not registered, only {names:?}");
	}
}

/// Only these have a voice on a machine with nothing installed.
#[cfg(any(windows, target_os = "macos"))]
#[test]
fn speaks_through_the_best_backend() {
	let prism = Prism::new().expect("prism starts");
	let backend = prism.create_best().expect("a backend starts");
	backend.speak("Hello from prismer.", true).expect("speech starts");
}
