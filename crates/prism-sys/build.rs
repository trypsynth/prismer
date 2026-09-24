//! Builds and links the vendored prism library.

use std::{env, path::PathBuf};

use cmake::Config;

/// The Windows system libraries prism links itself, in `PrismPlatformWindows.cmake`.
///
/// A static prism leaves these to the final link too, and nothing else names them there. An
/// application only linked them by luck, when another dependency happened to pull them in;
/// otherwise it failed with unresolved UI Automation, RPC and Windows Runtime symbols.
const WINDOWS_SYSTEM_LIBS: &[&str] = &["ole32", "onecore", "runtimeobject", "uiautomationcore", "rpcrt4", "powrprof"];

/// The frameworks prism links itself on macOS, in `PrismPlatformApple.cmake`.
///
/// `IOKit` and `CoreFoundation` are for the power notifier, which is always linked, so a static
/// binary fails to link without them even before any backend is used.
const MACOS_FRAMEWORKS: &[&str] = &["Foundation", "AVFoundation", "AppKit", "IOKit", "CoreFoundation"];

/// The frameworks prism links itself on iOS, in `PrismPlatformApple.cmake`.
const IOS_FRAMEWORKS: &[&str] = &["Foundation", "AVFoundation", "UIKit"];

fn main() {
	println!("cargo:rerun-if-env-changed=PRISM_LIB_DIR");
	let static_link = env::var("CARGO_FEATURE_STATIC").is_ok();
	if let Ok(dir) = env::var("PRISM_LIB_DIR") {
		println!("cargo:rustc-link-search=native={dir}");
	} else {
		build_vendored(static_link);
	}
	let kind = if static_link { "static" } else { "dylib" };
	println!("cargo:rustc-link-lib={kind}=prism");
	if static_link {
		link_cpp_runtime();
		link_windows_import_libs();
		link_apple_frameworks();
	}
}

/// Links the frameworks the Apple backends and power notifier use.
///
/// A shared prism links these itself; a static one leaves them to the final link.
fn link_apple_frameworks() {
	let frameworks = match env::var("CARGO_CFG_TARGET_OS").as_deref() {
		Ok("macos") => MACOS_FRAMEWORKS,
		Ok("ios") => IOS_FRAMEWORKS,
		_ => return,
	};
	for framework in frameworks {
		println!("cargo:rustc-link-lib=framework={framework}");
	}
}

/// Links what the Windows backends import, and delay loads every DLL behind
/// them.
///
/// A shared prism links these itself. A static one hands the job to whoever
/// is doing the final link, and the delay loading is the part that matters:
/// without it the screen reader DLLs become hard dependencies, and the
/// program will not start on a machine that does not have all of them.
///
/// The list is written by prism's own install step, so it always matches the
/// backends that were actually built.
fn link_windows_import_libs() {
	if env::var("CARGO_CFG_TARGET_OS").as_deref() != Ok("windows") {
		return;
	}
	for lib in WINDOWS_SYSTEM_LIBS {
		println!("cargo:rustc-link-lib=dylib={lib}");
	}
	let Some(lib_dir) = env::var_os("OUT_DIR").map(PathBuf::from).map(|out| out.join("lib")) else { return };
	let manifest = lib_dir.join("prism-static-windows.txt");
	let Ok(text) = std::fs::read_to_string(&manifest) else {
		println!("cargo:warning=prism-static-windows.txt is missing; the Windows backends will not link");
		return;
	};
	let mut dlls = Vec::new();
	for line in text.lines() {
		let mut parts = line.split_whitespace();
		let (Some(lib), Some(dll)) = (parts.next(), parts.next()) else { continue };
		println!("cargo:rustc-link-lib=dylib={lib}");
		dlls.push(dll);
	}
	// Cargo only applies rustc-link-arg to the crate that emits it, so the
	// delay-load flags cannot be set from here: they have to be on the final
	// link. This publishes the list instead, and a `links = "prism"` crate's
	// metadata reaches every direct dependent as DEP_PRISM_DELAY_LOAD_DLLS.
	println!("cargo:delay_load_dlls={}", dlls.join(";"));
}

fn build_vendored(static_link: bool) {
	let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").expect("cargo sets CARGO_MANIFEST_DIR"));
	let source = manifest_dir.join("prism");
	assert!(
		source.join("CMakeLists.txt").exists(),
		"vendored prism source not found at {}; run `git submodule update --init` \
		 or set PRISM_LIB_DIR to a directory containing a prebuilt prism library",
		source.display()
	);
	println!("cargo:rerun-if-changed=prism/CMakeLists.txt");
	println!("cargo:rerun-if-changed=prism/cmake");
	println!("cargo:rerun-if-changed=prism/source");
	println!("cargo:rerun-if-changed=prism/include");
	let mut cfg = Config::new(&source);
	let _ = cfg.define("BUILD_SHARED_LIBS", if static_link { "OFF" } else { "ON" });
	if env::var("CARGO_CFG_TARGET_ENV").as_deref() == Ok("msvc") {
		// prism defaults to the static CRT, but the objects must use the same
		// CRT rustc links: the non-debug one, static or dynamic per the
		// crt-static target feature.
		let crt_static =
			env::var("CARGO_CFG_TARGET_FEATURE").is_ok_and(|features| features.split(',').any(|f| f == "crt-static"));
		let _ = cfg.define("CMAKE_MSVC_RUNTIME_LIBRARY", if crt_static { "MultiThreaded" } else { "MultiThreadedDLL" });
	}
	let dst = cfg.build();
	println!("cargo:rustc-link-search=native={}", dst.join("lib").display());
	println!("cargo:rustc-link-search=native={}", dst.join("bin").display());
}

fn link_cpp_runtime() {
	let os = env::var("CARGO_CFG_TARGET_OS").unwrap_or_default();
	let target_env = env::var("CARGO_CFG_TARGET_ENV").unwrap_or_default();
	match (os.as_str(), target_env.as_str()) {
		// MSVC objects carry /DEFAULTLIB directives for their C++ runtime,
		// but not for the delay-load helper: prism_shutdown calls
		// FUnloadDelayLoadedDLL2, which lives in delayimp. Linking prism as a
		// DLL pulled that in on its own; linking it statically leaves it to
		// whoever is doing the linking, and without it the consumer fails
		// with "unresolved external symbol __FUnloadDelayLoadedDLL2".
		(_, "msvc") => println!("cargo:rustc-link-lib=delayimp"),
		("macos" | "ios" | "tvos" | "watchos" | "visionos", _) => println!("cargo:rustc-link-lib=c++"),
		("android", _) => println!("cargo:rustc-link-lib=c++_shared"),
		_ => println!("cargo:rustc-link-lib=stdc++"),
	}
}
