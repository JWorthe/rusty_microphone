pub mod transforms;

#[cfg(not(any(target_arch="wasm32",target_os="emscripten")))]
extern crate gtk;
#[cfg(not(any(target_arch="wasm32",target_os="emscripten")))]
extern crate cairo;
#[cfg(not(any(target_arch="wasm32",target_os="emscripten")))]
pub mod gui;

#[cfg(not(any(target_arch="wasm32",target_os="emscripten")))]
extern crate portaudio;
#[cfg(not(any(target_arch="wasm32",target_os="emscripten")))]
pub mod audio;

#[cfg(any(target_arch="wasm32",target_os="emscripten"))]
pub mod emscripten_api;
