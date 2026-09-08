//! Registers a backend written in Rust, then speaks through it.

use std::error::Error;

use prismer::{CustomBackend, Features, Prism, RegistryBuilder};

/// A backend that prints instead of speaking, and remembers what it was given.
struct Printer {
	spoken: Vec<String>,
}

impl CustomBackend for Printer {
	fn speak(&mut self, text: &str, interrupt: bool) -> prismer::Result<()> {
		println!("speak (interrupt: {interrupt}): {text}");
		self.spoken.push(text.to_owned());
		Ok(())
	}

	fn stop(&mut self) -> prismer::Result<()> {
		println!("stop, after {} utterances", self.spoken.len());
		Ok(())
	}
}

fn main() -> Result<(), Box<dyn Error>> {
	let mut builder = RegistryBuilder::new()?;
	// The declared features and the implemented methods have to agree, so this
	// backend declares exactly the two it overrides.
	let id = builder.add_backend("Example Printer", 10, Features::SPEAK | Features::STOP, || {
		Some(Printer { spoken: Vec::new() })
	})?;
	let registry = builder.freeze()?;
	let prism = Prism::builder().registry(&registry).build()?;
	println!("{} backends registered, custom backend id {:#x}", prism.backend_count(), id.0);
	let backend = prism.create(id)?;
	backend.initialize()?;
	println!("using backend: {}", backend.name());
	backend.speak("Hello from a custom backend", true)?;
	backend.speak("Written in Rust", false)?;
	backend.stop()?;
	Ok(())
}
