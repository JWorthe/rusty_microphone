extern crate rusty_microphone;

use rusty_microphone::*;

fn main() {
    let gui_result = gui::start_gui();
    if gui_result.is_err() {
        println!("Failed to initialize");
        return;
    }
}
