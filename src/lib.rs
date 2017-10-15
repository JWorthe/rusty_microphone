pub mod transforms;

#[cfg(not(target_os = "emscripten"))]
extern crate gtk;
#[cfg(not(target_os = "emscripten"))]
extern crate cairo;
#[cfg(not(target_os = "emscripten"))]
pub mod gui;

#[cfg(not(target_os = "emscripten"))]
extern crate portaudio;
#[cfg(not(target_os = "emscripten"))]
pub mod audio;

#[cfg(target_os = "emscripten")]
pub mod emscripten_api;
