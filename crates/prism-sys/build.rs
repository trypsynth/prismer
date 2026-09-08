//! Builds and links the vendored prism library.

// cmake::Config is a builder whose setters return `&mut Self` for chaining;
// discarding that borrow is the normal way to call them.
#![allow(unused_results)]

use std::{env, path::PathBuf};

use cmake::Config;

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
	}
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
	cfg.define("BUILD_SHARED_LIBS", if static_link { "OFF" } else { "ON" });
	if env::var("CARGO_CFG_TARGET_ENV").as_deref() == Ok("msvc") {
		// prism defaults to the static CRT, but the objects must use the same
		// CRT rustc links: the non-debug one, static or dynamic per the
		// crt-static target feature.
		let crt_static =
			env::var("CARGO_CFG_TARGET_FEATURE").is_ok_and(|features| features.split(',').any(|f| f == "crt-static"));
		cfg.define("CMAKE_MSVC_RUNTIME_LIBRARY", if crt_static { "MultiThreaded" } else { "MultiThreadedDLL" });
	}
	let dst = cfg.build();
	println!("cargo:rustc-link-search=native={}", dst.join("lib").display());
	println!("cargo:rustc-link-search=native={}", dst.join("bin").display());
}

fn link_cpp_runtime() {
	let os = env::var("CARGO_CFG_TARGET_OS").unwrap_or_default();
	let target_env = env::var("CARGO_CFG_TARGET_ENV").unwrap_or_default();
	match (os.as_str(), target_env.as_str()) {
		// MSVC objects carry /DEFAULTLIB directives for their C++ runtime.
		(_, "msvc") => {}
		("macos" | "ios" | "tvos" | "watchos" | "visionos", _) => println!("cargo:rustc-link-lib=c++"),
		("android", _) => println!("cargo:rustc-link-lib=c++_shared"),
		_ => println!("cargo:rustc-link-lib=stdc++"),
	}
}
