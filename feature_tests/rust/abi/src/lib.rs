//! The proof that the generated crate's ABI transcription still matches the runtime.
//!
//! The generated crate carries its own `#[repr(C)]` ABI types, so that a Rust
//! consumer links nothing but the provider cdylib and no provider-side crate can
//! satisfy a symbol the consumer's code calls. The hazard that buys is drift: a
//! transcription can move a field and still compile, because nothing on either side
//! names the other. The tests in this crate are the check, and they are the reason
//! the transcription is admissible at all.
//!
//! - `tests/layout.rs` compares the two definitions: size, alignment, and where the
//!   data actually lands in a value built by the runtime's own constructors.
//! - `tests/drop_semantics.rs` pins the conversions that move a payload out of the
//!   ABI container, the place where a transcription that "just works" for integers
//!   can still free an allocation twice.
