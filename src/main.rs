extern crate rusty_microphone;

#[cfg(not(target_os = "emscripten"))]
fn main() {
    use rusty_microphone::*;

    let gui_result = gui::start_gui();
    if gui_result.is_err() {
        println!("Failed to initialize");
        return;
    }
}

#[cfg(target_os = "emscripten")]
fn main() {
    println!("Hello Emscripten");
}
