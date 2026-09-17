//! Passes prism-sys's delay-load list through to whoever depends on prismer.
//!
//! Cargo hands a `links` crate's metadata to its direct dependents only, so
//! without this an application depending on prismer could not see what
//! prism-sys published.

fn main() {
	println!("cargo:rerun-if-changed=build.rs");
	if let Ok(dlls) = std::env::var("DEP_PRISM_DELAY_LOAD_DLLS") {
		println!("cargo:delay_load_dlls={dlls}");
	}
}
