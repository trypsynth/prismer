use std::{error::Error, thread, time::Duration};

use prismer::{Features, Prism};

fn main() -> Result<(), Box<dyn Error>> {
	let prism = Prism::new()?;
	println!("prism {} with {} registered backends", prismer::version_string(), prism.backend_count());
	let backend = prism.acquire_best()?;
	println!("using backend: {}", backend.name());
	backend.speak("Hello from Rust!", false)?;
	if backend.supports(Features::IS_SPEAKING) {
		while backend.is_speaking()? {
			thread::sleep(Duration::from_millis(50));
		}
	} else {
		thread::sleep(Duration::from_secs(3));
	}
	Ok(())
}
