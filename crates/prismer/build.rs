//! Passes prism-sys's delay-load list through to whoever depends on prismer.
//!
//! Cargo hands a `links` crate's metadata to its direct dependents only, so
//! without this an application depending on prismer could not see what
//! prism-sys published. prismer's own tests and examples are applications
//! too, and get the same flags an application's build script would set.

fn main() {
	println!("cargo:rerun-if-changed=build.rs");
	if let Ok(dlls) = std::env::var("DEP_PRISM_DELAY_LOAD_DLLS") {
		println!("cargo:delay_load_dlls={dlls}");
		for dll in dlls.split(';').filter(|dll| !dll.is_empty()) {
			println!("cargo:rustc-link-arg-tests=/DELAYLOAD:{dll}");
			println!("cargo:rustc-link-arg-examples=/DELAYLOAD:{dll}");
		}
	}
}
