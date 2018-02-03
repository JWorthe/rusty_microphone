pub mod model;
pub mod signal;
pub mod correlation;
pub mod pitch;

#[cfg(not(target_arch = "wasm32"))]
extern crate gtk;
#[cfg(not(target_arch = "wasm32"))]
extern crate cairo;
#[cfg(not(target_arch = "wasm32"))]
pub mod gui;

#[cfg(not(target_arch = "wasm32"))]
extern crate portaudio;
#[cfg(not(target_arch = "wasm32"))]
pub mod audio;

#[cfg(target_arch = "wasm32")]
pub mod wasm_api;

