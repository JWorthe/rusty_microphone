extern crate musician_training;

use musician_training::*;

fn main() {
    let gui_result = gui::start_gui();
    if gui_result.is_err() {
        println!("Failed to initialize");
        return;
    }
}
