extern crate rusty_microphone;

#[cfg(not(target_arch = "wasm32"))]
fn main() {
    use rusty_microphone::*;

    let gui_result = gui::start_gui();
    if gui_result.is_err() {
        println!("Failed to initialize");
        return;
    }
}

#[cfg(target_arch = "wasm32")]
fn main() {
    println!("Hello Wasm");
}
