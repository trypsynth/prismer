# Prismer

Rust bindings to [prism](https://github.com/ethindp/prism), the platform-agnostic reader interface for speech and messages. Prism unifies screen readers and TTS engines (SAPI, NVDA, JAWS, VoiceOver, AVSpeech, speech-dispatcher, Orca, Android TTS, WebSpeech, and more) behind one API, so this crate gives your Rust application speech, braille, and screen reader output on every major platform.

## Crates

- `prism-sys`: raw FFI bindings to the prism C API, `no_std`.
- `prismer`: safe idiomatic wrapper.

## Quick start

```rust
let prism = prismer::Prism::new()?;
let backend = prism.create_best()?;
backend.speak("Hello from Rust!", false)?;
```

See `crates/prismer/examples/speak.rs` for a fuller example, including feature detection and waiting for speech to finish.

## Picking a backend

There are four ways to get a backend:

| Method | Backend state | Initialized on return |
| --- | --- | --- |
| `create_best()` | private to you | yes |
| `create(id)` | private to you | no, call `initialize()` |
| `acquire_best()` | shared with other callers | yes |
| `acquire(id)` | shared with other callers | maybe, call `initialize()` and treat `AlreadyInitialized` as success |

Use `create_best()` unless you have a specific reason not to. The `acquire` family returns a cached instance, so a voice, rate, or pitch set through one handle is visible through every other handle to that backend. Reach for it only when sharing that state is what you want.

## Building

Prism is vendored as a git submodule and built automatically by `prism-sys`'s build script, so all you need is a C++23 toolchain and CMake 3.24+:

```sh
git clone --recurse-submodules https://github.com/trypsynth/prismer
cd prismer
cargo build
```

If you cloned without `--recurse-submodules`, run `git submodule update --init` first.

To link against a prebuilt prism instead (skipping the CMake build), set `PRISM_LIB_DIR` to the directory containing the library:

```sh
set PRISM_LIB_DIR=path\to\prism\build
cargo build
```

By default prism is built and linked as a shared library; note that your application must be able to find it (next to the executable on Windows, or on the loader path elsewhere) at runtime. Enable the `static` feature to build and link it statically instead — the build script links the C++ runtime for you, though some backends may need additional system libraries.

## Status

Early draft. Covered: context init and configuration (including availability callbacks), registry queries, backend creation and acquisition, speech, braille, output, playback control, volume, rate, pitch, voice enumeration and selection, audio format queries, and speak-to-memory with a closure callback. Not yet wrapped: the registry builder (custom backends and plugin libraries) and the logging API, though both are present in `prism-sys`.

## License

MIT. Prism itself is MPL-2.0.
